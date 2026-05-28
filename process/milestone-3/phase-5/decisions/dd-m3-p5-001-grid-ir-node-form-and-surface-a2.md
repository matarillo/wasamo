### DD-M3-P5-001 — Grid IR node form and Surface A2 author surface

**Status:** Proposed

**Context:** Grid is a new layout primitive in `wasamo-ir` and
`wasamo-runtime`. Phase 5 must commit to (i) the IR node shape
(per-kind tag vs structural variant), (ii) the author-facing surface
family that declares tracks, membership, and cell content, and (iii)
the `Cell` wrapper's contract and indexing convention.

Unlike Phase 2 Box / Phase 3 WrapPanel / Phase 4 ScrollView, Grid
introduces **variable-length sub-structure** (a track-list per axis)
and **parent-scoped child contract attributes**. Owner has explicitly
required that Phase 5 not treat the WPF / CSS Grid family
(track-list + child placement) as the default. The framing
([../requirements/framing.md](../requirements/framing.md)) compares
five surface families flatly — A (track-list + placed content child),
A2 (track-list + placed `Cell` wrapper), B (pure structural Row /
Cell), D (Grid columns + structural rows), and C (definition nodes +
structural rows) — on the five owner-specified axes (`.ui` author
taste, spanning, shared track sizing, future iteration foreclosure,
and component-extension-model).

The 2026-05-28 owner alignment settled the **structurally branching
sub-decision** here: **Surface A2** (track-list + placed `Cell`
wrapper). The remaining Surface-A2 sub-decisions — `Cell` as a
single-child layout wrapper, placement defaults, span defaults,
zero-based indexing at the `.ui` boundary, alignment carried on
`Cell`, and no intermediate Visual — are written below as ADR
Recommendations and approved at ADR review.

**Sub-issues:**

- **IR node shape.** Per-kind tag parallel to `Box`, `WrapPanel`,
  and `ScrollView`, vs a structural variant in `IrLayout`. Per-kind
  tag continues the M3 primitive pattern. Grid additionally
  introduces the `TrackSize` domain type (defined in DD-M3-P5-002)
  and explicit `Cell` per-kind tag.
- **Surface family.** Compared in framing across A / A2 / B / D / C.
  Owner-settled at **Surface A2**.
- **`Cell` contract.** Grid-owned single-child layout wrapper
  carrying `row`, `column`, `row-span`, `column-span`, `h-align`,
  `v-align`. Not a general-purpose widget; not usable outside Grid.
- **Indexing convention.** Zero-based at the `.ui` boundary and
  zero-based internally.
- **Minimum valid shape.** Grid requires at least one row track and
  at least one column track; zero `Cell` children produce an empty
  drawn subtree with the resolved Grid outer size.
- **Track declaration syntax.** First-class track-list value on
  Grid's `columns:` / `rows:` attributes (e.g. `columns: 180 1*
  2*`), not a string-encoded form. Syntax detail belongs to
  DD-M3-P5-002.

**Options (IR node shape):**

- **Option A — Per-kind tag with new `TrackSize` domain type +
  per-kind `Cell` tag (recommended).** `Grid` appears as
  `widget_type: "Grid"` and `Cell` appears as
  `widget_type: "Cell"` on the generic `IrNode`, parallel to
  Box / WrapPanel / ScrollView. Grid carries `columns:` and
  `rows:` attributes whose values are a Grid-specific
  `Vec<TrackSize>` domain type populated from existing `IntLit`
  plus a Grid-only star-token shape. `Cell` carries `row`,
  `column`, `row-span`, `column-span` as existing `i32` literals,
  and `h-align` / `v-align` as identifier literals.
  - What you gain: consistency with all prior layout primitives;
    no new `IrType` or `IrLiteral` variant; the parser accepts
    the generic shape unchanged except for the narrow track-list
    parser path declared in DD-M3-P5-002.
  - What you give up: nothing relative to the established
    pattern.
- Option B — Structural variant in `IrLayout`. Grid participates
  as a layout-flavour discriminator rather than a widget kind.
  - What you gain: arguably cleaner separation of "container that
    arranges children" from "leaf widget".
  - What you give up: contradicts Phase 2 / 3 / 4 precedent;
    rewires the IR's existing categorisation; cost without
    visible benefit at Phase 5 scope.

**Options (surface family):**

The framing compares A / A2 / B / D / C in detail
([../requirements/framing.md §Author-facing `.ui` surface options](../requirements/framing.md#author-facing-ui-surface-options))
across the five owner-specified axes. The owner alignment selected
A2; the alternatives are summarised below as rationale.

- **Option A — Track-list + placed content children.** `Grid {
  columns: ...; Text { row: 0 column: 0 ... } }`. Compact and
  explicit; Grid-specific metadata appears on arbitrary content
  widgets (first built-in precedent for parent-scoped child
  metadata).
- **Option A2 — Track-list + placed `Cell` wrapper (recommended;
  owner-settled).** `Grid { columns: ...; Cell { row: 0 column: 0
  Text { ... } } }`. Track sharing is automatic (canonical
  `columns:` / `rows:`); Grid-specific metadata is contained on
  `Cell` rather than on arbitrary content widgets; both spanning
  axes are symmetric `(row, column, row-span, column-span)`
  tuples.
- Option B — Pure structural `Row` / `Cell`. `Grid { Row { Cell {
  width: 180 ... } Cell { width: 1* ... } } }`. Document structure
  mirrors visible structure, but shared column sizing requires a
  reconciliation rule (B-reject / B-first / B-independent — only
  B-reject is well-defined as Grid).
- Option D — Grid columns + structural rows. `Grid { columns: ...;
  Row { height: ... Cell { ... } } }`. Shared columns canonical
  but the surface is intentionally asymmetric (column-side
  parent-owned, row-side structural).
- Option C — Definition nodes + structural rows. `Grid {
  ColumnDefs { ColumnDef { width: ... } ... } RowDefs { ... } Row
  { Cell { ... } ... } }`. Shared track sizing solved by
  construction at the cost of an extra surface layer (definition
  nodes).

**Comparison (surface family — axes from framing):**

| Axis | A | **A2 (selected)** | B | D | C |
|---|---|---|---|---|---|
| `.ui` author taste (irregular placement) | Most compact | Compact (one `Cell` wrapper layer) | Verbose for irregular layouts | Compact for columns, verbose for irregular row placement | Most boilerplate (definition nodes) |
| Spanning (row + column symmetry) | Symmetric `(row, col, row-span, col-span)` on content child | **Symmetric `(row, col, row-span, col-span)` on `Cell`** | Asymmetric (column-span intra-`Row`; row-span needs implicit-skip vs explicit-placeholder rule) | Same asymmetry as B for rows | Same asymmetry as B for rows |
| Shared track sizing | Automatic | **Automatic** | Requires reconciliation rule (B-reject minimum) | Automatic for columns | Automatic via definition nodes |
| Future iteration foreclosure (post-M3 only) | Non-foreclosing | **Non-foreclosing** | Non-foreclosing | Non-foreclosing | Non-foreclosing |
| Component-extension-model precedent | Parent-scoped metadata on arbitrary widgets (first such precedent) | **Grid-owned wrapper carries child contract attributes** | Structural child node kinds with own schemas | Mixed (parent config + structural content) | Definition-node pattern (non-visual container shape) |

Surface A2 was selected because it preserves A's irregular-placement
power and automatic shared track sizing while containing Grid-specific
metadata in a Grid-owned wrapper (avoiding `row` / `column` on
arbitrary content widgets). The cost is one wrapper layer and weaker
row-structure readability than B / D / C.

**Options (`Cell` contract):**

- **Option A — Grid-owned single-child layout wrapper (recommended).**
  `Cell` accepts exactly one content child. `wasamoc check` rejects
  `Cell { }` (0 children) and `Cell { X Y }` (>1 children) with a
  diagnostic naming the offending shape. `Cell` outside a `Grid`
  parent is rejected (no general-purpose use).
  - What you gain: "1 cell 1 child" surface invariant enforced
    syntactically; the same-cell overlap question collapses to the
    structurally simpler "two `Cell`s in the same rectangle"
    question handled by DD-M3-P5-003; ZStack's overlay
    responsibility stays uncontaminated.
  - What you give up: authors who want multiple children in one
    cell must wrap them in a layout container (`Cell { VStack {
    Text {} Text {} } }`); a tiny ergonomic cost paid in exchange
    for a clean invariant.
- Option B — Multi-child `Cell` with implicit z-stack semantics.
  `Cell { A B }` paints `A` below `B`.
  - What you gain: ergonomic shorthand.
  - What you give up: this is exactly ZStack's job; admitting it
    on `Cell` re-introduces the overlay surface A2 was selected to
    avoid. Contradicts the "1 cell 1 child" boundary in A2.
- Option C — Multi-child `Cell` rejected at parse, accepted at
  validate. Inconsistent gating; rejected on uniformity grounds.

**Options (placement defaults):**

- **Option A — Explicit `row` and `column` required on every
  `Cell` in a multi-`Cell` Grid; a one-`Cell` Grid may default to
  `(0, 0)` (recommended).** Phase 5 has no auto-placement policy.
  - What you gain: every multi-`Cell` Grid is self-describing; no
    silent placement; the diagnostic surface for "Cell omitted
    row" is local and unambiguous; auto-placement is reserved as a
    future surface decision rather than implicit.
  - What you give up: the smallest Grid examples write `row: 0
    column: 0` on a single `Cell` redundantly; the one-`Cell` Grid
    escape clause mitigates this for the smallest demo case.
- Option B — Default `(0, 0)` everywhere. Multi-`Cell` Grids may
  omit placement and silently overlap at `(0, 0)`.
  - What you gain: smallest possible single-`Cell` example.
  - What you give up: a multi-`Cell` Grid that forgets one
    placement silently collapses; the diagnostic surface becomes
    non-local. Direct conflict with the "1 cell 1 child" rule.
- Option C — Auto-placement (document-order assignment to next
  free cell).
  - What you gain: ergonomic for table-like layouts.
  - What you give up: introduces a placement algorithm that
    DD-M3-P5-004 does not own; couples DD-M3-P5-003 and
    DD-M3-P5-001; explicitly out of Phase 5 scope per the
    framing's surface-pair invalid-combination check.

**Options (span defaults and bounds):**

- **Option A — `row-span` and `column-span` default to 1; both
  must be positive integers; `row + row-span <= rows.len()` and
  `column + column-span <= columns.len()` (recommended).** Per the
  owner alignment, both axes are admitted. DD-M3-P5-003 owns the
  full span surface; this DD records the IR-side defaults.
  - What you gain: every `Cell` resolves to a deterministic
    rectangle; out-of-range spans are reject-at-validate, not
    clamp-at-arrange (DD-M3-P5-006); span omission is the common
    case and writes as zero attributes.
  - What you give up: nothing relative to A2's coordinate
    symmetry.
- Option B — `column-span` only admitted; `row-span` deferred
  with attribute name reserved. The original DD-M3-P5-003 axis-2
  branch.
  - What you gain: smallest span surface.
  - What you give up: under A2 the two axes are symmetric and
    deferral adds a defer-and-reject path with no surface saving;
    framing explicitly settled both axes.

**Options (indexing convention):**

- **Option A — Zero-based at `.ui` boundary and internally
  (recommended).** `row: 0` is the first row; valid `row` values
  are `[0, rows.len())`.
  - What you gain: consistency with runtime / test conventions
    and with array-indexing intuition in Rust; no translation
    layer between `.ui` and internal indices; diagnostic indices
    match author indices.
  - What you give up: `row: 1` does **not** mean "first row"
    (potential confusion for authors coming from one-based
    surfaces like XAML grid `Grid.Row="0"` which is also
    zero-based, but CSS-counted contexts use one-based line
    numbers — the asymmetry is documented in the dsl_spec
    chapter).
- Option B — One-based at `.ui` boundary, zero-based internally.
  Lossy translation; rejected on diagnostic clarity grounds.

**Options (alignment carrier — interaction with DD-M3-P5-005):**

- **Option A — Alignment lives on `Cell` (recommended).** `Cell {
  h-align: end v-align: center <content> }`. Per-Cell alignment is
  the only mechanism for non-stretch placement within a resolved
  cell.
  - What you gain: keeps all Grid-specific child contract
    attributes on `Cell`; content widgets do not learn
    Grid-specific alignment merely because they appear inside
    Grid; symmetric with A2's `row` / `column` carrier; matches
    the Surface B / D / C alignment-on-`Cell` convention so a
    future surface family change would not relocate alignment.
  - What you give up: nothing.
- Option B — Alignment lives on the content widget directly.
  Contradicts A2's "Grid-specific metadata stays on `Cell`"
  contract.

**Options (minimum valid Grid shape):**

- **Option A — At least one row and at least one column required
  (recommended).** A Grid with empty `columns:` or empty `rows:`
  surfaces a `wasamoc check` diagnostic and `WASAMO_ERR_IR_MALFORMED`
  at `validate()`. Zero `Cell` children produces a Grid with a
  resolved outer rectangle (per the parent allocation) but no
  drawn cell content.
  - What you gain: track-resolution algorithm in DD-M3-P5-004 has
    a well-defined input domain (at least one fixed or star track
    per axis); zero-cell Grid is structurally a valid empty
    container.
  - What you give up: nothing relative to A2.

**Decision (Recommendation):**

- IR node shape: **Option A** (per-kind `Grid` and `Cell` tags
  with `TrackSize` domain type).
- Surface family: **Option A2** (track-list + placed `Cell`
  wrapper) — owner-settled at framing.
- `Cell` contract: **Option A** (single-child layout wrapper).
- Placement defaults: **Option A** (explicit `row` / `column` in
  multi-`Cell` Grid; default `(0, 0)` only for a one-`Cell`
  Grid).
- Span defaults / bounds: **Option A** (default `1`; positive
  integers; span within declared track count).
- Indexing convention: **Option A** (zero-based at `.ui` boundary
  and internally).
- Alignment carrier: **Option A** (on `Cell`).
- Minimum valid Grid shape: **Option A** (`columns.len() >= 1` and
  `rows.len() >= 1`; zero `Cell` children allowed).

**Forward-compat exposure:**

- **`auto` / intrinsic tracks (Post-Phase-5 hand-off item 1).**
  The `TrackSize` domain type is the extension point for a future
  `Auto` variant; admitting it requires a measure-side demand pass
  in DD-M3-P5-004 but no IR shape change here.
- **Named lines / template areas (Post-Phase-5 hand-off item 2).**
  Surface A2's placement metadata on `Cell` does not foreclose a
  future area-name attribute (e.g. `Cell { area: "header" ... }`);
  such an attribute would be additive and would lower to the same
  `(row, column, row-span, column-span)` rectangle.
- **Bindable track / placement (Post-Phase-5 hand-off item 3).**
  Constant-only constraint is a scope decision, not a surface
  decision; existing binding-effect machinery covers later
  bindable extensions.
- **Iteration template generating `Cell`s (Post-Phase-5 hand-off
  item 4).** Surface A2 makes this structurally possible because
  every `Cell` is explicit; framing axis 4 (foreclosure check)
  holds.
- **Per-cell clipping and `clip:` attribute (Post-Phase-5 hand-off
  item 6).** Adding a per-cell clip attribute later is additive;
  the Grid outer-bounds clip from DD-M3-P5-005 is independent.

**Technical risk re-evaluation:**

- **First built-in `Cell`-wrapper precedent.** Surface A2 is the
  first built-in layout primitive in wasamo that introduces a
  Grid-owned wrapper carrying parent-scoped child contract
  attributes. The risk is that future custom layout components
  (M4+ component-extension-model) inherit a wrapper-based
  precedent rather than a parent-scoped-attached-property
  precedent. Per framing decision FD-J, this is recorded as
  trajectory rather than fault — neither precedent is inherently
  better, and the alternative (Surface A) would have set the
  parent-scoped attached-property precedent instead. Phase 5 does
  not commit the future component-extension model to either
  trajectory; both remain compatible with Surface A2.
- **`Cell` outside `Grid`.** The single-child contract is enforced
  at `wasamoc check`; a `Cell` appearing as a non-Grid child is
  rejected at parse / check time. This is the structurally
  simplest disposition and matches the Surface A2 framing's
  "`Cell` is a Grid-owned wrapper" intent.
- **Track-list value shape vs binding.** Phase 5 ships
  constant-only `columns:` / `rows:`. A future bindable `columns:`
  would require the track-list domain type to participate in the
  binding pipeline (`TypedValue` machinery, M4+); Surface A2 does
  not block this but it is **not** Phase 5 scope.

**Layering with DD-M3-P5-002 / DD-M3-P5-003 / DD-M3-P5-004 /
DD-M3-P5-005 / DD-M3-P5-006:**

- DD-M3-P5-001 names *what* a Grid contains (track lists + `Cell`
  children) and what a `Cell` carries (placement + span +
  alignment metadata + single content child).
- DD-M3-P5-002 settles the `TrackSize` value forms admitted in
  Phase 5 (fixed integer pixels + weighted star; no `auto`).
- DD-M3-P5-003 settles the placement / span / conflict surface
  for `Cell` children (this DD records the defaults; DD-M3-P5-003
  records the conflict-detection rule and the span-bound
  validation).
- DD-M3-P5-004 settles the track-resolution algorithm (consumes
  DD-M3-P5-001's track-list and `Cell` rectangles).
- DD-M3-P5-005 settles the arrange / overflow / z-order /
  alignment-application semantics (consumes DD-M3-P5-001's
  alignment-on-Cell carrier).
- DD-M3-P5-006 settles which DD-M3-P5-001 invariants (minimum
  track count, Cell child-count) are dual-gated at `wasamoc
  check` and runtime `validate()`.

Invalid surface-pair combinations explicitly rejected by this DD
in combination with downstream DDs (per the framing's invalid-
combinations check):

- DD-M3-P5-001 = Surface A2 + DD-M3-P5-003 = auto-placement
  admitted + DD-M3-P5-004 = no document-order placement
  algorithm. Auto-placement under A2 has no placement algorithm
  defined; Phase 5 explicitly does not admit auto-placement (this
  DD's placement-default Option A).
- DD-M3-P5-001 = string-encoded track list + DD-M3-P5-006 =
  parser-level track-list diagnostics required. Recommendation
  Option A uses first-class track-list value, so this combination
  does not arise.
