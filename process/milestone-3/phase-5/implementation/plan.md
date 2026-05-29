## Task list

### T0 — Moment 1 document sync

Opens execution after ADR acceptance and records the design draft in
the upstream documents named by the ADR's
[Moment 1 commit set](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2).
T0 closes when this implementation plan lands and all preceding
Moment 1 commits are on the pre-doc branch.

- [x] `process/milestone-3/phase-5/decisions/preamble.md` and
      `dd-m3-p5-001` through `dd-m3-p5-006` flipped to
      `Status: Accepted` (commit `f4f741f`, 2026-05-28).
- [x] `docs/dsl_spec.md` §4.12 Grid layout primitive chapter added
      as design-spec draft with Phase status marker `M3-Phase 5
      design accepted; implementation pending`; §4.4 widget registry
      row added for `Grid` only with `Cell` pointer to §4.12; top
      Status header advanced to include `M3-Phase 5 design accepted
      (implementation pending)`; revision history v1.3 entry
      (commit `5ea39a6`, 2026-05-29).
- [x] `docs/dsl_spec.md` §8.11 Loader validation policy retroactive
      spec-gap fold: extended from Phase 2-only to cover Phase 3
      WrapPanel non-negative attributes (DD-M3-P3-006), Phase 4
      ScrollView single-content-child (DD-M3-P4-006), and Phase 5
      Grid / Cell invariants (DD-M3-P5-006), per owner review on
      2026-05-29 (commit `5ea39a6`).
- [x] `docs/architecture.md` §6.8.7 Phase 5 Grid paragraph added
      (IrNode kind-payload carrier c1; `IrProp.value` strictly
      `IrLiteral`; Cell IR-only; constant-only attributes preserving
      the per-type writer seam; no §6.5 sync-visuals change); top
      Status header advanced to include `M3-Phase 5 design accepted
      (implementation pending)` (commit `5ea39a6`).
- [x] `process/milestone-3/plan.md` Phase 5 row populated
      (Status `in progress`; Progress file and ADR links wired
      to this directory and the ADR set); Phase 6 row Notes record
      the M3-Phase 4 R1 (Window title wiring) owning-phase cross-
      reference per M3-Phase 5 FD-E; Phase 1-4 Tracking-table stale
      links retroactively fixed in the same commit per owner review
      (commit `f2998f2`, 2026-05-29).
- [x] `process/milestone-3/phase-5/implementation/preamble.md`
      and this `plan.md` opened with `status: active` and the FD-C
      / FD-G / constraints §5 step-end / phase-end retrospective
      split represented in T7 from the start (THIS commit).
- [x] `docs/abi_spec.md` deliberately untouched at Moment 1 per
      DD-M3-P5-001 / DD-M3-P5-006: Grid adds no host-facing ABI
      surface; `LayoutError::GridUnboundedStarAxis` is
      runtime-internal and no `WASAMO_LAYOUT_ERROR_*` extension is
      introduced; no `PropertyValue` / `IrType` / `IrLiteral`
      variant added.

### T1 — `wasamoc check`: Grid surface and diagnostics

Discharges the **representative-fixture portion** of ADR
[verification closure evidence item (1)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
(representative fixed + weighted-star + spanning fixtures in unit
tests; all reject / diagnostic cases for the surface). The **gallery
Grid slice positive control** half of item (1) closes at T5 (the
slice's `.ui` compiles cleanly); the **evidence aggregation** for
item (1) closes at T7 alongside the CI gates.

- [x] **Pre-implementation spike** for risk
      [R-A](./preamble.md#technical-risks-planning-time-recon):
      settle the grafting shape of the narrow Grid-specific
      track-list parser path against `parse_widget_decl` /
      `parse_property_bind` (widget-type context routing) and
      decide whether `n*` admits a new lexer token, before opening
      the bullets below. Record the chosen shape in
      [log.md](./log.md). **Settled 2026-05-29** (see log.md
      T1 R-A entry): widget-type routing in `parse_widget_decl`;
      payload-less `Token::Star` (no fused `n*` literal); `1*` vs
      `1 *` distinguished by span adjacency; new `Member::GridTracks`
      AST carrier.
- [x] Register `Grid` in `wasamoc`'s known widget registry / check
      surface; recognise `Cell` as a Grid-internal IR node kind
      that is rejected outside a `Grid` parent per DD-M3-P5-001 /
      DD-M3-P5-006. (Runtime widget-kind materialisation is T3.)
      `Grid` added to `KNOWN_WIDGET_TYPES`; `Cell` special-cased in
      the `check.rs` WidgetDecl arm (no unknown-widget warning;
      `Cell`-outside-`Grid` rejected).
- [x] Add the narrow Grid-specific track-list parser path for
      `columns:` / `rows:` per DD-M3-P5-002 (does not open a general
      list / collection grammar). Widget-type routing in
      `parse_widget_decl` → `parse_grid_track_list`; payload-less
      `Token::Star` (lexer) + span adjacency per the R-A spike.
- [x] Implement Surface A2 Grid / Cell check-side diagnostics per
      DD-M3-P5-001 / DD-M3-P5-002 / DD-M3-P5-003 / DD-M3-P5-005 /
      DD-M3-P5-006 (track-list shape including reserved-future
      `auto` diagnostic; placement-attribute presence with the
      single-Cell escape clause; Cell single-child; placement / span
      value range; same-cell / overlapping-rectangle conflict;
      unknown-attribute rejection on Grid and Cell; alignment-value
      vocabulary). `check_grid` / `check_cell` in `check.rs`.
- [x] Add `wasamoc` positive / negative tests covering the
      representative-fixture half of ADR evidence item (1). Positive
      controls (fixed + weighted-star + spanning) and a reject case
      per diagnostic land inline in `parser.rs` / `check.rs` /
      `lower.rs` / `emit.rs` test modules.
- [x] `wasamoc` emits the Grid carrier c1 (per DD-M3-P5-001) to
      textual IR in a Phase-5 implementation shape so that
      `IrProp.value` stays strictly `IrLiteral` and Grid's
      `columns:` / `rows:` track lists live in the Grid-specific
      kind payload, not in `IrProp` entries. The final textual
      shape feeds the T7 `dsl_spec.md` §8 fold. Emitted as
      `tracks <axis> = <track-list>` lines (`emit.rs`); unit star is
      canonicalised to `1*`.

### T2 — Layout engine: Grid track-resolution and arrange

Discharges ADR
[verification closure evidence item (2)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence).
Per
[CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules),
the layout engine is pure logic; tests are pure-logic unit tests on
the algorithm's `(input → output)` shape.

- [x] **Mitigation for risk
      [R-D](./preamble.md#technical-risks-planning-time-recon):**
      settle and record the Grid data shape across `WidgetData::Grid`
      → `LayoutNode` (`Vec<TrackSize>` × 2, `Vec<CellPlacement>`,
      arrange-result cache) before implementing arrange. Decide
      whether to extend the existing flat-struct `LayoutNode`
      pattern (Phase 5 default per ADR) or escalate; record the
      chosen field shape in [log.md](./log.md). **Settled
      2026-05-29** (see log.md T2 R-D entry): flat-struct extension
      (`grid_columns` / `grid_rows: Vec<TrackSize>`,
      `cell_placements: Vec<CellPlacement>`); **no** arrange-result
      cache (per-child offsets written directly, re-derivable);
      layout-engine-local mirror types per the `Ratio` precedent
      (`layout::TrackSize` / `CellPlacement` reusing `Alignment` /
      `AxisBound`); `GridUnboundedStarAxis` fires at arrange-time per
      the ScrollView precedent. The `WidgetData::Grid` →
      `LayoutNode::grid` build boundary lands in T3.
- [x] Implement Grid measure / arrange in
      `wasamo-runtime/src/layout.rs` per DD-M3-P5-004 (per-axis
      fixed-first + weighted-star track resolution; `f32` prefix
      boundaries; spanning reconciliation; `LayoutError::GridUnboundedStarAxis`;
      reserved no-op slot before star distribution per
      DD-M3-P5-002's `auto` deferral) and DD-M3-P5-005 (per-Cell
      `h-align` / `v-align` with stretch default; layout-side
      outer-bounds-rect invariant; document-order z-order).
      `resolve_axis_tracks` / `prefix_boundaries` / `measure_grid` /
      `arrange_grid` / `align_in_cell` in `layout.rs`;
      `WidgetKind::Grid` + `LayoutError::GridUnboundedStarAxis` added
      (the latter also wired into `layout_error_to_winerr`).
- [x] Add pure-logic tests covering ADR evidence item (2). The
      Visual-side clip-install assertion is T4's responsibility,
      not T2's. 18 tests in `layout.rs` (fixed-only / weighted-star /
      mixed / both-axis spanning / negative-remaining / unbounded
      star-axis / per-Cell alignment incl. mixed / non-stretch-axis
      natural-extent measure (aspect Box) / layout-side
      outer-bounds-rect invariant / prefix boundaries / `Star(0)`
      defensive panic). The **layout-side** document-order substrate
      is covered (children/placement correspondence preserved in
      arrange order; overflowing cells produce overlapping geometry
      in document order); the **visible paint-precedence** half of
      z-order (later child on top under overlap = Visual-tree
      insertion order) is owned by the T6 smoke observation point
      ("document-order paint order is observed when overlapping
      content occurs"), not asserted in pure logic.
- [x] All-zero star sum cannot arise after DD-M3-P5-002 /
      DD-M3-P5-006 validate-time rejection; the corresponding
      layout-time defensive panic per DD-M3-P5-004 is retained.
      `resolve_axis_tracks` panics on a `Star` arm with
      `star_weight_sum == 0`; covered by the `should_panic` test.
- [x] Prefer pure free-function extraction. The test-module-only
      mirror struct pattern is reserved per
      [CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules)
      and used only if entanglement with a Win32 / WinRT-bound type
      prevents extraction. All Grid logic lives in free functions on
      the Win32/WinRT-free `layout.rs`; no mirror struct was needed.

### T3 — IR loader / `validate()` invariant evidence

Discharges ADR
[verification closure evidence item (3)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence).

- [x] **Pre-implementation spike** for risk
      [R-B](./preamble.md#technical-risks-planning-time-recon):
      settle how the **Grid path in `build_node`** bypasses the
      generic child append loop so IR Cell subtrees flatten into
      `WidgetNode` children + per-Cell `Vec<CellPlacement>` on
      `WidgetData::Grid` (`construct_widget`'s Grid arm only creates
      the Grid widget shell). Settle before opening the bullets
      below; record the chosen shape in [log.md](./log.md).
      **Settled 2026-05-29** (see log.md T3 R-B entry):
      `WidgetData::Grid` stores `layout` mirror types (loader does all
      IR→runtime conversion; `build_layout_tree` is a structural copy);
      `construct_widget` builds the shell + clip + placement vector and
      `build_node` branches the Grid child loop to flatten Cell content
      children; payload-less runtime `Token::Star` +
      whitespace-insensitive `tracks` parse; `validate_phase5` routes by
      kind and skips `Cell` as a standalone node.
- [x] Materialise `Grid` as a runtime widget kind backed by
      `KindPayload::Grid { columns, rows }` per DD-M3-P5-001 carrier
      c1 (`IrProp.value` stays strictly `IrLiteral`). Runtime loader
      parses the Phase-5 textual IR shape produced by `wasamoc` in
      T1 into the kind payload; the shared textual shape feeds the
      T7 `dsl_spec.md` §8 fold. Wire the `WidgetData::Grid` →
      `LayoutNode::grid` build boundary (`build_layout_tree`),
      constructing `cell_placements` parallel to `children`
      (log.md T2 R-D entry), and **remove the T2-era
      `#[allow(dead_code)]` forward-pointers** on `LayoutNode::grid`
      / `layout::TrackSize` once production has a caller.
      Runtime `tokenize` gained a payload-less `Token::Star` and
      `parse_node` a `tracks <axis> = …` arm
      (`parse_tracks_line`); `WidgetData::Grid` stores the layout
      mirror types and `WidgetNode::grid` installs the outer-bounds
      `InsetClip`; `build_layout_tree` Grid arm clones into
      `LayoutNode::grid`; both `#[allow(dead_code)]` removed.
- [x] `Cell` IR-loader path reads placement / span / alignment from
      standard `IrProp` entries (Int + Ident literals) and arranges
      each Cell's single content child as Grid's effective layout
      child. `Cell` is **not** registered in the runtime widget
      catalog. `extract_cell_placement` / `extract_alignment_prop`
      build the per-Cell `CellPlacement`; `build_node` branches the
      Grid child loop to flatten each Cell's single content child
      (R-B Decision 2); `construct_widget` has no `Cell` arm.
- [x] Implement runtime `validate()` defense-in-depth per
      DD-M3-P5-006 invariant table (track value range; placement /
      span value range; Cell single-content-child; same-cell /
      overlapping-rectangle conflict; alignment-value vocabulary;
      `Cell`-outside-`Grid`). All violations surface
      `WASAMO_ERR_IR_MALFORMED`; all checks are
      **reject-at-validate**, not clamp-at-arrange (the only
      layout-time gate is T2's `LayoutError::GridUnboundedStarAxis`).
      No `docs/abi_spec.md` change and no new ABI tag.
      `validate_phase5_node_invariants` / `validate_grid_invariants`
      / `validate_grid_cell` added and wired into `validate()`;
      `docs/abi_spec.md` untouched.
- [x] Add pure-logic tests covering ADR evidence item (3).
      25 new `ir_loader` tests (tracks parse incl. bare-`*` + `*=`
      regression; positive control; min row/col; track range;
      placement range; span range; Cell child-count; same-cell /
      overlapping-span conflict; multi-Cell omitted-placement
      origin collision; alignment vocabulary; non-Cell Grid child;
      Cell-outside-Grid; tracks-less Grid; and — per owner review #1 —
      non-Grid `kind_payload` rejection at parse + `validate()` for
      both a non-Grid node and a Cell, closing the carrier-c1
      Grid-only invariant gap).

### T4 — Windows-runtime layout and Visual evidence

Discharges ADR
[verification closure evidence item (4)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
and applies the Phase 4 T6 production-root-shape carry-forward per
[../requirements/constraints.md §1](../requirements/constraints.md#1-integration-test-fixture-parent-shape-は-production-root-shape-を必ずカバーする).
Mock-free Windows-runtime integration tests per
[CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules);
skip-guard inherits the Phase 2 T11 / Phase 3 / Phase 4 pattern
(fires on `0x80070005` from `wasamo_init`) and **fails** rather than
silently skips on a runner that cannot create the Compositor.

- [x] **Grid-rooted fixture (window-root Fill/Fill path).** `.ui`
      declares a Grid as the root widget with mixed fixed and
      weighted-star tracks containing `Cell { Box { ... } }`
      children in known cells. Drives
      `WidgetNode::run_layout_as_window_root` (Phase 4 T6 entry
      point). Asserts:
      - (a) Grid's resolved rectangle matches the parent allocation.
      - (b) Each Cell's content Visual offset matches the resolved
        cell rectangle origin (with parent-relative offsets per
        [`docs/architecture.md` §6.5](../../../../docs/architecture.md#65-widgetnode-and-visual-layer-sync)).
      - (c) Grid's outer Visual has a **non-null** clip (the
        `InsetClip` from DD-M3-P5-005) — clip presence assertion.
      - (d) Each Cell content Visual has `Visual.Clip = null` —
        clip absence regression guard, symmetric with the Phase 3
        T8 WrapPanel and Phase 4 ScrollView precedents.
      `grid_rooted_fixture_lays_out_cells_through_visual_tree` in
      `wasamo-runtime/tests/grid_layout_integration.rs` (Grid root,
      `columns: 100 1*` / `rows: 50 1*`, three Cells incl. a
      column-spanning footer; (a)-(d) plus size pinned on each cell).
- [x] **`VStack { Grid { ... } }` fixture (production root shape).**
      Same set of assertions (a)-(d) on the inner Grid. Matches the
      current gallery / counter / bool-demo `.ui` production root
      family and guards against the Phase 4 T6 runtime-boundary
      collapse class.
      `grid_vstack_root_fixture_pins_production_root_shape` (Button +
      Grid under a VStack root; fixed `rows: 50 50` keep cell row
      origins deterministic against the font-metric-dependent Grid
      outer height; Grid outer height `> 0` is the T6 Fill-collapse
      regression gate).
- [x] **Unbounded star-axis runtime fixture (preferred when
      ergonomic).** `.ui` places a Grid with at least one star track
      inside a parent whose corresponding axis is unbounded
      (synthesisable by embedding in an intrinsic-measure context).
      Assert layout pass returns
      `Err(LayoutError::GridUnboundedStarAxis)`. If no ergonomic
      IR-level fixture exists, **downgrade** this case to pure-logic
      coverage in T2 and record the decision under the Decisions
      log in [log.md](./log.md).
      **Downgraded** to pure-logic coverage (no DSL/IR parent passes a
      Grid an unbounded axis at arrange, and a Grid is always
      `Fill`/`Fill` so the `measure_grid` Shrink probe is unreachable
      from `.ui`); T2's `grid_resolve_unbounded_star_axis_errors` /
      `grid_arrange_unbounded_star_axis_errors` retain the coverage.
      Decision recorded in [log.md](./log.md) (T4 downgrade entry),
      mirroring the Phase 4 ScrollView T4 disposition.
- [x] Verify the skip-guard fires (i.e. test FAILS, not skips) on
      an environment where `wasamo_init` returns `0x80070005`,
      before landing T4 — per
      [CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules).
      Local "passed without skip" does not prove the guard.
      Discharged by inheritance per the Phase 4 T4 disposition:
      `init_runtime_or_skip` is reused **byte-identically** (same
      `wasamo_init` surface, same `0x80070005` predicate, same
      GitHub-Actions fail assert) from the SSH-dev-box-verified Phase 2
      T11 / Phase 3 T8 / Phase 4 T4 guard, and Phase 5 introduces **no
      new runtime capability path** (the sole `wasamo_init` failure
      predicate is the same HRESULT). No new guard logic was authored,
      so there is no new triggering branch to re-verify.

### T5 — End-to-end gallery `.ui` and assistant-side build / launch

Discharges the **assistant-automated portion** of ADR
[verification closure evidence item (5)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
and the **gallery positive-control portion** of item (1) — the slice
`.ui` compiles cleanly through `wasamoc check` (T1 closed the
representative-fixture / diagnostic halves of item (1) earlier; the
gallery slice closes the positive-control half here). The visible-
correctness portion of item (5) (owner-manual GUI smoke) is owned by
T6 per FD-I. T5's assistant-automated evidence is **build + launch +
launch-time screenshot capture + assistant analysis** (per Codex
review #1); `Start-Process` survival is the supporting "no early crash"
signal and the assistant analysis is a pre-T6 baseline, not a substitute
for the owner's visible-correctness judgment.

- [x] Grow `examples/gallery/gallery.ui` **additively** with a
      sibling slice containing a Grid composition matching the FD-H
      minimum visible-proof shape: a 3-row × 3-column Grid with at
      least five Cells —
      - one header `Cell` spanning all three columns,
      - three middle-row `Cell`s in separate columns,
      - one footer `Cell` spanning all three columns,
      - mixed fixed + at least one weighted-star track per axis,
      - exercising column-span in the real `.ui` (row-span discharged
        by T2-T4 per FD-C).
      The Phase 3 standalone WrapPanel slice and Phase 4 ScrollView
      slice stay untouched.
      Grid sibling added at the top of the gallery VStack
      (`columns: 120 1* 2*`, `rows: 36 1* 36`; header + footer
      `column-span: 3`; three `row: 1` middle Cells in columns 0/1/2);
      existing WrapPanel / ScrollView slices left byte-identical. (Row
      fixed tracks reduced from `48 1* 64` to `36 1* 36` per Codex review
      #2 Finding 1 so the `1*` middle/star row is legibly sized in the
      screenshot.)
- [x] Cell content uses `Box { fill: ... }` + `Text { text: ... }`
      per the Phase 2 DD-M3-P2-006 placeholder pattern. No Image
      widget (deferred to M4 per Phase 2).
- [x] Include at least one Cell whose content intentionally overflows
      its resolved rectangle (e.g. fixed-track sum exceeding Grid's
      bounded extent, or a Cell with content larger than its cell)
      so the **Grid outer-bounds clip is visibly observable** in T6
      smoke. Without this, the T6 clip-visibility observation point
      cannot be exercised on the live binary.
      Footer Cell uses `v-align: start` + a wide `aspect: 4:1` Box: the
      aspect-derived height (cell width / 4) exceeds the fixed 36 px
      footer row, overflowing below the last row = below the Grid outer
      rectangle (see [log.md](./log.md) T5 entry).
- [x] Build and run `examples/gallery-rust/`. Record assistant-automated
      visual evidence (launch + screenshot capture + assistant analysis,
      per Codex review #1); `Start-Process` survival is the supporting
      "no early crash" signal. C / Zig gallery hosts remain out of Phase 5
      scope.
      `wasamoc check` exit 0; `cargo build -p gallery-rust --release`
      green; screenshot
      [evidence/t5-gallery-grid-launch.png](./evidence/t5-gallery-grid-launch.png)
      captured at launch and analysed by the assistant — non-blank screen,
      header 3-col span, three separate middle-row columns with legible
      labels ("C0 fixed 120" / "C1 star 1*" / "C2 star 2*", the 2* column
      visibly ~2× the 1* column), footer span + outer-bounds clip baseline,
      untouched WrapPanel/buttons (see [log.md](./log.md) T5 entry). Visible-correctness judgment is T6
      owner-owned (the assistant analysis is a pre-T6 baseline, not a
      substitute).

### T6 — Owner-manual GUI smoke and any visible-correctness fix

Discharges the **visible-correctness portion** of ADR
[verification closure evidence item (5)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
and the A11 gallery-proof owner-acceptance half. This step exists so
visible smoke is verified — and fixed if it fails — **before** any
phase-close mechanical work (spec / plan status flips) lands in T7,
matching the Phase 4 T5 / T6 split rationale.

- [x] Owner runs `target/release/gallery-rust.exe` (or builds-and-
      runs `cargo run -p gallery-rust --release` if the T5 binary is no
      longer on disk). The window title reads **"Wasamo"**, not the
      `.ui` `title: "Gallery"`, because Window-title wiring is the
      unresolved M3-Phase 4 R1 carried to Phase 6 (FD-E / `m3-plan.md`
      Phase 6 row Notes) — a **known residual, NOT a Grid-smoke
      pass/fail criterion**. The host is DPI-unaware on this
      125%-scaled box (logical 800×600 bitmap-stretched to physical
      1000×750); this affects only assistant screenshot tooling
      (capture must be per-monitor-DPI-aware), not the owner's direct
      on-screen viewing. Observation points:
      - column tracks are fixed vs. variable — **resize observation
        required** (a single launch size only shows the `C2`:`C1`
        ratio, not that `C1` / `C2` are flexible star tracks; fixed
        widths could coincidentally match the ratio):
        - at launch: `C0` is the narrowest; `C2` ≈ 2× `C1`;
        - widen / narrow the window: `C0` stays ≈120 logical px
          (constant), while `C1` / `C2` grow / shrink with the slack
          and keep `C2`:`C1` ≈ 2:1.
        (Assistant baseline confirms: `C0` held ~150 physical px from
        an 820→1500 px window while the stars absorbed the slack, and
        the Photo WrapPanel reflowed 6→10 per row — real re-layout, not
        a bitmap stretch.)
      - the spanning header `Cell` spans all three columns;
      - the three middle-row `Cell`s occupy separate columns;
      - the spanning footer `Cell` spans all three columns;
      - Grid outer-bounds clip — verify against the `gallery.ui`
        source, not merely "cut sharply" (clipped content is invisible,
        so the criterion is what is MISSING): the footer `Cell` declares
        `Box { aspect: 4:1 } + Text` spanning all three columns, whose
        natural size is the full Grid width × (width / 4) ≈ a ~190
        logical-px-tall pink rectangle carrying a centred label. On
        screen the outer-bounds clip leaves only a **thin ~36 px
        (one-footer-row) pink strip with NO visible text** — the centred
        Text sits ~95 logical px down in the box, below the Grid's
        bottom edge, and is removed by the clip — and the pink **does
        not bleed into the gap / Photo row below**.
        Positive control via resize: widening the window grows the 4:1
        box's natural height, but the visible pink strip stays pinned at
        the footer-row height (clipped to the Grid bottom) and never
        grows downward over the Photos. (Assistant baseline: the strip
        held ~45 physical px from an 820→1500 px window while the box's
        natural height ~doubled, and the text never appeared.) The
        definitive clip-PRESENCE proof is the T4 integration test (Grid
        outer Visual has a non-null `InsetClip`) + T2 overflow geometry;
        this owner-visible check corroborates it.
      - document-order paint order under overlapping content is **not
        exercisable in this gallery slice**, so it is **not an
        owner-visible observation point here**. The slice's only
        overflow (the footer) leaves the Grid *downward* and is removed
        by the outer-bounds clip, so no two Cells' content overlap (the
        nine grid cells are fully covered by the header span / three
        middle Cells / footer span — no empty target, and per-cell clip
        is absent per DD-M3-P5-005). The layout-side overlap-geometry ↔
        document-order correspondence is covered by T2
        (`grid_arrange_overflowing_cells_overlap_in_document_order`);
        the **visible pixel paint-precedence stays unobserved**,
        accepted on the layout-side substrate + Visual-tree
        insertion-order assumption. The divergence from the ADR/plan
        "visible proof" framing is dispositioned in T7 evidence
        aggregation (Codex review 2026-05-29). The T5 retro's
        "中段/footer overlap" prediction was geometrically incorrect;
        corrected in `t6.md`. Adding a dedicated overlap fixture (not
        polluting the gallery slice) is the preferred path if a visible
        demo is later wanted.
- [x] Owner explicitly accepts the smoke result, or records a fail
      observation note. **If smoke fails:** the implementation fix
      lands additively on the T6 branch (new commits); the smoke
      checklist above is re-run to green before T6 closes. Fix scope
      stays inside the Phase 5 ADR (`process/milestone-3/phase-5/decisions/`)
      / `docs/dsl_spec.md` §4.12 / `docs/architecture.md` §6.8.7;
      any fix requiring a normative spec change escalates to T7
      Moment 2 (or, if unsuitable for Moment 2, a mid-ADR addendum).
      Fix iterations stay inside T6 until the smoke checklist is
      green.
- [x] T6 step-end retrospective recorded at
      `process/milestone-3/phase-5/retrospectives/t6.md`
      (retrospectives.md checklist items 1-11).

### T7 — Phase-end gates and Moment 2 re-sync

Discharges the m3-plan phase-end criteria for Phase 5 and the
Moment 2 doc-set listed in the
[ADR preamble Moment 2 section](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2).
Per
[../requirements/constraints.md §5](../requirements/constraints.md#5-phase-最終-step-の-retrospective--progress-checklist-は-step-end-と-phase-end-を分割する)
(FD-G), the step-end retrospective is **owned by T7**; the phase-end
retrospective is **NOT owned by T7** and stays unchecked at T7 close
— see the two retrospective bullets at the end of this list.

- [ ] `cargo fmt --all -- --check` green locally and on CI.
- [ ] `cargo build --release --workspace` green locally and on CI.
- [ ] `cargo test --workspace` green locally and on CI.
- [ ] Windows-only integration evidence green on CI (skip-guard
      verified per T4).
- [ ] `docs/dsl_spec.md` §4.12 Phase status marker flips to
      `M3-Phase 5 closed; implementation-synced`; document-level
      Status header updated to reflect Phase 5 closed
      (implementation-synced); revision history v1.4 entry recording
      the Moment 2 close, any impl/spec divergence corrections, and
      any earlier-phase retroactive spec-gap fold surfaced during
      re-sync (owner-confirmation required per the retroactive
      spec-gap minimum-fold pattern).
- [ ] `docs/dsl_spec.md` §8 textual IR grammar / AST shape folded
      to match the Phase 5 implementation shape of Grid carrier c1
      (`KindPayload::Grid { columns, rows }`) as landed by T1
      (wasamoc emit) and T3 (runtime loader parse). The §8 fold was
      deferred from Moment 1 because the carrier's textual shape
      pins at implementation time; per the retroactive spec-gap
      minimum-fold pattern this lands in the same revision-history
      v1.4 entry as the §4.12 status flip.
- [ ] `docs/architecture.md` top Status flips to include
      `M3-Phase 5 complete`; any implementation-divergent paragraphs
      in §6.8.7 re-synced to actual landed shape.
- [ ] `process/milestone-3/plan.md` Phase 5 row Status flips to
      `complete`.
- [ ] `docs/abi_spec.md` re-confirmed untouched (Grid added no
      host-facing ABI surface; `LayoutError::GridUnboundedStarAxis`
      remained host-internal). Touch only if a Moment 2 surprise
      forced an ABI surface change, in which case escalate to ADR
      revision per the ADR preamble's three retrospectives.md
      §phase-sync ADR-touch cases.
- [ ] `process/milestone-3/phase-5/decisions/preamble.md` /
      `dd-m3-p5-*.md` touched **only** if one of the three
      retrospectives.md §phase-sync ADR-touch cases applies
      (AC discharged-vs-impl divergence; out-of-phase residual
      cross-reference; thesis-level finding). Otherwise the ADR
      set stays at its Moment 1 Accepted state.
- [ ] Step retro `phase-sync` items from T1-T7 close into
      `doc-folded` / `carry-forward` / `local-only` —
      **no open `phase-sync` items survive past phase close**.
- [ ] [log.md](./log.md) records the phase-close evidence pointer,
      CI run id, implementation summary distilled from T1-T6, and
      any final post-merge distillation.
- [ ] Carry-forward inputs to the next phase's pre-doc recorded
      under [handoff.md](./handoff.md) (covering at minimum: any
      residual surfaced during T2-T6; any out-of-phase R found
      during gallery smoke; Phase 6 inputs including the R1 Window-
      title wiring assignment already cross-referenced in
      `m3-plan.md` Phase 6 row Notes).
- [ ] Front-matter `status` flips from `active` to `closing` on this
      file in the same commit that flips T7's "all gates green"
      checkbox above. No further task checkboxes are added after
      this point.
- [ ] **T7 step-end retrospective recorded** at
      `process/milestone-3/phase-5/retrospectives/t7.md`
      (retrospectives.md checklist items 1-11; step → phase merge
      gate; **owned by T7**, this is a T7 deliverable).
- [ ] **Phase-end retrospective recorded** at
      `process/milestone-3/phase-5/retrospectives/phase-end.md`
      (retrospectives.md checklist items 12-18; phase → main merge
      gate; **NOT owned by T7**, performed on the phase branch
      after T7 merges in by a separate retro commit per
      [../requirements/constraints.md §5](../requirements/constraints.md#5-phase-最終-step-の-retrospective--progress-checklist-は-step-end-と-phase-end-を分割する)).
      **This bullet stays `[ ]` at T7 close**; the phase-end retro
      commit on the phase branch flips it. Phase 5 phase-end retro
      additionally decides whether to promote constraints §5 into
      `process/procedures/retrospectives.md` as a project-wide rule
      based on the lived experience of running this split from the
      start.
