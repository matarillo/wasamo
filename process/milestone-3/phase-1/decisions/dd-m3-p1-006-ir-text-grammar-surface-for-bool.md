### DD-M3-P1-006 — IR text grammar surface for `bool`

**Status:** Accepted

**Context:**
The IR text format (DD-M2-P6-002) is the on-disk form `wasamoc` emits
and `wasamo-runtime` parses. Its normative grammar lives in
[docs/dsl_spec.md §8 "Wasamo IR — Normative Specification (M2)"](../dsl_spec.md#8-wasamo-ir--normative-specification-m2).
Adding `bool` requires updates to the IR §§ on types, literals, and
handler expressions (per DD-M3-P1-003 Option A: `BoolLit` and
`BoolPropRead` productions).

This DD is mostly a sub-decision of DD-M3-P1-001..003 made explicit so
the IR text spec update is not treated as an afterthought.

**Options:**

Option A — Spell IR text bool literals as `true` / `false` and add
`BoolLit` / `BoolPropRead` productions verbatim (recommended)
- Matches DD-M3-P1-002 surface syntax; matches DD-M3-P1-003 expression
  shape.

  - **Technical risk:** Low.

Option B — Spell IR text bool literals as `#t` / `#f` or `0` / `1` for
brevity
- Diverges from the surface `.ui` syntax. No real gain.

**Recommendation:** Option A. The IR text and `.ui` surface should
agree on bool spelling; divergence would pay only in characters saved.

This DD is kept independent rather than folded into DD-M3-P1-001..003
because the IR text grammar is the **public spec surface** that A12
(DSL public draft) commits to (see
[m3-plan.md A12](../plans/m3-plan.md#acceptance-criteria) and
[docs/dsl_spec.md §8](../dsl_spec.md#8-wasamo-ir--normative-specification-m2)).
Even when an IR text change is mechanically derived from an in-memory
IR change, the spec-surface decision deserves an explicit DD so that
the per-phase spec update (A11) has a single citable record.

---
