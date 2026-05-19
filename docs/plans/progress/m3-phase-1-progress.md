---
phase: M3-Phase 1
title: bool scalar binding
status: active
adr: docs/decisions/m3-phase-1-bool-scalar.md
plan: docs/plans/m3-plan.md
opened: 2026-05-19
---

# M3-Phase 1 — `bool` scalar binding: Progress

This is the live task list and execution log for M3-Phase 1. The
design decisions are frozen in
[m3-phase-1-bool-scalar.md](../../decisions/m3-phase-1-bool-scalar.md);
this file is mutable per
[plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle).

Task ordering follows the dependency direction
`wasamo-ir → wasamoc → wasamo-runtime → tests → host/spec`, so each
commit builds on a green workspace per
[CLAUDE.md §Commit rules](../../../CLAUDE.md). Items may be split,
reordered, or merged when implementation reveals a tighter ordering
— this list is the record of what actually happens, not a frozen
prediction.

## Task list

### T1 — `wasamo-ir`: add `bool` to the type / literal / handler surfaces

Discharges DD-M3-P1-001, DD-M3-P1-002 (IR variant half),
DD-M3-P1-003. **Landed in 7cc52f4 (2026-05-19).**

- [x] `IrType::Bool` variant added; every existing `match` on
      `IrType` in the workspace gains a `Bool` arm (or an explicit
      reject) so the compiler enforces completeness.
- [x] `IrLiteral::Bool(bool)` variant added; parallel to `Int` /
      `Str`.
- [x] `HandlerExpr::BoolLit(bool)` and `HandlerExpr::BoolPropRead {
      path }` added; existing `IntLit` / `PropRead` left
      i32-implicit (no rename per DD-M3-P1-003 Option A).
- [x] Pure-logic unit tests in `wasamo-ir` covering construction
      and equality of the new variants.

Retrospective:
[docs/notes/m3-phase-1/t1-step-end-retrospective.md](../../notes/m3-phase-1/t1-step-end-retrospective.md).

### T2 — `wasamoc` lexer / parser: `true` / `false` keywords and bool literal

Discharges DD-M3-P1-002 surface-syntax half, DD-M3-P1-006 (IR text
spelling). **Landed in 992e7e1 (2026-05-19).**

- [x] Lexer recognises `true` and `false` as reserved keywords;
      reservation is a hard error if used as an identifier.
- [x] Parser produces `IrLiteral::Bool(true|false)` for surface
      bool literals (via `Expr::BoolLit` AST node + `lower_expr` /
      `lower_rhs_expr` / `lower_state` bool arms).
- [x] `wasamoc` IR text emitter writes bool literals as `true` /
      `false` and emits `BoolLit` / `BoolPropRead` productions
      verbatim (emit arms were wired mechanically in T1 per
      DD-M3-P1-006; T2 adds end-to-end coverage in
      `wasamoc::emit` tests).
- [x] Unit tests: parse `state ready: bool = false`; parse
      `Button { enabled: true }`; reject `true` / `false` as state
      names and as property-bind LHS.

Retrospective:
[docs/notes/m3-phase-1/t2-step-end-retrospective.md](../../notes/m3-phase-1/t2-step-end-retrospective.md).

### T3 — `wasamoc` checker: state-type table and bool type-checking

Discharges DD-M3-P1-010. **Landed in 710eea8 + 3cbe257 (2026-05-19).**

- [x] Parse-time `HashMap<String, TypeName>` populated from `state`
      declarations (already present pre-T3 as
      `check::Namespace = HashMap<String, TypeName>`; T3 reuses it
      via `expr_static_type` for ident resolution against declared
      types).
- [x] `check_members` widened to carry enclosing widget context
      (`check_members_inner` with `enclosing_widget: Option<&str>`)
      so `bind` LHS type-checking sees the target property's
      declared `TypeName` via the new `widget_prop_type` catalog.
- [x] Accept / reject rules from DD-M3-P1-010's table implemented
      as compile-time diagnostics with line/column:
  - accept: `state ready: bool = false`,
    `bind enabled: ready` (bool/bool), `bind enabled: true`.
  - reject: `state ready: bool = 0`, `state ready: bool =
    "false"`, `bind enabled: 1`, `bind text: true` (string
    target), `bind text: ready` (bool source / string target),
    `state x: i32 = 5; bind enabled: x`.
- [x] Unit tests cover every row of DD-M3-P1-010's table.

Retrospective:
[docs/notes/m3-phase-1/t3-step-end-retrospective.md](../../notes/m3-phase-1/t3-step-end-retrospective.md).

Notes:

- DD-M3-P1-010's example uses an abstract `bind label: <…>` target
  to illustrate a `String`-typed property. T3 implements the same
  rejection on the concrete catalog entry `Text.text: string` (the
  string-typed property that actually exists in M2). The mismatch
  shape and diagnostic structure are identical.
- The widget-property catalog in `wasamoc::check` is intentionally
  soft and Phase-1-minimal: only `Text.text`, `Button.text`, and
  `Button.enabled` are listed. Properties handled by ident keyword
  values (`Button.style: accent`, `Text.font: title`) and
  component-level binds (`title: "…"`, `backdrop: mica`) pass
  through with no type-check, preserving M2 patterns. The
  `wasamoc` catalog mirrors `wasamo-runtime`'s `resolve_prop_key`
  table (DD-M3-P1-009) but lives independently in the compiler so
  `wasamoc check` is self-contained.

### T4 — `wasamoc` lowering: identifier → typed `*PropRead`

Discharges DD-M3-P1-003 / DD-M3-P1-010 interaction (typed
lowering). **Landed in 5a5ba28 (2026-05-19).**

- [x] Identifier lowering consults the state-type table: `bool`
      state name → `BoolPropRead`; `i32` → `PropRead`; `String` →
      `StrPropRead`. Applied uniformly to (1) `lower_expr` for
      prop-bind RHS, (2) `lower_rhs_expr` for handler RHS (with
      `&Namespace` now threaded through `lower_block` /
      `lower_statement`), and (3) `lower_string_parts` for string-
      interpolation parts.
- [x] Unit test asserts lowering of `bind enabled: ready` for
      `state ready: bool = …` emits `BoolPropRead { path:
      "ready" }`. Six further tests cover the other three ident-
      resolution outcomes (i32 / string state → `PropRead` /
      `StrPropRead`; non-state keyword stays as
      `IrLiteral::Ident`) plus handler-RHS and string-interp
      paths.

Retrospective:
[docs/notes/m3-phase-1/t4-step-end-retrospective.md](../../notes/m3-phase-1/t4-step-end-retrospective.md).

Notes:

- The non-state ident pass-through (`theme: system`, `style:
  accent`, `font: title`, `backdrop: mica`) is what keeps the
  M2-era `.ui` corpus lowering unchanged under T4 — only idents
  that the namespace identifies as `state` become reactive
  bindings; everything else stays a static `IrLiteral::Ident`.
  This mirrors the soft-catalog discipline T3 chose on the
  checker side.
- Float-typed state idents (if any ever appear) fall through to
  the static-ident branch alongside non-state idents, because
  Phase 1 has no `*PropRead` variant for `f32` / `f64` and the
  checker rejects float earlier; the lower-side fallback is
  defensive only.

### T5 — `wasamo-runtime` IR loader: read new productions

Discharges the IR-text-load half of DD-M3-P1-006. **Landed in
6dbbbba + ef6e3a0 (2026-05-19).**

- [x] IR text loader accepts the new `IrType` / `IrLiteral` /
      `HandlerExpr` productions: `parse_state` resolves `"bool"`,
      `parse_literal` lowers bare `true` / `false` idents to
      `IrLiteral::Bool`, `parse_expr` recognises `true` / `false`
      in expression position as `HandlerExpr::BoolLit`, and
      `parse_sexpr` gains a `bool-prop-read` tag arm. Defense-in-
      depth validation (DD-M2-P6-009) covered `BoolLit` /
      `BoolPropRead` from T1.
- [x] Round-trip test (`wasamoc` emit → `wasamo-runtime` load)
      reconstructs `IrState { ty: Bool, default: Bool(false) }` and
      `HandlerExpr::BoolPropRead { path: "ready" }`. Lives as
      `bool_state_binding_emits_and_parses_bool_productions` in
      [wasamo-runtime/tests/ir_loader_roundtrip.rs](../../../wasamo-runtime/tests/ir_loader_roundtrip.rs)
      alongside the existing i32 / string round-trips.

Retrospective:
[docs/notes/m3-phase-1/t5-step-end-retrospective.md](../../notes/m3-phase-1/t5-step-end-retrospective.md).

Notes:

- `build_signal_registry`'s `IrType::Bool` arm still raises
  `unimplemented!()`. T5 only wires the parser; the build seam
  depends on `SignalRegistry::bools` and `Signal<bool>`, which
  arrive in T6 (widget catalog + `PropertyValue::Bool`) and T7
  (`EvalContext` bool surface). The arm's comment is updated to
  reflect the new sequencing.
- The cross-crate integration test in T5 part 2 piggybacks on the
  real T2 / T3 / T4 surfaces (lexer, checker, lower), so it
  doubles as a regression guard for the seam between
  `wasamoc::emit` and `parse_ir`. Any future divergence in IR
  text spelling (e.g. a tag rename) will fail it.

### T6 — `wasamo-runtime` widget catalog: `PropertyValue::Bool`, typed `resolve_prop_key`, `Button.enabled`

Discharges DD-M3-P1-005, DD-M3-P1-009. Also folds in T9 (C ABI
value-conversion arms) — see Notes. **Landed in a550bd9 + 36be13c +
cf8467a (2026-05-19).**

- [x] `PropertyValue::Bool(bool)` variant added in
      [widget.rs L77-L81](../../../wasamo-runtime/src/widget.rs#L77-L81).
- [x] `resolve_prop_key` widened to return `Option<(PropertyKey,
      IrType)>`; widget catalog rows carry `IrType` (M2 rows
      `Text.text`/`Text.font`/`Button.text`/`Button.style` retain
      their declared `Str`/`I32`/`Str`/`I32` types; new
      `Button.enabled` row carries `IrType::Bool`). The `IrType`
      tag is plumbed through the binding loader (`_prop_ty`) but
      not yet consumed — the per-type writer dispatch lands in T8.
- [x] `PROP_BUTTON_ENABLED = 5` u32 id added (next free slot).
- [x] `Button` widget setter dispatches `PROP_BUTTON_ENABLED` to
      `update_button_enabled`; `ButtonData.enabled` defaults to
      `true` at `WidgetNode::button` construction.
- [x] Phase 1 `Button.enabled` runtime contract:
      `hit_test_click_inner` guards Button dispatch on
      `btn.enabled`, skipping host callback / inline `clicked`
      handler / `enqueue_signal("clicked", …)` while preserving
      child hit-test traversal; `update_hover_inner` freezes
      hover/press transitions when disabled; `effective_button_color`
      paints the documented flat grey
      `(A=0x40, R=G=B=0x80)` directly (no
      `ColorKeyFrameAnimation`); layout slot is preserved (no
      `display: none` semantics). Focus / a11y / keyboard
      activation deferred to M4–M5 per ADR §Out of scope.
- [x] Mock-free Windows-only integration test
      `button_enabled_property_flips_visual_and_suppresses_click`
      in [wasamo-runtime/tests/button_enabled.rs](../../../wasamo-runtime/tests/button_enabled.rs);
      drives `wasamo_set_property(PROP_BUTTON_ENABLED,
      WASAMO_VALUE_BOOL)` and asserts the
      `CompositionColorBrush::Color()` flip plus click-callback
      suppression. Fails (not skips) on GitHub Actions when the
      runtime Compositor is unavailable.

Retrospective:
[docs/notes/m3-phase-1/t6-step-end-retrospective.md](../../notes/m3-phase-1/t6-step-end-retrospective.md).

Notes:

- **T9 fold:** Adding `PropertyValue::Bool` mechanically required
  exhaustive `match` arms in `abi::read_property_value`,
  `abi::write_property_value`, `abi::property_value_to_owned`, and
  `emit::owned_to_value` (the latter through a new `OwnedArg::Bool`
  variant). Rather than landing panic stubs, T9's planned
  ABI-side bool plumbing was completed in T6 part 1 (commit
  a550bd9). T9 below is marked `[x]` with this commit reference;
  no independent T9 commit will land. This is the
  "implementation reveals a tighter ordering" path permitted by
  [CLAUDE.md §Commit rules](../../../CLAUDE.md).
- **`SignalRegistry::bools` landed in T6 (part 2, commit
  36be13c)** even though the strict T6 acceptance bullets don't
  mention it. The bool registry is the natural pairing for the
  widget-side `PropertyValue::Bool` work and allows T7 to focus
  purely on the `EvalContext` trait surface without simultaneously
  introducing the data store. `build_signal_registry`'s
  `IrType::Bool` arm now resolves `Signal::new(default)` instead
  of T5's `unimplemented!()`.
- **Test sizing quirk:** `WidgetNode::button` does not size its
  background `SpriteVisual` (that is the host window's
  `layout::arrange` responsibility). The Windows integration test
  therefore calls `button.visual.SetSize(...)` directly so
  `hit_test_click` has a non-empty rect; T11's `.ui` fixture-based
  test will not need this because it runs through the window
  path.

### T7 — `EvalContext` bool trait surface and handler evaluator arm

Discharges DD-M3-P1-004 (Option B), DD-M3-P1-008 (Option A — pair
flip). **Landed in 46546a1 + 6d9217e (2026-05-19).**

- [x] `EvalContext::get_bool`, `EvalContext::read_bool_tracked`,
      `EvalContext::set_bool` added with default impls mirroring
      the M2 String shape (`UnknownProperty` for `get_bool` /
      `set_bool`; `read_bool_tracked` forwards to `get_bool`). Live
      impls follow in
      [reactive.rs](../../../wasamo-runtime/src/reactive.rs):
      `BindingEvalContext` reads via `registry.bools[path]` and
      rejects writes with `WriteInBindingContext`;
      `HandlerEvalContext` reads via `get_untracked()` and writes via
      `Signal::set`.
- [x] `evaluate()` peeks at `Assign`'s `rhs` and dispatches to
      `set_bool` when `rhs ∈ { BoolLit, BoolPropRead }`. Other
      variants stay on the existing i32 path; bare `BoolLit` /
      `BoolPropRead` and `CompoundAssign` with bool rhs continue to
      return `EvalError::TypeMismatch` (no implicit bool→i32
      coercion; no compound-bool semantics — both out of scope per
      ADR).
- [x] Unit tests cover the trait defaults
      (`eval_context_default_get_bool_is_unknown`,
      `eval_context_default_set_bool_is_unknown`,
      `read_bool_tracked_default_forwards_to_get_bool`), the new
      `Assign` arm on both `BoolLit` and `BoolPropRead`
      (`assign_bool_lit_writes_through_set_bool`,
      `assign_bool_prop_read_copies_value`,
      `assign_bool_prop_read_unknown_source_propagates_error`,
      `invoke_handler_drives_bool_assign`), the rejection of bare
      bool expressions and `CompoundAssign` with bool rhs
      (`evaluate_rejects_bare_bool_lit_in_handler_context`,
      `evaluate_rejects_bare_bool_prop_read_in_handler_context`,
      `evaluate_rejects_compound_assign_with_bool_rhs`), the i32
      regression guard (`assign_i32_lit_still_works_after_bool_arm`),
      and the live `BindingEvalContext` / `HandlerEvalContext`
      surfaces (`binding_ctx_get_bool_untracked_vs_tracked`,
      `binding_ctx_set_bool_returns_write_error`,
      `handler_ctx_set_bool_drives_signal_set`,
      `handler_ctx_set_bool_unknown_path_errors`).

Retrospective:
[docs/notes/m3-phase-1/t7-step-end-retrospective.md](../../notes/m3-phase-1/t7-step-end-retrospective.md).

Notes:

- The default impls follow the M2 String shape (`get_string` /
  `read_string_tracked` both defaulted) rather than the i32 shape
  (`get_i32` / `set_i32` required). This keeps existing
  `EvalContext` impls (e.g. `widget::NullEvalContext`, test
  fixtures) compiling without changes — they pick up the
  `UnknownProperty` defaults until they opt in to overriding.
- The bool-typed `Assign` arm returns `Ok(0)` to satisfy
  `evaluate()`'s `Result<i32, _>` contract without implicit
  bool→i32 coercion. Handlers run for side effects; the integer
  return value is only consumed by `Block`'s "last expression"
  semantics, where `0` is already the empty-block default.

### T8 — Binding evaluator and per-type writer seam

Discharges DD-M3-P1-007.

- [ ] `evaluate_bool_binding(expr, ctx) -> Result<bool,
      EvalError>` added in
      [handler.rs](../../../wasamo-runtime/src/handler.rs); accepts
      `BoolLit` / `BoolPropRead`, rejects all other variants with
      `EvalError::TypeMismatch`.
- [ ] `widget_write_property_bool(id, prop, value: bool)` added in
      [widget.rs](../../../wasamo-runtime/src/widget.rs);
      constructs `PropertyValue::Bool(bool)` and dispatches to the
      per-widget setter.
- [ ] Binding loader picks the bool writer when
      `resolve_prop_key` returns `IrType::Bool`; string writer
      otherwise. The reactive engine's `write_fn` parameter stays
      type-agnostic from the engine's perspective; the seam is at
      the loader call site
      ([architecture.md L714](../../architecture.md#L714)).
- [ ] Unit tests cover dispatch selection for bool and string
      target properties.

### T9 — C ABI value-conversion arms (no new functions)

Discharges the abi-side spec impact recorded in ADR §Spec impact
preview. **Folded into T6 part 1 (commit a550bd9, 2026-05-19).**

- [x] `read_property_value` / `write_property_value` /
      `property_value_to_owned` in
      [abi.rs](../../../wasamo-runtime/src/abi.rs) gained bool arms
      threading `PropertyValue::Bool(bool)` ↔ `WasamoValue::v_bool`
      (existing `WASAMO_VALUE_BOOL = 3` tag).
- [x] Property-observer payload conversion carries bool through:
      `OwnedArg::Bool(bool)` added in
      [emit.rs](../../../wasamo-runtime/src/emit.rs); `owned_to_value`
      builds the `WASAMO_VALUE_BOOL`-tagged `WasamoValue`.
- [x] No new public ABI functions added (DD-M3-P1-008 Option B
      explicitly deferred to its own future ADR).

See T6 Notes for why this landed inside T6.

### T10 — Spec / architecture documentation update (A11)

Discharges the per-phase spec sync acceptance criterion.

- [ ] [docs/dsl_spec.md](../../dsl_spec.md): §2.1 `true` / `false`
      keyword reservation; §2 bool literal token; §4.2 `bool` state
      declarations; §4.3 property binding with bool; §4.6
      expression grammar (`BoolLit`, `BoolPropRead`, bool `Assign`
      arm; `CompoundAssign` bool exclusion noted); §8 IR text
      grammar (`IrType` += `bool`; `BoolLit`; `BoolPropRead`);
      widget catalog entry for `Button.enabled` with the narrow
      Phase 1 contract.
- [ ] [docs/architecture.md](../../architecture.md): §6
      SignalRegistry snippet around
      [L717-L744](../../architecture.md#L717-L744) updated to
      include `bools: HashMap<String, Signal<bool>>`; prose extends
      "M2 supports `i32` and `String` Signals" to include `bool`
      and notes `HandlerExpr::BoolPropRead` evaluates through
      `BindingEvalContext::read_bool_tracked`. Binding write-seam
      description around
      [L714](../../architecture.md#L714) updated to describe the
      per-type `write_fn` selection from DD-M3-P1-007.
- [ ] F5 (`TypedValue`) deferral cross-reference preserved in both
      docs.
- [ ] External-implementor smoke check on the spec edits: the
      bool-specific additions are sufficient for a reader to
      reproduce the Phase 1 surface against a hypothetical host
      (Phase 8 bar applied at phase-end).

### T11 — `.ui` fixture and end-to-end host evidence

Discharges Phase 1 verification closure item (4) from the ADR.

- [ ] A `.ui` fixture declares `state ready: bool = true; Button {
      enabled: ready; on click { ready = false } }` (or
      equivalent) and lives where the chosen host can load it.
- [ ] Working default: extend `examples/counter-rust` or add a
      minimal `examples/bool-demo-rust/` host. Final choice
      recorded below in §Decisions log when execution lands.
- [ ] Host launches to a visible window where clicking the button
      visibly greys it.

### T12 — Phase-end gates

Discharges the m3-plan §Phase-end criteria checklist.

- [ ] `cargo build --release --workspace` and
      `cargo test --workspace` green locally and on GitHub Actions
      CI; CI run link recorded below.
- [ ] Windows-only mock-free integration test from T6 passes on
      CI (fails — not skips — if Compositor capability missing).
- [ ] Spec & architecture edits from T10 reviewed for
      external-implementor reproducibility.
- [ ] Residuals captured under [docs/notes/m3/](../../notes/m3/)
      if any surfaced during execution; phase ADR's residual
      section (if applicable) points at them.
- [ ] Phase-end retrospective entry added in
      [docs/notes/retrospectives.md](../../notes/retrospectives.md);
      merge & push gating handled per owner-facing protocol.

## Decisions log

(none yet — execution opened 2026-05-19.)

## CI / verification log

(empty — populated as tasks land.)

## Out-of-phase residuals

(empty — record here anything discovered during execution that is
out of Phase 1 scope, and file a `docs/notes/m3/` entry pointing
back to it per the m3-plan §Phase-end criteria.)
