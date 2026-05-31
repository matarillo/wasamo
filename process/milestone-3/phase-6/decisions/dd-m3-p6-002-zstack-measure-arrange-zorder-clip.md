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

## Options

### Sizing policy

- **S1 — Union (per-axis max) of children (recommended).** ZStack's
  desired size on each axis = the **max** of its children's measured
  desired sizes on that axis. A `Fill` child resolves to the parent
  allocation (existing Fill rule), so a ZStack containing a Fill
  scrim resolves to the parent allocation; a ZStack of only intrinsic
  children sizes to the largest child. This is the SwiftUI ZStack
  model.
- **S2 — Always Fill the parent allocation.** ZStack ignores child
  sizes and always consumes the full parent rectangle.
- **S3 — Size to the first (or a designated) child.** ZStack tracks a
  single "primary" child's size.

### Per-child arrange / alignment default

- **AL1 — Default `center`, `h-align`/`v-align` overrides
  (recommended).** Each child is measured against the ZStack content
  rect, then anchored within it; default anchor is `center` on both
  axes. A `Fill` child / `Stretch` alignment expands to the full
  content rect (existing cross-axis rule).
- **AL2 — Default `stretch`** (mirror Grid Cell). Each child fills the
  ZStack content rect unless `h-align`/`v-align` override.
- **AL3 — No per-child alignment** — every child fills the content
  rect, full stop (no override surface in Phase 6).

### z-order

- **Z1 — Document order = paint order (recommended).** Later child on
  top; no author-facing layering attribute. Inherits Grid
  DD-M3-P5-005.
- **Z2 — Explicit `z-index` attribute.** Author controls paint order.

### Clip

- **C1 — ZStack outer-bounds clip on, per-child clip out
  (recommended).** Inherits Grid DD-M3-P5-005 verbatim.
- **C2 — No clip** (children may paint past the ZStack rect).

## Comparison

**Sizing.** S2 (always Fill) is wrong for a general overlap primitive:
a ZStack used to stack a few intrinsic badges over an icon should size
to the icon, not to the whole parent. S3 (primary child) needs an
author concept of "primary" that the surface (DD-001) does not have.
S1 (union) is the standard, composes correctly with `Fill` (the
lightbox scrim is `Box { fill … }` sized Fill/Fill, which drives the
root ZStack to the parent allocation **through** the normal Fill rule,
not through a ZStack special case), and needs no new vocabulary. S1
also keeps ZStack's unbounded-axis behaviour identical to the existing
conventions: a `Fill` child on an unbounded parent axis follows the
same rule the engine already applies (no ZStack-specific
`LayoutError`).

**Alignment default.** The lightbox needs a **centered** photo over a
**filling** scrim. Under AL1, the scrim (Fill/Fill) fills regardless
of alignment (a `Fill` constraint expands to the full content rect by
the existing cross-axis rule), and the photo (intrinsic) centers by
default — exactly the wireframe. AL2 (stretch default) would stretch
the photo to fill unless every overlay child opts out with
`h-align: center; v-align: center`, which is the common case for
overlays and therefore the wrong default. The center default is also
the SwiftUI ZStack default and reads as "stacked and centered", which
matches author intuition for an overlay container. (Grid Cell defaults
to stretch because a grid cell is a *slot* the content should fill;
a ZStack layer is an *overlay* that should sit at its natural size,
centered — different primitive, different default, deliberately.)

**z-order.** Z2 (`z-index`) is out of scope per the framing and the
pre-doc — paint order is document order, full stop. Z1 inherits the
Grid precedent and is the A4 wording itself ("sibling z-order by
document order").

**Clip.** C1 inherits Grid DD-M3-P5-005. The lightbox positive control
("scrim covers the thumbnails behind; photo/caption/nav painted over
the scrim") is a z-order proof, not a clip proof, but the outer-bounds
clip keeps an overflowing overlay child (e.g. a too-large photo
placeholder) from painting past the ZStack rect, consistent with every
other M3 container.

## Recommendation

**S1 + AL1 + Z1 + C1.**

- **Sizing (S1):** ZStack's desired size on each axis is the per-axis
  **max** of its children's measured desired sizes. `Fill` children
  resolve through the existing Fill rule (so a Fill child yields a
  parent-allocation-sized ZStack). ZStack introduces **no new
  `LayoutError`**: unbounded-axis behaviour is whatever the children's
  own Fill / Shrink resolution already produces under the existing
  conventions.
- **Arrange / alignment (AL1):** each child is measured against the
  ZStack content rect and anchored within it. Default `h-align` /
  `v-align` = **`center`**. `Stretch` alignment or a `Fill` constraint
  expands the child to the full content rect (existing cross-axis
  rule, reused unchanged). `start` / `center` / `end` overrides anchor
  the child's measured size. All children share the **same** content
  rect (the overlap region) — that is the defining property of ZStack.
- **z-order (Z1):** paint order = document order; first child at the
  bottom, last child on top. No `z-index`. This is enforced by the
  existing document-order `sync_visuals` insertion; ZStack changes
  nothing in `sync_visuals` (the convention already paints children in
  child-vector order).
- **Clip (C1):** the ZStack outer-bounds clip lands on ZStack's own
  Visual via `Visual.Clip = InsetClip{0,0,0,0}` (no intermediate
  Visual, 1 WidgetNode = 1 Visual). Per-child clip is out of scope;
  each child Visual has `Visual.Clip = null` (regression guard,
  symmetric with WrapPanel / ScrollView / Grid).

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
- **Fill-scrim interaction** is the one subtlety: the lightbox relies
  on a Fill/Fill scrim driving the root ZStack to the parent
  allocation. This is the existing Fill rule, not a ZStack special
  case, so it is covered by the union-sizing + Fill-child unit test
  rather than a bespoke path — lower risk than a ZStack-specific Fill
  override would carry.
- **No intermediate Visual** keeps `sync_visuals` untouched (the
  ScrollView intermediate-Visual complexity is a negative precedent,
  deliberately not repeated), so the z-order/clip behaviour rides the
  already-tested document-order insertion path.
