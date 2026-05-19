---
title: M3-Phase 1 / T7 step-end retrospective
status: recorded
created: 2026-05-19
scope: step-end
task: T7 — EvalContext bool trait surface and handler evaluator arm
---

# M3-Phase 1 / T7 step-end retrospective

## Scope

`docs/plans/progress/m3-phase-1-progress.md` の **T7**
("`EvalContext` bool trait surface and handler evaluator arm") の
step-end retrospective。T7 が discharge する DD は
DD-M3-P1-004 (Option B — full `get_bool` + `read_bool_tracked` +
`set_bool` trait surface) と DD-M3-P1-008 (Option A — pair flip:
handler-side bool writes admitted in Phase 1)。

対象コミット:

- `46546a1 feat(wasamo-runtime): EvalContext bool surface + bool Assign arm (M3-Phase 1 T7, part 1)`
- `6d9217e feat(wasamo-runtime): bool arms on BindingEvalContext / HandlerEvalContext (M3-Phase 1 T7, part 2)`

これは step-end の gate であり、phase-end retrospective ではない。

## Current Judgment

2026-05-19 時点で T7 step-end 基準は **達成済み**。

- `EvalContext` trait
  ([wasamo-runtime/src/handler.rs](../../../wasamo-runtime/src/handler.rs))
  に以下 3 method を追加:
  - `get_bool(&self, path) -> Result<bool, EvalError>` —
    default impl は `UnknownProperty` 返却。
  - `read_bool_tracked(&self, path) -> Result<bool, EvalError>` —
    default impl は `get_bool` への forward。
  - `set_bool(&mut self, path, value: bool) -> Result<(), EvalError>` —
    default impl は `UnknownProperty` 返却。
  - default impl は M2 の `String` 系 (`get_string` /
    `read_string_tracked`) と同じ「すべて defaulted」shape を採用。
    M2 i32 系のように `get_i32` / `set_i32` を required にすると
    既存実装 (例: `widget::NullEvalContext`, テスト fixture) が
    軒並みコンパイル不能になるため、後方互換を優先。
- `evaluate()` の `Assign` arm を `rhs` でディスパッチする形に拡張:
  - `rhs == HandlerExpr::BoolLit(b)` → `ctx.set_bool(lhs, b)` →
    `Ok(0)`
  - `rhs == HandlerExpr::BoolPropRead { path }` →
    `ctx.get_bool(path)` → `ctx.set_bool(lhs, v)` → `Ok(0)`
  - それ以外 → 既存 i32 経路 (`evaluate(rhs)` → `set_i32`)
  - 返り値 `0` は「side-effect only」の表現。bool→i32 coercion を
    導入しない (DD-M3-P1-001 Option B は明示却下) ので、整数返却を
    満たすための無害値として `0` を選んだ。`Block` の "last value"
    semantics でも空ブロックの default が `0` なので衝突しない。
- bare `BoolLit` / `BoolPropRead` (i.e. `Assign` で wrap されない
  形) は引き続き `TypeMismatch`。`CompoundAssign` の `rhs` に
  bool 式を置いたケースも、`rhs` を `evaluate(rhs, ctx)` で評価する
  際にこの arm が踏まれて `TypeMismatch` を返す ⇒ compound-bool が
  silently 通らないことが保証される。
- `BindingEvalContext`
  ([wasamo-runtime/src/reactive.rs](../../../wasamo-runtime/src/reactive.rs))
  に `get_bool` (untracked) / `read_bool_tracked` (tracked) /
  `set_bool` (returns `WriteInBindingContext`) を追加。binding は
  read-only contract (DD-M2-P5-006 = A) を踏襲。
- `HandlerEvalContext` に `get_bool` (untracked) と `set_bool`
  (registry.bools の `Signal::set` を駆動) を追加。これにより
  `Assign { rhs: BoolLit }` が live runtime 上で `Signal<bool>` の
  reactive cascade を発火する経路が完成。
- Unit tests:
  - handler.rs に 10 件追加 (trait default 3、Assign arm 4、bare
    bool reject 2、compound bool reject 1、i32 regression 1)。
  - reactive.rs に 4 件追加 (binding ctx の tracked/untracked、
    binding ctx の write rejection、handler ctx 経由の Signal
    cascade 起動、handler ctx の unknown path)。
- `cargo build --release --workspace` / `cargo test --workspace`
  いずれも local で green。
  - `wasamo-runtime` lib テスト: 132 → 146 件 (handler に 10 件、
    reactive に 4 件、計 14 件追加)。146 件はざっくり handler 46 +
    reactive 37 + その他 lib モジュール 63 件相当。
  - 他 crate / integration test の件数変化なし、全件 green を維持。

T7 の blocker は残っていない。

## Main Learning

中心的な学びは、**`EvalContext` の bool default impl shape は
"i32 mirror" ではなく "String mirror" を採るのが正しい** こと。

- progress file の T7 文言は "default impls mirroring the M2 i32
  shape" と書いていた。だが M2 i32 は厳密には `get_i32` /
  `set_i32` が required で、`read_i32_tracked` だけが default を
  持つ。これをそのまま bool に当てると `get_bool` / `set_bool` が
  required になり、既存 EvalContext impl (NullEvalContext、各
  unit test の MapCtx 派生など) を全部広げる作業が発生する。
  しかも MapCtx は **handler.rs 内のテストの中だけで bool field を
  必要とする** ので、required にすると外側のテスト fixture も
  巻き込む副作用が出る。
- 実際の M2 String 系 (`get_string` / `read_string_tracked`) は
  全部 defaulted で `UnknownProperty` / 自分への forward を返す。
  bool は string と同じ "後から追加された型" 属性を持つので、
  default を全部入れて新規実装で override する方が trait の
  進化方針として一貫している。progress file の "i32 shape" 表現は
  「読み・書きの method pair が揃っている」という意味で書かれて
  いたが、default の所在まで mirror すると後方互換が崩れる、と
  いう罠を踏みかけて回避した。
- 結果として、progress file の T7 entry の文言を "M2 String shape"
  に修正し、本 retrospective の Current Judgment にも明示した。
  Phase 後の T10 (spec sync) で `architecture.md` の EvalContext
  trait 説明に同じ判断理由を残す予定 (Phase 1 ADR の DD-M3-P1-004
  は trait の "存在" を確定しており、default の有無は明示してい
  ない — そこは実装の裁量範囲)。

副次的な学び:

- **`Assign` 評価器を `rhs` でディスパッチする shape は単純だが
  forward-compat 上の含意がある**。今は bool/i32 の 2 経路だが、
  M3-Phase 5 で f32 や追加スカラを足すと同じ `match rhs` が分岐
  3 経路目を持つ。`HandlerExpr::Assign` の `rhs` 型が `Box<HandlerExpr>`
  である限り、ディスパッチは call site でしか起こせず、これは
  DD-M3-P1-007 の per-type seam 思想 (F5 deferral の構造的
  enforcement) と整合する。`TypedValue` を導入したら `Assign` の
  shape も変わるので、その時にまとめて refactor すれば良く、現状
  は最小拡張で十分。
- **bool-typed `Assign` arm の return value は意図的に `Ok(0)`
  にした**。`HandlerExpr::Block` の評価器は `last = evaluate(stmt,
  ctx)?` で各 statement の i32 を蓄積する。bool assign が
  `Block` の最後にあると last value も 0 になる ⇒ 「ブロックが
  bool 操作で終わった = 0 を返す」という解釈は、整数文脈で副作用
  だけが意味を持つ標準的な解釈と一致する。
- **`HandlerEvalContext::set_bool` の test** は単に `set_bool` を
  直接呼ぶのではなく `evaluate(&Assign{rhs: BoolLit(false)}, ctx)`
  経由で駆動した。これにより T7 の 2 つのコミット (trait surface と
  live wiring) が end-to-end で繋がることを 1 つの test で
  保証できる。`HandlerEvalContext` 単体テストだけだと `evaluate()`
  の bool arm が `set_bool` を呼ぶ経路を hit しないので不十分。

## Checklist

1. **本作業の主要な学び:** あり。
   - default impl の所在は trait の進化方針として M2 String 系を
     mirror するのが正しい、という観測。progress file 文言の
     "M2 i32 shape" は「読み・書きの method pair が揃っている」
     の意味であって「required/defaulted の所在を mirror する」
     意味ではなかった、と再解釈して本 retrospective に記録。
     ADR レベルの追加 DD は不要 — DD-M3-P1-004 は trait surface の
     "存在" を確定するもので、default の所在は実装裁量。

2. **仕様文書 (`abi_spec.md` / `architecture.md` / `dsl_spec.md`)
   の変更:** **なし (T10 で扱う)**
   - `EvalContext::*_bool`、`BindingEvalContext` / `HandlerEvalContext`
     の bool 実装、`evaluate()` の bool-typed `Assign` arm の文書化は
     T10 (A11 = per-phase spec sync) で実施。
   - `architecture.md` §6 の binding evaluator / signal registry
     scnippet に "M2 supports `i32` and `String` Signals" を `+ bool`
     に拡張するのも T10 で扱う。Phase 1 ADR の Spec impact preview
     に既に列挙済み。

3. **ローカル clean rebuild:** **green**
   - `cargo build --release --workspace`: green
   - `cargo test --workspace`: 全件 green
     - `wasamo-runtime` lib: 146 件 (T6 完了時 132 件 + handler 10
       件 + reactive 4 件 = 146 件で計算一致)
     - 他 integration / 他 crate: 件数変化なしで green を維持
   - 途中、`cargo test --workspace` 1 回目で `wasamo.dll.lib`
     未生成のリンクエラー (counter-rust 経由) を踏んだが、
     `cargo build -p wasamo-dll` で `wasamo.dll.lib` を生成してから
     再実行で green。これは [docs/architecture.md §1 "DSL build
     pipeline"](../../architecture.md#dsl-build-pipeline-m2-phase-6-onward)
     に書かれた既知の build ordering 制約 (workspace test の
     冷起動時にだけ顕在化) で、T7 固有の問題ではない。
   - GitHub Actions 上の clean rebuild は phase-end gate (T12) で
     確認する。

4. **PO に相談すべき設計判断・トレードオフ:** **なし**
   - DD-M3-P1-004 (Option B) と DD-M3-P1-008 (Option A) が trait の
     必要 method を確定済み。
   - 実装裁量は (a) default impl の所在 (上記 §Main Learning の
     "String shape" 採用)、(b) bool-typed `Assign` arm の返り値を
     `Ok(0)` にする、の 2 点だが、いずれも既存 M2 パターンに
     沿った最も保守的な解釈で、PO 相談を要する設計判断ではない。

### step-end 固有

5. **plan/ADR に記載の step 目的から外れた「ついで」のリファクタ・
   構造変更:** **なし**
   - 既存 `Assign` arm の構造を `match rhs.as_ref() { ... }` で
     包んだのは bool 経路追加の最小修正で、i32 経路の挙動は変更
     していない (テスト `assign_i32_lit_still_works_after_bool_arm`
     で regression guard 済み)。リファクタとは呼べない。
   - test 用 `MapCtx` への `bools` HashMap 追加・`with_bools` /
     `get_b` ヘルパ追加も bool テストのための最小拡張のみ。

6. **現在の phase ADR への追加 DD 必要性:** **なし**
   - 既存 DD-M3-P1-004 / DD-M3-P1-008 で T7 範囲は完全にカバー。

7. **既存 ADR の Proposed 項目の新規追加、または Proposed → Accepted
   への昇格:** **なし**
   - 当該 ADR は全 DD Accepted 済み。

8. **`m3-plan.md` の AC 追加・変更、または Phase 構成の追加・統合・
   分割:** **なし**
   - A9 / A11 / A12 等の文言は変更なし。Phase 構成変更なし。

9. **後続 step に持ち越す仮実装・近似・新規 `dead_code` 警告:**
   **持ち越しなし、`dead_code` 警告なし**
   - `EvalContext::get_bool` / `set_bool` の default impl は
     "後方互換のための薄い shim" だが、これは仮実装ではなく
     trait 進化方針上の意図的設計。T8 以降で `evaluate_bool_binding`
     が `read_bool_tracked` を呼ぶようになっても default は
     正しく forward する。
   - bool-typed `Assign` arm の `Ok(0)` 返却は意図的選択で、
     仮実装ではない。

10. **タスクリストの後続 step 見直し:** **T7 の status 更新のみ**
    - progress file の T7 行を `[x]` に更新し、acceptance bullet を
      実装に合わせて細分化、Notes に default impl の "String shape"
      採用理由と `Ok(0)` 返却の理由を追記。
    - T8 (binding evaluator + per-type writer seam): T6 / T7 で
      `SignalRegistry::bools` と `EvalContext::*_bool` が出揃った
      ので、T8 は `evaluate_bool_binding`、`widget_write_property_bool`、
      binding loader の `IrType` ディスパッチを追加するだけで
      `Button.enabled` が `bind enabled: ready` 経由で動く状態。
    - T10–T12 の構成・順序に変更なし。

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

T7 で追加したテストと、走らせた command を記録する。

新規 unit tests (handler.rs `tests` モジュール):

| Test | What it asserts |
|---|---|
| `eval_context_default_get_bool_is_unknown` | default `get_bool` / `read_bool_tracked` が `UnknownProperty` を返す |
| `eval_context_default_set_bool_is_unknown` | default `set_bool` が `UnknownProperty` を返す |
| `read_bool_tracked_default_forwards_to_get_bool` | default `read_bool_tracked` は `get_bool` 経由で値を返す |
| `assign_bool_lit_writes_through_set_bool` | `Assign { rhs: BoolLit(false) }` が `set_bool` 経由で書く、返り値は `Ok(0)` |
| `assign_bool_prop_read_copies_value` | `Assign { rhs: BoolPropRead { path: "other" } }` が source の bool を target に複写、source 側は不変 |
| `assign_bool_prop_read_unknown_source_propagates_error` | unknown path に対する `BoolPropRead` が `UnknownProperty` を返し、target は不変 |
| `invoke_handler_drives_bool_assign` | `invoke_handler` 経由 (inline click handler の実経路) で bool assign が動く |
| `evaluate_rejects_bare_bool_lit_in_handler_context` | bare `BoolLit` は handler context で `TypeMismatch` |
| `evaluate_rejects_bare_bool_prop_read_in_handler_context` | bare `BoolPropRead` は handler context で `TypeMismatch` |
| `evaluate_rejects_compound_assign_with_bool_rhs` | `CompoundAssign { rhs: BoolLit }` は `TypeMismatch` で target 不変 |
| `assign_i32_lit_still_works_after_bool_arm` | i32 `Assign` 経路の regression guard |

新規 unit tests (reactive.rs `tests` モジュール):

| Test | What it asserts |
|---|---|
| `binding_ctx_get_bool_untracked_vs_tracked` | `get_bool` は dep を登録しない、`read_bool_tracked` は登録する |
| `binding_ctx_set_bool_returns_write_error` | binding context で `set_bool` を呼ぶと `WriteInBindingContext` |
| `handler_ctx_set_bool_drives_signal_set` | `evaluate(&Assign{rhs: BoolLit(false)}, HandlerEvalContext)` が `Signal<bool>::set` を駆動し、依存 Effect が再走 |
| `handler_ctx_set_bool_unknown_path_errors` | 未登録パスへの `set_bool` は `UnknownProperty` |

T7 acceptance との対応:

| T7 checklist | Coverage |
|---|---|
| `EvalContext::get_bool` + `read_bool_tracked` + `set_bool` (default impls) | 46546a1 (part 1) `handler.rs` `EvalContext` trait |
| `EvalContext` 実装 (BindingEvalContext / HandlerEvalContext) | 6d9217e (part 2) `reactive.rs` |
| `evaluate()` Assign arm が `rhs` で bool/i32 をディスパッチ | 46546a1 (part 1) `handler.rs` `evaluate` |
| `CompoundAssign` over bool 等の他 bool 構文は reject | 46546a1 (part 1) `evaluate_rejects_compound_assign_with_bool_rhs` テスト |
| trait default の unit test | 46546a1 (part 1) `eval_context_default_*` テスト |
| 新規 Assign arm の unit test | 46546a1 (part 1) `assign_bool_*` / `invoke_handler_drives_bool_assign` テスト |
| 既存 i32 path の regression guard | 46546a1 (part 1) `assign_i32_lit_still_works_after_bool_arm` テスト |
| live BindingEvalContext / HandlerEvalContext の test | 6d9217e (part 2) `binding_ctx_*_bool*` / `handler_ctx_set_bool_*` テスト |

実行コマンド:

```text
cargo build -p wasamo-runtime
cargo test -p wasamo-runtime --lib handler::
cargo test -p wasamo-runtime --lib reactive::
cargo build --release --workspace
cargo build -p wasamo-dll
cargo test --workspace
```

いずれも green。`wasamo-runtime` lib の総 test 件数は T6 完了時の
132 件から 146 件に増加 (handler に 10 件、reactive に 4 件)。

## Follow-Up

T7 から後続 task への明示的な引き渡し:

- **T8 (binding evaluator + per-type writer seam):** T7 で
  `BindingEvalContext::read_bool_tracked` / `get_bool` が出揃ったの
  で、T8 の `evaluate_bool_binding(expr, ctx) -> Result<bool,
  EvalError>` は `BoolLit` / `BoolPropRead` の 2 variant を受けて
  それぞれ値返却 / `ctx.read_bool_tracked(path)` を呼ぶだけの形
  になる。`widget_write_property_bool(id, prop, value: bool)` の
  追加と binding loader の `IrType::Bool` 分岐 (T6 で
  `_prop_ty` として hold 済みのタグを consume) で `Button.enabled`
  の `bind enabled: ready` 経路が完成する。
- **T10 (spec sync):** `EvalContext::*_bool` の trait surface、
  `evaluate()` の bool `Assign` arm、`BindingEvalContext` /
  `HandlerEvalContext` の bool 実装を `architecture.md` の
  該当 §§ に追記。default impl が "String mirror" を採った理由は
  軽く触れて、reader が trait 進化方針を読み取れるようにする。
- **T11 (.ui fixture + host evidence):** T7+T8 で
  `state ready: bool = true; Button { enabled: ready; on click {
  ready = false } }` の end-to-end が動くようになる。T7 の
  `handler_ctx_set_bool_drives_signal_set` test は単体テスト
  レベルで同じ経路の生存確認を済ませている。

これらはすべて progress file の T8–T12 として既に列挙済み。T7 単体
で新たに発見された follow-up は無い。
