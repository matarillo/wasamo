### DD-M3-P3-006 — IR-loader defense-in-depth invariants

**Status:** Accepted

**Context:**
Phase 2 T7 surfaced the principle: IR-load → runtime-materialise
invariants belong in pure-logic `validate()`, not in WinRT-bound
`build_node`, so the same invariant is enforced regardless of which
entry point materialises the IR. Phase 3 extends this with WrapPanel's
invariants.

**Options (attribute value range — non-negative integer):**

Option A — Two-gate defense-in-depth: `wasamoc check` rejects negative
literals at compile time; `validate()` rejects negative IR at IR-load
time (recommended)
- `item-spacing`, `line-spacing` (DD-003), and `item-cross-size`
  (DD-004) all ship as `i32` attributes whose spec admits
  **non-negative values only**. Both gates required because
  `wasamo_load_ui`'s memory-IR path does not pass through the
  compiler; the runtime `validate()` is the last line of defence
  for the spec invariant. Pattern mirrors Phase 2 DD-M3-P2-005's
  RATIO rejection (structural pattern identical; literal threshold
  differs — Phase 2 RATIO rejects `<= 0` because zero is structurally
  meaningless, Phase 3 integers reject `< 0` only).

  - What you gain: Invariant holds even for IR produced outside
    `wasamoc`. Symmetric with Phase 1 T14 and Phase 2 T7 discipline.
  - **Technical risk:** Low.

Option B — Single-gate (`wasamoc check` only)
- Trust `wasamoc check`; do not duplicate the rejection in
  `validate()`.

  - What you give up: Contradicts the Phase 2 T7 precedent. The
    `wasamo_load_ui` memory-IR path bypasses `wasamoc`, so a
    negative-`item-spacing` IR loaded from memory would proceed to
    layout with an out-of-spec value.

**Options (zero handling — author-requested degenerate vs error):**

Option A — Zero is a *valid* setting for all three attributes; not a
silent-zero footgun (recommended)
- `item-spacing: 0` / `line-spacing: 0` — touching items / lines.
  This is Phase 3's default value; visible-zero by construction.
- `item-cross-size: 0` — each line collapses to zero cross-axis
  extent (no thumbnails rendered, line count still computed). Spec
  text records this as an *author-requested degenerate layout*,
  distinct from the "no extent to resolve" runtime errors of
  DD-005's unbounded-both-axes branch and the
  `BoxAspectUnboundedBoth` case.

  - What you gain: Zero has an unambiguous semantic — a written-out
    intentional setting in the `.ui` source is honoured. Distinct
    from the *absence* of any bound source (the unbounded-both-axes
    case), which is the actual error.
  - **Technical risk:** Low.

Option B — `wasamoc check` warns on `item-cross-size: 0`
- A zero-cross-size WrapPanel renders nothing; warn that the author
  may have made a mistake.

  - What you give up: Mixes "the author wrote 0" with "the author
    forgot to set the value" — the latter is impossible because the
    attribute is optional and the unset case has its own well-defined
    behaviour (DD-004 Option (a) passthrough). Reject as redundant.

**Options (child count):**

Option A — WrapPanel admits 0 or more children; no upper bound; no
structural rejection (recommended)
- Empty WrapPanel is structurally valid (see DD-001 0-child shape).
  Unlike Box (single-child-only per DD-M3-P2-001), WrapPanel has no
  child-count restriction.

  - **Technical risk:** Low.

**Options (orientation values — conditional on DD-002):**

Conditional on DD-002 Option B / C (orientation attribute exposed).
DD-002's recommendation is Option A (not exposed), so this sub-issue
collapses; recorded for completeness in case DD-002 flips.

Option A — `validate()` rejects unknown orientation values
(conditional, recommended if attribute exists)
- Would be rejected by `wasamoc check` first, but the two-gate
  principle applies.

**Options (error class):**

Option A — All WrapPanel invariant violations surface as
`WASAMO_ERR_IR_MALFORMED` (recommended)
- Consistent with Phase 2's `Box`-child-count rejection error class.

**Recommendation:** Option A for every sub-issue —

- Non-negative integer range: two-gate defense (`wasamoc check` +
  `validate()`).
- Zero: valid for all three attributes; author-requested degenerate
  layout.
- Child count: 0 or more; no rejection.
- Orientation: conditional on DD-002 (collapses under DD-002 Option A
  recommendation).
- Error class: `WASAMO_ERR_IR_MALFORMED`.

**Forward-compat exposure:** No exposure differential between
candidate options — the defense-in-depth pattern is additive across
phases, and the zero-handling stance does not constrain future
attributes (a future bindable attribute would extend the per-attribute
validate logic; the constant-only path Phase 3 ships is what gets
extended, not replaced).

---
