### DD-M3-P1-003 — `HandlerExpr` variants for `bool`

**Status:** Accepted

**Context:**
`HandlerExpr` is the shared IR ([wasamo-ir/src/lib.rs L28–L49](../../wasamo-ir/src/lib.rs#L28-L49))
between `wasamoc::lower`/`emit` and the runtime evaluator. M2 chose the
type-suffix pattern (DD-M2-P6-003 = Option A): `IntLit` / `StrLit` for
literals, `PropRead` (i32) / `StrPropRead` (string) for property reads.
The Phase 1 question is how `bool` joins this enum.

Per DD-M3-P1-004 Option B and DD-M3-P1-008 Option A, Phase 1 admits
handler-side bool writes through the existing `HandlerExpr::Assign`
variant (no new write variant introduced — `Assign` already exists
from M2 and only its `rhs` set widens to include `BoolLit` and
`BoolPropRead`). The evaluator-side widening is recorded in DD-004 /
DD-008; this DD records only the new literal / property-read
variants. `CompoundAssign` over bool remains out of scope (no
naturally bool-typed `CompoundOp` exists; see Out of scope below).

**Options:**

Option A — Add `HandlerExpr::BoolLit(bool)` and
`HandlerExpr::BoolPropRead { path }`, mirroring the `Str*` pattern
(recommended)
- Additive variants; no rename of existing variants. The implicit
  `PropRead` / `IntLit` retain their (i32) typing by convention,
  matching the M2 status quo.

  - What you gain: Continues the DD-M2-P6-003 discipline without
    modification. Every site that already handles `Str*` learns the
    same pattern for `Bool*`. Pattern-match exhaustiveness compiler-
    enforces completeness.
  - What you give up: Mild asymmetry remains — `PropRead`/`IntLit`
    *implicitly* mean i32 while `Str*` and `Bool*` carry explicit
    suffixes. This is a pre-existing M2 wart; this DD does not fix it
    (see Option C).
  - **Technical risk:** Low.

Option B — Unify all literals and property reads into a single typed
form: `HandlerExpr::Lit { value: TypedLiteral }` and
`HandlerExpr::PropRead { path: String, ty: IrType }`
- Replaces the type-suffix pattern with type-on-variant.

  - What you gain: Cleaner shape; the type is a first-class field.
  - What you give up: Re-opens DD-M2-P6-003. Touches every existing
    `HandlerExpr` match site in `wasamoc` lowering, the IR text
    emitter, the runtime evaluator, and the IR loader. Also drifts
    toward `TypedValue` (F5 deferral) — the whole point of the
    type-suffix pattern was to defer that union. Out of phase scope.
  - **Technical risk:** Medium — large refactor surface across two
    crates and the IR text spec.

Option C — Option A plus renaming `PropRead` → `IntPropRead` and
`IntLit` left as-is (or also renamed) for symmetry
- Cosmetic clarity: every variant's type becomes explicit in the name.

  - What you gain: Symmetry. A reader of `HandlerExpr` no longer needs
    to know "`PropRead` happens to mean i32 because i32 was first."
  - What you give up: Rename churn across `wasamoc` and runtime
    evaluator for a payoff that is purely readability. Touches the IR
    text grammar (`PropRead` vs `IntPropRead` token spelling) and so
    bumps the spec normatively without functional change.
  - **Technical risk:** Low; cost is reviewer attention, not
    correctness.

**Recommendation:** Option A. The rename in Option C is tempting and
the right end-state, but the phase pays its scope dividend by *not*
re-opening M2 IR grammar text. If a future phase touches the IR text
grammar substantively, fold the rename in there.

**Forward-compat exposure:** Option A leaves the M2 asymmetry in place.
If `TypedValue` is later admitted (which would supersede the
type-suffix pattern in its entirety), the rename in Option C would be
discarded anyway, so its absence does not increase exposure.

---
