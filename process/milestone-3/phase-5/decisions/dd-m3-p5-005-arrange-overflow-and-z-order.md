### DD-M3-P5-005 — Arrange algorithm, overflow, and z-order

**Status:** Accepted

**Context:** Once DD-M3-P5-004 resolves per-axis tracks and per-`Cell`
rectangles, the arrange pass places each `Cell`'s single content
child within its resolved rectangle. Phase 5 must commit to:

- the child-alignment surface within a resolved cell (default and
  override mechanism);
- the overflow / clipping policy at three levels (per-cell, Grid
  outer-bounds, and intentional overlay);
- the z-order rule for paint-overflow that incidentally overlaps;
- the visual-layer ownership (intermediate Visual vs Grid's own
  Visual carries clip and offsets);
- the verification fixture parent shape (production root shape per
  Phase 4 carry-forward).

The 2026-05-28 owner alignment settled the **structurally branching
sub-decisions**: paint overflow between cells is allowed; Grid
applies an **outer-bounds clip**; per-cell clipping stays out of
scope; the **z-order rule** is document order (later child paints
on top), without an explicit `z-index` attribute. The remaining
sub-decisions (alignment carrier and defaults, Visual ownership,
fixture shape) are written below as Recommendations and approved at
ADR review.

**Sub-issues:**

- **Child alignment inside cell.** Default and override mechanism;
  alignment-carrier is `Cell` per DD-M3-P5-001 / DD-M3-P5-003.
- **Per-cell clipping.** Out of scope; the `Cell` rectangle does
  not clip its content child.
- **Grid outer-bounds clipping.** Owner-settled: on. Grid's own
  Visual carries `Visual.Clip = InsetClip{0,0,0,0}`.
- **Paint order within Grid.** Owner-settled: document order
  (later child paints on top); no explicit `z-index`.
- **Visual ownership.** Grid does not introduce an intermediate
  Visual (unlike Phase 4 ScrollView). The 1 WidgetNode = 1 Visual
  convention is preserved.
- **Verification fixture parent shape.** Production root shape
  coverage per Phase 4 T6 carry-forward (Phase 5 constraints.md
  §1).

**Options (child alignment inside cell):**

- **Option A — Stretch / stretch default; per-`Cell` `h-align` /
  `v-align` override admitted in Phase 5 (recommended).**
  - `h-align` values: `start`, `center`, `end`, `stretch`
    (default).
  - `v-align` values: `start`, `center`, `end`, `stretch`
    (default).
  - When stretch, the content fills the resolved cell rectangle on
    that axis; the content's measured size on that axis is the
    cell extent.
  - When non-stretch, the content measures naturally on that axis
    and is anchored at the start / center / end of the resolved
    cell rectangle.
  - Attributes live on `Cell` (DD-M3-P5-001 alignment-carrier
    Option A), not on the content widget.
  - What you gain: practical Grid layouts work without wrapper
    hacks (centered title, right-aligned actions, icon
    placement); stretch default keeps the Grid composition
    primitive useful for box-fill cells (the FD-H gallery proof
    uses stretch for color boxes); alignment surface matches the
    structural-family convention so a hypothetical future surface
    family change does not relocate alignment.
  - What you give up: introduces six new identifier-literal
    values (`start` / `center` / `end` / `stretch` across two
    axes) into the `wasamoc check` vocabulary; small surface
    cost.
- Option B — Stretch only; no per-`Cell` alignment override in
  Phase 5.
  - What you gain: smallest surface.
  - What you give up: makes the FD-H gallery proof's "Gallery"
    title impossible to center horizontally within its spanning
    header cell without a wrapper; framing FD-C explicitly
    recommends admitting per-Cell alignment in Phase 5; defers a
    near-universal layout need for no scope benefit.
- Option C — Per-`Cell` `align: <h>,<v>` compound attribute.
  Rejected on the same grounds as DD-M3-P5-003 placement Option B
  (avoid Grid-specific compound-literal grammar).

**Options (per-cell clipping):**

- **Option A — Per-cell clipping out of scope (recommended).** A
  `Cell`'s resolved rectangle does **not** clip its content
  child's paint. Content that exceeds the cell rectangle paints
  past the cell boundary (and may be visible if another `Cell`
  does not paint over it; this is governed by the document-order
  z-order rule below).
  - What you gain: Grid stays a pure layout container (like
    WrapPanel) and does not become a clipping primitive (unlike
    ScrollView); the `Cell` rectangle is a measure-arrange
    rectangle, not a paint-clip rectangle; per-cell clip is
    additive in a future phase (Post-Phase-5 hand-off item 6) if
    author demand warrants.
  - What you give up: a `Cell` whose content is larger than its
    resolved rectangle will paint overflow; the Grid outer-bounds
    clip below contains it within Grid's overall rectangle.
- Option B — Per-cell clipping on by default. Every `Cell`'s
  Visual carries an inner clip matching its resolved rectangle.
  - What you gain: visually compartmentalised cells.
  - What you give up: introduces per-cell Visual.Clip
    installation work in the Visual-layer sync; turns `Cell` into
    a clip-owning primitive that diverges from "layout-only
    wrapper" framing; Phase 5 has no acceptance requirement for
    per-cell clip; framing settled this as out of scope.

**Options (Grid outer-bounds clipping):**

- **Option A — Grid outer-bounds clip on; Grid's own Visual
  carries `Visual.Clip = InsetClip{0,0,0,0}` (recommended; owner-
  settled at framing).** Grid's outer rect on a **bounded** axis
  equals the parent's allocation (per Phase 3 WrapPanel /
  Phase 4 ScrollView precedent: Grid does not grow to accommodate
  an oversized track-resolved extent — see DD-M3-P5-004 "Grid
  outer rect"). On an **unbounded** axis (only reachable with no
  star tracks per DD-M3-P5-004 unbounded-star branch), Grid's
  outer rect equals the track-resolved `fixed_sum`. Grid's outer
  rectangle is Grid's `Visual` rectangle; the
  `Visual.Clip = InsetClip{0,0,0,0}` applies to that rectangle.
  Cell rectangles inside Grid use the prefix boundaries from
  DD-M3-P5-004; Cell rectangles that extend past the outer rect
  (the `fixed_sum > bound` case) overflow and the clip cuts off
  their paint.
  - What you gain: a Grid that does not fit its parent (e.g.
    fixed-track sum exceeds parent bound; oversized spanning
    child) does not bleed into sibling layout regions; the clip
    semantics are local to Grid's Visual (no intermediate Visual
    is required for the clip); the clip composes with parent
    clips (the parent's clip already constrains the larger
    region; the Grid clip adds an inner constraint matching
    Grid's resolved outer extent).
  - What you give up: Grid that intentionally wants to paint
    outside its parent allocation must redesign with sizing; the
    clip is a structural commitment, not an author-facing
    attribute.
- Option B — No Grid outer-bounds clip; paint overflow may exit
  Grid's outer rectangle.
  - What you gain: no clip installation work.
  - What you give up: Grid becomes the only M3 layout primitive
    whose paint can escape its own Visual rectangle (Phase 4
    ScrollView clips its viewport; Phase 3 WrapPanel paints within
    its own outer-rect by construction since children are placed
    inside lines); inconsistent; owner-rejected at framing.

**Options (paint order within Grid):**

- **Option A — Document order (recommended; owner-settled).**
  Children are painted in document order; later children paint on
  top of earlier children when their paint regions incidentally
  overlap (paint overflow exits a Cell's rectangle and crosses
  into another Cell's rectangle). No explicit `z-index` attribute.
  - What you gain: predictable rule with zero new surface
    attributes; consistent with the existing M3 sibling paint
    order; intentional overlay (multiple deliberately-overlapping
    paint regions) is **not** Grid's responsibility — that is
    ZStack (Phase 6) — so Grid does not need a layering surface;
    the rule is local (no Grid-wide z-stack computation).
  - What you give up: an author who wants a specific paint order
    must reorder document children; no way to express "Cell A
    paints below Cell B" without source reordering.
- Option B — Explicit `z-index` attribute on `Cell`. Author
  declares paint order numerically.
  - What you gain: source order independent of paint order.
  - What you give up: introduces a numeric layering attribute
    Phase 5 has no acceptance requirement for; same-cell overlap
    is rejected (DD-M3-P5-003) so the principal use case for
    `z-index` does not apply; framing explicitly settled this as
    out of scope; ZStack is the surface for intentional overlay.
- Option C — Cell-rectangle paint order = row-major order
  (independent of document order). Rejected because it makes
  paint order non-local to document inspection.

**Options (Visual ownership):**

- **Option A — Grid uses the 1 WidgetNode = 1 Visual convention;
  no intermediate Visual (recommended).** Grid's own `Visual`
  carries `Visual.Clip = InsetClip{0,0,0,0}`. Each `Cell`'s
  content child Visual is a direct child of Grid's Visual in
  `sync_visuals()`. (`Cell` itself does not own a Visual in the
  Composition layer — it is a layout-only IR wrapper, and its
  resolved rectangle is applied to the content child's Visual
  offset.)
  - What you gain: matches WrapPanel's convention (no
    intermediate Visual); avoids the Phase 4 ScrollView pattern
    (Phase 4 needed an intermediate Visual for the scroll-offset
    translation, which Grid does not have); smaller Visual tree;
    `sync_visuals()` requires no Grid-specific extension.
  - What you give up: per-cell clipping (out of scope per Option
    A above) would require an intermediate Visual if admitted
    later; that future phase's surface change is the right place
    to revisit, not Phase 5.
- Option B — Grid owns an intermediate clip-host Visual between
  its outer Visual and the cell content Visuals.
  - What you gain: clip and content offsets are separated across
    two Visuals (cosmetically cleaner separation).
  - What you give up: extends the `sync_visuals()` convention
    again (Phase 4 was the first extension; Phase 5 would be the
    second in two consecutive phases); no concrete need (Grid has
    no translation analog to ScrollView's offset, so the
    intermediate Visual would carry zero offset); framing FD-C
    notes the Phase 4 ScrollView pattern as a *negative*
    precedent for Grid.

**Options (verification fixture parent shape):**

- **Option A — Two integration fixtures: Grid-rooted and
  `VStack { Grid {...} }`-rooted (recommended; per Phase 4 T6
  carry-forward).** Both shapes cover the runtime-boundary class
  that Phase 4 T6 surfaced.
  - What you gain: the Grid-rooted fixture exercises the
    window-root Fill/Fill path (`WidgetNode::run_layout_as_window_root`);
    the VStack(Grid)-rooted fixture matches the current gallery /
    counter / bool-demo `.ui` production root family and guards
    against the same runtime-boundary collapse class that hit
    Phase 4 T6; helper infrastructure can be shared between the
    two fixtures, but each is a distinct evidence line.
  - What you give up: two fixtures instead of one (small evidence
    cost paid in exchange for the carry-forward discipline).
- Option B — One integration fixture (Grid-rooted only).
  - What you gain: smallest evidence surface.
  - What you give up: contradicts Phase 4 T6 carry-forward
    (constraints.md §1: integration test fixture parent shape
    must cover production root shape); Phase 4 T6 lesson says
    helper-compatible tests can miss production root collapse;
    rejected on carry-forward grounds.

**Decision (Recommendation):**

- Child alignment inside cell: **Option A** (stretch default;
  per-`Cell` `h-align` / `v-align` admitted with `start` /
  `center` / `end` / `stretch` values).
- Per-cell clipping: **Option A** (out of scope; Cell rectangle
  does not clip content).
- Grid outer-bounds clipping: **Option A** (on; `Visual.Clip =
  InsetClip{0,0,0,0}` on Grid's own Visual) — owner-settled at
  framing.
- Paint order: **Option A** (document order; no `z-index`) —
  owner-settled at framing.
- Visual ownership: **Option A** (1 WidgetNode = 1 Visual; no
  intermediate Visual).
- Verification fixture parent shape: **Option A** (Grid-rooted +
  VStack(Grid)-rooted; both required) — per Phase 4 T6 carry-
  forward.

**Forward-compat exposure:**

- **Per-cell clipping (Post-Phase-5 hand-off item 6).** Future
  per-cell clip would install `Visual.Clip` on each `Cell`'s
  Visual (which would also require admitting Cell-owned Visuals,
  reversing the Option A Visual-ownership decision for the
  per-cell-clip case). Phase 5 does not foreclose this; it is the
  natural extension if author demand warrants.
- **Author-facing layering attribute.** Phase 5 fixes paint order
  to document order; a future phase may admit `Cell { z-index: ...
  }` if a use case emerges that ZStack cannot serve; Phase 5 does
  not foreclose this.
- **Intermediate Visual for transforms.** A future Grid
  transformation surface (rotation, scaling) would land on an
  intermediate Visual; Phase 5 does not introduce one but the
  pattern (Phase 4 ScrollView's intermediate Visual) is available
  as precedent.
- **Per-axis alignment-policy attribute on Grid.** A future
  `Grid { default-h-align: start ... }` would change the per-Cell
  default; additive at Grid level.

**Technical risk re-evaluation:**

- **Paint overflow between cells under common content sizes.**
  For practical Grid sizes with Box-fill or Text content, overflow
  is rare (Box stretches to fill; Text wraps within its measured
  width). The principal overflow case is intentional (an oversized
  spanning child) or pathological (an over-tight Grid where fixed
  tracks exceed parent bound, per DD-M3-P5-004 negative-remaining-
  space). The document-order z-order rule covers both.
- **Grid outer-bounds clip composition with parent clips.** The
  Grid's clip is `InsetClip{0,0,0,0}` applied to Grid's own
  rectangle; the parent's clip (if any) already constrains the
  parent's region. The composition is the intersection of clip
  rectangles; standard Composition semantics. No special
  composition rule is required.
- **Visual-ownership stability under future per-cell clip.**
  Admitting per-cell clip later would require admitting Cell-owned
  Visuals; this is a structural change but it does not violate
  Phase 5's contract (the per-cell clip surface is explicitly
  out of scope, so future admission of it is the right place for
  the structural change).
- **Alignment stretch with naturally-sized content.** When the
  alignment is stretch on an axis, the content is measured with
  the cell extent on that axis. Content that requests its
  natural measure may be smaller than the cell; the existing
  layout-engine convention is to extend the content to the cell
  extent on the stretch axis. For Text, this means stretch-h
  produces a Text whose measured-width equals the cell width
  (typical Text layout behavior under bounded width).
- **Stretch vs intrinsic content on the unbounded star branch.**
  This branch errors at DD-M3-P5-004 (unbounded-star); arrange
  does not run.

**Layering with DD-M3-P5-001 / DD-M3-P5-002 / DD-M3-P5-003 /
DD-M3-P5-004 / DD-M3-P5-006:**

- DD-M3-P5-005 consumes:
  - Per-`Cell` resolved rectangle from DD-M3-P5-004.
  - `Cell` alignment attributes (`h-align`, `v-align`) from
    DD-M3-P5-001 / DD-M3-P5-003.
  - Document order of `Cell`s within Grid (existing IR child
    ordering).
- DD-M3-P5-005 produces:
  - Per-content-widget arranged rectangle (the content child's
    final `(x, y, w, h)` in Grid-local coordinates, after
    alignment).
  - Grid's outer Visual clip installation
    (`Visual.Clip = InsetClip{0,0,0,0}`).
  - Document-order paint sequence (existing `sync_visuals()`
    convention).
- DD-M3-P5-006 dual-gates the alignment-value vocabulary at
  `wasamoc check` and `validate()` (invalid `h-align` / `v-align`
  values surface diagnostics).

Invalid combinations explicitly rejected:

- DD-M3-P5-005 = per-cell clip on + DD-M3-P5-001 = `Cell` is
  layout-only without a Visual. Does not arise: per-cell clip is
  out of scope.
- DD-M3-P5-005 = intermediate Visual + DD-M3-P5-001 = 1 WidgetNode
  = 1 Visual preserved. Does not arise: Recommendation Option A
  preserves the convention.

**Spec content seed (Moment 1 §4.12 draft):**

The DD-M3-P5-005 sub-issues map 1:1 to the §4.12 chapter arrange /
overflow / z-order section:

1. After track resolution (DD-M3-P5-004), each `Cell`'s resolved
   rectangle is `(column_boundary[column], row_boundary[row],
   column_boundary[column + column-span] -
   column_boundary[column], row_boundary[row + row-span] -
   row_boundary[row])`.
2. The content widget is placed within the resolved cell rectangle
   per `Cell`'s `h-align` and `v-align` attributes; defaults are
   stretch / stretch.
3. Stretch alignment extends the content to the cell extent on the
   axis; non-stretch anchors the content's natural measure at
   start / center / end.
4. The content widget is **not** clipped by the cell rectangle;
   paint that exceeds the cell rectangle may paint into sibling
   regions within the Grid.
5. Grid's outer rect on a bounded axis equals the parent
   allocation (Grid does not grow to accommodate oversized
   track-resolved extent); on an unbounded axis (only reachable
   with no star tracks), it equals `fixed_sum`. The
   `Visual.Clip = InsetClip{0,0,0,0}` on Grid's own Visual applies
   to this rectangle and prevents paint from escaping it.
6. Paint order is document order: later `Cell`s in source paint on
   top of earlier `Cell`s when their paint regions overlap.
7. Grid does not provide intentional overlay; same-cell occupancy
   is rejected (DD-M3-P5-003). Intentional overlay is Phase 6
   ZStack.
8. Grid uses the existing 1 WidgetNode = 1 Visual convention; no
   intermediate Visual is introduced.
