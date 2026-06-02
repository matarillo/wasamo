# M3-Phase 1 — `bool` scalar binding: Architecture Decisions

**Phase:** M3-Phase 1 (`bool` scalar binding)
**Date:** 2026-05-19
**Status:** Accepted

## Context

M3 acceptance criterion **A9** (see
[process/_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
[m3-plan.md](../../plan.md#acceptance-criteria)):

> `bool` admitted as the third scalar binding type alongside `i32` and
> `String`. The `TypedValue` generic value union remains deferred.

The M3 plan ([m3-plan.md §Phase breakdown](../../plan.md#phase-breakdown))
places this as Phase 1 because it is the **hard prerequisite** for
M3-Phase 6 (conditional rendering grammar A7 rides on a `bool` binding)
and M3-Phase 8 (Button `selected` state A10 rides on a `bool` binding).
Phase 1 closes when `bool` threads through the same `wasamo-ir` ↔
`wasamoc` ↔ `wasamo-runtime` path that `i32` and `String` already
travel, with one live `WidgetNode` attribute proving propagation; the
grammar surfaces that consume `bool` are out of this phase.

The M2 end-state shape that this phase must extend without breaking:

- `wasamo-ir` ([wasamo-ir/src/lib.rs](../../../../wasamo-ir/src/lib.rs)):
  `IrType` has two variants `I32 | Str`; `IrLiteral` has
  `Int | Str | Ident`; `HandlerExpr` uses **type-suffixed variants**
  (`IntLit` / `StrLit` / `PropRead` / `StrPropRead`) rather than a
  unified typed value.
- `EvalContext`
  ([wasamo-runtime/src/handler.rs](../../../../wasamo-runtime/src/handler.rs)):
  type-suffixed methods (`get_i32` / `get_string` /
  `read_i32_tracked` / `read_string_tracked` / `set_i32`). `set_string`
  is **absent** — strings are read-only in M2 because no handler writes
  to them. `evaluate()` returns `Result<i32, EvalError>`; binding-side
  evaluation has a separate string-typed path.
- Widget catalog
  ([wasamo-runtime/src/widget.rs](../../../../wasamo-runtime/src/widget.rs)):
  `Rectangle | VStack | HStack | Text | Button`; `PropertyValue` enum is
  `I32(i32) | String(String)`; per-widget per-attribute `PROP_*` u32
  IDs in [ir_loader.rs](../../../../wasamo-runtime/src/ir_loader.rs) lines
  799–802.

This ADR is framed against A9 and the M2 type-suffix pattern. It does
**not** re-open F5 (`TypedValue` deferral) — adding `bool` as a third
scalar is a different question, as recorded in
[m3-target-app-predoc.md — Tabs / Button selected-state surface closure (Reservation 3)](../../requirements/spec.md#保留-3-closure-tabs--button-選択状態-surface--採用-bool-を-3-つ目の-scalar-として導入).

The acceptance lens for this phase is narrow: A9 is satisfied when
`bool` reads through the live `.ui → IR → runtime` path on one widget
attribute. Consumers of `bool` (conditional rendering, Button selected)
are explicitly out of scope here.

---

## Out of scope (for M3-Phase 1; recorded explicitly)

- **Comparison and logical operators on `bool`** (`==`, `!=`, `&&`,
  `||`, `!`). Phase 1 establishes literal and property-read bool
  propagation only. Conditional rendering (M3-Phase 6) is where bool
  expressions get exercised; if it needs operators, the Phase 6 ADR
  introduces them with the surface they support.
- **`i32` ↔ `bool` coercion / truthy semantics.** Type-tagged
  representation means a bool-context expression must already be
  bool-typed; an i32 value is not implicitly `0 → false`. If a use
  case demands coercion, the Phase that needs it opens a DD.
- **`CompoundAssign` over bool** (i.e. `ready += true`, `ready *= x`).
  DD-M3-P1-004 Option B and DD-M3-P1-008 Option A admit *only* the
  `Assign { rhs: BoolLit | BoolPropRead }` shape in `evaluate()`.
  Compound assignment for bool has no agreed semantics (no
  `CompoundOp` is naturally bool-typed) and is not introduced by
  Phase 1.
- **State-write C ABI primitive** (e.g. `wasamo_set_state(name,
  WasamoValue)`). DD-M3-P1-008 Option B; deferred to its own ADR
  pending demand from a phase whose evidence shape requires host-
  side state mutation (Phase 1's evidence is admitted via handler-
  side `set_bool` per DD-M3-P1-008 Option A, sidestepping this
  question for Phase 1). Plausible triggers: M4 input / event-source
  phases needing async state injection from outside the handler
  model (timer-driven, I/O completion, host-side animation
  parameters), or a pre-M6 ABI-hardening review. Deadline-bounded
  by M6 C ABI freeze — post-1.0 ABI surface is append-only, so if
  this primitive is needed at all it must land before the freeze.
  No current ROADMAP item names it as a hard prerequisite; the
  VISION §4 Principle 2 two-channel model ("events up, bindings
  down") also argues against treating host-observed-property →
  host-written-state as a routine channel, so this deferral is
  principled rather than tacit.
- **Full `Button.enabled` interaction-state contract.** Phase 1
  narrows to: click suppression, layout slot preserved, minimal
  visual. Deferred to M4 (input/focus) / M5 (a11y): keyboard
  focusability and tab-order when disabled, AccessKit /
  `aria-disabled` semantics, hover and focus visual states, key
  activation suppression. See DD-M3-P1-005's "Out of scope (Phase 1)"
  sub-list.
- **`PropertyValue` / binding writer becoming a generic value union.**
  DD-M3-P1-007 keeps per-type evaluators and per-type writers so the
  binding pipeline never funnels through one runtime value tag. The
  decision is structural F5 enforcement, not just current scope.
- **`Button.selected` and any other widget attribute beyond
  `Button.enabled`.** A10 lives in M3-Phase 8 ADR; this phase opens
  one attribute, no more.
- **Visibility / conditional rendering / subtree presence.** A7 lives
  in M3-Phase 6 ADR; DD-M3-P1-005 rejects Option B to keep this
  decoupled.
- **`TypedValue` generic value union.** F5 deferral is preserved
  ([m3-start-framing.md §F5](../../requirements/framing.md#l335)).
  Adding `bool` as a third tagged scalar is not the same decision as
  introducing a typed value union.
- **Per-symbol IR text grammar rename** (`PropRead` →
  `IntPropRead`). DD-M3-P1-003 Option C; deferred to a future phase
  that has substantive IR grammar reason to touch the names.

## Owner-agreement checkpoints

Two of the DDs above are load-bearing value judgements that warrant
explicit yes/no from the owner before this ADR moves to Accepted.
All other DDs follow mechanically from these two.

### Checkpoint 1 — DD-M3-P1-004 ⇄ DD-M3-P1-008 pair flip

**Question:** Does Phase 1 admit handler-side bool writes
(`set_bool` on `EvalContext` + bool-typed `Assign` arm in
`evaluate()`)?

**Default answer:** Yes (DD-004 Option B + DD-008 Option A —
"handler-side bool write" path).

**Framing for owner:** The pair flip is not scope creep — it is the
**minimum** addition that lets the Phase 1 live-propagation evidence
ride the `.ui` surface. The alternatives are concretely worse:

- Read-only `EvalContext` + new `wasamo_set_state` ABI primitive
  (DD-008 Option B): introduces a permanent public ABI surface whose
  design (component identity, observer firing, thread-affinity)
  isn't closed; that belongs in its own ADR, not piggy-backed on a
  scalar introduction.
- Read-only `EvalContext` + test-only state mutation hook
  (DD-008 Option C): cheapest delta, but Phase 1's evidence is no
  longer connected to the public surface that A12 (DSL public draft)
  is supposed to record.
- Read-only `EvalContext` + initial-value only (DD-008 Option D):
  the entire reactive pipeline for bool goes unexercised — m3-plan
  Phase 1 explicitly calls for "live `WidgetNode` propagation."

The cost of saying yes is **one trait method** (`set_bool`,
mirroring the existing `set_i32`) and **one new `evaluate()` arm**
(bool-typed `Assign`). The cost of saying no is one of the three
alternatives above. In that sense the flip keeps Phase 1 small,
not large.

### Checkpoint 2 — DD-M3-P1-005 Option A vs Option E

**Question:** Does Phase 1 surface `Button.enabled: bool` as a
public DSL attribute (Option A), or ship the bool plumbing without
committing the public widget spec to any specific bool attribute
(Option E)?

**Default answer:** Option A (public `Button.enabled`, intentionally
narrow contract).

**Phase 1 `Button.enabled` contract if Option A is taken:** The
public DSL spec entry guarantees only:

- a bool-typed `enabled` attribute on `Button`;
- default `true`;
- when `false`, the layout slot is preserved (no `display: none`
  behaviour);
- when `false`, click-handler dispatch is suppressed;
- a minimal disabled visual (greyed colours, no animation).

The contract **explicitly defers** to M4 (input/focus) and M5
(accessibility): keyboard focusability and tab-order when disabled,
AccessKit / `aria-disabled` semantics, hover and focus visual
variations, key activation suppression.

**Trade-off framing:** Option A puts the live-propagation proof on
a public widget attribute, which keeps the evidence aligned with
A11/A12 (per-phase spec sync; DSL public draft). The risk is that
"disabled control" semantics can creep — the narrow contract above
is the fence.

Option E falls back to a runtime-internal bool probe property that
exists in the widget catalog but is not in `docs/dsl_spec.md`. It
sidesteps the disabled-control framing entirely, at the cost of
A9's public-spec evidence shrinking to scalar / literal /
type-checking — there is no bool-bound widget attribute in the
M3 public DSL draft from Phase 1's work in that scenario.

The default favours Option A on the grounds that A12 (DSL public
draft) is better served by evidence that reaches a user-visible
attribute, but the disabled-control objection is real; if click
suppression alone is judged too heavy for Phase 1, Option E is the
clean fallback.

---

## Summary of decisions

| ID | Topic | Recommendation |
|---|---|---|
| DD-M3-P1-001 | `IrType` extension | Option A — add `IrType::Bool` |
| DD-M3-P1-002 | `IrLiteral` + surface syntax | Option A / Option A — `IrLiteral::Bool(bool)` and `true` / `false` keywords |
| DD-M3-P1-003 | `HandlerExpr` variants for bool | Option A — add `BoolLit` + `BoolPropRead`, no rename of existing variants |
| DD-M3-P1-004 | `EvalContext` method shape | Option B — full `get_bool` + `read_bool_tracked` + `set_bool` (paired with DD-M3-P1-008 Option A) |
| DD-M3-P1-005 | Phase 1 evidence widget attribute | Option A — `Button.enabled: bool` with narrowed Phase 1 contract (click suppression + minimal visual; focus / a11y deferred to M4–M5) |
| DD-M3-P1-006 | IR text grammar surface | Option A — `true` / `false` spelling, parallel `Bool*` productions |
| DD-M3-P1-007 | Binding eval result shape + writer signature | Option A — per-type binding evaluator (`evaluate_bool_binding`) + per-type writer (`widget_write_property_bool`); `PropertyValue::Bool(bool)` added but not unified into a value union |
| DD-M3-P1-008 | Mutation source for Phase 1 evidence | Option A — admit handler-side bool writes in Phase 1 (`set_bool` + `evaluate()` bool-typed `Assign` arm); flips DD-M3-P1-004 to its Option B |
| DD-M3-P1-009 | Property type metadata + writer dispatch | Option A — `resolve_prop_key` returns `(PropertyKey, IrType)`; widget catalog row carries the type |
| DD-M3-P1-010 | `wasamoc` type-checker scope for bool | Option A — full state/binding/identifier type-checking at `wasamoc check`; mismatches are compile-time diagnostics |

Implementation task list: belongs in the Phase 1 progress file
`docs/plans/progress/m3-phase-1-progress.md` (created when this ADR
is Accepted and Phase 1 starts execution); not in this ADR and not
in `m3-plan.md` itself. See
[plans/README.md §Scope rule (plan vs ADR)](../../../README.md#scope-rule-plan-vs-adr)
and [plans/README.md §Phase progress file lifecycle](../../../README.md#phase-progress-file-lifecycle)
for the authoritative location and the `active → closing → retired`
lifecycle the file follows. The Progress table in
[m3-plan.md](../../plan.md) carries only a one-row index entry
pointing at this progress file.

## Spec impact preview (for owner agreement)

When this ADR is accepted, the following docs change in the same Phase
1 commit set (per A11 same-phase synchronisation):

- [docs/dsl_spec.md](../../../../docs/dsl_spec.md) — extensions in two regions:
  - **DSL surface** (§§ 2–4): `true` / `false` keyword reservation in
    §2.1; bool literal token in §2; `bool` type in §4.2 (`in-out
    property`) / state declarations; bool in §4.3 (property binding)
    and §4.6 (expressions: `BoolLit`, `BoolPropRead`, and `Assign`
    with bool-typed RHS; `CompoundAssign` over bool excluded).
  - **IR text grammar** (§8): `IrType` production adds `bool`;
    literals add `BoolLit`; handler expressions add `BoolPropRead`.
  - `Button.enabled` attribute documented in the widget catalog
    section (minimal disabled styling permitted in M3; no animation
    contract).
- [docs/architecture.md](../../../../docs/architecture.md) — §6.7.7 SignalRegistry
  snippet updated:
  add `bools: HashMap<String, Signal<bool>>` alongside `i32s` and
  `strings`; the surrounding prose extends "M2 supports `i32` and
  `String` Signals" to include `bool` and notes that
  `HandlerExpr::BoolPropRead` evaluates through
  `BindingEvalContext::read_bool_tracked`. F5 deferral cross-reference
  is preserved. The binding write-seam description around
  [architecture.md §6.7.7](../../../../docs/architecture.md#677-binding-registration-api-after-m2) is also updated to reflect
  DD-M3-P1-007: `write_fn` is per-type at the call site rather than a
  single string-baked function pointer.
- [docs/abi_spec.md](../../../../docs/abi_spec.md) — **no new ABI surface added.**
  `WASAMO_VALUE_BOOL = 3` and `v_bool` already exist
  ([abi_spec.md §3.3](../../../../docs/abi_spec.md), [abi.rs L74-L90](../../../../wasamo-runtime/src/abi.rs#L74-L90))
  from M2; Phase 1 only connects this existing tag through the
  property-write path that previously dropped it. Specifically,
  `read_property_value` / `write_property_value` and
  `property_value_to_owned` ([abi.rs L745-L749](../../../../wasamo-runtime/src/abi.rs#L745-L749))
  gain bool arms; `PropertyValue` ([widget.rs L77-L80](../../../../wasamo-runtime/src/widget.rs#L77-L80))
  gains `Bool(bool)`; the property observer payload conversion
  carries bool through to `WasamoValue::v_bool`. Existing ABI
  function signatures and value-tag numeric assignments are
  untouched (M6 freeze scope unchanged; this phase is pre-freeze).
- [wasamoc/src/check.rs](../../../../wasamoc/src/check.rs) and adjacent
  lowering — per DD-M3-P1-010: state-name → declared-type table
  built at parse, used to lower identifiers to typed `*PropRead`
  variants and to type-check `bind` LHS / RHS pairings.

No ROADMAP revision is anticipated — A9 is already explicit, this ADR
operationalises it.

## Phase 1 verification closure (what counts as A9 evidence)

This section is not a DD — it records the agreed shape of the proof
that closes Phase 1, so the implementation plan in
[m3-plan.md Progress](../../plan.md) inherits a concrete target
rather than re-litigating "what does live propagation mean here?".

A9 (`bool` admitted as third scalar) is considered satisfied when
**all four** of the following are observed, in this order:

1. **Unit-test evidence (host-independent).** Pure-logic tests in
   `wasamoc` (parse + check + lower) and in `wasamo-runtime`
   non-Windows-bound modules (handler evaluator, binding evaluator,
   `SignalRegistry`) cover: bool literal parsing; type-checker
   accept/reject pairs from DD-M3-P1-010's table; `evaluate()`
   bool `Assign` arm; `evaluate_bool_binding` accept set;
   `Signal<bool>::set` triggering effect re-run. These run on any
   CI runner.

2. **IR text round-trip evidence.** `wasamoc` emits, `wasamo-runtime`
   loads, and an in-process test reads back: `state ready: bool =
   false` as `IrState { ty: Bool, default: Bool(false) }`;
   `bind enabled: ready` as `HandlerExpr::BoolPropRead`. Tests the
   DD-M3-P1-001/002/003/006 surfaces together.

3. **Windows-runtime live-propagation evidence (CI-gated).** A
   mock-free integration test (per CLAUDE.md "Testing rules") on the
   Windows CI runner: an `.ui` fixture declares
   `state ready: bool = true; Button { enabled: ready; on click {
   ready = false } }`. The test loads the IR, invokes the button's
   click signal, observes `Signal<bool>::get_untracked()` flips to
   `false`, and observes that the `Button`'s widget-side
   `PROP_BUTTON_ENABLED` reflects the new value. Fails (not skips)
   on a runner that cannot create the Compositor — the test gates
   A9 evidence in CI, not local convenience.

4. **End-to-end host evidence (one host suffices).** One of
   `examples/counter-rust`, `examples/counter-c`, or
   `examples/counter-zig` is extended (or a new minimal `bool-demo`
   example is added under `examples/`) to drive the Phase 1 fixture
   through to a visible window. The choice of host is recorded in
   the m3-plan Phase 1 Progress section once implementation starts;
   the Rust host is the working default because its build path
   doesn't require the `wasamoc.exe` ordering dance (CLAUDE.md
   "Build ordering requirements"). C and Zig hosts are not required
   for Phase 1 — the C-ABI bool plumbing is exercised by the
   `PropertyValue::Bool` ↔ `WasamoValue::v_bool` conversion arms
   reached via the property-observer payload, which the
   Windows-runtime test in (3) already covers.

Items (1)–(3) are required; item (4) is the visible proof that ties
Phase 1's evidence back to the m3-plan target-app trajectory. Items
(1) and (3) together close DD-M3-P1-007's per-type writer seam (the
unit test covers the dispatch logic; the integration test covers the
end-to-end live propagation). DD-M3-P1-008's choice of handler-side
mutation is what makes items (3) and (4) achievable without new ABI
surface.

The acceptance/non-acceptance of test items (1)–(4) is the
operational form of "Phase 1 done"; the corresponding implementation
checklist (which crate / which test file / which `.ui` fixture)
belongs in m3-plan Progress, not here.
