### DD-M3-P3-005 — Measure-arrange algorithm (novel normative spec)

**Status:** Accepted

**Context:**
The load-bearing DD of Phase 3. The first M3 phase to introduce a
novel measure-arrange *paradigm* — two-stage measure-arrange — into
`docs/dsl_spec.md`. The DD settles the line-formation algorithm and
its edge cases; the ADR section is also the *seed* of the dsl_spec
chapter (Moment 1 lands the spec chapter in design-spec draft form;
Moment 2 re-syncs to implementation findings).

The DD has the broadest spec content of any Phase 3 DD; rather than
enumerate every sub-issue as Options (most have one defensible
answer once DD-001 / DD-004 are settled), the Options below cover
the genuinely contested sub-issues; the un-contested sub-issues are
recorded in the **Recommendation** prose as direct statements of the
spec text Phase 3 will ship.

**Options (unbounded main-axis parent):**

When the parent provides no main-axis bound, WrapPanel cannot wrap —
there is no boundary to compare cumulative line extent against. The
realistic context is an outer intrinsic-sizing measure pass (e.g.
WrapPanel inside a future Phase 5 Grid cell whose width is being
computed intrinsically before star sizing resolves, or a host-driven
measure for window-sizing). **Phase 4 ScrollView is *not* the
canonical example** — ScrollView's vertical-scroll use in the gallery
bounds the *main* axis (WrapPanel main-axis = WrapPanel width =
viewport width) and unbounds the *cross* axis. Citing ScrollView
here would muddy the Phase 4 contract.

Option A — One-line flow: all children flow on a single line
(recommended)
- WrapPanel-without-main-axis-bound degenerates to HStack-equivalent
  layout: every child sits on one line, in document order. The line's
  cross-axis extent follows the same per-line rule as any other line
  (DD-004-bound or passthrough, max of children's reported cross-axis
  sizes).

  - What you gain: WrapPanel composes with intrinsic-sizing measure
    passes rather than blowing up. The one-line outcome is *visible*
    (the caller sees a long row), not silent like a zero-extent
    dropout. Defensible reading: "no place to wrap, so don't".
  - **Technical risk:** Low.

Option B — Layout-time runtime error
(`LayoutError::WrapPanelUnboundedMain`)
- Symmetric with Phase 2 DD-M3-P2-005's unbounded-both-axes case
  for aspect-fixed Box. The layout pass emits an error when
  encountering a WrapPanel with no bounded main-axis.

  - What you gain: Honest behaviour — no silent degeneration.
    Author-error-detection at runtime.
  - What you give up: Phase 2's no-silent-dropout virtue doesn't
    transfer cleanly: Phase 2's degenerate Box was structurally
    zero-extent (silent invisible failure); the Phase 3 degenerate
    WrapPanel is structurally one-line-flow (visible non-failure).
    Erroring on the visible-non-failure case is more aggressive than
    erroring on the zero-extent case. WrapPanel inside any future
    intrinsic-sizing context (Grid cell width derivation, host
    measure pass) would blow up; the layout engine would have to
    pre-check before invoking WrapPanel measure. New
    `LayoutError::WrapPanelUnboundedMain` variant required.

Option C — Take the child's intrinsic union as the main-axis bound,
then wrap
- Compute `sum(child_intrinsic_main) + spacing × (n−1)` as the
  pseudo-bound, then run the line breaker.

  - What you give up: Incoherent — once you've taken the union as
    the bound, all children fit on one line (the bound is exactly
    the sum). Degenerates to Option A but via a circuitous route.

**Options (LayoutError surface — consequent on the unbounded
main-axis choice):**

Option A — No new `LayoutError` variant (consequent on Option A
above; recommended)
- The one-line-flow degeneration uses the same layout machinery as
  the normal bounded-main-axis path; no new error variant.

  - **Technical risk:** Low.

Option B — Add `LayoutError::WrapPanelUnboundedMain` (consequent on
Option B above)
- Symmetric with Phase 2's `LayoutError::BoxAspectUnboundedBoth`.

  - What you give up: New variant, only justified if the
    unbounded-main-axis branch is treated as an error.

**Options (oversized first-child of a line — line-breaker rule):**

The DD-001 inequality

```
current_line_main + (line_empty ? 0 : item_spacing)
  + next_child_main_intrinsic
  <= parent_main_bound
```

evaluates to *false* when `line_empty == true` and
`next_child_main_intrinsic > parent_main_bound` — i.e. the candidate
would be the first child of an empty line *and* its intrinsic
main-axis size alone exceeds the parent's main-axis bound. Without
an explicit rule for this case, the algorithm is ambiguous: a
naïve "fail-the-test → start-a-new-line" reading loops forever
(the candidate fails on every new line). A spec-complete line
breaker must commit to one of the options below.

Option A — Unconditional placement on `line_empty` (recommended)
- When `line_empty == true`, the candidate child is placed on the
  current line regardless of the inequality. The line's recorded
  main extent equals the (oversized) child's intrinsic main extent
  and *may exceed* `parent_main_bound`. The inequality is consulted
  only for subsequent children of the same line — they will not
  fit alongside the oversized child, so each closes the current
  line and starts a new one, where the same unconditional-placement
  rule applies.

  - What you gain: Deterministic, infinite-loop-free, matches the
    general WrapPanel convention across WPF / Slint / most-frameworks
    (an item that does not fit anywhere still appears, occupying
    its own line). The "line extent may exceed bound" outcome is
    visible (the caller sees an overflowing row), not silent.
  - What you give up: WrapPanel's per-line main-axis extent is no
    longer guaranteed `<= parent_main_bound`. Downstream code that
    consumed "the line extent is bounded by parent" must instead
    consume "the line extent is `max(child_intrinsic_main,
    parent_main_bound)`-ish". The arrangement / paint clip option
    below handles the visible-extent question separately so this
    asymmetry does not leak into the WrapPanel's outer-bounds
    contract.
  - **Technical risk:** Low.

Option B — Layout-time runtime error
(`LayoutError::WrapPanelOversizedChild`)
- The layout pass emits an error when a child's intrinsic main-axis
  size exceeds `parent_main_bound` and no line accommodates it.
  New variant required.

  - What you give up: An author-error reading that does not match
    the usual WrapPanel convention — *some* oversized children are
    legitimate (a long string in a Text widget, a wide thumbnail
    set against a narrow window). Erroring closes off the visible-
    overflow recovery path Option A preserves. Also requires a new
    `LayoutError` variant for a case Option A handles without one.

Option C — Skip oversized children silently
- Drop the candidate; do not place it on any line.

  - What you give up: Silent dropouts are bug-magnets (Phase 2
    DD-005 rejected the same shape for unbounded-both-axes Box).
    Visually missing children with no diagnostic.

**Options (oversized line — arrangement / paint clip):**

Option A under the previous sub-issue allows a line's recorded
main extent to exceed `parent_main_bound`. The arrangement / paint
pass then needs a separate rule for what the visible rendering of
such a line looks like — independent of the line-breaker decision.

Option A — Visible overflow at the WrapPanel boundary; WrapPanel
outer main-axis equals `parent_main_bound` (recommended)
- WrapPanel's outer main-axis size is `parent_main_bound`
  unconditionally (does *not* grow to accommodate oversized lines).
  An oversized child paints at its measured extent, which means its
  right edge extends past the WrapPanel's outer rectangle. Whether
  visible clipping occurs is the responsibility of the WrapPanel's
  *parent*: Phase 4 ScrollView clips by definition; a plain HStack
  parent does not. Matches the WPF / Slint / Compose convention
  "overflow is visible unless someone clips" and avoids propagating
  a parent-bound violation up the tree (the WrapPanel itself stays
  within its allocated rectangle as far as its parent is concerned).

  - What you gain: WrapPanel's outer-bounds contract with its
    parent is unchanged from the no-oversized case (claims
    `parent_main_bound`, no more). Parents that need clipping
    (ScrollView) get it by their own clip surface; parents that do
    not (plain HStack) accept visible overflow as the documented
    outcome. The WrapPanel-side rule is simple: outer main-axis =
    `parent_main_bound`, period.
  - What you give up: Authors must understand that an oversized
    child can paint outside the WrapPanel. `dsl_spec.md` §4.10
    pitfalls note documents this alongside the "huge thumbnail"
    pitfall from DD-004.
  - **Technical risk:** Low.

Option B — Clip oversized children at the WrapPanel boundary
- The arrangement pass installs a clip rectangle at the WrapPanel's
  outer main-axis bound; oversized children are visually clipped at
  that boundary (the on-screen rectangle is truncated).

  - What you give up: Silently truncates content — the author who
    intentionally placed an oversized child sees it cut off, with
    no visible signal that more content exists. Conflicts with the
    Option A convention. Also requires the WrapPanel to install a
    clip surface, which Phase 4 ScrollView's clip surface would
    redundantly stack over.

Option C — Grow the WrapPanel main-axis to fit the largest line
- WrapPanel outer main-axis = `max(parent_main_bound,
  max_line_main_extent)`. WrapPanel returns its grown size to the
  parent's layout pass.

  - What you give up: Violates the parent's main-axis bound from
    inside — the parent told WrapPanel "you have W pixels" and
    WrapPanel returns "I actually took W' > W". Cascading upward
    parent-bound violations are exactly the kind of layout
    surprise the bounded-main-axis contract exists to prevent.
    Phase 4 ScrollView would have no way to compose with a
    grow-to-fit WrapPanel — ScrollView assumes WrapPanel respects
    its viewport-width bound.

**Options (unbounded cross-axis parent — corollary of DD-004
Option (a)):**

When the parent's cross-axis is itself unbounded — and the author has
not set `item-cross-size` — each child receives an unbounded cross-
axis constraint. A `Box { aspect: ratio }` child in this state has
both axes unbounded and hits Phase 2 DD-005's
`LayoutError::BoxAspectUnboundedBoth` runtime error, surfaced with
the Box's IR location.

Option A — Treat as *expected* runtime outcome; no new error,
no WrapPanel-side intervention (recommended)
- WrapPanel does not synthesise a cross-axis bound out of nowhere.
  The author must set `item-cross-size` or wrap the WrapPanel in a
  sized parent. The Phase 4 ScrollView gallery use case illustrates
  the resolution path — ScrollView bounds the main axis (= WrapPanel
  width = viewport width) and leaves the cross axis unbounded for
  scroll, but the gallery sub-screen sets `item-cross-size: 88`
  explicitly so the unbounded cross-axis is never the child's bound.

  - **Technical risk:** Low. Existing Phase 2 error fires.

Option B — Add a WrapPanel-specific variant
(`LayoutError::WrapPanelUnboundedCrossWithAspectChild`)
- Replace Phase 2's Box-side error with a WrapPanel-aware one that
  names the WrapPanel as well as the Box.

  - What you give up: New variant duplicates an existing error
    class. The Phase 2 error already surfaces the Box's IR location;
    a layered backtrace of "Box inside WrapPanel that inherited
    unbounded cross from parent" is a diagnostic-quality
    improvement, not a structural one — defer to a future
    diagnostic-surface phase rather than fold into Phase 3.

**Recommendation:** Option A across the unbounded-main / LayoutError /
unbounded-cross / oversized-first-child / oversized-line sub-issues.
Beyond those, the full Phase 3 algorithm is:

1. **Bounded main-axis parent (happy path).** Children are measured
   against an unbounded main-axis constraint (per DD-001) and a
   DD-004-defined cross-axis constraint. The line breaker greedily
   appends children to the current line. The acceptance rule is
   two-cased:

   - **First child of a line (`line_empty == true`).** The candidate
     is placed unconditionally — the inequality below is *not*
     consulted. The line's recorded main extent equals the child's
     intrinsic main extent and may exceed `parent_main_bound`
     (per oversized-first-child Option A).
   - **Subsequent children of the same line (`line_empty == false`).**
     The candidate is placed iff the inequality

     ```
     current_line_main + item_spacing
       + next_child_main_intrinsic
       <= parent_main_bound
     ```

     holds. When it fails, a new line starts and the candidate
     becomes the first child of that new line (the unconditional-
     placement rule then applies).

2. **Cross-axis line sizing.** Depends on DD-004's `item-cross-size`:
   - When set: each child receives `item-cross-size` as its
     cross-axis bound; the line's cross-axis extent is exactly
     `item-cross-size`. A `Box { aspect: num:den }` child derives
     main-axis extent = `item-cross-size × num / den` per Phase 2
     DD-005's bounded-axis-wins rule.
   - When unset: each child receives the parent's cross-axis
     constraint as its cross-axis bound (the WrapPanel-level
     passthrough). The line's cross-axis extent is the max of
     children's reported cross-axis sizes. A `Box { aspect: num:den }`
     child derives main-axis extent = `parent_cross × num / den` per
     Phase 2 DD-005 — the "huge thumbnail" path DD-004's pitfall
     note warns about.

3. **WrapPanel outer cross-axis size.** Sum of line cross-axis
   extents plus `line_spacing × (line_count − 1)` (per DD-003's
   line-spacing semantics: no trailing margin after the last line).

4. **WrapPanel outer main-axis size.** Equals `parent_main_bound`
   when bounded — unconditionally, even when one or more lines
   contain an oversized first child whose intrinsic extent exceeds
   `parent_main_bound` (per oversized-line Option A; the WrapPanel
   does not grow upward to accommodate oversized children).
   One-line-flow under unbounded-main-axis claims the cumulative
   intrinsic instead.

   **Visible overflow of oversized children.** When an oversized
   first-child's intrinsic main extent exceeds `parent_main_bound`,
   the child paints at its measured extent — its right edge extends
   past the WrapPanel's outer main-axis bound. WrapPanel does *not*
   install a clip surface for this case; visible clipping is the
   responsibility of an enclosing parent that supplies one
   (Phase 4 ScrollView is the canonical example; a plain HStack
   parent does not clip and visible overflow remains visible).
   This is the WPF / Slint / Compose "overflow is visible unless
   someone clips" convention.

5. **Per-line cross-axis item alignment.** Heterogeneous-cross-axis
   line members are centred within the line per DD-001 Option A.

6. **Spacing interaction with overflow comparison.** The inequality
   in step 1 above is normative for the `line_empty == false` case
   (subsequent children); the `line_empty == true` case bypasses the
   inequality unconditionally. No trailing `item_spacing` accrues
   after the last child of a line. Total WrapPanel main-axis used by
   *content* is the max over lines of their cumulative extents
   (bounded by `parent_main_bound` only when no line contains an
   oversized first child; otherwise unbounded above by the line's
   oversized child). WrapPanel's outer main-axis size (step 4)
   remains `parent_main_bound` regardless.

7. **Rounding contract.** Inherits Phase 2 DD-M3-P2-005's discipline:
   parent bounds enter as `f32`; integer comparisons on main-axis
   budget are computed in `f32` directly (spacing values are `i32`,
   promoted to `f32` for the comparison; child intrinsic sizes are
   `f32` from the layout engine). No pixel-snapping in Phase 3.

8. **LayoutError surface.** No new `LayoutError` variant in Phase 3
   (consequent on Option A under the unbounded-main-axis sub-issue).
   The unbounded-cross-axis-with-aspect-child case fires Phase 2's
   existing `LayoutError::BoxAspectUnboundedBoth`. ABI / host-visible
   surface remains internal (no `WASAMO_LAYOUT_ERROR_*` extension
   in Phase 3); the Box-side error class is host-internal for now per
   the Phase 2 precedent.

**Forward-compat exposure:** The recommendation is dual-compatible
with the foreseeable future events:

- Phase 4 ScrollView pairs `ScrollView { WrapPanel { … } }`.
  ScrollView bounds the main axis (viewport width) and leaves the
  cross axis unbounded for scroll. The gallery sub-screen sets
  `item-cross-size` explicitly, so the unbounded cross is never the
  child's bound. The one-line-flow degeneration is unreachable from
  this pairing (ScrollView always supplies a main-axis bound).
- Phase 5 Grid lands as the second novel-normative-spec phase. Grid
  rehearses the spec-drafting discipline started here; the Phase 3
  spec text is not a constraint on Grid's algorithm. WrapPanel may
  appear inside a Grid cell, in which case the intrinsic-sizing
  measure pass that derives the cell's width feeds WrapPanel a
  bounded main-axis on the second pass — the one-line-flow branch
  may briefly engage during the first (intrinsic) pass, which is
  the canonical use Option A is designed for.
- Phase 6 ZStack lands; WrapPanel may appear as a ZStack child. The
  algorithm is unaffected — ZStack passes parent bounds through.
- Phase 7 iteration grammar lands. Generated children become regular
  WrapPanel children; the line breaker is iteration-agnostic.

Option B (unbounded-main runtime error) would have forced every
intrinsic-sizing context (Grid cell width derivation in Phase 5; any
future host-driven measure pass) to pre-check before invoking
WrapPanel measure, an asymmetric constraint not imposed on
HStack / VStack. Option B's exposure is real and not free.

---
