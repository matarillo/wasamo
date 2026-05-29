### DD-M3-P5-006 — IR-loader defense-in-depth invariants

**Status:** Accepted

**Context:** Phase 2 T7 surfaced the principle: IR-load → runtime-
materialise invariants belong in pure-logic `validate()`, not in
WinRT-bound `build_node`, so the same invariant is enforced
regardless of which entry point materialises the IR. Phase 3 T6
extended this with WrapPanel's value-range invariants (negative-
literal rejection). Phase 4 DD-M3-P4-006 introduced a **compound
shape**: structural child-count rejection (Phase-2-flavour) plus
runtime clamp for the offset value (which is *not* a validate-time
reject; bound state may legitimately transition through negative
values).

Phase 5 Grid's invariants are a **broader version of the same
compound shape**, with two notable differences:

- Grid has **more structural invariants** than any prior layout
  primitive (minimum row / column count, Cell single-child, surface
  lowering, placement-in-range, span-in-range, cross-`Cell`
  conflict rejection).
- Grid has **no runtime-clamp invariant** (no analogue to
  ScrollView's `offset-y` clamp). All Phase 5 Grid invariants are
  **reject-at-validate**, not clamp-at-arrange. Placement / span
  values are structurally meaningful values without defensible
  clamped interpretation; a "clamped" `column: 5` in a 2-column
  Grid would displace a legitimately-placed Cell and produce
  order-dependent layout.

This DD records which Grid invariants are dual-gated by `wasamoc
check` (compile-time half) and runtime `validate()` (memory-IR-load
half), preserving the two-gate defense-in-depth pattern that Phase
1 / Phase 2 T7 / Phase 3 T6 / Phase 4 DD-M3-P4-006 established.

**Sub-issues:**

- **Minimum row / column count.** Grid must declare at least one
  row track and at least one column track (DD-M3-P5-001 minimum-
  shape).
- **Surface lowering.** The chosen author surface (Surface A2 per
  DD-M3-P5-001) lowers successfully into `TrackSize` sequences plus
  per-`Cell` logical membership tuples. The lowering is owned by
  the parser plus the narrow track-list parser path
  (DD-M3-P5-002); `wasamoc check` and runtime `validate()` are
  defense-in-depth safety nets.
- **Track value range.** Fixed track values must be `>= 1`; star
  weights must satisfy `1 <= weight <= 1024`. The per-weight cap
  (DD-M3-P5-002) combined with DD-M3-P5-004's `u64`
  star-weight-sum accumulator bounds the per-axis sum at the type
  level (no "realistic track count" assumption is required for
  overflow safety). The deferred `auto` token is rejected with a
  reserved-future diagnostic (DD-M3-P5-002).
- **Cell child-count.** Each `Cell` accepts exactly one content
  child (DD-M3-P5-001).
- **Cell placement-attribute presence (compile-time-only).** In a
  Grid with >= 2 Cells, every Cell must declare both `row` and
  `column`; in a Grid with exactly 1 Cell, missing `row` and/or
  `column` lowers to `0` at `wasamoc lower` per
  DD-M3-P5-001's placement-default Option A. Memory IR has
  explicit values after lowering (no "missing" representation at
  the IR boundary); runtime `validate()` therefore only checks
  the range invariant below.
- **Placement value range.** `Cell.row` in `[0, rows.len())` and
  `Cell.column` in `[0, columns.len())` (DD-M3-P5-003).
- **Span value range.** `Cell.row-span >= 1`, `Cell.column-span
  >= 1`, and the resolved rectangle fits within declared track
  count (DD-M3-P5-003).
- **Same-cell / overlapping-rectangle conflicts.** No two `Cell`s
  in the same Grid share any resolved cell (DD-M3-P5-003).
- **Alignment-value vocabulary.** `h-align` and `v-align` values
  are in `{ start, center, end, stretch }` (DD-M3-P5-005).
- **Error class.** All Grid IR-loader invariant violations
  surface as `WASAMO_ERR_IR_MALFORMED`, consistent with Phase 2 /
  3 / 4 precedent.

**Gate ownership table (recommended):**

| Invariant | `wasamoc check` | runtime `validate()` | Layout-time |
|---|---|---|---|
| Grid has at least one row and at least one column | Reject | Reject | (n/a) |
| Grid surface lowers successfully into `TrackSize` sequences + logical `Cell` membership tuples | Reject (parser primary) | Reject (defense-in-depth) | (n/a) |
| Cell placement-attribute presence: in a Grid with >= 2 Cells, every Cell declares both `row` and `column`; in a Grid with exactly 1 Cell, missing `row` and/or `column` lowers to `0` | Reject if multi-Cell omission; lower-to-`0` if single-Cell omission | (n/a — memory IR has explicit `row` / `column` values after lowering; the range-check rows below cover post-lowering) | (n/a) |
| Fixed track value `> 0` | Reject | Reject | (n/a) |
| Star weight in `[1, 1024]` (per-weight cap; combined with DD-M3-P5-004's `u64` star-weight-sum accumulator, bounds the per-axis sum at the type level for any structurally feasible IR — see DD-M3-P5-002) | Reject | Reject | (n/a) |
| `auto` token reserved-future | Reject (named diagnostic) | Reject | (n/a) |
| `Cell` has exactly one content child | Reject | Reject | (n/a) |
| `Cell.row` in `[0, rows.len())` | Reject | Reject | (n/a) |
| `Cell.column` in `[0, columns.len())` | Reject | Reject | (n/a) |
| `Cell.row-span >= 1` and `Cell.column-span >= 1` | Reject | Reject | (n/a) |
| `Cell.row + Cell.row-span <= rows.len()` and `Cell.column + Cell.column-span <= columns.len()` | Reject | Reject | (n/a) |
| No two `Cell`s share any resolved cell | Reject | Reject | (n/a) |
| `Cell.h-align` and `Cell.v-align` value in vocabulary | Reject | Reject | (n/a) |
| Unbounded star-axis parent at layout | (n/a) | (n/a) | `LayoutError::GridUnboundedStarAxis` |
| Fixed track sum exceeds parent bound | (n/a) | (n/a) | Star tracks resolve to `0` (overflow per DD-M3-P5-005); not a fault |

All Grid invariants are **reject-at-validate**, not clamp-at-
arrange. The only layout-time gate is the unbounded-star error
(DD-M3-P5-004); negative-remaining-space is not a fault.

**Options:**

- **Option A — Full dual-gate at `wasamoc check` and `validate()`;
  no clamp-at-arrange (recommended).** Every invariant in the
  table above is enforced at both compile time and runtime.
  Surface lowering has parser as the primary diagnostic surface
  (token-level errors with source location); `wasamoc check` and
  `validate()` are safety nets for the cases the parser cannot
  cleanly express (e.g. the `auto` token is parsed successfully
  as an identifier and then rejected at lowering).
  - What you gain: each invariant is enforced at the layer
    appropriate to its shape; the parser surface provides the
    best diagnostics for syntax-level errors; the two-gate
    pattern catches malformed memory IR that bypasses `wasamoc`;
    no silent layout-time degradation (all violations are
    rejected upfront); symmetric with Phase 4 DD-M3-P4-006's
    structural-rejection half of the compound shape.
  - What you give up: nothing relative to the alternatives.
- Option B — Validate-only (rely on runtime `validate()` for the
  invariants `wasamoc check` would also catch). Rejected because
  Phase 1 / 2 T7 / 3 T6 / 4 explicitly established the two-gate
  principle.
- Option C — Layout-time clamp for placement / span values (clamp
  `Cell.column` to `[0, columns.len() - 1]` and `Cell.column-span`
  to `[1, columns.len() - column]`).
  - What you gain: never errors at validate for these invariants.
  - What you give up: a `Cell` with `column: 5` in a 2-column
    Grid would silently clamp to `column: 1`, displacing a
    legitimately-placed `Cell` at `column: 1`; conflict detection
    becomes layout-time and order-dependent; rejected per
    DD-M3-P5-003 placement-bounds Option A grounds.
- Option D — Compound shape with a runtime-clamp invariant (the
  Phase 4 DD-M3-P4-006 shape). No Grid invariant has a defensible
  clamp semantics, so the compound shape collapses to "structural
  invariants only". Recorded for completeness; this is
  structurally equivalent to Option A.

**Decision (Recommendation):**

Option A. The `validate()` extension rejects every structural
invariant in the table above. Parser-level diagnostics are the
primary surface for syntax-level errors (track-list shape, `auto`
token); `wasamoc check` and `validate()` are the defense-in-depth
safety nets for the cases the parser does not handle. No layout-
time clamp applies to placement / span / conflict invariants.

**Error class:**

- **Compile-time (`wasamoc check`):** All violations surface as
  diagnostics naming the offending shape, attribute, value, or
  conflict. Diagnostic text obligations per individual DDs (e.g.
  DD-M3-P5-002 reserved-future `auto` diagnostic; DD-M3-P5-003
  span-vs-bound specificity).
- **Runtime (`validate()`):** All violations surface as
  `WASAMO_ERR_IR_MALFORMED`, consistent with Phase 2 / 3 / 4
  precedent.
- **Layout time:** The only Grid layout-time error is
  `LayoutError::GridUnboundedStarAxis` (DD-M3-P5-004), which is
  not a `validate()`-time concern (it depends on the parent's
  axis bound, which is not known at IR-load time).

**Bound-direction validation (consequence of DD-M3-P5-001 / 002
constant-only constraint):**

Per the Phase 5 constant-only constraint, no Grid or Cell
attribute is bindable. The DD-M3-P4-006 "bound-direction
validation" sub-issue (which collapsed for Phase 4 because DD-003
chose Option B bindable read-only) collapses for Phase 5 because
no attribute is bindable at all. Recorded for completeness: if a
future phase admits bindable Grid attributes, the bound-direction
validation question (which states are mutable, whether placement
can be bound, whether the validate-time check would need to defer
to layout-time) would need its own DD.

**Forward-compat exposure:**

- **`auto` admission (Post-Phase-5 hand-off item 1).** When
  `auto` is admitted, this DD's `auto`-reserved-future diagnostic
  is removed; the `Auto` variant becomes valid and `validate()`
  checks any auto-specific invariants (e.g. that auto tracks
  composed with star tracks in the same axis are not both unbounded
  in a way that contradicts the demand pass).
- **Bindable attributes (Post-Phase-5 hand-off item 3).** When
  Grid attributes become bindable, the placement / span / conflict
  invariants become **layout-time** for the bound case (the bound
  value is not known at IR-load time). `validate()` would relax
  for bound attributes and the layout-time check would gain the
  rejection logic; this is a structural change that needs its own
  DD.
- **Per-cell clip / `z-index` attributes (Post-Phase-5 hand-off
  items 6, 5).** Future per-cell clip or layering attributes
  would add their own validate-time invariants (e.g.
  `clip: <bool>` value-range; `z-index: <i32>` range). Phase 5
  validate surface is the natural extension point.
- **Track-list grammar extensions (named lines, `minmax`).** New
  `TrackSize` variants extend the validate vocabulary; the gate
  pattern is unchanged.

**Technical risk re-evaluation:**

- **Diagnostic-text quality for nested invariants.** A `Cell`
  with `column: 1, column-span: 3` in a 2-column Grid has two
  failures (column technically in range; column + column-span out
  of range). The diagnostic should use the more specific failure
  (DD-M3-P5-003 technical risk). This is a diagnostic-text
  obligation, not a structural risk.
- **Cross-`Cell` conflict diagnostic locality.** Conflict
  diagnostics name both conflicting `Cell`s and the shared
  resolved cell coordinate. Implementation must surface both
  `Cell`s' source locations; this is a parser-token-tracking
  concern.
- **Symmetry between `wasamoc check` and `validate()`
  diagnostics.** Where the same invariant is dual-gated, the
  diagnostic text should be similar (so a memory-IR-load error
  reads like the `wasamoc check` diagnostic the author would have
  seen if they had compiled). Phase 4 T7 dispositions established
  the precedent; Phase 5 inherits.
- **Validate-pass cost.** The Grid validate pass is `O(n_cells)`
  for value-range checks plus `O(n_cells^2)` for pairwise
  rectangle overlap. For practical Grid sizes (`n_cells <= 30`),
  this is trivially fast. For pathologically large Grids, the
  same scalability comment as DD-M3-P5-003 applies.
- **`Cell` outside `Grid`.** `wasamoc check` rejects `Cell` as a
  non-Grid child; `validate()` reuses the same structural-parent
  check. Defense-in-depth.

**Layering with DD-M3-P5-001 / DD-M3-P5-002 / DD-M3-P5-003 /
DD-M3-P5-004 / DD-M3-P5-005:**

- DD-M3-P5-006 (this DD) consumes:
  - Minimum-shape rule from DD-M3-P5-001.
  - Surface lowering shape from DD-M3-P5-001 / DD-M3-P5-002.
  - Track-value-range constraints from DD-M3-P5-002.
  - Cell child-count rule from DD-M3-P5-001.
  - Placement / span / conflict constraints from DD-M3-P5-003.
  - Alignment-value vocabulary from DD-M3-P5-005.
- DD-M3-P5-006 produces:
  - `wasamoc check` diagnostic surface coverage.
  - Runtime `validate()` invariant gates.
- DD-M3-P5-004 owns the only layout-time gate
  (`LayoutError::GridUnboundedStarAxis`); it is **not** dual-
  gated at `validate()` because it depends on the parent's
  axis bound.

Invalid combinations explicitly rejected:

- DD-M3-P5-006 = validate-only (no `wasamoc check`). Does not
  arise: Recommendation Option A dual-gates.
- DD-M3-P5-006 = clamp-at-arrange for placement / span. Does not
  arise: Recommendation Option A reject-at-validate.
- DD-M3-P5-003 = spans may exceed declared track count +
  DD-M3-P5-006 = validation-only defense. Does not arise: this DD's
  Option A dual-gates the span-bound invariant.
