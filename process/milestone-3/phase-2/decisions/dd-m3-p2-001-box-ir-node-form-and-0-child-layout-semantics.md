### DD-M3-P2-001 — Box IR node form and 0+ child layout semantics

**Status:** Accepted

**Context:**
Box is a new layout primitive in `wasamo-ir` and `wasamo-runtime`.
Phase 2 must commit to (i) the IR node shape, (ii) the 0-child
shape, (iii) the child measure pass, (iv) child alignment within
Box bounds, (v) overflow / clip behaviour, and (vi) the multi-child
semantics. The last is the load-bearing sub-issue: Box's N-child
layout must **not** be a back-door ZStack — overlay is A4 /
Phase 6's responsibility, and Phase 2's contract here directly
shapes what Phase 6 ZStack's primitive contribution can be.

**Options (IR node shape):**

Option A — Per-kind tag parallel to `HStack` / `VStack` / `Rectangle`
(recommended)
- `WidgetKind::Box` joins the existing per-kind enumeration; the
  layout function in `wasamo-runtime` dispatches on the tag.

  - What you gain: Symmetric with every existing M2 widget. Pattern
    matching on `WidgetKind` is exhaustive at compile time.
  - What you give up: A new tag everywhere `WidgetKind` is matched.
    The set is small and discoverable.
  - **Technical risk:** Low. Pure additive extension of an existing
    enum.

Option B — Structural variant in an `IrLayout` umbrella
- Introduce an `IrLayout` family enum carrying Box, HStack, VStack as
  variants of a "layout container" category, distinct from leaf
  widgets (Text / Button / Rectangle).

  - What you give up: A new structural axis without payoff — M2 already
    treats HStack / VStack as per-kind tags, not as a separate family.
    Adopting an umbrella here would re-open DD-M2-P6 territory for one
    new widget, mid-milestone.
  - **Technical risk:** Medium. Touches every M2 layout dispatch site.

**Options (0-child shape):**

A Box with no children but with `aspect` and/or `fill` must still
produce a visual rectangle. This is the placeholder-shape minimum and
the structural support for DD-006.

Option A — `Box { }` (0 children) is valid and renders the
aspect-derived rectangle filled with `fill` (recommended)
- The IR loader admits empty `children` lists; the layout pass
  produces a sized rectangle; the visual pass paints `fill` (or
  transparent if `fill` is absent).

  - **Technical risk:** Low.

Option B — Reject 0-child Box at IR load
- Diagnoses empty containers as a `wasamoc check` error.

  - What you give up: DD-006's placeholder pattern degenerates: a
    Box-with-just-fill rectangle cannot exist as a structural scrim
    or as the "no label yet" thumbnail in Phase 3.
  - **Technical risk:** Low to implement; pays back negatively in
    DSL ergonomics.

**Options (multi-child semantics — load-bearing):**

Option A — Single-child-only; multi-child rejected at `wasamoc check`
**and** at `ir_loader::build_node` (defense in depth) (recommended)
- A Box admits 0 or 1 child. 2+ children is a compile-time diagnostic
  in `wasamoc check`, **and** is independently rejected when
  `wasamo-runtime::ir_loader` materialises a Box IR node with `len(children) > 1`.
  Both gates are required because `wasamo_load_ui`'s memory-IR path does
  not pass through the compiler; the runtime gate is the last line of
  defence for the spec invariant. The "0+ child container" wording of A6
  is honoured by admitting 0 and 1; "+" is read as "at-most-one in Phase
  2" with the surface widened (if at all) by Phase 6 when ZStack lands.

  - What you gain: Maximum structural defence against a back-door
    ZStack. Phase 6's ZStack gets full latitude to define z-order
    and multi-child overlap semantics without inheriting an implicit
    Phase 2 contract. The two M3 gallery uses of Box (0-child scrim,
    1-child placeholder) both fit. The diagnostic message points
    users at ZStack / VStack / HStack for multi-child needs. The
    `ir_loader` gate makes the invariant hold even for IR produced
    outside `wasamoc`.
  - What you give up: A6's "0+" surface wording is narrowed at the
    spec level — readers see "Box admits 0–1 child in M3 Phase 2;
    multi-child overlap belongs in ZStack (Phase 6)." This is a real
    public-surface narrowing, recorded in `docs/dsl_spec.md` and in
    the Phase 2 spec marker.
  - **Technical risk:** Low. Both gates are small additions.

Option B — All children share full Box bounds; no z-order declared
- The IR admits N children. Each child measures against Box bounds;
  their visual stacking order is document order, but no z-order
  *semantics* are spec'd — overlapping behaviour is "implementation
  defined" until Phase 6 ZStack lands.

  - What you gain: Honours A6's "0+" literally. No Phase 2 → Phase 6
    spec drift if Phase 6 chooses to re-spec on top.
  - What you give up: "Implementation defined" overlap is a footgun
    Phase 6 will inherit. Either Phase 6 ZStack confirms the implicit
    behaviour as the explicit one (so Phase 2 silently set the
    contract) or it contradicts it (so Phase 6 has to break Phase 2's
    proof). The framing flags this as the back-door-ZStack risk.

Option C — Document-order top-left stacking, each child consuming
the next available space
- Stack-of-rows semantics inside Box. Effectively a degenerate VStack.

  - What you give up: Adds a third stacking primitive to M3 mid-milestone
    with no acceptance criterion calling for it. Conflicts with the
    "pure primitive" framing of Phase 2.

**Options (child measure pass; conditional on at least 1 child):**

Option A — Box's resolved outer bounds (from DD-005) are passed
through to the child as the child's measure constraint (recommended)
- The child measures against the full inner bounds. Smaller children
  align (per the alignment sub-decision below); larger children clip
  (per the overflow sub-decision below).

  - **Technical risk:** Low.

Option B — Child intrinsic size capped at Box bounds (`min(intrinsic,
box)`)
- The child gets its intrinsic size if it fits, the Box bound otherwise.

  - What you give up: Two layout behaviours for "child smaller than
    Box" (intrinsic) vs "child larger than Box" (capped). A child's
    visual position depends on its intrinsic dimensions in non-obvious
    ways. Phase 3 WrapPanel-of-thumbnails would inherit this
    variability.

**Options (child alignment within Box bounds; conditional on at
least 1 child):**

Option A — Center (recommended)
- A child smaller than Box is centred horizontally and vertically
  inside the Box. No per-child override in Phase 2.

  - What you gain: Matches the placeholder use case (a Text label
    centred over a coloured rectangle is the visual the M3 gallery
    references). No new attribute surface in Phase 2.
  - What you give up: No top-left / per-child alignment in Phase 2.
    If a later phase needs per-child alignment (e.g. a "caption
    bottom-aligned" pattern), it opens its own DD; Phase 2 reserves
    no surface for it.

Option B — Top-left
- The child anchors at Box's top-left corner.

  - What you give up: Visual mismatch with the placeholder use
    case — labels typically read centred.

Option C — Configurable per-child via a new attribute
- Add `align: <center|top-left|...>` to children of Box.

  - What you give up: New attribute surface unmotivated by any
    Phase 2 acceptance criterion. Out of phase scope; defer.

**Options (overflow / clip; conditional on at least 1 child):**

Option A — Clip the child to Box bounds (recommended)
- A child measuring larger than Box bounds is visually clipped to
  the rectangle. Layout slot does not grow.

  - What you gain: Consistent with M4 ScrollView's separate clip
    surface — Phase 4 inherits a Box that already clips, so
    ScrollView's contribution is the *scrollable viewport*, not the
    clipping primitive. Honours the layering note: aspect-derived
    bounds are inviolable.
  - **Technical risk:** Low. A clip rectangle is a Direct2D / Visual
    Layer primitive.

Option B — Visible overflow (child paints outside Box bounds)
- The child renders at its intrinsic / measured size, painting
  outside the Box's visual rectangle.

  - What you give up: Breaks the "Box visually equals its
    aspect-derived rectangle" contract that placeholders and scrims
    rely on. Adjacent siblings (Phase 3 WrapPanel-of-Boxes) would
    paint over each other if any one overflows.

**Recommendation:** Option A for every sub-issue —

- IR shape: per-kind tag (`WidgetKind::Box`).
- 0-child: valid; renders aspect-derived rectangle filled with
  `fill`.
- Multi-child: single-child-only; 2+ rejected at both `wasamoc check`
  and `ir_loader::build_node` (defense in depth).
- Child measure: Box bounds passed through unchanged.
- Child alignment: centred.
- Overflow: clip.

Design quality dominates here, particularly on multi-child. The
single-child-only stance is the load-bearing defence against
inheriting an implicit ZStack contract. The placeholder use case
(`Box { aspect: 1:1; fill: #ccc; Text { ... } }`) and the scrim use
case (`Box { fill: #00000080 }`) are both 0 or 1 child; A6's "0+"
surface is narrowed accordingly in the spec text. Phase 6's ZStack
ADR is then free to widen the multi-child surface in whichever
shape ZStack needs, without contradicting Phase 2.

**Forward-compat exposure:** Option A's exposure under foreseeable
future events (see Out of scope below):

- Phase 6 ZStack opens multi-child overlap. The narrowed "single-child"
  Box surface is structurally compatible: ZStack widens the
  *multi-child* surface separately; Box's single-child contract does
  not need revision. Option B would have ZStack contradicting or
  ratifying an implicit Phase 2 multi-child contract, so its
  exposure is asymmetrically higher.
- A future "image widget" landing in M4+ does not pressure Box's
  child-layout contract — DD-006's placeholder pattern is the bridge,
  and image widgets become leaf children of Box like Text does today.
- The `align: ...` per-child attribute (Option C of the alignment
  sub-issue) is additive if a later phase needs it; Phase 2's "centred,
  no override" default does not foreclose it.

---
