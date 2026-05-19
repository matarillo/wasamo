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
DD-M3-P1-003.

- [ ] `IrType::Bool` variant added; every existing `match` on
      `IrType` in the workspace gains a `Bool` arm (or an explicit
      reject) so the compiler enforces completeness.
- [ ] `IrLiteral::Bool(bool)` variant added; parallel to `Int` /
      `Str`.
- [ ] `HandlerExpr::BoolLit(bool)` and `HandlerExpr::BoolPropRead {
      path }` added; existing `IntLit` / `PropRead` left
      i32-implicit (no rename per DD-M3-P1-003 Option A).
- [ ] Pure-logic unit tests in `wasamo-ir` covering construction
      and equality of the new variants.

### T2 — `wasamoc` lexer / parser: `true` / `false` keywords and bool literal

Discharges DD-M3-P1-002 surface-syntax half, DD-M3-P1-006 (IR text
spelling).

- [ ] Lexer recognises `true` and `false` as reserved keywords;
      reservation is a hard error if used as an identifier.
- [ ] Parser produces `IrLiteral::Bool(true|false)` for surface
      bool literals.
- [ ] `wasamoc` IR text emitter writes bool literals as `true` /
      `false` and emits `BoolLit` / `BoolPropRead` productions
      verbatim.
- [ ] Unit tests: parse `state ready: bool = false`; parse
      `bind enabled: true`; reject `true` as a state name.

### T3 — `wasamoc` checker: state-type table and bool type-checking

Discharges DD-M3-P1-010.

- [ ] Parse-time `HashMap<String, IrType>` populated from `state`
      declarations.
- [ ] `check_members` (or equivalent) widened to carry enclosing
      widget context so `bind` LHS type-checking sees the target
      property's declared `IrType`.
- [ ] Accept / reject rules from DD-M3-P1-010's table implemented
      as compile-time diagnostics with line/column:
  - accept: `state ready: bool = false`,
    `bind enabled: ready` (bool/bool), `bind enabled: true`.
  - reject: `state ready: bool = 0`, `state ready: bool =
    "false"`, `bind enabled: 1`, `bind label: true`,
    `bind label: ready` (bool source / string target),
    `state x: i32 = 5; bind enabled: x`.
- [ ] Unit tests cover every row of DD-M3-P1-010's table.

### T4 — `wasamoc` lowering: identifier → typed `*PropRead`

Discharges DD-M3-P1-003 / DD-M3-P1-010 interaction (typed
lowering).

- [ ] Identifier lowering consults the state-type table: `bool`
      state name → `BoolPropRead`; `i32` → `PropRead`; `String` →
      `StrPropRead`.
- [ ] Unit test asserts lowering of `bind enabled: ready` for
      `state ready: bool = …` emits `BoolPropRead { path:
      "ready" }`.

### T5 — `wasamo-runtime` IR loader: read new productions

Discharges the IR-text-load half of DD-M3-P1-006.

- [ ] IR text loader accepts the new `IrType` / `IrLiteral` /
      `HandlerExpr` productions.
- [ ] Round-trip test (`wasamoc` emit → `wasamo-runtime` load)
      reconstructs `IrState { ty: Bool, default: Bool(false) }` and
      `HandlerExpr::BoolPropRead { path: "ready" }`.

### T6 — `wasamo-runtime` widget catalog: `PropertyValue::Bool`, typed `resolve_prop_key`, `Button.enabled`

Discharges DD-M3-P1-005, DD-M3-P1-009.

- [ ] `PropertyValue::Bool(bool)` variant added in
      [widget.rs L77-L80](../../../wasamo-runtime/src/widget.rs#L77-L80).
- [ ] `resolve_prop_key` widened to return `Option<(PropertyKey,
      IrType)>`; widget catalog rows carry `IrType` (M2 rows
      retain their declared types).
- [ ] `PROP_BUTTON_ENABLED` u32 id added (next free slot).
- [ ] `Button` widget setter dispatches `PROP_BUTTON_ENABLED` to
      the bool-typed setter; default value `true`.
- [ ] Phase 1 `Button.enabled` runtime contract: layout slot
      preserved when `false`; click-handler dispatch suppressed
      when `false`; minimal disabled visual (greyed colours, no
      animation). Out of Phase 1: focus / a11y / hover-state /
      key-activation behaviour (deferred per ADR §Out of scope).
- [ ] Mock-free Windows-only integration test asserting
      `PROP_BUTTON_ENABLED` flips the visual state on a live
      `WidgetNode`.

### T7 — `EvalContext` bool trait surface and handler evaluator arm

Discharges DD-M3-P1-004 (Option B), DD-M3-P1-008 (Option A — pair
flip).

- [ ] `EvalContext::get_bool`, `EvalContext::read_bool_tracked`,
      `EvalContext::set_bool` added with default impls mirroring
      the M2 i32 shape.
- [ ] `evaluate()` gains an arm for `Assign { lhs, rhs: BoolLit |
      BoolPropRead }`; rejects other bool-typed compound forms
      (`CompoundAssign` over bool is out of scope per ADR §Out of
      scope).
- [ ] Unit tests cover the new arm and the trait defaults.

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
preview.

- [ ] `read_property_value` / `write_property_value` /
      `property_value_to_owned` in
      [abi.rs L745-L749](../../../wasamo-runtime/src/abi.rs#L745-L749)
      gain bool arms threading `PropertyValue::Bool(bool)` ↔
      `WasamoValue::v_bool` (existing `WASAMO_VALUE_BOOL = 3` tag).
- [ ] Property-observer payload conversion carries bool through.
- [ ] No new public ABI functions added (DD-M3-P1-008 Option B
      explicitly deferred to its own future ADR).

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
