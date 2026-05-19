---
title: M3-Phase 1 / T2 step-end retrospective
status: recorded
created: 2026-05-19
scope: step-end
task: T2 — wasamoc lex/parse bool + lower to IrLiteral::Bool / HandlerExpr::BoolLit
---

# M3-Phase 1 / T2 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-1-progress.md` の **T2** ("`wasamoc`
lexer / parser: `true` / `false` keywords and bool literal") の
step-end retrospective。T2 が discharge する DD は DD-M3-P1-002 の
surface syntax 半分 と DD-M3-P1-006 の emit 半分 (T1 で mechanical
反映済み; T2 は parser→lower→emit の end-to-end 経路を閉じる位置に
入った)。

対象コミット:

- `992e7e1 feat(wasamoc): lex/parse bool literals; lower to IrLiteral::Bool / HandlerExpr::BoolLit (M3-Phase 1 T2)`

これは step-end の gate であり、phase-end retrospective ではない。

## Current Judgment

2026-05-19 時点で T2 step-end 基準は **達成済み**。

- Lexer: `Keyword::True` / `Keyword::False` を追加し、`scan_ident`
  が `"true"` / `"false"` を `Token::Kw(Keyword::True|False)` に
  ルーティング。`Token::Ident` を返すことが無くなったため、識別子を
  期待する全ての位置 (`expect_ident`, `parse_member` の Ident 分岐)
  で自動的に reject される。
- AST: `Expr::BoolLit { value: bool, span: Span }` を追加。
  `Expr::span()` の網羅 arm を更新。
- Parser: `parse_expr` の valid token 集合に
  `Token::Kw(Keyword::True | Keyword::False)` を追加し、`Expr::BoolLit`
  を生成。`parse_type_name` の "unknown type" メッセージを
  `bool` 含めて更新 (TypeName::Bool 受理は既存)。
- Lower: `lower_state` に `TypeName::Bool → IrType::Bool` と
  `Expr::BoolLit{value} → IrLiteral::Bool(value)` の arm。`lower_expr`
  (静的 bind) に `Expr::BoolLit → IrLiteral::Bool` の arm。
  `lower_rhs_expr` (handler RHS) に `Expr::BoolLit → HandlerExpr::BoolLit`
  の arm。
- Check: `check_expr_type` の valid 値集合に `Expr::BoolLit { .. }` を
  追加。詳細な type-checking は T3 範囲なので、ここでは「FloatLit と
  同じ反応にならない」だけを保証。
- `cargo build --workspace` / `cargo test --workspace` どちらも local
  で green。`wasamoc` 単体は 77 件 passed (T2 で新規追加 11 件含む)。

T2 の blocker は残っていない。

## Main Learning

中心的な学びは「`true`/`false` を `Token::BoolLit(bool)` ではなく
`Keyword::True`/`Keyword::False` として lex する選択が、reservation
セマンティクスをコードでなくトークン分類で自動成立させた」という点。

- `scan_ident` は識別子文字列を読み終えた後に keyword テーブルを
  引き、`true`/`false` を見つけたら `Token::Kw` に変換する。これに
  より `Token::Ident("true")` という値は **構造的に作れない**。
- `expect_ident()` は `matches!(self.peek(), Token::Ident(_))` を
  ガード条件としているので、`true` が来た時点で自動的に
  "expected identifier, found `true`" エラーになる。reservation を
  別途のチェック関数として書く必要が無かった。
- `parse_member` の Ident 分岐に入る前段でも同じことが起きるので、
  `Button { true: false }` のような property-bind LHS の `true`
  も自動 reject される。テストでは "expected member" メッセージで
  落ちる経路を確認した。

副次的な学びとして、T1 の retrospective で「下流の exhaustive match
を着手前に grep で網羅した」のと同じパターンを T2 にも適用したが、
T2 では `Expr` enum の network が `Expr::span()` / `lower_expr` /
`lower_rhs_expr` / `lower_state` / `check_expr_type` の 5 箇所に
限定されており、すべて compile error として表面化した。Float の既存
reject パターン (`Expr::FloatLit`) が良い反例として残っており、
bool を「FloatLit と同じく reject か、それとも受理か」を意識して
書けた。

T2 の commit 分割の判断: 当初は lexer / parser / lower を別 commit に
分けることも考えたが、

- AST に `Expr::BoolLit` を追加した瞬間に exhaustive match (`Expr::span`,
  `lower_expr`, `lower_rhs_expr`, `check_expr_type`) が割れる
- lexer に `Keyword::True/False` を追加した瞬間に `Keyword::description`
  の網羅 arm と `scan_ident` の match arm が同時に割れる

ため、intermediate な commit が build green を保てない (CLAUDE.md
Commit rules の "Bundling is required to keep the build/tests passing
at every commit" 例外に該当)。1 コミットで着地した。

## Checklist

1. **本作業の主要な学び:** あり。
   - reservation を keyword route で実装すると、identifier 検査側に
     追加コードが要らない。Token 分類が型システムの強制を肩代わり
     する形になった。
   - T1 の retrospective で記録した「exhaustive match の事前 grep」
     パターンは T2 でも有効。Expr enum の 5 箇所が全て compile error
     で表面化したので、`cargo check` の error log を見ながら走らせ
     なくても commit を bisect 可能な状態に保てた。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:** **なし**
   - T2 の対象は `wasamoc` crate (lexer / parser / AST / lower / check)
     のみ。surface syntax `true` / `false` キーワード予約や bool
     literal token は dsl_spec §2 / §2.1 / §4.2 の変更対象だが、その
     spec sync (A11) は T10 の責任範囲なので T2 では行わない。

3. **ローカル clean rebuild:** **green**
   - `cargo build --workspace`: green (4.96s)
   - `cargo test --workspace`: 全件 green (`wasamoc` 77 件、
     `wasamo-runtime` 123 件、`wasamo-ir` 7 件、他)
   - GitHub Actions 上の clean rebuild は phase-end gate で確認する
     (T12)。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - DD-M3-P1-002 (surface syntax `true`/`false`) は ADR Accepted
     済み。Token type 内部表現 (`Keyword::True/False` か
     `Token::BoolLit(bool)` か) は ADR が決めていないが、純粋に
     実装内部の選択であり、外から見える振る舞いに差は無い (どちらでも
     reservation と error message の質は同程度に作れる)。
     `Keyword::True/False` を選んだ理由は「reservation セマンティクス
     をトークン分類で自動成立させる」「`expect_ident` の既存ガードで
     自然に reject される」「error message の `` `true` ``/`` `false` ``
     リテラル表記が `Keyword::description()` の既存機構で出る」の 3 点
     (上の Main Learning に記載)。PO 判断を要する design call ではない。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 編集対象は lexer / parser / AST / lower / check のみで、すべて
     bool literal 経路の追加に直接寄与。`parse_type_name` のエラー
     文言を `"expected i32 or string"` → `"expected i32, string, or
     bool"` に揃えたのは bool 受理側との既存不整合の修正で、bool 追加
     と意味的に等価な範囲。format churn なし。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - 既存 DD-M3-P1-001..010 で T2 範囲はすべてカバーされている。
     Token 内部表現 (Keyword vs BoolLit) は実装ノイズであり、DD を
     立てる粒度ではない。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR は 2026-05-19 時点で全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A9 / A11 / A12 の文言は変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **意図的な暫定処理あり、`dead_code` 警告なし**
   - `lower_expr` で `Expr::Ident { name }` は依然として
     `IrLiteral::Ident(name)` (= 静的識別子リテラル) を返す。bool
     state name (`bind enabled: ready`) を `BoolPropRead` に下げる
     ロジックは T3 (type table 構築) + T4 (typed lowering) の責務で
     あり、T2 では触らない。
   - `check_expr_type` は `Expr::BoolLit { .. }` を valid 値集合に
     加えただけで、DD-M3-P1-010 の accept/reject table 全体は T3 で
     導入する。現状は「FloatLit と同じく reject されない」だけを
     保証する状態。
   - 新規 `dead_code` 警告は観測していない (cargo build / test の
     warning 出力に新規ノイズ無し)。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - progress file の T2 行を `[x]` に更新し、retrospective への
      link を追加。
    - T3 以降の task 構成・順序・依存関係に T2 実装から見て調整すべき
      点は出ていない。T3 の `check_members` widening と state-type
      table 構築は、T2 で `Expr::BoolLit` を check 側に通したので、
      表面の AST 変更追加なしで進められる。

## Fast-Track Judgment

Fast-track criteria を満たしている。

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): なし
- item 5 (ついでのリファクタ): なし (parse_type_name エラー文言の
  bool 追記は bool 受理側との整合性回復で、意味的範囲内)
- item 6 (追加 DD 必要性): なし
- item 7 (Proposed → Accepted 昇格): なし
- item 8 (plan AC / Phase 構成変更): なし
- item 9 (持ち越し): 意図的な暫定 (T3/T4 で解消) のみ

blocking item なし。

## Verification Notes

T2 で追加したテストと、走らせた command を記録する。

新規テスト:

- `wasamoc/src/lexer.rs`:
  - `bool_keywords`
  - `bool_keywords_not_treated_as_idents`
  - `bool_keyword_word_boundary`
- `wasamoc/src/parser.rs`:
  - `state_decl_bool_false`
  - `state_decl_bool_true`
  - `property_bind_bool_literal`
  - `true_rejected_as_state_name`
  - `false_rejected_as_state_name`
  - `true_rejected_as_widget_property_name`
- `wasamoc/src/lower.rs`:
  - `bool_state_lowered_to_ir_state`
  - `bool_literal_prop_bind_lowered_to_ir_prop`
  - `bool_literal_in_handler_lowered_to_handler_expr`
- `wasamoc/src/emit.rs`:
  - `bool_state_emitted`
  - `bool_literal_prop_emitted`
  - `bool_literal_in_handler_emitted`

実行コマンド:

```text
cargo build --workspace
cargo test --workspace
```

いずれも green。`wasamoc` 単体テストは 77 passed (T1 完了時 64 件から
+13)。

## Follow-Up

T2 から後続 task への明示的な引き渡し:

- **T3 (`wasamoc` checker, state-type table + accept/reject):**
  `collect_state_namespace` で既に `HashMap<String, TypeName>` を
  構築しているが、現状は `Bool` を含む `TypeName` を素直に格納する
  だけで、DD-M3-P1-010 の accept/reject table は未実装。T3 は
  - state default の型一致 (`state ready: bool = 0` reject など)
  - `bind` LHS の widget-property 型一致
  - identifier 解決時の state type 参照
  を入れる。`check_expr_type` の Expr::BoolLit arm は既に存在する
  ので、T3 はそこに型一致ルールを足す形になる。
- **T4 (typed lowering):** `lower_expr` の `Expr::Ident { name }` arm
  が現状 `IrLiteral::Ident(name)` を静的リテラルとして返す。T4 で
  `ns` を参照して bool 状態名なら `BoolPropRead`、i32 状態名なら
  `PropRead`、string 状態名なら `StrPropRead` に振り分ける。
- **T5 (IR loader):** T2 自体は wasamoc 側のみで、`wasamo-runtime`
  の IR text loader (`parse_state` / `parse_literal` / `parse_sexpr`)
  は T1 から catch-all 経由で reject される状態のまま。T5 で正規 arm
  を追加する (`"bool"` 型名、`Token::Ident("true"|"false")` literal、
  `"bool-prop-read"` sexpr)。T2 の emit 側が正しく `true` / `false`
  / `(bool-prop-read PATH)` を吐けることはテストで確認済みなので、
  T5 は emit 出力を入力として round-trip テストできる状態に整っている。

これらはすべて progress file の T3–T10 として既に列挙済み。T2 単体で
新たに発見された follow-up は無い。
