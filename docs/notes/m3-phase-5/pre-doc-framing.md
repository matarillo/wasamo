# M3-Phase 5 pre-doc framing

**Status:** reviewed; pending owner alignment; input artefact for ADR drafting
**Date:** 2026-05-26
**Targets phase:** M3-Phase 5 (Grid layout primitive)

Per the project's doc-driven workflow established at
[M2-Phase 6 pre-doc framing](../m2-phase-6/m2-phase-6-pre-doc-framing.md)
and continued through
[M3-Phase 2 pre-doc framing](../m3-phase-2/m3-phase-2-pre-doc-framing.md),
[M3-Phase 3 pre-doc framing](../m3-phase-3/pre-doc-framing.md),
and
[M3-Phase 4 pre-doc framing](../m3-phase-4/pre-doc-framing.md),
individual DDs are not negotiated one-by-one in chat. Framing is
aligned first, then the full ADR is drafted in one pass as
`Status: Proposed`, reviewed, and flipped to `Status: Accepted`.
This note records the framing intended for owner alignment before
ADR drafting begins; it remains as an input artefact and is not
promoted into the ADR.

The preceding M3 phases supply several things this framing inherits
rather than re-derives:

- **Two-moment spec-sync structure** (Moment 1 design-spec draft at
  ADR-Accepted commit; Moment 2 implementation re-sync at phase
  close), with section-level `**Phase status:**` markers in the
  affected `docs/dsl_spec.md` chapter. See
  [m3-phase-2 framing decision D](../m3-phase-2/m3-phase-2-pre-doc-framing.md#d-upstream-document-revision-timing-two-sync-moments).
- **Moment-is-not-a-commit-unit rule**, recorded in
  [CLAUDE.md §Commit rules](../../../CLAUDE.md#commit-rules): each
  constituent document lands as its own commit on the pre-doc
  branch, scoped by review concern, not by Moment.
- **No fast-track at step-end or phase-end**: every merge requires
  owner explicit approval. The pre-doc-branch landing of this
  framing's downstream commits (ADR, spec sync, progress doc) is no
  exception.
- **Step-end item 10 routing**: cross-step / cross-phase design
  constraints are classified into one of `doc-folded`,
  `phase-sync`, `carry-forward`, or `local-only`. Phase 5 starts
  with the Phase 4 vocabulary already in force.
- **Final-step retrospective split**: Phase 5's progress file should
  split the final step's step-end retrospective from the
  phase-end retrospective from the start, per
  [predoc-inputs.md §5](./predoc-inputs.md#5-phase-最終-step-の-retrospective--progress-checklist-は-step-end-と-phase-end-を分割する).

---

## Phase 5 acceptance criteria (restated)

- **A2** (see [ROADMAP.md M3](../../../ROADMAP.md#m3-dsl-surface),
  [m3-plan.md §Acceptance criteria](../../plans/m3-plan.md#acceptance-criteria)):

  > Grid layout primitive (1 cell 1 child, star sizing + spanning;
  > same-cell overlap is **not** provided — overlay is ZStack's
  > responsibility).

  The three load-bearing parts are **track definition**, **star
  sizing**, and **row / column spanning**. "1 cell 1 child" is a
  Phase 5 scope boundary: a Grid child occupies one resolved cell
  rectangle or one spanning rectangle; deliberate overlap in the
  same resolved cell belongs to Phase 6 ZStack, not to Grid.

- **A11 (operational obligation).** `.ui`, `wasamo-ir`, `wasamoc`,
  `wasamo-runtime`, `docs/dsl_spec.md`, and the
  `examples/gallery/` sub-screen all advance within Phase 5. Phase
  5 is not allowed to become a pure layout-engine exercise; the
  visible gallery proof must show Grid in the actual `.ui -> IR ->
  runtime` path.

- **A12 (DSL specification first public draft obligation).** The
  Phase 5 Grid chapter is part of the per-phase drafting path toward
  the M3 public `docs/dsl_spec.md` draft. Phase 5 should add new
  `docs/dsl_spec.md` §4.12 (Grid layout primitive) at Moment 1 and
  hold it to the external-reader bar at close: a reader with only
  `docs/dsl_spec.md` should be able to reproduce Grid's track
  declaration surface, placement attributes, star sizing, spanning,
  validation, and overflow semantics against a hypothetical host.

- **Second novel-normative-spec phase.** Phase 3 introduced the
  first novel measure-arrange spec through WrapPanel's line breaker.
  Phase 5 is the second and heavier novel-normative-spec phase:
  Grid's star sizing requires a deterministic track-resolution
  algorithm, including how fixed and star tracks interact with
  spanning children. `auto` / intrinsic tracks are explicitly a
  design pressure point, but the draft recommendation below defers
  them from Phase 5 so star sizing remains the load-bearing content.
  Acceptance for the spec text is
  the [m3-plan.md §Milestone-end criteria item 5](../../plans/m3-plan.md#milestone-end-criteria)
  external-reader bar, applied at phase close.

- **Downstream commitments grounded in Phase 5.** Phase 8's Gallery
  E2E proof needs a 2D composition surface for the target app. Phase
  6 may place a lightbox above the gallery via ZStack, and Phase 7
  may generate thumbnail children through iteration, but neither
  phase should be the first proof that the gallery can express
  a stable 2D layout. Phase 5 must therefore ship a fixed-child
  Grid gallery slice that later phases can grow rather than replace.
  Phase 7's iteration grammar is the first likely consumer that may
  generate Grid children from a template; Phase 5 does not design
  iteration, but it must keep child placement as ordinary child
  attributes so a future iteration template can emit `row` /
  `column` / span metadata without inventing a second placement
  channel.

---

## Layering note (DD-001 ⇄ DD-002 ⇄ DD-003 ⇄ DD-004 ⇄ DD-005)

Grid's DD chain is wider than Phase 4's ScrollView chain because the
algorithm has two independent axes and a spanning phase. The chain is:

- **DD-001 (IR shape and child placement surface).** Settles that
  Grid is a new layout primitive, how rows / columns are declared,
  how each child declares its `row`, `column`, `row-span`, and
  `column-span`, and how malformed placement is rejected.
- **DD-002 (track sizing forms and constants).** Settles which
  track sizing forms Phase 5 admits: fixed integer pixels, star
  weights, and whether an intrinsic / auto track exists in Phase 5.
  Track-list declaration syntax belongs to DD-001; DD-002 consumes
  that syntax and defines what each track token can mean.
- **DD-003 (child membership and same-cell conflict policy).**
  Settles the "1 cell 1 child" rule precisely: duplicate origin
  cells, overlapping spans, default placement, and out-of-range
  indices.
- **DD-004 (track-resolution algorithm).** Consumes DD-001 through
  DD-003 to produce the pure-data measure pass: fixed tracks consume
  definite space, intrinsic tracks consume measured child demand if
  admitted, star tracks divide remaining space by weight, and
  spanning children are reconciled with per-track sizes.
- **DD-005 (arrange algorithm and visual composition).** Places each
  child into its resolved cell or span rectangle and preserves the
  existing one-WidgetNode / one-Visual convention unless a concrete
  Grid-specific Visual need appears.
- **DD-006 (IR-loader defense-in-depth invariants).** Settles which
  malformed Grid states are rejected by `wasamoc check` and by
  runtime `validate()` for memory IR.

The chain is **axis-parallel but algorithmically centralised**:
DD-001 / DD-002 define the coordinate system and track inputs;
DD-003 defines legal child occupancy; DD-004 is the load-bearing
algorithmic DD; DD-005 arranges against DD-004's resolved tracks.

Concrete consequences for the ADR's Options tables:

- DD-003 must not recommend same-cell overlap as Grid behavior; that
  contradicts A2 and steals Phase 6's ZStack responsibility.
- DD-004 must not leave star sizing as an implementation detail.
  Star distribution is the central normative content of Phase 5.
- DD-004 must define an unbounded-parent branch. Grid will be used in
  intrinsic contexts; leaving both axes as "whatever runtime does"
  would repeat the class of fixture gap caught during Phase 4.
- DD-005 should default to the existing Visual tree convention. Grid
  does not need a ScrollView-style intermediate Visual unless a
  selected DD creates a specific translation / clip / transform owner.

Invalid DD combinations that the ADR Options tables must reject or
mark non-recommended:

- **DD-002 = no weighted star surface** with **DD-004 = weighted-star
  distribution algorithm**. The algorithm would describe a surface
  authors cannot express.
- **DD-002 = `auto` deferred** with **DD-004 = auto-track demand
  distribution as normative Phase 5 behavior**. If `auto` is not in
  the surface, the algorithm may reserve future terminology but must
  not depend on auto tracks.
- **DD-002 = string-encoded track list** with **DD-006 = parser-level
  track-list diagnostics required**. The DSL parser sees a string
  literal; track-list syntax errors are checked by `wasamoc check`
  and runtime validation after the string is parsed into Grid's
  domain type.
- **DD-003 = auto-placement admitted** with **DD-004 = no
  document-order placement algorithm**. Auto-placement is not just a
  default; it is a layout policy. The current recommendation avoids
  this combination by not admitting auto-placement in Phase 5.
- **DD-003 = same-cell overlap allowed** with **A2 / DD-005 = Grid
  provides no overlay semantics**. Painting overflow may cross a cell
  boundary (DD-005), but deliberate occupancy overlap is rejected.
- **DD-003 = spans may exceed declared track count** with **DD-006 =
  validation-only defense**. Out-of-range spans must be invalid at
  both `wasamoc check` and runtime `validate()`; layout must not
  clamp them silently.
- **DD-004 = star division permits zero or negative weights** with
  **DD-006 = no positive-weight validation**. Star weights are
  positive integers; `0*`, negative weights, and all-zero star sums
  are malformed rather than clamped around divide-by-zero.

---

## Agreed DD slate (6 entries proposed)

The Phase 5 ADR (working title
`docs/decisions/m3-phase-5-grid.md`) will carry the following six DDs.

### DD-M3-P5-001 — Grid IR node form and row / column declaration surface

Grid is a new layout primitive in `wasamo-ir` and `wasamo-runtime`.
Phase 5 must commit to the IR node shape and the author-facing syntax
for declaring row and column tracks.

Sub-issues:

- **IR node shape.** Per-kind tag parallel to `Box`, `WrapPanel`,
  and `ScrollView`, vs a structural variant in `IrLayout`. Default:
  per-kind tag, continuing the M3 primitive pattern. Unlike Box /
  WrapPanel / ScrollView, Grid introduces variable-length
  sub-structure: a `TrackSize` domain type (fixed / star now; auto
  reserved for later) and per-child placement metadata. The ADR must
  make those sub-types explicit rather than pretending Grid is only
  another stringly widget tag.
- **Track declaration syntax.** A compact attribute form such as
  `columns: ...` / `rows: ...`, repeated declarations, or a nested
  track-list child surface. Default: compact attributes whose RHS is
  a string-encoded track list (for example `columns: "180 1* 2*"`).
  This avoids reopening general list / collection grammar in Phase 5,
  at the cost of moving track-list syntax diagnostics out of the core
  DSL parser and into wasamoc's Grid-specific check plus runtime
  validation.
- **Minimum valid shape.** Empty rows / columns are malformed; a Grid
  needs at least one row and one column. Zero children are valid and
  produce an empty drawn subtree with a resolved outer size.
- **Indexing convention.** Row and column indices are zero-based
  internally; the ADR must decide whether the `.ui` surface exposes
  zero-based values or a friendlier one-based author convention.
  Default recommendation for consistency with runtime / tests:
  zero-based, documented explicitly.

### DD-M3-P5-002 — Track sizing forms and star sizing surface

Phase 5 must choose the track sizing forms that are normative in
`docs/dsl_spec.md`.

Sub-issues:

- **Fixed tracks.** Integer pixel track sizes, likely reusing
  existing signed `IntLit` token plumbing but rejecting negative
  values at `wasamoc check` / IR validation.
- **Star tracks.** Unit star (`*`) and weighted star (`2*`, `3*`,
  etc.) vs only unit star in Phase 5. Default: admit positive
  integer weights because star weighting is central to A2 and avoids
  a knowingly incomplete star surface. Star weights are parsed into
  positive integers; zero and negative weights are malformed.
- **Auto / intrinsic tracks.** Admit `auto` now, defer it, or emulate
  it through fixed / star only. Draft recommendation: **defer `auto`
  from Phase 5**. The Phase 5 spec reserves the concept as future
  work but does not define auto-track demand distribution. This keeps
  the central algorithm fixed + weighted-star + spanning rather than
  letting auto dominate the proof.
- **No DSL parser extension for track lists.** Because DD-001
  recommends a string-encoded track list, Phase 5 needs no DSL parser
  extension for Grid tracks; semicolon member-separator acceptance
  remains a post-Phase-4 open question outside Phase 5 thesis scope.
  See the Phase 4 close disposition of T5 副次学び #3 in
  [m3-phase-4-progress.md](../../plans/progress/m3-phase-4-progress.md#decisions-log).

### DD-M3-P5-003 — Child placement, spanning, and conflict policy

Grid children need per-child placement metadata.

Sub-issues:

- **Placement attributes.** `row`, `column`, `row-span`, and
  `column-span` as constant-only Grid-parent-scoped attributes on
  direct Grid children vs wrapper nodes. Default: direct child
  attributes, because the surface is local to placement and mirrors
  common Grid models, but this is **not** a general widget-catalog
  property. A `Text { row: 0 }` is meaningful when the Text is a
  direct child of Grid; the same attribute outside a Grid-parent
  context remains invalid / unknown.
- **Defaults.** Decide whether omitted `row` / `column` defaults to
  `(0, 0)` or whether explicit placement is required. Default
  recommendation: a single-child Grid may omit `row` / `column` and
  default to `(0, 0)`; a Grid with two or more children requires
  explicit `row` and `column` on every child. No auto-placement
  policy exists in Phase 5.
- **Span defaults and bounds.** Omitted `row-span` /
  `column-span` default to `1`. A span must be positive and must not
  exceed the declared row / column count.
- **Conflict policy.** Duplicate cell origins and overlapping spans
  are rejected in Phase 5. This preserves A2's "1 cell 1 child" rule
  and leaves overlay to Phase 6 ZStack.

### DD-M3-P5-004 — Track-resolution algorithm

This is the load-bearing algorithmic DD.

Sub-issues:

- **Axis independence.** Rows and columns use the same track
  resolution algorithm over different dimensions unless a concrete
  child-measure dependency requires an explicit ordering.
- **Fixed / star distribution.** Fixed tracks consume definite
  parent space first; remaining bounded space is divided among star
  tracks by positive weight. If remaining space is negative, star
  tracks resolve to zero and overflow is handled by child paint /
  parent clipping rules already established for layout primitives.
- **Rounding.** Track resolution works in `f32` layout space, matching
  the existing layout engine convention. Star distribution divides the
  remaining `f32` space by integer weights; no integer pixel snap is
  introduced in Phase 5. Any final device-pixel snapping remains a
  renderer / platform concern, not a Grid algorithm step. Track
  boundaries are deterministic prefix boundaries: `boundary[0] = 0`,
  interior `boundary[n]` is the cumulative `f32` sum of resolved
  track widths before `n`, and the final boundary is the Grid's
  resolved outer extent on that axis rather than an independently
  snapped pixel value.
- **Intrinsic / auto tracks.** The draft recommendation defers auto
  tracks. If owner reverses that decision, DD-004 must define which
  child measurements contribute to each auto track and how spanning
  children distribute demand across multiple tracks before ADR
  Accepted.
- **Spanning reconciliation.** Children spanning multiple tracks are
  measured against the combined resolved span. Phase 5 defers `auto`,
  so no track grows after fixed / star resolution: oversized spanning
  children overflow per DD-M3-P5-005's paint-overflow rule. The
  `auto`-as-growth-target rule is reserved for a future phase that
  admits `auto`.
- **Unbounded parent branch.** The ADR must define how star tracks
  behave when the parent bound on an axis is unbounded. Default for
  review: star tracks act as zero-minimum intrinsic tracks unless
  fixed content supplies size; star-only unbounded axes therefore
  resolve to zero on that axis rather than producing NaN /
  infinity. Because star weights are positive integers, an all-zero
  weight sum is rejected before layout.

### DD-M3-P5-005 — Arrange algorithm and visual-layer contract

Grid arranges each child into the rectangle formed by its resolved
row / column span.

Sub-issues:

- **Child alignment inside cell.** Stretch by default vs leading /
  centered placement. Default: stretch within the resolved span,
  consistent with stack cross-axis stretch and the target app's need
  for stable thumbnail cells.
- **Overflow.** Grid never installs a clip in Phase 5. A child whose
  desired size exceeds its cell paints according to the existing
  parent / clip rules. This can produce visible paint overflow into a
  neighbouring cell's visual area; that is permitted overflow, not
  deliberate same-cell overlay. Occupancy overlap remains invalid, and
  ZStack remains the surface for intentional overlay.
- **Visual ownership.** Grid should not introduce an intermediate
  Visual. It is a pure layout container like WrapPanel, not a
  viewport / translation primitive like ScrollView.
- **Production root shape.** Verification must include at least one
  integration fixture whose parent shape matches production gallery
  root usage, per [predoc-inputs.md §1](./predoc-inputs.md#1-integration-test-fixture-parent-shape-は-production-root-shape-を必ずカバーする).

### DD-M3-P5-006 — IR-loader defense-in-depth invariants

The ADR must decide which Grid invariants are dual-gated by
`wasamoc check` and runtime `validate()`. Recommended gate ownership:

| Invariant | Gate |
|---|---|
| Grid has at least one row and at least one column | Structural; both `wasamoc check` and runtime `validate()` |
| Track-list string parses successfully into a `TrackSize` sequence | **Phase 5 new string-internal gate**; `wasamoc check` is primary, runtime `validate()` is the memory-IR safety net |
| Track sizes are positive where required; star weights are positive integers (`0*`, negative weights, and all-zero star sums are malformed) | Value range; both gates |
| Child placement indices are in range of the declared track count | Cross-attribute value range; both gates |
| Spans are positive and `origin + span <= track_count` | Cross-attribute value range; both gates |
| Same-cell conflicts / overlapping spans are rejected | Cross-child structural check; both gates |

---

### Out of scope (to be carried in the ADR's Out-of-scope section)

- Same-cell overlap / overlay. Phase 6 ZStack owns overlay.
- Responsive breakpoint grammar, media queries, and named areas.
- General list / collection syntax beyond the minimum needed to
  express row and column tracks.
- Grid-level clip attributes or per-cell clipping. Grid never
  installs a clip in Phase 5; clipping remains the responsibility of
  an enclosing clipping parent such as ScrollView or a future
  explicit surface.
- Bindable Grid track definitions or bindable child placement.
  Phase 5 should keep Grid attributes constant-only unless owner
  explicitly expands scope.
- Drag-resizable columns / rows, splitters, and any pointer-driven
  layout resize. These remain M4 or later input-handling work.
- Scrollbar, wheel, drag-to-scroll, and `scroll_y` Signal write-back.
  The Phase 4 `scroll_y` drift item is an M4 handoff, not a Phase 5
  Grid topic.
- Window chrome / theming. The R1 Window-title residual is assigned
  below, but its implementation is not Phase 5 thesis scope unless
  owner explicitly redirects.

---

## Owner-agreed framing decisions

These are draft recommendations for owner review. Once aligned, this
section becomes the owner-agreed agenda for the ADR draft.

### A. DD slate completeness

The six-DD slate is complete if it answers:

- what Grid is in IR and `.ui`;
- how tracks are declared;
- how star sizing is specified;
- how children are placed and spans are validated;
- how layout resolves tracks and arranges children; and
- which malformed cases are rejected at both compiler and runtime
  boundaries.

No separate DD is proposed for C ABI because Phase 5 should not add a
host-facing ABI surface if Grid uses constant-only attributes and the
existing memory-IR loading path.

### B. Pre-doc-discipline check

Phase 5 is one of the phases the M3 plan explicitly warns may bog
down in spec complexity. The mitigation is to start the `dsl_spec.md`
Grid chapter during pre-doc / ADR acceptance, not to defer the spec
until implementation close.

The ADR should therefore land with:

- DD-M3-P5-004 written as a deterministic algorithm, not a sketch;
- a Moment 1 `docs/dsl_spec.md` Grid chapter with `Phase status:
  M3-Phase 5 design accepted; implementation pending`; and
- a Moment 2 close checklist item that re-syncs the chapter against
  the implementation.

### C. Verification strategy

Phase 5 verification should include:

- pure layout tests for fixed tracks, weighted star tracks, spanning,
  and unbounded-parent behavior;
- `wasamoc check` tests for malformed track lists, out-of-range
  placement, invalid spans, and overlap conflicts;
- runtime IR validation tests for the same invariant classes, so
  memory IR cannot bypass compiler checks;
- at least one mock-free Windows integration test that constructs a
  Grid through the runtime widget path; and
- a gallery visible proof that grows `examples/gallery/gallery.ui`
  with a Grid-backed slice.

The integration fixture must cover production root shape, per Phase 4
carry-forward. Recommended minimum: one Grid-rooted fixture and one
`VStack { Grid { ... } }` fixture, because the latter matches the
current gallery root family and protects against another
runtime-boundary sizing miss.

Evidence items do not collapse just because they share helper
infrastructure. A single fixture builder may be reused, but the ADR's
verification closure should keep separate evidence lines for
algorithmic layout, compiler diagnostics, runtime validation,
Windows-runtime integration, and gallery visible proof. This preserves
the Phase 4 lesson that a helper-compatible test can still miss a
production parent shape.

### D. Non-root Shrink parent with Fill child

[predoc-inputs.md §2](./predoc-inputs.md#2-non-root-の-shrink-container-が-fill-子を持つ場合の挙動)
requires Phase 5 to make this design space explicit.

Draft recommendation: Phase 5 keeps the existing convention.
Window-root `WidgetNode::run_layout_as_window_root` may force the
root to Fill / Fill, but non-root Shrink containers with Fill children
continue to follow the existing
`degenerate_fill_in_shrink_parent_clamps_to_zero` behavior. Grid does
not create a Grid-specific exception.

If owner wants Grid to pierce that convention, the change should be
recorded as a broader layout DD rather than hidden inside Grid.

### E. R1 Window-title wiring owning phase

[predoc-inputs.md §4](./predoc-inputs.md#4-r1-window-title-wiring-の-owning-phase-割当--phase-5-pre-doc-内で必須完了)
requires Phase 5 pre-doc framing to assign the owning phase for
R1 — Gallery host Window title wiring.

Draft recommendation: assign R1 to **M3-Phase 6 (ZStack +
conditional rendering)**, not to Phase 5.

Rationale:

- R1 is a real M3 residual: `.ui` `title:` must drive the actual
  native Window title.
- R1 is not Grid thesis scope. Putting it in Phase 5 would distract
  from star sizing and Grid's spec-heavy layout algorithm.
- Phase 6 is the natural next candidate because the lightbox UX makes
  window-level metadata more visible while still leaving enough runway
  before the hard deadline of M3-Phase 8 Gallery E2E close.
- Phase 7 is a worse fit because iteration grammar should stay
  focused on template expansion / item context rather than pulling in
  host-window metadata.
- Phase 8 is too late because it is the Gallery E2E close and public
  spec promotion phase; using the deadline phase as the owning phase
  leaves no recovery window if the host path needs more than a small
  fix.

Required follow-through after owner alignment:

- update [m3-plan.md](../../plans/m3-plan.md) Phase 6 Notes with
  "M3-Phase 4 R1 (Window title wiring) owning phase";
- record in the Phase 5 ADR that R1 is Phase 5 thesis scope **out**;
  and
- cross-reference the assignment from the Phase 4 R1 residual entry
  or the Phase 5 progress file, depending on the chosen commit
  shape.

### F. Phase 4 residual scan — disposition

The Phase 4 progress file's
[Out-of-phase residuals](../../plans/progress/m3-phase-4-progress.md#out-of-phase-residuals)
contains one open residual at Phase 5 pre-doc time:

- **R1 — Gallery host Window title wiring.** Disposition: assign to
  M3-Phase 6 per framing decision E. Phase 5 records the assignment
  but does not implement it.

Related Phase 3 residual context:

- **Phase 3 R2 — `sync_visuals` coverage gap.** Closed inside Phase
  4 T4, with evidence recorded in
  [m3-phase-4-progress.md](../../plans/progress/m3-phase-4-progress.md#t4--windows-runtime-layout-and-visual-evidence-including-r2-closure).
  No Phase 5 action.

Phase 5 does not create an additional residual bucket during pre-doc.
If Grid implementation discovers real-but-out-of-scope issues, they
must be recorded under Phase 5 progress / notes with owner phase,
resolution condition, and deadline, following the R1 pattern.

### G. Upstream-document revision timing (two sync moments)

Phase 5 follows the same two-moment document rule as Phases 2-4:

**Moment 1 — ADR Accepted commit set (design-spec draft).** The
following documents are expected to land as separate commits by review
concern, not as a single "Moment 1" bundle:

- `docs/decisions/m3-phase-5-grid.md` — ADR `Status: Accepted`
  flip.
- `docs/dsl_spec.md` — new §4.12 Grid chapter as a design-spec
  draft, plus the §4.4 widget registry row. The chapter records
  `Phase status: M3-Phase 5 design accepted; implementation
  pending`.
- `docs/architecture.md` — expected to receive an entry for the Grid
  IR variant, including the new `TrackSize` domain type and
  Grid-parent-scoped child placement metadata, mirroring how Box /
  WrapPanel / ScrollView are documented. It may also add a layout-
  engine paragraph if the accepted track-resolution algorithm warrants
  durable cross-phase commentary.
- `docs/abi_spec.md` — untouched in the recommended path. Grid adds
  no host-facing ABI surface and no `PropertyValue` tag.
- `docs/plans/m3-plan.md` — R1 owning-phase note on the Phase 6 row
  if owner accepts framing decision E. This is a plan-note edit, not
  part of the Grid thesis commit.
- `docs/plans/progress/m3-phase-5-progress.md` — new progress file
  opened after ADR acceptance, with the final-step retrospective
  split present from the start.

**Moment 2 — Phase close commit set (implementation re-sync).**

- `docs/dsl_spec.md` §4.12 — section marker flips to "closed;
  implementation-synced", plus any corrections required if the
  design draft and implementation diverged.
- `docs/architecture.md` — re-sync if implementation created a
  durable runtime / layout-engine convention not already captured at
  Moment 1.
- `docs/plans/m3-plan.md` — Phase 5 row status and ADR/progress
  pointers update when Phase 5 closes.
- `docs/plans/progress/m3-phase-5-progress.md` — close evidence,
  CI pointer, residuals, and lifecycle transition recorded.
- Step retro `phase-sync` items must all close into `doc-folded` /
  `carry-forward` / `local-only` at Moment 2; no open
  `phase-sync` item survives phase close.

### H. Phase 5 visible proof

Phase 5 should grow `examples/gallery/gallery.ui` additively with a
Grid-backed composition rather than replacing the existing Box /
WrapPanel / ScrollView proof.

The visible proof should:

- exercise fixed and star tracks in the actual gallery `.ui`;
- include at least one spanning child if DD-M3-P5-003 admits spans;
- avoid using ZStack-style overlap; and
- remain compatible with Phase 6 placing a lightbox over the result
  later.

Recommended minimum shape: a 3-row x 3-column Grid with at least five
children, matching the author-facing gallery-slice example below:
one header child spanning all three columns, three middle-row children
occupying separate columns, and one footer / metadata child spanning
all three columns. This minimum exercises fixed tracks, at least one
star track, explicit placement, and spanning without requiring
ScrollView / ZStack composition.

Composition with existing primitives is allowed but not the central
proof. The minimum visible proof should not require
`ScrollView { Grid { ... } }` or `Grid { ScrollView { ... } }` to be
accepted. If the implementation naturally includes such composition,
it may be tested as an extra confidence check, but Phase 5's A2
evidence is Grid track sizing / placement / spanning itself.

### I. GUI smoke responsibility separation

Phase 5 should preserve the Phase 4 lesson: automated build / launch
evidence and owner-visible correctness are distinct gates.

Draft recommendation: the progress file includes a dedicated
owner-manual GUI smoke step if the Grid gallery slice changes visible
layout enough that automated tests cannot fully judge it. The final
mechanical close step should run only after that visible proof is
green or an explicit owner fail observation has been recorded and
resolved.

### J. Live-note re-evaluation triggers — handling

The `docs/notes/*` live notes are settled upfront so the ADR Inputs
section can cite their disposition rather than re-deciding:

- **[architectural-family.md](../architectural-family.md) — stays
  consumed.** Grid is a built-in layout primitive in the established
  tree-with-bindings family.
- **[layout-engine.md](../layout-engine.md) — partial fire.**
  Grid directly exercises §3.1 DPI / logical-pixel discipline and
  §3.4 cache invalidation. Disposition: keep layout in logical
  `f32` coordinates; no pixel snapping; no subtree dirty propagation
  in Phase 5. The 1,000-node threshold remains unfired because the
  gallery slice is fixed-child and well below that scale.
- **[dsl-grammar.md](../dsl-grammar.md) — partial fire.** DD-M3-P5-002
  intentionally avoids general list / collection grammar by using a
  string-encoded track list. Q1 widget ids, Q3 iteration grammar, and
  Q5 expression grammar remain Phase 6 / Phase 7+ unless Grid
  implementation unexpectedly needs them.
- **[component-extension-model.md](../component-extension-model.md) —
  unfired.** Grid is built-in, not a user-defined layout component.
- **[typed-value-evaluator.md](../typed-value-evaluator.md) —
  unfired in the recommended path.** Grid track / placement
  attributes are constant-only. No item context, no bindable track
  definitions, and no new typed evaluator value are introduced.
  Phase 7 may reopen item-context pressure.
- **[workspace-layout.md](../workspace-layout.md) — unfired.** No new
  crate is expected.
- **[verification-environments.md](../verification-environments.md) /
  [headless-verification.md](../headless-verification.md) — fired via
  inherited discipline.** Phase 5 keeps fail-rather-than-silently-skip
  gates and separates headless evidence from owner-visible GUI smoke.
- **[process-rules-ssot.md](../process-rules-ssot.md) — relevant.**
  Phase 5 keeps the ADR / progress / retrospective role split and the
  step item 10 disposition vocabulary.
- **[release-distribution.md](../release-distribution.md) — unfired.**
  Phase 5 introduces no release / packaging surface.

### K. Grid mental model anchor in dsl_spec

The Moment 1 `docs/dsl_spec.md` Grid chapter should start with a
short mental-model anchor before the algorithm:

- rows and columns define tracks;
- fixed tracks take definite space first;
- star tracks divide remaining bounded space by weight;
- children occupy exactly one cell rectangle or one rectangular span;
- Grid arranges children into resolved rectangles and does not
  provide overlay or clipping by itself.

This mirrors the short mental-model anchors added for WrapPanel and
ScrollView and gives external readers a stable entry point before the
track-resolution details.

**Ecosystem contrast (one bullet each).** Grid's surface intersects
several incompatible mental models:

- **WPF `Grid`.** WPF uses `RowDefinition` / `ColumnDefinition` and
  attached `Grid.Row` / `Grid.Column` properties. Wasamo adopts the
  row / column placement idea but does not introduce definition nodes
  or attached-property machinery in Phase 5; rows / columns are
  compact Grid attributes and child placement is ordinary child
  metadata.
- **CSS Grid.** CSS Grid has named lines, template areas, auto-flow,
  fractional units, minmax, gap, and dense placement. Wasamo Phase 5
  is narrower: fixed tracks, weighted star tracks, explicit placement,
  rectangular spans, and no auto-placement / named areas.
- **Jetpack Compose / SwiftUI grids.** Those ecosystems often model
  adaptive or lazy grids that generate children from data. Wasamo
  Phase 5 is not lazy and does not generate children; Phase 7
  iteration is the future place where generated Grid children can be
  introduced.
- **ZStack / overlay models.** Grid does not provide intentional
  overlay. Paint overflow may be visible if a child is larger than
  its cell, but two children may not deliberately occupy the same
  cell; Phase 6 ZStack owns overlay.

---

## Author-facing `.ui` examples under the current recommendations

This section is illustrative input for owner alignment, not final
grammar. It is intentionally present in the framing note because
DD-M3-P5-002's string-encoded track-list recommendation is easier to
review when owner can see the resulting author-facing `.ui` shape.
The exact tokenisation inside the string is still owned by the ADR,
but the examples below show the recommended compact `rows:` /
`columns:` attribute shape:

- compact row / column declarations on `Grid`;
- positive integer fixed tracks;
- unit and weighted star tracks;
- explicit child placement with `row` / `column`;
- positive `row-span` / `column-span`;
- no auto-placement;
- no same-cell overlap; and
- no ZStack-style overlay.

### Example 1 — fixed sidebar + star content

A two-column layout with a fixed navigation rail and a flexible content
area:

```wasamo-ui
Grid {
  columns: "180 *"
  rows: "*"

  Box {
    row: 0
    column: 0
    fill: #243447ff
    Text { text: "Albums" }
  }

  Box {
    row: 0
    column: 1
    fill: #f5f7faff
    Text { text: "Selected album" }
  }
}
```

The fixed track receives 180 px. The star track receives the remaining
bounded width. The two children occupy different cells, so this stays
inside the "1 cell 1 child" rule.

### Example 2 — weighted star columns

A three-column gallery header where the center region gets twice the
remaining width of each side region:

```wasamo-ui
Grid {
  columns: "1* 2* 1*"
  rows: "72"

  Text {
    row: 0
    column: 0
    text: "Back"
  }

  Text {
    row: 0
    column: 1
    text: "Summer Trip"
  }

  Text {
    row: 0
    column: 2
    text: "Share"
  }
}
```

If the Grid receives 800 px of width and no fixed columns exist, the
resolved column widths are proportional to `1 : 2 : 1`. DD-M3-P5-004
uses deterministic `f32` prefix boundaries and no integer pixel snap.

### Example 3 — spanning without overlap

A layout where a hero tile spans two columns, while smaller tiles sit
below it:

```wasamo-ui
Grid {
  columns: "1* 1*"
  rows: "220 120"

  Box {
    row: 0
    column: 0
    column-span: 2
    fill: #336699cc
    Text { text: "Featured photo" }
  }

  Box {
    row: 1
    column: 0
    fill: #88aa55cc
    Text { text: "Detail A" }
  }

  Box {
    row: 1
    column: 1
    fill: #aa6655cc
    Text { text: "Detail B" }
  }
}
```

The first child occupies the rectangular span `(row 0, columns 0..2)`.
The two lower children occupy separate cells. This is valid because no
resolved cell is claimed by more than one child.

### Example 4 — gallery slice candidate

A Phase 5 gallery proof could add a fixed-child Grid slice without
replacing the existing Box / WrapPanel / ScrollView proof:

```wasamo-ui
Grid {
  columns: "96 1* 96"
  rows: "64 1* 120"

  Text {
    row: 0
    column: 0
    column-span: 3
    text: "Gallery"
  }

  Box {
    row: 1
    column: 0
    fill: #2f4050ff
    Text { text: "Prev" }
  }

  Box {
    row: 1
    column: 1
    fill: #3f7caccc
    Text { text: "Preview" }
  }

  Box {
    row: 1
    column: 2
    fill: #2f4050ff
    Text { text: "Next" }
  }

  Text {
    row: 2
    column: 0
    column-span: 3
    text: "Metadata and actions"
  }
}
```

This example exercises fixed tracks, star sizing, and spanning in a
shape that later Phase 6 can overlay with ZStack and later Phase 7 can
populate through iteration. It deliberately does not express overlay
inside Grid.

### Invalid shapes the recommendation rejects

The current recommendation intentionally rejects the following shapes:

```wasamo-ui
Grid {
  columns: "1*"
  rows: "1*"

  Box {
    row: 0
    column: 0
    fill: #336699cc
  }

  Box {
    row: 0
    column: 0
    fill: #aa6655cc
  }
}
```

Both children claim the same cell. Phase 5 treats this as a diagnostic,
not as "last child wins" overlay.

```wasamo-ui
Grid {
  columns: "1* 1*"
  rows: "1*"

  Box {
    column: 0
    fill: #336699cc
  }

  Box {
    row: 0
    column: 1
    fill: #88aa55cc
  }
}
```

The first child omits `row` in a multi-child Grid. Under the current
recommendation, Phase 5 does not provide auto-placement; multi-child
Grid authors spell `row` and `column` explicitly. A single-child Grid
may omit both and default to `(0, 0)`.

```wasamo-ui
Grid {
  columns: "1* 1*"
  rows: "1*"

  Box {
    row: 0
    column: 1
    column-span: 2
    fill: #336699cc
  }
}
```

This span exceeds the declared column count and is rejected by both
`wasamoc check` and runtime IR validation.

If owner accepts `auto` / intrinsic tracks in DD-M3-P5-002, a later
draft can add an example such as a metadata column sized by content.
The current recommendation defers `auto`, so the examples above stay
within fixed + weighted-star semantics.

---

## Inputs absorbed

### From [predoc-inputs.md](./predoc-inputs.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §1 integration fixture parent shape must cover production root shape | Direct input | Framing decision C; DD-M3-P5-005 verification note |
| §2 non-root Shrink container + Fill child design space | Design-space decision | Framing decision D; draft recommendation is status quo |
| §3 `scroll_y` Signal drift | Out of scope for Phase 5 | Out-of-scope section; M4 handoff only |
| §4 R1 Window-title wiring owning phase | Required pre-doc assignment | Framing decision E; residual scan decision F; m3-plan Phase 6 note after owner alignment |
| §5 final-step retrospective split | Process rule | Opening assumptions; Phase 5 progress file template |

### From [m3-plan.md](../../plans/m3-plan.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §Acceptance criteria — A2 | Constraint | Acceptance restatement; DD-M3-P5-001..006 |
| §Acceptance criteria — A11 | Operational rule | Acceptance restatement; framing decisions C / G / H |
| §Acceptance criteria — A12 | Spec obligation | Acceptance restatement; framing decisions B / G / K; new `docs/dsl_spec.md` §4.12 |
| Phase breakdown: star sizing is central algorithmic content | Algorithmic constraint | DD-M3-P5-004; framing decision B |
| Same-cell overlap is ZStack responsibility | Scope boundary | Out-of-scope section; DD-M3-P5-003 conflict policy; framing decision K ecosystem contrast |
| Risk: Grid measure-arrange spec complexity | Mitigation | Moment 1 spec drafting in framing decision G; mental-model anchor in framing decision K |

### From [m3-gallery-wireframe.html](../../references/m3-gallery-wireframe.html)

| Element | Disposition | Consumed at |
|---|---|---|
| 2D gallery composition pressure | Visible-proof reference | Framing decision H; author-facing examples |
| Future lightbox / overlay relationship | Downstream constraint | DD-M3-P5-003 conflict policy; framing decision K (ZStack owns overlay) |
| Future iteration-generated thumbnails | Downstream constraint | Acceptance restatement downstream paragraph; Phase 7 handoff note |

### From [docs/decisions/m3-phase-3-wrap-panel.md](../../decisions/m3-phase-3-wrap-panel.md)

| DD / precedent | Disposition | Consumed at |
|---|---|---|
| DD-M3-P3-005 pure-data measure-arrange | Pattern reuse | DD-M3-P5-004 |
| First novel-normative-spec phase discipline | Pattern reuse | Acceptance restatement; framing decisions B / G / K |
| Paint overflow not clipped by layout primitive itself | Pattern reuse with Grid-specific clarification | DD-M3-P5-005 (paint overflow may enter neighbouring visual area; occupancy overlap still invalid) |

### From [docs/decisions/m3-phase-4-scroll-view.md](../../decisions/m3-phase-4-scroll-view.md)

| DD / precedent | Disposition | Consumed at |
|---|---|---|
| Runtime-boundary root-shape lesson | Direct input | Framing decision C; DD-M3-P5-005 verification note |
| ScrollView intermediate Visual pattern | Negative precedent | DD-M3-P5-005 (Grid remains one WidgetNode / one Visual unless a concrete DD demands otherwise) |
| Phase 4 R1 residual | Carry-forward assignment | Framing decisions E / F |
| Phase 4 `scroll_y` drift | Out of scope | Out-of-scope section; M4 handoff only |

### From [m2-to-m3-handover.md](../m2-to-m3-handover.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §3 reactive drain residuals | Out of scope in recommended path | Live-note decision J; constant-only Grid attributes do not pressure drain residuals |
| §4 `TypedValue` deferral | Discipline reminder | Live-note decision J; no bindable track / placement and no item context in Phase 5 |

---

## Next session — handoff

To move from this draft to ADR drafting:

1. Owner reviews framing decisions A-K, especially DD-M3-P5-002
   (string-encoded track list and
   `auto` deferral), DD-M3-P5-003 (single-child default vs
   multi-child explicit placement), DD-M3-P5-004 (unbounded star
   behavior), DD-M3-P5-005 (paint overflow vs overlay), and decision
   E (R1 assigned to Phase 6).
2. If aligned, draft `docs/decisions/m3-phase-5-grid.md` as
   `Status: Proposed`.
3. Draft the Moment 1 `docs/dsl_spec.md` Grid chapter and widget
   registry row.
4. Create `docs/plans/progress/m3-phase-5-progress.md` with the
   final-step retrospective split already present.
5. Update `docs/plans/m3-plan.md` to assign R1 to Phase 6 if owner
   accepts decision E.
