### DD-M3-P3-002 — Orientation attribute

**Status:** Accepted

**Context:**
Whether WrapPanel exposes an `orientation: <horizontal|vertical>`
attribute in Phase 3, or hardcodes horizontal main-axis with
vertical reserved for a later DD.

**Options (exposure):**

Option A — Hardcode horizontal main-axis; do not expose `orientation`
in Phase 3 (recommended)
- WrapPanel's main axis is horizontal; cross axis is vertical. No
  `orientation` attribute on the IR node, no surface syntax, no
  `wasamoc` lex / parse / check addition. The Phase 3
  `dsl_spec.md` §4.10 chapter records "horizontal main-axis only in
  Phase 3; a later phase opens vertical via an additive DD".

  - What you gain: Smallest Phase 3 surface — no attribute plumbing,
    no spec text spec'ing the axis swap (DD-005 spec is half the
    length). The vertical-main-axis case has no Phase 3 sub-screen,
    no Phase 4 / 5 downstream dependency, and is a clean additive
    extension if a later phase needs it.
  - What you give up: An author who anticipates needing vertical
    wrap (e.g. a future right-side tag column) writes a different
    WrapPanel-shape today and migrates when vertical lands. The
    migration is purely additive (add `orientation: vertical`),
    not a rewrite.
  - **Technical risk:** Low.

Option B — Expose `orientation: horizontal | vertical` as a
constant-only attribute in Phase 3
- WrapPanel admits both orientations. DD-005's algorithm specifies
  both axes symmetrically (main-axis flow + cross-axis stack, with
  "main" and "cross" defined by the orientation). The bindable
  sub-issue collapses to constant-only per the Phase 1 / Phase 2
  seam-building discipline (no Phase 3 sub-screen calls for animated
  orientation).

  - What you gain: Vertical WrapPanel is available the moment a
    future sub-screen needs it; no migration required.
  - What you give up: DD-005's spec writes the axis swap explicitly,
    doubling the algorithmic spec text. The Phase 3 sub-screen
    exercises only the horizontal path, so the vertical-path spec
    text has no Phase 3 evidence. Discipline of "build the seam in
    the phase that needs it" argues against.

Option C — Expose `orientation` as a bindable attribute
- All of Option B's surface, plus a per-type writer seam triple for
  an enum-typed orientation property.

  - What you give up: All of Option B's objections, plus the
    speculative seam-building that DD-M3-P2-004 ruled out for
    `aspect` / `fill` symmetrically. No use case in any Phase 3 / 4
    / 5 sub-screen.

**Recommendation:** Option A — do not expose `orientation` in Phase 3.
Horizontal main-axis is hardcoded; vertical opens additively when a
later phase surfaces a use case. The bindable sub-issue collapses by
construction: no attribute, no bindable question.

**Forward-compat exposure:** Option A is structurally additive. When
a future phase needs vertical, the addition is: new optional attribute
on the IR node, new `wasamoc` lex / parse / check for the enum
keywords, new arm in DD-005's algorithm. None of this revises Phase 3
plumbing — it extends it. Option B / C would have committed Phase 3
to spec text and surface area with no evidence to validate them; the
forward-compat saving from "ship now to avoid revising later" is
illusory because there is nothing to revise (the Phase 3 algorithm
operates on `main_axis` / `cross_axis` parameters that the future
attribute would simply re-bind).

---
