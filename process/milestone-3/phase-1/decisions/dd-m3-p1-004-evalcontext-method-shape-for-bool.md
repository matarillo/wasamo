### DD-M3-P1-004 — `EvalContext` method shape for `bool`

**Status:** Accepted

**Context:**
`EvalContext` is the trait through which `HandlerExpr` evaluation
reaches the host's reactive store. M2 added string reads as
`get_string` + `read_string_tracked` (both with default impls that
error or forward) but did **not** add `set_string`, because no handler
writes to a string in M2.

Phase 1's evidence shape (per DD-M3-P1-008 Option A) requires live
mutation of bound `bool` state from a `.ui`-side handler, which means
the trait has to admit a bool-typed write. `CompoundAssign` over
`bool` is not exercised — there is no naturally bool-typed
`CompoundOp` — but plain `Assign { rhs: BoolLit | BoolPropRead }` is.

**Options:**

Option A — Add `get_bool` + `read_bool_tracked` only; no `set_bool`
- Mirrors the M2 String shape exactly. Sufficient only if some
  *other* path drives `Signal<bool>::set` (host-side ABI, test hook,
  or no live mutation at all — see DD-M3-P1-008).

  - What you gain: Minimal trait surface.
  - What you give up: Phase 1's evidence cannot be a `.ui`-only
    proof; the mutation source has to come from elsewhere. Once
    DD-M3-P1-008 lands as anything other than its own Option B
    (separate state-write ABI) or C (test hook), this trait shape
    becomes insufficient.
  - **Technical risk:** Low to implement; load-bearing on
    DD-M3-P1-008 picking a non-handler route.

Option B — Add the full `get_bool` + `read_bool_tracked` + `set_bool`
(recommended)
- Eager symmetry with `i32`. Pair with one bool-typed arm in
  `evaluate()` for `Assign { rhs: BoolLit | BoolPropRead }`.

  - What you gain: `.ui`-only live-propagation proof (e.g. a button
    with `on click { ready = false }`). Mirrors `set_i32` which
    already exists for the M2 counter handler. Phase 8 (selected
    state, A10) will require handler-side bool writes for any
    toggle construct — pre-shipping `set_bool` removes a Phase 8
    hard prerequisite at low marginal cost.
  - What you give up: One trait method and one new `evaluate()` arm
    beyond what a pure read-only path would need.
  - **Technical risk:** Low. Shape mirrors `set_i32` verbatim.

**Recommendation:** Option B. This recommendation is load-bearing
paired with DD-M3-P1-008 Option A — together they form the
"handler-side bool write" path that makes Phase 1's evidence
self-contained in `.ui`. The earlier draft of this DD recommended
Option A on a misread M2 precedent (treating "M2 has no `set_string`"
as a "read-first" principle, when in fact M2 simply had no
handler writing strings). The bool case is materially different:
Phase 1 explicitly needs live propagation, which means *something*
must call `Signal<bool>::set`; the cheapest, in-spec route is the
handler-side trait extension. If DD-M3-P1-008 lands as Option B
(separate state-write ABI), this DD flips back to its Option A.

---
