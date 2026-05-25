# M3-Phase 3 — WrapPanel layout primitive: Architecture Decisions

**Phase:** M3-Phase 3 (WrapPanel layout primitive)
**Date:** 2026-05-21
**Status:** Accepted

## Context

M3 acceptance criterion **A3** (see
[ROADMAP.md M3](../../ROADMAP.md#m3-dsl-surface),
[m3-plan.md §Acceptance criteria](../plans/m3-plan.md#acceptance-criteria)):

> WrapPanel layout primitive, demonstrating that DSL can express a
> two-stage measure-arrange — linear main-axis placement plus
> cross-axis wrap on main-axis overflow — that goes beyond the linear
> arrangement (HStack / VStack) established in M2.

The pre-doc framing for this phase was aligned with the owner on
2026-05-21 and is recorded in
[docs/notes/m3-phase-3/pre-doc-framing.md](../notes/m3-phase-3/pre-doc-framing.md).
That framing fixed the 6-DD slate carried below, the visible-proof
location (framing decision E — grow Phase 2's `examples/gallery/` +
`examples/gallery-rust/` sub-screen into a WrapPanel of Boxes), the
verification-strategy menu picks (framing decision C), and the
two upstream-document-revision moments inherited verbatim from
Phase 2 (framing decision D).

Per the M2-Phase 2 framing decision D postmortem
([m3-phase-2 framing notes](../notes/m3-phase-2/m3-phase-2-pre-doc-framing.md)),
the "Moment is not a commit unit" rule applies from the start of
Phase 3: each upstream-document edit in a Moment lands as its own
commit on the pre-doc branch, scoped by review concern, not bundled.

The M2/M3-Phase-1/M3-Phase-2 end-state shape that this phase extends
without breaking:

- `wasamo-ir` ([wasamo-ir/src/lib.rs](../../wasamo-ir/src/lib.rs)):
  `IrType` is `I32 | Str | Bool`; `IrLiteral` is `Int | Str | Ident |
  Bool | Ratio | Color`. The `Ratio` / `Color` literals were added in
  Phase 2 (DD-M3-P2-002 / DD-M3-P2-003) as Box-internal domain types
  with no `PropertyValue` widening. Phase 3 reuses existing `i32`
  plumbing for all WrapPanel attributes; no new literal form is
  introduced.
- `wasamo-runtime` widget catalog
  ([wasamo-runtime/src/widget.rs](../../wasamo-runtime/src/widget.rs)):
  `Rectangle | VStack | HStack | Text | Button | Box`. `PropertyValue`
  enum is `I32(i32) | String(String) | Bool(bool)`; no new variant
  added in Phase 2. Phase 3 adds `WrapPanel` as a per-kind tag (DD-001).
- Layout engine
  ([wasamo-runtime/src/layout.rs](../../wasamo-runtime/src/layout.rs)):
  pure-data `LayoutNode` / `measure` / `arrange` boundary, Win32/WinRT-
  free. Phase 2 introduced `LayoutError::{BoxAspectUnboundedBoth,
  BoxNoExtent}`; Phase 3 inherits the error class and may extend it
  conditionally (DD-005).
- Binding pipeline: per-type writer seam pattern (DD-M3-P1-007). All
  Phase 3 WrapPanel attributes are constant-only (per DD-002 / DD-003
  / DD-004 below), so no new seam triple is built. F5 (`TypedValue`
  deferral) is held in force by construction.
- `wasamoc` ([wasamoc/src/check.rs](../../wasamoc/src/check.rs)):
  state-name → declared-type table; identifier resolution lowers to
  typed `*PropRead` variants. Phase 3 adds no new value type; the
  WrapPanel attributes are integer literals and `wasamoc check` rejects
  negative literals via the existing diagnostic surface.

This ADR is framed against A3 and the m3-plan's "first M3 phase to
introduce novel normative measure-arrange spec in `docs/dsl_spec.md`"
designation
([m3-plan.md §Phase breakdown](../plans/m3-plan.md#phase-breakdown)).
It does **not** re-open F5 (`TypedValue` deferral) — every attribute
the WrapPanel exposes ships constant-only; bindable surface is
deferred per attribute to the phase that first needs it. Image-widget
deferral remains in force, carried by Phase 2's DD-M3-P2-006 placeholder
pattern; Phase 3 simply lays out instances of the pattern in a wrap.

The acceptance lens for this phase: A3 is satisfied when (i) `.ui`
declares `WrapPanel { item-cross-size: <i32>; item-spacing: <i32>;
line-spacing: <i32>; <Box-children> }` and the shared crates lower →
load → render it with correct two-stage measure-arrange, (ii) the
WrapPanel chapter lands in `docs/dsl_spec.md` §4.10 as a normative
spec at the milestone-end criteria bar
([m3-plan.md §Milestone-end criteria item 5](../plans/m3-plan.md#milestone-end-criteria))
applied at phase close, and (iii) `examples/gallery/` +
`examples/gallery-rust/` are grown additively from Phase 2's single-Box
sub-screen into a WrapPanel of Box thumbnails. Per A11, all sides
advance together by phase close.

### Governance note (M3 phase-ADR vs RFC transition)

[VISION.md §9.2](../../VISION.md#92-decision-making) describes a
"gradual transition to RFC-based consensus" for M3 onward, with
major changes discussed in `docs/rfcs/`.
[docs/decisions/README.md §Scope and relation to RFCs](./README.md#scope-and-relation-to-rfcs)
echoes the same wording. Both texts treat M1/M2 phase-ADR practice
as the pre-transition baseline.

In observed practice, M3 Phase 1
([m3-phase-1-bool-scalar.md](./m3-phase-1-bool-scalar.md)) and
M3 Phase 2 ([m3-phase-2-box-layout.md](./m3-phase-2-box-layout.md))
both ran as phase ADRs without invoking the RFC process;
`docs/rfcs/` does not yet exist in the repository, and the
RFC-process content (template, lifecycle, acceptance rule) has not
been written. This Phase 3 ADR continues the M3 phase-ADR pattern
Phase 1 / Phase 2 established, for consistency with the realised
M3 flow and because the RFC machinery is not in place.

The disconnect between VISION / decisions-README text ("transition
to RFC") and the observed M3 phase-ADR practice is noted here so
it is not silently propagated through Phase 3. **Resolving the
disconnect is out of scope for this Phase 3 ADR** and warrants a
separate vision ADR — either revising VISION §9.2 / decisions-README
§Scope to match the realised phase-ADR flow (deferring RFC adoption
to a later milestone), or standing up `docs/rfcs/` and re-routing
M3 phases through it (which would retroactively reframe M3 Phase 1 /
Phase 2 / Phase 3 as transitional).

**Owner-confirmation request:** this ADR is *Proposed* on the
assumption that the phase-ADR path is acceptable for Phase 3, on
parity with M3 Phase 1 / Phase 2. If the owner instead wants
Phase 3 gated on RFC-process setup, this ADR's `Status: Accepted`
flip blocks until the governance question is resolved upstream.

**Resolution (post-hoc, 2026-05-25).** The upstream governance
question was resolved by
[vision-governance-rfc-deferral.md DD-V-018](./vision-governance-rfc-deferral.md#dd-v-018--defer-rfc-adoption-to-post-10),
which collapsed the three-stage governance trajectory into two
stages (pre-1.0 BDFL + ADRs, post-1.0 open governance + RFC
machinery introduced together). The phase-ADR path was therefore
the correct choice for Phase 3 and remains the path for the
remainder of the pre-1.0 period. This historical note is left
intact per the supersede rule.

### Layering note (DD-001 ⇄ DD-004 ⇄ DD-005)

Phase 3's structural DD (DD-001 — IR shape + child layout contract),
its item-sizing-source DD (DD-004 — cross-axis bound), and its
algorithmic DD (DD-005 — measure-arrange algorithm) are **layered**,
with the dependency direction inverted relative to Phase 2:

- **Phase 2 (Box).** Outer bounds resolve *without* considering child
  intrinsic size when `aspect` is set. Aspect-derived bounds win;
  children do not grow the Box. Outer first, inner second.
- **Phase 3 (WrapPanel).** WrapPanel's outer bounds *do* depend on
  children. Each child's main-axis intrinsic size determines which
  line it joins; each line's cross-axis extent depends on the children
  in that line (and on DD-004's `item-cross-size`, when set);
  WrapPanel's cross-axis outer size is the sum of line cross-axis
  extents (plus any line-spacing). The main-axis outer size is bounded
  by the parent's main-axis constraint. **Inner first, outer second**
  on the cross axis; outer-equals-constraint on the main axis.

DD-004 sits *between* DD-001 and DD-005 in the chain: DD-001 says
"children measured against unbounded main-axis + cross-axis
passthrough"; DD-004 settles *which* cross-axis is passed through
(parent's, or `item-cross-size`-bounded); DD-005 then runs the line
breaker on the resulting child measures.

This inverted layering is the structural content of "two-stage
measure-arrange". Each DD's Recommendation prose cites the layering
so reviewers can verify Option respect for the dependency direction.

Concrete consequence: the following combinations are **invalid** and
do not appear as recommended options —

- DD-005 = "WrapPanel cross-axis ignores children" with any DD-001
  multi-child Option (would make WrapPanel a Grid row, not a
  WrapPanel).
- DD-001 = "children share full WrapPanel main-axis bounds" with any
  DD-005 wrapping algorithm (would make wrapping structurally
  unreachable).
- DD-004 = "no cross-axis constraint policy at all" with any DD-005
  line cross-axis sizing Option (DD-005's line breaker needs a defined
  cross-axis-bound source, even when that source happens to be
  unbounded).

---

### DD-M3-P3-001 — WrapPanel IR node form and N-child main-axis flow contract

**Status:** Accepted

**Context:**
WrapPanel is a new layout primitive in `wasamo-ir` and
`wasamo-runtime`. Phase 3 must commit to (i) the IR node shape,
(ii) the 0-child / 1-child / N-child shapes, (iii) child measure
input, (iv) line membership rule (the overflow comparison), and
(v) cross-axis alignment of items within a line.

**Options (IR node shape):**

Option A — Per-kind tag parallel to `HStack` / `VStack` /
`Rectangle` / `Box` (recommended)
- `WidgetKind::WrapPanel` joins the existing per-kind enumeration;
  the layout function in `wasamo-runtime` dispatches on the tag.
  Phase 2 DD-M3-P2-001 settled the per-kind-tag answer for Box;
  Phase 3 inherits unless evidence forces re-opening.

  - What you gain: Symmetric with every existing M2/M3 widget.
    Pattern matching on `WidgetKind` remains exhaustive at compile
    time.
  - What you give up: One more tag at every `WidgetKind` match site.
    Set is small and discoverable.
  - **Technical risk:** Low.

Option B — Structural variant in an `IrLayout` umbrella
- Re-opens the M2-Phase-6 / M3-Phase-2 layout-family question
  ("would HStack / VStack / Box / WrapPanel benefit from being
  grouped under a structural family enum?") mid-milestone for one new
  widget.

  - What you give up: No payoff inside Phase 3, and the regrouping
    would touch every M2/M3-Phase-2 layout dispatch site. The
    discipline of "build the seam in the phase that needs it" applies:
    no phase yet needs the umbrella.
  - **Technical risk:** Medium.

**Options (0-child / 1-child shape):**

Option A — Both valid; 0-child produces a zero-extent line set,
1-child produces a one-line layout (recommended)
- The IR loader admits empty `children` lists and single-child lists;
  the layout pass produces a zero-line / one-line outcome
  respectively. Consistent with Phase 2 Box's 0-child shape rule.

  - What you gain: Author can write `WrapPanel { … }` with a list
    being populated in iteration order (relevant once Phase 7
    iteration lands) without the IR loader rejecting the in-progress
    shape.
  - **Technical risk:** Low.

Option B — Reject 0-child WrapPanel at IR load; warn / reject 1-child
WrapPanel as "use Box / HStack instead"
- Diagnoses empty / degenerate WrapPanel as a `wasamoc check`
  warning or error.

  - What you give up: 0-child WrapPanel has a real use under future
    iteration (an empty collection yields zero items); rejecting it
    at IR-load contradicts the iteration grammar's future shape.
    1-child WrapPanel is structurally meaningful (the wrap is
    conditional on multi-item content; one item is the boundary case
    of "no wrap yet").

**Options (N-child main-axis flow / overflow comparison):**

Option A — Document-order placement; spacing-aware overflow inequality
with a `line_empty` carve-out (recommended)
- Children are placed in document order along the main axis. When
  the current line already has at least one child
  (`line_empty == false`), the next child is accepted onto the line
  iff

  ```
  current_line_main + item_spacing
    + next_child_main_intrinsic
    <= parent_main_bound
  ```

  When the inequality fails, the next child starts a new line. The
  **first child of any line** (`line_empty == true`) is placed
  unconditionally — even if its intrinsic main-axis extent alone
  exceeds `parent_main_bound`. The line's recorded extent may then
  exceed the bound (the "oversized first-child" case spec'd in DD-005;
  visible overflow handling and the WrapPanel outer-main-axis rule
  are DD-005's "oversized line — arrangement / paint clip" sub-issue).
  No trailing `item_spacing` accrues after the last child of a line.
  DD-001 records the **contract** (document-order, spacing-aware,
  unconditional first-child placement); DD-005 records the
  algorithmic statement and the WrapPanel-outer-bounds consequence.

  - What you gain: Eliminates "trailing margin" ambiguity. Deterministic
    against floating-point intrinsic sizes (the only float in the
    inequality is `next_child_main_intrinsic`; integer literals are
    promoted symmetrically).
  - **Technical risk:** Low.

Option B — Strict greater-than (next-child-must-strictly-fit)
- Drops the `<=` in favour of `<`. A line that ends exactly at
  `parent_main_bound` rejects any further child even if that child
  has zero intrinsic main-axis.

  - What you give up: Zero-width children (degenerate case; not
    expected in Phase 3's sub-screen but possible once iteration
    lands) would force a new line redundantly. The behavioural
    difference is invisible in any realistic test fixture but the
    spec would have to explain it.

**Options (child measure input):**

Option A — Unbounded main-axis + DD-004-defined cross-axis (recommended)
- Each child is measured against an **unbounded main-axis constraint**
  (so the child reports its intrinsic main-axis size, which feeds
  line membership) and a **cross-axis constraint defined by DD-004**
  — either WrapPanel's `item-cross-size` when set, or the parent's
  cross-axis constraint passed through when unset.

  - What you gain: The line breaker has stable input regardless of
    whether the WrapPanel sits inside a bounded or unbounded
    cross-axis parent (the answer is determined by DD-004, not by an
    implicit re-derivation here). Honours the layering note: DD-001
    consumes DD-004's settlement; it does not duplicate it.
  - **Technical risk:** Low.

Option B — Each child receives a slot-sized main-axis constraint
- Each child gets `parent_main_bound / N` (or similar) as its
  main-axis measure input.

  - What you give up: This is the Grid cell semantic. A WrapPanel
    whose children all measure to the same main-axis extent reduces
    the line breaker to a counting exercise — line membership becomes
    "how many fit per row, all the same width". Reject as a back-door
    Grid for the same reason Phase 2 rejected back-door ZStack.
    Contradicts the layering note's DD-001 → DD-005 input shape.

**Options (cross-axis item alignment within a line):**

Option A — Center (recommended, mirrors Phase 2 Box DD-001 default)
- When a line's members have heterogeneous cross-axis sizes
  (smaller items shorter than the line max), smaller items are
  centred on the cross axis within the line. No per-child override
  in Phase 3.

  - What you gain: Matches Phase 2 Box's `align: center` default.
    Reader builds one mental model for "where does the smaller thing
    sit?" across both primitives. The Phase 3 sub-screen uses uniform
    1:1 thumbnails (cross-axis sizes match by construction), so the
    default has no *observable* effect in the gallery proof — Phase 3
    settles the default for the eventual heterogeneous-line case
    rather than reserving no surface.
  - **Technical risk:** Low.

Option B — Start (top in horizontal main-axis)
- Smaller items anchor at the cross-axis-start edge of the line.

  - What you give up: Contradicts Phase 2 Box's alignment default
    without justification. A heterogeneous line of mixed-aspect
    thumbnails would top-align rather than centre, which is the
    less-common photo-gallery convention.

Option C — Configurable per-child via a new attribute
- Add `align: <center|start|...>` to children of WrapPanel.

  - What you give up: New attribute surface unmotivated by any Phase
    3 acceptance criterion. Out of phase scope; defer to the phase
    that needs it.

**Recommendation:** Option A for every sub-issue —

- IR shape: per-kind tag (`WidgetKind::WrapPanel`).
- 0-child / 1-child: both valid.
- N-child main-axis flow: document-order placement with spacing-aware
  `<=` overflow inequality (the inequality is normatively defined in
  DD-005's "Spacing interaction with overflow comparison" sub-issue;
  this DD records the contract).
- Child measure input: unbounded main-axis + DD-004-defined
  cross-axis.
- Cross-axis item alignment: centred within line.

Design quality dominates on the child-measure-input sub-issue: Option B
(slot-sized main-axis) would degenerate WrapPanel into Grid territory
and contradict the layering note. The other sub-issues follow Phase 2's
established discipline (per-kind tag, both empty-and-1-child valid,
centred alignment as the no-override default).

**Forward-compat exposure:** Option A's exposure under foreseeable
future events (see Out of scope):

- Phase 5 Grid lands with its own per-cell measure semantics. The
  per-kind tag stays additive; Grid does not reuse WrapPanel's
  `WidgetKind` slot.
- Phase 7 iteration grammar lands. Iteration generates children into
  WrapPanel; the 0-child shape (an empty iteration result) and the
  document-order placement contract both survive.
- A future per-child `align` attribute is additive — Phase 3's
  centred default does not foreclose Option C if a later phase
  surfaces a heterogeneous-alignment use case.

---

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

### DD-M3-P3-003 — Spacing attributes (item-spacing, line-spacing, padding)

**Status:** Accepted

**Context:**
Whether Phase 3 exposes item spacing (main-axis gap between siblings
within a line), line spacing (cross-axis gap between lines), padding
(inset between WrapPanel bounds and the line set), or none.

Spacing is the *gap between* items, not the *size of* items. The
item-size source is DD-004's question; once items have a size, spacing
decides what visible gap separates them. The wireframe's 12px gap
(see [docs/references/m3-gallery-wireframe.html](../references/m3-gallery-wireframe.html))
is therefore a DD-003 question conditional on DD-004 settling the
88×88 thumbnail extent. If DD-004 ships zero item-sizing attribute,
DD-003's spacing has no thumbnails to space.

**Options (item-spacing and line-spacing — surface scope):**

Option A — Ship `item-spacing: <i32>` and `line-spacing: <i32>` as
constant-only attributes, default `0` (recommended)
- Two new optional attributes on the WrapPanel IR node, both
  carrying `i32` pixel values. `item-spacing` separates siblings
  within a line on the main axis; `line-spacing` separates lines
  on the cross axis. Defaults: 0 (touching items / lines if
  unset). `wasamoc check` rejects negative literals; `validate()`
  rejects negative IR (DD-006). No new `IrType`, no new
  `IrLiteral` variant — `i32` plumbing already exists from M2 /
  Phase 1.

  - What you gain: Reuses existing `i32` plumbing — lex, parse,
    check, IR literal form, runtime decode all unchanged from M2.
    The Phase 3 gallery sub-screen can express the wireframe's 12px
    gap (`item-spacing: 12; line-spacing: 12`). Default 0 is
    visible-by-construction (touching items if unset, not silent
    invisibility).
  - What you give up: Two new attributes — but each is an additive
    optional attribute, not a structural surface change.
  - **Technical risk:** Low.

Option B — Ship neither; Phase 3 sub-screen accepts touching thumbnails
- WrapPanel has no spacing attributes. The gallery sub-screen
  visually deviates from the wireframe (no gaps between thumbnails)
  but the layout primitive ships smaller.

  - What you give up: Visible deviation from the wireframe in the
    sub-screen. The deviation is recoverable (a later phase adds
    spacing), but the gallery proof becomes "WrapPanel wraps" rather
    than "WrapPanel produces the wireframe's layout". The
    presence-of-gap question is structural to author intuition about
    "what does a WrapPanel look like"; deferring it creates an
    expectations-mismatch for any reader coming from CSS / WPF / XAML
    where gap is table stakes.

Option C — Ship `item-spacing` only; defer `line-spacing` to a later phase
- One attribute, not two. The wireframe's cross-axis gap between
  lines is unreachable in Phase 3.

  - What you give up: Visual asymmetry — main-axis gaps present,
    cross-axis gaps absent. The wireframe shows both. No spec
    saving relative to Option A — the lex / parse / check for the
    second attribute is mechanical.

Option D — Single combined `spacing: <i32>` attribute
(applies to both axes uniformly)
- One attribute setting the same value on both axes.

  - What you give up: Forces authors with mixed-axis gap intent
    (e.g. tight horizontal grid, looser vertical separation) to
    pick one. A later phase that splits the attribute then has a
    deprecation problem.

**Options (surface form, conditional on Option A):**

Option A — Bare integer literal `item-spacing: 12` (recommended)
- Per Phase 2 DD-M3-P2-002 / DD-M3-P2-003 discipline, no new
  `PropertyValue` variant unless the attribute is bindable —
  integer pixel spacing reuses `IrLiteral::Int` and the existing
  `i32` plumbing.

  - **Technical risk:** Low.

Option B — Pair literal `spacing: 12 12` (main cross)
- A structural pair literal, akin to DD-002's ratio form.

  - What you give up: New literal form for one attribute that has
    no obvious "the two values are inseparable" reading. The
    wireframe's gaps are equal but conceptually two different
    distances. Reject as over-engineering for the value shape.

**Options (bindable surface, conditional on Option A):**

Option A — Constant-only in Phase 3 (recommended)
- Phase 3 mirrors DD-002's stance and Phase 1 / Phase 2 seam-
  building discipline. Phase 3 sub-screen has no animated spacing
  use case.

  - What you gain: No new per-type writer pair built. Phase 3 reuses
    the existing `i32` literal plumbing; a future bindable-spacing
    phase either reuses the M2 string-baked `register_binding` path
    that `IrType::I32` properties currently dispatch to, or opens a
    typed-`i32` evaluator/writer pair if that phase warrants it.
    Phase 3 itself adds no engine plumbing.
  - **Technical risk:** Low.

Option B — Admit bindable spacing in Phase 3
- `bind item-spacing: <state-of-int>` works.

  - What you give up: Speculative seam *registration* for an
    attribute no Phase 3 sub-screen exercises reactively. Phase 1
    discipline argues against.

**Options (padding):**

Option A — Defer padding to a later phase (recommended)
- WrapPanel has no padding attribute in Phase 3. The Phase 3
  sub-screen accepts whatever left-edge behaviour the bare WrapPanel
  default produces (children flush with WrapPanel's main-axis-start
  edge).

  - What you gain: Smaller Phase 3 surface. Padding is a parent /
    container concept that interacts with Phase 4 ScrollView's clip
    surface and any future M4+ layout-with-margin work; settling
    its semantics deserves its own phase. The wireframe's left-edge
    margin (x=36 in a 20-padded frame) is achievable in Phase 3 by
    wrapping the WrapPanel in an outer HStack with a spacer — not
    elegant, but it ships A3 without expanding scope.
  - What you give up: The sub-screen has no first-class way to
    express the wireframe's left-edge margin; either it visually
    deviates or it composes with an outer wrapper. Framing decision E
    accepts the visual deviation as in-scope.

Option B — Ship `padding: <i32>` (uniform inset) in Phase 3
- One scalar inset applied to all four edges.

  - What you give up: Commits Phase 3 to a uniform-inset reading;
    a later phase that needs per-edge padding (`padding-left`,
    `padding-top`, …) either re-spec's the attribute or coexists
    awkwardly. Better to defer the whole question.

Option C — Ship 4-tuple padding `padding: <top> <right> <bottom> <left>`
- CSS-style four-edge padding.

  - What you give up: New literal form (a 4-tuple) for an attribute
    no Phase 3 sub-screen requires. Defer.

**Recommendation:** Option A for every sub-issue —

- Ship `item-spacing: <i32>` and `line-spacing: <i32>` as
  constant-only attributes; default 0 for both.
- Surface form: bare integer literal; reuses existing `i32`
  plumbing; no new `PropertyValue` variant.
- Bindable: constant-only in Phase 3.
- Padding: defer.

The spacing question turns on the wireframe-fidelity tension and the
surface-cost asymmetry. Option A pays a small attribute-plumbing cost
(two `i32` attributes, both reusing existing plumbing) for full
wireframe fidelity; Option B saves the plumbing at the cost of a
visibly-degraded sub-screen. The framing's working direction is Option
A, but the trade-off is a value judgement; **see Owner-agreement
checkpoint 1**.

**Forward-compat exposure:** Option A is dual-compatible with both
foreseeable future events:

- A future bindable-spacing phase admits binding for the attribute
  at that point. It can reuse the M2 string-baked `register_binding`
  path that `IrType::I32` properties currently dispatch to, or open
  a typed-`i32` evaluator/writer pair if that phase warrants it;
  no revision of the Phase 3 IR shape or the spacing semantics is
  required.
- A future padding-introducing phase adds a separate attribute (or
  attribute group); the absence of padding in Phase 3 does not
  pressure the eventual padding surface to be backward-compatible
  with any Phase-3 convention.

Option B / C / D would have committed Phase 3 to surface shapes
(no spacing / one-axis-only spacing / combined spacing) that either
demand visible-deviation acceptance or constrain a later split.

---

### DD-M3-P3-004 — Item sizing source (WrapPanel item cross-axis bound)

**Status:** Accepted

**Context:**
WrapPanel measures children to determine line breaks; the child
measure constraint must come from somewhere. The Phase 3 wireframe's
88×88 thumbnail (`Box { aspect: 1:1; fill: …; Text { … } }`) has
**no intrinsic size of its own** — Phase 2 DD-M3-P2-005's aspect
projection derives the unbounded axis from the bounded axis, so the
Box needs *one* bounded axis as input. If WrapPanel passes the
parent's full cross-axis constraint through to each child, a 1:1 Box
thumbnail in an 800×600 window inherits ~600 cross-axis bound and
grows to ~600×600 — not 88×88. The DD settles where the per-item
cross-axis bound comes from. This is the load-bearing question for
the gallery sub-screen's *visible* correctness.

**Options (attribute exposure):**

Option A — Expose `item-cross-size: <i32>` (working name) as an
optional constant-only attribute (recommended)
- A new optional `i32` attribute on the WrapPanel IR node carrying
  the cross-axis bound passed to each child during measure. Name is
  orientation-neutral so a later phase that admits vertical
  orientation does not need to rename. Alternative names rejected:
  `item-cross-extent` (verbose), `item-height` (orientation-coupled
  WPF-style — rejected for the same reason DD-002 defers orientation),
  `cell-size` (Grid-flavoured — confusable), `thumbnail-size`
  (use-case-specific).

  - What you gain: An explicit, orientation-neutral knob for the
    per-item cross-axis bound. The Phase 3 gallery sub-screen sets
    `item-cross-size: 88` and gets 88×88 thumbnails from 1:1 Boxes
    deterministically. No new `IrType`, no new `IrLiteral` variant
    — `i32` plumbing already exists.
  - **Technical risk:** Low.

Option B — No item-sizing attribute on WrapPanel; require authors
to size each Box child explicitly
- WrapPanel never supplies a per-item cross-axis bound; each child
  must carry intrinsic size of its own (i.e. each Box thumbnail
  carries explicit `width: 88; height: 88`).

  - What you give up: `width` / `height` on Box are out of Phase 2
    DSL surface (Phase 2 DD-M3-P2-005 forward-looking-only). Adopting
    Option B would force opening those attributes in Phase 3 with
    no DD pre-doc'd for them — scope creep. Also: the gallery
    sub-screen would have to re-specify 88 at every thumbnail
    instead of once at the WrapPanel level, which is exactly the
    repetition pattern that motivates container-level item sizing
    in WPF / Slint / Compose.

**Options (default behaviour when `item-cross-size` is unset):**

Option (a) — Pass parent's full cross-axis constraint through to
each child (WPF / Slint precedent, recommended)
- When `item-cross-size` is unset, each child receives the parent's
  cross-axis constraint as its cross-axis measure input. A `Box {
  aspect: 1:1 }` thumbnail in a parent with `H=600` measures to
  `600 × 600` (not the wireframe's 88×88, but a defensible reading
  of "no override → no transformation").

  - What you gain: Composes naturally with sized parents. A
    WrapPanel inside a fixed-height container without
    `item-cross-size` produces children sized to the container —
    matches WPF / Slint "no override → full passthrough" intuition.
  - What you give up: An aspect-only Box thumbnail in a tall
    WrapPanel becomes huge — author-facing footgun. The
    `dsl_spec.md` chapter contains a "common pitfalls" note pointing
    aspect-only-Box authors at the attribute.
  - **Technical risk:** Low.

Option (b) — Measure with unbounded cross-axis
- When `item-cross-size` is unset, each child receives an unbounded
  cross-axis constraint. An aspect-only Box then has both axes
  unbounded and immediately hits Phase 2's
  `LayoutError::BoxAspectUnboundedBoth`.

  - What you gain: No silent "huge thumbnail" outcome — authors
    using aspect-only children without `item-cross-size` get an
    error directly.
  - What you give up: WrapPanel-in-a-sized-container with
    natural-cross-axis children (e.g. Text-only children measuring
    cross-axis from font) loses access to the container's
    cross-axis bound by default — those children measure as if the
    container were unbounded. Counter-intuitive for the common
    case.

Option (c) — `wasamoc check` requires `item-cross-size` when any
direct child uses `aspect`-only sizing
- Compile-time guard: if WrapPanel has no `item-cross-size` and at
  least one direct child is a Box with `aspect` and no other size
  source, `wasamoc check` rejects.

  - What you gain: Footgun caught at author time.
  - What you give up: `wasamoc check` must statically classify
    children by "size source" (aspect-only Box vs natural-intrinsic
    Text vs future non-Box children). The classifier scales poorly
    as the widget catalogue grows. Too strong for a general WrapPanel
    surface.

**Options (per-line cross-axis sizing rule when `item-cross-size` is set):**

Option A — Line cross-axis extent equals `item-cross-size` uniformly
(WPF-`ItemHeight`-style, recommended)
- Every line's cross-axis extent is exactly `item-cross-size`
  regardless of how much each child consumes. Smaller children
  align (per DD-001's centred default) within the line; larger
  children clip (consistent with Phase 2 Box's overflow rule).

  - What you gain: Deterministic line extent. WrapPanel's outer
    cross-axis size is `(line_count × item-cross-size) +
    (line_count − 1) × line_spacing` — directly computable. Matches
    WPF semantics on the *visible* axis: when authors set
    `ItemHeight`, lines are that tall.
  - **Technical risk:** Low.

Option B — Line cross-axis extent is the max of children's
*reported* cross-axis sizes (intrinsic-driven, Slint-style)
- The line's extent collapses to whatever the children actually
  consumed, ignoring `item-cross-size` as a line-extent setter.

  - What you give up: `item-cross-size` becomes purely a measure-
    input bound, with no effect on visible line extent when children
    measure smaller. Authors who expect "all my thumbnails are 88
    tall" by setting `item-cross-size: 88` get a smaller line when
    a thumbnail happens to report a smaller intrinsic. Surprising;
    contradicts the "uniform thumbnail grid" use case.

**Options (`wasamoc check` warning for aspect-only-Box without
`item-cross-size`):**

Option A — Ship the warning in Phase 3
- When a WrapPanel declares no `item-cross-size` and at least one
  direct child is a Box with `aspect` and no other size source,
  `wasamoc check` emits a **warning** (not error) suggesting the
  attribute. The warning is structurally cheap (it doesn't have to
  be sound across all child shapes — only the known aspect-only-Box
  footgun) and preserves Option (a)'s default.

  - What you gain: Author-time guidance for the most common
    failure mode without committing the spec to "size source
    classification".
  - **Technical risk:** Low.

Option B — Defer the warning to a later phase
- Phase 3 ships Option (a) default with no `wasamoc check`
  warning. The `dsl_spec.md` pitfall note is the only guidance.

  - What you give up: Authors learn the footgun at runtime
    (huge-thumbnail visual outcome) rather than at compile time.
    Recoverable but slower.

**Options (bindable surface):**

Option A — Constant-only in Phase 3 (recommended)
- Mirrors DD-002 / DD-003 / Phase 1 / Phase 2 discipline. No new
  per-type writer seam; `i32` literal plumbing already exists.

  - **Technical risk:** Low.

Option B — Admit bindable `item-cross-size`
- A theme-driven thumbnail size could vary at runtime.

  - What you give up: Speculative seam-registration for an
    attribute no Phase 3 sub-screen exercises reactively. Discipline
    of "build the seam in the phase that needs it" applies.

**Recommendation:** Option A across the attribute-exposure /
default-behaviour / per-line / bindable sub-issues, with the
`wasamoc check` warning shipped in Phase 3 —

- Expose `item-cross-size: <i32>` as an optional constant-only
  attribute.
- Default when unset: Option (a) — pass parent's full cross-axis
  constraint through. This matches WPF / Slint precedent and the
  principle "WrapPanel does not redefine child measure when the
  author has not asked it to". The `dsl_spec.md` chapter contains a
  "common pitfalls" note pointing aspect-only-Box authors at the
  attribute.
- Per-line cross-axis sizing rule when attribute set: uniform —
  line cross-axis extent equals `item-cross-size`.
- `wasamoc check` warning: ship in Phase 3 (Option A under the
  warning sub-issue) — narrow guard for the known aspect-only-Box
  footgun.
- Bindable surface: constant-only.

The default-behaviour Option pick (Option (a) vs (b) vs (c)) is a
load-bearing value judgement; the warning ship/defer pick is a
companion judgement. **See Owner-agreement checkpoint 2**.

The Phase 3 gallery sub-screen sets `item-cross-size: 88` explicitly,
so the WrapPanel of 1:1 Boxes produces 88×88 thumbnails matching the
wireframe (no warning triggered).

**Forward-compat exposure:** Option A's exposure under foreseeable
future events:

- A future bindable-`item-cross-size` phase admits binding for the
  attribute at that point. It can reuse the M2 string-baked
  `register_binding` path that `IrType::I32` properties currently
  dispatch to, or open a typed-`i32` evaluator/writer pair if that
  phase warrants it; no revision of Phase 3 IR shape or default
  behaviour is required.
- A future vertical-orientation phase (DD-002 deferred) does not
  need to rename — `item-cross-size` is orientation-neutral by
  design.
- A future Image-widget phase that gives children natural cross-axis
  size: the "when set, override; when unset, passthrough" semantics
  survive — Image children with natural size measure normally under
  the parent passthrough, and the `item-cross-size` override
  continues to mean "use this as the measure bound".

Option (b) (unbounded default) would have committed Phase 3 to a
counter-intuitive composition with sized parents that the future
Image-widget phase would have had to re-spec around. Option (c)
(compile-time-requires) would have committed `wasamoc check` to a
classifier that scales poorly.

---

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

## Out of scope (for M3-Phase 3; recorded explicitly)

- **ScrollView pairing.** Phase 4. The wireframe shows
  `ScrollView { WrapPanel { … } }` for the overflow state; Phase 3
  ships the WrapPanel only, no viewport / clip / content offset
  binding. The gallery sub-screen's main-axis remains bounded by
  the window directly until Phase 4.
- **Padding attribute on WrapPanel.** Deferred; left-edge behaviour
  in Phase 3 sub-screen accepts the bare-WrapPanel default. Per
  DD-003 Option A's deferral.
- **Per-child main-axis size override** (e.g. "this thumbnail spans
  2 columns"). Grid territory, Phase 5.
- **Per-child cross-axis alignment override attribute** (DD-001
  Option C). The centred default applies uniformly in Phase 3.
- **Iteration grammar.** Phase 7. The wireframe shows generated
  thumbnails, but Phase 3 ships with a hand-written fixed set in the
  gallery sub-screen.
- **Image widget surface.** M4+; placeholder pattern from Phase 2
  DD-M3-P2-006 carries through unchanged.
- **`TypedValue` generic value union.** F5 maintained
  ([m2-to-m3-handover.md §4](../notes/m2-to-m3-handover.md)).
  DD-002 / DD-003 / DD-004's constant-only stances preserve the
  deferral structurally; no Phase 3 attribute pressures `TypedValue`.
- **Bindable surface for any WrapPanel attribute** exposed by
  DD-002 / DD-003 / DD-004. Each DD's bindable sub-issue settles
  constant-only in Phase 3; the per-type writer seam pattern from
  Phase 1 / Phase 2 remains available for the phase that first opens
  a bindable WrapPanel surface.
- **Vertical-main-axis WrapPanel.** DD-002 settles horizontal-only;
  vertical opens additively when a future phase needs it.
- **Per-item explicit dimensions on the Box child** (`width` /
  `height` are out of M3-Phase 2 DSL surface). DD-004 settles the
  Phase 3 sizing source as `item-cross-size` at WrapPanel level,
  not as item-level dimensions on Box.
- **Vertical-scroll viewport in the gallery sub-screen.** Per
  framing decision E: the Phase 3 sub-screen sizes the thumbnail
  set to wrap within the default window (800×600); overflow handling
  arrives with Phase 4 ScrollView.
- **C / Zig host parity for the WrapPanel sub-screen.**
  [m3-plan.md §Phase-end criteria item 5](../plans/m3-plan.md#phase-end-criteria)
  calls for at least one host per phase; Phase 8 broadens the full
  gallery to all three. Phase 3 grows `examples/gallery-rust/` only.
- **WrapPanel-specific layered diagnostics**
  (`LayoutError::WrapPanelUnboundedCrossWithAspectChild` or similar
  layered backtraces). DD-005 Option A under the unbounded-cross
  sub-issue defers diagnostic-quality improvements to a future
  surface phase; Phase 3 lets Phase 2's existing
  `LayoutError::BoxAspectUnboundedBoth` fire with the Box's IR
  location.

### Phase 3 implementation residuals (out-of-phase, filed at close)

Implementation findings that surfaced during Phase 3 T1–T9 but fall
outside this ADR's scope are recorded under
[m3-phase-3-progress.md §Out-of-phase residuals](../plans/progress/m3-phase-3-progress.md#out-of-phase-residuals):

- **R1** — `.gitignore` `*.uic` pattern (cross-cutting build
  hygiene).
- **R2** — `sync_visuals` ↔ pure-layout boundary test gap
  (post-Phase-3 test coverage; the architecture clarification half
  was folded into [docs/architecture.md §6.5](../architecture.md)
  in the T10 close).

## Owner-agreement checkpoints

Two of the DDs above carry value judgements with multiple defensible
answers; they warrant explicit yes/no from the owner before this ADR
moves to Accepted. All other DDs follow mechanically from these two
and from the Phase 2 / Phase 1 inheritance.

### Checkpoint 1 — DD-M3-P3-003 spacing surface scope

**Question:** Does Phase 3 ship `item-spacing: <i32>` and
`line-spacing: <i32>` as constant-only attributes (Option A,
recommended), or does it ship no spacing attributes and accept the
visible deviation from the wireframe in the gallery sub-screen
(Option B)?

**Default answer:** Option A — ship both attributes; default 0;
reuse existing `i32` plumbing.

**Framing for owner:** Option A pays a small attribute-plumbing cost
(two new optional integer attributes on the WrapPanel IR node,
both reusing existing `i32` literal plumbing) for full
wireframe fidelity in the gallery sub-screen. The cost is genuinely
small — no new `IrType`, no new `IrLiteral` variant, no new
`PropertyValue` variant, no ABI surface change. Both attributes are
constant-only; no per-type writer seam built.

Option B saves the plumbing at the cost of a visibly-degraded
sub-screen (touching thumbnails rather than the wireframe's 12px
gap). The deviation is recoverable in a later phase, but the gallery
proof becomes "WrapPanel wraps" rather than "WrapPanel produces the
wireframe's layout". For readers coming from CSS / WPF / XAML
(where gap is table stakes), the absence creates an
expectations-mismatch about what a WrapPanel looks like.

The trade-off is design-quality dominated: Option A is the wireframe-
faithful path with minimal surface cost; Option B is the
minimal-surface path with a visible deviation.

### Checkpoint 2 — DD-M3-P3-004 default behaviour and warning

**Question:** When `item-cross-size` is unset on a WrapPanel, what
does the child receive as its cross-axis measure constraint?
**(a)** parent's full cross-axis constraint passed through
(recommended); **(b)** unbounded cross-axis constraint; **(c)**
`wasamoc check` rejects (compile-time-requires `item-cross-size`).
And: does Phase 3 ship the `wasamoc check` warning for
aspect-only-Box children without `item-cross-size`?

**Default answer:** Option (a) — parent passthrough — plus ship the
warning in Phase 3.

**Framing for owner:** This is the most load-bearing value judgement
in the ADR. The default-behaviour pick determines what happens when an
author writes a WrapPanel of aspect-only Box thumbnails *without*
setting `item-cross-size`:

- Option (a): The thumbnails inherit the parent's cross-axis bound.
  In a tall WrapPanel inside an 800×600 window, a 1:1 Box becomes
  ~600×600 — a single huge thumbnail, with subsequent thumbnails
  pushed onto new lines. The behaviour is *visibly wrong* but
  *deterministically derivable* from the spec; the `dsl_spec.md`
  pitfall note + the `wasamoc check` warning (shipped per the
  companion judgement) point the author at the fix.
- Option (b): The thumbnails immediately hit Phase 2's
  `LayoutError::BoxAspectUnboundedBoth` runtime error. The author
  gets an error rather than a wrong visual, but WrapPanel-in-a-sized-
  container with natural-cross-axis children (Text-only chips
  measuring cross-axis from the font) loses access to the
  container's cross-axis bound — those children measure as if the
  container were unbounded. Counter-intuitive for the common case.
- Option (c): `wasamoc check` rejects the WrapPanel-of-aspect-only-
  Box-without-`item-cross-size` pattern at compile time. The error
  is loud and early, but `wasamoc check` must statically classify
  children by "size source" — a classifier that scales poorly as the
  widget catalogue grows.

The framing's working direction (Option (a) + ship warning) is the
WPF / Slint precedent and the principle "WrapPanel does not redefine
child measure when the author has not asked it to". The warning is
the soft Option (c) — narrow, structurally cheap, catches the known
footgun without committing the spec to size-source classification.

The companion judgement (ship the warning vs defer it):

- Ship: Author-time guidance for the common failure mode. Cost is
  one diagnostic emission; the warning text references the
  `dsl_spec.md` pitfall note.
- Defer: Authors learn the footgun at runtime (huge-thumbnail visual)
  rather than at compile time. Recoverable but slower; reduces the
  Phase 3 diagnostic surface to the minimum.

The Phase 3 gallery sub-screen sets `item-cross-size: 88` explicitly
under both pick directions, so the answer here does not affect the
visible proof — it affects what authors who *omit* the attribute
experience.

---

## Summary of decisions

The **Forward-compat exposure** column rates the recommended option
of each DD per [decisions README §Risk evaluation](./README.md#risk-evaluation).
All six rate `Low` because every recommendation is structurally
additive against the foreseeable future events catalogued in
**Out of scope** above (Phase 4 ScrollView, Phase 5 Grid, Phase 6
ZStack, Phase 7 iteration, M4+ bindable / theming / Image-widget) —
none of those events forces a revision to the Phase 3 IR shape,
default behaviour, or spec text; each extends rather than rewrites.

Per-DD exposure paragraphs in each section above remain the
authoritative narrative; the column is the at-a-glance rating.
(The decisions-README requires the column for the recommended
option of every DD; this Phase 3 ADR adheres to the format. M3
Phase 1 / Phase 2 summary tables omit the column — surfacing the
question of whether to backfill them belongs in a separate
docs-cleanup pass, not this ADR.)

| ID | Topic | Recommendation | Forward-compat exposure |
|---|---|---|---|
| DD-M3-P3-001 | WrapPanel IR node form + N-child main-axis flow contract | Option A across sub-issues — per-kind tag `WidgetKind::WrapPanel`; 0-child and 1-child both valid; document-order placement with spacing-aware `<=` inequality and **unconditional placement of the first child of any line** (normatively spec'd in DD-005); child measure input is unbounded main-axis + DD-004-defined cross-axis; cross-axis item alignment is centred within line | Low |
| DD-M3-P3-002 | Orientation attribute | Option A — hardcode horizontal main-axis; no `orientation` attribute exposed in Phase 3; vertical opens additively in a later phase | Low |
| DD-M3-P3-003 | Spacing attributes (item-spacing / line-spacing / padding) | Option A — ship `item-spacing: <i32>` and `line-spacing: <i32>` as constant-only attributes (default 0; reuse existing `i32` plumbing; no new `PropertyValue` variant); defer padding to a later phase (load-bearing — see Checkpoint 1) | Low |
| DD-M3-P3-004 | Item sizing source | Option A across sub-issues — expose `item-cross-size: <i32>` as optional constant-only attribute; default behaviour when unset is Option (a) parent-cross-axis passthrough; per-line cross-axis sizing rule when attribute set is uniform `item-cross-size`; ship `wasamoc check` warning for aspect-only-Box children without `item-cross-size` (load-bearing — see Checkpoint 2) | Low |
| DD-M3-P3-005 | Measure-arrange algorithm (novel normative spec) | Option A across sub-issues — bounded main-axis greedy line breaker with spacing-aware `<=` inequality for subsequent children and **unconditional placement for the first child of any line** (oversized first-child line extent may exceed `parent_main_bound`; WrapPanel outer main-axis still equals `parent_main_bound` and visible overflow paints past the WrapPanel rectangle); cross-axis line sizing per DD-004 (uniform when `item-cross-size` set; max-of-children when unset); WrapPanel outer cross-axis is sum of line extents + `line_spacing × (line_count − 1)`; unbounded main-axis degenerates to **one-line flow** (no new `LayoutError` variant); unbounded cross-axis with aspect-only child fires Phase 2's existing `LayoutError::BoxAspectUnboundedBoth`; rounding contract inherits Phase 2 DD-005 (no pixel-snapping in Phase 3) | Low |
| DD-M3-P3-006 | IR-loader defense-in-depth invariants | Option A across sub-issues — two-gate defense (`wasamoc check` + `validate()`) for non-negative integer ranges on `item-spacing` / `line-spacing` / `item-cross-size`; zero is valid (author-requested degenerate layout); no child-count restriction (0+); orientation rule conditional on DD-002 (collapses under DD-002 recommendation); error class `WASAMO_ERR_IR_MALFORMED` | Low |

Implementation task list: belongs in the Phase 3 progress file
`docs/plans/progress/m3-phase-3-progress.md` (created when this ADR
is Accepted and Phase 3 starts execution); not in this ADR and not
in `m3-plan.md` itself. See
[plans/README.md §Scope rule (plan vs ADR)](../plans/README.md#scope-rule-plan-vs-adr)
and [plans/README.md §Phase progress file lifecycle](../plans/README.md#phase-progress-file-lifecycle)
for the authoritative location and the `active → closing → retired
→ archived` lifecycle the file follows. The Progress table in
[m3-plan.md](../plans/m3-plan.md) carries only a one-row index entry
pointing at this progress file.

## Spec impact preview (for owner agreement)

When this ADR is Accepted, the following docs change in the
**Moment 1** commit set (framing decision D — ADR-Accepted /
design-spec draft). Each constituent lands as its own commit on the
pre-doc branch, scoped by review concern per
[CLAUDE.md §Commit rules](../../CLAUDE.md#commit-rules):

- **ADR `Status: Accepted` flip** — this file.
- [docs/dsl_spec.md](../dsl_spec.md) — new **§4.10 WrapPanel chapter**
  alongside Phase 2's §4.9 Box chapter. The chapter contains:
  - The WrapPanel sizing mental-model subsection (framing decision H)
    — four-fact anchor + ecosystem-contrast block (WPF / Compose /
    CSS), positioned before the formal measure-arrange algorithm so
    the reader builds the model before the rules. Cross-referenced
    from DD-005's Recommendation prose in this ADR.
  - The two-stage measure-arrange algorithm (DD-005 Recommendation
    prose lifted into normative spec form, with the inequality
    derived from DD-001 and the **first-child unconditional
    placement** carve-out spelled out).
  - The **oversized first-child / oversized line behaviour** —
    line breaker places oversized first-child unconditionally; line
    extent may exceed `parent_main_bound`; WrapPanel outer main-axis
    is still `parent_main_bound`; visible overflow paints past the
    WrapPanel rectangle unless an enclosing parent clips
    (per DD-005 oversized-line Option A).
  - Attribute reference: `item-cross-size` (DD-004), `item-spacing`
    / `line-spacing` (DD-003).
  - The "common pitfalls" note: aspect-only Box children without
    `item-cross-size` (DD-004 Recommendation companion) **and**
    oversized children painting past the WrapPanel boundary
    (DD-005 oversized-line Option A note).
  - Section marker
    `**Phase status:** M3-Phase 3 ADR-accepted design draft; pending
    implementation re-sync` at the chapter top.
- [docs/architecture.md](../architecture.md) §6 — WrapPanel entry
  under the M2-revised IR section if structural placement warrants;
  short paragraph noting (a) WrapPanel's two-stage measure-arrange
  is the first M3 layout primitive with cross-axis line aggregation,
  (b) the per-type binding seam is *not* extended by Phase 3 (all
  WrapPanel attributes constant-only) so the F5 deferral is
  unpressured, and (c) layout engine boundary remains Win32/WinRT-
  free — the line breaker operates on pure data
  ([predoc-inputs.md §8](../notes/m3-phase-3/predoc-inputs.md)).
- [docs/abi_spec.md](../abi_spec.md) — **no changes in Phase 3**.
  No new ABI public function, no new `WASAMO_VALUE_*` tag, no new
  arms in `abi.rs`. All WrapPanel attributes are constant-only
  `i32` and stay on the WrapPanel-internal IR node; they do not
  traverse `PropertyValue`-mediated paths. No new
  `WASAMO_LAYOUT_ERROR_*` extension — DD-005 Option A adds no
  variant, and the unbounded-cross case fires Phase 2's existing
  Box-side error.
- [docs/plans/m3-plan.md](../plans/m3-plan.md) — Progress section's
  Phase 3 row populated (Status: `in progress`; ADR link; progress
  file link).
- [docs/plans/progress/m3-phase-3-progress.md](../plans/progress/) —
  new file opened with task list mapped to this ADR's verification
  closure items below.
- [docs/notes/retrospectives.md](../notes/retrospectives.md) —
  **no Phase-3-specific amendment expected at framing time** per
  framing decision F's process-rules-ssot disposition. Phase 2's
  `cargo fmt` discipline tightening was a one-off; no analogous
  pattern is anticipated for Phase 3 at the framing stage. The
  retrospective discipline (forward distillation per
  [[feedback_retro_forward_distillation]]) carries forward without
  text changes.

The **Moment 2** commit set (framing decision D — Phase close /
implementation re-sync) lands at phase end; the WrapPanel-chapter
spec marker flips to `**Phase status:** M3-Phase 3 closed;
implementation-synced`, any divergence between the design-spec
draft and the implementation is corrected in the same commit, and
earlier-phase spec gaps surfaced during re-sync may fold per
[[feedback_retroactive_spec_gap_fold]] with explicit owner
confirmation. The Phase 3 progress file is retired in the same
commit set per the standard `active → closing → retired →
archived` lifecycle.

No ROADMAP revision is anticipated — A3 is already explicit, this
ADR operationalises it.

## Phase 3 verification closure (what counts as A3 evidence)

This section is not a DD — it records the agreed shape of the proof
that closes Phase 3 per framing decision C, so the implementation
plan inherits a concrete target rather than re-litigating "what does
WrapPanel's verification mean here?".

A3 (WrapPanel layout primitive — two-stage measure-arrange) is
considered satisfied when **all five** of the following are observed:

1. **`wasamoc check` compile-time evidence (host-independent).**
   Pure-logic tests in `wasamoc`'s check / lower path cover:
   - **Negative literal rejection** (DD-006 two-gate, compile-time
     half) — `item-spacing: -1`, `line-spacing: -1`,
     `item-cross-size: -1` each surface a `wasamoc check`
     diagnostic naming the rejected attribute. The runtime
     `validate()` half is covered by item 3 below.
   - **Aspect-only-Box warning** (DD-004 Recommendation companion)
     — a WrapPanel directly containing one or more `Box { aspect:
     <ratio>; … }` children with no `item-cross-size` set on the
     WrapPanel surfaces a `wasamoc check` *warning* (not error)
     suggesting the attribute. The warning fires on the framing's
     working pick; if Checkpoint 2's companion judgement flips to
     "defer the warning", this bullet is dropped together with
     the implementation.
   - **Sub-screen positive control** — the gallery sub-screen's
     `.ui` (item 5 below) is *not* expected to emit either the
     rejection or the warning (`item-cross-size: 88` is set
     explicitly); compiling it cleanly is the positive control.

   These run on any CI runner; the diagnostics are pure-logic in
   `wasamoc`.

2. **Line-breaker and arrange unit-test evidence (host-independent).**
   Pure-logic tests against the layout engine's WrapPanel
   measure/arrange (`wasamo-runtime/src/layout.rs` extension) cover:
   bounded main-axis happy path with multi-line wrap; bounded main-
   axis happy path with single-line fit (no wrap needed);
   **oversized-first-child unconditional placement** (a single
   child whose intrinsic main extent exceeds `parent_main_bound` is
   placed on a one-child line whose recorded extent exceeds the
   bound; subsequent children start new lines); **WrapPanel outer
   main-axis equals `parent_main_bound` even when an oversized
   first-child is present** (per DD-005 oversized-line Option A);
   **arrange-pass evidence of visible overflow** — for the
   oversized-first-child fixture, the arranged child rect's main-
   axis end (`child.x + child.w` in horizontal-main-axis terms)
   exceeds the WrapPanel rect's main-axis end. This is the pure-
   data observable form of "child paints past the WrapPanel
   rectangle"; the absence of a clip surface installed by WrapPanel
   itself is item 4's runtime-side concern; unbounded-main-axis
   branch (one-line flow per DD-005); cross-axis line sizing when
   `item-cross-size` is set (uniform per-line extent); cross-axis
   line sizing when `item-cross-size` is unset (max of children's
   reported sizes); spacing-aware overflow inequality for
   `line_empty == false` (no trailing margin); `item-spacing: 0`
   and `line-spacing: 0` degenerate layouts; `item-cross-size: 0`
   author-requested degenerate layout; unbounded-cross-axis-with-
   aspect-child propagating to Phase 2's
   `LayoutError::BoxAspectUnboundedBoth`. These run on any CI
   runner; the line breaker and arrange pass are pure functions
   (input → output) per framing decision C.

3. **IR-loader / `validate()` invariant evidence (host-independent).**
   Pure-logic tests in `wasamo-runtime`'s `ir_loader::validate()`
   path cover (DD-006 two-gate, runtime half): negative literal
   rejection for `item-spacing`, `line-spacing`, `item-cross-size`
   in memory-IR that bypasses `wasamoc` (each surfaced as
   `WASAMO_ERR_IR_MALFORMED`); 0-child WrapPanel valid; 1-child
   WrapPanel valid; multi-child WrapPanel valid (no upper bound).
   Symmetric with Phase 2 T7's `validate()` discipline.

4. **Windows-runtime layout evidence (CI-gated).** A mock-free
   integration test (per
   [CLAUDE.md §Testing rules](../../CLAUDE.md#testing-rules)) on
   the Windows CI runner exercises two fixtures:

   - **Wrap-path fixture (primary).** A `.ui` declares a WrapPanel
     with `item-cross-size: 88; item-spacing: 12; line-spacing: 12`
     and 5–10 `Box { aspect: 1:1; fill: #336699cc; Text { text: …
     } }` children inside a parent of known main-axis size. The
     test loads the IR, runs the layout pass, and asserts (a) the
     WrapPanel's resolved rectangle matches the expected outer
     dimensions (main-axis = `parent_main_bound`; cross-axis =
     `line_count × 88 + (line_count − 1) × 12`), (b) each child's
     resolved rectangle is 88×88, (c) child positions match the
     expected line / column assignment given the main-axis bound.
   - **Oversized-child fixture (visible-overflow regulation).** A
     small `.ui` declares a WrapPanel inside a narrow parent
     (e.g. `parent_main_bound = 100`) with a single
     `item-cross-size: 50` Box child whose intrinsic main-axis
     extent exceeds the bound (`Box { aspect: 4:1 }` →
     intrinsic main = `50 × 4 = 200`). The test asserts (a) the
     WrapPanel's resolved rectangle still has `main-axis = 100`
     (per DD-005 oversized-line Option A — WrapPanel does not grow),
     (b) the child's resolved rectangle has `width = 200` and
     `x + width > 100` (visible overflow at the runtime layer is
     preserved through to the Visual Layer tree), and (c) WrapPanel
     installs **no clip surface** on its Composition visual — this
     is observable through the Visual tree the runtime produces
     (the WrapPanel's `ContainerVisual` has no `Clip` property set
     by Phase 3 code). The absence-of-clip assertion is the
     runtime-side complement to item 2's arrange-pass evidence:
     together they establish that the spec'd "visible overflow,
     parent clips" convention is implemented end-to-end.

   Both fixtures fail (not skip) on a runner that cannot create the
   Compositor — the test gates A3 evidence in CI, not local
   convenience. Skip-guard inherits the Phase 2 T11 pattern verbatim
   (fires on `0x80070005` from `wasamo_init`).

5. **End-to-end host evidence (visible smoke).**
   `examples/gallery/gallery.ui` is **grown additively** from
   Phase 2's single-Box sub-screen into a WrapPanel of uniform
   1:1 Box thumbnails (5–10 items, hand-written; no iteration,
   no ScrollView). `examples/gallery-rust/` (already a workspace
   member from Phase 2) builds and runs the grown sub-screen.
   `Start-Process` launch is recorded as successful by the assistant;
   **visual correctness** of the WrapPanel rendering (lines wrap
   correctly at the expected main-axis budget; line-spacing
   produces the expected cross-axis gaps; item-spacing produces
   the expected main-axis gaps; the sub-screen visually matches
   the wide-state wireframe within reason) is **owner-manual GUI
   smoke** per framing decision G — the assistant does not assert
   on pixel- or eyeball-level correctness.

Items (1)–(4) are required for A3 acceptance; item (5) ties the
evidence back to the m3-plan target-app trajectory and grows the
gallery sub-screen Phase 2 seeded. C and Zig hosts for the
WrapPanel sub-screen are explicitly **not** required in Phase 3
(per framing decision E and the Out of scope list); Phase 8
broadens the full gallery to all three.

Per [predoc-inputs.md §10](../notes/m3-phase-3/predoc-inputs.md),
evidence items (1)–(4) do not collapse into one even though they
share helper infrastructure — the `wasamoc check` diagnostics, the
line-breaker tests, the IR-load `validate()` gate tests, and the
Windows integration test each have distinct evidence meanings
(compile-time guard enforcement; algorithm correctness; runtime-
side invariant enforcement; live-runtime composition).

The acceptance / non-acceptance of test items (1)–(5) is the
operational form of "Phase 3 done"; the corresponding
implementation checklist (which crate / which test file / which
fixture) belongs in the Phase 3 progress file, not here.
