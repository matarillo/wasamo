---
title: M3-Phase 1 / T3 step-end retrospective
status: recorded
created: 2026-05-19
scope: step-end
task: T3 — wasamoc checker, state-type table and bool type-checking
---

# M3-Phase 1 / T3 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-1-progress.md` の **T3** ("`wasamoc`
checker: state-type table and bool type-checking") の step-end
retrospective。T3 が discharge する DD は DD-M3-P1-010 (accept/reject
table 全体)。

対象コミット:

- `710eea8 feat(wasamoc): type-check state defaults against declared type (M3-Phase 1 T3, part 1)`
- `3cbe257 feat(wasamoc): type-check ` + "`bind`" + ` LHS against widget-property catalog (M3-Phase 1 T3, part 2)`

これは step-end の gate であり、phase-end retrospective ではない。

## Current Judgment

2026-05-19 時点で T3 step-end 基準は **達成済み**。

- State default 型チェック (commit 1):
  - 新規ヘルパ `expr_static_type(expr, ns) -> Option<TypeName>`、
    `types_compatible`、`type_name_display`。
  - `check_state_defaults` パスを `check` 本体に追加。declared
    `TypeName` と default の static type が不一致なら error
    diagnostic。Float source 側は `check_expr_type` 既存 reject 経路に
    任せて重複報告を抑止。
- Bind LHS 型チェック (commit 2):
  - 新規 `widget_prop_type(widget, prop) -> Option<TypeName>` —
    Phase 1 catalog は `Text.text: string`, `Button.text: string`,
    `Button.enabled: bool` の 3 行のみ。それ以外は `None` を返す
    soft catalog。
  - `check_members` を internal `check_members_inner` に分離し、
    `enclosing_widget: Option<&str>` を carry。`WidgetDecl` 再帰で
    `Some(type_name)` を渡す。
  - `check_property_bind_target` が enclosing widget + catalog entry
    + source static type が **全部揃ったときだけ** 型一致を検査。
    どれか欠ければ pass-through (M2 の `theme: system` /
    `Button { style: accent }` / `Text { font: title }` /
    component-level `title: "Counter"` / `backdrop: mica` を
    そのまま受理する)。
- 新規テスト 14 件 (state-default 5 + bind 9) を追加し DD-M3-P1-010
  table の全行を carbon-copy。
- `cargo build --workspace` / `cargo test --workspace` どちらも local
  で green。`wasamoc` 単体は 91 件 passed (T2 完了時 77 件から +14)。

T3 の blocker は残っていない。

## Main Learning

中心的な学びは「`wasamoc` 側の widget-property catalog を **soft** に
保ったことで、新しい型ルールを追加しても M2 で書かれた既存 `.ui` を
壊さずに済んだ」という点。

- "soft" の運用ルールは「enclosing widget context が無い」「catalog
  entry が無い」「source static type が `None` (= ns に無い ident)」
  のいずれかなら no-op。三つ揃ったときだけ厳密に型一致を見る。
- これにより `Button { style: accent }` の `accent` は ns に無い
  ident なので `expr_static_type` が `None` を返し、catalog 側に
  `Button.style` を載せていてもパスする。`theme: system` も同様。
- 副次的効果: catalog は T6 で wasamo-runtime 側の
  `resolve_prop_key(widget, prop) -> (PropertyKey, IrType)` と平行に
  伸びる予定だが、その整合は wasamoc 単独でテストできる (runtime と
  別 crate)。両側が独立に bool / string / i32 を判定して同じ結論に
  辿り着く構造が DD-M3-P1-009 の意図に近い。

副次的な学びとして、DD-M3-P1-010 の reject table に出てくる
abstract な `bind label: <…>` (target `String`) は、catalog 上の
具体的なエントリ `Text.text: string` に読み替えて実装した。DD
原文は「string-typed widget property に bool を bind する」シナリオ
を示すための例示なので、catalog に実在する `Text.text` を string
代表として使う方が test の意味も IR 経路も実装と一致する。
progress file の T3 ノートにこの読み替えを明記した。

T3 の commit 分割の判断: state-default と bind-LHS は次のように
独立性が高いので 2 commit に分けた:

- state-default は `check` 関数本体に新パス (`check_state_defaults`)
  を追加するだけで `check_members` の構造には触らない。
- bind-LHS は `check_members` の signature を変える (`enclosing_widget`
  を carry) ため、中間状態が build green を保てる範囲で独立。

両 commit とも独立に `cargo test --workspace` green で着地。
CLAUDE.md Commit rules の「1 task = 1 commit が default、build/tests
を保てる範囲で分割可」の幅に収まっている。

## Checklist

1. **本作業の主要な学び:** あり。
   - soft catalog 運用ルール (enclosing widget / catalog entry /
     source type の 3 つが揃った時だけ厳密化) は M2 互換を保ったまま
     新ルールを増やす一般パターンとして残る。後続 T6 (runtime 側
     catalog 拡張) でも同じ判断軸が効きそう。
   - DD の abstract 例 (`bind label: …`) を具体エントリ (`Text.text`)
     に読み替える際の判断は progress file の T3 Notes に明記する
     形で reviewable に残した。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:** **なし**
   - T3 の対象は `wasamoc::check` のみ。型診断のメッセージ仕様や
     widget property catalog の DSL-spec 化 (`Button.enabled: bool`
     を §4 widget catalog に書く等) は A11 = T10 の責任範囲で、
     T3 では行わない。

3. **ローカル clean rebuild:** **green**
   - `cargo build --workspace`: green
   - `cargo test --workspace`: 全件 green (`wasamoc` 91 件、
     `wasamo-runtime` 123 件、他)
   - GitHub Actions 上の clean rebuild は phase-end gate で確認する
     (T12)。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - DD-M3-P1-010 (Option A 採用、`wasamoc` で full accept/reject)
     は ADR Accepted 済み。catalog の具体エントリ (Text.text /
     Button.text / Button.enabled) は DD-M3-P1-005 (`Button.enabled`
     を Phase 1 evidence にする) と DD-M3-P1-009 (catalog row が
     `(key, IrType)` を持つ) から機械的に導出されるもので、PO 判断
     を要する設計呼び出しは無い。
   - "soft catalog" 方針 (catalog 外の prop は pass-through) は DD
     が明示していない実装内部の選択だが、`bind enabled: 1` と
     `theme: system` を両立させるためには必須で、外から見える振る
     舞いは「DD table に書かれた組み合わせは全部 catch、それ以外は
     従来通り」となり PO 期待値と一致するため設計呼び出しに昇格
     しない。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 編集対象は `wasamoc/src/check.rs` のみ。`check_members` を
     `check_members_inner` に分けたのは `enclosing_widget` パラメタを
     carry するためで、bind 型検査と直接寄与。`expr_static_type` /
     `types_compatible` / `type_name_display` の 3 ヘルパも state
     default と bind の両方で共有されるため scope 内。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - 既存 DD-M3-P1-001..010 で T3 範囲はすべてカバーされている。
     soft catalog の運用ルールは DD-M3-P1-009 (runtime 側) と
     DD-M3-P1-010 (wasamoc 側) の implementation noise であり、DD を
     立てる粒度ではない。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR は 2026-05-19 時点で全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A9 / A11 / A12 の文言は変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **意図的な暫定処理あり、`dead_code` 警告なし**
   - `lower.rs::lower_expr` の `Expr::Ident { name }` arm は依然
     `IrLiteral::Ident(name)` を返す。T3 は **検査** のみ提供し、
     ident → typed `*PropRead` への振り分け (DD-M3-P1-010 が
     "identifier resolution at lowering time" として load-bearing で
     指している部分) は T4 が consume する。T3 で構築した
     `Namespace` (= `HashMap<String, TypeName>`) は T4 の typed
     lowering pass にそのまま渡せる形になっている。
   - `lower_string_parts` の interp 部 (`HandlerExpr::PropRead` /
     `HandlerExpr::StrPropRead`) は M2 由来の i32/string 2 分岐の
     ままで Bool ident を `PropRead` (i32-implicit) に落としている。
     これも T4 で `BoolPropRead` arm を入れる範囲。
   - handler 文の RHS 型検査 (e.g. `ready = 5` を bool target に
     対して reject) は DD-M3-P1-010 table の対象外なので T3 では
     実装しない。実機的には T7 (evaluator が bool typed `Assign`
     arm を持つ) で runtime 側 reject される。
   - 新規 `dead_code` 警告は観測していない。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - progress file の T3 行を `[x]` に更新し、retrospective への
      link と "abstract `bind label:` を `Text.text` に読み替え"
      ノートを追加。
    - T4 以降の task 構成・順序・依存関係に T3 実装から見て調整すべき
      点は出ていない。T4 (typed lowering) は T3 で既に存在する
      `Namespace` を `lower_expr` から参照するだけの追加で、T3 で
      新たに作った widget-property catalog は T4 では使わない (T4 は
      state side、catalog は widget property side)。

## Fast-Track Judgment

Fast-track criteria を満たしている。

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): なし
- item 5 (ついでのリファクタ): なし (`check_members` の内部分割は
  scope 内の signature 変更)
- item 6 (追加 DD 必要性): なし
- item 7 (Proposed → Accepted 昇格): なし
- item 8 (plan AC / Phase 構成変更): なし
- item 9 (持ち越し): 意図的な暫定 (T4 / T7 で解消) のみ

blocking item なし。

## Verification Notes

T3 で追加したテストと、走らせた command を記録する。

新規テスト (commit 1, state default):

- `wasamoc/src/check.rs`:
  - `bool_state_default_accepted`
  - `bool_state_default_int_literal_rejected`
  - `bool_state_default_string_literal_rejected`
  - `i32_state_default_bool_literal_rejected`
  - `string_state_default_bool_literal_rejected`

新規テスト (commit 2, bind LHS):

- `wasamoc/src/check.rs`:
  - `bind_bool_target_bool_state_ident_accepted`
  - `bind_bool_target_bool_literal_accepted`
  - `bind_bool_target_int_literal_rejected`
  - `bind_string_target_bool_literal_rejected`
  - `bind_string_target_bool_state_ident_rejected`
  - `bind_bool_target_i32_state_ident_rejected`
  - `bind_unknown_property_no_type_check`
  - `bind_component_level_no_type_check`
  - `bind_string_target_string_literal_accepted`

DD-M3-P1-010 の table の各行とテストの対応:

| Pattern (DD table) | Test |
|---|---|
| `state ready: bool = false` ✓ valid | `bool_state_default_accepted` |
| `state ready: bool = 0` reject | `bool_state_default_int_literal_rejected` |
| `state ready: bool = "false"` reject | `bool_state_default_string_literal_rejected` |
| `bind enabled: ready` (bool/bool) ✓ valid | `bind_bool_target_bool_state_ident_accepted` |
| `bind enabled: 1` reject | `bind_bool_target_int_literal_rejected` |
| `bind enabled: true` ✓ valid | `bind_bool_target_bool_literal_accepted` |
| `bind label: true` reject (string target) | `bind_string_target_bool_literal_rejected` (`Text.text`) |
| `bind label: ready` reject (bool→string) | `bind_string_target_bool_state_ident_rejected` (`Text.text`) |
| `state x: i32 = 5; bind enabled: x` reject | `bind_bool_target_i32_state_ident_rejected` |

加えて soft catalog 動作の負側 (= 何もしないこと) を保証する 2 件:

- `bind_unknown_property_no_type_check` — `Text { font: title }`,
  `Button { style: accent }` (catalog 外 prop + keyword-value ident)
- `bind_component_level_no_type_check` — `title: "Counter"`,
  `backdrop: mica` (component-level bind)

`bind_string_target_string_literal_accepted` は positive control で、
catalog 上の string-typed prop に対する正規 binding が型エラーで落ち
ないことを保証する (existing `dynamic_string_interp_lowered_to_ir_binding`
が lower 側で同等を見ているが、check 側にも明示)。

実行コマンド:

```text
cargo build --workspace
cargo test --workspace
```

いずれも green。`wasamoc` 単体テストは 91 passed (T2 完了時 77 件から
+14)。`wasamo-runtime` 等の他 crate は変更なしで 123 件 green を維持。

## Follow-Up

T3 から後続 task への明示的な引き渡し:

- **T4 (typed lowering):** `lower_expr` の `Expr::Ident { name }` arm
  と `lower_string_parts` の interp arm を、T3 で完成形になった
  `Namespace` (`HashMap<String, TypeName>`) を参照する形に書き換える。
  T3 は **検査** のみ ident → state type の対応を見ており、IR 出力は
  まだ `IrLiteral::Ident` / `HandlerExpr::PropRead` (i32-implicit) の
  まま。T4 で `TypeName::Bool → BoolPropRead`、`TypeName::Str →
  StrPropRead`、`TypeName::Int → PropRead` の振り分けを入れる。
- **T6 (`wasamo-runtime` widget catalog):** wasamo-runtime 側で
  `resolve_prop_key` が `Option<(PropertyKey, IrType)>` を返すように
  なるとき、wasamoc 側の `widget_prop_type` と意味的に一致する必要
  がある (`Button.enabled: bool`, `Text.text: string`,
  `Button.text: string`)。T6 のレビュー時は wasamoc 側 catalog と
  突き合わせて drift していないことを確認する。drift 検出のための
  cross-crate integration test は Phase 1 では立てない (両側とも
  hard-coded で同じ DD を読んでいるため drift しない前提)。M3-Phase 6
  で conditional rendering が catalog を更に拡張するときは再考。
- **T10 (spec sync):** `Button.enabled` を `docs/dsl_spec.md` の
  widget catalog セクションに追加するタイミングで、wasamoc が
  type-check で reject する DD-M3-P1-010 table が dsl_spec から
  reproducible である (= 同じ table を spec 側にも示す) かを確認する。

これらはすべて progress file の T4–T10 として既に列挙済み。T3 単体で
新たに発見された follow-up は無い。
