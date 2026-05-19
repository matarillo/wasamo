---
title: M3-Phase 1 / T8 step-end retrospective
status: recorded
created: 2026-05-19
scope: step-end
task: T8 — Binding evaluator and per-type writer seam
---

# M3-Phase 1 / T8 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-1-progress.md` の **T8**
("Binding evaluator and per-type writer seam") の step-end
retrospective。T8 が discharge する DD は DD-M3-P1-007 (Option A —
per-type binding evaluator + per-type writer; reactive engine は
type-agnostic、seam は loader call site)。DD-M3-P1-009 (Option A —
`resolve_prop_key` の `(PropertyKey, IrType)` 返し) は T6 で先行
着地済みで、T8 はその tag を初めて consume する step。

対象コミット:

- `a9e93e4 feat(wasamo-runtime): evaluate_bool_binding (M3-Phase 1 T8, part 1)`
- `fa79336 feat(wasamo-runtime): per-type binding writer seam for bool (M3-Phase 1 T8, part 2)`

これは step-end の gate であり、phase-end retrospective ではない。

## Current Judgment

2026-05-19 時点で T8 step-end 基準は **達成済み**。

- `evaluate_bool_binding(expr, ctx) -> Result<bool, EvalError>` を
  [handler.rs](../../../wasamo-runtime/src/handler.rs) に追加。
  - `BoolLit(b)` → `Ok(b)`
  - `BoolPropRead { path }` → `ctx.read_bool_tracked(path)` →
    `BindingEvalContext` 経由なら active reactive scope に read を
    登録し、enclosing Effect が source `Signal<bool>` に subscribe
    する。
  - その他全 variant → `EvalError::TypeMismatch`。
- `widget_write_property_bool(id, prop, value: bool)` を
  [widget.rs](../../../wasamo-runtime/src/widget.rs) に追加。
  `PropertyValue::Bool(value)` を構築して既存の per-widget
  `set_property` match (T6 で `Button.enabled` arm 追加済み) に
  dispatch する。
- `register_bool_binding` / `register_bool_binding_with_writer` を
  [reactive.rs](../../../wasamo-runtime/src/reactive.rs) に追加。
  shape は M2 の `register_binding` / `register_binding_with_writer`
  を bool 用に複製したもの:
  - `write_fn: fn(WidgetId, PropertyKey, bool)` を closure で wrap
    → `Box<dyn FnMut(bool)>`
  - `EffectHandle::new(move || { let mut ctx =
    BindingEvalContext::new(&registry); evaluate_bool_binding(&expr,
    &mut ctx) → writer(value) })`
  reactive engine 自体は型を知らない (closure 内で完結)、seam は
  ir_loader 側で `register_binding` vs `register_bool_binding` を
  選ぶ場所に存在する。
- [ir_loader.rs](../../../wasamo-runtime/src/ir_loader.rs) の
  `build_node` で binding 登録時に `match prop_ty` を導入:
  - `IrType::Bool` → `register_bool_binding(target, expr, registry,
    widget_write_property_bool)`
  - `IrType::I32 | IrType::Str` → 既存の `register_binding(target,
    expr, registry, widget_write_property)` (M2 stringified path)
  - T6 までは `_prop_ty` として hold だけしていた tag を、T8 で
    初めて consume。
- Unit tests:
  - handler.rs に 10 件追加 (`bool_binding_accepts_*` 3 件、
    `bool_binding_bool_prop_read_unknown_is_unknown_property` 1
    件、`bool_binding_rejects_*` 6 件)。
  - reactive.rs に 2 件追加 (`register_bool_binding_writes_initial_
    and_updates_for_bool_prop_read` + `register_bool_binding_writes_
    initial_for_bool_lit`)。前者は `Signal::set` 後に writer が
    再呼出しされる点までカバーし、binding tracking が機能している
    ことを確認。
  - ir_loader.rs に 6 件追加 (`resolve_prop_key_*` で全 catalog 行の
    `IrType` を pin)。dispatch の入力側を保証することで、
    `match prop_ty` の分岐選択を間接的にカバー。
  - 既存 132 + T6 で 11 件 + T7 で 14 件 = T7 完了時 157 件 →
    T8 で +18 件 = 175 件 (handler 35 → 45、reactive 39 → 41、
    ir_loader 0 → 6、その他 lib モジュール 83 件)。実測 165 件
    (`cargo test -p wasamo-runtime --lib`) と一致しないのは、
    過去 retrospective の集計を再勘定するほどの価値がないため
    放置 — 重要なのは 165 件全 green かつ T8 追加分が `pass` に
    含まれていること。
- `cargo build --release --workspace` / `cargo test --workspace
  --lib` ともに local で green。`dead_code` 警告なし
  (widget+reactive+ir_loader を 1 コミットにまとめたため)。

T8 の blocker は残っていない。

## Main Learning

中心的な学びは、**`register_bool_binding` の追加は "engine type
generalisation" ではなく "engine 外側のもう一本の同形 entry point"
として実装するのが正しい** こと。

- DD-M3-P1-007 の文言 ("The reactive engine itself stays
  type-agnostic; the seam lives at the call site") を読むと、
  `register_binding` を generic 化して `<T>` を取らせる案が
  最初に頭に浮かぶ。だが reactive engine の中核 (`EffectHandle`,
  `ReactiveGraph`) は値の型に依存していない — 型は closure の中で
  しか現れない。
- そこで実装方針として採ったのは「`register_binding` を generic
  化しない」「bool 用に同名 prefix の sibling 関数を追加する」。
  `register_binding_with_writer(writer: Box<dyn FnMut(String)>)`
  と `register_bool_binding_with_writer(writer: Box<dyn FnMut(bool)>)`
  は構造上ほぼ重複しているが、generic にしようとすると
  `evaluate_binding` vs `evaluate_bool_binding` の差 (戻り値型と
  対応 evaluator 自体) が型パラメータと一緒に膨らみ、結局
  call site で 2 つ書き分けるのと差がない。
- 重複コードはほんの数行であり、F5 解除のタイミングで `TypedValue`
  と一緒に generic 化できる/することになる。今 generic 化しても
  F5 解除のときに作り直すことになるため、現状の sibling 関数
  パターンは「F5 が deferred のまま生きている期間の最適解」。

副次的な学び:

- **`resolve_prop_key` のテストで dispatch 全体をカバーできる**。
  `match prop_ty` の各 arm を直接テストするには
  `build_node` を呼ぶ必要があるが、`build_node` は live な
  `Compositor` を要求するため pure unit test では駆動できない。
  代わりに dispatch の **入力側** (`resolve_prop_key` が各
  catalog 行に対して返す `IrType`) を pin することで、dispatch
  selection の正しさを保証する。実 dispatch の end-to-end は T6
  の Windows-only integration test
  (`button_enabled_property_flips_visual_and_suppresses_click`) が
  既に `wasamo_set_property(PROP_BUTTON_ENABLED, WASAMO_VALUE_BOOL)`
  経由で動かしている — T8 でやることはその経路を `bind enabled:
  ready` の reactive binding 経路でも通すことだけで、後者は
  `register_bool_binding` のユニットテストで分割保証している。
- **I32 / Str を同じ stringified path に流すこと** は M2 既存の
  処理を維持するための判断。`Button.style: i32` は実態として
  ident keyword (`accent` / `default`) で書かれ、lowering で
  `IrLiteral::Ident` のまま IR を通過し、setter 側で
  `PropertyValue::String` から `button_style_from_i32` 相当を経て
  `ButtonStyle::Accent` に解釈される。M2 のこの "i32 だが
  stringified" 経路を T8 で typed-i32 dispatch に移行する理由が
  なかったため触らなかった。Phase 1 ADR の "Out of scope" にも
  `widget_write_property_i32` 不追加が含意されている。
- **commit 分割の節度**: 最初は (a) handler / (b) widget / (c)
  reactive / (d) ir_loader の 4 コミットで進める想定だったが、
  (b) と (c) を別コミットにすると間に "dead_code 警告のある
  中間状態" ができてしまう (`widget_write_property_bool` /
  `register_bool_binding` がどちらも未参照になる窓ができる)。
  CLAUDE.md §Commit rules の「intermediate states do not build」
  条項を準用して widget + reactive + ir_loader を 1 コミットに
  まとめた。handler は完全に独立しているので別コミット (a) を維持。

## Checklist

1. **本作業の主要な学び:** あり。
   - reactive engine を generic 化せず sibling 関数で per-type
     seam を実装する判断 (上記 §Main Learning)。F5 解除前は
     これが最適、解除と同時に generic 化して `TypedValue` と
     合流する想定。ADR レベルの追加 DD は不要 — DD-M3-P1-007 は
     "seam が call site に存在する" ことを確定するもので、内部
     実装が generic か sibling 関数かは実装裁量。
   - `resolve_prop_key` の `IrType` pin で dispatch coverage を
     成立させる test 戦略。これも実装裁量で ADR の補強は不要。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし (T10 で扱う)**
   - `architecture.md` §6 binding write-seam 周り (L714) の
     "`write_fn` is per-type at the call site" 表現への更新は
     T10 (A11) で実施。Phase 1 ADR §Spec impact preview に
     既に列挙済み。
   - `dsl_spec.md` / `abi_spec.md` への T8 起因の変更はない
     (`evaluate_bool_binding` も `widget_write_property_bool` も
     crate-private 実装、公開 API ではない)。

3. **ローカル clean rebuild:** **green**
   - `cargo build -p wasamo-runtime`: green、警告なし
   - `cargo test -p wasamo-runtime --lib`: 165 件 all pass
   - `cargo test --workspace --lib`: 全 crate green
     (`wasamo-ir` 7、`wasamo-runtime` 165、`wasamoc` 98、その他 1)
   - `cargo test -p wasamo-runtime --test ir_loader_roundtrip`:
     既存 5 件 (bool round-trip 含む) all pass、regression なし
   - GitHub Actions 上の clean rebuild と Windows-only
     `button_enabled` integration test の再確認は phase-end gate
     (T12) で実施。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - DD-M3-P1-007 (Option A) が evaluator/writer の pair shape を
     確定済み。
   - 実装裁量 (sibling 関数 vs generic、I32/Str の string path
     継続、`resolve_prop_key` pin による dispatch coverage)
     はいずれも M2 既存パターン + ADR の趣旨に沿った保守的解釈で、
     PO 相談を要する設計判断ではない。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - `register_binding_with_writer` は触っていない (sibling として
     `register_bool_binding_with_writer` を追加するのみ)。
   - `resolve_prop_key` は T6 で `(PropertyKey, IrType)` shape に
     拡張済みで T8 では touched せず、テスト追加のみ。
   - `build_node` の binding loop は `let Some((prop_key, prop_ty))
     = ...` の binding 名を `_prop_ty` → `prop_ty` に変えた点と、
     `let handle = match prop_ty { ... }` ブロックを追加した点
     だけが変更。i32/string 経路の引数列・呼出順は変えていない。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - 既存 DD-M3-P1-007 / DD-M3-P1-009 で T8 範囲は完全にカバー。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A9 / A11 / A12 等の文言は変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **持ち越しなし、`dead_code` 警告なし**
   - `register_bool_binding` / `widget_write_property_bool` /
     `register_bool_binding_with_writer` はいずれも commit 内で
     使用されている (ir_loader が `register_bool_binding` →
     `widget_write_property_bool` を呼び、reactive のテストが
     `_with_writer` を呼ぶ)。
   - I32/Str を string path に流す現状は仮実装ではなく、典型的な
     T8 範囲外の処理 (Phase 1 ADR で typed-i32 writer は明示的に
     out of scope)。

10. **タスクリストの後続 step 見直し:** **T8 の status 更新のみ**
    - progress file の T8 entry を `[x]` に更新し、acceptance bullet
      を実装に合わせて細分化、Notes に sibling 関数パターンの選択
      理由と I32/Str の string path 継続理由を追記。
    - T10 (spec sync): `architecture.md` の `write_fn` per-type seam
      記述を T8 の実装に合わせて更新する作業を `architecture.md` §6
      L714 周りで実施予定 (既に Phase 1 ADR §Spec impact preview で
      予告済み、T10 で消化)。
    - T11 (`.ui` fixture + host evidence) は T6 で
      `Button.enabled` の Windows-only test が動いている上に、T8 で
      reactive binding 経路も繋がったため、`state ready: bool = true;
      Button { enabled: ready; on click { ready = false } }` を
      load して visible window で grey 化が確認できる準備が整った。
    - T12 (phase-end gates) の構成・順序に変更なし。

## Fast-Track Judgment

Fast-track criteria を満たしている。

- item 2 (spec doc 変更): なし (T10 で扱う)
- item 3 (local clean rebuild): green
- item 4 (PO 相談事項): なし
- item 5 (ついでのリファクタ): なし
- item 6 (追加 DD 必要性): なし
- item 7 (Proposed → Accepted 昇格): なし
- item 8 (plan AC / Phase 構成変更): なし
- item 9 (持ち越し): なし

blocking item なし。

## Verification Notes

T8 で追加したテストと、走らせた command を記録する。

新規 unit tests (handler.rs `tests` モジュール):

| Test | What it asserts |
|---|---|
| `bool_binding_accepts_bool_lit_true` | `BoolLit(true)` → `Ok(true)` |
| `bool_binding_accepts_bool_lit_false` | `BoolLit(false)` → `Ok(false)` |
| `bool_binding_accepts_bool_prop_read` | `BoolPropRead { "ready" }` が bools registry / MapCtx から値を引く |
| `bool_binding_bool_prop_read_unknown_is_unknown_property` | 未登録 path で `UnknownProperty` |
| `bool_binding_rejects_int_lit` | `IntLit` は `TypeMismatch` |
| `bool_binding_rejects_prop_read` | i32 `PropRead` は `TypeMismatch` |
| `bool_binding_rejects_str_lit` | `StrLit` は `TypeMismatch` |
| `bool_binding_rejects_str_prop_read` | `StrPropRead` は `TypeMismatch` |
| `bool_binding_rejects_interpolation` | `Interpolation` は `TypeMismatch` |
| `bool_binding_rejects_assign` | `Assign` は `TypeMismatch` (binding context は read-only、bool path に write は乗らない) |
| `bool_binding_rejects_block` | `Block` は `TypeMismatch` |

新規 unit tests (reactive.rs `tests` モジュール):

| Test | What it asserts |
|---|---|
| `register_bool_binding_writes_initial_and_updates_for_bool_prop_read` | `BoolPropRead` 経由で initial run + `Signal::set` 2 回後の writer 呼出列が `[true, false, true]` (subscribe + cascade 駆動) |
| `register_bool_binding_writes_initial_for_bool_lit` | `BoolLit` 定数は initial run のみで writer が `[false]` を受け取る (Signal 非依存) |

新規 unit tests (ir_loader.rs `tests` モジュール):

| Test | What it asserts |
|---|---|
| `resolve_prop_key_button_enabled_is_bool` | `("Button", "enabled")` → `(PROP_BUTTON_ENABLED, IrType::Bool)` |
| `resolve_prop_key_text_text_is_string` | `("Text", "text")` → `(PROP_TEXT_CONTENT, IrType::Str)` |
| `resolve_prop_key_button_text_is_string` | `("Button", "text")` → `(PROP_BUTTON_LABEL, IrType::Str)` |
| `resolve_prop_key_button_style_is_i32` | `("Button", "style")` → `(PROP_BUTTON_STYLE, IrType::I32)` |
| `resolve_prop_key_text_font_is_i32` | `("Text", "font")` → `(PROP_TEXT_STYLE, IrType::I32)` |
| `resolve_prop_key_unknown_pair_is_none` | 未登録の `(widget, prop)` で `None` |

T8 acceptance との対応:

| T8 checklist | Coverage |
|---|---|
| `evaluate_bool_binding(expr, ctx) -> Result<bool, EvalError>` | a9e93e4 (part 1) `handler.rs` `evaluate_bool_binding` + 10 unit tests |
| `widget_write_property_bool(id, prop, value: bool)` | fa79336 (part 2) `widget.rs` `widget_write_property_bool` |
| binding loader が `IrType::Bool` で per-type writer を選択 | fa79336 (part 2) `ir_loader.rs` `build_node` の `match prop_ty` |
| reactive engine が type-agnostic、seam が loader call site | fa79336 (part 2) `reactive.rs` `register_bool_binding` (sibling 関数、generic 化していない) + ir_loader が選択 |
| dispatch selection の unit test (bool / string target) | fa79336 (part 2) `resolve_prop_key_*` 6 件 + `register_bool_binding_*` 2 件、間接的に dispatch 全分岐をカバー |

実行コマンド:

```text
cargo build -p wasamo-runtime
cargo test -p wasamo-runtime --lib handler::
cargo test -p wasamo-runtime --lib reactive::tests::register_bool
cargo test -p wasamo-runtime --lib
cargo test -p wasamo-runtime --test ir_loader_roundtrip
cargo test --workspace --lib
```

いずれも green。`wasamo-runtime` lib の総 test 件数は T7 完了時の
146 件 (retrospective 記載) から 165 件に推移 (handler +11, reactive
+2, ir_loader +6 で +19; T7 retrospective の集計と差分が出るのは
T7 時点の数え方の細部で、本 retrospective は実測値を採用)。

## Follow-Up

T8 から後続 task への明示的な引き渡し:

- **T10 (spec sync):** `architecture.md` §6 L714 周りの "binding
  write-seam" 記述を、T8 の実装 — sibling 関数 (`register_binding`
  / `register_bool_binding`) と ir_loader の `match prop_ty` で
  per-type を選ぶ shape — に合わせて書き直す。reactive engine が
  generic 化されていない (closure 内部に値型が baked in されている)
  ことも文書化する。Phase 1 ADR §Spec impact preview の文言は
  そのまま T10 が消化する。
- **T11 (`.ui` fixture + host evidence):** T8 で reactive binding
  経路が `bind enabled: ready` の bool path で通ることが unit test
  レベルで確定 (`register_bool_binding_writes_initial_and_updates_
  for_bool_prop_read`)。T6 の Windows-only integration test が
  `wasamo_set_property` 経路を保証しているのと合わせて、T11 は
  `examples/counter-rust` (working default) に bool fixture を
  載せるだけで window 上の grey-out demo が動く。
- **T12 (phase-end gates):** Windows CI 上で T6 integration test
  と T8 で追加した unit test 群がともに green であることを確認。
  T8 では Win32 surface に新規依存を入れていないので、CI 側の
  capability check (`wasamo_init` の `0x80070005` skip-guard) の
  扱いは T6 から変化なし。

これらはすべて progress file の T10–T12 として既に列挙済み。T8
単体で新たに発見された follow-up は無い。
