### DD-M3-P4-004 — Clip surface installation and Composition primitive choice

**Status:** Accepted

**Context:** A5 names "viewport clip" as a load-bearing
component. Phase 3 T8 established that **WrapPanel installs no
clip surface** (see
[Phase 3 ADR DD-005 oversized-line section](./m3-phase-3-wrap-panel.md));
ScrollView is the **dual** — it must install a clip surface
because the gallery's overflow state (`ScrollView { … }`) is
exactly where the "parent clips" contract Phase 3 deferred to
becomes active. The DD settles (i) which Composition primitive
implements the clip, (ii) which Composition primitive applies the
offset, and (iii) where in the Visual tree the clip sits.

**Options (clip primitive):**

- **Option A — `Visual.Clip = InsetClip{0,0,0,0}` (recommended).**
  An InsetClip whose insets are all zero, applied to the
  ScrollView's outer Visual whose extent matches the viewport.
  - What you gain: canonical Windows.UI.Composition pattern for
    "clip to my own bounds"; the clip extent automatically
    follows the Visual's `Size` property on resize; no manual
    rectangle bookkeeping.
  - What you give up: nothing relative to the alternatives;
    InsetClip is the most idiomatic primitive for this use.
- Option B — `Visual.Clip = RectangleClip` (or
  `CompositionGeometricClip` with `CompositionRectangleGeometry`)
  with explicit `(0, 0, viewport_w, viewport_h)` extent.
  - What you gain: explicit rectangle is easier to reason about
    when reading code.
  - What you give up: manual rectangle bookkeeping — the
    rectangle must be re-set on every viewport size change
    (window resize), adding a code path Option A does not need.
- Option C — `Visual.Clip = InsetClip` with non-zero insets
  derived from a future `padding` attribute.
  - What you gain: forward-compatible with future padding.
  - What you give up: over-engineered for Phase 4 (padding is out
    of A5 minimal scope); the zero-inset Option A composes
    additively with a future padding attribute without rework.

**Options (offset application primitive):**

- **Option A — `Visual.Offset` on the ScrollView-owned
  intermediate content Visual (recommended).** Mutation =
  `SetOffset(0, -offset_y, 0)`
  (negative because moving content up exposes lower content
  through the viewport).
  - What you gain: matches the existing M2 visual-layer
    convention (LayoutNode offsets → parent-relative
    `Visual.Offset` per
    [architecture.md §6.5](../architecture.md#65-widgetnode-and-visual-layer-sync));
    no new Composition primitive introduced; `i32` pixel offset
    + no animation makes the simpler primitive sufficient.
  - What you give up: fractional offsets and Composition-driven
    animation become awkward (would require switching to
    `TransformMatrix`) — neither is Phase 4 territory.
- Option B — `Visual.TransformMatrix` on the content child
  Visual. Mutation = `SetTransformMatrix(Matrix4x4.CreateTranslation(0,
  -offset_y, 0))`.
  - What you gain: forward-compatible with fractional offsets and
    Composition animation; M4's smooth-scroll work would land
    here naturally.
  - What you give up: heavier primitive for the Phase 4 use
    case; introduces a divergence from the
    `LayoutNode.offset → Visual.Offset` convention §6.5
    establishes (TransformMatrix is an alternative, not a
    parallel); M4 can switch when momentum / smooth-scroll
    actually pressures it without breaking the Phase 4 surface.

**Options (Visual tree shape):**

- **Option A — Outer (clipped) + inner (offset) (recommended).**
  ScrollView's own Visual carries the clip; an intermediate
  content Visual is its child and carries the scroll position
  via `Visual.Offset`. Visual tree:
  ```
  ScrollView Visual (Size = viewport, Clip = InsetClip{0,0,0,0})
    └── content Visual (Offset = (0, -offset_y, 0))
          └── … widget tree (Box thumbnails / WrapPanel / etc.)
  ```
  The inner content Visual is **ScrollView-owned intermediate
  infrastructure**, not the single child widget's own Visual.
  ScrollView attaches its single content child's widget Visual
  beneath this intermediate Visual rather than directly under
  the outer clipped Visual, so the scroll translation
  (`Visual.Offset = (0, -offset_y, 0)` on the intermediate
  Visual) stays separated from the child widget's own
  layout-derived `Visual.Offset` written by `sync_visuals` per
  [architecture.md §6.5](../architecture.md#65-widgetnode-and-visual-layer-sync).
  This is a localised extension of the existing
  "1 WidgetNode = 1 Visual" convention §6.5 establishes —
  ScrollView is the first WidgetNode to own a second
  intermediate Visual beneath its outer one; the §6.5 paragraph
  added in Moment 1 (per §Upstream document revisions) records
  the convention extension.
  - What you gain: clean separation of "viewport" (outer) from
    "scrollable canvas" (inner); the clip naturally clips the
    translated content; the child widget's layout offset and
    ScrollView's scroll offset do not conflate onto a single
    Visual; verified compatible with §6.5's parent-relative
    offset convention.
  - What you give up: nothing relative to the natural Composition
    tree shape.

**Decision:** Option A (`Visual.Clip = InsetClip{0,0,0,0}`) +
Option A (`Visual.Offset` on the ScrollView-owned intermediate
content Visual) + Option A (outer-clipped / inner-offset tree
shape). All three match the existing M2 visual-layer offset
convention, use the simplest clip primitive for the required
viewport clip, and avoid introducing a TransformMatrix-based
offset path.

**R2 (Phase 3 carry-over) — close inside Phase 4.** Phase 3 T9
surfaced a `sync_visuals` bug whose root cause was the implicit
absolute-vs-parent-relative offset convention. The architecture
fix landed in
[architecture.md §6.5](../architecture.md#65-widgetnode-and-visual-layer-sync);
the test-coverage half was filed open as R2 (per
[Phase 3 ADR Out-of-phase residuals](./m3-phase-3-wrap-panel.md)).
Per Phase 4 framing decision F, R2 closes inside Phase 4 via the
Windows integration test's three-level offset assertion
(ScrollView Visual at parent offset X, ScrollView-owned
intermediate content Visual at offset (0, -offset_y) relative to
ScrollView, Box thumbnails inside content at their own offsets —
the three-level nesting Phase 3 lacked). See §Phase 4 verification
closure item 4 below.

**Layering with DD-003 / DD-005.** DD-003 supplies the `offset_y`
value (read-only-bound `i32` per recommendation, clamped per
DD-005); DD-004 applies it via `Visual.Offset` on the
ScrollView-owned intermediate content Visual; DD-005's arrange
pass re-computes the clamp on every layout pass.
