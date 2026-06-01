# DD-M3-P6-002 — ZStack measure / arrange + z-order + clip contract

**Status:** Proposed
**Phase:** M3-Phase 6
**AC:** A4 (ZStack layout primitive — sibling z-order by document order)

## Context

DD-M3-P6-001 fixes ZStack as a per-kind tag taking direct children in
document order. This DD fixes the **layout and paint contract**: how
ZStack measures its own size, how it arranges each child within the
overlap region, the z-order (paint order), and the clip.

ZStack's purpose is overlap: every child occupies the **same** overlap
region, painted back-to-front. This is the SwiftUI `ZStack` model and
the shape the lightbox needs (a scrim covering the area, a centered
photo above it, caption / nav above that).

Inherited precedent from Phase 5 Grid (DD-M3-P5-005) that ZStack must
stay consistent with:

- **document order = paint order** (later child on top), no explicit
  `z-index`;
- **outer-bounds clip** lands on the container's own Visual via
  `Visual.Clip = InsetClip{0,0,0,0}`; **per-child clip is out of
  scope**;
- **no intermediate Visual** (1 WidgetNode = 1 Visual) — the
  container clips on its own Visual, unlike Phase 4 ScrollView which
  locally added an intermediate content Visual;
- the layout engine is a pure-data `measure` / `arrange` boundary, and
  the `Alignment` enum (`Leading`/`start`, `Center`, `Trailing`/`end`,
  `Stretch`) is reused per-axis as `h-align` / `v-align`.

## Decision dependency summary

Most of this DD's sub-issues (Sizing / z-order / Clip) are local. One —
**Per-child alignment** — is a **light cross-DD bundle this DD owns**
(full phase map: preamble §Cross-DD decision dependencies):

- **Primary (here):** the alignment default (AL1 center) and its
  **carrier** — a parent-owned placement vector parallel to `children`,
  consumed by the ZStack/Grid parent context.
- **Surface consequence — DD-M3-P6-001:** `h-align` / `v-align` are
  authored as ordinary child `IrProp`s (no new IR vocabulary).
- **Diagnostics consequence:** the props are admitted only on a ZStack
  direct child (and a Grid `Cell`), consumed by the parent before the
  child's own unknown-prop check, and rejected elsewhere.

This bundle is lighter than the conditional-body one: there is no
either/or fork, only the single carrier shape the parent-context
consume rule fixes (Per-child arrange / alignment Recommendation).

## Sub-issues

- **Sizing policy**: how ZStack computes its own desired size, and what
  its default size constraint is.
- **Per-child arrange / alignment default**: how each child is
  positioned within the overlap region, and the default anchor.
- **z-order**: how paint order among overlapping children is decided.
- **Clip**: whether and where ZStack clips.

## Sizing policy

### Options

- **S1 — Union (per-axis max) of children, ZStack default constraint
  `Fill/Fill`**
  - ZStack's *desired* size on an axis is the **max** of its children's
    measured desired sizes (the union) — but ZStack itself defaults to
    `Fill/Fill` (like Grid / ScrollView), so on a **bounded** parent
    axis it takes the full parent allocation and the union only governs
    the **Shrink / unbounded-axis** desired-size report.
  - A `Fill` child contributes **`0.0`** to the union (the engine's
    `Fill` measure rule — [layout.rs:440](../../../../wasamo-runtime/src/layout.rs):
    "Fill children return 0.0 — they take whatever the parent allocates
    in arrange()") and fills its allocated rect in *arrange*, **not** by
    inflating the union. So a ZStack of only intrinsic children sizes to
    the largest child **on a Shrink/unbounded axis**; on a bounded axis
    the `Fill/Fill` default fills the parent. SwiftUI's ZStack is the
    union reference, but its full-screen-overlay-via-flexible-child
    behaviour does **not** transfer (Wasamo `Fill` measures `0.0`, not
    the offered size), so the lightbox's full-viewport scrim comes from
    the **ZStack's own `Fill` default**, not from the child.
  - What you gain: the standard desired-size policy with **no new
    vocabulary** and **no new `LayoutError`**; unbounded-axis behaviour
    stays identical to existing conventions (the union is the finite
    anchor, the Grid model); the overlay-first `Fill/Fill` default is
    exactly what the lightbox driver wants.
  - What you give up: an **owner-visible trade-off** — with no
    author-facing `width:`/`height:` surface in Phase 6, an intrinsic
    (bounded "size to the largest child") ZStack is **not
    author-expressible** until a future size-constraint surface; the
    SwiftUI-style intrinsic ZStack (badges sized to the icon they
    overlay) is weaker in the default experience until then.

- **S2 — Always Fill the parent allocation**
  - ZStack ignores child sizes and always consumes the full parent
    rectangle.
  - What you gain: the simplest possible rule.
  - What you give up: discards the union desired-size the
    Shrink/unbounded axis needs — there is no intrinsic sizing at all,
    even where the surrounding layout has no bound to give.

- **S3 — Size to the first (or a designated) child**
  - ZStack tracks a single "primary" child's size.
  - What you gain: an intrinsic-flavoured size without a per-axis max.
  - What you give up: needs a "primary child" concept the surface
    (DD-001) does not have, and mis-sizes whenever the largest child is
    not the designated one.

### Comparison

S2 (always Fill, ignoring children entirely) and S3 (size to a
"primary" child) are rejected as *sizing models*: S2 discards the union
desired-size the Shrink/unbounded axis needs, and S3 needs a "primary
child" concept the surface (DD-001) does not have. S1 (union) is the
standard desired-size policy and needs no new vocabulary. The
load-bearing sub-decision S1 carries is **ZStack's own default size
constraint**, because Wasamo's `Fill` measures **`0.0`**
([layout.rs:440](../../../../wasamo-runtime/src/layout.rs)), not the
offered size the way SwiftUI's flexible children do: a `Fill` scrim
child therefore does **not** drive the ZStack's measured size up, so
the lightbox's full-viewport scrim cannot arrive "through the union".
It arrives because the **ZStack itself defaults to `Fill/Fill`**
(matching Grid / ScrollView), taking the full parent allocation, with
the scrim then filling that content rect in *arrange*.

This is an **owner-visible trade-off, not a free choice.** Phase 6
ships **no author-facing `width:`/`height:` size-constraint surface**
(only `viewport-*` exists, and it is rejected at `wasamoc check` —
[check.rs:2584](../../../../wasamoc/src/check.rs)), so an author
**cannot** opt a ZStack back to intrinsic ("size to the largest
child") sizing this phase. Choosing `Fill/Fill` therefore decides that
**ZStack is an overlay-first container whose default is to fill its
parent allocation**; the SwiftUI-style intrinsic ZStack (a few badges
sized to the icon they overlay) is **weaker in the default experience
until a per-widget size-constraint surface arrives** (a future
additive phase). The owner accepts this because Phase 6's ZStack
driver is the lightbox overlay (FD-B), for which fill-the-parent is
exactly the wanted default. S1 also keeps ZStack's unbounded-axis
behaviour identical to existing conventions: the union is the finite
anchor on an unbounded axis (the Grid model), so no ZStack-specific
`LayoutError` arises.

### Recommendation

**S1 + default constraint `Fill/Fill`.** ZStack defaults to `Fill/Fill`
(like Grid / ScrollView). On a **bounded** parent axis it takes the
full parent allocation; on a **Shrink / unbounded** axis its desired
size is the per-axis **max** of its children's measured desired sizes
(the union). A `Fill` child contributes **`0.0`** to that union and
fills its allocated rect in *arrange* — it does **not** inflate the
ZStack's measured size
([layout.rs:440/673](../../../../wasamo-runtime/src/layout.rs)). The
lightbox's full-viewport scrim therefore comes from the **ZStack's own
`Fill` default** taking the parent allocation (then the scrim filling
that content rect), **not** from a Fill child driving the union. ZStack
introduces **no new `LayoutError`** and **no ZStack-specific Fill
special case**. **Owner-visible trade-off:** with no author
`width:`/`height:` surface in Phase 6, ZStack is an **overlay-first**
container — its default fills the parent, and an intrinsic ZStack
(sizing to its largest child on a bounded axis) is not expressible
until a future size-constraint surface; accepted because the lightbox
overlay is the Phase 6 driver.

## Per-child arrange / alignment default

### Options

- **AL1 — Default `center`, `h-align`/`v-align` overrides**
  - Each child is measured against the ZStack content rect, then
    anchored within it; default anchor is `center` on both axes. A
    `Fill` child / `Stretch` alignment expands to the full content rect
    (existing cross-axis rule).
  - What you gain: matches the lightbox wireframe (centered photo over a
    filling scrim) with no per-child ceremony; it is the SwiftUI ZStack
    default and reads as "stacked and centered", matching author
    intuition for an overlay.
  - What you give up: needs a per-child alignment carrier in the layout
    layer (a parallel placement vector, see Recommendation) — a bounded
    impl cost, mirroring Grid's `cell_placements`.

- **AL2 — Default `stretch` (mirror Grid Cell)**
  - Each child fills the ZStack content rect unless `h-align`/`v-align`
    override.
  - What you gain: consistent with the Grid Cell default.
  - What you give up: would stretch the photo to fill unless every
    overlay child opts out with `h-align: center; v-align: center` —
    the wrong default for overlays (a ZStack layer is an overlay that
    should sit at its natural size, not a slot to fill).

- **AL3 — No per-child alignment**
  - Every child fills the content rect, full stop (no override surface
    in Phase 6).
  - What you gain: the simplest contract.
  - What you give up: no way to center the photo over the scrim — the
    lightbox cannot be expressed.

### Comparison

The lightbox needs a **centered** photo over a **filling** scrim. Under
AL1, the scrim (Fill/Fill) fills regardless of alignment (a `Fill`
constraint expands to the full content rect by the existing cross-axis
rule), and the photo (intrinsic) centers by default — exactly the
wireframe. AL2 (stretch default) would stretch the photo to fill unless
every overlay child opts out with `h-align: center; v-align: center`,
which is the common case for overlays and therefore the wrong default.
The center default is also the SwiftUI ZStack default and reads as
"stacked and centered", which matches author intuition for an overlay
container. (Grid Cell defaults to stretch because a grid cell is a
*slot* the content should fill; a ZStack layer is an *overlay* that
should sit at its natural size, centered — different primitive,
different default, deliberately.)

### Recommendation

**AL1.**

- Each child is measured against the ZStack content rect and anchored
  within it. Default `h-align` / `v-align` = **`center`**. `Stretch`
  alignment or a `Fill` constraint expands the child to the full
  content rect (existing cross-axis rule, reused unchanged). `start` /
  `center` / `end` overrides anchor the child's measured size. All
  children share the **same** content rect (the overlap region) — that
  is the defining property of ZStack.
- **Per-child alignment carrier (impl-readiness).** `h-align` /
  `v-align` are authored as `IrProp` ident-literals on the ZStack child
  (DD-M3-P6-001, no new IR vocabulary). At the runtime layout layer they
  are carried as **parent-owned metadata parallel to `children`**,
  mirroring Grid's `cell_placements`
  ([layout.rs:224](../../../../wasamo-runtime/src/layout.rs)): a **lean**
  per-child placement (h/v `Alignment` only — **not** Grid's
  row/column/span `CellPlacement`) is extracted by `construct_widget`
  from each child's `IrProp`s in document order, so `LayoutNode.children[i]`
  is anchored by `placements[i]` and the arrange loop zips the two exactly
  as Grid does ([layout.rs:1220](../../../../wasamo-runtime/src/layout.rs)).
  The carrier is this parallel placement vector, **not** a generic
  per-child alignment field added to `WidgetNode` / `LayoutNode`.
  **Validation scope:** `h-align` / `v-align` are admitted only on a
  **ZStack direct child** (and a Grid `Cell`); on any other widget they
  are rejected at `wasamoc check` and runtime `validate()`. The exact
  placement struct/field set is an implementation-task detail; the
  contract is the carrier shape (parallel vector) and the admission scope.
- **Validation/extraction ordering (impl-readiness).** Because
  `h-align` / `v-align` are not props the child widget itself owns, the
  **parent context decides** before the child's own prop check: when the
  parent is a ZStack (or Grid for a `Cell`), `wasamoc check` and the
  loader `validate()` **consume `h-align` / `v-align` as parent-owned
  child-placement annotations first and exclude them from the child's
  own prop set**, so the child's normal unknown-prop rejection never
  sees them. When the parent is **not** a ZStack/Grid, the same two
  attributes are **rejected** (a placement annotation has no meaning
  outside an overlap/grid parent) rather than leaking through as an
  accepted child prop. So the two failure modes the ordering guards
  against — (i) a ZStack child's `h-align` wrongly rejected as an
  unknown child prop, and (ii) a non-ZStack child's `h-align` wrongly
  accepted — are both closed by "parent-context consume-then-exclude,
  reject elsewhere."

## z-order

### Options

- **Z1 — Document order = paint order**
  - Later child on top; no author-facing layering attribute. Inherits
    Grid DD-M3-P5-005.
  - What you gain: inherits the Grid precedent and **is the A4 wording
    itself** ("sibling z-order by document order"); no author surface to
    design or validate.
  - What you give up: no explicit author control over paint order —
    which is out of A4 scope anyway.

- **Z2 — Explicit `z-index` attribute**
  - Author controls paint order independent of document order.
  - What you gain: explicit per-child layering control.
  - What you give up: out of scope per the framing and the pre-doc; a
    new attribute to design, lower, and validate for no Phase 6 driver.

### Comparison

Z2 (`z-index`) is out of scope per the framing and the pre-doc — paint
order is document order, full stop. Z1 inherits the Grid precedent and
is the A4 wording itself ("sibling z-order by document order").

### Recommendation

**Z1.** Paint order = document order; first child at the bottom, last
child on top. No `z-index`. This is enforced by the existing
document-order `sync_visuals` insertion; ZStack changes nothing in
`sync_visuals` (the convention already paints children in child-vector
order).

## Clip

### Options

- **C1 — ZStack outer-bounds clip on, per-child clip out**
  - Inherits Grid DD-M3-P5-005 verbatim: the ZStack outer-bounds clip
    lands on ZStack's own Visual via `Visual.Clip = InsetClip{0,0,0,0}`;
    each child Visual has `Visual.Clip = null`.
  - What you gain: keeps an overflowing overlay child (e.g. a too-large
    photo placeholder) from painting past the ZStack rect, consistent
    with every other M3 container; reuses the Grid precedent (no new
    mechanism).
  - What you give up: no per-child clip surface (out of scope this
    phase).

- **C2 — No clip**
  - Children may paint past the ZStack rect.
  - What you gain: overflowing children remain fully visible.
  - What you give up: inconsistent with every other M3 container; an
    overflowing overlay leaks outside the ZStack bounds.

### Comparison

C1 inherits Grid DD-M3-P5-005. The lightbox positive control ("scrim
covers the thumbnails behind; photo/caption/nav painted over the
scrim") is a z-order proof, not a clip proof, but the outer-bounds clip
keeps an overflowing overlay child (e.g. a too-large photo placeholder)
from painting past the ZStack rect, consistent with every other M3
container.

### Recommendation

**C1.** The ZStack outer-bounds clip lands on ZStack's own Visual via
`Visual.Clip = InsetClip{0,0,0,0}` (no intermediate Visual, 1
WidgetNode = 1 Visual). Per-child clip is out of scope; each child
Visual has `Visual.Clip = null` (regression guard, symmetric with
WrapPanel / ScrollView / Grid).

## Spec content seed

Normative spec content for `dsl_spec.md` §4.13: the "stacked, centered,
back-to-front, clipped to bounds" mental model; the union sizing rule
with the Fill interaction; the alignment default and override table;
the document-order z-order rule; the outer-bounds clip rule.

## Forward-compat exposure

- **`z-index`** — additive child attribute later; document order stays
  the default when absent. No ZStack shape change.
- **Per-child clip (`clip:` on a child)** — additive child attribute
  later; the ZStack outer-bounds clip remains independent. (Grid's
  DD-M3-P5-005 forward-compat reused.)
- **ZStack background `fill`** — could become a ZStack-level attribute
  later (allow-list grows); Phase 6's scrim is a child `Box`.
- **Alignment vocabulary** — `start`/`center`/`end`/`stretch` reuses
  the existing `Alignment` enum; a future baseline/edge anchor would
  be an additive enum variant, no ZStack change.
- **Per-widget size-constraint surface** (`width:`/`height:` =
  `fill`/`shrink`/fixed) — not in Phase 6 (no widget exposes one). When
  it lands, it is the path that lets an author override ZStack's
  `Fill/Fill` default to an intrinsic (union-sized) ZStack, relaxing the
  overlay-first default the Sizing comparison commits to. Additive; no
  ZStack-shape change (the engine already resolves `Fill`/`Shrink`/
  `Fixed` per axis — [layout.rs:83](../../../../wasamo-runtime/src/layout.rs)).

## Technical risk re-evaluation

- **No new `LayoutError`** ⇒ no new failure-mode surface; the
  unbounded-star precedent (Grid DD-M3-P5-004) does not recur because
  ZStack has no intrinsic sizing pass that diverges on unbounded axes
  — it defers entirely to child Fill/Shrink resolution.
- **z-order is not dischargeable by pure logic alone** — the layout
  layer computes per-child rects, but "later child paints on top" is a
  Visual-tree property. Per the verification closure, the z-order
  proof requires a Windows-runtime integration test asserting child
  Visual order, plus the assistant/owner visible positive control
  (scrim dim + over-painting on the open lightbox frame). Pure-logic
  tests cover sizing/arrange/alignment only.
- **Fill-scrim interaction** is the one subtlety, and an earlier draft
  mis-stated its direction: a `Fill` child measures **`0.0`**
  ([layout.rs:440](../../../../wasamo-runtime/src/layout.rs)), so it does
  **not** drive the ZStack's size. The full-viewport scrim comes from the
  **ZStack's own `Fill/Fill` default** taking the parent allocation,
  after which the scrim fills the content rect in *arrange*. The
  pure-logic test covers (i) union desired-size on a Shrink/unbounded
  axis (a Fill child contributing `0.0`) and (ii) a Fill child filling
  the ZStack content rect under a bounded allocation — no ZStack-specific
  Fill override path.
- **No intermediate Visual** keeps `sync_visuals` untouched (the
  ScrollView intermediate-Visual complexity is a negative precedent,
  deliberately not repeated), so the z-order/clip behaviour rides the
  already-tested document-order insertion path.
