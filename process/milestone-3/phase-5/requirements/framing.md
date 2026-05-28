---
title: M3-Phase 5 framing — Grid layout primitive
status: draft
created: 2026-05-26
restored-from: 15ea0e35e2a499744d166712b53a17c4d68c91ff
target-phase: M3-Phase 5
---

# M3-Phase 5 framing

**Former status:** reviewed; pending owner alignment; input artefact for ADR drafting
**Restored:** 2026-05-28 from commit `15ea0e35e2a499744d166712b53a17c4d68c91ff`
**Targets phase:** M3-Phase 5 (Grid layout primitive)

Per the project's doc-driven workflow established at
[M2-Phase 6 pre-doc framing](../../../milestone-2/phase-6/requirements/framing.md)
and continued through
[M3-Phase 2 pre-doc framing](../../phase-2/requirements/framing.md),
[M3-Phase 3 pre-doc framing](../../phase-3/requirements/framing.md),
and
[M3-Phase 4 pre-doc framing](../../phase-4/requirements/framing.md),
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
  [m3-phase-2 framing decision D](../../phase-2/requirements/framing.md#d-upstream-document-revision-timing-two-sync-moments).
- **Moment-is-not-a-commit-unit rule**, recorded in
  [CLAUDE.md §Commit rules](../../../../CLAUDE.md#commit-rules): each
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
- **Final-step retrospective split**: Phase 5's implementation plan should
  split the final step's task-end retrospective from the
  phase-end retrospective from the start, per
  [constraints.md §5](./constraints.md#5-phase-最終-step-の-retrospective--progress-checklist-は-step-end-と-phase-end-を分割する).

---

## Phase 5 acceptance criteria (restated)

- **A2** (see [process/_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
  [process/milestone-3/plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)):

  > Grid layout primitive (1 cell 1 child, star sizing + spanning;
  > same-cell overlap is **not** provided — overlay is ZStack's
  > responsibility).

  The three load-bearing parts are **track definition**, **star
  sizing**, and **row / column spanning**. "1 cell 1 child" is a
  Phase 5 scope boundary: a Grid child occupies one resolved cell
  rectangle or one spanning rectangle; deliberate overlap in the
  same resolved cell belongs to Phase 6 ZStack, not to Grid.

  The roadmap wording "row / column spanning" reads both axes as in
  scope, but per-axis admission inside Phase 5 (column-span only,
  row-span only, both, or one deferred while the attribute name is
  reserved) is a DD-M3-P5-003 sub-issue, not a framing-time
  commitment. Whichever choice the ADR settles, the surface-family
  impact differs between coordinate and structural families; that
  impact is recorded as a comparison axis (DD-M3-P5-001 axis 2 below)
  so the ADR can weigh it independently of the M3 admission scope.

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
  `docs/dsl_spec.md` should be able to reproduce Grid's track /
  membership surface, star sizing, spanning,
  validation, and overflow semantics against a hypothetical host.

- **Second novel-normative-spec phase.** Phase 3 introduced the
  first novel measure-arrange spec through WrapPanel's line breaker.
  Phase 5 is the second and heavier novel-normative-spec phase:
  Grid's star sizing requires a deterministic track-resolution
  algorithm, including how fixed and star tracks interact with
  spanning children. `auto` / intrinsic tracks are explicitly a
  design pressure point. The conservative draft recommendation below
  defers them from Phase 5 not to minimize implementation effort, but
  to avoid shipping half-specified `auto` / spanning interactions that
  would constrain v1.0 compatibility. The ADR must either reserve the
  future `auto` slot cleanly or fully specify `auto` now.
  Acceptance for the spec text is
  the [process/milestone-3/plan.md §Milestone-end criteria item 5](../../plan.md#milestone-end-criteria)
  external-reader bar, applied at phase close.

- **Downstream commitments grounded in Phase 5.** Phase 8's Gallery
  E2E proof needs a 2D composition surface for the target app. Phase
  6 may place a lightbox above the gallery via ZStack, but it should
  not be the first proof that the gallery can express a stable 2D
  layout. Phase 5 must therefore ship a fixed-child Grid gallery
  slice that later phases can grow rather than replace.

  Grid is **not** an M3 iteration target. The accepted target-app
  pre-doc
  ([process/milestone-3/requirements/spec.md](../../requirements/spec.md))
  positions Grid as an independent static 2D layout primitive
  (Layout primitive A, thesis = 2D measure-arrange + spanning) and
  decomposes the collection-driven "List" responsibility into
  WrapPanel + ZStack + the iteration grammar; Grid is not part of
  that decomposition, and the iteration grammar's M3 target is the
  WrapPanel-backed thumbnail collection. Phase 7 therefore does not
  generate Grid children in M3. Surface selection in DD-M3-P5-001
  must not be driven by Phase 7 iteration ergonomics. The iteration
  grammar stays general, so the only iteration-related requirement on
  the chosen surface is a **foreclosure check** — it must not make a
  future, post-M3 Grid iteration structurally impossible — and every
  candidate surface (A / A2 / B / D / C) passes that check, so it is
  non-differentiating.

---

## Layering note (DD-001 ⇄ DD-002 ⇄ DD-003 ⇄ DD-004 ⇄ DD-005)

Grid's DD chain is wider than Phase 4's ScrollView chain because the
algorithm has two independent axes and a spanning phase. The chain is:

- **DD-001 (IR shape and author-facing Grid surface).** Settles that
  Grid is a new layout primitive and compares the author-facing
  surface families without presuming the WPF / CSS Grid model:
  track-list attributes plus placed content children, track-list
  attributes plus placed `Cell` wrappers, structural `Row` / `Cell`
  children, hybrid Grid columns plus structural rows, or definition-node
  variants.
- **DD-002 (track sizing forms and constants).** Settles which
  track sizing forms Phase 5 admits: fixed integer pixels, star
  weights, and whether an intrinsic / auto track exists in Phase 5.
  Track-list declaration syntax belongs to DD-001; DD-002 consumes
  that syntax and defines what each track token can mean.
- **DD-003 (child membership and same-cell conflict policy).**
  Settles the "1 cell 1 child" rule precisely for whichever DD-001
  surface is chosen: duplicate coordinate claims in a placement
  surface, malformed structural rows / cells in a structural surface,
  overlapping spans, default placement, and out-of-range indices.
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
- **DD-001 = string-encoded track list** with **DD-006 =
  parser-level track-list diagnostics required**. If owner chooses a
  string mini-language, the core DSL parser sees a string literal;
  track-list syntax errors are checked by `wasamoc check` and runtime
  validation after the string is parsed into Grid's domain type. The
  framing now asks owner to compare string encoding against Grid-specific
  first-class track-list values (A / A2 / D where applicable),
  structural `Row` / `Cell` surfaces, and definition-node surfaces so
  token-level diagnostics and future extensions are not trapped inside a
  string by default.
- **DD-001 = explicit-coordinate surface (A / A2)** with **DD-003 =
  auto-placement admitted** and **DD-004 = no document-order
  placement algorithm**. Auto-placement is not just a default; it is a
  layout policy. If DD-001 chooses structural membership (B / D / C),
  row / cell membership supplies placement structurally and this invalid
  combination does not apply.
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

The Phase 5 ADR set
(`process/milestone-3/phase-5/decisions/preamble.md` plus one
`dd-*.md` file per DD) will carry the following six DDs.

### DD-M3-P5-001 — Grid IR node form and author-facing surface

Grid is a new layout primitive in `wasamo-ir` and `wasamo-runtime`.
Phase 5 must commit to the IR node shape and the author-facing syntax
for declaring tracks, membership, and cell content. Owner has not yet
committed to the WPF / CSS Grid model where the parent declares tracks
and children carry placement metadata, so the ADR must compare that
model flatly against structural `Row` / `Cell` surfaces.

Sub-issues:

- **IR node shape.** Per-kind tag parallel to `Box`, `WrapPanel`,
  and `ScrollView`, vs a structural variant in `IrLayout`. Default:
  per-kind tag, continuing the M3 primitive pattern. Unlike Box /
  WrapPanel / ScrollView, Grid introduces variable-length
  sub-structure. The exact sub-types depend on the chosen surface:
  track-list surfaces need `TrackSize` plus placement metadata (on the
  content child in A, on `Cell` in A2); structural surfaces need row /
  cell group structure plus any per-row / per-cell sizing metadata. The
  ADR must make those sub-types explicit rather than pretending Grid is
  only another stringly widget tag.
- **Surface family.** Owner has explicitly required that Phase 5 not
  treat the WPF / CSS Grid family (track-list + child placement) as
  the default. The ADR must compare the following families flatly:
  - **Surface A — track-list + direct child placement:** `Grid { columns: 180 1*; rows:
    64 *; Text { row: 0; column: 1; ... } }`. Tracks are declared on
    the Grid; placement is parent-scoped child metadata.
  - **Surface A2 — track-list + placed Cell wrapper:** `Grid {
    columns: 180 1*; rows: 64; Cell { row: 0; column: 1; Text { ... } }
    }`. Tracks are declared on the Grid; placement metadata lives on
    a layout-only `Cell` wrapper rather than on the content widget.
  - **Structural row / cell (pure):** `Grid { Row { Cell { ... } Cell {
    ... } } Row { ... } }`, with sizing metadata directly on `Row` /
    `Cell`. Shared column widths across rows are emergent rather than
    declared; see the shared-track-sizing axis below.
  - **Surface D — Grid columns + structural rows:** `Grid { columns:
    180 1*; Row { height: 48; Cell { ... } Cell { ... } } }`. Column
    tracks are declared once on Grid, while row membership and row
    heights are structural. This preserves row / cell readability while
    avoiding the pure structural shared-column reconciliation problem.
  - **Definition nodes + content rows:** explicit `ColumnDefs` /
    `RowDefs` definition nodes plus structural content rows. A structural
    variant that hoists track definitions to Grid level so column
    widths are shared across rows by construction.
  - **String-encoded track-list fallback:** `columns: "180 1*"`,
    a degenerate placement-family variant preserved as a fallback
    rather than a recommended option.

  The comparison must use the five owner-specified axes, applied
  symmetrically to every family above:

  1. **`.ui` author taste** — verbosity, readability, and how natural
     the surface feels for forms, gallery slices, and irregular layouts.
  2. **Spanning** — whether rectangular row / column spans are
     expressible directly, how span declarations sit in the surface
     (child metadata vs `Cell` attribute), and how malformed spans
     surface to authors. The two axes are not symmetric across
     surface families and the ADR must record the asymmetry rather
     than treat "spanning" as one axis. Coordinate families (A / A2)
     handle row-span and column-span uniformly because membership lives
     in a single `(row, column, row-span, column-span)` tuple per child;
     admitting or deferring either axis is a pure scope decision with
     no surface restructure. Structural families (B / D / C) treat the
     two axes differently: column-span is intra-`Row` (advance sibling
     `Cell`s within document order) and falls out of the surface
     naturally, while row-span causes a `Cell` in `Row[i]` to occupy a
     slot in `Row[i+1]`, which requires a separate rule for how
     `Row[i+1]` represents the consumed slot (implicit skip, as in
     HTML `rowspan`, vs explicit placeholder `Cell {}` carrying a
     "covered from above" marker). Whether row-span is admitted in
     Phase 5 or deferred is settled in DD-M3-P5-003; this axis still
     records the surface impact in both cases, because deferral does
     not erase the extension shape — admitting row-span later under a
     structural surface re-opens the same implicit-vs-explicit
     question.
  3. **Shared track sizing** — whether column widths are shared across
     rows by construction (track-list, hybrid-column, and definition-node families) or
     must be reconciled across independent Row declarations (pure
     structural). This axis is load-bearing: a structural surface that
     allows per-row column widths is a different layout primitive than
     Grid, not a stylistic variant.
  4. **Future iteration** — a **foreclosure check, not a
     differentiator**. Grid is not an M3 iteration target (see the
     "Downstream commitments grounded in Phase 5" paragraph above and
     [spec.md](../../requirements/spec.md)): Phase 7's iteration
     grammar drives the WrapPanel-backed thumbnail collection, not
     Grid children. The ADR therefore must not select a surface for
     iteration ergonomics. This axis only asks whether a surface would
     make a future, post-M3 Grid iteration structurally *impossible*;
     each surface has a conceivable iteration shape (placed content
     children carrying `row` / `column` metadata, placed `Cell`
     wrappers, structural rows generated against Grid-level columns or
     definition nodes), so none forecloses it and the axis does not
     separate the candidates.
  5. **Component-extension-model** — what each surface implies about
     future custom layouts and their child contracts. Surface A
     introduces parent-scoped metadata on arbitrary Grid children as a
     first built-in precedent; Surface A2 contains that metadata in a
     Grid-owned `Cell` wrapper; structural surfaces introduce new
     structural child node kinds (`Row`, `Cell`) instead; Surface D
     combines parent-owned column tracks with structural content rows.

  Implementation effort and parser diagnostics are secondary concerns
  and must not drive the surface selection.
  **No surface recommendation is made at framing time.** A critical
  recommendation would require choosing which tradeoff matters most:
  A's irregular-layout power and metadata precedent, A2's wrapper-based
  containment of Grid metadata, B's direct structural authoring with
  reconciliation burden, D's hybrid parent-columns / structural-rows
  split, or C's explicit shared tracks with an extra definition layer.
  The current owner alignment goal is to compare those tradeoffs flatly
  before the ADR selects one.
- **Track declaration syntax.** The location of track sizing
  information depends on the selected surface, and the ADR must not
  presume one location:
  - In the track-list families (A / A2), the preferred syntax is
    compact attributes on `Grid` whose RHS is a **Grid-specific
    first-class track-list value** (for example `columns: 180 1*
    2*`); string-encoded fallback exists but is not the default.
  - In the pure structural family, sizing lives on `Row` / `Cell`
    constructs directly (`Row { height: 64 }`, `Cell { width: 180
    }`), and the ADR must resolve how column widths reconcile across
    rows (see shared track sizing axis).
  - In Surface D, column sizing lives on `Grid.columns` while row sizing
    lives on structural `Row { height: ... }`. This is intentionally
    asymmetric: columns are shared by construction; rows remain visible
    in document structure.
  - In the definition-node family, sizing lives on hoisted
    `ColumnDefs` / `RowDefs` definition children at Grid level, so
    column widths are shared by construction at the cost of an extra
    surface layer. These names intentionally avoid reusing content
    `Row` for row definitions; if ADR chooses shorter names, it must
    explain how parent-scoped meanings stay clear to authors.

  In every case, Phase 5 must not open a general list / collection
  grammar merely to express Grid.
- **Minimum valid shape.** Empty rows / columns are malformed; a Grid
  needs at least one row and one column. Zero children are valid and
  produce an empty drawn subtree with a resolved outer size.
- **Indexing convention.** Row and column indices are zero-based
  internally regardless of the chosen surface. Whether authors ever
  see numeric indices depends on the surface family:
  - Surface A makes indices author-visible on every placed content
    child; Surface A2 makes them author-visible on every placed `Cell`.
    ADR must decide zero-based vs one-based at the `.ui` boundary.
    Recommended at that branch: zero-based for consistency with runtime /
    tests.
  - Pure structural and definition-node families make indices mostly
    invisible — document order assigns membership. Indices may still
    surface inside diagnostics (e.g. "row 2, cell 3 spans past the
    declared column count") and the ADR should fix the convention
    used there.
  - Surface D also makes indices mostly diagnostic-only: row membership
    is structural, and column membership is document order within each
    `Row` against Grid-level columns.

### DD-M3-P5-002 — Track sizing forms and star sizing surface

Phase 5 must choose the track sizing forms that are normative in
`docs/dsl_spec.md`. The forms themselves are surface-independent (a
  fixed pixel width is the same value whether it lives in a Grid
attribute, a `Cell` attribute, or a `ColumnDef` definition); only the
location and parsing differ.

Sub-issues:

- **Fixed tracks.** Integer pixel track sizes, likely reusing
  existing signed `IntLit` token plumbing but rejecting negative
  values at `wasamoc check` / IR validation. Carrier varies by
  surface: `columns: 180 1*` (A / A2 track-list families),
  `columns: 180 1*` plus `Row { height: ... }` (Surface D),
  `Cell { width: 180 }` (pure structural), `ColumnDef { width: 180 }`
  (definition-node).
- **Star tracks.** Unit star (`*`) and weighted star (`2*`, `3*`,
  etc.) vs only unit star in Phase 5. Default: admit positive
  integer weights because star weighting is central to A2 and avoids
  a knowingly incomplete star surface. Star weights are parsed into
  positive integers; zero and negative weights are malformed. The
  star value form is identical across surfaces; only the attribute
  carrying it changes.
- **Auto / intrinsic tracks.** Admit `auto` now, defer it while
  reserving the algorithm slot, or emulate it through fixed / star
  only. Conservative draft recommendation: **defer `auto` from Phase
  5 while explicitly reserving where an auto-demand pass would sit**.
  This is a compatibility choice, not an effort-minimisation choice:
  a half-baked `auto` track without spanning demand distribution would
  be worse for v1.0 than a clearly deferred surface. Owner may instead
  choose the aggressive option: admit `auto` in Phase 5 and require
  DD-M3-P5-004 to fully specify auto-track demand, including spanning
  children, before ADR Accepted. This decision is independent of the
  DD-001 surface choice.
- **Shared track sizing across rows.** Star and fixed forms above
  describe per-track values; the surface choice from DD-001
  determines whether column widths are shared across rows:
  - In the track-list families (A / A2), `columns:` on Grid declares one
    canonical column track list; sharing is automatic.
  - In Surface D, `columns:` on Grid declares one canonical column track
    list; sharing is automatic for columns, while `Row { height: ... }`
    declares row heights structurally.
  - In the definition-node family, `ColumnDefs { ColumnDef ... }` on
    Grid plays the same role; sharing is automatic.
  - In the pure structural family, `Cell { width: ... }` sits inside
    each `Row`. The ADR must answer how the layout engine reconciles
    different widths declared in different rows: reject as malformed
    (rows must agree column-by-column), take the first row as
    canonical and validate the rest, or treat each row as
    independently sized (which is no longer Grid semantics and
    should be rejected upfront). Without this rule, the pure
    structural surface is underdefined.
- **Track-list parser surface.** Parser exposure depends on the
  surface:
  - Track-list families with first-class track-list value: Phase 5 adds
    a narrow parser path for Grid `rows:` / `columns:` attributes
    without opening general list / collection grammar.
  - Track-list families with string-encoded form: syntax diagnostics
    move to Grid-specific `wasamoc check` and runtime validation.
    Editor highlighting / completion / source location degrade.
  - Surface D: same narrow parser path for Grid `columns:` as A / A2,
    but rows are structural children using existing attribute plumbing.
  - Pure structural and definition-node families: no compact
    `rows:` / `columns:` track-list parser at all — `Row` / `Cell` /
    `ColumnDef` / `RowDef` definition nodes are just structural children
    of `Grid`, and their sizing attributes use existing widget attribute
    plumbing.

  Semicolon member-separator acceptance remains a post-Phase-4 open
  question outside Phase 5 thesis scope. See the Phase 4 close
  disposition of T5 副次学び #3 in
  [m3-phase-4-progress.md](../../phase-4/implementation/plan.md#decisions-log).

### DD-M3-P5-003 — Child membership, spanning, and conflict policy

Grid children need a way to become members of resolved cells. That may
be numeric child placement metadata on the content child, numeric
placement metadata on a `Cell` wrapper, structural row / cell
membership, or a hybrid. DD-M3-P5-003 must not presume one model before DD-M3-P5-001
chooses the author-facing surface.

Sub-issues are organised in parallel paragraphs per surface family so
that no family is presented as the implicit default.

- **Membership surface.** The surface families differ on how a child
  becomes a member of a resolved cell:
  - **Surface A — direct placement:** numeric attributes (`row`, `column`,
    `row-span`, `column-span`) on each Grid direct child. These are
    parent-scoped child metadata read by the Grid parent, not
    general widget catalog properties. The component-extension-model
    implication (first built-in precedent for parent-scoped child
    metadata) is recorded under FD-J.
  - **Surface A2 — placed Cell wrapper:** numeric attributes (`row`,
    `column`, `row-span`, `column-span`) on a Grid-owned `Cell` direct
    child. The content widget is the single child of `Cell` and receives
    no Grid-specific metadata. This keeps track-list sharing and
    irregular placement from A while containing Grid-specific child
    contract attributes inside a wrapper.
  - **Pure structural family:** document structure assigns
    membership. `Row` is a Grid direct child; `Cell` is a `Row`
    direct child; the content widget is the (single) `Cell` direct
    child. No widget receives `row` / `column` attributes.
  - **Surface D — Grid columns + structural rows:** membership is
    structural like B / C but with parent-owned columns. `Row` is a Grid direct child; `Cell` is a
    `Row` direct child; each `Cell` advances through the Grid-level
    column list in document order. No widget receives `row` / `column`
    attributes, and no `Cell` carries explicit coordinates.
  - **Definition-node family:** same membership rule as pure
    structural for content rows. The hoisted `ColumnDefs` / `RowDefs`
    definitions are siblings of content rows, not membership carriers.
- **Defaults.** Per-surface:
  - **Surface A:** decide whether omitted `row` / `column` on content
    children defaults to `(0, 0)` or whether explicit placement is
    required. Recommended at that branch: a single-child Grid may omit
    `row` / `column` and default to `(0, 0)`; a Grid with two or more
    children requires explicit `row` and `column` on every child. No
    auto-placement policy exists in Phase 5 regardless of family.
  - **Surface A2:** same placement default question as A, but the
    attributes live on `Cell`. Recommended at that branch: every `Cell`
    in a multi-Cell Grid must carry explicit `row` and `column`; if a
    one-Cell Grid omits them, it may default to `(0, 0)`.
  - **Pure structural / Surface D / definition-node families:** document
    structure replaces placement defaults. Child order inside a
    `Row` defines cell membership; missing `Cell` children inside a
    `Row` are either malformed or implicit empty cells, and the ADR
    must commit one rule. Recommended at this branch: missing cells
    are malformed (an explicit empty `Cell {}` is required to skip a
    column), so structural integrity is checked at `wasamoc check`
    rather than papered over at layout time.
- **Span defaults and bounds.** Per-surface:
  - **Surface A:** omitted `row-span` / `column-span` default
    to `1`; span attributes live on the content widget as
    parent-scoped child metadata.
  - **Surface A2:** omitted `row-span` / `column-span` default to `1`;
    span attributes live on `Cell`. Because `row` / `column` are
    explicit, the span does not consume sibling positions by document
    order; conflict detection works over resolved `Cell` rectangles.
  - **Pure structural / Surface D / definition-node families:** spans live on
    `Cell` (`Cell { column-span: 2 ... }`), never on the content widget.
    Omitted spans default to `1`. A spanning `Cell` consumes column
    slots in document order within its parent `Row`; the ADR must
    define whether skipped columns from a span are implicit (the next
    sibling `Cell` occupies column 3 after a `column-span: 2` Cell in
    column 0) or explicit (the author must still write `Cell {}`
    placeholders). Recommended at this branch: implicit — spans
    consume slots so sibling Cells advance to the next free column.

  In every family a span must be positive and must not exceed the
  declared row / column count.

- **Per-axis admission scope (column-span vs row-span).** The ADR
  must decide whether Phase 5 admits column-span only, row-span only,
  both axes, or both with one axis deferred while its attribute name
  is reserved. The A2 roadmap wording reads both axes as in scope,
  but per-axis deferral is a legitimate ADR choice for narrowing the
  novel-normative-spec surface. This is a scope decision, not a
  framing-time recommendation, and must be settled before
  DD-M3-P5-004's algorithm and DD-M3-P5-005's arrange commit to
  span-reconciliation behavior. The decision interacts with the
  selected surface family; the ADR must record the interaction it
  accepts rather than treating row-span as a pure scope variable:
  - **Surface A / A2 (coordinate).** The two axes are symmetric.
    Admitting row-span reuses the same `(row, column, row-span,
    column-span)` rectangle conflict check and adds no new surface
    concept. Deferring row-span requires reserving the attribute name
    and rejecting it at `wasamoc check` / runtime validation until
    admitted, but no surface restructure.
  - **Surface B / D / C (structural).** The two axes are not
    symmetric. Admitting row-span requires the ADR to commit to one
    of two rules for `Row[i+1]` when `Row[i]` contains `Cell {
    row-span: 2 ... }`:
    - *implicit skip* — `Row[i+1]`'s `Cell` children silently
      bypass the column occupied from above (HTML `rowspan`-like).
      Reading `Row[i+1]` alone no longer yields its visible cell
      sequence, which dilutes Surface B's "document structure mirrors
      visible structure" claim and adds non-local context to
      diagnostics.
    - *explicit placeholder* — `Row[i+1]` must contain a `Cell {}`
      (or a dedicated covered-from-above marker) for the occupied
      column. Local readability is preserved, but authors hand-track
      multi-row coverage (and, should a post-M3 milestone ever iterate
      Grid, a `Row`-generating template would have to emit consistent
      placeholders — not an M3 concern, since Grid is not an M3
      iteration target).

    Deferring row-span from Phase 5 leaves this rule choice open for
    a later phase but does not erase it; admitting row-span now forces
    the ADR to commit to one rule and reflect it in DD-M3-P5-006's
    structural validation surface.

  This sub-issue is intentionally raised at framing time so the ADR
  Options table can show row-span examples per surface (or explicitly
  mark them deferred) instead of inheriting an implicit column-span-
  only baseline from the framing's illustrative examples.
- **Conflict policy.** Duplicate cell origins and overlapping spans
  are rejected in Phase 5. This preserves A2's "1 cell 1 child" rule
  and leaves overlay to Phase 6 ZStack. The conflict-detection input
  varies by surface (resolved `(row, column, row-span, column-span)` on
  content children for Surface A; the same tuple on `Cell` wrappers for
  Surface A2; resolved Cell-to-column mapping for the structural and
  hybrid structural-row families) but the rejection rule is uniform.

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
- **Intrinsic / auto tracks.** The conservative draft recommendation
  defers auto tracks while reserving an explicit future algorithm
  slot before star distribution. If owner reverses that decision,
  DD-004 must define which child measurements contribute to each auto
  track and how spanning children distribute demand across multiple
  tracks before ADR Accepted.
- **Spanning reconciliation.** Children spanning multiple tracks are
  measured against the combined resolved span. Phase 5 defers `auto`,
  so no track grows after fixed / star resolution: oversized spanning
  children overflow per DD-M3-P5-005's paint-overflow rule. The
  `auto`-as-growth-target rule is reserved for a future phase that
  admits `auto`.
- **Unbounded parent branch.** The ADR must define how star tracks
  behave when the parent bound on an axis is unbounded. Options to
  compare explicitly: star tracks act as zero-minimum intrinsic tracks
  unless fixed content supplies size, or layout raises a
  Grid-specific unbounded-star error analogous to Phase 4's
  ScrollView unbounded-axis precedent. Because star weights are
  positive integers, an all-zero weight sum is rejected before layout
  in either option.

### DD-M3-P5-005 — Arrange algorithm and visual-layer contract

Grid arranges each logical cell child into the rectangle formed by its
resolved row / column span. In Surface A, that span comes from
parent-scoped metadata on the content child. In Surface A2, it comes
from parent-scoped metadata on the `Cell` wrapper. In structural and
hybrid structural-row surfaces, it comes from the surrounding `Row` /
`Cell` structure and any span metadata on `Cell`.

Sub-issues:

- **Child alignment inside cell.** Stretch by default in every
  surface family. Recommended: admit per-child alignment overrides
  (`h-align` / `v-align`) in Phase 5, because practical Grid layouts
  need centered text, right-aligned actions, and icon placement
  without wrapper hacks. The defaults remain stretch / stretch for
  stable cells. Carrier varies by surface:
  - **Surface A:** alignment is parent-scoped child metadata
    on the content widget (`Text { h-align: center ... }`),
    co-located with `row` / `column`.
  - **Surface A2:** alignment lives on `Cell` (`Cell { row: 0
    column: 1 h-align: center Text { ... } }`), keeping all Grid child
    contract metadata on the wrapper and leaving content widgets clean.
  - **Pure structural / Surface D / definition-node families:** alignment lives
    on `Cell` (`Cell { h-align: center Text { ... } }`), not on the
    content widget. This keeps `Cell` as the carrier of all
    Grid-specific child contract attributes and avoids splitting
    grid metadata between `Cell` (span) and content widget
    (alignment).
- **Overflow and clipping.** The ADR should separate three concepts:
  per-cell clipping, Grid outer-bounds clipping, and intentional
  overlay. Per-cell clipping is out of scope. Same-cell / span
  occupancy overlap remains invalid, and ZStack remains the surface
  for intentional overlay. The Options table should compare the
  current pure-layout behavior (child paint follows existing parent /
  clip rules and may overflow) with a Grid outer-bounds clip that
  prevents paint from escaping the Grid's own rectangle without
  turning Grid into a ScrollView-style viewport.
- **Visual ownership.** Grid should not introduce an intermediate
  Visual. It is a pure layout container like WrapPanel, not a
  viewport / translation primitive like ScrollView.
- **Production root shape.** Verification must include at least one
  integration fixture whose parent shape matches production gallery
  root usage, per [constraints.md §1](./constraints.md#1-integration-test-fixture-parent-shape-は-production-root-shape-を必ずカバーする).

### DD-M3-P5-006 — IR-loader defense-in-depth invariants

The ADR must decide which Grid invariants are dual-gated by
`wasamoc check` and runtime `validate()`. Recommended gate ownership:

| Invariant | Gate |
|---|---|
| Grid has at least one row and at least one column | Structural; both `wasamoc check` and runtime `validate()` |
| The chosen author surface lowers successfully into `TrackSize` sequences plus logical cell membership | **Phase 5 new Grid-surface gate**; parser diagnostics are primary where syntax permits, while `wasamoc check` and runtime `validate()` remain defense-in-depth safety nets |
| Track sizes are positive where required; star weights are positive integers (`0*`, negative weights, and all-zero star sums are malformed) | Value range; both gates |
| Placement indices or structural row / cell membership are in range of the declared track count | Cross-attribute / structural value range; both gates |
| Spans are positive and do not exceed declared track count | Cross-attribute / structural value range; both gates |
| Same-cell conflicts / overlapping spans are rejected | Cross-child structural check; both gates |

The second invariant intentionally hides different work per surface:
Surface A lowers `columns:` plus per-content placement metadata; Surface
A2 lowers `columns:` plus per-`Cell` placement metadata; Surface B lowers
canonical-row inference plus structural Cells; Surface D lowers
`columns:` plus structural Cell order; Surface C lowers `ColumnDefs` /
`RowDefs` plus structural Cell order. The ADR should make that lowering
shape explicit before treating the validation burden as equal.

---

### Out of scope (to be carried in the ADR's Out-of-scope section)

- Same-cell overlap / overlay. Phase 6 ZStack owns overlay.
- Responsive breakpoint grammar, media queries, and named areas.
- General list / collection syntax beyond the minimum needed to
  express row and column tracks.
- Grid-level clip attributes and per-cell clipping. A fixed Grid
  outer-bounds clip is an ADR option for DD-M3-P5-005, but no
  author-facing `clip:` attribute or per-cell clip surface is in
  Phase 5 scope.
- Bindable Grid track definitions or bindable child placement.
  Phase 5 should keep Grid attributes constant-only unless owner
  explicitly expands scope, but DD-M3-P5-001 / 002 should note whether
  the chosen Grid surface leaves a future path for bindable
  track pieces.
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

To avoid confusion with Surface D, framing-decision labels use the
`FD-*` prefix in this section.

### FD-A. DD slate completeness

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

### FD-B. Pre-doc-discipline check

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

### FD-C. Verification strategy

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

### FD-D. Non-root Shrink parent with Fill child

[constraints.md §2](./constraints.md#2-non-root-の-shrink-container-が-fill-子を持つ場合の挙動)
requires Phase 5 to make this design space explicit.

Draft recommendation: Phase 5 keeps the existing convention.
Window-root `WidgetNode::run_layout_as_window_root` may force the
root to Fill / Fill, but non-root Shrink containers with Fill children
continue to follow the existing
`degenerate_fill_in_shrink_parent_clamps_to_zero` behavior. Grid does
not create a Grid-specific exception.

If owner wants Grid to pierce that convention, the change should be
recorded as a broader layout DD rather than hidden inside Grid.

### FD-E. R1 Window-title wiring owning phase

[constraints.md §4](./constraints.md#4-r1-window-title-wiring-の-owning-phase-割当--phase-5-pre-doc-内で必須完了)
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

- update [process/milestone-3/plan.md](../../plan.md) Phase 6 Notes with
  "M3-Phase 4 R1 (Window title wiring) owning phase";
- record in the Phase 5 ADR that R1 is Phase 5 thesis scope **out**;
  and
- cross-reference the assignment from the Phase 4 R1 residual entry
  or the Phase 5 implementation log, depending on the chosen commit
  shape.

### FD-F. Phase 4 residual scan — disposition

The Phase 4 implementation plan's
[Out-of-phase residuals](../../phase-4/implementation/plan.md#out-of-phase-residuals)
contains one open residual at Phase 5 pre-doc time:

- **R1 — Gallery host Window title wiring.** Disposition: assign to
  M3-Phase 6 per FD-E. Phase 5 records the assignment
  but does not implement it.

Related Phase 3 residual context:

- **Phase 3 R2 — `sync_visuals` coverage gap.** Closed inside Phase
  4 T4, with evidence recorded in
  [Phase 4 implementation plan](../../phase-4/implementation/plan.md#t4--windows-runtime-layout-and-visual-evidence-including-r2-closure).
  No Phase 5 action.

Phase 5 does not create an additional residual bucket during pre-doc.
If Grid implementation discovers real-but-out-of-scope issues, they
must be recorded under the Phase 5 implementation log or handoff with owner phase,
resolution condition, and deadline, following the R1 pattern.

### FD-G. Upstream-document revision timing (two sync moments)

Phase 5 follows the same two-moment document rule as Phases 2-4:

**Moment 1 — ADR Accepted commit set (design-spec draft).** The
following documents are expected to land as separate commits by review
concern, not as a single "Moment 1" bundle:

- `process/milestone-3/phase-5/decisions/preamble.md` and
  `process/milestone-3/phase-5/decisions/dd-*.md` — ADR
  `Status: Accepted` flip.
- `docs/dsl_spec.md` — new §4.12 Grid chapter as a design-spec
  draft, plus the §4.4 widget registry row. The chapter records
  `Phase status: M3-Phase 5 design accepted; implementation
  pending`.
- `docs/architecture.md` — expected to receive an entry for the Grid
  IR variant, including the new `TrackSize` domain type and the
  accepted child-membership representation (parent-scoped placement
  metadata on content children, placement metadata on `Cell` wrappers,
  structural rows / cells, hybrid Grid columns with structural rows, or
  definition-node-backed structural rows), mirroring how Box / WrapPanel
  / ScrollView are documented. It may also add a layout-engine paragraph
  if the accepted track-resolution algorithm warrants durable cross-phase
  commentary.
- `docs/abi_spec.md` — untouched in the recommended path. Grid adds
  no host-facing ABI surface and no `PropertyValue` tag.
- `process/milestone-3/plan.md` — R1 owning-phase note on the Phase 6 row
  if owner accepts FD-E. This is a plan-note edit, not
  part of the Grid thesis commit.
- `process/milestone-3/phase-5/implementation/preamble.md` /
  `process/milestone-3/phase-5/implementation/plan.md` — implementation
  planning opened after ADR acceptance, with the final-step
  retrospective split represented in the task plan from the start.

**Moment 2 — Phase close commit set (implementation re-sync).**

- `docs/dsl_spec.md` §4.12 — section marker flips to "closed;
  implementation-synced", plus any corrections required if the
  design draft and implementation diverged.
- `docs/architecture.md` — re-sync if implementation created a
  durable runtime / layout-engine convention not already captured at
  Moment 1.
- `process/milestone-3/plan.md` — Phase 5 row status and ADR / implementation
  pointers update when Phase 5 closes.
- `process/milestone-3/phase-5/implementation/log.md` and
  `process/milestone-3/phase-5/retrospectives/phase-end.md` — close
  evidence, CI pointer, residuals, and lifecycle transition recorded.
- Step retro `phase-sync` items must all close into `doc-folded` /
  `carry-forward` / `local-only` at Moment 2; no open
  `phase-sync` item survives phase close.

### FD-H. Phase 5 visible proof

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
star track, child membership, and spanning without requiring
ScrollView / ZStack composition.

Composition with existing primitives is allowed but not the central
proof. The minimum visible proof should not require
`ScrollView { Grid { ... } }` or `Grid { ScrollView { ... } }` to be
accepted. If the implementation naturally includes such composition,
it may be tested as an extra confidence check, but Phase 5's A2
evidence is Grid track sizing / placement / spanning itself.

### FD-I. GUI smoke responsibility separation

Phase 5 should preserve the Phase 4 lesson: automated build / launch
evidence and owner-visible correctness are distinct gates.

Draft recommendation: the implementation plan includes a dedicated
owner-manual GUI smoke step if the Grid gallery slice changes visible
layout enough that automated tests cannot fully judge it. The final
mechanical close step should run only after that visible proof is
green or an explicit owner fail observation has been recorded and
resolved.

### FD-J. Live-note re-evaluation triggers — handling

The `docs/notes/*` live notes are settled upfront so the ADR Inputs
section can cite their disposition rather than re-deciding:

- **[architectural-family.md](../../../../docs/notes/architectural-family.md) — stays
  consumed.** Grid is a built-in layout primitive in the established
  tree-with-bindings family.
- **[layout-engine.md](../../../../docs/notes/layout-engine.md) — partial fire.**
  Grid directly exercises §3.1 DPI / logical-pixel discipline and
  §3.4 cache invalidation. Disposition: keep layout in logical
  `f32` coordinates; no pixel snapping; no subtree dirty propagation
  in Phase 5. The 1,000-node threshold remains unfired because the
  gallery slice is fixed-child and well below that scale.
- **[dsl-grammar.md](../../../../docs/notes/dsl-grammar.md) — fired narrowly.**
  DD-M3-P5-001 now compares multiple Grid-specific surfaces:
  first-class track-list values with direct child placement, placed
  `Cell` wrappers, hybrid Grid columns + structural rows, structural
  `Row` / `Cell`, and definition-node variants. None of these should
  open general list /
  collection grammar by accident, but each has parser implications.
  Q1 widget ids, Q3 iteration grammar, and Q5 expression grammar
  remain Phase 6 / Phase 7+ unless Grid implementation unexpectedly
  needs them.
- **[component-extension-model.md](../../../../docs/notes/component-extension-model.md) —
  partial fire.** Grid is built-in, not a user-defined layout
  component, but DD-M3-P5-001 / 003 now explicitly compare a
  parent-scoped child metadata surface, a placed-`Cell` wrapper surface,
  pure structural `Row` / `Cell`, hybrid Grid columns with structural
  rows, and definition-node structural rows. The
  component-extension-model implication differs by surface:
  - If Surface A is selected, `row` / `column` / span /
    alignment become the first built-in parent-scoped child metadata
    precedent, foreshadowing a future attached-property-style
    mechanism for custom layouts.
  - If the placed-`Cell` wrapper family is selected, the precedent is
    narrower: Grid-specific child contract attributes are contained in a
    Grid-owned wrapper node while track sharing remains parent-level.
    This avoids placing `row` / `column` on arbitrary widgets but still
    leaves a path to parent-scoped wrappers for custom layouts.
  - If Surface B is selected, the precedent set instead is
    "custom layouts may introduce structural child node kinds with
    their own attribute schemas" (analogous to how `Row` / `Cell`
    have Grid-specific sizing and span attributes). This is a
    different extension trajectory; neither is inherently better,
    and the framing must not prejudge which one Phase 5 sets.
  - If Surface D is selected, the precedent is mixed: a parent-owned
    track declaration coexists with structural child nodes. Future
    custom layouts could use this "parent config + structural content"
    pattern without introducing explicit child coordinates.
  - If Surface C is selected, it adds a second structural precedent:
    custom containers may have non-visual definition nodes such as
    `ColumnDefs` / `RowDefs` that declare container shape separately
    from visible content rows.
- **[typed-value-evaluator.md](../../../../docs/notes/typed-value-evaluator.md) —
  unfired for Phase 5 execution, noted for future compatibility.**
  Grid track / membership attributes remain constant-only unless
  owner explicitly expands scope. No item context, no bindable track
  definitions, and no new typed evaluator value are introduced in the
  recommended Phase 5 execution path, but DD-M3-P5-001 / 002 should
  avoid choosing a surface that makes future bindable track pieces
  unnatural — this applies to every surface family (bindable
  `columns:` value in the track-list / hybrid-column families, bindable
  `Cell { width: ... }` in pure structural, bindable `ColumnDef { width: ... }` in
  definition-node). Phase 7 may reopen item-context pressure.
- **[workspace-layout.md](../../../../docs/notes/workspace-layout.md) — unfired.** No new
  crate is expected.
- **[verification-environments.md](../../../../docs/notes/verification-environments.md) /
  [headless-verification.md](../../../../docs/notes/headless-verification.md) — fired via
  inherited discipline.** Phase 5 keeps fail-rather-than-silently-skip
  gates and separates headless evidence from owner-visible GUI smoke.
- **[process-rules-ssot.md](../../../cross-milestone/decisions/process-rule-ssot.md) — relevant.**
  Phase 5 keeps the ADR / progress / retrospective role split and the
  step item 10 disposition vocabulary.
- **[release-distribution.md](../../../../docs/notes/release-distribution.md) — unfired.**
  Phase 5 introduces no release / packaging surface.

### FD-K. Grid mental model anchor in dsl_spec

The Moment 1 `docs/dsl_spec.md` Grid chapter should start with a
short mental-model anchor before the algorithm. Until DD-M3-P5-001 is
accepted, the framing should present five equally serious author
models:

- rows and columns define tracks (their location depends on the surface);
- fixed tracks take definite space first;
- star tracks divide remaining bounded space by weight;
- in a **track-list + placement** surface, tracks are declared on
  Grid and children carry parent-scoped metadata that places and
  aligns them within resolved cells;
- in a **track-list + placed Cell** surface, tracks are declared on
  Grid and `Cell` wrappers carry placement / span / alignment metadata,
  leaving content widgets free of Grid-specific attributes;
- in a **pure structural Row / Cell** surface, document structure
  assigns children to cells, sizing metadata lives on `Row` /
  `Cell`, and the ADR must define how column widths are reconciled
  across rows;
- in a **Grid columns + structural rows** surface, Grid declares shared
  columns once while `Row` / `Cell` document structure assigns content
  to those columns and carries row heights;
- in a **definition-node + structural rows** surface, hoisted
  `ColumnDefs` / `RowDefs` definition children on Grid declare shared
  track sizes, while content rows mirror the visible structure;
- in every surface, children occupy exactly one cell rectangle or
  one rectangular span; and
- Grid arranges children into resolved rectangles and does not
  provide intentional overlay. Per-cell clipping is out of scope;
  outer-bounds clipping is an ADR option, not a framing-settled rule.

This mirrors the short mental-model anchors added for WrapPanel and
ScrollView and gives external readers a stable entry point before the
track-resolution details.

**Ecosystem contrast (one bullet each).** Grid's surface intersects
several incompatible mental models:

- **WPF `Grid`.** WPF uses `RowDefinition` / `ColumnDefinition` and
  attached `Grid.Row` / `Grid.Column` properties. This maps to the
  track-list + direct child-placement family; the placed-`Cell` variant
  keeps WPF-like coordinates but routes them through a wrapper instead
  of an attached-property-like child metadata surface. Wasamo should not
  assume either family just because WPF is familiar; if chosen, it
  should be chosen for explicit placement and spanning power, not by
  inertia.
- **CSS Grid.** CSS Grid has named lines, template areas, auto-flow,
  fractional units, minmax, gap, and dense placement. Wasamo Phase 5
  can borrow the idea of tracks plus placed children, but CSS is not a
  reason to reject a structural Row / Cell `.ui` if owner prefers a
  table-like authoring model.
- **Table / form builders.** Many UI builders expose rows and cells
  structurally. This maps to the structural `Row` / `Cell` family:
  it is more verbose for irregular placement but more direct for
  forms, settings panes, and fixed gallery slices. The shared
  track-sizing problem (column widths consistent across rows) is the
  load-bearing tension here — pure structural surfaces leave it
  emergent and must add reconciliation rules; definition-node
  variants solve it by construction at the cost of an extra surface
  layer. That tradeoff is not resolved in framing.
- **Cell-wrapper builders.** Some APIs make an explicit item / cell
  wrapper the carrier of layout metadata. This maps to Surface A2: it
  preserves parent-level tracks while avoiding `row` / `column` on
  arbitrary content widgets. The cost is an extra wrapper without the
  row-structure readability of B / D / C.
- **Form builders with shared columns.** Some form-oriented APIs keep
  column definitions at the container level while rows remain structural.
  This maps to Surface D: it moves shared columns to parent config while
  keeping rows structural, but it is asymmetric (`columns:` on Grid,
  `height:` on Row) and weaker for irregular coordinate placement than
  A / A2.
- **Jetpack Compose / SwiftUI grids.** Those ecosystems often model
  adaptive or lazy grids that generate children from data. Wasamo
  Grid does not follow that model: it is a static 2D composition
  primitive, and the collection-driven surface in M3 is WrapPanel +
  the iteration grammar, not Grid (see the "Downstream commitments
  grounded in Phase 5" paragraph and [spec.md](../../requirements/spec.md)).
  Grid generating children from data is neither a Phase 5 nor a Phase 7
  concern; it is only a non-foreclosed post-M3 possibility.
- **ZStack / overlay models.** Grid does not provide intentional
  overlay. Paint overflow may be visible if a child is larger than
  its cell, but two children may not deliberately occupy the same
  cell; Phase 6 ZStack owns overlay.

---

## Author-facing `.ui` surface options

This section is illustrative input for owner alignment, not final
grammar. It is intentionally present in the framing note because
owner has explicitly required Phase 5 not to treat the WPF / CSS Grid
family as the default. The five surface families below are shown
with parallel examples (sidebar + star, weighted star + alignment,
spanning) so the ADR can compare them on the five owner-specified
axes — `.ui` author taste, spanning, shared track sizing, future
iteration, component-extension-model — symmetrically rather than
through asymmetric coverage.

Expanded owner-readable notes for each surface live in
[surface-options/](./surface-options/README.md). Those files are
supplemental requirements notes; this framing document remains the
phase requirements SSOT.

### Surface A — track-list + placed children

Tracks live on `Grid` as Grid-specific first-class track-list values.
Each child carries parent-scoped placement metadata. Shared track
sizing is automatic by construction: there is exactly one canonical
`columns:` / `rows:` declaration.

**Example A1 — fixed sidebar + star content:**

```wasamo-ui
Grid {
  columns: 180 *
  rows: *

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

**Example A2 — weighted star + alignment:**

```wasamo-ui
Grid {
  columns: 1* 2* 1*
  rows: 72

  Text {
    row: 0
    column: 0
    text: "Back"
  }

  Text {
    row: 0
    column: 1
    h-align: center
    v-align: center
    text: "Summer Trip"
  }

  Text {
    row: 0
    column: 2
    h-align: end
    text: "Share"
  }
}
```

If the Grid receives 800 px of width and no fixed columns exist, the
resolved column widths are proportional to `1 : 2 : 1`. DD-M3-P5-004
uses deterministic `f32` prefix boundaries and no integer pixel snap.

**Example A3 — spanning:**

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 220 120

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
The two lower children occupy separate cells.

Strength: Surface A is the most compact explicit-coordinate surface and
keeps shared tracks canonical on Grid. Cost: Grid-specific placement /
alignment metadata appears on content widgets.

### Surface A2 — track-list + placed Cell wrapper

Tracks live on `Grid` as Grid-specific first-class track-list values,
as in Surface A. Unlike Surface A, placement / span / alignment metadata
lives on a Grid-owned `Cell` wrapper. The content widget is the single
child of `Cell`.

**Example A2.1 — fixed sidebar + star content:**

```wasamo-ui
Grid {
  columns: 180 *
  rows: *

  Cell {
    row: 0
    column: 0
    Box {
      fill: #243447ff
      Text { text: "Albums" }
    }
  }

  Cell {
    row: 0
    column: 1
    Box {
      fill: #f5f7faff
      Text { text: "Selected album" }
    }
  }
}
```

The track-resolution semantics match Surface A. The authoring
difference is that Grid metadata is isolated on `Cell`, not on `Box`.

**Example A2.2 — weighted star + alignment:**

```wasamo-ui
Grid {
  columns: 1* 2* 1*
  rows: 72

  Cell {
    row: 0
    column: 0
    Text { text: "Back" }
  }

  Cell {
    row: 0
    column: 1
    h-align: center
    v-align: center
    Text { text: "Summer Trip" }
  }

  Cell {
    row: 0
    column: 2
    h-align: end
    Text { text: "Share" }
  }
}
```

Alignment lives on `Cell`, so future content widgets do not need to
learn Grid-specific alignment attributes merely because they appear
inside Grid.

**Example A2.3 — spanning:**

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 220 120

  Cell {
    row: 0
    column: 0
    column-span: 2
    Box {
      fill: #336699cc
      Text { text: "Featured photo" }
    }
  }

  Cell {
    row: 1
    column: 0
    Box {
      fill: #88aa55cc
      Text { text: "Detail A" }
    }
  }

  Cell {
    row: 1
    column: 1
    Box {
      fill: #aa6655cc
      Text { text: "Detail B" }
    }
  }
}
```

Strength: Surface A2 keeps A's irregular-placement power and automatic
shared track sizing while containing Grid metadata on `Cell`. Cost: it
adds a wrapper and still lacks B / D / C's row-structure readability.

### Surface B — pure structural Row / Cell

Document structure mirrors the visible rows and cells. Sizing
metadata lives on `Row` / `Cell` directly. No widget receives `row` /
`column` attributes.

The load-bearing question for this family is **shared track sizing**:
since each `Row` declares its own `Cell` widths, the ADR must commit
to one of:

- **B-reject:** widths declared in row 2+ must equal widths declared
  by a canonical non-spanning row; mismatches are malformed. The
  canonical row is the first row whose Cells cover the full column
  count without spans. This handles header / footer rows that are only
  spanning Cells.
- **B-first:** widths are only read from the first row, even if that
  row contains spans; later row widths are ignored or warned on. Less
  safe and ambiguous when the first row is a spanning header.
- **B-independent:** each row is sized independently. This is no
  longer Grid semantics (it is a stack of HStacks) and should not be
  considered a Grid surface; recorded here only to be rejected.

The examples below assume **B-reject** only to make Surface B
well-defined for comparison. This is not a recommendation for B; it is
the minimum rule needed before B can be compared fairly with A, A2, D,
and C.
In these examples, non-spanning Cells in the canonical row carry
`width`; spanning Cells omit `width` and are sized by the columns they
span.

**Example B1 — fixed sidebar + star content:**

```wasamo-ui
Grid {
  Row {
    height: *

    Cell {
      width: 180
      Box {
        fill: #243447ff
        Text { text: "Albums" }
      }
    }

    Cell {
      width: 1*
      Box {
        fill: #f5f7faff
        Text { text: "Selected album" }
      }
    }
  }
}
```

The first row is also the canonical non-spanning row, so it declares
two columns (180 px, star). Any additional non-spanning row must
declare the same two widths or be rejected by `wasamoc check`.

**Example B2 — weighted star + alignment:**

```wasamo-ui
Grid {
  Row {
    height: 72

    Cell {
      width: 1*
      Text { text: "Back" }
    }

    Cell {
      width: 2*
      h-align: center
      v-align: center
      Text { text: "Summer Trip" }
    }

    Cell {
      width: 1*
      h-align: end
      Text { text: "Share" }
    }
  }
}
```

Alignment lives on `Cell`, not on `Text`. This keeps Grid-specific
child contract attributes consolidated on `Cell`.

**Example B3 — spanning:**

```wasamo-ui
Grid {
  Row {
    height: 220

    Cell {
      column-span: 2
      Box {
        fill: #336699cc
        Text { text: "Featured photo" }
      }
    }
  }

  Row {
    height: 120

    Cell {
      width: 1*
      Box {
        fill: #88aa55cc
        Text { text: "Detail A" }
      }
    }

    Cell {
      width: 1*
      Box {
        fill: #aa6655cc
        Text { text: "Detail B" }
      }
    }
  }
}
```

The first row contains only a spanning Cell, so it cannot be the
canonical width declaration. Under B-reject, the second row is the
canonical non-spanning row and declares two `1*` columns; the first
row's spanning Cell is sized and validated against that inferred
column vector.
This inference rule is the main fragility of Surface B in
shared-track-sizing terms.

Strength: Surface B is the closest to a pure table-like document
structure. Cost: shared column sizing requires a reconciliation rule
such as B-reject before it can behave as Grid rather than independent
rows.

### Surface D — Grid columns + structural rows

Shared column tracks live on `Grid` as a first-class `columns:` value.
Rows and cells remain structural. Row heights live on `Row`; cells do
not carry `width` because they consume the Grid-level columns in
document order.

**Example D1 — fixed sidebar + star content:**

```wasamo-ui
Grid {
  columns: 180 *

  Row {
    height: *

    Cell {
      Box {
        fill: #243447ff
        Text { text: "Albums" }
      }
    }

    Cell {
      Box {
        fill: #f5f7faff
        Text { text: "Selected album" }
      }
    }
  }
}
```

Column sharing comes from a canonical `columns:` declaration, while
membership comes from structural rows and cells.

**Example D2 — weighted star + alignment:**

```wasamo-ui
Grid {
  columns: 1* 2* 1*

  Row {
    height: 72

    Cell {
      Text { text: "Back" }
    }

    Cell {
      h-align: center
      v-align: center
      Text { text: "Summer Trip" }
    }

    Cell {
      h-align: end
      Text { text: "Share" }
    }
  }
}
```

Alignment lives on `Cell`; column weights live on Grid.

**Example D3 — spanning:**

```wasamo-ui
Grid {
  columns: 1* 1*

  Row {
    height: 220

    Cell {
      column-span: 2
      Box {
        fill: #336699cc
        Text { text: "Featured photo" }
      }
    }
  }

  Row {
    height: 120

    Cell {
      Box {
        fill: #88aa55cc
        Text { text: "Detail A" }
      }
    }

    Cell {
      Box {
        fill: #aa6655cc
        Text { text: "Detail B" }
      }
    }
  }
}
```

Strength: Surface D keeps structural rows while declaring shared columns
once. Cost: columns are parent-level while rows are structural, so the
surface is intentionally asymmetric. The asymmetry is load-bearing,
not accidental: form / settings-pane use cases typically want shared
columns fixed at design time while rows grow with data. Adding a
parallel `rows:` attribute to Grid (a hypothetical "D-with-rows")
does not refine Surface D — it collapses D into either Surface A2
(if `Row` becomes ceremonial) or motivates Surface C (if both axes
should be hoisted symmetrically). See the
[Surface D asymmetry-is-intentional note](./surface-options/surface-d-grid-columns-structural-rows.md#asymmetry-is-intentional)
for the full critical analysis.

### Surface C — definition nodes + structural rows

Hoisted `ColumnDefs` / `RowDefs` definition children on Grid declare shared
track sizes once. Content rows mirror the visible structure but carry
no sizing. This is the only surface where shared track sizing is
solved by construction without conflating row / column declarations
with content.

**Example C1 — fixed sidebar + star content:**

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 180 }
    ColumnDef { width: 1* }
  }

  RowDefs {
    RowDef { height: 1* }
  }

  Row {
    Cell {
      Box {
        fill: #243447ff
        Text { text: "Albums" }
      }
    }

    Cell {
      Box {
        fill: #f5f7faff
        Text { text: "Selected album" }
      }
    }
  }
}
```

**Example C2 — weighted star + alignment:**

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 1* }
    ColumnDef { width: 2* }
    ColumnDef { width: 1* }
  }

  RowDefs {
    RowDef { height: 72 }
  }

  Row {
    Cell {
      Text { text: "Back" }
    }

    Cell {
      h-align: center
      v-align: center
      Text { text: "Summer Trip" }
    }

    Cell {
      h-align: end
      Text { text: "Share" }
    }
  }
}
```

**Example C3 — spanning:**

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 1* }
    ColumnDef { width: 1* }
  }

  RowDefs {
    RowDef { height: 220 }
    RowDef { height: 120 }
  }

  Row {
    Cell {
      column-span: 2
      Box {
        fill: #336699cc
        Text { text: "Featured photo" }
      }
    }
  }

  Row {
    Cell {
      Box {
        fill: #88aa55cc
        Text { text: "Detail A" }
      }
    }

    Cell {
      Box {
        fill: #aa6655cc
        Text { text: "Detail B" }
      }
    }
  }
}
```

Track count is fixed by the `ColumnDefs` definition. Spans are validated
against that fixed count regardless of which row contains them.

Strength: Surface C gives rows and columns symmetrical explicit
definition nodes and keeps content rows structural. Cost: it introduces
the most boilerplate and a non-visual definition-node pattern.

### Gallery slice candidate — parallel across surfaces

A Phase 5 gallery proof could add a fixed-child Grid slice without
replacing the existing Box / WrapPanel / ScrollView proof. The same
visible result is shown below in all five surface families so the
owner-facing decision is which `.ui` form should become the public
surface, not which surface is even capable of expressing the proof.

**Gallery slice in Surface A:**

```wasamo-ui
Grid {
  columns: 96 1* 96
  rows: 64 1* 120

  Text {
    row: 0
    column: 0
    column-span: 3
    h-align: center
    v-align: center
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

**Gallery slice in Surface A2:**

```wasamo-ui
Grid {
  columns: 96 1* 96
  rows: 64 1* 120

  Cell {
    row: 0
    column: 0
    column-span: 3
    h-align: center
    v-align: center
    Text { text: "Gallery" }
  }

  Cell {
    row: 1
    column: 0
    Box { fill: #2f4050ff Text { text: "Prev" } }
  }

  Cell {
    row: 1
    column: 1
    Box { fill: #3f7caccc Text { text: "Preview" } }
  }

  Cell {
    row: 1
    column: 2
    Box { fill: #2f4050ff Text { text: "Next" } }
  }

  Cell {
    row: 2
    column: 0
    column-span: 3
    Text { text: "Metadata and actions" }
  }
}
```

**Gallery slice in Surface B (B-reject variant):**

```wasamo-ui
Grid {
  Row {
    height: 64

    Cell {
      column-span: 3
      h-align: center
      v-align: center
      Text { text: "Gallery" }
    }
  }

  Row {
    height: 1*

    Cell {
      width: 96
      Box { fill: #2f4050ff Text { text: "Prev" } }
    }

    Cell {
      width: 1*
      Box { fill: #3f7caccc Text { text: "Preview" } }
    }

    Cell {
      width: 96
      Box { fill: #2f4050ff Text { text: "Next" } }
    }
  }

  Row {
    height: 120

    Cell {
      column-span: 3
      Text { text: "Metadata and actions" }
    }
  }
}
```

The shared track sizing fragility is visible: the column widths
`96 1* 96` are declared by the middle row and adopted by the spanning
header / footer rows. If any row's column widths disagree, `wasamoc
check` must reject the Grid.
This continues the B-reject assumption used only to make Surface B
well-defined for comparison, not a framing-time recommendation.

**Gallery slice in Surface D:**

```wasamo-ui
Grid {
  columns: 96 1* 96

  Row {
    height: 64

    Cell {
      column-span: 3
      h-align: center
      v-align: center
      Text { text: "Gallery" }
    }
  }

  Row {
    height: 1*

    Cell { Box { fill: #2f4050ff Text { text: "Prev" } } }
    Cell { Box { fill: #3f7caccc Text { text: "Preview" } } }
    Cell { Box { fill: #2f4050ff Text { text: "Next" } } }
  }

  Row {
    height: 120

    Cell {
      column-span: 3
      Text { text: "Metadata and actions" }
    }
  }
}
```

In Surface D, the column vector `96 1* 96` is declared once on Grid,
while row membership remains structural.

**Gallery slice in Surface C:**

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 96 }
    ColumnDef { width: 1* }
    ColumnDef { width: 96 }
  }

  RowDefs {
    RowDef { height: 64 }
    RowDef { height: 1* }
    RowDef { height: 120 }
  }

  Row {
    Cell {
      column-span: 3
      h-align: center
      v-align: center
      Text { text: "Gallery" }
    }
  }

  Row {
    Cell { Box { fill: #2f4050ff Text { text: "Prev" } } }
    Cell { Box { fill: #3f7caccc Text { text: "Preview" } } }
    Cell { Box { fill: #2f4050ff Text { text: "Next" } } }
  }

  Row {
    Cell {
      column-span: 3
      Text { text: "Metadata and actions" }
    }
  }
}
```

Each example exercises fixed tracks, star sizing, and spanning in a
shape that later Phase 6 can overlay with ZStack. The slice stays a
fixed-child Grid: Grid is not an M3 iteration target (the thumbnail
collection is WrapPanel-backed per [spec.md](../../requirements/spec.md)),
so Phase 7 does not populate this Grid through iteration. None
expresses overlay inside Grid.

### Invalid shapes — parallel across surfaces

Phase 5 must reject the same logical errors in every surface family.
The shapes below show the same error class expressed in each surface
so the ADR can compare diagnostic surfaces flatly.

**Duplicate cell claim:**

Surface A:

```wasamo-ui
Grid {
  columns: 1*
  rows: 1*

  Box { row: 0 column: 0 fill: #336699cc }
  Box { row: 0 column: 0 fill: #aa6655cc }
}
```

Surface A2: two widgets inside a placed `Cell` body. Unless `Cell` is
defined as a single-child wrapper, this would need a child-stacking rule:

```wasamo-ui
Grid {
  columns: 1*
  rows: 1*

  Cell {
    row: 0
    column: 0
    Box { fill: #336699cc }
    Box { fill: #aa6655cc }
  }
}
```

Surface B / D / C: the same single-`Cell` body problem appears inside a
structural row:

```wasamo-ui
Grid {
  Row {
    Cell {
      Box { fill: #336699cc }
      Box { fill: #aa6655cc }
    }
  }
}
```

Phase 5 should define `Cell` as single-child in A2 and the structural
families, so this shape is rejected at parse time as "`Cell` accepts
exactly one content child."

**Span exceeds declared track count:**

Surface A:

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 1*

  Box {
    row: 0
    column: 1
    column-span: 2
    fill: #336699cc
  }
}
```

Surface A2:

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 1*

  Cell {
    row: 0
    column: 1
    column-span: 2
    Box { fill: #336699cc }
  }
}
```

Surface B (B-reject):

```wasamo-ui
Grid {
  Row {
    Cell { width: 1* Box { fill: #2f4050ff } }
    Cell { width: 1* column-span: 2 Box { fill: #336699cc } }
  }
}
```

Surface D:

```wasamo-ui
Grid {
  columns: 1* 1*

  Row {
    Cell { Box { fill: #2f4050ff } }
    Cell { column-span: 2 Box { fill: #336699cc } }
  }
}
```

Surface C:

```wasamo-ui
Grid {
  ColumnDefs {
    ColumnDef { width: 1* }
    ColumnDef { width: 1* }
  }
  RowDefs {
    RowDef { height: 1* }
  }

  Row {
    Cell { Box { fill: #2f4050ff } }
    Cell { column-span: 2 Box { fill: #336699cc } }
  }
}
```

All five are rejected by `wasamoc check` and runtime IR validation:
the span consumes columns 1 and 2, but only columns 0 and 1 are
declared.

**Shared track sizing mismatch (Surface B only):**

```wasamo-ui
Grid {
  Row {
    Cell { width: 180 Box { fill: #2f4050ff } }
    Cell { width: 1* Box { fill: #3f7caccc } }
  }

  Row {
    Cell { width: 200 Box { fill: #88aa55cc } }
    Cell { width: 1* Box { fill: #aa6655cc } }
  }
}
```

Under B-reject this is rejected because row 1 declares `180`-wide
column 0 and row 2 declares `200`-wide column 0. Surface A, Surface A2,
Surface D, and Surface C cannot express this error in the first place —
`columns:` or `ColumnDefs { ... }` declares each column exactly once. This is a
diagnostic and authoring distinction the ADR must weigh if the
structural authoring model remains attractive.

**Implicit placement under Surface A / A2 (degenerate omission):**

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 1*

  Box { column: 0 fill: #336699cc }
}
```

Surface A2:

```wasamo-ui
Grid {
  columns: 1* 1*
  rows: 1*

  Cell {
    column: 0
    Box { fill: #336699cc }
  }
}
```

This is meaningful in Surface A and A2: a placed content child or placed
`Cell` omits `row` in a multi-child Grid context. Phase 5 should not
silently auto-place the child unless DD-M3-P5-003 also defines a
document-order placement algorithm. Surfaces B, D, and C avoid this error
by making membership explicit in the document tree.

If owner accepts `auto` / intrinsic tracks in DD-M3-P5-002, a later
draft can add an example such as a metadata column sized by content
(`columns: auto 1*` in Surface A / A2 / D, `Cell { width: auto }` in B,
or `ColumnDef { width: auto }` in C). The conservative recommendation
defers `auto`, so the examples above stay within fixed + weighted-
star semantics.

---

## Inputs absorbed

### From [constraints.md](./constraints.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §1 integration fixture parent shape must cover production root shape | Direct input | FD-C; DD-M3-P5-005 verification note |
| §2 non-root Shrink container + Fill child design space | Design-space decision | FD-D; draft recommendation is status quo |
| §3 `scroll_y` Signal drift | Out of scope for Phase 5 | Out-of-scope section; M4 handoff only |
| §4 R1 Window-title wiring owning phase | Required pre-doc assignment | FD-E; FD-F; milestone plan Phase 6 note after owner alignment |
| §5 final-step retrospective split | Process rule | Opening assumptions; Phase 5 implementation plan template |

### From [process/milestone-3/plan.md](../../plan.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §Acceptance criteria — A2 | Constraint | Acceptance restatement; DD-M3-P5-001..006 |
| §Acceptance criteria — A11 | Operational rule | Acceptance restatement; FD-C / FD-G / FD-H |
| §Acceptance criteria — A12 | Spec obligation | Acceptance restatement; FD-B / FD-G / FD-K; new `docs/dsl_spec.md` §4.12 |
| Phase breakdown: star sizing is central algorithmic content | Algorithmic constraint | DD-M3-P5-004; FD-B |
| Same-cell overlap is ZStack responsibility | Scope boundary | Out-of-scope section; DD-M3-P5-003 conflict policy; FD-K ecosystem contrast |
| Risk: Grid measure-arrange spec complexity | Mitigation | Moment 1 spec drafting in FD-G; mental-model anchor in FD-K |

### From [m3-gallery-wireframe.html](../../requirements/gallery-wireframe.html)

| Element | Disposition | Consumed at |
|---|---|---|
| 2D gallery composition pressure | Visible-proof reference | FD-H; author-facing examples |
| Future lightbox / overlay relationship | Downstream constraint | DD-M3-P5-003 conflict policy; FD-K (ZStack owns overlay) |
| Future iteration-generated thumbnails | Routed to WrapPanel, not Grid | Acceptance restatement downstream paragraph: per [spec.md](../../requirements/spec.md) the thumbnail collection is WrapPanel-backed; Grid is not an M3 iteration target, so this is a foreclosure check (DD-M3-P5-001 axis 4), not a Grid surface driver |

### From [M3-Phase 3 decisions](../../phase-3/decisions/preamble.md)

| DD / precedent | Disposition | Consumed at |
|---|---|---|
| DD-M3-P3-005 pure-data measure-arrange | Pattern reuse | DD-M3-P5-004 |
| First novel-normative-spec phase discipline | Pattern reuse | Acceptance restatement; FD-B / FD-G / FD-K |
| Paint overflow not clipped by layout primitive itself | Pattern reuse with Grid-specific clarification | DD-M3-P5-005 Options table must compare pure layout overflow with Grid outer-bounds clipping; occupancy overlap still invalid either way |

### From [M3-Phase 4 decisions](../../phase-4/decisions/preamble.md)

| DD / precedent | Disposition | Consumed at |
|---|---|---|
| Runtime-boundary root-shape lesson | Direct input | FD-C; DD-M3-P5-005 verification note |
| ScrollView intermediate Visual pattern | Negative precedent | DD-M3-P5-005 (Grid should not become a ScrollView-style viewport; outer-bounds clipping, if chosen, must be justified separately) |
| Phase 4 R1 residual | Carry-forward assignment | FD-E / FD-F |
| Phase 4 `scroll_y` drift | Out of scope | Out-of-scope section; M4 handoff only |

### From [M2 handoff](../../../milestone-2/handoff.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §3 reactive drain residuals | Out of scope in recommended path | FD-J; constant-only Grid attributes do not pressure drain residuals, but the chosen Grid surface should not block future bindable tracks |
| §4 `TypedValue` deferral | Discipline reminder | FD-J; no bindable track / membership and no item context in Phase 5 unless owner explicitly expands scope |

---

## Next session — handoff

To move from this draft to ADR drafting:

1. Owner reviews FD-A through FD-K. The surface family choice in
   DD-M3-P5-001 must come first because DD-002 / DD-003 / DD-005
   sub-issues branch on it. Specifically:
   - DD-M3-P5-001: Surface A (track-list + direct child placement) vs
     Surface A2 (track-list + placed `Cell` wrapper) vs Surface B
     (pure structural Row / Cell with B-reject shared sizing rule) vs
     Surface D (Grid columns + structural rows) vs Surface C
     (definition nodes + structural rows), compared on the five axes
     (`.ui` taste, spanning, shared track sizing, future iteration,
     component-extension-model).
   - DD-M3-P5-002: `auto` defer-with-slot vs fully specified `auto`
     now (orthogonal to surface).
   - DD-M3-P5-003: membership, spanning, and conflict policy under
     the selected surface.
   - DD-M3-P5-004: zero-minimum unbounded star vs Grid-specific
     layout error.
   - DD-M3-P5-005: alignment-carrier location (depends on surface),
     pure overflow vs Grid outer-bounds clipping.
   - FD-E: R1 assigned to Phase 6.
2. If aligned, draft `process/milestone-3/phase-5/decisions/preamble.md`
   plus the six `dd-*.md` files as `Status: Proposed`.
3. Draft the Moment 1 `docs/dsl_spec.md` Grid chapter and widget
   registry row.
4. Create `process/milestone-3/phase-5/implementation/preamble.md`
   and `process/milestone-3/phase-5/implementation/plan.md` with the
   final-step retrospective split already represented.
5. Update `process/milestone-3/plan.md` to assign R1 to Phase 6 if owner
   accepts FD-E.
