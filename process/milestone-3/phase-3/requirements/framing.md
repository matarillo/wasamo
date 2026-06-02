# M3-Phase 3 pre-doc framing

**Status:** framing aligned with owner (2026-05-21); input artefact for ADR drafting
**Date:** 2026-05-21
**Targets phase:** M3-Phase 3 (WrapPanel layout primitive)

Per the project's doc-driven workflow established at
[M2-Phase 6 pre-doc framing](../m2-phase-6/m2-phase-6-pre-doc-framing.md)
and continued through
[M3-Phase 2 pre-doc framing](../m3-phase-2/m3-phase-2-pre-doc-framing.md),
individual DDs are not negotiated one-by-one in chat — framing is
aligned first, then the full ADR is drafted in one pass as
`Status: Proposed`, reviewed, and flipped to `Status: Accepted`.
This note records the framing intended for owner alignment before
ADR drafting begins; it remains as an input artefact and is not
promoted into the ADR.

The two preceding M3 phases supply two things this framing inherits
rather than re-derives:

- The two-moment spec-sync structure (Moment 1 design-spec draft at
  ADR-Accepted commit; Moment 2 implementation re-sync at phase
  close), with section-level `**Phase status:**` markers in the
  affected `docs/dsl_spec.md` chapter. See
  [m3-phase-2 framing decision D](../m3-phase-2/m3-phase-2-pre-doc-framing.md#d-upstream-document-revision-timing-two-sync-moments).
- The Moment-is-not-a-commit-unit rule, recorded in
  [CLAUDE.md §Commit rules](../../../CLAUDE.md#commit-rules): each
  constituent document lands as its own commit on the pre-doc
  branch, scoped by review concern, not by Moment. The Moment is
  "achieved" when every constituent commit has landed. Phase 3
  starts under this rule; no Phase-2-style postmortem is expected.

---

## Phase 3 acceptance criteria (restated)

- **A3** (see [process/_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
  [m3-plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)):

  > WrapPanel layout primitive, demonstrating that DSL can express
  > a two-stage measure-arrange — linear main-axis placement plus
  > cross-axis wrap on main-axis overflow — that goes beyond the
  > linear arrangement (HStack / VStack) established in M2.

- **A11 (operational obligation).** `.ui`, `wasamo-ir`, `wasamoc`,
  `wasamo-runtime`, `docs/dsl_spec.md`, and the
  `examples/gallery/` sub-screen all advance within Phase 3. The
  foundational-phase exception is scoped to Phase 1; Phase 2 seeded
  `examples/gallery/` + `examples/gallery-rust/` (framing decision
  F there), and Phase 3 grows that sub-screen rather than scrapping
  or duplicating it. The growth is additive: Phase 2's `Box {
  aspect: 16:9; fill: #336699cc; Text { ... } }` becomes a
  WrapPanel of Boxes.

- **First M3 phase with novel normative measure-arrange spec.**
  [m3-plan.md §Phase breakdown](../../plan.md#phase-breakdown)
  names Phase 3 as "the first M3 phase to introduce novel normative
  measure-arrange spec in `docs/dsl_spec.md`, so the spec-drafting
  discipline gets exercised early". Phase 2's DD-005 had non-trivial
  spec content (aspect projection, unbounded-axis edge cases) but
  introduced no new measure-arrange *paradigm*; Phase 3 does. The
  novel paradigm is the **two-stage measure-arrange** itself
  (intrinsic child measure → line formation → cross-axis sum).
  Acceptance for the spec text is the
  [m3-plan.md §Milestone-end criteria item 5](../../plan.md#milestone-end-criteria)
  bar — *could a reader of only `docs/dsl_spec.md` reproduce this
  surface against a hypothetical host that already provides the
  C ABI?* — applied at phase close rather than at milestone close.

- **Downstream commitments grounded in Phase 3.** Phase 4
  (ScrollView, minimal) is the immediate consumer: the gallery's
  thumbnail strip becomes a `ScrollView { WrapPanel { … } }` once
  Phase 4 lands. Phase 4 ScrollView bounds the **main** axis (=
  WrapPanel width = viewport width) and leaves the **cross**
  axis unbounded for vertical scroll. Phase 3 must therefore
  settle two things in a form Phase 4 can compose around: (i)
  WrapPanel's unbounded-axis / intrinsic-sizing behaviour on
  *both* axes (DD-005), and (ii) a thumbnail sizing path that
  does not collapse when the parent's cross-axis is unbounded
  (DD-004's `item-cross-size` — the gallery sub-screen carries
  this through Phase 4 unchanged). Phase 5 (Grid) is the second
  novel-normative-spec phase and explicitly benefits from
  WrapPanel's spec rehearsal, per the
  [m3-plan.md §Risks](../../plan.md#risks) "WrapPanel /
  Grid measure-arrange spec complexity" mitigation.

---

## Layering note (DD-001 ⇄ DD-004 ⇄ DD-005)

Like Phase 2, Phase 3's structural DD (DD-001 — IR shape + child
layout contract) and its algorithmic DD (DD-005 — measure-arrange
algorithm) are layered, but Phase 3 has a *third* DD in the chain
— DD-004 (item sizing source) — that supplies the cross-axis
bound passed to children. The dependency direction is **inverted**
relative to Phase 2:

- **Phase 2.** Box's outer bounds (DD-005) resolve *without*
  considering child intrinsic size when `aspect` is set. Aspect-
  derived bounds win; children do not grow the Box. Outer first,
  inner second.
- **Phase 3.** WrapPanel's outer bounds *do* depend on children:
  each child's main-axis intrinsic size determines which line it
  joins; each line's cross-axis extent depends on the children
  in that line (and on DD-004's `item-cross-size`, when set);
  WrapPanel's cross-axis outer size is the sum of line cross-axis
  extents (plus any cross-axis spacing). The main-axis outer size
  is bounded by the parent's main-axis constraint. **Inner first,
  outer second** on the cross axis; outer-equals-constraint on
  the main axis.

DD-004 sits *between* DD-001 and DD-005 in the chain: DD-001 says
"children measured against unbounded main-axis + cross-axis
passthrough"; DD-004 settles *which* cross-axis is passed through
(parent's, or `item-cross-size`-bounded); DD-005 then runs the
line breaker on the resulting child measures.

This inverted layering is the structural content of "two-stage
measure-arrange". The ADR's Option tables for DD-001, DD-004, and
DD-005 should cite this layering note in their Recommendation
prose so the reviewer can verify each Option respects the
dependency direction.

Concrete consequences for the ADR's Options tables: the following
combinations are **invalid** and should not appear as recommended
cells —

- DD-005 = "WrapPanel cross-axis ignores children" with any DD-001
  multi-child Option (contradicts the layering — WrapPanel cannot
  be a fixed-cross-axis container in Phase 3; that would be a Grid
  row, not a WrapPanel).
- DD-001 = "children share full WrapPanel main-axis bounds" with
  any DD-005 wrapping algorithm (contradicts the layering — if
  every child gets full main-axis bounds, wrapping is structurally
  unreachable).
- DD-004 = "no cross-axis constraint policy / source
  selection at all" (i.e. WrapPanel makes no choice between
  passthrough / `item-cross-size`-bound / unbounded) with any
  DD-005 line cross-axis sizing Option (contradicts the chain —
  DD-005's line breaker needs a defined cross-axis-bound source,
  even if the chosen source happens to be unbounded; Option (a)
  passthrough + parent-unbounded is a *defined* source whose
  outcome is unbounded, not an *undefined* source).

---

## Agreed DD slate (6 entries proposed)

The Phase 3 ADR (working title
`process/milestone-3/phase-3/decisions/preamble.md`) will carry the following
six DDs.

### DD-M3-P3-001 — WrapPanel IR node form and N-child main-axis flow contract

WrapPanel is a new layout primitive in `wasamo-ir` and
`wasamo-runtime`. Phase 3 must commit to (i) the IR node shape,
(ii) the 0-child / 1-child / N-child shapes, (iii) child measure
input, (iv) line membership rule, and (v) cross-axis alignment of
items within a line.

Sub-issues:

- **IR node shape.** Per-kind tag parallel to `HStack` / `VStack` /
  `Rectangle` / `Box`, vs a structural variant in `IrLayout`.
  Phase 2 DD-M3-P2-001 settled the per-kind-tag answer for Box;
  Phase 3 inherits unless evidence forces re-opening. Default: per-
  kind tag.
- **0-child / 1-child shape.** A WrapPanel with zero children
  produces a zero-extent line set; with one child it produces a
  one-line layout. Both should be valid (consistent with Box's
  0-child shape rule). The diagnostic question is whether
  `wasamoc check` warns about an empty WrapPanel as a probable
  author error.
- **N-child main-axis flow.** Children are placed along the main
  axis in document order. When the next child's main-axis extent
  exceeds the remaining main-axis budget on the current line, a
  new line starts. This is A3's "linear main-axis placement plus
  cross-axis wrap on main-axis overflow" verbatim. The DD settles
  the *exact* overflow condition: strict greater-than vs greater-
  or-equal, and how spacing interacts with the comparison.
- **Child measure input.** Each child is measured against an
  unbounded main-axis constraint (so the child reports its
  intrinsic main-axis size). The cross-axis constraint passed
  to the child comes from **DD-004** (either WrapPanel's
  `item-cross-size`, when set, or the parent's cross-axis
  constraint passed through unchanged, when unset). The
  alternative — each child gets a slot-sized *main-axis*
  constraint — is the Grid cell semantic and does not belong
  in WrapPanel; reject as a back-door Grid the same way Phase 2
  rejected back-door ZStack.
- **Cross-axis item alignment within a line.** When a line's
  members have heterogeneous cross-axis sizes, smaller items
  align where? Top / center / baseline / configurable per-child?
  Phase 3 sub-screen uses uniform 1:1 thumbnails (cross-axis sizes
  match by construction), so the question has no observable
  answer in the gallery proof — Phase 3 settles a default for
  the eventual heterogeneous-line case rather than reserving
  no surface.

**Layering with DD-005.** The N-child main-axis flow rule names
*what* a WrapPanel does; DD-005 names *how* the line breaker
computes it. DD-001's "children measured against unbounded main-
axis constraint" is the input to DD-005's line formation. Any
Option in DD-001 that gives children a slot-sized main-axis
constraint contradicts DD-005's intrinsic-driven line formation.

**Inputs consumed.** [predoc-inputs.md §1](./predoc-inputs.md)
(WrapPanel must consume Box intrinsic sizing rather than redefine
it; the question of "fixed slot vs max constraint vs unbounded"
maps onto the child-measure-input sub-issue above);
[predoc-inputs.md §3](./predoc-inputs.md) (multi-child overlap is
out of WrapPanel scope; Phase 3 does not add an item-stacking
mode).

### DD-M3-P3-002 — Orientation attribute

Whether WrapPanel exposes an `orientation: <horizontal|vertical>`
attribute in Phase 3, or hardcodes horizontal main-axis with
vertical reserved for a later DD.

Sub-issues:

- **Exposure.** The Phase 3 gallery sub-screen uses horizontal
  main-axis only ([m3-gallery-wireframe.html](../../references/m3-gallery-wireframe.html)
  "thumbnail grid"). Exposing `orientation` in Phase 3 admits a
  vertical-main-axis WrapPanel that has no acceptance criterion
  calling for it. Hardcoding horizontal narrows the surface; a
  later phase that needs vertical wrap opens its own DD and adds
  the attribute additively.
- **Surface form if exposed.** Enum-like
  `orientation: horizontal` / `orientation: vertical`, vs
  per-axis boolean (`vertical: true`), vs reused token form. The
  enum-keyword form mirrors the M2 `theme: system` / `backdrop:
  mica` precedent in [examples/gallery/gallery.ui](../../../examples/gallery/gallery.ui).
- **Bindable surface (if attribute exposed).** Inherits the
  Phase 1 / Phase 2 seam-building discipline: build the seam in
  the phase that needs it. A bindable orientation is a layout-
  paradigm-switching binding with no Phase 3 sub-screen calling
  for it; the speculative-seam argument from
  [m3-phase-2-box-layout.md DD-M3-P2-004](../../phase-2/decisions/preamble.md)
  applies symmetrically. Recommendation: if orientation is
  exposed at all in Phase 3, it is constant-only.

**Recommendation direction (for framing alignment, not the ADR
text):** hardcode horizontal main-axis in Phase 3 — orientation
attribute is **not** exposed. The vertical-main-axis case has no
Phase 3 sub-screen, no Phase 4 / 5 downstream dependency, and is
a clean additive extension if a later phase needs it. Exposing
`orientation` would also force DD-005 to spec the cross-axis-
wraps-main-axis swap, which is real additional spec content with
no gallery-proof justification. The bindable sub-issue then
collapses: no attribute, no bindable question.

**Inputs consumed.** [predoc-inputs.md §5](./predoc-inputs.md)
(constant-only value boundary preserved); F5 (`TypedValue`
deferral) structurally protected by not exposing the attribute
at all.

### DD-M3-P3-003 — Spacing attributes (item-spacing, line-spacing, padding)

Whether Phase 3 exposes item spacing (main-axis gap between
siblings within a line), line spacing (cross-axis gap between
lines), padding (inset between WrapPanel bounds and the line
set), or none.

**Relationship to DD-004 (item sizing).** Spacing is the *gap
between* items, not the *size of* items. The item-size source is
DD-004's question; once items have a size, spacing decides what
visible gap separates them. The wireframe's 12px gap is therefore
a DD-003 question conditional on DD-004 settling the 88×88
thumbnail extent. If DD-004 ships zero item-sizing attribute,
DD-003's spacing has no thumbnails to space.

Sub-issues:

- **Scope question.** The Phase 3 sub-screen uses uniform
  thumbnails (DD-004); the wireframe shows a 12px gap between
  them in the wide state
  ([m3-gallery-wireframe.html](../../references/m3-gallery-wireframe.html)).
  If Phase 3 ships with zero spacing, the gallery sub-screen
  visually deviates from the wireframe (touching thumbnails); if
  Phase 3 ships item-spacing and line-spacing, that's two new
  attributes with their own value-type / literal-position
  decisions.
- **Surface form if exposed.** Pixel integer
  (`item-spacing: 12`), single-axis float, or a structural pair
  (`spacing: 12 12`). Per DD-M3-P2-002 / DD-M3-P2-003
  discipline, no new `PropertyValue` variant unless the attribute
  is bindable — for integer pixel spacing, reuse `IrLiteral::Int`
  and the existing `i32` plumbing.
- **Bindable surface (if attribute exposed).** Constant-only in
  Phase 3, mirroring DD-002's stance and Phase 1 / Phase 2 seam-
  building discipline. A phase that needs animated spacing opens
  the per-type writer seam at that point — but since spacing is
  `i32`, the seam triple already exists from M2; only the call-
  site `ir_loader::build_node` registration is new in that
  future phase, not the engine.
- **Padding scope.** Padding (inset between WrapPanel bounds and
  the line set) is a separate concept from spacing (gap between
  siblings). Phase 3 may ship neither / one / both; the
  wireframe's padding question is whether the leftmost thumbnail
  starts at WrapPanel's left edge or with a margin. Visually,
  the wireframe shows a 16px margin on the left (x=36 within a
  20-padded frame), suggestive of either padding or an outer
  HStack wrapper. The framing direction is to **defer padding to
  a later phase** (Phase 4 ScrollView is one candidate; otherwise
  a follow-up DD) and have the Phase 3 sub-screen accept whatever
  left-edge behaviour the bare-WrapPanel default produces.

**Recommendation direction (for framing alignment):** ship
`item-spacing: <i32>` and `line-spacing: <i32>` as constant-only
attributes in Phase 3, with `0` as the default for both. Defer
padding. Once DD-004's thumbnail extent is settled, these
attributes carry the wireframe's inter-item gap; without DD-004
they have no thumbnails to space and the question is moot. The
surface cost is minimal — `i32` reuses existing plumbing, no new
literal form, no new `PropertyValue` variant, no ABI surface
change. An alternative is to ship Phase 3 with zero spacing and
accept touching thumbnails in the sub-screen; the framing records
both as options and asks for the owner call.

**Inputs consumed.** [predoc-inputs.md §5](./predoc-inputs.md)
(constant-only boundary by default);
[predoc-inputs.md §12](./predoc-inputs.md) (do not reach for
Box's future `width` / `height` to set thumbnail size — spacing
is a WrapPanel-level attribute about gaps, item size is DD-004's
question about cross-axis bound).

### DD-M3-P3-004 — Item sizing source (WrapPanel item cross-axis bound)

WrapPanel measures children to determine line breaks; the child
measure constraint must come from somewhere. The Phase 3
wireframe's 88×88 thumbnail (`Box { aspect: 1:1; fill: ...;
Text { ... } }`) has **no intrinsic size of its own** — `aspect`
derives the unbounded axis from the bounded axis per Phase 2
DD-M3-P2-005, so the Box needs *one* bounded axis as input. If
WrapPanel passes the parent's full cross-axis constraint through
to each child, a 1:1 Box thumbnail in an 800×600 window inherits
~600 cross-axis bound and grows to ~600×600 — not 88×88. The DD
settles where the per-item cross-axis bound comes from. This is
the load-bearing question for the gallery sub-screen's
*visible* correctness.

Sub-issues:

- **Attribute exposure.** Whether WrapPanel exposes a
  `item-cross-size: <i32>` (working name) attribute carrying the
  cross-axis bound passed to each child during measure. The name
  is orientation-neutral so a later phase that admits vertical
  orientation does not need to rename. Alternative names
  considered: `item-cross-extent`, `item-height`
  (orientation-coupled WPF-style — rejected for the same reason
  DD-002 keeps orientation deferred), `cell-size` (Grid-flavoured
  — confusable), `thumbnail-size` (use-case-specific — rejected).
- **Default behaviour when attribute is unset.** Options for the
  ADR's Option table:
  - (a) Pass parent's full cross-axis constraint through to each
    child (WPF / Slint precedent). Author-facing footgun: an
    aspect-only Box thumbnail in a tall WrapPanel becomes huge.
  - (b) Measure with unbounded cross-axis. Forces author to give
    items intrinsic size; `Box { aspect: 1:1 }` becomes a
    Phase 2 DD-005 "both axes unbounded" runtime error.
  - (c) `wasamoc check` requires `item-cross-size` when any
    child uses `aspect` and has no other size source. Compile-
    time guard.
- **Interaction with child shapes that have natural intrinsic
  size.** Text-only children (e.g. a WrapPanel of word chips for
  a future tag-cloud sub-screen) measure cross-axis from the
  font, not from a WrapPanel attribute. The DD must clarify
  whether `item-cross-size` (when set) *overrides* such intrinsic
  measure, or only acts as a default for items that lack it.
  WPF's `ItemHeight` overrides; Slint's behaviour is intrinsic-
  driven. Recommendation: when set, `item-cross-size` is the
  cross-axis bound *passed to* the child measure (the child can
  still report a smaller intrinsic; whether the line uses the
  attribute value or the child's reported size is the DD's
  per-line cross-axis sizing rule below).
- **Per-line cross-axis sizing rule when attribute is set.** If
  every child receives `item-cross-size` as its cross-axis
  bound, the line's cross-axis extent is `item-cross-size`
  uniformly, regardless of how much each child uses. This is
  the WPF-uniform-cell semantics. Alternative: max of children's
  *reported* cross-axis sizes within the line — which would
  collapse heterogeneous items to their natural sizes. The
  recommendation is uniform: `item-cross-size`, when set, is the
  line's cross-axis extent.
- **Bindable surface.** Constant-only in Phase 3, mirroring
  DD-002 / DD-003. `i32` plumbing already exists; no new
  `IrType` or `PropertyValue` variant.

**Recommendation direction (for framing alignment):** ship
`item-cross-size: <i32>` as a constant-only optional attribute.
Default behaviour when unset: **Option (a) — pass parent's full
cross-axis constraint through**, matching WPF / Slint precedent
and the principle "WrapPanel does not redefine child measure when
the author has not asked it to". The `dsl_spec.md` chapter
contains a "common pitfalls" note pointing aspect-only-Box
authors at the attribute.

Option (c) compile-time-requires is rejected as too strong for a
general WrapPanel surface: it would force `wasamoc check` to
statically classify children by "size source" (aspect-only Box
vs natural-intrinsic Text chip vs future non-Box children), a
responsibility that scales poorly as the widget catalogue grows.
The softer alternative — **a `wasamoc check` warning (not
error)** when a WrapPanel declares no `item-cross-size` and at
least one direct child uses `aspect`-only sizing — is a Phase 3
candidate worth considering in the ADR as a separate sub-issue
of DD-004. The warning is structurally cheaper than Option (c)
(it does not have to be sound across all child shapes — only the
known footgun case), preserves Option (a)'s default, and gives
authors author-time guidance without committing the spec to
"size source classification". The ADR records both "ship the
warning in Phase 3" and "defer the warning" as options; the
framing's working direction is to **ship the warning** but
defers the final pick to the ADR's Option table.

The Phase 3 gallery sub-screen sets `item-cross-size: 88`
explicitly; the WrapPanel of 1:1 Boxes then produces 88×88
thumbnails matching the wireframe (no warning triggered).

**Layering with DD-001 / DD-005.** This DD is the cross-axis
input to DD-001's child measure pass and to DD-005's line
formation:

- DD-001 says "children measured against unbounded main-axis
  constraint + cross-axis constraint passed through". This DD
  settles *which* cross-axis constraint is passed through:
  parent's full cross-axis (default), or `item-cross-size`-
  bounded (when attribute set).
- DD-005's line formation consumes per-child intrinsic main-axis
  after that measure. When `item-cross-size` is set, child
  intrinsic main-axis becomes aspect-derived from
  `item-cross-size` for aspect-locked Box children; otherwise
  it's whatever the child measures with parent's cross-axis as
  input.
- DD-005's per-line cross-axis sizing rule (when
  `item-cross-size` is set) is the uniform-attribute-value rule
  above; when unset, the line's cross-axis is the max of
  children's reported cross-axis sizes (which, for a parent-
  passthrough cross-axis, can be the full parent cross-axis —
  consistent with the "huge thumbnail" footgun).

**Inputs consumed.** [predoc-inputs.md §1](./predoc-inputs.md)
(WrapPanel consumes Box intrinsic sizing without redefining it;
the consume direction is "cross-axis bound from WrapPanel →
main-axis derived in Box per Phase 2 DD-005");
[predoc-inputs.md §12](./predoc-inputs.md) (do not reach for Box
`width` / `height` to solve thumbnail sizing — the bound lives
at WrapPanel level, not item level).

### DD-M3-P3-005 — Measure-arrange algorithm (novel normative spec)

The load-bearing DD of Phase 3. The first M3 phase to introduce
a novel measure-arrange paradigm into `docs/dsl_spec.md`. The
DD settles the line-formation algorithm and its edge cases; the
ADR section is also the *seed* of the dsl_spec chapter (Moment 1
lands the spec chapter in design-spec-draft form; Moment 2
re-syncs to implementation findings).

Sub-issues:

- **Bounded main-axis parent (happy path).** Children are
  measured against an unbounded main-axis constraint (per
  DD-001). The line breaker greedily appends children to the
  current line; the acceptance rule is two-cased (this
  two-case form was settled during ADR review on 2026-05-21
  after a single-inequality reading was found to leave the
  `line_empty == true ∧ next_child_main_intrinsic >
  parent_main_bound` case ambiguous — see the "Spacing
  interaction with overflow comparison" sub-issue below for the
  carve-out and the "Oversized first child / oversized line"
  sub-issue below for the visible-overflow / outer-bound
  settlement):
  - **First child of a line (`line_empty == true`).** Placed
    unconditionally on the current line, regardless of whether
    its intrinsic main-axis extent exceeds `parent_main_bound`.
    The line's recorded extent equals the child's intrinsic
    extent and may exceed the bound.
  - **Subsequent children (`line_empty == false`).** Placed iff
    the spacing-aware inequality (below) holds; when it fails,
    a new line starts and the candidate becomes the first child
    of that new line (where the unconditional-placement rule
    applies).

  The cross-axis extent of a line is the max of its members'
  cross-axis intrinsic sizes; WrapPanel's outer cross-axis
  extent is the sum of line cross-axis extents plus
  line-spacing × (line count − 1).
- **Unbounded main-axis parent (intrinsic-sizing context).**
  When the parent provides no main-axis bound, WrapPanel cannot
  wrap — there is no boundary to compare cumulative line extent
  against. The realistic context is an outer intrinsic-sizing
  measure pass (e.g. WrapPanel inside a future Phase 5 Grid cell
  whose width is being computed intrinsically before star
  sizing resolves, or a host-driven measure for window-sizing).
  **Phase 4 ScrollView is *not* the canonical example**:
  ScrollView's vertical-scroll use in the gallery bounds the
  *main* axis (WrapPanel main-axis = WrapPanel width = viewport
  width) and unbounds the *cross* axis (so content can scroll
  vertically). Citing ScrollView here would muddy the Phase 4
  contract.
  Options: (a) all children flow on one line (degenerate to
  HStack); (b) layout-time runtime error symmetric with Phase 2
  DD-005's unbounded-both-axes case; (c) take the child's
  intrinsic union as the main-axis bound, then wrap (incoherent
  — degenerate to (a) once one line is reached). The framing
  recommendation is **(a) — one-line flow**: WrapPanel-without-
  main-axis-bound has a defensible reading (no place to wrap,
  so don't), it composes with intrinsic-sizing measure passes
  rather than blowing up, and the one-line outcome is *visible*
  (the caller sees a long row), not silent like a zero-extent
  dropout. Option (b)'s no-silent-dropout virtue from Phase 2
  does not transfer: Phase 2's degenerate Box was structurally
  zero-extent; the Phase 3 degenerate WrapPanel is structurally
  one-line-flow.
- **Cross-axis line sizing.** Depends on DD-004's
  `item-cross-size` settlement:
  - When `item-cross-size` is **set**: each child receives
    `item-cross-size` as its cross-axis bound; the line's
    cross-axis extent is exactly `item-cross-size` (uniform
    per-line; WPF-`ItemHeight`-style semantics). A Box child
    with `aspect: num:den` derives main-axis extent =
    `item-cross-size × num / den` per Phase 2 DD-005's
    bounded-axis-wins rule.
  - When `item-cross-size` is **unset**: each child receives
    the parent's cross-axis constraint as its cross-axis bound
    (the WrapPanel-level passthrough — default per DD-004
    Option (a)). The line's cross-axis extent is the max of
    children's reported cross-axis sizes. A Box child with
    `aspect: num:den` derives main-axis extent =
    `parent_cross × num / den` per Phase 2 DD-005, which is
    the "huge thumbnail" path DD-004 warns about.

  The Phase 3 spec states this explicitly under §4.10: "a
  WrapPanel child whose intrinsic sizing depends on cross-axis
  bounds receives either the WrapPanel's `item-cross-size` (when
  set) or the parent's cross-axis constraint (when unset) as
  its cross-axis bound for measure".
- **Unbounded cross-axis parent.** A corollary of DD-004 Option
  (a) (parent passthrough) is that when the parent's cross-axis
  is *itself* unbounded — and the author has not set
  `item-cross-size` — each child receives an unbounded cross-
  axis constraint. A `Box { aspect: ratio }` child in this state
  has both axes unbounded and hits Phase 2 DD-005's
  `LayoutError::BoxAspectUnboundedBoth` runtime error, surfaced
  with the Box's IR location. The Phase 3 spec records this as
  the *expected* outcome: WrapPanel does not synthesise a bound
  out of nowhere; the author must set `item-cross-size` or wrap
  the WrapPanel in a sized parent. Phase 4 ScrollView's vertical-
  scroll use of WrapPanel illustrates the resolution path —
  ScrollView bounds the main axis (= WrapPanel width = viewport
  width) and leaves the cross axis unbounded for scroll, but the
  gallery sub-screen sets `item-cross-size: 88` explicitly so
  the unbounded cross-axis is never the child's bound.
- **Per-line cross-axis item alignment.** When a line's members
  have heterogeneous cross-axis sizes (smaller items shorter
  than the line max), how do they align — start (top in
  horizontal main), center, end, stretch? Phase 2 DD-001 chose
  "center" for Box's single child; the same default in
  WrapPanel is a defensible mirror. Phase 3 gallery sub-screen
  uses uniform 1:1 thumbnails so the default is unobservable
  in the proof.
- **Spacing interaction with overflow comparison.** When the
  comparison "next child fits on current line" is evaluated, the
  inter-item gap must be counted *only between* items, not as a
  trailing margin on the last one. The rule applies **only** to
  subsequent children (`line_empty == false`); the first child
  of a line is placed unconditionally per the bounded-main-axis
  carve-out above. For subsequent children, the spec states the
  rule as the inequality:

  ```
  next_child fits on current_line  iff
      current_line_main
      + item_spacing
      + next_child_main_intrinsic
      <= parent_main_bound
  ```

  (The earlier draft of this sub-issue carried a single
  inequality with a `line_empty ? 0 : item_spacing` term and
  no separate `line_empty == true` carve-out; ADR review on
  2026-05-21 found that form ambiguous against oversized first
  children and split it into the two cases recorded here.)
  After the last child of a line, no trailing `item_spacing` is
  added to the cumulative line extent. Total WrapPanel main-
  axis used **by content** is the max over lines of their
  cumulative extents (bounded by `parent_main_bound` only when
  no line contains an oversized first child; otherwise unbounded
  above by that line's oversized child — see the next
  sub-issue). This eliminates the "trailing margin" reading.
- **Oversized first child / oversized line (added during ADR
  review on 2026-05-21).** When the first child of a line has
  an intrinsic main extent that exceeds `parent_main_bound`,
  the bounded-main-axis carve-out above places it on that line
  anyway (line extent may exceed bound). The ADR settles two
  separate downstream rules:
  - **WrapPanel outer main-axis size.** Stays equal to
    `parent_main_bound` (does not grow to accommodate oversized
    children). Cascading parent-bound violations are excluded.
  - **Visible overflow.** The oversized child paints at its
    measured extent, so its main-axis end exceeds the
    WrapPanel's outer main-axis bound. WrapPanel does **not**
    install a clip surface; visible clipping is the responsibility
    of an enclosing parent (Phase 4 ScrollView is the canonical
    clipping parent; a plain HStack parent does not clip).
    Matches the WPF / Slint / Compose "overflow is visible
    unless someone clips" convention.

  The alternative options (layout-time runtime error on
  oversized child; silent skip; WrapPanel grows to fit; clip at
  WrapPanel boundary) were considered and rejected in the ADR
  ([`process/milestone-3/phase-3/decisions/preamble.md`](../decisions/preamble.md)
  DD-005 "Options (oversized first-child of a line)" and
  "Options (oversized line — arrangement / paint clip)").
- **Rounding contract.** Inherits Phase 2 DD-005's discipline:
  parent bounds enter as `f32`; integer comparisons (`<=`,
  `>`) on main-axis budget are computed in `f32` directly
  (spacing values are `i32`, promoted to `f32` for the
  comparison; child intrinsic sizes are `f32` from the layout
  engine). No pixel-snapping in Phase 3.
- **LayoutError surface.** Phase 2 introduced
  `LayoutError::{BoxAspectUnboundedBoth, BoxNoExtent}`. If
  Phase 3 chooses Option (b) above (runtime error on unbounded
  main-axis), a new `LayoutError::WrapPanelUnboundedMain` variant
  is added; ABI / host-visible surface remains internal per
  [predoc-inputs.md §9](./predoc-inputs.md). If the framing
  picks Option (a) (one-line flow), no new variant is needed.

**Layering with DD-001 / DD-004.** DD-005's algorithm assumes
each child has been measured against an unbounded main-axis
constraint plus a **DD-004-defined cross-axis constraint** (per
DD-001's child-measure-input sub-issue and DD-004's
`item-cross-size`-vs-passthrough settlement). The cross-axis
constraint may itself be unbounded — that is the
DD-004-Option-(a) + parent-unbounded case the "Unbounded
cross-axis parent" sub-issue above covers, and the
`LayoutError::BoxAspectUnboundedBoth` outcome is then the
expected runtime surface. Any Option in DD-005 that re-measures
children with a different constraint contradicts DD-001 /
DD-004.

**Inputs consumed.** [predoc-inputs.md §4](./predoc-inputs.md)
(spec-drafting bar; Phase 3's spec outline must include line
formation I/O, main-axis overflow behaviour, cross-axis line
sizing, spacing / padding treatment or out-of-scope statement,
and unbounded-parent behaviour);
[predoc-inputs.md §1](./predoc-inputs.md) (WrapPanel consumes
Box intrinsic sizing; do not redefine);
[predoc-inputs.md §8](./predoc-inputs.md) (layout engine stays
Win32/WinRT-free; the algorithm operates on pure data);
[predoc-inputs.md §9](./predoc-inputs.md) (LayoutError extension
permitted; ABI surface deferred unless host observes the error).

### DD-M3-P3-006 — IR-loader defense-in-depth invariants

Phase 2 T7 surfaced the principle: IR-load → runtime-materialise
invariants belong in pure-logic `validate()`, not in WinRT-bound
`build_node`, so the same invariant is enforced regardless of
which entry point materialises the IR. Phase 3 extends this with
WrapPanel's invariants.

Sub-issues:

- **Child count.** WrapPanel admits 0 or more children. Unlike
  Box, no upper bound; no diagnostic-worthy structural rejection.
  Empty WrapPanel is structurally valid (see DD-001 0-child
  shape).
- **Attribute value ranges.** `item-spacing`, `line-spacing`
  (DD-003) and `item-cross-size` (DD-004) all ship as integer
  attributes whose spec admits **non-negative values**.
  `wasamoc check` rejects negative literals at compile time, and
  `validate()` rejects them at IR-load time; the **two-gate
  defense-in-depth pattern mirrors Phase 2** (the structural
  pattern is the same as DD-M3-P2-005's `RATIO` zero rejection,
  though the literal threshold differs: Phase 2 RATIO rejects
  `<= 0` because zero is structurally meaningless, Phase 3
  integers reject `< 0` only).
- **Zero as author-requested degenerate layout (DD-004 / DD-003).**
  Zero is a **valid setting** for all three attributes, not a
  silent-zero footgun:
  - `item-spacing: 0` / `line-spacing: 0` — touching items / lines.
    Phase 3 default; visible-zero by construction.
  - `item-cross-size: 0` — each line collapses to zero cross-axis
    extent (no thumbnails rendered, line count still computed).
    The spec text records this as an *author-requested degenerate
    layout*, distinct from the "no extent to resolve" runtime
    errors of DD-005's unbounded-both-axes branch and the
    `BoxAspectUnboundedBoth` case. The distinction is that
    `item-cross-size: 0` is a written-out intentional setting in
    the `.ui` source; the unbounded-both-axes case is the absence
    of any bound source.
- **Orientation values if DD-002 admits the attribute.**
  `validate()` rejects unknown orientation values (would be
  rejected by `wasamoc check` first, but the two-gate principle
  applies). Conditional on DD-002's exposure decision — when the
  attribute is not exposed (framing recommendation), this
  sub-issue collapses and `validate()` has nothing to check.
- **Error class.** All WrapPanel invariant violations surface as
  `WASAMO_ERR_IR_MALFORMED`, consistent with Phase 2's
  `Box`-child-count rejection.

**Inputs consumed.** [predoc-inputs.md §7](./predoc-inputs.md)
(defense-in-depth gate is pure validation, not WinRT-bound
materialisation).

---

### Out of scope (to be carried in the ADR's Out-of-scope section)

- ScrollView pairing (Phase 4). The wireframe shows
  `ScrollView { WrapPanel { ... } }` for the overflow state;
  Phase 3 ships the WrapPanel only, no viewport / clip / content
  offset binding.
- Padding attribute on WrapPanel (deferred; left-edge behaviour
  in Phase 3 sub-screen accepts the bare-WrapPanel default).
- Per-child main-axis size override (e.g. "this thumbnail spans
  2 columns" — Grid territory, Phase 5).
- Per-child cross-axis alignment override attribute (DD-005's
  per-line item alignment default applies uniformly).
- Iteration grammar (Phase 7) — the wireframe shows generated
  thumbnails, but Phase 3 ships with a hand-written fixed set in
  the gallery sub-screen.
- Image widget surface — placeholder pattern from Phase 2 DD-006
  carries through.
- `TypedValue` generic value union (F5 maintained).
- Bindable surface for any WrapPanel attribute exposed by
  DD-002 / DD-003 / DD-004 (each DD's bindable sub-issue settles
  constant-only in Phase 3; a later phase opens the seam per
  attribute when it needs to).
- Vertical-main-axis WrapPanel (DD-002 settles horizontal-only;
  vertical opens additively).
- Per-item explicit dimensions on the Box child (`width` /
  `height` are out of M3-Phase 2 DSL surface; DD-004 settles the
  Phase 3 sizing source as `item-cross-size` at WrapPanel level,
  not as item-level dimensions).

---

## Owner-agreed framing decisions

### A. DD slate completeness

The 6 DDs above are proposed as the cut.
[predoc-inputs.md](./predoc-inputs.md) item-by-item disposition
appears in **Inputs absorbed** below. The mapping is densest at
DD-005 because most of the predoc-inputs items concern algorithm
shape; DD-003 (spacing) absorbs the visible-proof tension; the
remaining items map to framing decisions C–G.

### B. Pre-doc-discipline check

Per [process/README.md §Pre-doc discipline](../../../README.md),
the framing must verify that the proposed DD slate serves A3, not
merely execute the m3-plan task description literally. Check:

- A3 enumerates a layout primitive that does
  *linear main-axis placement plus cross-axis wrap on main-axis
  overflow*. The 6 DDs map directly to (i) IR shape + child layout
  contract (DD-001), (ii) orientation attribute (DD-002), (iii)
  spacing attributes (DD-003), (iv) item sizing source (DD-004 —
  *thumbnail cross-axis bound*), (v) measure-arrange algorithm
  (DD-005), (vi) IR-loader defense-in-depth (DD-006).
- The slate neither drops nor adds material relative to A3. No
  Grid-like per-cell spanning, no ScrollView clip surface, no
  iteration grammar.
- **Bindable-surface DD folded.** Phase 2's DD-M3-P2-004 was a
  standalone "bindable surface for `aspect` and `fill`" DD;
  Phase 3 folds the analogous question into each attribute DD
  (DD-002, DD-003, DD-004) as a sub-issue because the recommended
  answer in every case is *constant-only* and the choice is
  driven by the attribute's gallery use, not by a cross-attribute
  policy. The freed slate slot is used by DD-004 (item sizing),
  the load-bearing question for visible thumbnail correctness
  that Phase 2 framing did not anticipate at this granularity.
- The m3-plan §Phase breakdown line "first novel normative
  measure-arrange spec" is acknowledged at the Phase 3 acceptance
  restatement above and is the load-bearing framing for DD-005's
  spec content depth.

### C. Verification strategy

Per [m3-plan.md §Verification strategy](../../plan.md#verification-strategy),
Phase 3 chooses from the menu:

- **`wasamoc` check-side pure-logic tests** for compile-time
  diagnostics (DD-006 two-gate compile-time half + DD-004
  Recommendation companion warning). Covers negative-literal
  rejection on `item-spacing` / `line-spacing` / `item-cross-size`
  (DD-006) and the aspect-only-Box-without-`item-cross-size`
  warning under WrapPanel (DD-004, conditional on the "ship the
  warning" Checkpoint 2 companion pick). Added during ADR review
  on 2026-05-21 because DD-006's two-gate requirement and DD-004's
  warning pick are first-class spec commitments that need
  evidence on the `wasamoc` side, not just on the runtime side.
- **Pure-logic unit tests** for the line breaker and arrange pass
  (DD-005). Per [predoc-inputs.md §6](./predoc-inputs.md), this is
  the primary evidence shape for the novel algorithm. The line
  breaker is a pure function (line input → line output) and
  exercises the bounded / unbounded main-axis branches,
  spacing-before-comparison rule, oversized-first-child
  unconditional placement, cross-axis sum, and per-line cross-axis
  alignment default without touching Compositor. The arrange pass
  is similarly pure and is the place where the oversized-child
  rect-overflow observation lands (resolved child rect main-axis
  end exceeds the WrapPanel's resolved rect main-axis end — the
  pure-data form of "child paints past the WrapPanel rectangle").
- **Pure-logic unit tests** for IR-loader invariants (DD-006
  runtime half), symmetric with Phase 2 T7's `validate()`
  discipline.
- **Mock-free Windows-only integration test** (CI-gated, fails
  rather than skips per
  [CLAUDE.md §Testing rules](../../../CLAUDE.md)) for live
  WrapPanel materialisation through `.ui → IR → runtime` on a
  real `WidgetNode`. The integration test verifies that the
  layout engine's line breaker / arrange output is consumed by
  the runtime to produce correctly positioned child visuals.
  Scope is two narrow fixtures: the gallery sub-screen wrap-path
  with a fixed main-axis bound, *and* an oversized-first-child
  fixture that asserts (a) WrapPanel outer rect does not grow,
  (b) child rect extends past it, (c) WrapPanel's
  `ContainerVisual` has no clip surface installed by Phase 3 code
  — the runtime-side complement to the pure-arrange overflow
  observation.
- **Visible smoke** via the WrapPanel-of-Boxes sub-screen in
  `examples/gallery/` + `examples/gallery-rust/` (framing
  decision E) for owner-manual GUI smoke (framing decision G).

Per [predoc-inputs.md §10](./predoc-inputs.md), evidence items
do not collapse just because they share helper infrastructure —
the `wasamoc` check-side tests, in-crate line-breaker / arrange
tests, IR-load `validate()` gate tests, and Windows integration
tests each have distinct evidence meanings (compile-time guard
enforcement; algorithm correctness; runtime-side invariant
enforcement; live-runtime composition including the
visible-overflow regulation).

### D. Upstream-document revision timing (two sync moments)

Phase 3 inherits the two-moment structure from
[m3-phase-2 framing decision D](../m3-phase-2/m3-phase-2-pre-doc-framing.md#d-upstream-document-revision-timing-two-sync-moments)
**without modification**, except that the Postmortem rule
(Moment-is-not-a-commit-unit; per-review-concern commits) applies
from the start rather than being learned mid-phase. The Phase 3
`dsl_spec.md` section marker mirrors the Phase 2 form:

```
**Phase status:** M3-Phase 3 ADR-accepted design draft; pending
implementation re-sync
```

flipping at phase close to:

```
**Phase status:** M3-Phase 3 closed; implementation-synced
```

placed as the first line under the WrapPanel chapter heading
(new §4.10 alongside Phase 2's §4.9 Box chapter). The chapter
appears as the **design-spec draft** in Moment 1 (ADR-Accepted
commit) and is re-synced in Moment 2 (phase close); Phase 2's
two-stage cadence is the precedent.

**Moment 1 — ADR Accepted commit (design-spec draft).**
Constituent commits (each its own commit on the pre-doc branch,
per CLAUDE.md §Commit rules):

- `process/milestone-3/phase-3/decisions/preamble.md` — ADR `Status: Accepted` flip.
- `docs/dsl_spec.md` — new §4.10 WrapPanel chapter as design-spec
  draft; IR notation in §8.x extended if the spacing literal form
  requires it (it doesn't, per DD-003 recommendation — `i32`
  reuses existing plumbing).
- `docs/architecture.md` — WrapPanel entry under the M2-revised
  IR section if structural placement warrants; layout engine
  section updated for the new pure-data layout types per
  [predoc-inputs.md §8](./predoc-inputs.md).
- `docs/plans/m3-plan.md` — Progress section's Phase 3 row
  populated (Status: in progress; Progress file link; ADR link).
- `docs/plans/progress/m3-phase-3-progress.md` — new file opened
  with task list mapped to ADR's verification closure items.
- `docs/notes/retrospectives.md` — no Phase-3-specific amendment
  expected at framing time (Phase 2 framing decision E was a
  one-off `cargo fmt` discipline tightening).

Implementation begins only after these commits land; the
constituent shape preserves review-concern separability under
[CLAUDE.md §Commit rules](../../../CLAUDE.md#commit-rules).

**Moment 2 — Phase close commit set (impl re-sync).**

- `docs/dsl_spec.md` §4.10 — corrections required because the
  design draft and implementation diverged (per
  [predoc-inputs.md §6 / retroactive spec-gap fold](./predoc-inputs.md)
  inherited from Phase 2 — earlier-phase spec gaps surfaced
  during the re-sync may fold into the same commit with explicit
  owner confirmation). Section marker flips to "closed;
  implementation-synced".
- `docs/plans/progress/m3-phase-3-progress.md` — phase-close
  retrospective link, CI evidence pointer, impl summary.
- `docs/plans/m3-plan.md` Progress row — Status flips to
  complete.

### E. Phase 3 visible proof — grow Phase 2's gallery sub-screen

The Phase 3 visible proof grows
[examples/gallery/gallery.ui](../../../examples/gallery/gallery.ui)
from Phase 2's single `Box { aspect: 16:9; fill: ...; Text { ... } }`
into a **WrapPanel of Box thumbnails**. The growth path is
additive, not a scrap-and-rebuild, per
[predoc-inputs.md §11](./predoc-inputs.md).

The thumbnail set composition question
([predoc-inputs.md §2](./predoc-inputs.md)) is settled as follows:

- **Uniform 1:1 placeholders.** A fixed set of square thumbnails
  (5–10 items, label-only Box + Text children) populates the
  WrapPanel. This matches the wireframe's main thumbnail grid and
  exercises the line-breaker on the simplest possible heterogeneous-
  count basis without bringing in mixed-aspect surprises that would
  belong to a later DD.
- **No mixed aspect in Phase 3.** Mixed-aspect items would be the
  better evidence that WrapPanel handles variable-extent children,
  but they conflate two questions (line breaking + per-line
  alignment of heterogeneous items). Defer to a follow-up if the
  evidence is needed; Phase 3 sub-screen does not require it for
  A3 discharge.
- **Rust host only.** Per
  [m3-plan.md §Phase-end criteria item 5](../../plan.md#phase-end-criteria),
  Phase 3 ships at least one host's gallery proof;
  `examples/gallery-rust/` is the canonical one (already seeded by
  Phase 2 framing decision F). C and Zig host parity comes at
  Phase 8 with the full gallery.
- **No ScrollView, no iteration.** The fixed set is hand-written
  in `.ui`; iteration grammar is Phase 7's job. The set is sized
  to wrap within the default window (800×600 per the M2-set
  `DEFAULT_WINDOW_WIDTH` / `DEFAULT_WINDOW_HEIGHT` —
  [wasamo-runtime/src/abi.rs](../../../wasamo-runtime/src/abi.rs)).

**`examples/gallery/` is still a partial gallery, not the A1
proof.** A1 acceptance lives in Phase 8 per the
[acceptance ↔ phase mapping](../../plan.md#acceptance--phase-mapping);
Phase 3 grows the slice from Phase 2's Box rectangle into a
wrapped grid of Box rectangles.

### F. Live-note re-evaluation triggers — handling

[predoc-inputs.md §13–§16](./predoc-inputs.md) flag specific
`docs/notes/*` audit items. The framing settles their disposition
upfront so the ADR Inputs section can cite settled handling
rather than re-deciding:

- **[architectural-family.md](../architectural-family.md) — trigger #1 already fired in Phase 1 / Phase 2.**
  The trigger ("M3 DSL spec drafting begins") was first met when
  Phase 1's `bool` scalar ADR and Phase 2's Box chapter began
  writing `docs/dsl_spec.md`; Phase 3 is not the first firing.
  Phase 3's framing re-confirms the hypothesis specifically
  before the **first novel layout-primitive spec content** lands.
  WrapPanel does **not** require view-function re-execution,
  template expansion, or diff / reuse semantics; it is a static
  child-list layout primitive. Disposition: "re-read and confirm
  hypothesis still holds"; no vision decision record. Recorded
  in DD-001's Inputs list as a consumed input.
- **[layout-engine.md](../layout-engine.md) — partial fire.**
  WrapPanel is the first M3 phase whose layout work is non-trivial
  per the layout-engine open question 3.x set. Specific dispositions:
  - 3.1 DPI scaling — defer. Phase 3 stays in logical-pixel `f32`
    coordinate space; physical-pixel / hinting work remains the
    open question's purview.
  - 3.2 AccessKit sync — not applicable (M4).
  - 3.3 async measure — not applicable (Image deferred to M4).
  - 3.4 cache invalidation — Phase 3's WrapPanel attributes are
    constant-only (DD-002 / DD-003 / DD-004) and the gallery
    sub-screen's children are static (no iteration grammar yet —
    Phase 7), so **child count change, item-spacing change, and
    `item-cross-size` change are not runtime triggers** in
    Phase 3. The only runtime layout trigger is parent main-axis
    bound change (window resize via `WM_SIZE`), which already
    rides the existing whole-window dirty path. If a later phase
    makes any of these attributes bindable or admits iteration-
    driven child mutation, the same whole-window dirty path is
    expected to remain adequate until the 1,000-node performance
    threshold is met. Subtree-dirty is not in Phase 3 scope.
  - 3.5 user-defined layout — not applicable. WrapPanel is a
    built-in primitive.
- **[dsl-grammar.md](../dsl-grammar.md) — mostly unfired.** Phase 3
  ships a hand-written thumbnail set; no template-local scope, no
  iteration, no qualified state reference. Q1 / Q3 remain deferred
  to Phase 7. Q4 (component extension) remains M4+. Q5 (template
  interpolation) remains M3-Phase-6+.
- **[component-extension-model.md](../component-extension-model.md) — unfired.**
  WrapPanel is a built-in component; custom layout / import /
  registry is M3-out-of-scope.
- **[typed-value-evaluator.md](../typed-value-evaluator.md) — conditional
  fire deferred.** DD-002 / DD-003 / DD-004 each settle constant-
  only for the WrapPanel attributes they admit, so the
  `TypedValue` deferral is not pressured in Phase 3.
- **[workspace-layout.md](../workspace-layout.md) — unfired.** Phase 3
  adds no new crate.
- **[verification-environments.md](../verification-environments.md) /
  [headless-verification.md](../headless-verification.md).**
  Phase 3 inherits Phase 2's skip-guard pattern verbatim: GitHub
  Actions integration test fails rather than silently skips;
  local-developer guard skips when `wasamo_init` returns
  `0x80070005`. Framing decision C already commits Phase 3 to
  this shape.
- **[process-rules-ssot.md](../process-rules-ssot.md) Q6 — relevant.**
  The 3-role boundary (execution log / step retrospective / phase
  acceptance evidence) decided informally in Phase 2 carries
  forward; Phase 3 does not introduce a new evidence document
  type. Verification closure items are distilled into the ADR's
  acceptance / verification section; step-end retrospectives
  cover learnings and Follow-Ups; phase progress file is the
  short-term execution log.

### G. GUI smoke responsibility separation

Inherits [m3-phase-2 framing decision G](../m3-phase-2/m3-phase-2-pre-doc-framing.md#g-gui-smoke-responsibility-separation-predoc-inputs-5).
Visual correctness of WrapPanel rendering (lines wrap correctly at
the expected main-axis budget; line-spacing produces the expected
cross-axis gaps; item-spacing produces the expected main-axis gaps;
the sub-screen visually matches the wide-state wireframe within
reason) is **owner-manual GUI smoke**. The assistant records
`Start-Process` launch command success and any captured headless
integration output but does not assert on visual rendering. The
ADR's verification strategy section distinguishes headless test
gates from owner GUI smoke gates per Phase 2 precedent.

### H. WrapPanel sizing mental model — short anchor in dsl_spec §4.10

The facts a reader needs to internalise to use WrapPanel correctly are
distributed across DD-001 (child measure input), DD-004 (cross-axis
bound source), and DD-005 (line-formation algorithm including its
unbounded-axis branches). The framing direction is to consolidate them
into a **single short subsection** in the user-facing spec so the
reader is not forced to reconstruct the model by reading three DDs in
sequence:

1. **Main-axis intrinsic measure.** WrapPanel measures children against
   an unbounded main-axis constraint; line membership is decided by the
   child's reported main-axis intrinsic extent.
2. **Cross-axis bound source.** Each child receives a cross-axis bound
   from one of two sources — `item-cross-size` when set on the
   WrapPanel, or the parent's cross-axis constraint passed through
   unchanged when `item-cross-size` is unset.
3. **Aspect-only Box requires a cross-axis bound.** A `Box { aspect: r }`
   child has no intrinsic size of its own; without a finite cross-axis
   bound (either from `item-cross-size` or from a bounded parent),
   Phase 2 DD-005's `LayoutError::BoxAspectUnboundedBoth` fires.
   WrapPanel does **not** synthesise a bound out of nowhere.
4. **No wrap boundary ⇒ one-line flow.** When the parent supplies no
   main-axis bound, there is no boundary against which to break lines;
   all children flow on a single line. This is DD-005's unbounded-
   main-axis branch (recommended Option (a)).

**Placement.** The subsection lives in `docs/dsl_spec.md` §4.10 (the
new WrapPanel chapter), positioned before the formal measure-arrange
algorithm so the reader builds the model before the rules. The ADR's
DD-005 Recommendation prose cross-references this subsection rather
than restating the four facts. The ADR is the design-decision
archaeology; dsl_spec §4.10 is the user-facing home for the mental
model, because the users who need the model read the spec, not the
ADR.

**Ecosystem contrast (one bullet each).** `item-cross-size` is a
**Wasamo-specific abstraction**, and readers will arrive carrying
analogues that do not transfer cleanly. The subsection includes a
short contrast block so the reader's prior intuition is corrected
before they apply it:

- **WPF `ItemHeight` / `ItemWidth`** — orientation-coupled fixed cell
  extent. `item-cross-size` is orientation-neutral and conceptually a
  **bound passed to child measure**, not a cell-extent override. Per
  DD-004's per-line cross-axis sizing rule, the *visible* outcome
  matches WPF in the uniform case (line cross-axis extent equals
  `item-cross-size`), but the primitive is "child-measure bound", not
  "cell to lay child into".
- **Flutter / Jetpack Compose natural child size.** Those frameworks
  expect the child to report a natural size and the parent to size
  around it. WrapPanel's default behaviour (when `item-cross-size` is
  unset) is *closer* to this — parent constraints pass through and
  children measure naturally — but children with no natural cross-axis
  size (the aspect-only Box) are supported in Wasamo by setting
  `item-cross-size`, not by a "compute natural size" fallback.
- **CSS `gap`.** Applies to a flex/grid container's *spacing* between
  items, not to item sizing. `item-cross-size` is **not** a `gap`
  analogue. `item-spacing` / `line-spacing` (DD-003) are the CSS-`gap`
  analogues. The nearest CSS analogue to `item-cross-size` is
  `flex-basis` on children — but Wasamo lifts the decision to container
  level rather than repeating it per item.

**This is a docs framing decision, not a design change.** The design
recommended by DD-001 / DD-004 / DD-005 stands; the subsection exists
only to provide a single short anchor for the model the recommended
design implies. The Moment 1 `dsl_spec.md` draft must include the
subsection (it is part of the WrapPanel chapter, not an optional
add-on); the Moment 2 re-sync may refine wording based on what
implementation reveals (in particular, whether the warning shipped per
DD-004's framing direction changes how the aspect-only-Box bullet is
phrased).

**Inputs consumed.** Owner direction during framing alignment
(2026-05-21): four-fact mental model identified as the minimal handle
for WrapPanel correctness; ecosystem-contrast block identified as
needed because `item-cross-size` has no clean WPF / Compose / CSS
analogue and readers will mis-map it without an explicit anchor.

---

## Inputs absorbed

### From [predoc-inputs.md](./predoc-inputs.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §1 WrapPanel consumes Box intrinsic sizing | Premise / sub-issue input | DD-001 (child measure input); DD-004 (cross-axis bound is the consume direction); DD-005 (cross-axis line sizing references Phase 2's aspect-in-intrinsic-context rule) |
| §2 Placeholder thumbnail = normative gallery asset shape | Discipline reminder | Framing decision E (uniform 1:1 placeholders, no mixed aspect in Phase 3); DD-004 (thumbnail extent comes from `item-cross-size`, not from the placeholder pattern itself) |
| §3 Multi-child overlap remains out of Box scope | Premise (no Phase 3 action) | Out-of-scope section (no Phase 3 change to Box's single-child contract; ZStack remains Phase 6) |
| §4 Spec-drafting bar | Constraint | Phase 3 acceptance criteria restatement (novel normative spec); DD-005 (the spec outline items map to DD-005 sub-issues) |
| §5 Constant-only value boundary | Premise | DD-002 / DD-003 / DD-004 each carry a bindable sub-issue settling constant-only in Phase 3; no new `PropertyValue` variant; no per-type writer seam built speculatively |
| §6 Verification shape inheritance | Premise | Framing decision C (verification strategy mirrors Phase 2's menu picks; skip guard from T11 inherited) |
| §7 IR-loader defense-in-depth → pure validation | Premise / sub-issue input | DD-006 (validate() vs build_node placement) |
| §8 layout engine stays Win32/WinRT-free | Premise | DD-005 sub-issue (layout-local pure structs; runtime → layout boundary); ADR architecture.md edit at Moment 1 |
| §9 Layout-time error surface (internal `LayoutError` vs ABI) | Sub-issue input | DD-005 sub-issue (LayoutError extension conditional on the unbounded-main-axis Option chosen; ABI surface deferred unless host observes) |
| §10 Verification items do not collapse via infrastructure sharing | Discipline reminder | Framing decision C (originally four evidence categories; ADR review on 2026-05-21 added `wasamoc` check-side compile-time evidence as a distinct category, so the closure now has four executable categories — `wasamoc` check / line-breaker + arrange / runtime `validate()` / Windows integration — plus owner-manual gallery smoke) |
| §11 Gallery as additive growth | Premise | Framing decision E (grow Phase 2 sub-screen, no scrap-and-rebuild) |
| §12 Box future-width/height rule not mixed with WrapPanel item sizing | Discipline reminder | DD-004 (thumbnail size source lives at WrapPanel level as `item-cross-size`, not as item-level `width` / `height` on Box); DD-003 (spacing is a separate WrapPanel-level concept about gaps between items) |
| §13 docs/notes audit | Direct input | Framing decision F (per-note disposition) |
| §14 architectural-family — read, do not over-decide | Direct input | Framing decision F (architectural-family bullet) |
| §15 layout-engine open questions selection | Direct input | Framing decision F (layout-engine bullet) |
| §16 verification/process notes — evidence placement | Direct input | Framing decision F (process-rules-ssot Q6 bullet); framing decision C |

### From [m3-plan.md](../../plan.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §Acceptance criteria — A3 | Constraint | Framing decision B (pre-doc-discipline check) |
| §Acceptance criteria — A11 | Constraint | Phase 3 acceptance restatement (operational obligation); framing decision D (two-moment sync) |
| §Acceptance criteria — A12 | Constraint | DD-005 spec content depth (Phase 8 promotes the per-phase chapters; Phase 3's chapter is held to the same external-reader bar) |
| §Phase breakdown — Phase 3 description | Constraint | Phase 3 acceptance restatement (first novel normative measure-arrange spec) |
| §Phase dependencies — Phase 3 → 4 chain | Constraint | DD-005 (WrapPanel unbounded-axis / intrinsic-sizing behaviour settled before Phase 4; Phase 4 ScrollView is expected to bound the main axis and unbound the cross axis, so the explicit `item-cross-size` path from DD-004 is what the gallery growth relies on); framing decision E (sub-screen growth path) |
| §Verification strategy | Menu | Framing decision C |
| §Phase-end criteria item 5 (gallery sub-screen per phase) | Hard constraint | Framing decision E (Phase 3 grows Phase 2's seed; Rust host only) |
| §Risks — WrapPanel / Grid spec complexity | Adjacent risk | DD-005 (Phase 3 is the rehearsal that lowers Phase 5 Grid spec risk) |
| §Risks — Spec-drafting drift | Mitigation | Framing decision D (Moment 1 lands design-spec draft; phase does not close with TODO spec text) |

### From [m3-gallery-wireframe.html](../../references/m3-gallery-wireframe.html)

| Element | Disposition | Consumed at |
|---|---|---|
| Numbered callout (1) "WrapPanel — top-level container for the thumbnail grid; children flow along the main axis and wrap on cross axis when viewport main-axis is exceeded" | Premise (matches A3) | DD-001 (N-child main-axis flow); DD-005 (bounded-main-axis branch) |
| Numbered callout (2) "Box with `aspect: 1/1` — square thumbnail cell" | Premise | Framing decision E (uniform 1:1 placeholders) |
| Wide-state main wireframe (5 cols × 4 rows with 12px gaps) | Visible-proof reference | Framing decision E (gallery sub-screen target); DD-003 (item-spacing / line-spacing) |
| Narrow re-wrap proof strip (3-col wrap on width change) | Visible-proof reference | Framing decision C (Windows integration test fixture exercises a fixed main-axis bound, not viewport-resize) |
| Overflow proof strip (vertical scroll) | Out of Phase 3 scope | Out-of-scope (Phase 4 ScrollView) |

### From [process/milestone-3/phase-2/decisions/preamble.md](../../phase-2/decisions/preamble.md)

| DD | Disposition | Consumed at |
|---|---|---|
| DD-M3-P2-001 (Box IR + 0/1-child) | Premise — Phase 3 consumes single-child Box as thumbnail item | DD-001 (WrapPanel item is a Box; Phase 3 does not revise Box's child-count contract); framing decision E |
| DD-M3-P2-002 / DD-M3-P2-003 (Ratio / Color literals) | Premise — no new value type in Phase 3 unless DD-003 / DD-004 requires one | DD-003 / DD-004 (integer pixel spacing and `item-cross-size` both reuse existing `i32` plumbing; no new literal form, no new `PropertyValue` variant) |
| DD-M3-P2-004 (constant-only bindable surface) | Pattern reuse | DD-002 / DD-003 / DD-004 — Phase 3 folds the bindable question into each attribute DD's sub-issue rather than carrying a standalone DD; the constant-only stance is mirrored per attribute |
| DD-M3-P2-005 (aspect measure-arrange + LayoutError) | Pattern reuse | DD-005 (rounding contract reused; LayoutError extension pattern reused conditionally; aspect-in-intrinsic-sizing-context rule cited by DD-005's cross-axis line sizing sub-issue when DD-004 sets `item-cross-size`) |
| DD-M3-P2-006 (Box + Text placeholder canonicalization) | Premise | Framing decision E (sub-screen uses the canonical pattern); DD-004 (the pattern's *shape* is normative; the *size* comes from WrapPanel `item-cross-size`) |

### From [m2-to-m3-handover.md](../m2-to-m3-handover.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §1 `wasamo-ir` shared IR crate | Premise | DD-001 (new IR form = grammar + variant + loader wiring triple) |
| §2 `HandlerExpr` unified | Premise | DD-002 / DD-003 / DD-004 (each attribute's bindable sub-issue settles constant-only in Phase 3, so the unified enum is not extended; if a later phase admits binding, the literal lands as a type-suffixed variant per the pattern) |
| §3 reactive drain residuals | Out of scope | Out-of-scope (WrapPanel does not touch the drain) |
| §4 `TypedValue` deferral | Discipline reminder | DD-002 / DD-003 / DD-004 (each attribute's constant-only stance plus the per-type seam pattern from Phase 1 / Phase 2 preserves F5; no Phase 3 attribute pressures `TypedValue`) |

---

## Next session — handoff

Once framing is owner-aligned, the next session begins ADR drafting:

1. Create `process/milestone-3/phase-3/decisions/preamble.md` (working title)
   as `Status: Proposed`, carrying the 6 DDs above with full Option
   tables, Recommendation prose, and the two-axis risk / exposure
   evaluation per DD (per
   [process/README.md §Risk evaluation](../../../README.md)).
2. Owner review pass.
3. On `Status: Accepted` flip, the upstream document edits
   enumerated under **framing decision D Moment 1** land as
   **per-review-concern commits** on the pre-doc branch (not a
   single bundle), per
   [CLAUDE.md §Commit rules](../../../CLAUDE.md#commit-rules).
4. Phase progress file
   `docs/plans/progress/m3-phase-3-progress.md` opens with
   `Status: active`; the m3-plan.md Progress row flips from
   `not started` to `in progress`.
5. Implementation phase proceeds. At phase close,
   **framing decision D Moment 2** lands per-review-concern:
   `docs/dsl_spec.md` §4.10 re-sync, progress file retired per
   the standard lifecycle, phase-end retrospective recorded per
   the *retro forward distillation* discipline (Main Learning
   forwarded to a Phase 4 pre-doc input note within the same
   phase close — see
   [docs/notes/retrospectives.md §Retrospective Main Learning の前送り](../retrospectives.md)
   for the in-repo policy text).
