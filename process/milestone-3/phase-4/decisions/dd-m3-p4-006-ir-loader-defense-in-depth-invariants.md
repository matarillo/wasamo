### DD-M3-P4-006 — IR-loader defense-in-depth invariants

**Status:** Accepted

**Context:** Phase 2 T7 surfaced the principle: IR-load → runtime-
materialise invariants belong in pure-logic `validate()`, not in
WinRT-bound `build_node`, so the same invariant is enforced
regardless of which entry point materialises the IR. Phase 3 T6
extended this with WrapPanel's value-range invariants (negative-
literal rejection). Phase 4 extends it with ScrollView's
invariants, which are a **different shape** than either Phase 2
(structural placement) or Phase 3 (value range): Phase 4 needs a
**compound** shape combining structural child-count rejection
(Phase-2-flavour) with runtime-clamp for the offset value (which
is *not* a validate-time reject) per
[m3-phase-4 pre-doc-inputs §5](../notes/m3-phase-4/pre-doc-inputs.md).

**Sub-issues:**

- **Child count.** Per DD-001 (exactly 1 child), `validate()`
  rejects 0-child and >1-child ScrollView with
  `WASAMO_ERR_IR_MALFORMED`. Symmetric with Phase 2 T7's `Box`
  child-count rejection in shape.
- **Offset value range.** Per DD-003 (`offset-y: <i32>`),
  `wasamoc check` rejects non-`IntLit` RHS shapes at compile
  time (existing infrastructure). The Phase 3 DD-006 "negative
  literal rejection" pattern **does not apply**: negative
  offsets are layout-time-clamped to 0 per DD-005 (not IR-
  rejected) because an author may bind a `state.scroll_y` that
  legitimately transitions through negative values during state
  changes. The two-gate defense-in-depth pattern still applies,
  but the runtime gate is the **clamp in DD-005's arrange pass**,
  not a `validate()`-time reject. This is the value-range half
  of the compound shape, distinct from Phase 3's pattern.
- **Bound-direction validation (conditional on DD-003).** Per
  DD-003 Option B (bindable read-only) decision, this sub-issue
  collapses — `validate()` has no mutability check to perform.
  Recorded for completeness: if DD-003 were Option C (bindable
  in-out), `validate()` would need to check the bound state is
  mutable, which is currently outside the IR's vocabulary and
  would have to defer to `wasamoc check`.
- **Error class.** All ScrollView IR-loader invariant violations
  surface as `WASAMO_ERR_IR_MALFORMED`, consistent with Phase 2 /
  Phase 3 precedent.

**Options:**

- **Option A — Compound: structural child-count gate at
  `validate()` + runtime clamp for offset at the arrange pass
  (recommended).** Matches the invariant shape per
  pre-doc-inputs §5.
  - What you gain: each invariant is enforced at the layer
    appropriate to its shape; structural invariants are rejected
    early (Phase-2-flavour), value-range invariants are
    accommodated dynamically (binding-friendly).
  - What you give up: nothing relative to the alternatives.
- Option B — Reject negative offset at `validate()`. Same
  pattern as Phase 3 DD-006.
  - What you gain: consistency with Phase 3 pattern.
  - What you give up: makes bindable offset fragile (any binding
    transition through a negative intermediate value becomes a
    runtime error); contradicts the layout-time clamp semantics
    DD-005 specifies.
- Option C — No defense-in-depth at all (rely on `wasamoc
  check`). Phase 1 / Phase 2 T7 / Phase 3 T6 explicitly
  established the two-gate principle; abandoning it for Phase 4
  is regressive.

**Decision:** Option A (compound: structural child-count gate +
runtime clamp). The `validate()` extension rejects 0 and >1
children; the runtime arrange pass clamps the offset. No
`validate()`-time offset value-range check.

**Layering with DD-001 / DD-003 / DD-005.** Inherits child-count
contract from DD-001; relies on DD-003's read-only binding
direction to make the no-mutability-check sub-issue collapse;
delegates value-range enforcement to DD-005's arrange pass.
