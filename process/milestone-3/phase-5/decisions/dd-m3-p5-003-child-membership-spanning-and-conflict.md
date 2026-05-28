### DD-M3-P5-003 — Child membership, spanning, and conflict policy

**Status:** Proposed

**Context:** Grid children become members of resolved cells via the
`Cell` wrapper carrying placement and span metadata (per DD-M3-P5-001
Surface A2). Phase 5 must commit to:

- the placement-attribute set (`row`, `column`) and the spanning-
  attribute set (`row-span`, `column-span`);
- the defaults for each attribute (DD-M3-P5-001 records the IR-side
  defaults; this DD records the placement / spanning surface and
  its interaction with the "1 cell 1 child" rule);
- the conflict-detection rule for same-cell occupancy and
  overlapping spans;
- the validity bounds for placement and span values; and
- the per-axis admission scope (both axes vs one).

The 2026-05-28 owner alignment settled the **structurally branching
sub-decisions**: both spanning axes are admitted (column-span and
row-span), Surface A2 makes the two axes symmetric `(row, column,
row-span, column-span)` rectangles, and same-cell / overlapping-
rectangle conflicts are rejected (preserving the A2 "1 cell 1 child"
boundary; ZStack owns overlay).

**Sub-issues:**

- **Membership surface.** `Cell` is the membership carrier under
  Surface A2 (DD-M3-P5-001). The content widget is the single
  child of `Cell` and carries no Grid-specific metadata.
- **Placement attributes.** `row: <i32>` and `column: <i32>` on
  `Cell`. Zero-based per DD-M3-P5-001.
- **Span attributes.** `row-span: <i32>` and `column-span: <i32>`
  on `Cell`. Default `1`.
- **Per-axis admission scope.** Owner-settled: both axes.
- **Conflict policy.** Two `Cell`s in the same Grid whose resolved
  `(row, column, row-span, column-span)` rectangles share at least
  one resolved cell are rejected.
- **Placement / span value bounds.** All four values are integers;
  `row` / `column` are in `[0, rows.len())` / `[0, columns.len())`;
  span values are in `[1, ...]` and the resolved rectangle must
  fit within the declared track count.

**Membership surface (consequence of DD-M3-P5-001 Surface A2):**

Each `Cell` declares its resolved rectangle as
`(row, column, row-span, column-span)` tuple. Document order within
`Grid` provides the **z-order** (DD-M3-P5-005), not membership.
Membership is fully determined by the explicit placement / span
attributes.

```wasamo-ui
Grid {
  columns: 96 1* 96
  rows: 64 1* 120

  // Cell A: rectangle (row 0, col 0, row-span 1, col-span 3)
  Cell {
    row: 0
    column: 0
    column-span: 3
    Text { text: "Gallery" }
  }

  // Cell B: rectangle (row 1, col 0, row-span 1, col-span 1)
  Cell {
    row: 1
    column: 0
    Box { fill: #2f4050ff }
  }
}
```

**Options (placement attribute set):**

- **Option A — `row: <i32>` and `column: <i32>` on `Cell`
  (recommended).** Zero-based; defaults to `(0, 0)` only when the
  Grid has exactly one `Cell` per DD-M3-P5-001's placement-default
  Option A.
  - What you gain: minimal placement surface; deterministic
    per-`Cell` resolution; no document-order coupling; explicit-
    over-implicit per A2's design intent.
  - What you give up: smallest multi-`Cell` example writes
    `(row, column)` on every `Cell` (cost of explicit placement).
- Option B — `at: <row,column>` compound attribute. E.g. `Cell { at:
  1,0 ... }`.
  - What you gain: shorter syntax for the common case.
  - What you give up: opens a new comma-separated literal shape
    inside Grid's narrow parser path for cosmetic benefit;
    distinguishes Grid from other primitives' attribute idioms.
- Option C — Document-order auto-placement (no explicit `row` /
  `column`; document order maps to row-major cell sequence).
  - What you gain: ergonomic for fully populated table-like
    Grids.
  - What you give up: contradicts DD-M3-P5-001's placement-default
    Option A (no auto-placement in Phase 5); irregular layouts
    (the spanning header / footer slice in FD-H gallery proof) are
    unexpressible; explicitly excluded by the framing's invalid-
    combinations check.

**Options (span attribute set and defaults):**

- **Option A — `row-span: <i32>` and `column-span: <i32>` on
  `Cell`; both default to 1; both must be positive integers
  (recommended).** Per the owner alignment, both axes are admitted.
  - What you gain: symmetric surface; default 1 means non-spanning
    `Cell`s omit both attributes (zero-attribute case is the
    norm); positive-integer bound is dual-gated at `wasamoc check`
    and `validate()` (DD-M3-P5-006); `Cell` rectangle resolution
    is a single tuple, simplifying DD-M3-P5-004 spanning
    reconciliation.
  - What you give up: nothing relative to A2.
- Option B — `row-span` deferred (column-span admitted only); the
  `row-span` attribute name is reserved and rejected at `wasamoc
  check` until admitted. The original DD-M3-P5-003 axis-2 deferral
  branch from the pre-alignment framing.
  - What you gain: smallest span surface; smallest pure-layout
    test surface (column-span only).
  - What you give up: contradicts the owner-settled "both axes
    admitted" decision; under Surface A2 the two axes are
    symmetric and admitting both adds zero surface concept beyond
    the shared rectangle check.
- Option C — `span: <row-span,column-span>` compound attribute.
  Rejected on the same grounds as the placement Option B above.

**Options (conflict policy):**

- **Option A — Same-cell / overlapping-rectangle conflicts
  rejected (recommended).** Two `Cell`s in the same Grid whose
  resolved `(row, column, row-span, column-span)` rectangles share
  at least one resolved cell are rejected at `wasamoc check` and
  `validate()` (DD-M3-P5-006). This preserves A2's "1 cell 1
  child" boundary verbatim.
  - What you gain: A2 scope boundary held; ZStack remains the
    surface for intentional overlay; conflict-detection algorithm
    is a single rectangle-overlap check across every pair of
    `Cell`s; diagnostic surface is local ("Cell A at (row 0, col
    0) and Cell B at (row 0, col 0) share resolved cell (0, 0)").
  - What you give up: authors who want overlay-like effects must
    use ZStack (Phase 6); no implicit overlay shortcut.
- Option B — Same-cell occupancy allowed with implicit document-
  order z-stack.
  - What you gain: ergonomic overlay shortcut.
  - What you give up: contradicts A2 explicit "1 cell 1 child";
    steals Phase 6 ZStack's responsibility; the document-order
    z-order rule from DD-M3-P5-005 would have to expand from "for
    paint-overflow that happens to overlap" to "for intentional
    overlay" — a much stronger commitment with no acceptance
    requirement.
- Option C — Same-cell occupancy allowed with explicit `z-index`
  attribute. Rejected because Phase 5 does not admit `z-index` per
  DD-M3-P5-005.

**Comparison (conflict detection across surface families — for
rationale):**

Under Surface A2, conflict detection is uniform: every `Cell` has
explicit `(row, column, row-span, column-span)`, so the algorithm
is "for each pair `(A, B)` of `Cell`s in this Grid, do their
rectangles overlap?". Under structural families (B / D / C),
conflict detection works over resolved Cell-to-column mappings with
implicit-skip vs explicit-placeholder rules for row-span. Surface A2
is structurally the simplest conflict-detection target.

**Options (placement / span value bounds — dual-gating):**

- **Option A — All bounds checked at both `wasamoc check` and
  `validate()` (recommended).** Per DD-M3-P5-006 invariant table:
  - `row` in `[0, rows.len())`;
  - `column` in `[0, columns.len())`;
  - `row-span >= 1`;
  - `column-span >= 1`;
  - `row + row-span <= rows.len()`;
  - `column + column-span <= columns.len()`;
  - For every pair of `Cell`s, no rectangle overlap.

  All eight invariants surface a `wasamoc check` diagnostic at
  compile time and `WASAMO_ERR_IR_MALFORMED` at runtime
  `validate()`. Violations are **reject-at-validate**, not
  **clamp-at-arrange**, because placement / span are
  structurally meaningful values that have no defensible clamped
  interpretation (unlike Phase 4 ScrollView's `offset-y`, which
  has a clear `[0, max]` clamp semantics per binding transition).
  - What you gain: deterministic diagnostic surface; no silent
    layout-time degradation; symmetric with Phase 2 T7 / Phase 4
    DD-M3-P4-006 structural rejection pattern.
  - What you give up: nothing.
- Option B — Layout-time clamp (placement / span clamped to
  valid range at arrange).
  - What you gain: never errors at validate.
  - What you give up: a `Cell` whose `column: 5` in a 2-column
    Grid would silently clamp to `column: 1`, displacing a
    legitimately-placed `Cell` at `column: 1`; conflict detection
    becomes layout-time and order-dependent; rejected per Phase 2
    / 4 precedent.

**Decision (Recommendation):**

- Placement attribute set: **Option A** (`row` + `column` on
  `Cell`, zero-based; defaults per DD-M3-P5-001 placement-default
  Option A).
- Span attribute set: **Option A** (`row-span` + `column-span`,
  default `1`; both axes admitted) — owner-settled at framing.
- Conflict policy: **Option A** (same-cell / overlapping-rectangle
  rejected) — owner-settled at framing.
- Placement / span value bounds: **Option A** (all bounds
  dual-gated at `wasamoc check` and `validate()`; reject-at-
  validate, not clamp-at-arrange).

The complete `Cell` placement / span surface in Phase 5:

| Attribute | Type | Default | Valid range | Violations |
|---|---|---|---|---|
| `row` | `i32` | `0` (one-Cell Grid only) | `[0, rows.len())` | `wasamoc check` + `validate()` reject |
| `column` | `i32` | `0` (one-Cell Grid only) | `[0, columns.len())` | `wasamoc check` + `validate()` reject |
| `row-span` | `i32` | `1` | `[1, rows.len() - row]` | `wasamoc check` + `validate()` reject |
| `column-span` | `i32` | `1` | `[1, columns.len() - column]` | `wasamoc check` + `validate()` reject |

Same-cell / overlapping-rectangle conflicts are detected by
pairwise rectangle-overlap check and surfaced as
`wasamoc check` diagnostic + `WASAMO_ERR_IR_MALFORMED`.

**Forward-compat exposure:**

- **`auto`-placement (Post-Phase-5 hand-off).** Phase 5 explicitly
  excludes auto-placement; admitting it later requires a placement
  algorithm DD and the placement-default decision in DD-M3-P5-001
  would need to be revisited. The `Cell` placement surface itself
  does not foreclose auto-placement (a future `Cell` with omitted
  `row` / `column` would invoke an auto-placement pass).
- **Area-name attribute (Post-Phase-5 hand-off item 2).** A future
  `Cell { area: "header" ... }` would lower to the same `(row,
  column, row-span, column-span)` rectangle; the conflict-detection
  rule remains rectangle-based and is unchanged.
- **Bindable placement (Post-Phase-5 hand-off item 3).** A future
  bindable `Cell { row: {focused_row} ... }` would require `row` /
  `column` / span attributes to participate in the binding
  pipeline; the validity bounds become **layout-time** rather than
  validate-time for the binding case, and the conflict-detection
  rule must run on every binding-effect re-resolve. Phase 5 ships
  constant-only placement, so this concern is not live.

**Technical risk re-evaluation:**

- **Pairwise rectangle-overlap algorithm scalability.** For
  practical Grid sizes (under 30 `Cell`s; gallery slice: 5),
  pairwise overlap is `O(n^2)` over `n <= 30` and trivially fast.
  For pathologically large `n`, a sweep-line or interval-tree
  algorithm could be substituted without surface change. Phase 5
  records this risk and defers re-evaluation if author demand
  produces Grids with `n > 100`.
- **Conflict-diagnostic locality.** Diagnostics name the two
  conflicting `Cell`s and the shared resolved cell coordinate.
  This is local to the Grid; no cross-Grid diagnostic ambiguity.
- **Out-of-range diagnostics under partially-out-of-range spans.**
  A `Cell` with `column: 1` and `column-span: 3` in a 2-column Grid
  has both a valid `column` and an invalid `column + column-span`.
  The diagnostic surface uses the more specific failure (`column-
  span: 3 exceeds available 1 column from column 1`), not the
  generic "out of range" message. This is a diagnostic-text
  obligation.
- **Span that equals full row / column.** `Cell { column: 0
  column-span: <columns.len()> ... }` is admitted (the spanning
  header in the FD-H gallery proof is exactly this case). The
  resolved rectangle covers the full row; the algorithm does not
  treat full-span as a special case.

**Layering with DD-M3-P5-001 / DD-M3-P5-002 / DD-M3-P5-004 /
DD-M3-P5-005 / DD-M3-P5-006:**

- DD-M3-P5-001 establishes the `Cell` carrier and the IR-side
  attribute defaults.
- DD-M3-P5-002 establishes the track-list value forms (this DD
  consumes `rows.len()` / `columns.len()` for span bounds).
- DD-M3-P5-003 (this DD) establishes the placement / span surface
  and the conflict-detection rule.
- DD-M3-P5-004 consumes the resolved `(row, column, row-span,
  column-span)` rectangles for span reconciliation in track
  resolution.
- DD-M3-P5-005 consumes the resolved rectangle for arrange / cell
  rectangle / alignment-within-cell.
- DD-M3-P5-006 dual-gates the placement / span / conflict
  invariants at `wasamoc check` and `validate()`.

Invalid combinations explicitly rejected by this DD in combination
with downstream DDs:

- DD-M3-P5-003 = same-cell overlap allowed + DD-M3-P5-005 = Grid
  provides no overlay semantics. Recommendation Option A rejects
  same-cell overlap; this combination does not arise.
- DD-M3-P5-003 = spans may exceed declared track count +
  DD-M3-P5-006 = validation-only defense. Recommendation Option A
  reject-at-validate dual-gates the bound; this combination does
  not arise.
- DD-M3-P5-003 = auto-placement admitted + DD-M3-P5-001 = explicit-
  coordinate surface A2 with no document-order placement
  algorithm. Per the framing's invalid-combinations check; this
  DD's placement Option A explicitly rejects auto-placement.
