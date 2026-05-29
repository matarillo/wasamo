# M3-Phase 5 — Grid layout primitive: Architecture Decisions

**Phase:** M3-Phase 5 (Grid layout primitive)
**Date:** 2026-05-28
**Status:** Accepted

## Context

M3 acceptance criterion **A2** (see
[../../../_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
[../../plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)):

> Grid layout primitive (1 cell 1 child, star sizing + spanning;
> same-cell overlap is **not** provided — overlay is ZStack's
> responsibility).

The three load-bearing parts are **track definition**, **star
sizing**, and **row / column spanning**. "1 cell 1 child" is a
Phase 5 scope boundary: a Grid child occupies one resolved cell
rectangle or one spanning rectangle; deliberate overlap in the
same resolved cell belongs to Phase 6 ZStack, not to Grid.

Two further milestone obligations apply to Phase 5:

- **A11 (operational obligation).** `.ui`, `wasamo-ir`, `wasamoc`,
  `wasamo-runtime`, `docs/dsl_spec.md`, and the `examples/gallery/`
  sub-screen all advance within Phase 5. The visible gallery proof
  must show Grid in the actual `.ui -> IR -> runtime` path.
- **A12 (DSL specification first public draft obligation).** The
  Phase 5 Grid chapter is part of the per-phase drafting path
  toward the M3 public `docs/dsl_spec.md` draft. Phase 5 adds new
  [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) §4.12 (Grid
  layout primitive) at Moment 1 and holds it to the external-reader
  bar at close.

The pre-doc framing for this phase was aligned with the owner on
2026-05-28 and is recorded in
[../requirements/framing.md](../requirements/framing.md) (`Owner
alignment outcome` section). That alignment settled the
structurally branching decisions carried below; the remaining
Surface-A2 sub-decisions are recorded as ADR `Recommendation`
directions and approved at the `Status: Proposed` → `Accepted`
review pass.

Per the M3-Phase 2 framing decision D postmortem
([../../phase-2/requirements/framing.md](../../phase-2/requirements/framing.md))
and Phase 3 / Phase 4 same-shape inheritance, the
"Moment is not a commit unit" rule applies: each upstream-document
edit in a Moment lands as its own commit on the pre-doc branch,
scoped by review concern per
[../../../../CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules).

The M2 / M3-Phase-1..4 end-state shape that this phase extends
without breaking:

- `wasamo-ir`: `IrType` is `I32 | Str | Bool`; `IrLiteral` is `Int
  | Str | Ident | Bool | Ratio | Color`. Phase 5 introduces **no
  new `IrType` and no new `IrLiteral` variant**. Grid's track-list
  values are a Grid-specific domain type (`TrackSize` per
  DD-M3-P5-002) carried in a **Grid-specific kind payload on
  `IrNode`** (carrier **c1** per DD-M3-P5-001) — not in
  `IrProp.value`, which stays strictly `IrLiteral`. The track-list
  values are populated from existing `IntLit` plus a star-token
  shape parsed inside Grid's narrow attribute path (DD-M3-P5-002).
  Placement metadata (`row`, `column`, `row-span`, `column-span`)
  and alignment (`h-align`, `v-align`) on `Cell` continue to use
  the existing `IrProp` machinery (`i32` / ident literals) per
  DD-M3-P5-001 / DD-M3-P5-003 / DD-M3-P5-005.
- `wasamo-runtime` widget catalog: `Rectangle | VStack | HStack |
  Text | Button | Box | WrapPanel | ScrollView` (Phase 4 added
  `ScrollView`). Phase 5 adds **`Grid`** as a per-kind tag in
  the runtime widget catalog (DD-M3-P5-001). `Cell` appears as
  an IR node kind (`widget_type: "Cell"`) for parser / IR-loader
  purposes but is **not** registered as a runtime widget kind;
  Grid's lowering consumes Cell IR subtrees directly to extract
  placement / span / alignment metadata and arranges each Cell's
  single content child as the Grid's effective layout child.
  Consequence: the WidgetNode / Visual tree contains one node
  for Grid and one node per Cell's content widget; Cell itself
  does not materialise as a WidgetNode or Visual (1 WidgetNode
  = 1 Visual convention preserved per DD-M3-P5-005). `Cell`
  outside a `Grid` parent is rejected at `wasamoc check` and
  runtime `validate()` (DD-M3-P5-006).
- Layout engine: pure-data `LayoutNode` / `measure` / `arrange`
  boundary, Win32/WinRT-free. Phase 2 introduced
  `LayoutError::{BoxAspectUnboundedBoth, BoxNoExtent}`; Phase 4
  added `LayoutError::ScrollViewUnboundedAxis`. Phase 5 extends
  this with **`LayoutError::GridUnboundedStarAxis`** per
  DD-M3-P5-004 (Grid-specific unbounded-star error; Flutter-style
  precedent consistent with the Phase 4 ScrollView unbounded-axis
  rule).
- Binding pipeline: per-type writer seam pattern (DD-M3-P1-007).
  Phase 5 Grid attributes are **constant-only** in the recommended
  scope; `columns:` / `rows:` track lists and `Cell` placement
  metadata do not admit bindings in Phase 5. F5 (`TypedValue`
  deferral) is held in force by construction. The chosen surface
  does not foreclose future bindable track pieces (see DD-M3-P5-002
  forward-compat exposure).
- `wasamoc`: state-name → declared-type table; identifier
  resolution lowers to typed `*PropRead` variants. Phase 5 adds no
  new value type; `wasamoc check` extends to Grid's surface (track
  lists, Cell membership, span ranges) per DD-M3-P5-006.
- Composition / Visual Layer: the **1 WidgetNode = 1 Visual**
  convention holds for Grid (no intermediate Visual, unlike Phase 4
  ScrollView which locally extended the convention). Grid is a
  pure layout container like WrapPanel; the outer-bounds clip lands
  on Grid's own Visual via `Visual.Clip = InsetClip{0,0,0,0}` per
  DD-M3-P5-005, without introducing a separate clip-owner Visual.
  Document-order paint order is the existing convention; Phase 5
  records it as a normative rule for the Grid chapter (DD-M3-P5-005
  z-order rule) but does not change `sync_visuals()`.

This ADR is framed against A2 and the milestone plan's "star sizing
+ spanning; same-cell overlap is **not** provided" phrasing
([../../plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)).
Phase 5 is the **second novel-normative-spec phase** of M3 (Phase 3
WrapPanel was the first): Grid's star sizing is the heavier
algorithmic content of M3, and the conservative `auto` deferral
recorded in DD-M3-P5-002 is a compatibility choice (avoiding
half-specified `auto` / spanning interactions that would constrain
v1.0) rather than an effort-minimisation choice.

It does **not** re-open F5 (`TypedValue` deferral) — Grid attributes
ship as constant-only literals; no `f64` / ratio shape, no new
value type. Image-widget deferral remains in force; the Phase 5
gallery slice content is Box + Text per Phase 2 DD-M3-P2-006
placeholder pattern.

The acceptance lens for this phase: A2 is satisfied when (i) `.ui`
declares `Grid { columns: <track-list>; rows: <track-list>; Cell {
row: <i32>; column: <i32>; [row-span: <i32>;] [column-span: <i32>;]
[h-align: ...] [v-align: ...] <single content child> } ... }` and
the shared crates lower → load → render it with correct fixed /
weighted-star track resolution, both-axis spanning, "1 cell 1
child" conflict rejection, and the Grid outer-bounds clip; (ii) the
new Grid chapter lands in [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md)
§4.12 as a normative spec at the milestone-end-criteria bar
([../../plan.md §Milestone-end criteria item 5](../../plan.md#milestone-end-criteria))
applied at phase close; and (iii) `examples/gallery/gallery.ui` is
grown additively with a Grid-backed slice (the Phase 3 standalone
WrapPanel slice and Phase 4 ScrollView slice stay untouched). Per
A11, all sides advance together by phase close.

## Decisions

The Phase 5 ADR carries six DDs:

| DD | Title | Recommendation summary |
|---|---|---|
| [DD-M3-P5-001](./dd-m3-p5-001-grid-ir-node-form-and-surface-a2.md) | Grid IR node form and Surface A2 author surface | Per-kind tag; **Surface A2** (track-list + placed `Cell` wrapper); `Cell` as single-child layout wrapper; zero-based indices at `.ui` boundary |
| [DD-M3-P5-002](./dd-m3-p5-002-track-sizing-forms-fixed-and-weighted-star.md) | Track sizing forms (fixed + weighted star) | Fixed integer pixels + weighted star (positive integer weights); **`auto` deferred** with reserved algorithm slot |
| [DD-M3-P5-003](./dd-m3-p5-003-child-membership-spanning-and-conflict.md) | Child membership, spanning, and conflict policy | `Cell` membership via explicit `row` / `column`; **both axes admitted** (column-span + row-span), default `1`; same-cell / overlapping-rectangle conflicts rejected |
| [DD-M3-P5-004](./dd-m3-p5-004-track-resolution-algorithm.md) | Track-resolution algorithm | Fixed-first then weighted-star distribution; `f32` prefix boundaries (no integer pixel snap); **`LayoutError::GridUnboundedStarAxis`** when star tracks meet an unbounded parent axis |
| [DD-M3-P5-005](./dd-m3-p5-005-arrange-overflow-and-z-order.md) | Arrange algorithm, overflow, and z-order | Stretch default + per-`Cell` `h-align` / `v-align`; **Grid outer-bounds clip** on; paint overflow between cells allowed; per-cell clip out of scope; **document-order z-order** (no explicit `z-index`); no intermediate Visual |
| [DD-M3-P5-006](./dd-m3-p5-006-ir-loader-defense-in-depth-invariants.md) | IR-loader defense-in-depth invariants | Min row/col count, surface lowering, track sizes / star weights, placement-in-range, span-in-range, and conflict rejection are dual-gated at `wasamoc check` and runtime `validate()`; offset-style runtime clamp does not apply (placement / span violations are reject-at-validate, not clamp-at-arrange) |

## Phase 5 verification closure (what counts as A2 evidence)

This section is not a DD — it records the agreed shape of the proof
that closes Phase 5 per framing decision FD-C, so the
implementation plan inherits a concrete target.

A2 (Grid layout primitive — 1 cell 1 child, star sizing + spanning)
has two evidence layers:

- **Automated / CI-gated A2 evidence:** items (1)–(4).
- **Phase-close / A11 gallery proof:** item (5), including the
  owner-manual visible smoke for the grown gallery sub-screen.

Phase 5 closes only when **all five** of the following are
observed:

1. **`wasamoc check` compile-time evidence (host-independent).**
   Pure-logic tests in `wasamoc`'s check / lower path cover:
   - **Surface lowering positive controls** — the gallery Grid
     slice's `.ui` (item 5 below) compiles cleanly; representative
     fixed + weighted-star + spanning fixtures in unit tests also
     compile cleanly. Lowers to Grid's `TrackSize` sequences plus
     logical Cell membership per DD-M3-P5-001 / DD-M3-P5-002 /
     DD-M3-P5-003.
   - **Track-list shape rejection** — `columns:` / `rows:` with
     non-integer fixed values, with `0*` / negative-weight star
     tokens, with star weights `> 1024` (per-weight upper bound
     per DD-M3-P5-002), with the deferred `auto` token, and
     with empty track lists each surface a `wasamoc check`
     diagnostic naming the offending shape (per DD-M3-P5-002).
   - **Placement / span value rejection** — `Cell` with non-`i32`
     `row` / `column`, with out-of-range `row` / `column`, with
     `row-span` / `column-span` `<= 0`, with span values that
     overflow the declared track count, and with malformed
     defaults each surface a `wasamoc check` diagnostic (per
     DD-M3-P5-003 / DD-M3-P5-006).
   - **Cell single-child rejection** — `Cell { }` (0 content
     children) and `Cell { Text {} Text {} }` (>1 content
     children) each surface a `wasamoc check` diagnostic (per
     DD-M3-P5-001 / DD-M3-P5-006).
   - **Same-cell / span conflict rejection** — two `Cell`s with
     equal `(row, column)` or with overlapping
     `(row, column, row-span, column-span)` rectangles surface a
     `wasamoc check` diagnostic (per DD-M3-P5-003 / DD-M3-P5-006).
   - **Unknown Grid / Cell attribute rejection** — attributes
     outside the documented surface (e.g. `gap`, `auto-flow`,
     `z-index`, `clip`) are rejected on Grid; attributes outside
     `row` / `column` / `row-span` / `column-span` / `h-align` /
     `v-align` are rejected on Cell.

   These run on any CI runner; the diagnostics are pure-logic in
   `wasamoc`.

2. **Measure-arrange unit-test evidence (host-independent).**
   Pure-logic tests against the layout engine's Grid measure-arrange
   (`wasamo-runtime/src/layout.rs` extension) cover:
   - **Fixed-only tracks** — 1, 2, and 3-track configurations
     resolve to the declared widths; remaining bounded space is
     not redistributed.
   - **Weighted-star tracks** — single `*`, multiple `*`, mixed
     unit-and-weighted (`1* 2* 1*`), and all-equal weights divide
     remaining bounded space proportionally. `f32` prefix
     boundaries are deterministic.
   - **Mixed fixed + star tracks** — fixed tracks consume definite
     space first; remaining bounded space divides among star tracks
     by weight.
   - **Both-axis spanning** — column-span and row-span resolve to
     the combined rectangle of the spanned tracks. A span that
     equals the full row / column is allowed; a span that exceeds
     the declared track count is rejected at validate time, not
     clamped (per DD-M3-P5-006).
   - **Negative remaining space** — when fixed tracks alone exceed
     the bounded space, star tracks resolve to zero width; the
     paint-overflow rule in DD-M3-P5-005 then applies.
   - **Unbounded star-axis parent** — fires
     `LayoutError::GridUnboundedStarAxis` (reject test; pins the
     DD-M3-P5-004 / DD-M3-P5-002 branch). All-zero star sum (every
     star weight is zero) cannot arise after DD-M3-P5-002's
     validate-time rejection, but the corresponding layout-time
     assertion is left as a defensive panic per DD-M3-P5-004.
   - **Per-Cell alignment** — `h-align` / `v-align` defaults are
     stretch; explicit `start` / `center` / `end` overrides anchor
     the content within the resolved cell rectangle (per
     DD-M3-P5-005).
   - **Grid outer-bounds clip presence** — a Cell whose content
     overflows the cell rectangle paints past the cell boundary but
     not past the Grid's outer rectangle (the clip is asserted by
     item 4 under the real Visual tree; in unit tests this is
     covered as a layout-side outer-rect invariant).
   - **Document-order z-order** — overlapping painted content (e.g.
     a spanning header Cell followed by a non-spanning sibling that
     happens to overlap due to overflow) has later children on top.

   These run on any CI runner; the measure-arrange algorithm is a
   pure function (input → output) per framing decision FD-C.

3. **IR-loader / `validate()` invariant evidence (host-independent).**
   Pure-logic tests in `wasamo-runtime`'s `ir_loader::validate()`
   path cover (DD-M3-P5-006 structural / value-range gates,
   runtime half):
   - **Min row / column count** — Grid with empty `columns:` or
     empty `rows:` surfaces `WASAMO_ERR_IR_MALFORMED`.
   - **Track value range** — fixed pixel values `<= 0`, star
     weights `<= 0`, and star weights `> 1024` (per-weight upper
     bound per DD-M3-P5-002 / DD-M3-P5-006) surface
     `WASAMO_ERR_IR_MALFORMED` (all-zero star sum is a special
     case of the same rule and cannot arise once each individual
     weight is `>= 1`).
   - **Placement value range** — `Cell` with `row < 0`,
     `column < 0`, `row >= rows.len()`, or
     `column >= columns.len()` surfaces `WASAMO_ERR_IR_MALFORMED`.
   - **Span value range** — `Cell` with `row-span <= 0`,
     `column-span <= 0`, `row + row-span > rows.len()`, or
     `column + column-span > columns.len()` surfaces
     `WASAMO_ERR_IR_MALFORMED`.
   - **Same-cell / overlapping-span conflicts** — two `Cell`s
     occupying the same `(row, column)` rectangle in any way
     surface `WASAMO_ERR_IR_MALFORMED`.
   - **Cell child-count** — `Cell` with 0 or >1 content children
     surfaces `WASAMO_ERR_IR_MALFORMED` (Phase 2 T7 / Phase 4
     DD-M3-P4-006 child-count discipline applied to Cell).

4. **Windows-runtime layout evidence (CI-gated, including production-root
   shape).** Mock-free integration tests (per
   [../../../../CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules))
   on the Windows CI runner exercise:

   - **Grid-rooted fixture (one of the two required parent shapes
     per Phase 4 T6 carry-forward).** A `.ui` declares a Grid as
     the root widget with mixed fixed and weighted-star tracks
     containing `Cell { Box { ... } }` children in known cells. The
     test loads the IR, runs the layout pass, and asserts:
     - (a) the Grid's resolved rectangle matches the parent
       allocation (the window-root Fill/Fill from Phase 4 T6's
       `WidgetNode::run_layout_as_window_root` path);
     - (b) each Cell's content Visual offset matches the resolved
       cell rectangle origin (with parent-relative offsets per
       [../../../../docs/architecture.md §6.5](../../../../docs/architecture.md#65-widgetnode-and-visual-layer-sync));
     - (c) the Grid's outer Visual has `Visual.Clip` set to a
       non-null clip (the InsetClip from DD-M3-P5-005) — clip
       **presence** assertion;
     - (d) each Cell content Visual has `Visual.Clip = null` —
       clip **absence** regression guard, symmetric with the
       Phase 3 T8 WrapPanel and Phase 4 ScrollView precedents.
   - **`VStack { Grid { ... } }` fixture (production root shape;
     second of the two required parent shapes).** A `.ui` places a
     Grid inside a VStack root, matching the current gallery /
     counter / bool-demo `.ui` production root family. The test
     asserts the same set of conditions (a)–(d) against the inner
     Grid. This fixture guards against the Phase 4 T6 runtime-
     boundary collapse class.
   - **Unbounded star-axis runtime fixture.** A `.ui` declares a
     Grid with at least one star track inside a parent whose
     corresponding axis is unbounded (synthesisable by embedding in
     an intrinsic-measure context). The test asserts the layout
     pass returns `Err(LayoutError::GridUnboundedStarAxis)`. If no
     ergonomic way to synthesise this fixture exists at the IR
     level, the unbounded-parent case may be exercised purely in
     unit tests (item 2) and this fixture downgraded to pure-logic;
     the integration-test version is preferred when feasible.

   All fixtures fail (not skip) on a runner that cannot create the
   Compositor — the test gates A2 evidence in CI, not local
   convenience. Skip-guard inherits the Phase 2 T11 / Phase 3 /
   Phase 4 pattern verbatim (fires on `0x80070005` from
   `wasamo_init`).

5. **End-to-end host evidence (visible smoke).**
   `examples/gallery/gallery.ui` is **grown additively** by adding
   a sibling slice containing a Grid composition with at least one
   header Cell spanning all columns, three middle-row Cells in
   separate columns, and one footer Cell spanning all columns (FD-H
   minimum visible-proof shape: a 3-row × 3-column Grid with at
   least five Cells, exercising fixed tracks, at least one star
   track, child membership, and column-span). `examples/gallery-rust/`
   builds and runs the grown sub-screen. `Start-Process` launch is
   recorded as successful by the assistant; **visual correctness**
   (column tracks render at the declared widths; spanning header
   spans all three columns; middle-row Cells occupy separate
   columns; Grid outer-bounds clip is visible when a Cell's content
   intentionally overflows; document-order paint order is observed
   when overlapping content occurs) is **owner-manual GUI smoke**
   per framing decision FD-I — the assistant does not assert on
   pixel- or eyeball-level correctness.

   The gallery slice exercises **column-span** in the real `.ui`;
   **row-span** is discharged by items (2)–(4) (pure layout +
   `wasamoc check` + runtime `validate()`), not by the visible
   proof. This split is intentional per FD-C: row-span has no
   surface-restructuring effect under Surface A2, so the algorithmic
   / validation evidence is the right discharge.

Items (1)–(4) are the automated A2 evidence set. Item (5) is
required for Phase 5 close under A11: it ties the evidence back to
the milestone-plan target-app trajectory and grows the gallery sub-
screen Phase 2 / 3 / 4 seeded with the canonical static 2D
composition Phase 6 will later overlay with ZStack.

Per the framing FD-C "evidence lines do not collapse" rule, items
(1)–(4) do not merge into one even if they share helper
infrastructure — `wasamoc check` diagnostics, measure-arrange unit
tests, `validate()` invariant tests, and Windows-runtime integration
tests each have distinct evidence meanings, and the Phase 4 T6
runtime-boundary lesson is the reason items (2) and (4) both exist.

The acceptance / non-acceptance of test items (1)–(5) is the
operational form of "Phase 5 done"; the corresponding implementation
checklist (which crate / which test file / which fixture) belongs
in the Phase 5 implementation plan, not here.

## Post-Phase-5 hand-off

Per the framing's `auto` deferral and Surface A2 forward-compat
exposure, the following surfaces are explicitly **anticipated for a
future Grid-related phase or post-M3 work** and are documented here
so later input / iteration / theming work has a named landing
point:

1. **`auto` / intrinsic track sizing.** Tracks whose size is
   determined by content demand. DD-M3-P5-002 reserves the
   algorithm slot before star distribution in DD-M3-P5-004;
   admitting `auto` requires defining how spanning children
   distribute demand across multiple auto tracks. Future
   admission is a **vocabulary extension on the `TrackSize`
   domain type** (a new `Auto` variant) — Phase 5's Grid /
   Cell structure, authoring surface, and `IrProp` machinery
   stay unchanged; only the `TrackSize` enum grows and
   DD-M3-P5-004's reserved demand pass becomes live.

2. **Named lines and template-area surfaces.** CSS Grid-style named
   track lines and `grid-template-areas`-style 2D shorthand are
   explicitly out of Phase 5 scope. The Surface A2 placement
   metadata on `Cell` does not foreclose a future area-name
   attribute (e.g. `area: header`); such an attribute would be
   additive and would lower to the same `(row, column, row-span,
   column-span)` rectangle.

3. **Bindable track / placement attributes.** Phase 5 Grid
   attributes are constant-only. A later phase may add bindable
   track-list values (`columns: {sidebar_width} 1*`) or bindable
   Cell placement (`Cell { row: {focused_row} ... }`). Surface A2
   does not foreclose this — the binding-effect machinery already
   exists per DD-M3-P1-007 / Phase 4 DD-M3-P4-003; the gating is
   F5 (`TypedValue` deferral) and Phase 5 scope.

4. **Iteration template generating `Cell`s (post-M3).** Per the
   accepted target-app pre-doc
   ([../../requirements/spec.md](../../requirements/spec.md)),
   Grid is **not** an M3 iteration target — the iteration grammar's
   M3 target is the WrapPanel-backed thumbnail collection. A
   future, post-M3 milestone may admit `for item in items { Cell {
   row: ... column: ... <content> } }`. Surface A2 makes this
   structurally possible (every `Cell` is explicit), so the
   iteration foreclosure check from framing axis 4 holds.

5. **Drag-resizable splitter / column drag.** Pointer-driven track
   resize is an M4+ input-handling concern; no Phase 5 surface
   forecloses it.

6. **Per-cell clipping and an author-facing `clip:` surface.**
   Phase 5 ships only the Grid outer-bounds clip. A later phase
   may admit per-cell clipping (e.g. `Cell { clip: true ... }`) if
   author demand warrants it; the Grid outer-bounds clip remains
   independent.

None of items 1–6 require modifying Phase 5's IR shape, Cell
contract, default behaviour, or measure-arrange algorithm. All six
are additive on top of the Phase 5 surface.

## Out of scope

The following are not included in Phase 5 and are not deferred by
oversight — each is explicitly out of A2 scope or deferred to a
later phase / milestone (consolidated from
[../requirements/framing.md §Phase 5 scope](../requirements/framing.md#phase-5-scope)):

- **Same-cell overlap / overlay** — A2 explicit boundary; Phase 6
  ZStack owns overlay.
- **`auto` / intrinsic track sizing** — DD-M3-P5-002 defers with
  reserved algorithm slot (Post-Phase-5 hand-off item 1).
- **Per-cell clipping and any author-facing `clip:` attribute** —
  DD-M3-P5-005 ships only Grid outer-bounds clip (Post-Phase-5
  hand-off item 6).
- **Explicit `z-index` / author-facing paint-order control** —
  DD-M3-P5-005 fixes paint order to document order; Phase 5 exposes
  no layering attribute. Intentional overlay is Phase 6 ZStack.
- **Responsive breakpoint grammar, media queries, named areas, and
  `grid-template-areas`-style shorthand** — Post-Phase-5 hand-off
  item 2.
- **Bindable Grid track definitions or bindable Cell placement** —
  Post-Phase-5 hand-off item 3. Phase 5 Grid attributes are
  constant-only.
- **Iteration-generated `Cell`s** — Grid is not an M3 iteration
  target ([../../requirements/spec.md](../../requirements/spec.md));
  Post-Phase-5 hand-off item 4.
- **Drag-resizable columns / rows, splitters, pointer-driven
  layout resize** — Post-Phase-5 hand-off item 5.
- **Scrollbar widget, wheel handler, drag-to-scroll, and `scroll_y`
  Signal write-back** — Phase 4 M4 hand-off items; not a Phase 5
  Grid topic.
- **R1 — Gallery host Window title wiring** — Phase 4 carry-over
  residual; Phase 5 framing decision FD-E assigns R1 to **M3-Phase
  6** (ZStack + conditional rendering). Phase 5 thesis scope **out**;
  the plan-note edit lands as part of the Moment 1 commit set.
- **Phase 4 `scroll_y` Signal drift** — M4 handoff; not a Phase 5
  Grid topic.
- **`TypedValue` generic value union** (F5 maintained — Phase 5
  introduces no new scalar type).
- **Non-root Shrink container × Fill child Grid-specific
  exception** — FD-D: Phase 5 keeps the existing convention
  (`degenerate_fill_in_shrink_parent_clamps_to_zero`). Grid does
  not pierce the convention. The orthogonal observability concern
  (detecting the silent collapse during debugging) is carved out
  to [../../../../docs/notes/developer-debugging.md](../../../../docs/notes/developer-debugging.md)
  and out of Phase 5 thesis scope.
- **Window chrome / theming** — out of Phase 5 unless explicitly
  redirected.
- **Image widget as Cell content** — Image deferred to M4 or later
  per Phase 2 DD-M3-P2-006; Phase 5 Cell content is Box + Text /
  Button placeholders.
- **Background `fill` on Grid** — Phase 5 does not introduce a
  Grid-level `fill` attribute; the visible background is whatever
  parent / sibling provides.
- **Nested Grids** — structurally permitted (nothing in the IR or
  layout forbids it), but Phase 5 ships no test fixture or sub-
  screen exercising the case. Unbounded-parent runtime error from
  DD-M3-P5-004 covers the pathological inner Grid whose parent is
  itself an unbounded-axis Grid.

## Upstream document revisions (Moment 1 / Moment 2)

Phase 5 follows the two-moment structure inherited from Phase 2 / 3
/ 4 framing decision D and recorded in framing decision FD-G. Doc
set and commit shape follow the per-review-concern rule in
[../../../../CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules)
and [../../../procedures/retrospectives.md](../../../procedures/retrospectives.md).
The Phase 5 `dsl_spec.md` section marker mirrors the Phase 2 / 3 /
4 form:

```
**Phase status:** M3-Phase 5 design accepted; implementation pending
```

flipping at phase close to:

```
**Phase status:** M3-Phase 5 closed; implementation-synced
```

placed as the first line under the Grid chapter heading (new §4.12
alongside §4.9 Box, §4.10 WrapPanel, §4.11 ScrollView).

**Moment 1 — ADR Accepted commit set (design-spec draft).**
Constituent commits, each landing as its own commit on the pre-doc
branch per the per-review-concern rule. The draft-side doc set
Phase 5 commits to at Moment 1 is enumerated below; the Moment 2
phase-sync doc set is a related but distinct rule and not the
mirror of this list:

- `process/milestone-3/phase-5/decisions/preamble.md` and
  `dd-m3-p5-*.md` (this directory) — ADR `Status: Accepted` flip.
- [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) — new §4.12
  Grid chapter as design-spec draft (DD-M3-P5-001 through
  DD-M3-P5-006 sub-issues as the chapter outline; the Grid mental-
  model anchor + ecosystem-contrast subsection per framing
  decision FD-K). Plus a **§4.4 widget registry row for `Grid`
  only** (Grid is the runtime widget kind). **`Cell` is defined
  in §4.12 as a Grid-specific child wrapper construct, not a
  §4.4 registry entry** — Cell is an IR node kind consumed by
  Grid's lowering (per DD-M3-P5-001) and is rejected outside a
  `Grid` parent (per DD-M3-P5-006); listing it in §4.4 would
  imply free-standing widget status the DDs explicitly reject.
  A short pointer from §4.4 to §4.12 names Cell as the Grid
  child wrapper so external readers know where to find it.
  Section marker: `Phase status: M3-Phase 5 design accepted;
  implementation pending`.
- [`docs/architecture.md`](../../../../docs/architecture.md) — Grid
  entry under the IR section, including the `TrackSize` domain
  type and the **Grid-specific kind payload on `IrNode`** (carrier
  c1 per DD-M3-P5-001; `IrProp.value` stays strictly `IrLiteral`);
  the Surface A2 child-membership representation (`Cell`-wrapper
  with `row` / `column` / `row-span` / `column-span` / `h-align`
  / `v-align` via existing `IrProp` machinery); layout-engine
  paragraph for the Grid track-resolution algorithm (fixed-first
  + weighted-star + unbounded-star error); no §6.5 sync-visuals
  change (Grid uses the existing 1 WidgetNode = 1 Visual
  convention; the outer clip lands on Grid's own Visual without
  an intermediate Visual).
- [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no touch
  expected** per DD-M3-P5-001 / DD-M3-P5-006 (LayoutError stays
  internal; Grid adds no host-facing ABI surface and no
  `PropertyValue` tag).
- [`../../plan.md`](../../plan.md) — Phase 5 row populated
  (Status: in progress; implementation plan link; ADR link).
  Phase 6 row Notes get the "M3-Phase 4 R1 (Window title wiring)
  owning phase" cross-reference (FD-E). This is a plan-note edit,
  not part of the Grid thesis commit.
- `process/milestone-3/phase-5/implementation/preamble.md` /
  `process/milestone-3/phase-5/implementation/plan.md` —
  implementation planning opened after ADR acceptance, with the
  final-step retrospective split (FD-C / FD-G) represented in the
  task plan from the start per
  [../requirements/constraints.md §5](../requirements/constraints.md#5-phase-最終-step-の-retrospective--progress-checklist-は-step-end-と-phase-end-を分割する).

Implementation begins only after these commits land.

**Moment 2 — Phase close commit set (impl re-sync).**

- [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) §4.12 —
  section marker flips to "closed; implementation-synced", plus any
  corrections required if the design draft and implementation
  diverged (marker flip is required regardless of divergence;
  corrections are conditional on what re-sync surfaces). Earlier-
  phase spec gaps surfaced during the re-sync may fold into the
  same commit with explicit owner confirmation (retroactive
  spec-gap minimum-fold pattern inherited from Phase 2 / 3 / 4).
- [`docs/architecture.md`](../../../../docs/architecture.md) — top
  Status flips to `M3-Phase 5 complete`; impl-divergent paragraphs
  re-synced.
- `process/milestone-3/phase-5/implementation/log.md` and
  `process/milestone-3/phase-5/retrospectives/phase-end.md` —
  phase-close retrospective link, CI evidence pointer, impl
  summary; the implementation plan then enters the standard
  `in-progress → completed` lifecycle.
- [`../../plan.md`](../../plan.md) Phase 5 row — Status flips to
  complete.
- This ADR — touch only if one of the three retrospectives.md
  §phase-sync ADR-touch cases applies (AC discharged-vs-impl
  divergence; out-of-phase residual cross-ref; thesis-level
  finding).
- Step retro `phase-sync` items must all close into `doc-folded` /
  `carry-forward` / `local-only` at Moment 2 — no open
  `phase-sync` items survive past phase close.

No ROADMAP revision is anticipated — A2 is already explicit; this
ADR operationalises it.

## Inputs absorbed

Mapping from [../requirements/framing.md](../requirements/framing.md)
framing decisions and aligned outcomes to DDs and ADR sections:

| Source | Disposition | Consumed at |
|---|---|---|
| Owner alignment outcome — DD-M3-P5-001 Surface A2 | Settled branching | DD-M3-P5-001 Recommendation |
| Owner alignment outcome — DD-M3-P5-002 `auto` deferred with reserved slot | Settled branching | DD-M3-P5-002 Recommendation; Post-Phase-5 hand-off item 1 |
| Owner alignment outcome — DD-M3-P5-003 both spanning axes admitted | Settled branching | DD-M3-P5-003 Recommendation |
| Owner alignment outcome — DD-M3-P5-004 Grid-specific unbounded-star error | Settled branching | DD-M3-P5-004 Recommendation; `LayoutError::GridUnboundedStarAxis` |
| Owner alignment outcome — DD-M3-P5-005 paint overflow + Grid outer-bounds clip + document-order z-order | Settled branching | DD-M3-P5-005 Recommendation |
| FD-A — DD slate completeness | Discipline | DD slate (6 DDs); §Out of scope |
| FD-B — Pre-doc spec-drafting discipline | Constraint | §Upstream document revisions (Moment 1 §4.12 design draft) |
| FD-C — Verification strategy | Constraint | §Phase 5 verification closure items 1–5 |
| FD-D — Non-root Shrink × Fill child = status quo | Settled branching | §Out of scope (Grid does not pierce the convention) |
| FD-E — R1 owning phase = M3-Phase 6 | Settled branching | §Out of scope; §Upstream document revisions Moment 1 plan-note edit |
| FD-F — Phase 4 residual scan disposition | Direct input | §Out of scope (R1 → Phase 6; R2 closed in Phase 4) |
| FD-G — Two-moment sync structure | Constraint | §Upstream document revisions (Moment 1 / Moment 2) |
| FD-H — Phase 5 visible proof (gallery slice) | Constraint | §Phase 5 verification closure item 5 |
| FD-I — GUI smoke responsibility separation | Discipline | §Phase 5 verification closure item 5 (owner-manual GUI smoke clause) |
| FD-J — Live-note re-evaluation triggers | Disposition table | (No direct ADR section — per-note disposition feeds DD layering and §Out of scope; live notes themselves are not modified by Phase 5 unless impl re-sync requires it) |
| FD-K — Grid mental-model anchor in dsl_spec | Spec content | DD-M3-P5-005 spec content seed; the mental-model + ecosystem-contrast subsection lands in `dsl_spec.md` §4.12 at Moment 1 |

Mapping from framing DD slate (DD-M3-P5-001..006) to this ADR's DD
numbering: 1:1.

Cross-phase inputs:

| Source | Disposition | Consumed at |
|---|---|---|
| M3-Phase 3 DD-M3-P3-005 (pure-data measure-arrange pattern) | Pattern reuse | DD-M3-P5-004 |
| M3-Phase 3 paint-overflow-not-clipped-by-layout-primitive precedent | Pattern reuse with Grid-specific clarification | DD-M3-P5-005 (Grid outer-bounds clip is additive; occupancy overlap remains invalid) |
| M3-Phase 4 DD-M3-P4-005 (offset clamp / outer-rect-vs-content-rect separation) | Pattern reuse | DD-M3-P5-005 (Grid outer-rect = parent allocation; overflowing content paints past cell but clips at Grid outer) |
| M3-Phase 4 DD-M3-P4-006 (compound IR-loader invariant shape) | Pattern reuse | DD-M3-P5-006 |
| M3-Phase 4 T6 runtime-boundary root-shape lesson | Direct input | §Phase 5 verification closure item 4 (production root shape coverage) |
| M3-Phase 4 ScrollView intermediate Visual pattern | Negative precedent | DD-M3-P5-005 (Grid does **not** introduce an intermediate Visual; outer-bounds clip lands on Grid's own Visual) |
| M3-Phase 4 R1 residual | Carry-forward assignment | §Out of scope (R1 → Phase 6 per FD-E / FD-F) |
| M2 handoff §3 reactive drain residuals | Out of scope in recommended path | §Out of scope (Grid is constant-only; F5 held in force) |
| M2 handoff §4 `TypedValue` deferral | Discipline reminder | §Out of scope; Post-Phase-5 hand-off item 3 |

## Revision history

| Date | Change |
|---|---|
| 2026-05-28 | Status flipped to Accepted. DD-M3-P5-001 through DD-M3-P5-006 owner-accepted after multi-round codex review pass (Grid outer-rect rule across DD-004 / DD-005 / preamble; Cell is IR-only and not in `wasamo-runtime`'s widget catalog or `dsl_spec.md` §4.4; Cell placement-attribute presence lowering rule; star-weight cap `[1, 1024]` as deliberate Phase 5 safety limit closing the per-axis sum at the type level via DD-M3-P5-004's `u64` accumulator; `TrackSize` IR carrier c1 — Grid-specific kind payload on `IrNode`; `auto` future admission framed as `TrackSize` vocabulary extension rather than IR structural change). |
| 2026-05-28 | Initial draft (Status: Proposed). All 6 DDs at Proposed pending owner review pass. Framing-level owner alignment confirmed on 2026-05-28 ([../requirements/framing.md §Owner alignment outcome](../requirements/framing.md#owner-alignment-outcome-2026-05-28)) settles DD-M3-P5-001..005 branching choices and FD-D / FD-E; Surface-A2 sub-decisions remain ADR-review approvals. |
