---
title: M3-Phase 3 / T6 step-end retrospective
status: recorded
created: 2026-05-21
scope: step-end
task: T6 — wasamo-runtime IR loader + validate() defense-in-depth (WrapPanel)
---

# M3-Phase 3 / T6 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-3-progress.md` の **T6**
(`wasamo-runtime` IR loader + `validate()` defense-in-depth) の
step-end retrospective。T6 が discharge する材料は次:

- DD-M3-P3-001 の materialisation 経路 —
  `ir_loader::construct_widget` の `"WrapPanel"` arm を立て、IR text
  → `IrNode { widget_type: "WrapPanel", … }` → `WidgetData::WrapPanel`
  を直接 wire する (Box-internal-pattern; `PropertyValue` を経由しない)。
- DD-M3-P3-006 の **runtime gate (compile-time との two-gate defense
  の片側)** — `validate()` で `item-cross-size` / `item-spacing` /
  `line-spacing` の **negative IntLit** を `IrLoadError::Validate` で
  reject。C ABI 表面では `WASAMO_ERR_IR_MALFORMED` に翻訳される
  (DD-M2-P6-005 / DD-M2-P6-009 の既存 mapping を継承)。
- ADR の Phase 3 verification closure **evidence item 3** (runtime
  gate). `wasamoc check` (T1) が compile-time 半分を持ち、本 T6 が
  runtime 半分を持つ。`wasamo_load_ui` の memory-IR path は wasamoc
  を経由しないので、両 gate が必要。

対象コミット (2 件):

- `577a9b4 feat(wasamo-runtime): WrapPanel IR loader + validate() negative gate (M3-Phase 3 T6)`
- `0774dee docs(m3-phase-3): flip T6 checkboxes (wasamo-runtime IR loader + validate)`

step-end の gate であり phase-end retrospective ではない。merge 先は
phase ブランチ `feat/m3-phase-3` (ff)。本 step (T6) は単一 task =
単一 step 構造で、現在のブランチは `feat/m3-phase-3-t6`。

## Current Judgment

2026-05-21 時点で T6 step-end 基準は **達成済み**。fast-track 判定は
**適格** (checklist item 2–8 が「なし」、item 3 green、item 9 は
T6 自体が introduce した placeholder / dead_code は無し)。

- `ir_loader::construct_widget` の `"WrapPanel"` arm:
  `extract_int_prop(&node.props, "item-cross-size")` ほか 3 件で
  presence-preserving に `Option<i32>` を抽出し、そのまま
  `WidgetNode::wrap_panel(compositor, item_cross_size, item_spacing,
  line_spacing)` に渡す。**loader 側に default knowledge を持ち込まない**
  — `unwrap_or(0)` のような defaulting は呼ばない。これは T5
  retrospective で析出した「defaults at the runtime layer in T5 は
  catalog constructor の責務」の素直な帰結。catalog 内の
  `apply_wrap_panel_defaults` が単一権威 site で absent→default を
  resolve する。
- `validate_phase3_node_invariants(node)` を新規追加し、`validate()`
  の末尾で呼ぶ。`widget_type == "WrapPanel"` の node 上の
  `item-cross-size` / `item-spacing` / `line-spacing` prop に対し、
  `IrLiteral::Int(n)` で `n < 0` のものを `IrLoadError::Validate(format!(
  "`WrapPanel.{}` must be non-negative, got {}", ...))` で reject。
  子 node も再帰的に walk するので、nested WrapPanel も gate される。
  - **scope 判断**: attribute-position rejection (non-WrapPanel widget
    上の同名 prop) は compile-time の `wasamoc check` T1 側の責務とし、
    runtime gate には含めない。progress doc T6 の文言が "negative
    item_cross_size / item_spacing / line_spacing values" のみを
    指していることに従う (Phase 2 T7 の `validate_phase2_node_invariants`
    が ratio/color の placement 全体を検査していたのとは scope が違う)。
  - **non-IntLit RHS の扱い**: `extract_int_prop` が non-IntLit を
    silently `None` 扱いする (= 属性不在として default 適用) ため、
    本 gate は IntLit のみを対象にする。memory-IR で例えば
    `IrLiteral::Str("…")` が混入しても reject されずに「不在」扱い
    される — これは Phase 2 T7 が ratio/color の placement gate を
    Box 以外でも reject する設計と非対称だが、progress doc T6 が
    「negative 値」のみを対象に明示していることに合わせた。
- 新規 unit test 9 件 (`wasamo-runtime/src/ir_loader.rs::tests`):
  - `wrap_panel_zero_children_is_valid` — DD-M3-P3-001 no-lower-bound。
  - `wrap_panel_single_child_is_valid` — 1-child accept。
  - `wrap_panel_multi_child_is_valid` — 4-child (no upper bound)。
  - `wrap_panel_rejects_negative_item_cross_size` — `-1` reject、
    error message に `item-cross-size` と `non-negative` を含む。
  - `wrap_panel_rejects_negative_item_spacing` — `-5` reject、同上。
  - `wrap_panel_rejects_negative_line_spacing` — `-10` reject、同上。
  - `wrap_panel_accepts_zero_on_all_three_attributes` —
    DD-M3-P3-006 の zero-handling (`< 0` であって `<= 0` ではない)。
  - `wrap_panel_accepts_positive_values_on_all_three_attributes` —
    full positive accept (96 / 8 / 12)。
  - `wrap_panel_negative_value_in_nested_node_is_rejected` —
    VStack の子 WrapPanel に対する `-3` を recurse で gate。
  - **error は全件 `IrLoadError::Validate` であり**
    `assert_malformed_display_nonempty` で `is_malformed() == true`
    と display message non-empty を pin (= C ABI 翻訳が
    `WASAMO_ERR_IR_MALFORMED` 側に落ちることを保証)。
- T5-era の `#[allow(dead_code)]` forward-pointer を 3 箇所外す:
  - `WidgetData::WrapPanel` variant — `construct_widget` から
    `WidgetNode::wrap_panel` 経由で構築される。
  - `WidgetNode::wrap_panel` constructor — `construct_widget`
    `"WrapPanel"` arm の caller。
  - `apply_wrap_panel_defaults` 自由関数 — `WidgetNode::wrap_panel`
    内部から呼ばれる。
  - 6 → 3 への減少。残る 3 は `LayoutNode.item_cross_size` /
    `item_spacing` / `line_spacing` の field で、T7 (measure-arrange)
    が reader を入れて初めて lift する (T5 retrospective Follow-Up
    と一致)。
- **Clean rebuild gate (post-commit; commit `0774dee`):**
  値は本 retrospective 末尾の "Verification Notes" 節に記録。
  `wasamo-runtime` lib は **216 passed** (T5 の 207 から +9)。

T6 の blocker は残っていない。T7 (Layout engine: WrapPanel
line-breaker and arrange) へ進める。

## Main Learning

最も load-bearing な学びは **「runtime gate の scope は progress doc
の文言に厳密に従い、Phase 2 T7 の placement-gate pattern には
無批判に揃えない」**。

- Phase 2 T7 の `validate_phase2_node_invariants` は ratio/color
  literal の placement 全体 (Box 以外で出ること、Box でも `aspect` /
  `fill` 以外の prop 名で出ること) を reject する設計だった。これは
  DD-M3-P2-002 / DD-M3-P2-003 が "Box-internal field、`PropertyValue`
  を経由しない" という構造的 invariant に基づくため、placement 違反は
  runtime memory-IR でも error にすべき (型レベルで違反させない手段が
  ない)。
- 一方 Phase 3 の DD-M3-P3-006 runtime gate は **非負整数** という
  値レベルの spec invariant の last-line-of-defence であり、属性位置
  自体は compile-time の `wasamoc check` T1 が完全に押さえている。
  runtime で attribute-position も検査するか? 検査しない方が:
  - progress doc 文言と整合 (T6 は "negative values" のみ明記)
  - 既存 `extract_int_prop` を再利用できる (placement gate を入れる
    なら専用 walker が必要)
  - error surface が膨れない (`WASAMO_ERR_IR_MALFORMED` の error 文言
    集合が小さいほど ABI consumer 側で扱いやすい)
- 「Phase 2 と同じパターンに揃える」誘惑は強かったが、両者の対象が
  違う (Phase 2: 構造的 placement / Phase 3: 値範囲) ことが見えてからは、
  scope を分離する方が retrospective rule 「revise docs, don't work
  around them」の意図に近い (progress doc に "全 placement reject" と
  書いていない以上、勝手に拡張しない)。

次に load-bearing な学びは **「T5 retrospective Follow-Up を
loader 実装で素直に履行することで、T6 の `#[allow(dead_code)]`
lifting が機械的に進む」**:

- T5 で `apply_wrap_panel_defaults` を catalog 側に切り出した瞬間、
  T6 の loader は default 知識を持つ必要がなくなり、`extract_int_prop`
  の戻り値 `Option<i32>` をそのまま流すだけになった。loader が
  3-line 程度の単純 arm で済む。
- 結果として `#[allow(dead_code)]` lifting も機械的: 3 markers (variant
  + constructor + helper) が **同一の commit** で自然に lift する。
  Phase 2 T6 → T7 が `WidgetData::Box` 単一 marker を lift した形と
  symmetric だが、helper が増えた分だけ markers が多い (= 3 件)。
- **Phase 2 T6 retro の Main Learning** が再現: 「`#[allow(dead_code)]`
  は forward-pointer marker であり安全網ではない」。これは T7 で
  `LayoutNode` の 3 field を lift するときにも適用される (= T7 で
  marker が残るのは bug、漏らさず lift する)。

副次的な学び:

- **`IrLiteral::Int` の signed `i32` 表現は Phase 2 末で既に確立して
  おり、negative 値が runtime 表面に到達できる**。これは lexer が
  `-1` を `Token::Int(-1)` として lex すること、`parse_literal` が
  そのまま `IrLiteral::Int(-1)` を作ること、`tokenize` の `'-'` arm
  (line 384–402) が digit lookahead で negative `IntLit` を発する
  ことに依存する。Phase 2 では Ratio (`<num>:<den>`) が
  `num: i32` を持っていたが、構文的に符号付きで来ることが想定
  されていない (compile-time で `-1:2` は不正)。Phase 3 で初めて
  「`IrLiteral::Int` が memory-IR で負になりうる」という前提を
  runtime gate で扱う形になった。
- **`validate()` が現在 `parse_ir` の中で呼ばれる位置**: 構文 parse 後・
  return 前。construct_widget は build_node 経由で `validate()` の
  後にしか呼ばれない (build_widget_tree から)。したがって
  `construct_widget` の `WrapPanel` arm に「負値を assert」のような
  防衛コードを入れる必要はない — `validate()` で必ず弾かれる前提で
  良い。これは Phase 2 T7 の Box / Ratio / Color materialisation arm
  と同じ pattern。

## Checklist

1. **本作業の主要な学び:** あり (記述項目)。
   - runtime gate の scope は progress doc 文言に厳密に従う
     (placement rejection は compile-time、runtime は値範囲)。
   - T5 で catalog 側に default を閉じたことで、T6 loader 実装は
     極小 (extract_int_prop の戻り値を流すだけ) + dead_code 3 件が
     同時 lift する機械的な完了になる。
   - `IrLiteral::Int(i32)` は memory-IR 上で負値を取りうる
     (Phase 2 では明示的問題化されなかった前提)。
   - `validate()` の実行位置 (parse 後・build 前) により、construct
     arm 側で値範囲を再検証する必要がない (Phase 2 T7 と同 pattern)。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`) の変更:**
   **なし**
   - T6 は `wasamo-runtime` 内部の loader / validate 経路のみ。
     `dsl_spec §4.10` の WrapPanel 表面は T1–T4 で draft 済み、
     Moment 2 spec re-sync は T10 の責任。`abi_spec` への影響は
     `IrLoadError::Validate` の error class が `WASAMO_ERR_IR_MALFORMED`
     に既に mapping 済みのため不要。`architecture.md` への影響なし。

3. **ローカル clean rebuild:** **green**
   - `cargo fmt --all -- --check` (post-commit state; commit `0774dee`):
     zero exit。
   - `cargo clean`: 3408 files, 989.4 MiB removed。
   - `cargo build --release --workspace`: green (42.09s)。
   - `cargo build --workspace`: green (debug, 39.54s)。
   - `cargo test --workspace`: failure 0 件。
     - `wasamo-runtime` lib: **216 passed** (T5 の 207 から +9:
       validate WrapPanel test 9 件)。
     - `wasamoc` lib: 202 passed (T5 と同じ)。
     - `wasamo-ir`: 12 passed。
     - `wasamo-runtime` integration `ir_loader_roundtrip`: 6 passed。
     - 他 crate (ABI / DLL / binding / counter-rust / gallery-rust /
       bool-demo-rust / counter-c) 全 green。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - T6 範囲は DD-M3-P3-001 / DD-M3-P3-006 から機械的に降りる。
     scope 判断 (attribute-position は runtime gate 対象外、
     non-IntLit RHS は absent 扱い) は progress doc 文言の素直な
     読みであり、設計判断ではない (Main Learning 第 1 項参照)。
   - extract_int_prop を再利用するか専用 helper を作るかは、
     T5 retrospective Follow-Up が「presence を保つ抽出 helper を
     新設」と書いていたが、実コード確認の結果 `extract_int_prop`
     自体が既に `Option<i32>` を返す (defaults を呼び出し側で
     `.unwrap_or(0)` する) 設計だったため、新規 helper は不要だった。
     再利用で済ませる判断はオーナーレビュー範囲に含めず、本
     retrospective に記録した。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 変更 file は 2 件 (`wasamo-runtime/src/ir_loader.rs` +
     `wasamo-runtime/src/widget.rs`)。`#[allow(dead_code)]` 3 件の
     lifting は T5 が forward-pointer として置いた marker の自然な
     完了であり、ついでリファクタではない (T5 retrospective
     Follow-Up にも明記されている操作)。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - DD-M3-P3-001 / DD-M3-P3-006 で T6 範囲は完全にカバー。
     scope 判断 (Main Learning 第 1 項) は新規 DD 化を要する設計
     判断ではなく、既存 DD の素直な読み。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **なし (本 step 内で新規 introduce はゼロ)**
   - T6 自体は placeholder / approximation / 新規 `dead_code` を
     introduce していない。むしろ T5-era markers を 6 → 3 へ
     減らした。
   - 残る 3 markers (`LayoutNode.item_cross_size` / `item_spacing`
     / `line_spacing`) は T5 で導入された forward-pointer であり、
     T6 ではなく T7 (measure-arrange) で reader が入って初めて
     lift する設計。T5 retrospective Follow-Up にも明記済み。
     **これらは T6 が「持ち越す」ものではなく T5 が継続して保有
     しているもの** — 同様に `measure_wrap_panel` の `Ok((0.0, 0.0))`
     placeholder と `arrange` `WidgetKind::WrapPanel` arm の no-op
     も T5 由来で T6 では触れていない。

10. **タスクリストの後続 step 見直し:** **なし (進行通り)**
    - T7 / T8 / T9 / T10 の構成・順序・依存関係に T6 実装から見て
      調整すべき点は出ていない。
    - T7 への follow-up は下記 "Follow-Up" 節に明示。

## Fast-Track Judgment

Fast-track criteria を **満たす**:

- item 2 (spec doc 変更): なし
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): なし
- item 5 (ついでリファクタ): なし
- item 6 (追加 DD): なし
- item 7 (Proposed 増加/昇格): なし
- item 8 (m3-plan AC 変更): なし
- (item 9 は (FT) ではないが、本 step では「なし」)

memory `feedback_step_end_gate_discipline.md` の規律
("item 2–8 すべて『なし』+ item 3 green に厳密") を満たす。
step→phase ブランチへの ff merge をオーナー明示確認後に実行する
(retrospectives.md §進行手順 step 3 の fast-track 規定では事後通知
可だが、本プロジェクトの運用ルール上は最低 1 回のオーナー確認を
保持する慣行に従う)。

## Verification Notes

T6 で追加したテストと、走らせた command を記録する。

新規テスト (ir_loader, wasamo-runtime): 9 件
(`wasamo-runtime/src/ir_loader.rs::tests`):

- `wrap_panel_zero_children_is_valid`
- `wrap_panel_single_child_is_valid`
- `wrap_panel_multi_child_is_valid`
- `wrap_panel_rejects_negative_item_cross_size`
- `wrap_panel_rejects_negative_item_spacing`
- `wrap_panel_rejects_negative_line_spacing`
- `wrap_panel_accepts_zero_on_all_three_attributes`
- `wrap_panel_accepts_positive_values_on_all_three_attributes`
- `wrap_panel_negative_value_in_nested_node_is_rejected`

実行コマンド (post-commit; commit `0774dee` 時点):

```text
cargo fmt --all -- --check                 (post-commit state; zero exit)
cargo clean                                (3408 files, 989.4 MiB)
cargo build --release --workspace          (42.09s, green)
cargo build --workspace                    (debug; 39.54s, green)
cargo test --workspace                     (failure 0)
```

いずれも green。`wasamo-runtime` lib test は **216 passed**
(T5 の 207 から +9)、他 crate の test count は T5 と同じ。

## Follow-Up

T6 から後続 task への明示的な引き渡し:

- **T7 (Layout engine: WrapPanel line-breaker and arrange):**
  `measure_wrap_panel` placeholder (`Ok((0.0, 0.0))`) を
  DD-M3-P3-005 の bounded / unbounded main-axis line-breaker に
  置き換える。`arrange` の `WidgetKind::WrapPanel` arm を children
  の per-line arrange に置き換える。`LayoutNode.item_cross_size` /
  `item_spacing` / `line_spacing` を読むことで残る 3 件の
  `#[allow(dead_code)]` も自然に lift する。CLAUDE.md §Testing
  rules に従い、まず free-function 抽出を検討し、`LayoutNode`
  自体が Win32/WinRT-free なので free-function で書ける見込み
  (test-only mirror pattern は不要のはず)。
- **T8 (Windows-runtime integration test):** `construct_widget` の
  `"WrapPanel"` arm を end-to-end で exercise する材料が T8 で
  揃う (T6 単体では Compositor を要するため pure-logic test では
  カバーできない)。T8 の wrap-path / oversized-child fixture は
  validate → construct → measure → arrange の経路全体を回す。
- **non-IntLit RHS 取り扱いの非対称性 (Phase 2 vs Phase 3):**
  Main Learning 副次節に記載のとおり、Phase 2 T7 が
  ratio/color の placement を Box 以外で reject するのに対し、
  Phase 3 T6 は non-IntLit RHS を silently absent 扱いする。
  これは progress doc T6 の scope に従った設計判断だが、Phase 8
  の WrapPanel 拡張 (例えば binding を許可する場合) で見直しが
  必要になる可能性がある。Out-of-phase residual ではなく
  forward-pointer として記録するに留める (現 phase の DD では
  問題化されていない)。

これらはすべて progress file の T7 / T8 として既に列挙済み。
T6 単体で新たに発見された Out-of-phase residual は無し。
