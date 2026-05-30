## Decisions log

- **T1 — R-A pre-implementation spike: Grid track-list parser grafting
  shape + `n*` lexer-token decision (2026-05-29).** Settles
  [preamble.md risk R-A](./preamble.md#technical-risks-planning-time-recon)
  and the first
  [plan.md T1 bullet](./plan.md#t1--wasamoc-check-grid-surface-and-diagnostics)
  before the implementation bullets open. Spot-checked against
  [`wasamoc/src/parser.rs`](../../../../wasamoc/src/parser.rs),
  [`wasamoc/src/lexer.rs`](../../../../wasamoc/src/lexer.rs), and
  [`wasamoc/src/ast.rs`](../../../../wasamoc/src/ast.rs) at HEAD
  `ce82287`.

  **Problem confirmed.** `parse_property_bind`
  ([parser.rs:233](../../../../wasamoc/src/parser.rs#L233)) reads
  exactly one `Expr` through `parse_expr`
  ([parser.rs:381](../../../../wasamoc/src/parser.rs#L381)), which
  admits only single scalar tokens (`StringLit` / `IntLit` /
  `FloatLit` / `Measurement` / `Ident` / `true` / `false` /
  `RatioLit` / `ColorLit`). A whitespace-separated track list
  (`columns: 180 1* 2*`) cannot ride the generic property-bind path:
  after `180` the `parse_widget_decl` member loop re-enters
  `parse_member`, sees an `IntLit`, and errors `expected member`.
  Separately, the lexer's `'*'` arm
  ([lexer.rs:267](../../../../wasamoc/src/lexer.rs#L267)) errors
  `unexpected '*'` for any `*` not immediately followed by `=`, so
  `*` / `1*` do not tokenize at all today. **A lexer change is
  therefore unavoidable** — the only open question was its shape.

  **Decision 1 — routing: widget-type context in `parse_widget_decl`.**
  Every Grid (component root or nested) is parsed as a
  `Member::WidgetDecl` through `parse_widget_decl`
  ([parser.rs:251](../../../../wasamoc/src/parser.rs#L251)); a Grid's
  `columns:` / `rows:` only ever appear inside that widget body. So
  `parse_widget_decl` is the single, complete routing site. After
  reading `type_name` and `{`, compute `is_grid = type_name ==
  "Grid"` and, in the member loop, route to the Grid-specific
  track-list parser **only** when `is_grid` and the upcoming tokens
  are `Ident("columns" | "rows")` followed by `Colon`; every other
  member (including non-track Grid attributes and all non-Grid
  widgets) stays on `parse_member` unchanged. This is the
  widget-type context routing the risk anticipated; it does **not**
  open a general list/collection grammar (DD-M3-P5-002 Option A) and
  leaves `parse_property_bind` / `parse_expr` untouched. A
  `columns:` / `rows:` attribute on a non-Grid widget keeps its
  current generic-scalar behaviour (and is rejected downstream as an
  unknown attribute), so the track-list grammar never leaks outside
  Grid.

  **Decision 2 — lexer token: payload-less `Token::Star`, no fused
  `n*` literal.** The minimal lexer change wins: the `'*'` arm emits
  a new payload-less `Token::Star` instead of erroring when the next
  char is not `=` (the `*=` → `Token::StarEq` path is untouched).
  The lexer learns *nothing* about track lists — it does not fuse an
  adjacent leading integer into a weighted-star literal. All
  track-grammar knowledge (fixed vs unit-star vs weighted-star,
  weight range `[1, 1024]`, fixed `>= 1`) lives in the Grid-scoped
  parser path, matching DD-M3-P5-002's "narrow Grid-specific parser
  path" boundary. Considered and rejected: fusing `n*` into a single
  `StarLit(weight)` token in `scan_number` (the
  `RatioLit` / `Measurement` precedent). Rejected because
  `RatioLit` / `Measurement` are general-purpose value literals
  usable across many attributes, whereas a weighted-star is
  meaningful *only* inside a Grid track list; putting `n*` fusion in
  the lexer would add a globally-recognised token for a hyper-local
  grammar. The payload-less `Star` is the smallest concession (one
  character that was previously valid only as `*=`); a stray `*` in
  any non-track position now surfaces a parser-level `expected
  expression`/`expected member` diagnostic instead of the old
  lexer-level `unexpected '*'`, which is acceptable (no existing
  valid `.ui` uses a bare `*`).

  **Decision 3 — adjacency via spans distinguishes `1*` from `1 *`.**
  The track grammar must separate `1*` (one weighted-star track of
  weight 1) from `1 *` (a `Fixed(1)` track followed by a unit-star
  track) — both are legitimate but different track lists. With the
  payload-less `Star`, the Grid track-list parser reassembles the
  weighted star by checking byte-span adjacency: an `IntLit(n)`
  immediately followed by `Star` with `int_tok.span.end ==
  star_tok.span.start` lowers to `Star(n)`; otherwise the `IntLit`
  is a `Fixed(n)` track and a following `Star` (non-adjacent or
  standalone) is a unit `Star(1)`. This is the same adjacency
  mechanism `RatioLit` already relies on (whitespace before `:`
  defeats the ratio — see lexer test
  `integer_with_whitespace_before_colon_is_not_ratio`), so the
  `1 *` vs `1*` rule is consistent with existing surface behaviour.
  Weight/fixed value-range validation is deferred to `wasamoc check`
  (`auto` reserved-future and `1.5*` rejections likewise), keeping
  the lexer free of value policy exactly as it is for `RatioLit`
  (`0:0` lexes; validity checked later).

  **Decision 4 — AST carrier: new `Member::GridTracks` variant.** The
  track-list parser produces a new `Member` variant (working shape
  `Member::GridTracks { axis: TrackAxis, tracks: Vec<TrackSize>,
  span }`, with a `wasamoc`-side `TrackSize` AST enum mirroring the
  `wasamo-ir` `TrackSize { Fixed(i32), Star(u32) }` of DD-M3-P5-002)
  rather than reusing `Member::PropertyBind` with a vector-shaped
  `Expr`. This keeps `Expr` strictly scalar and keeps `columns:` /
  `rows:` off the generic property/`IrProp` path, which is the AST-side
  expression of DD-M3-P5-001 carrier **c1** ("Grid track lists live
  outside `IrProp`, in a Grid-specific payload; `IrProp.value` stays
  strictly `IrLiteral`"). Lowering reads `Member::GridTracks` into the
  `KindPayload::Grid { columns, rows }` carrier; non-track Grid
  attributes and all `Cell` attributes continue through the existing
  `PropertyBind` → `IrProp` machinery. Exact field/enum names are
  finalised when the implementation bullets land and are reconciled
  with `wasamo-ir`'s `KindPayload` at that point.

  **Carried into the implementation bullets.** R-C (the
  `kind_payload` construction-site spread) is settled inside the T1
  `wasamoc` emit bullet and the T3 runtime-loader bullet, not in this
  spike; the shared textual IR shape produced by T1 emit feeds the T7
  `dsl_spec.md` §8 fold per the plan.

- **T1 — R-C construction-site discipline + carrier-c1 textual IR
  shape (2026-05-29).** Settles the two items the R-A spike deferred
  to the implementation bullets.

  **R-C disposition: explicit field, no `Default`.** Adding
  `IrNode.kind_payload: Option<KindPayload>` (DD-M3-P5-001 carrier c1)
  touches 5 construction sites workspace-wide: 1 in `wasamoc` lowering
  ([`wasamoc/src/lower.rs`](../../../../wasamoc/src/lower.rs) — the
  Grid arm sets `Some(KindPayload::Grid { .. })`, every other kind
  `None`), 1 production site + 3 round-trip-test sites in the runtime
  loader ([`wasamo-runtime/src/ir_loader.rs`](../../../../wasamo-runtime/src/ir_loader.rs)
  — all `None`; T3 sets the Grid payload). The IR types deliberately
  derive **no** `Default` (matching the existing no-`Default` style —
  no `..Default::default()` appears anywhere in the codebase today),
  so the new field surfaces every site at compile time rather than
  silently defaulting. The R-A spike risk-table option "derive
  `Default` / add an `IrNode::new` builder" was rejected on that
  consistency ground: a compile error at each site is the desired
  forcing function for a structural IR change.

  **Carrier-c1 textual IR shape (T7 §8 fold input).** `wasamoc emit`
  writes Grid track lists as keyword-led lines at the top of the
  Grid node body, parallel to the existing `prop` / `bind` / `on`
  line forms:

  ```text
  node Grid {
      tracks columns = 180 1* 2*
      tracks rows = 1* 1*
      node Cell {
          prop row = 0
          prop column = 0
          node Text { prop text = "header" }
      }
  }
  ```

  - `tracks <axis> = <track-list>` where `<axis>` is `columns` /
    `rows` and `<track-list>` is a space-separated sequence of fixed
    integers and `<weight>*` star tokens. Unit star is canonicalised
    to `1*` (the IR weight is explicit; mirrors the canonical
    color-emit alpha normalisation). Track lists never appear as
    `prop` entries — the carrier-c1 invariant (`IrProp.value` stays
    strictly `IrLiteral`).
  - `Cell` emits as an ordinary `node Cell { … }` subtree carrying
    `prop row`/`column`/`row-span`/`column-span`/`h-align`/`v-align`
    as standard `IrLiteral` entries (Int + Ident). The runtime loader
    flattens Cell subtrees into Grid's effective children + per-Cell
    placement in T3; T1 only fixes the emit shape.

  This textual shape is the concrete target for the T3 runtime-loader
  `tracks` parse and the T7 `dsl_spec.md` §8 grammar / AST fold (both
  deferred from Moment 1 because the carrier's textual form pins at
  implementation time).

- **T2 — R-D mitigation: Grid `LayoutNode` field shape (2026-05-29).**
  Settles [preamble.md risk R-D](./preamble.md#technical-risks-planning-time-recon)
  and the first
  [plan.md T2 bullet](./plan.md#t2--layout-engine-grid-track-resolution-and-arrange)
  before the arrange implementation opens. Spot-checked against
  [`wasamo-runtime/src/layout.rs`](../../../../wasamo-runtime/src/layout.rs)
  (`LayoutNode`, `WidgetKind`, the `Ratio` mirror precedent) and
  [`wasamo-ir/src/lib.rs`](../../../../wasamo-ir/src/lib.rs)
  (`TrackSize` / `KindPayload`) at HEAD `b896f2b`.

  **Decision — extend the flat `LayoutNode` struct (R-D Option:
  flat extension, not enum-shaped refactor).** Grid continues the
  Phase 2–4 flat-struct pattern. Three new fields are added to
  `LayoutNode`, populated only on `WidgetKind::Grid` nodes and left
  empty (`Vec::new()`) on every other kind, mirroring how `aspect` /
  `item_cross_size` / `offset_y` sit dormant off-kind:

  - `grid_columns: Vec<TrackSize>` — per-axis column track list.
  - `grid_rows: Vec<TrackSize>` — per-axis row track list.
  - `cell_placements: Vec<CellPlacement>` — parallel to
    `LayoutNode.children`; `cell_placements[i]` is the placement of
    content child `children[i]`. Document order = children order =
    paint / z-order (DD-M3-P5-005 Option A).

  No arrange-result cache field is added. Unlike Phase 4 ScrollView
  (whose `applied_offset_y` cache bridges a clamp computed at arrange
  and read by `sync_visuals`) and Phase 3 WrapPanel (whose
  `wrap_measured_cross_bound` cache keeps the measure / arrange line
  break in step), Grid's per-Cell content offsets are written
  directly onto each child's `LayoutNode.offset` / `.size` by
  `arrange_grid`, and `sync_visuals` reads those existing fields. The
  Grid outer Visual clip (DD-M3-P5-005) is a fixed
  `InsetClip{0,0,0,0}` requiring no measure→arrange carrier. So the
  R-D "likely an arrange-result cache" is **not** taken: track
  resolution is cheap, deterministic, and re-derivable, and arrange
  is the single resolution site.

  **Layout-engine-local mirror types (the `Ratio` precedent).**
  `layout.rs` imports nothing from `wasamo-ir`; every value type it
  consumes (`SizeConstraint`, `Alignment`, `Ratio`) is layout-local,
  and the `WidgetData` → `LayoutNode` build boundary performs the
  conversion (the same shape as `box_values::Ratio` →
  `layout::Ratio`). Phase 5 follows that style rather than coupling
  the pure layout engine to the IR crate:

  - `layout::TrackSize { Fixed(i32), Star(u32) }` — structural mirror
    of `wasamo_ir::TrackSize`; `Fixed` promoted to `f32` only inside
    `resolve_axis_tracks` per the DD-M3-P5-004 `f32` rounding
    contract.
  - `layout::CellPlacement { row, column, row_span, column_span:
    u32; h_align, v_align: Alignment }` — reuses the existing
    `Alignment` enum (`Leading`=`start`, `Center`, `Trailing`=`end`,
    `Stretch`), so the six new alignment identifier-literals
    (DD-M3-P5-005) need no new layout-engine enum.
  - `layout::AxisBound { Bounded(f32), Unbounded }` — the
    `resolve_axis_tracks` input the DD-M3-P5-004 pseudocode names;
    `arrange_grid` derives it from `w.is_finite()` / `h.is_finite()`.

  The `WidgetData::Grid` → `LayoutNode::grid(...)` build-boundary
  conversion (i32/u32 IR fields → these mirror types) lands in **T3**
  alongside Grid widget-kind materialisation, exactly as Phase 4
  added `LayoutNode::scroll_view` in T2 and wired `build_layout_tree`
  in T3. T2 ships the constructor and the measure / arrange algorithm.
  The T2-era `LayoutNode::grid` constructor and `layout::TrackSize`
  carry `#[allow(dead_code)]` until T3 supplies a production caller —
  a release (non-test) build sees no constructor otherwise, mirroring
  the Phase 4 `scroll_view` forward-pointer. The forward-pointers are
  removed in T3 once `build_layout_tree` constructs Grid nodes.

  **Unbounded-star error timing.** `LayoutError::GridUnboundedStarAxis`
  fires inside `resolve_axis_tracks` (consumed by `arrange_grid`),
  mirroring Phase 4's arrange-time `ScrollViewUnboundedAxis` gate
  rather than a measure-time gate: a measure-time `avail = INFINITY`
  is the standard "how big do you want to be" idiom parents pass on a
  Shrink axis, so firing at measure would reject Grids that arrange
  to a finite cell. The pure-logic tests call `resolve_axis_tracks`
  with `AxisBound::Unbounded` directly to cover the error line (the
  T4 runtime-fixture downgrade clause).

  **Defensive panic retained.** `resolve_axis_tracks` panics if a
  `Star` arm is reached with `star_weight_sum == 0` (only possible
  via a `Star(0)` that DD-M3-P5-002 / DD-M3-P5-006 reject at
  `wasamoc check` / `validate()`). The panic guards the
  division-by-zero the DD-M3-P5-004 algorithm would otherwise hit;
  it is unreachable for validated IR.

- **T3 — R-B pre-implementation spike: Grid `build_node` Cell-flatten
  shape + runtime `tracks` IR-text parse (2026-05-29).** Settles
  [preamble.md risk R-B](./preamble.md#technical-risks-planning-time-recon)
  and the first
  [plan.md T3 bullet](./plan.md#t3--ir-loader--validate-invariant-evidence)
  before the implementation bullets open. Spot-checked against
  [`wasamo-runtime/src/ir_loader.rs`](../../../../wasamo-runtime/src/ir_loader.rs)
  (`tokenize`, `parse_node`, `build_node`, `construct_widget`),
  [`wasamo-runtime/src/widget.rs`](../../../../wasamo-runtime/src/widget.rs)
  (`WidgetData`, `WidgetNode::scroll_view`, `build_layout_tree`),
  [`wasamo-runtime/src/layout.rs`](../../../../wasamo-runtime/src/layout.rs)
  (`LayoutNode::grid`, `TrackSize` / `CellPlacement` / `Alignment`),
  and the T1 emit shape ([log.md T1 R-C entry](#decisions-log)) at
  HEAD `d91b14f`.

  **Decision 1 — `WidgetData::Grid` stores layout-engine mirror types;
  all IR→runtime conversion happens in the loader.** `WidgetData::Grid
  { columns: Vec<layout::TrackSize>, rows: Vec<layout::TrackSize>,
  cell_placements: Vec<layout::CellPlacement> }`. The
  `wasamo_ir::TrackSize` → `layout::TrackSize` conversion and the
  `Cell` `IrProp` → `layout::CellPlacement` extraction both run in
  `ir_loader::construct_widget`'s Grid arm; `build_layout_tree` then
  clones the three vectors straight into `LayoutNode::grid`. This is a
  deliberate refinement of the R-D planning note ("the `WidgetData::Grid`
  → `LayoutNode::grid` build-boundary conversion ... lands in T3"):
  the *track-type* conversion is trivially placeable at either layer,
  but the *placement* extraction (`find` the `row` / `column` /
  `row-span` / `column-span` / `h-align` / `v-align` `IrProp`s, apply
  defaults, map alignment idents) is irreducibly a loader concern —
  the same shape as the existing `extract_ratio_prop` →
  `box_values::Ratio` / `extract_color_prop` boundary. Splitting track
  conversion (build_layout_tree) from placement conversion (loader)
  across two layers would be strictly worse than keeping the loader the
  single "IR → runtime domain type" translation site, so both land in
  the loader and `build_layout_tree` stays a structural copy. A
  consequence: `widget.rs` imports `layout::{TrackSize, CellPlacement}`
  (already imports `layout::{self, Alignment, ...}`) and does **not**
  import `wasamo_ir` — the IR crate coupling stays confined to
  `ir_loader.rs`, matching the `box_values::Ratio` precedent.

  **Decision 2 — Cell flattening is a `build_node`-layer special case;
  `construct_widget` builds the shell *and* the placement vector.**
  Two sites cooperate, both iterating `node.children` (the IR `Cell`
  subtrees) in document order so `children[i]` stays parallel to
  `cell_placements[i]` (the R-D invariant):

  - `construct_widget`'s `"Grid"` arm reads `node.kind_payload` for the
    track lists and walks `node.children` to extract each `Cell`'s
    placement into `Vec<CellPlacement>`, then calls
    `WidgetNode::grid(compositor, columns, rows, cell_placements)`. The
    constructor creates the outer `SpriteVisual` and installs the
    DD-M3-P5-005 outer-bounds clip (`Visual.Clip = InsetClip{0,0,0,0}`,
    the same zero-inset auto-tracking clip ScrollView's outer Visual
    uses) and stores `WidgetData::Grid`. No background brush (Grid is a
    pure layout container).
  - `build_node`'s child loop is branched: for a `"Grid"` node it does
    **not** run the generic `for child in &node.children { build_node;
    append_child }` loop (that would try to materialise `Cell` as a
    widget — `construct_widget` has no `Cell` arm and would
    `UnknownWidget`). Instead, for each `Cell` child it builds the
    Cell's single content child (`cell.children.first()`) via
    `build_node` and appends that content widget to the Grid. `Cell`
    itself never reaches `construct_widget`, so it never materialises as
    a `WidgetNode` or `Visual` (DD-M3-P5-001 "Cell is IR-only").

  This is exactly the R-B-anticipated "`build_node` bypasses the
  generic child append loop; `construct_widget`'s Grid arm only creates
  the widget shell" — with the one refinement that the placement
  *vector* is built in `construct_widget` (which already has `node` and
  must hand `cell_placements` to the constructor), not pushed
  incrementally from `build_node`. The two loops iterate the same
  `node.children` in the same order, so parallelism holds without a
  shared mutable cursor.

  **Decision 3 — runtime `tracks` IR-text grammar: payload-less
  `Token::Star`, whitespace-insensitive weighted star.** The runtime
  IR tokenizer (`ir_loader::tokenize`) gains a payload-less
  `Token::Star`: the `'*'` arm emits `Token::Star` when the next char
  is not `=` instead of erroring (`*=` → `AssignOp(Mul)` is untouched;
  `+` / `/` keep erroring on a bare occurrence). `parse_node` gains a
  `tracks` arm (`tracks <axis> = <track-list>`), and a
  `parse_track_list` greedily consumes `Int` / `Star` tokens into
  `Vec<IrTrackSize>`: an `Int(n)` immediately followed by `Star` lowers
  to `Star(n as u32)`, a standalone `Int(n)` to `Fixed(n)`, and a
  standalone `Star` to `Star(1)`. The loop terminates at the next
  non-`Int`/`Star` token (the next `tracks` / `node` / `prop` keyword
  `Ident`, or `RBrace`), so no newline tracking is needed.

  Unlike `wasamoc`'s author-surface lexer (T1 R-A Decision 3), the
  runtime parser does **not** use byte-span adjacency to distinguish
  `1*` from `1 *`: the runtime IR is the *canonical machine format*
  emitted by `wasamoc` (T1 R-C: unit star canonicalised to `1*`, both
  track lists always present), where the author-surface `1*`-vs-`1 *`
  distinction has already been resolved at compile time. The runtime IR
  `tracks` grammar is therefore whitespace-insensitive: `Int` + `Star`
  (any spacing) = `Star(weight)`. The runtime tokenizer carries no
  spans (it produces `Vec<Token>`), so adding adjacency would be a
  disproportionate change for a distinction the machine format does not
  express. This is the §8 grammar shape the T7 `dsl_spec.md` fold
  records. `kind_payload` is set to `Some(KindPayload::Grid { .. })`
  iff at least one `tracks` line was seen (one-sided / empty track
  lists are left for `validate()` to reject, more lenient than
  `lower.rs`'s both-or-neither panic, since memory IR via
  `wasamo_load_ui` is untrusted).

  **Decision 4 — `validate_phase5_node_invariants` recursion skips
  `Cell` as a standalone node.** The pass mirrors the
  `validate_phase{2,3,4}_node_invariants` shape but routes by kind: a
  `"Grid"` node is validated as a unit (track ranges, min row/col
  count, per-`Cell` child-count + placement/span range + alignment
  vocabulary, pairwise rectangle overlap) and then recursion descends
  only into each `Cell`'s content child; a `"Cell"` node reached by the
  generic recursion (i.e. *not* as a Grid's direct child) is rejected
  as `Cell`-outside-`Grid`. Placement defaults at validate-time match
  the loader (`row` / `column` absent → `0`; `row-span` / `column-span`
  absent → `1`; alignment absent → not checked), so a multi-`Cell` Grid
  that omits placement is caught by the overlap check (two Cells both
  defaulting to `(0,0)`), preserving defense-in-depth without
  re-implementing the compile-time-only multi-Cell placement-presence
  diagnostic (DD-M3-P5-006 marks that row `(n/a)` at runtime). All
  violations surface `IrLoadError::Validate` → `WASAMO_ERR_IR_MALFORMED`.

- **T4 — unbounded star-axis runtime fixture downgraded to pure-logic
  (2026-05-29).** Settles the third
  [plan.md T4 bullet](./plan.md#t4--windows-runtime-layout-and-visual-evidence)
  ("Unbounded star-axis runtime fixture (preferred when ergonomic)"),
  which carries an explicit **downgrade** escape clause. Decision:
  **downgrade** — no Grid-specific Windows integration fixture for the
  unbounded star-axis error is added; the case stays at the pure-logic
  coverage T2 already landed. Mirrors the Phase 4 ScrollView T4
  disposition (`scroll_view_layout_integration.rs` header note).

  **Why no ergonomic `.ui` / IR fixture exists.**
  `LayoutError::GridUnboundedStarAxis` is raised by
  `resolve_axis_tracks` only when an axis carries a star track *and* is
  given `AxisBound::Unbounded`
  ([layout.rs:1072](../../../../wasamo-runtime/src/layout.rs#L1072)).
  Two paths reach that:
  - **arrange-time** (`arrange_grid`): every DSL parent in the Phase 5
    widget catalog (window root via `run_layout` / `run_layout_as_window_root`,
    VStack / HStack main + cross, Box, WrapPanel, ScrollView) passes a
    **finite** cell to its Grid child at arrange — the window root is
    finite, and the stack/box/scroll arrange paths each resolve a
    concrete child rectangle before recursing. No catalog parent hands a
    Grid an unbounded axis at arrange.
  - **measure-time Shrink probe** (`measure_grid` → `grid_shrink_extent`):
    only a `Shrink` Grid axis under an unbounded measure probe reaches
    `resolve_axis_tracks` with `Unbounded`. But DSL-authored `.ui`
    cannot set a Grid's `width` / `height` (dsl_spec §4 exposes no
    sizing attributes), so the loader always builds a Grid as
    `Fill` / `Fill` (`WidgetNode::grid` defaults — widget.rs). A `Fill`
    axis measures to `0.0`, never entering the `Shrink` branch. The
    branch is structurally unreachable from any `.ui` source.

  **Evidence retained.** The two T2 pure-logic tests in
  `wasamo-runtime::layout::tests` —
  `grid_resolve_unbounded_star_axis_errors`
  ([layout.rs:2477](../../../../wasamo-runtime/src/layout.rs#L2477)) and
  `grid_arrange_unbounded_star_axis_errors`
  ([layout.rs:2675](../../../../wasamo-runtime/src/layout.rs#L2675)) —
  pin both the resolve and arrange entry points. The downgrade is
  recorded in the `grid_layout_integration.rs` module header and in the
  T4 step-end retrospective Item 10.

- **T4 — Windows-runtime Grid layout / Visual evidence (2026-05-29).**
  Discharges ADR
  [verification closure evidence item (4)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence).
  Two
  mock-free Windows-only integration fixtures land in
  `wasamo-runtime/tests/grid_layout_integration.rs`:
  - **`grid_rooted_fixture_lays_out_cells_through_visual_tree`** — Grid
    as the component root with mixed fixed + weighted-star tracks on both
    axes (`columns: 100 1*`, `rows: 50 1*`), three Cells (two single +
    one column-spanning), driven through `run_layout_as_window_root`
    (window-root Fill/Fill path).
  - **`grid_vstack_root_fixture_pins_production_root_shape`** —
    `VStack { Button + Grid }` matching the gallery / counter / bool-demo
    production root family, the Phase 4 T6 production-root-shape
    carry-forward (constraints.md §1). Fixed rows (`50 50`) keep the cell
    row origins deterministic even though the VStack-allocated Grid
    outer height is font-metric dependent; the Grid outer height `> 0`
    assertion is the regression gate for the Phase 4 T6 Fill-collapse
    class.

  Both assert the (a)-(d) menu: (a) Grid outer rect = parent allocation;
  (b) each Cell content Visual offset = resolved cell origin (with size
  pinned, stretch alignment expanding to the cell extent including the
  span); (c) Grid outer Visual carries a non-null clip (DD-M3-P5-005
  `InsetClip`); (d) each Cell content Visual has no clip (per-cell
  clipping out of scope — symmetric with the Phase 3 WrapPanel / Phase 4
  ScrollView clip-absence guards). The skip-guard (`init_runtime_or_skip`)
  is reused byte-identically from the Phase 4 T4 scroll fixture; the
  Phase 5 re-confirmation that it fires (test FAILS, not skips) on the
  SSH dev box (`wasamo_init` → `0x80070005`) is the T4 checklist item-4
  owner/environment gate.

- **T5 — End-to-end gallery `.ui` Grid slice + assistant build / launch
  (2026-05-29).** Discharges the **assistant-automated portion** of ADR
  [verification closure evidence item (5)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
  and the **gallery positive-control half** of item (1) (the slice `.ui`
  compiles cleanly through `wasamoc check`). `examples/gallery/gallery.ui`
  grows **additively** with a Grid sibling at the top of the existing
  VStack; the Phase 3 standalone WrapPanel slice and the Phase 4
  ScrollView slice are left byte-identical. Slice shape (FD-H minimum
  visible-proof):
  - 3 rows × 3 columns, mixed fixed + weighted-star per axis
    (`columns: 120 1* 2*`, `rows: 36 1* 36`). The row fixed tracks were
    reduced from an initial `48 1* 64` to `36 1* 36` after Codex review #2
    (Finding 1): under the gallery VStack's modest height allocation the
    larger fixed rows starved the `1*` middle (star) row to a thin strip
    with unreadable labels, weakening the FD-H "fixed and star tracks
    visible in the real `.ui`" proof. Smaller fixed rows hand the
    remaining vertical space to the star row so the middle-row Cells are
    legibly sized in the screenshot;
  - a header `Cell` (`row: 0 column: 0 column-span: 3`) and a footer
    `Cell` (`row: 2 column: 0 column-span: 3`) each spanning all three
    columns (column-span exercised in real `.ui`; row-span discharged by
    T2–T4 per FD-C);
  - three middle-row `Cell`s in separate columns (`row: 1`, columns 0/1/2);
  - every Cell content is `Box { fill: … }` + `Text { text: … }` per the
    Phase 2 DD-M3-P2-006 placeholder pattern (no Image; M4-deferred).
  - **Overflow Cell for the T6 clip observation:** the footer Cell anchors
    its content with `v-align: start` and uses a wide `aspect: 4:1` Box.
    On the bounded width axis the Box stretches to the spanning cell width
    (≈ full Grid width); `aspect: 4:1` then derives a height of
    width / 4, which exceeds the fixed 36 px footer row, so the content
    overflows **below** the footer cell — and, since row 2 is the last
    row, below the Grid's outer rectangle. That overflow is the live
    target the T6 owner smoke uses to observe the DD-M3-P5-005
    outer-bounds clip on the real binary (per `arrange_grid`'s
    non-stretch-axis natural-extent measure + `align_in_cell` no-clamp
    behaviour, `layout.rs`). DSL has no comment syntax, so the rationale
    lives here rather than inline in the `.ui`.

  Verified `target/release/wasamoc.exe check examples/gallery/gallery.ui`
  exits 0 (positive control) and `cargo build -p gallery-rust --release`
  green (only the pre-existing benign "wasamo provides no linkable target"
  warning).

  **Assistant visual-evidence baseline (Codex review #1, 2026-05-29).**
  Codex flagged that `Start-Process` survival alone cannot show the Grid
  slice actually rendered, the initial screen is non-blank, or the
  intended sub-screen is in the viewport. The assistant-automated evidence
  for a GUI host is therefore strengthened to **launch + screenshot
  capture + assistant analysis**; `Start-Process` "stays running" is
  demoted to a supporting "no early crash" signal. Procedure used:
  `Start-Process target/release/gallery-rust.exe`, poll for
  `MainWindowHandle`, bring the window foreground + topmost, then
  `Graphics.CopyFromScreen` over its `GetWindowRect` (CopyFromScreen, not
  `PrintWindow`, because the Visual-Layer / DirectComposition client area
  reads back blank under `PrintWindow`). Artifact stored per workflow.md
  §5.4 (`implementation/evidence/`, `tN-<purpose>.<ext>`):
  [evidence/t5-gallery-grid-launch.png](./evidence/t5-gallery-grid-launch.png)
  (800×600 window capture; re-captured after the Finding-1 row-track fix).
  Assistant analysis of the image confirms:
  - the initial screen is **non-blank** (Composition rendering works);
  - the header `Cell` spans all three columns (the full-width band reads
    "Header (spans 3 columns)");
  - the three middle-row `Cell`s render as three separate columns with
    **legible** labels — "C0 fixed 120" (the narrow fixed 120 px column),
    "C1 star 1*", and "C2 star 2*" with the `2*` column visibly ~2× the
    width of the `1*` column, so the mixed fixed + weighted-star column
    proof and the vertical star row are both clearly visible;
  - the footer `Cell` spans the full width (magenta band) and its
    `aspect: 4:1` content overflows downward, cut off where the WrapPanel
    begins — the **Grid outer-bounds clip baseline** for the T6 owner
    smoke;
  - the untouched Phase 3 WrapPanel slice ("Photo 1–10") and the buttons
    still render. (The mica title bar shows the editor behind it — expected
    `backdrop: mica` translucency, not the opaque client area.)

  This assistant analysis is a **pre-T6 automated baseline**, not a
  replacement for the owner's visible-correctness judgment (T6 per FD-I);
  it does not do pixel-level track-width verification or exact clip-edge
  measurement. C / Zig gallery hosts remain out of Phase 5 scope.

- **T7 — Moment 2 implementation-sync + phase-close gates
  (2026-05-30).** Closes Phase 5's last step. Discharges the Moment 2
  doc re-sync and the m3-plan phase-end progress flips; the on-CI
  gates and the front-matter `active` → `closing` flip remain `[ ]`
  pending the push gate (separate from merge per
  [retrospectives.md §進行手順](../../../procedures/retrospectives.md)).
  Target commits on `feat/m3-phase-5-t7`:
  - `3bb1608 docs(m3-phase-5): T7 Moment 2 spec sync — Grid
    implementation-synced` (dsl_spec v1.3 → v1.4: §4.12 + header status
    → "closed; implementation-synced", §8.5 `track_decl` fold, §5/§2.2/§3
    earlier-phase spec-gap folds with owner confirmation; architecture
    top Status → "M3-Phase 5 complete" + §6.8.7 re-sync to the landed
    `WidgetData::Grid { columns, rows, cell_placements }`; abi_spec
    re-confirmed untouched). `Reviewed-by: codex`.
  - progress-doc flips (this batch): `process/milestone-3/plan.md`
    Phase 5 row Status → `complete` with the completed-row Notes
    pattern; `phase-5/implementation/plan.md` T7 doc-status checkboxes
    flipped (§4.12 / §8 / architecture / m3-plan row / abi_spec).
  - retro batch: this `log.md` entry +
    [retrospectives/t7.md](../retrospectives/t7.md) step-end retro.

  **Plan revision — T7-list ownership correction (option A,
  owner-approved 2026-05-30).** The T7 task list as frozen at T0
  carried two bullets — phase-`sync` close (then "no open phase-sync
  items survive past phase close") and the `handoff.md` carry-forward
  write-up — as **T7** deliverables. This conflicts with
  [retrospectives.md §15 / §6.3](../../../procedures/retrospectives.md),
  which assign the final `carry-forward` close and the `handoff.md`
  clean-up to the **phase-end** retro (item 15), and with the later
  [T6 retro owner decision (2026-05-30)](../retrospectives/t6.md)
  that the DPI carry-forward `handoff.md` entry lands at phase-end and
  "plan.md T7 へ owning bullet は追記しない". The T0 plan list predated
  that owner decision, so it carried stale ownership. Resolved in
  favour of the procedure SSOT + later owner decision: both bullets
  are re-tagged **NOT owned by T7** and stay `[ ]` at T7 close, like
  the phase-end retro bullet. T7's own `phase-sync` dispositions are
  recorded in the T7 step-end retro item 10; the phase-end retro
  performs the final close + `handoff.md` clean-up. (Precedent for
  recording an in-flight plan-list ownership correction in the log:
  M3-Phase 4 "T5/T6 split for owner-manual GUI smoke".) The
  `phase-5/implementation/plan.md` is the mutable phase plan
  (CLAUDE.md "Mutable during the phase"), so this revise is in-rule;
  the frozen milestone `plan.md` was touched only for the sanctioned
  Moment-2 row Status flip.

  **Post-commit clean rebuild (item 3 evidence).** Run on the T7
  working tree; all T7 changes are doc-only (`docs/` + `process/`, no
  Rust `src/` / `Cargo.toml` / `build.rs` / CI YAML), so the build /
  test state is identical to committed `3bb1608` and the T6 HEAD:
  - `cargo fmt --all -- --check` → exit 0;
  - `cargo clean` (removed 3367 files, 1.1 GiB) →
    `cargo build --workspace` (debug, 41.13s, green) →
    `cargo build --release --workspace` (46.55s, green) →
    `cargo test --workspace` → **627 passed / 0 failed** (16 wasamo-ir
    + 301 wasamo-runtime lib + 282 wasamoc lib + 28 integration/
    roundtrip; identical to T6's 627, +0, confirming the Moment-2 sync
    is doc-only). No new warnings (pre-existing "wasamo provides no
    linkable target" only).
  - **On-CI** clean rebuild + the Windows-only integration evidence
    (skip-guard verified per T4) are the phase-end gate (item 16),
    run from `workflow_dispatch` on the phase branch **before merge**;
    the local rebuild here is the proxy until then. CI YAML unchanged
    (no new language / build system).

- **Phase-end close (2026-05-30).** Phase 5 phase-end gate per
  [retrospectives.md items 12-18](../../../procedures/retrospectives.md),
  recorded in
  [retrospectives/phase-end.md](../retrospectives/phase-end.md).
  - **CI evidence (item 16).** Phase-branch `feat/m3-phase-5`
    (headSha `ca711bd`) `workflow_dispatch` run
    [`26683352589`](https://github.com/matarillo/wasamo/actions/runs/26683352589)
    conclusion **success** (~2m31s): `cargo fmt`/ debug + release build /
    `cargo test` / C-ABI (cl + clang-cl) / CMake / Zig / counter-c/rust/zig
    smoke all green. Local clean rebuild proxy (T7): 627 passed / 0 failed.
    The phase-end doc commits after this CI run are doc-only, so the run
    is the merge-gate ground truth.
  - **Implementation summary (T1-T6).** `wasamoc` registers `Grid`,
    parses `columns:` / `rows:` track lists through a narrow Grid-scoped
    path (bare `*` token; `1*` vs `1 *` by span adjacency) and emits the
    carrier-c1 `tracks <axis> = …` textual IR (T1). The pure-data layout
    engine resolves per-axis fixed-first + weighted-star tracks with `f32`
    prefix boundaries, reconciles spanning, and applies per-Cell alignment
    in document order (T2). The runtime loader materialises
    `KindPayload::Grid`, flattens each `Cell`'s content child into Grid's
    effective children with a parallel `cell_placements` vector, installs
    the outer-bounds `InsetClip`, and validates defense-in-depth (T3).
    Two mock-free Windows-runtime fixtures pin the Grid Visual tree and
    the production VStack-root shape (T4). The gallery `.ui` grew a 3×3
    Grid slice (T5), owner-accepted in the T6 visible smoke with resize
    positive controls. No new `IrType` / `IrLiteral` / `PropertyValue` /
    `WASAMO_*` ABI surface — `docs/abi_spec.md` re-confirmed untouched at
    phase-end (code-level: Grid's track lists ride the `KindPayload::Grid`
    carrier on `IrNode`, `LayoutError::GridUnboundedStarAxis` is
    runtime-internal, and no C-ABI export crate / bindings changed).
  - **phase-sync dispositions closed (item 15).** #1 Grid §8 grammar
    `doc-folded` (dsl_spec §8.5); #2 assistant-visible evidence + #4
    positive-control `doc-folded` (CLAUDE.md §Testing rules +
    verification-environments.md Obs 4, commits `b8b1f53` / `d83e446`);
    DPI `carry-forward` → M4 (handoff + VDR `2162867`); R1 residual →
    Phase 6; remainder `local-only`. No open `phase-sync` items survive.
  - **Post-merge distillation.** _(appended after the main no-ff merge:
    merge commit hash + post-merge main CI green re-confirmation.)_
