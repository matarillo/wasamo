// Pure layout engine — no Win32/WinRT dependencies; all logic here is unit-testable.

use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetKind {
    Rectangle,
    VStack,
    HStack,
    // M3-Phase 2 DD-M3-P2-001 per-kind tag for the Box layout primitive.
    // The aspect-driven inscribed-fit measure-arrange (DD-M3-P2-005) and the
    // DD-M3-P2-001 child measure / centred alignment / clip overflow are
    // wired below — see `measure_box` and `arrange_box`.
    Box,
    // M3-Phase 3 DD-M3-P3-001 per-kind tag for the WrapPanel layout primitive.
    // T5 wires the catalog half (variant + constructor + placeholder dispatch);
    // the DD-M3-P3-005 line-breaker measure-arrange lands in T7 — see
    // `measure_wrap_panel` / `arrange_wrap_panel` below.
    WrapPanel,
    // M3-Phase 4 DD-M3-P4-001 per-kind tag for the ScrollView layout
    // primitive. DD-M3-P4-005 measure-arrange (asymmetric content measure
    // + offset clamp) lives in `measure_scroll_view` / `arrange_scroll_view`
    // below; the IR-loader / widget-catalog half lands in T3.
    ScrollView,
    // M3-Phase 5 DD-M3-P5-001 per-kind tag for the Grid layout primitive.
    // DD-M3-P5-004 track resolution + DD-M3-P5-005 arrange / alignment live
    // in `resolve_axis_tracks` / `measure_grid` / `arrange_grid` below; the
    // IR-loader / widget-catalog half (`WidgetData::Grid` →
    // `LayoutNode::grid`) lands in T3. The track lists and per-Cell
    // placements ride the flat-struct fields `grid_columns` / `grid_rows` /
    // `cell_placements` (R-D mitigation, log.md T2 entry); `Cell` is IR-only
    // and never materialises as its own `LayoutNode`.
    Grid,
}

/// M3-Phase 5 DD-M3-P5-002 Grid track sizing form, layout-engine-local
/// mirror of `wasamo_ir::TrackSize` (the `Ratio` mirror precedent — the
/// pure layout engine imports no IR types; conversion happens at the
/// `WidgetData::Grid` → `LayoutNode::grid` build boundary in T3). `Fixed`
/// is integer DSL pixels promoted to `f32` only inside
/// `resolve_axis_tracks` per the DD-M3-P5-004 `f32` rounding contract;
/// `Star` carries a positive integer weight (validated to `[1, 1024]` at
/// `wasamoc check` / runtime `validate()`, not at this type). Unit star
/// `*` lowers to `Star(1)`.
//
// `#[allow(dead_code)]`: the variants are constructed at the T3
// `WidgetData::Grid` → `LayoutNode::grid` build boundary and by the
// pure-logic T2 tests; production has no constructor until T3 (Phase 4
// `scroll_view` forward-pointer precedent).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackSize {
    Fixed(i32),
    Star(u32),
}

/// M3-Phase 5 DD-M3-P5-003 / DD-M3-P5-005 per-`Cell` placement, parallel
/// to `LayoutNode.children` (`cell_placements[i]` places content child
/// `children[i]`). Zero-based `row` / `column`; `row_span` / `column_span`
/// default to `1`; `h_align` / `v_align` default to `Stretch`. Defaults
/// are applied at the T3 build boundary, not at this type. The existing
/// `Alignment` enum is reused per-axis (`Leading` = `start`, `Center`,
/// `Trailing` = `end`, `Stretch`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellPlacement {
    pub row: u32,
    pub column: u32,
    pub row_span: u32,
    pub column_span: u32,
    pub h_align: Alignment,
    pub v_align: Alignment,
}

/// M3-Phase 5 DD-M3-P5-004 per-axis bound input to `resolve_axis_tracks`.
/// `arrange_grid` derives it from the arrange-time cell extent
/// (`Bounded(w)` when `w.is_finite()`, else `Unbounded`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AxisBound {
    Bounded(f32),
    Unbounded,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SizeConstraint {
    Fixed(f32),
    Fill,
    Shrink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Leading,
    Center,
    Trailing,
    Stretch,
}

// M3-Phase 2 DD-M3-P2-002 / DD-M3-P2-005: aspect-ratio carrier on
// `LayoutNode`. The `wasamo-runtime` Box-internal `box_values::Ratio` is
// translated into this layout-engine-local type at `build_layout_tree`
// time; the layout engine itself stays Win32/WinRT-free.
//
// `num` / `den` are guaranteed positive — `wasamoc check` (T3) and the IR
// loader's `validate()` pass (T7) reject zero / negative ratio sides before
// any LayoutNode is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ratio {
    pub num: i32,
    pub den: i32,
}

// M3-Phase 2 DD-M3-P2-005: layout-time runtime errors. Surfaced from
// `measure` / `arrange` / `run_layout` as a Result so the GUI loop can
// observe them; the C ABI surface for this error class is deferred (no
// `wasamo_run_layout` call exists today — the layout pass runs implicitly
// at `WM_SIZE`).
//
// - `BoxAspectUnboundedBoth`: a Box carries `aspect:` and was given parent
//   bounds that are infinite on both axes (e.g. transitively inside a
//   doubly-Shrink ancestor with no Fixed seed).
// - `BoxNoExtent`: a Box with **no** `aspect:` and no children was given
//   parent bounds that are infinite on both axes — there is nothing to
//   derive its size from, and silent 0×0 is rejected per DD-M3-P2-005's
//   "Box has no extent to resolve" error class.
// - `ScrollViewUnboundedAxis`: a ScrollView was given parent bounds whose
//   scroll axis (vertical per DD-M3-P4-001) is infinite. ScrollView has no
//   viewport boundary to scroll within in that state, so the layout pass
//   fails per DD-M3-P4-002. The variant is **internal only** in Phase 4 —
//   no `WASAMO_LAYOUT_ERROR_*` ABI tag is added (no host can meaningfully
//   observe it; the C ABI for `wasamo_run_layout` does not yet exist).
//
// - `GridUnboundedStarAxis`: a Grid with at least one weighted-star track
//   on an axis was given an unbounded parent bound on that axis
//   (DD-M3-P5-004). Star tracks divide finite remaining space; an
//   unbounded axis has no finite space to divide, so the layout pass fails
//   (Flutter-style; consistent with the Phase 4 ScrollView unbounded-axis
//   precedent). The variant is **internal only** in Phase 5 — no
//   `WASAMO_LAYOUT_ERROR_*` ABI tag is added (Grid adds no host-facing ABI
//   surface; no host observes layout-error variants meaningfully).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    BoxAspectUnboundedBoth,
    BoxNoExtent,
    ScrollViewUnboundedAxis,
    GridUnboundedStarAxis,
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub kind: WidgetKind,
    pub width: SizeConstraint,
    pub height: SizeConstraint,
    pub spacing: f32,
    pub padding: f32,
    pub alignment: Alignment,
    /// DD-M3-P2-005: present on Box nodes when the DSL surface set
    /// `aspect: <num>:<den>`; `None` on every other kind and on Box
    /// without an `aspect` attribute.
    pub aspect: Option<Ratio>,
    /// DD-M3-P3-004: present on WrapPanel when the DSL surface set
    /// `item-cross-size: <i32>`; uniform per-line cross-axis size for
    /// every item. `None` on every other kind and on WrapPanel without
    /// the attribute (parent-cross passthrough per Option (a)).
    pub item_cross_size: Option<f32>,
    /// DD-M3-P3-003: WrapPanel-only gap (main-axis) between adjacent
    /// items on the same line. `0.0` on every other kind and on
    /// WrapPanel without `item-spacing` set (touching items).
    pub item_spacing: f32,
    /// DD-M3-P3-003: WrapPanel-only gap (cross-axis) between adjacent
    /// lines. `0.0` on every other kind and on WrapPanel without
    /// `line-spacing` set (touching lines).
    pub line_spacing: f32,
    /// DD-M3-P4-003: ScrollView's bound or literal `offset-y` value in
    /// `i32` pixels (the DSL surface type). `0` on every other kind and
    /// on a ScrollView whose `.ui` omits `offset-y`. The layout-time
    /// clamp (DD-M3-P4-005) consumes this field; the clamped applied
    /// offset is recorded back into `applied_offset_y` for the Visual
    /// layer to read at sync time.
    pub offset_y: i32,
    pub children: Vec<LayoutNode>,
    // Written by arrange():
    pub offset: (f32, f32),
    pub size: (f32, f32),
    /// DD-M3-P4-005 measure→arrange clamped scroll offset cache. After
    /// `arrange_scroll_view` runs, this holds the post-clamp `applied_y`
    /// (`f32`, in layout-engine units) that `sync_visuals()` writes to
    /// the ScrollView-owned intermediate content Visual's
    /// `Visual.Offset = (0, -applied_y, 0)` per DD-M3-P4-004. `0.0` on
    /// every other kind and on a freshly-constructed ScrollView whose
    /// `arrange` has not yet run.
    pub(crate) applied_offset_y: Cell<f32>,
    /// DD-M3-P3-005 measure→arrange cross-bound cache used by
    /// `measure_wrap_panel` / `arrange_wrap_panel`. When
    /// `item_cross_size` is unset the spec passes the parent of
    /// WrapPanel's cross-axis constraint through to each child
    /// (DD-M3-P3-004 Option (a)); that constraint is the `avail_h`
    /// measure received from its parent. Under the default
    /// `height: Shrink`, the WrapPanel's own arrange-time `h`
    /// equals its measure-time `desired_h` (the sum of line cross
    /// extents) — which is **not** the same as `avail_h`. Without
    /// a cache, `arrange_wrap_panel` would re-measure children
    /// against a different cross bound, producing a different line
    /// break from measure (a regression caught in review of the
    /// T7 initial implementation). `measure_wrap_panel` records
    /// the cross input it used here; `arrange_wrap_panel` reads it
    /// back so the line breaker re-runs against the same per-child
    /// constraint. Sentinel `f32::NAN` (initial value) means
    /// "no prior measure"; `arrange_wrap_panel` falls back to `h`
    /// in that case so a stand-alone `arrange` call (no prior
    /// `measure`) still produces a self-consistent layout.
    /// `item_cross_size` (when `Some`) always overrides the cache,
    /// so the happy path of the gallery sub-screen is unaffected.
    pub(crate) wrap_measured_cross_bound: Cell<f32>,
    /// DD-M3-P5-002 Grid column track list. Non-empty only on
    /// `WidgetKind::Grid`; `Vec::new()` on every other kind (mirrors how
    /// `aspect` / `offset_y` sit dormant off-kind — R-D flat-struct
    /// extension, log.md T2 entry). Consumed by `resolve_axis_tracks`.
    pub grid_columns: Vec<TrackSize>,
    /// DD-M3-P5-002 Grid row track list. See `grid_columns`.
    pub grid_rows: Vec<TrackSize>,
    /// DD-M3-P5-003 / DD-M3-P5-005 per-Cell placements, parallel to
    /// `children` (`cell_placements[i]` places `children[i]`). Empty on
    /// every non-Grid kind. Document order = children order = paint /
    /// z-order (DD-M3-P5-005 Option A).
    pub cell_placements: Vec<CellPlacement>,
}

impl LayoutNode {
    pub fn rectangle(width: SizeConstraint, height: SizeConstraint) -> Self {
        Self {
            kind: WidgetKind::Rectangle,
            width,
            height,
            spacing: 0.0,
            padding: 0.0,
            alignment: Alignment::Stretch,
            aspect: None,
            item_cross_size: None,
            item_spacing: 0.0,
            line_spacing: 0.0,
            offset_y: 0,
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
            applied_offset_y: Cell::new(0.0),
            wrap_measured_cross_bound: Cell::new(f32::NAN),
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            cell_placements: Vec::new(),
        }
    }

    pub fn vstack(spacing: f32, padding: f32, alignment: Alignment) -> Self {
        Self {
            kind: WidgetKind::VStack,
            width: SizeConstraint::Fill,
            height: SizeConstraint::Shrink,
            spacing,
            padding,
            alignment,
            aspect: None,
            item_cross_size: None,
            item_spacing: 0.0,
            line_spacing: 0.0,
            offset_y: 0,
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
            applied_offset_y: Cell::new(0.0),
            wrap_measured_cross_bound: Cell::new(f32::NAN),
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            cell_placements: Vec::new(),
        }
    }

    pub fn hstack(spacing: f32, padding: f32, alignment: Alignment) -> Self {
        Self {
            kind: WidgetKind::HStack,
            width: SizeConstraint::Shrink,
            height: SizeConstraint::Fill,
            spacing,
            padding,
            alignment,
            aspect: None,
            item_cross_size: None,
            item_spacing: 0.0,
            line_spacing: 0.0,
            offset_y: 0,
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
            applied_offset_y: Cell::new(0.0),
            wrap_measured_cross_bound: Cell::new(f32::NAN),
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            cell_placements: Vec::new(),
        }
    }

    // M3-Phase 2 DD-M3-P2-001 / DD-M3-P2-005 Box layout entry. Default
    // size constraints are `Shrink/Shrink` so parent containers (VStack /
    // HStack / window root) honour the size computed by `measure_box`
    // (inscribed-fit when `aspect` is set; shrink-to-fit child or parent
    // bounds when it is not).
    pub fn box_(aspect: Option<Ratio>) -> Self {
        Self {
            kind: WidgetKind::Box,
            width: SizeConstraint::Shrink,
            height: SizeConstraint::Shrink,
            spacing: 0.0,
            padding: 0.0,
            alignment: Alignment::Center,
            aspect,
            item_cross_size: None,
            item_spacing: 0.0,
            line_spacing: 0.0,
            offset_y: 0,
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
            applied_offset_y: Cell::new(0.0),
            wrap_measured_cross_bound: Cell::new(f32::NAN),
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            cell_placements: Vec::new(),
        }
    }

    // M3-Phase 3 DD-M3-P3-001 / DD-M3-P3-005 WrapPanel layout entry.
    // Main axis (horizontal per DD-M3-P3-002 Option A) defaults to `Fill`
    // so the WrapPanel outer main-axis matches the parent's main bound
    // unconditionally (DD-M3-P3-005 visible-overflow rule); cross axis
    // defaults to `Shrink` so the height collapses to the sum of line
    // extents derived in T7. `item_cross_size` / `item_spacing` /
    // `line_spacing` are wired through here so T7's measure-arrange can
    // read them off `LayoutNode` without a second hop through `WidgetData`.
    pub fn wrap_panel(item_cross_size: Option<f32>, item_spacing: f32, line_spacing: f32) -> Self {
        Self {
            kind: WidgetKind::WrapPanel,
            width: SizeConstraint::Fill,
            height: SizeConstraint::Shrink,
            spacing: 0.0,
            padding: 0.0,
            alignment: Alignment::Stretch,
            aspect: None,
            item_cross_size,
            item_spacing,
            line_spacing,
            offset_y: 0,
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
            applied_offset_y: Cell::new(0.0),
            wrap_measured_cross_bound: Cell::new(f32::NAN),
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            cell_placements: Vec::new(),
        }
    }

    // M3-Phase 4 DD-M3-P4-001 / DD-M3-P4-002 / DD-M3-P4-003 ScrollView
    // layout entry. Both axes default to `Fill` so the viewport tracks
    // the parent-allocated slot per DD-M3-P4-002 Option A (parent
    // constraint passthrough on both axes); `offset_y` carries the
    // bound or literal `offset-y` value (DD-M3-P4-003 `i32` pixels)
    // which `arrange_scroll_view` clamps per DD-M3-P4-005 and records
    // the applied offset in `applied_offset_y` for the Visual layer
    // (T4) to read.
    //
    // T3 wires the IR-loader / build_layout_tree path that calls this
    // constructor (`widget::WidgetData::ScrollView` →
    // `LayoutNode::scroll_view`), so the T2-era `#[allow(dead_code)]`
    // forward-pointer is no longer needed.
    pub fn scroll_view(offset_y: i32) -> Self {
        Self {
            kind: WidgetKind::ScrollView,
            width: SizeConstraint::Fill,
            height: SizeConstraint::Fill,
            spacing: 0.0,
            padding: 0.0,
            alignment: Alignment::Stretch,
            aspect: None,
            item_cross_size: None,
            item_spacing: 0.0,
            line_spacing: 0.0,
            offset_y,
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
            applied_offset_y: Cell::new(0.0),
            wrap_measured_cross_bound: Cell::new(f32::NAN),
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            cell_placements: Vec::new(),
        }
    }

    // M3-Phase 5 DD-M3-P5-001 / DD-M3-P5-004 / DD-M3-P5-005 Grid layout
    // entry. Both axes default to `Fill` so Grid's outer rect tracks the
    // parent allocation on a bounded axis (DD-M3-P5-004 "Grid outer rect");
    // on an unbounded axis with fixed-only tracks the outer rect collapses
    // to the resolved `fixed_sum` (handled in `measure_grid` /
    // `arrange_grid`). `columns` / `rows` are the per-axis track lists and
    // `cell_placements` is parallel to `children` (set by the caller after
    // construction). T3 wires the IR-loader / `build_layout_tree` path
    // (`WidgetData::Grid` → this constructor); until then production has no
    // caller, so the T2-era `#[allow(dead_code)]` forward-pointer mirrors
    // the Phase 4 `scroll_view` constructor. The pure-logic T2 tests
    // exercise it directly.
    #[allow(dead_code)]
    pub fn grid(
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
        cell_placements: Vec<CellPlacement>,
    ) -> Self {
        Self {
            kind: WidgetKind::Grid,
            width: SizeConstraint::Fill,
            height: SizeConstraint::Fill,
            spacing: 0.0,
            padding: 0.0,
            alignment: Alignment::Stretch,
            aspect: None,
            item_cross_size: None,
            item_spacing: 0.0,
            line_spacing: 0.0,
            offset_y: 0,
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
            applied_offset_y: Cell::new(0.0),
            wrap_measured_cross_bound: Cell::new(f32::NAN),
            grid_columns: columns,
            grid_rows: rows,
            cell_placements,
        }
    }
}

/// Returns the desired (width, height) of a node given available space.
/// Pass `f32::INFINITY` for unconstrained axes.
/// Fill children return 0.0 — they take whatever the parent allocates in arrange().
pub fn measure(node: &LayoutNode, avail_w: f32, avail_h: f32) -> Result<(f32, f32), LayoutError> {
    match node.kind {
        WidgetKind::Rectangle => Ok(measure_leaf(node)),
        WidgetKind::VStack => measure_vstack(node, avail_w),
        WidgetKind::HStack => measure_hstack(node, avail_h),
        WidgetKind::Box => measure_box(node, avail_w, avail_h),
        WidgetKind::WrapPanel => measure_wrap_panel(node, avail_w, avail_h),
        WidgetKind::ScrollView => measure_scroll_view(node, avail_w, avail_h),
        WidgetKind::Grid => measure_grid(node, avail_w, avail_h),
    }
}

// M3-Phase 3 T7 DD-M3-P3-005 measure: greedy line-breaker against an
// unbounded main-axis child constraint (DD-M3-P3-001) and a
// DD-M3-P3-004-defined per-child cross-axis input. Returns the
// WrapPanel's outer (main, cross) extent:
//
// - **Outer cross** = sum of per-line cross extents + `line_spacing ×
//   (line_count − 1)` (DD-M3-P3-005 step 3; no trailing margin after
//   the last line, mirroring HStack/VStack `spacing` semantics).
// - **Outer main, bounded parent** = `avail_w` unconditionally
//   (DD-M3-P3-005 step 4 — WrapPanel does not grow to accommodate
//   oversized first-children; the spec'd visible-overflow rule paints
//   past this rectangle). The `width` constraint resolves at
//   `resolve_axis` time: `Fill` returns `0.0` here (HStack/VStack
//   convention — "take what the parent allocates"); `Fixed(v)` returns
//   `v`; `Shrink` returns the max per-line main extent (oversized
//   first-child extents *do* dominate the Shrink return so a
//   `Shrink`-width WrapPanel reports the actual content extent up to
//   its parent rather than masking the oversized child).
// - **Outer main, unbounded parent** = max per-line main extent
//   (DD-M3-P3-005 unbounded-main Option A: one-line flow — the line
//   breaker degenerates to all-children-on-one-line; the cumulative
//   intrinsic surfaces as the WrapPanel's main extent).
//
// `LayoutError` propagates from per-child measure — DD-M3-P3-005
// unbounded-cross Option A lets Phase 2's
// `LayoutError::BoxAspectUnboundedBoth` fire with the Box's IR
// location when an aspect-only child is measured against unbounded
// cross (no `item-cross-size` set, parent cross also unbounded).
fn measure_wrap_panel(
    node: &LayoutNode,
    avail_w: f32,
    avail_h: f32,
) -> Result<(f32, f32), LayoutError> {
    // DD-M3-P3-004 Option (a): when `item-cross-size` is unset, the
    // child cross input is the parent's cross-axis constraint
    // (= `avail_h`). Cache it on the node so `arrange_wrap_panel`
    // can re-run the line breaker with the same per-child constraint
    // even when its own arrange-time `h` differs from `avail_h`
    // (typical under the default `height: Shrink`, where `h` ends
    // up being `desired_h` from this measure pass — the sum of line
    // cross extents — not the parent's cross-axis constraint).
    let child_cross_input = node.item_cross_size.unwrap_or(avail_h);
    node.wrap_measured_cross_bound.set(child_cross_input);

    let lines = compute_wrap_lines(node, avail_w, child_cross_input)?;
    let line_count = lines.len();
    let cross_sum: f32 = lines.iter().map(|l| l.cross_extent).sum();
    let line_spacing_total = if line_count > 1 {
        node.line_spacing * (line_count as f32 - 1.0)
    } else {
        0.0
    };
    let outer_cross = cross_sum + line_spacing_total;

    let max_line_main = lines.iter().map(|l| l.main_extent).fold(0.0_f32, f32::max);

    let outer_main = match &node.width {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => {
            if avail_w.is_finite() {
                0.0
            } else {
                // Unbounded-main parent + Fill WrapPanel: report the
                // one-line cumulative so a parent that resolves Fill
                // against an unbounded available has a finite anchor
                // (HStack/VStack pass `INFINITY` to children when
                // they themselves sit on a Shrink axis).
                max_line_main
            }
        }
        SizeConstraint::Shrink => max_line_main,
    };

    Ok((outer_main, outer_cross))
}

/// Per-child slot inside a single WrapPanel line, with the (main, cross)
/// extents the child measured to. `index` indexes back into
/// `LayoutNode.children` so `arrange_wrap_panel` can mutate the child
/// after the pure compute step.
struct WrapChild {
    index: usize,
    main_size: f32,
    cross_size: f32,
}

/// A single WrapPanel line: the children placed on it, the recorded
/// main-axis extent (sum of child main sizes + `item_spacing` between
/// adjacent siblings, with the first-child unconditional placement
/// from DD-M3-P3-005 admitted as the only way a line's `main_extent`
/// can exceed `parent_main_bound`), and the cross-axis extent (uniform
/// `item_cross_size` when set per DD-M3-P3-004; max of children's
/// reported cross sizes otherwise).
struct WrapLine {
    children: Vec<WrapChild>,
    main_extent: f32,
    cross_extent: f32,
}

/// Pure, mock-free line breaker shared by `measure_wrap_panel` and
/// `arrange_wrap_panel` (free-function extraction per
/// [CLAUDE.md §Testing rules]). Measures every child once against the
/// DD-M3-P3-001 unbounded-main + DD-M3-P3-004 cross constraint
/// (`child_cross_input` — resolved by the caller against either
/// `item_cross_size`, the measure-time cross cache, or the arrange
/// fallback so the two passes stay in step), then applies the
/// DD-M3-P3-005 greedy line breaker:
///
/// - **First child of any line** is placed *unconditionally* (the
///   `line_empty == true` carve-out): even when its intrinsic main
///   extent exceeds `parent_main_bound`, it occupies the line on its
///   own and the line's recorded `main_extent` may exceed the bound.
/// - **Subsequent children** are placed iff
///   `current_line_main + item_spacing + next_child_main <=
///    parent_main_bound` (the spacing-aware inequality of DD-M3-P3-001
///   /  DD-M3-P3-005 step 1); failure starts a new line where the
///   unconditional rule applies again to the same candidate.
/// - **Unbounded main-axis** parents skip the inequality entirely so
///   the line breaker degenerates to one-line flow
///   (DD-M3-P3-005 unbounded-main Option A).
fn compute_wrap_lines(
    node: &LayoutNode,
    main_bound: f32,
    child_cross_input: f32,
) -> Result<Vec<WrapLine>, LayoutError> {
    let measured: Vec<(f32, f32)> = node
        .children
        .iter()
        .map(|c| measure(c, f32::INFINITY, child_cross_input))
        .collect::<Result<Vec<_>, _>>()?;

    let main_bounded = main_bound.is_finite();
    let mut lines: Vec<WrapLine> = Vec::new();
    let mut current = WrapLine {
        children: Vec::new(),
        main_extent: 0.0,
        cross_extent: 0.0,
    };

    for (idx, (cm, cc)) in measured.into_iter().enumerate() {
        let line_empty = current.children.is_empty();
        let fits = if line_empty || !main_bounded {
            true
        } else {
            current.main_extent + node.item_spacing + cm <= main_bound
        };

        if !fits {
            lines.push(std::mem::replace(
                &mut current,
                WrapLine {
                    children: Vec::new(),
                    main_extent: 0.0,
                    cross_extent: 0.0,
                },
            ));
        }

        let new_main = if current.children.is_empty() {
            cm
        } else {
            current.main_extent + node.item_spacing + cm
        };
        current.children.push(WrapChild {
            index: idx,
            main_size: cm,
            cross_size: cc,
        });
        current.main_extent = new_main;
    }

    if !current.children.is_empty() {
        lines.push(current);
    }

    for line in lines.iter_mut() {
        line.cross_extent = if let Some(ics) = node.item_cross_size {
            ics
        } else {
            line.children
                .iter()
                .map(|c| c.cross_size)
                .fold(0.0_f32, f32::max)
        };
    }

    Ok(lines)
}

fn measure_leaf(node: &LayoutNode) -> (f32, f32) {
    let w = if let SizeConstraint::Fixed(v) = node.width {
        v
    } else {
        0.0
    };
    let h = if let SizeConstraint::Fixed(v) = node.height {
        v
    } else {
        0.0
    };
    (w, h)
}

fn measure_vstack(node: &LayoutNode, avail_w: f32) -> Result<(f32, f32), LayoutError> {
    let inner_w = (avail_w - 2.0 * node.padding).max(0.0);
    let child_desired: Vec<(f32, f32)> = node
        .children
        .iter()
        .map(|c| measure(c, inner_w, f32::INFINITY))
        .collect::<Result<Vec<_>, _>>()?;

    let n = node.children.len();
    let spacing_total = if n > 0 {
        node.spacing * (n as f32 - 1.0)
    } else {
        0.0
    };

    let desired_w = match &node.width {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => {
            let max_cw = child_desired
                .iter()
                .map(|&(w, _)| w)
                .fold(0.0_f32, f32::max);
            max_cw + 2.0 * node.padding
        }
    };

    let non_fill_h: f32 = node
        .children
        .iter()
        .zip(child_desired.iter())
        .filter(|(c, _)| c.height != SizeConstraint::Fill)
        .map(|(_, &(_, h))| h)
        .sum();

    let desired_h = match &node.height {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => non_fill_h + spacing_total + 2.0 * node.padding,
    };

    Ok((desired_w, desired_h))
}

fn measure_hstack(node: &LayoutNode, avail_h: f32) -> Result<(f32, f32), LayoutError> {
    let inner_h = (avail_h - 2.0 * node.padding).max(0.0);
    let child_desired: Vec<(f32, f32)> = node
        .children
        .iter()
        .map(|c| measure(c, f32::INFINITY, inner_h))
        .collect::<Result<Vec<_>, _>>()?;

    let n = node.children.len();
    let spacing_total = if n > 0 {
        node.spacing * (n as f32 - 1.0)
    } else {
        0.0
    };

    let desired_h = match &node.height {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => {
            let max_ch = child_desired
                .iter()
                .map(|&(_, h)| h)
                .fold(0.0_f32, f32::max);
            max_ch + 2.0 * node.padding
        }
    };

    let non_fill_w: f32 = node
        .children
        .iter()
        .zip(child_desired.iter())
        .filter(|(c, _)| c.width != SizeConstraint::Fill)
        .map(|(_, &(w, _))| w)
        .sum();

    let desired_w = match &node.width {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => non_fill_w + spacing_total + 2.0 * node.padding,
    };

    Ok((desired_w, desired_h))
}

// DD-M3-P2-005 measure: with `aspect`, inscribed-fit (bounded both axes) /
// bounded-axis-wins (one axis unbounded) / `BoxAspectUnboundedBoth` (both
// unbounded). Without `aspect`, shrink-to-fit the (at-most-one) child or
// match parent bounds when empty (collapsing unbounded axes to zero, and
// returning `BoxNoExtent` for fully-unbounded empty Boxes).
fn measure_box(node: &LayoutNode, avail_w: f32, avail_h: f32) -> Result<(f32, f32), LayoutError> {
    let w_bounded = avail_w.is_finite();
    let h_bounded = avail_h.is_finite();

    if let Some(ratio) = node.aspect {
        return match (w_bounded, h_bounded) {
            (true, true) => Ok(inscribed_fit(avail_w, avail_h, ratio)),
            (true, false) => Ok((avail_w, derive_height(avail_w, ratio))),
            (false, true) => Ok((derive_width(avail_h, ratio), avail_h)),
            (false, false) => Err(LayoutError::BoxAspectUnboundedBoth),
        };
    }

    if node.children.is_empty() {
        // No aspect, no child: match parent bounds; collapse to zero on any
        // unbounded axis; surface `BoxNoExtent` when *both* are unbounded
        // (same structural error class as the aspect-set unbounded-both case).
        return match (w_bounded, h_bounded) {
            (true, true) => Ok((avail_w, avail_h)),
            (true, false) => Ok((avail_w, 0.0)),
            (false, true) => Ok((0.0, avail_h)),
            (false, false) => Err(LayoutError::BoxNoExtent),
        };
    }

    // No aspect, single child (single-child invariant enforced upstream at
    // `wasamoc check` T3 and `ir_loader::build_node` T7): shrink-to-fit the
    // child's intrinsic measure against Box bounds.
    let child = &node.children[0];
    measure(child, avail_w, avail_h)
}

// Branch selection uses i64 to keep the inscribed-fit choice independent
// of f32 round-off; the derived axis is then computed in f32 per the
// DD-M3-P2-005 numeric / rounding contract.
fn inscribed_fit(w: f32, h: f32, ratio: Ratio) -> (f32, f32) {
    // Inputs are finite here; the negative case is excluded by the
    // call sites (parent bounds are >= 0 in this layout engine).
    let w64 = w as f64;
    let h64 = h as f64;
    let num64 = ratio.num as f64;
    let den64 = ratio.den as f64;
    if w64 * den64 <= h64 * num64 {
        // Width-constrained: match parent width, derive height.
        (w, derive_height(w, ratio))
    } else {
        // Height-constrained: match parent height, derive width.
        (derive_width(h, ratio), h)
    }
}

fn derive_height(w: f32, ratio: Ratio) -> f32 {
    w * (ratio.den as f32) / (ratio.num as f32)
}

fn derive_width(h: f32, ratio: Ratio) -> f32 {
    h * (ratio.num as f32) / (ratio.den as f32)
}

/// Assigns final offset and size, recurses into children.
pub fn arrange(node: &mut LayoutNode, x: f32, y: f32, w: f32, h: f32) -> Result<(), LayoutError> {
    let kind = node.kind;
    let padding = node.padding;
    let spacing = node.spacing;
    let alignment = node.alignment;

    match kind {
        WidgetKind::Rectangle => {
            node.offset = (x, y);
            node.size = (w, h);
            Ok(())
        }
        WidgetKind::VStack => {
            node.offset = (x, y);
            node.size = (w, h);
            arrange_vstack(&mut node.children, x, y, w, h, padding, spacing, alignment)
        }
        WidgetKind::HStack => {
            node.offset = (x, y);
            node.size = (w, h);
            arrange_hstack(&mut node.children, x, y, w, h, padding, spacing, alignment)
        }
        WidgetKind::Box => arrange_box(node, x, y, w, h),
        WidgetKind::WrapPanel => arrange_wrap_panel(node, x, y, w, h),
        WidgetKind::ScrollView => arrange_scroll_view(node, x, y, w, h),
        WidgetKind::Grid => arrange_grid(node, x, y, w, h),
    }
}

// M3-Phase 3 T7 DD-M3-P3-005 arrange: re-run the line breaker against
// the parent-allocated cell, then place each child within its line.
//
// - WrapPanel's outer rectangle is recorded as the parent-allocated
//   `(w, h)` (DD-M3-P3-005 step 4: outer main equals parent main
//   bound unconditionally; outer cross equals what the parent
//   allocated, which during the typical measure→arrange flow equals
//   the per-line-extent sum computed at measure time).
// - Children flow left-to-right per line; the first child of any line
//   is placed at `cur_main = x` regardless of its intrinsic main
//   extent. Subsequent siblings advance by `item_spacing` then their
//   own main size, with no trailing margin after the last child.
// - **Visible overflow** of an oversized first child surfaces as
//   `child.offset.0 + child.size.0 > node.offset.0 + node.size.0`
//   (horizontal main axis) — the spec'd
//   "child paints past the WrapPanel rectangle" outcome, which
//   downstream parents (Phase 4 ScrollView) clip via their own clip
//   surface; WrapPanel itself installs no clip (visual-layer
//   responsibility, not layout-engine concern).
// - Each child's cross-axis position is centred within the line
//   (DD-M3-P3-001 cross-axis alignment Option A); smaller children
//   sit centred inside `line.cross_extent`, larger children overflow
//   above and below their line (visible per Phase 2 Box overflow
//   convention; no clip installed here).
fn arrange_wrap_panel(
    node: &mut LayoutNode,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Result<(), LayoutError> {
    node.offset = (x, y);
    node.size = (w, h);

    // Re-derive the per-child cross-axis input the way measure did,
    // not the way the arrange-time allocation `h` suggests. The two
    // diverge whenever the WrapPanel's `height: Shrink` default
    // collapses its outer cross to `desired_h` (the sum of line cross
    // extents) — taking `h` as the child cross bound would re-measure
    // children against a different constraint and produce a different
    // line break from measure. `item_cross_size` always wins; the
    // cache (set by `measure_wrap_panel`) covers the unset path; the
    // raw `h` fallback only fires if `arrange` is called without a
    // prior `measure` on the same node (e.g. a direct unit-test call),
    // in which case there is no measure result to be consistent with.
    let child_cross_input = node.item_cross_size.unwrap_or_else(|| {
        let cached = node.wrap_measured_cross_bound.get();
        if cached.is_nan() {
            h
        } else {
            cached
        }
    });

    let lines = compute_wrap_lines(node, w, child_cross_input)?;
    let item_spacing = node.item_spacing;
    let line_spacing = node.line_spacing;
    let line_count = lines.len();

    let mut cur_cross = y;
    for (li, line) in lines.iter().enumerate() {
        let mut cur_main = x;
        let n = line.children.len();
        for (i, wc) in line.children.iter().enumerate() {
            let child_cross_offset = cur_cross + (line.cross_extent - wc.cross_size) / 2.0;
            let child = &mut node.children[wc.index];
            arrange(
                child,
                cur_main,
                child_cross_offset,
                wc.main_size,
                wc.cross_size,
            )?;
            cur_main += wc.main_size;
            if i < n - 1 {
                cur_main += item_spacing;
            }
        }
        cur_cross += line.cross_extent;
        if li < line_count - 1 {
            cur_cross += line_spacing;
        }
    }

    Ok(())
}

// M3-Phase 4 DD-M3-P4-005 measure: ScrollView's outer size equals the
// parent-allocated viewport regardless of content size, so measure does
// not recurse into the single content child here. The asymmetric
// content measure (`(viewport_w, +∞)`) happens at arrange time when the
// concrete viewport `w` is known.
//
// The DD-M3-P4-002 unbounded-scroll-axis error is detected at arrange
// time (the viewport is decided there), not here — measure-time
// `avail_h = INFINITY` is the standard "tell me how big you want to
// be" idiom that parents like VStack pass to their children, and
// firing here would make every ScrollView placed inside a `Shrink`
// vertical parent fail even though the parent will allocate a finite
// cell at arrange. See `arrange_scroll_view` for the actual gate.
fn measure_scroll_view(
    node: &LayoutNode,
    avail_w: f32,
    avail_h: f32,
) -> Result<(f32, f32), LayoutError> {
    let desired_w = match &node.width {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => avail_w,
    };
    let desired_h = match &node.height {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => avail_h,
    };
    Ok((desired_w, desired_h))
}

// M3-Phase 4 DD-M3-P4-005 arrange: the viewport `(w, h)` is what the
// parent allocated; ScrollView's own offset/size record that
// allocation. The single content child is then measured with
// `(viewport_w, +∞)` (DD-M3-P4-005 "bounded cross + unbounded scroll
// axis" / inverse of WrapPanel's measure input), the resulting
// content height drives the offset clamp, and the content is
// arranged at the viewport's top-leading corner translated upward by
// the clamped offset.
//
// - **Unbounded scroll axis** (`h.is_finite() == false`) is the
//   structurally meaningless case from DD-M3-P4-002: there is no
//   viewport boundary to scroll within. Fires
//   `LayoutError::ScrollViewUnboundedAxis` before measuring content
//   so the error names the structural problem rather than surfacing
//   downstream child errors first.
// - **No content child** (0-child ScrollView) is rejected by
//   `wasamoc check` (T1) and the runtime IR loader's `validate()`
//   (T3); layout treats the case as a no-op for robustness. The
//   `applied_offset_y` cache clamps to 0.
// - **Offset clamp** uses `max(0, content_h - viewport_h)` as the
//   upper bound. Negative `offset-y`, in-range `offset-y`, `offset-y`
//   at max, and `offset-y` larger than max all map to a well-defined
//   applied offset in `[0, max_offset]` (DD-M3-P4-005). The clamped
//   `applied_offset_y` is recorded on the ScrollView LayoutNode for
//   the Visual layer (T4) to read at sync time so the
//   ScrollView-owned intermediate content Visual can place
//   `Visual.Offset = (0, -applied_y, 0)` (DD-M3-P4-004) without
//   re-running the clamp arithmetic.
// - **Rounding contract** (DD-M3-P4-005 sub-issue): `offset_y` is
//   the `i32` DSL surface value, promoted to `f32` here for clamp
//   arithmetic and Visual offset writes. No pixel-snapping is
//   applied; the rounding contract matches Phase 2 / Phase 3.
// - **Content cross-axis** mirrors the existing single-child layout
//   convention: a `Fill`-width child expands to the viewport width;
//   a `Shrink` / `Fixed` child sits at its measured cross extent. On
//   the scroll axis, the child arranges at its measured `ch_desired`
//   (Fill-height children measure to `0` and degenerate to a
//   zero-height arrangement — consistent with the existing
//   Fill-in-Shrink-parent convention).
fn arrange_scroll_view(
    node: &mut LayoutNode,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> Result<(), LayoutError> {
    if !h.is_finite() {
        return Err(LayoutError::ScrollViewUnboundedAxis);
    }

    node.offset = (x, y);
    node.size = (w, h);

    if node.children.is_empty() {
        node.applied_offset_y.set(0.0);
        return Ok(());
    }

    let child = &mut node.children[0];
    let (cw_desired, ch_desired) = measure(child, w, f32::INFINITY)?;

    let final_cw = if child.width == SizeConstraint::Fill {
        w
    } else {
        cw_desired
    };
    let final_ch = ch_desired;

    let offset_f = node.offset_y as f32;
    let max_offset = (final_ch - h).max(0.0);
    let applied = offset_f.clamp(0.0, max_offset);
    node.applied_offset_y.set(applied);

    arrange(child, x, y - applied, final_cw, final_ch)
}

// ── M3-Phase 5 Grid (DD-M3-P5-004 / DD-M3-P5-005) ──────────────────────────────

// DD-M3-P5-004 per-axis track resolution: fixed tracks consume definite
// space first; the remaining bounded space divides among star tracks by
// positive integer weight, over `f32` prefix boundaries (no integer pixel
// snap). Rows and columns invoke this independently.
//
// - **Unbounded-star branch**: any star track on an unbounded axis fires
//   `LayoutError::GridUnboundedStarAxis` (there is no finite space to
//   divide). Mirrors the Phase 4 ScrollView unbounded-axis gate.
// - **Negative remaining**: when `fixed_sum >= bound`,
//   `remaining_after_fixed` clamps to `0.0` and every star track resolves
//   to width `0`; fixed tracks retain their declared size (Cell rectangles
//   past `bound` overflow and are clipped at Grid's outer-bounds clip per
//   DD-M3-P5-005).
// - **`auto` slot**: reserved (no-op in Phase 5) between fixed-sum
//   computation and star distribution per the DD-M3-P5-002 deferral.
// - **Star weight sum** accumulates in `u64` (per-weight cap 1024 closes
//   overflow at the type level); the `u64 -> f32` cast is exact for any
//   realistic Grid (DD-M3-P5-004 precision note).
fn resolve_axis_tracks(
    tracks: &[TrackSize],
    axis_bound: AxisBound,
) -> Result<Vec<f32>, LayoutError> {
    let fixed_sum: f32 = tracks
        .iter()
        .filter_map(|t| match t {
            TrackSize::Fixed(px) => Some(*px as f32),
            TrackSize::Star(_) => None,
        })
        .sum();
    let star_weight_sum: u64 = tracks
        .iter()
        .map(|t| match t {
            TrackSize::Star(w) => *w as u64,
            TrackSize::Fixed(_) => 0,
        })
        .sum();
    let has_star = star_weight_sum > 0;

    if has_star && matches!(axis_bound, AxisBound::Unbounded) {
        return Err(LayoutError::GridUnboundedStarAxis);
    }

    // Phase 5 `auto` pass reservation: no-op. A future phase admits
    // `TrackSize::Auto` and inserts a demand pass here (before star
    // distribution) per DD-M3-P5-004.

    let bound = match axis_bound {
        AxisBound::Bounded(b) => b,
        // No star track exists on an unbounded axis (the branch above
        // errored otherwise); the axis resolves to the fixed sum.
        AxisBound::Unbounded => fixed_sum,
    };
    let remaining_after_fixed = (bound - fixed_sum).max(0.0);

    let resolved = tracks
        .iter()
        .map(|t| match t {
            TrackSize::Fixed(px) => *px as f32,
            TrackSize::Star(weight) => {
                // Defensive: a `Star` arm is only reachable when a star
                // track exists, which forces `star_weight_sum >= 1` for
                // validated IR (`Star` weight is in `[1, 1024]` per
                // DD-M3-P5-002 / DD-M3-P5-006). A zero sum here means a
                // `Star(0)` slipped past validate(); guard the divide.
                if star_weight_sum == 0 {
                    panic!(
                        "Grid star track reached layout with zero total \
                         star weight; DD-M3-P5-006 validate() must reject \
                         Star(0) before arrange"
                    );
                }
                remaining_after_fixed * (*weight as f32 / star_weight_sum as f32)
            }
        })
        .collect();

    Ok(resolved)
}

// DD-M3-P5-004 prefix boundaries: `boundary[0] = 0.0`, `boundary[n] =
// boundary[n-1] + resolved[n-1]`, with a trailing `boundary[len]` equal to
// the total resolved track extent. Consumed by cell-rectangle resolution
// and (on an unbounded axis) by Grid's outer extent.
fn prefix_boundaries(resolved: &[f32]) -> Vec<f32> {
    let mut boundaries = Vec::with_capacity(resolved.len() + 1);
    let mut acc = 0.0_f32;
    boundaries.push(acc);
    for &r in resolved {
        acc += r;
        boundaries.push(acc);
    }
    boundaries
}

// DD-M3-P5-004 measure: Grid's outer size follows its size constraints
// (Fill default → 0.0, taking the parent allocation at arrange; Fixed → the
// literal). Under `Shrink`, the axis resolves to the total track extent
// against the available bound (star tracks fill the bound; fixed-only
// collapses to `fixed_sum`). Track resolution proper — and the
// unbounded-star gate on a Fill / Fixed axis — happen at arrange when the
// concrete cell is known (mirrors ScrollView); only the `Shrink` desired
// extent must resolve here.
fn measure_grid(node: &LayoutNode, avail_w: f32, avail_h: f32) -> Result<(f32, f32), LayoutError> {
    let desired_w = match &node.width {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => grid_shrink_extent(&node.grid_columns, avail_w)?,
    };
    let desired_h = match &node.height {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => grid_shrink_extent(&node.grid_rows, avail_h)?,
    };
    Ok((desired_w, desired_h))
}

// Shrink-axis desired extent = total resolved track extent against the
// available bound. `avail` may be unbounded (a Shrink parent's "how big do
// you want to be" probe); with star tracks that surfaces
// `GridUnboundedStarAxis` (the intended outcome), with fixed-only it is the
// `fixed_sum`.
fn grid_shrink_extent(tracks: &[TrackSize], avail: f32) -> Result<f32, LayoutError> {
    let bound = if avail.is_finite() {
        AxisBound::Bounded(avail)
    } else {
        AxisBound::Unbounded
    };
    let resolved = resolve_axis_tracks(tracks, bound)?;
    Ok(resolved.iter().sum())
}

// DD-M3-P5-004 / DD-M3-P5-005 arrange: resolve both axes against the
// parent-allocated cell, compute prefix boundaries, then place each content
// child within its Cell's resolved rectangle with per-Cell alignment.
//
// - **Grid outer rect** on a bounded axis equals the parent allocation
//   (Grid does not grow to accommodate an oversized track-resolved extent —
//   Phase 3 / Phase 4 precedent); on an unbounded axis (only reachable with
//   no star tracks per the unbounded-star gate) it equals the resolved
//   `fixed_sum` (the trailing prefix boundary).
// - **Cell rectangle** spans `column_boundary[column ..= column+span]` ×
//   `row_boundary[row ..= row+span]`; spanning Cells are measured against
//   the combined resolved span (no demand back-propagation in Phase 5).
// - **Alignment**: stretch (default) fills the cell on that axis;
//   non-stretch anchors the content's natural measure at start / center /
//   end. Per-cell clipping is out of scope (content overflow paints past
//   the cell and is contained only by Grid's outer-bounds clip, installed
//   on Grid's own Visual at sync_visuals time per DD-M3-P5-005).
// - **Paint / z-order** is document order, preserved by iterating
//   `children` (and the parallel `cell_placements`) in order.
fn arrange_grid(node: &mut LayoutNode, x: f32, y: f32, w: f32, h: f32) -> Result<(), LayoutError> {
    let col_bound = if w.is_finite() {
        AxisBound::Bounded(w)
    } else {
        AxisBound::Unbounded
    };
    let row_bound = if h.is_finite() {
        AxisBound::Bounded(h)
    } else {
        AxisBound::Unbounded
    };

    let col_sizes = resolve_axis_tracks(&node.grid_columns, col_bound)?;
    let row_sizes = resolve_axis_tracks(&node.grid_rows, row_bound)?;
    let col_b = prefix_boundaries(&col_sizes);
    let row_b = prefix_boundaries(&row_sizes);

    // Grid outer extent: bounded axis = parent allocation; unbounded axis =
    // total resolved extent (the trailing prefix boundary).
    let outer_w = if w.is_finite() {
        w
    } else {
        *col_b.last().unwrap_or(&0.0)
    };
    let outer_h = if h.is_finite() {
        h
    } else {
        *row_b.last().unwrap_or(&0.0)
    };
    node.offset = (x, y);
    node.size = (outer_w, outer_h);

    // `cell_placements` is parallel to `children`; `zip` borrows the two
    // disjoint fields and stops at the shorter (validate() guarantees equal
    // length, so a stray unplaced child is simply not arranged rather than
    // indexing out of range).
    for (child, placement) in node.children.iter_mut().zip(node.cell_placements.iter()) {
        let col_start = placement.column as usize;
        let col_end = (placement.column + placement.column_span) as usize;
        let row_start = placement.row as usize;
        let row_end = (placement.row + placement.row_span) as usize;

        let cell_left = x + col_b[col_start];
        let cell_right = x + col_b[col_end];
        let cell_top = y + row_b[row_start];
        let cell_bottom = y + row_b[row_end];
        let cell_w = cell_right - cell_left;
        let cell_h = cell_bottom - cell_top;

        // DD-M3-P5-005 measure input is per-axis: a stretch axis (or a
        // `Fill` content constraint, which `align_in_cell` also expands to
        // the cell extent) measures the content against the cell extent on
        // that axis; a non-stretch axis (`start` / `center` / `end`)
        // measures the content at its **natural extent** (unbounded probe,
        // the HStack/VStack idiom) so the spec'd natural-size anchoring —
        // and the overflow it can produce — is honoured. Measuring a
        // non-stretch axis against the cell extent would silently shrink a
        // bound-dependent child (e.g. an aspect Box, or wrapping content) to
        // the cell, weakening center / end / overflow vs the spec.
        let measure_w = if cell_axis_is_stretchy(placement.h_align, &child.width) {
            cell_w
        } else {
            f32::INFINITY
        };
        let measure_h = if cell_axis_is_stretchy(placement.v_align, &child.height) {
            cell_h
        } else {
            f32::INFINITY
        };
        let (desired_w, desired_h) = measure(child, measure_w, measure_h)?;
        let (cx, cw) = align_in_cell(
            placement.h_align,
            &child.width,
            desired_w,
            cell_left,
            cell_w,
        );
        let (cy, ch) = align_in_cell(
            placement.v_align,
            &child.height,
            desired_h,
            cell_top,
            cell_h,
        );
        arrange(child, cx, cy, cw, ch)?;
    }

    Ok(())
}

// DD-M3-P5-005: an axis behaves as "stretchy" (content fills the cell
// extent on that axis) when its alignment is `Stretch` (the default) or the
// content carries a `Fill` constraint on that axis. The `Fill`-as-stretch
// rule mirrors the existing `cross_axis_position` convention (Fill and
// Stretch both expand to the full inner extent); a `Fill` child has no
// natural extent to anchor, so it fills the cell regardless of the
// non-stretch alignment value. Shared by the `arrange_grid` measure-bound
// selection and `align_in_cell` so the two stay in lockstep.
fn cell_axis_is_stretchy(align: Alignment, constraint: &SizeConstraint) -> bool {
    align == Alignment::Stretch || *constraint == SizeConstraint::Fill
}

// DD-M3-P5-005 per-axis alignment within a resolved cell rectangle.
// Stretch alignment (the default) — or a `Fill` content constraint —
// extends the content to the full cell extent. Non-stretch anchors the
// content's natural measured extent at start (`Leading`) / center / end
// (`Trailing`); the content is **not** clamped to the cell (per-cell
// clipping is out of scope — overflow paints past the cell and is contained
// only at Grid's outer-bounds clip).
fn align_in_cell(
    align: Alignment,
    constraint: &SizeConstraint,
    desired: f32,
    cell_start: f32,
    cell_extent: f32,
) -> (f32, f32) {
    if cell_axis_is_stretchy(align, constraint) {
        return (cell_start, cell_extent);
    }
    match align {
        Alignment::Leading => (cell_start, desired),
        Alignment::Center => (cell_start + (cell_extent - desired) / 2.0, desired),
        Alignment::Trailing => (cell_start + cell_extent - desired, desired),
        Alignment::Stretch => unreachable!("stretch handled above"),
    }
}

// DD-M3-P2-005 arrange: re-derive the Box's resolved rectangle from the
// parent-allocated cell (so the painted region honours `aspect` even when
// the parent decided to allocate a strictly larger cell, e.g. under a
// `Stretch`-aligned cross-axis); then measure the (optional) single child
// against the resolved Box bounds and centre + clip it per DD-M3-P2-001.
fn arrange_box(node: &mut LayoutNode, x: f32, y: f32, w: f32, h: f32) -> Result<(), LayoutError> {
    let w_bounded = w.is_finite();
    let h_bounded = h.is_finite();

    let (rw, rh) = if let Some(ratio) = node.aspect {
        match (w_bounded, h_bounded) {
            (true, true) => inscribed_fit(w, h, ratio),
            (true, false) => (w, derive_height(w, ratio)),
            (false, true) => (derive_width(h, ratio), h),
            (false, false) => return Err(LayoutError::BoxAspectUnboundedBoth),
        }
    } else if node.children.is_empty() {
        match (w_bounded, h_bounded) {
            (true, true) => (w, h),
            (true, false) => (w, 0.0),
            (false, true) => (0.0, h),
            (false, false) => return Err(LayoutError::BoxNoExtent),
        }
    } else {
        // No-aspect Box with a child: shrink-to-fit semantics already
        // resolved at measure-time. At arrange the parent passes the
        // shrink-to-fit cell; use it as-is.
        (w, h)
    };

    node.offset = (x, y);
    node.size = (rw, rh);

    if let Some(child) = node.children.first_mut() {
        let (cw, ch) = measure(child, rw, rh)?;
        // DD-M3-P2-001 child measure / centred alignment / clip overflow.
        // A Fill-width / Fill-height child treats Box bounds as its target
        // extent (clipping is a no-op there); a Shrink / Fixed child gets
        // its intrinsic size, clipped to Box bounds if it overflows.
        let final_cw = if child.width == SizeConstraint::Fill {
            rw
        } else {
            cw.min(rw)
        };
        let final_ch = if child.height == SizeConstraint::Fill {
            rh
        } else {
            ch.min(rh)
        };
        let cx = x + (rw - final_cw) / 2.0;
        let cy = y + (rh - final_ch) / 2.0;
        arrange(child, cx, cy, final_cw, final_ch)?;
    }

    Ok(())
}

fn arrange_vstack(
    children: &mut [LayoutNode],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    padding: f32,
    spacing: f32,
    alignment: Alignment,
) -> Result<(), LayoutError> {
    let inner_x = x + padding;
    let inner_y = y + padding;
    let inner_w = (w - 2.0 * padding).max(0.0);
    let inner_h = (h - 2.0 * padding).max(0.0);

    let child_desired: Vec<(f32, f32)> = children
        .iter()
        .map(|c| measure(c, inner_w, f32::INFINITY))
        .collect::<Result<Vec<_>, _>>()?;

    let n = children.len();
    let fill_count = children
        .iter()
        .filter(|c| c.height == SizeConstraint::Fill)
        .count();
    let non_fill_h: f32 = children
        .iter()
        .zip(child_desired.iter())
        .filter(|(c, _)| c.height != SizeConstraint::Fill)
        .map(|(_, &(_, h))| h)
        .sum();
    let spacing_total = if n > 0 {
        spacing * (n as f32 - 1.0)
    } else {
        0.0
    };
    let remaining = (inner_h - non_fill_h - spacing_total).max(0.0);
    let fill_child_h = if fill_count > 0 {
        remaining / fill_count as f32
    } else {
        0.0
    };

    let mut cur_y = inner_y;
    for (i, child) in children.iter_mut().enumerate() {
        let (desired_w, desired_h) = child_desired[i];

        let child_h = if child.height == SizeConstraint::Fill {
            fill_child_h
        } else {
            desired_h
        };

        let (child_x, child_w) =
            cross_axis_position(&child.width, desired_w, inner_x, inner_w, alignment);

        arrange(child, child_x, cur_y, child_w, child_h)?;
        cur_y += child_h;
        if i < n - 1 {
            cur_y += spacing;
        }
    }
    Ok(())
}

fn arrange_hstack(
    children: &mut [LayoutNode],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    padding: f32,
    spacing: f32,
    alignment: Alignment,
) -> Result<(), LayoutError> {
    let inner_x = x + padding;
    let inner_y = y + padding;
    let inner_w = (w - 2.0 * padding).max(0.0);
    let inner_h = (h - 2.0 * padding).max(0.0);

    let child_desired: Vec<(f32, f32)> = children
        .iter()
        .map(|c| measure(c, f32::INFINITY, inner_h))
        .collect::<Result<Vec<_>, _>>()?;

    let n = children.len();
    let fill_count = children
        .iter()
        .filter(|c| c.width == SizeConstraint::Fill)
        .count();
    let non_fill_w: f32 = children
        .iter()
        .zip(child_desired.iter())
        .filter(|(c, _)| c.width != SizeConstraint::Fill)
        .map(|(_, &(w, _))| w)
        .sum();
    let spacing_total = if n > 0 {
        spacing * (n as f32 - 1.0)
    } else {
        0.0
    };
    let remaining = (inner_w - non_fill_w - spacing_total).max(0.0);
    let fill_child_w = if fill_count > 0 {
        remaining / fill_count as f32
    } else {
        0.0
    };

    let mut cur_x = inner_x;
    for (i, child) in children.iter_mut().enumerate() {
        let (desired_w, desired_h) = child_desired[i];

        let child_w = if child.width == SizeConstraint::Fill {
            fill_child_w
        } else {
            desired_w
        };

        let (child_y, child_h) =
            cross_axis_position(&child.height, desired_h, inner_y, inner_h, alignment);

        arrange(child, cur_x, child_y, child_w, child_h)?;
        cur_x += child_w;
        if i < n - 1 {
            cur_x += spacing;
        }
    }
    Ok(())
}

// Computes the cross-axis position and size for a child.
// Fill constraint and Stretch alignment both expand to the full inner extent.
fn cross_axis_position(
    constraint: &SizeConstraint,
    desired: f32,
    inner_start: f32,
    inner_extent: f32,
    alignment: Alignment,
) -> (f32, f32) {
    if *constraint == SizeConstraint::Fill || alignment == Alignment::Stretch {
        return (inner_start, inner_extent);
    }
    let d = desired.min(inner_extent);
    match alignment {
        Alignment::Leading => (inner_start, d),
        Alignment::Center => (inner_start + (inner_extent - d) / 2.0, d),
        Alignment::Trailing => (inner_start + inner_extent - d, d),
        Alignment::Stretch => unreachable!(),
    }
}

/// Top-level entry point: resolves the root node against window size, then arranges.
pub fn run_layout(root: &mut LayoutNode, window_w: f32, window_h: f32) -> Result<(), LayoutError> {
    let (desired_w, desired_h) = measure(root, window_w, window_h)?;
    let final_w = resolve_axis(&root.width, desired_w, window_w);
    let final_h = resolve_axis(&root.height, desired_h, window_h);
    arrange(root, 0.0, 0.0, final_w, final_h)
}

fn resolve_axis(constraint: &SizeConstraint, desired: f32, available: f32) -> f32 {
    match constraint {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => available,
        SizeConstraint::Shrink => desired,
    }
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_fixed_size() {
        let mut rect =
            LayoutNode::rectangle(SizeConstraint::Fixed(100.0), SizeConstraint::Fixed(50.0));
        arrange(&mut rect, 10.0, 20.0, 100.0, 50.0).unwrap();
        assert_eq!(rect.offset, (10.0, 20.0));
        assert_eq!(rect.size, (100.0, 50.0));
    }

    #[test]
    fn vstack_three_fixed_rects() {
        // VStack(spacing=10, padding=0) containing three 100×40 rectangles.
        // Expected: stacked vertically, each at correct y offset.
        let mut stack = LayoutNode::vstack(10.0, 0.0, Alignment::Stretch);
        for _ in 0..3 {
            stack.children.push(LayoutNode::rectangle(
                SizeConstraint::Fixed(100.0),
                SizeConstraint::Fixed(40.0),
            ));
        }

        run_layout(&mut stack, 400.0, 600.0).unwrap();

        // VStack width=Fill → 400; height=Shrink → 3*40 + 2*10 = 140
        assert_eq!(stack.size, (400.0, 140.0));
        assert_eq!(stack.children[0].offset, (0.0, 0.0));
        assert_eq!(stack.children[0].size, (400.0, 40.0)); // Stretch → fills width
        assert_eq!(stack.children[1].offset, (0.0, 50.0)); // 40 + 10
        assert_eq!(stack.children[2].offset, (0.0, 100.0)); // 40 + 10 + 40 + 10
    }

    #[test]
    fn vstack_with_padding() {
        let mut stack = LayoutNode::vstack(0.0, 20.0, Alignment::Stretch);
        stack.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(60.0),
            SizeConstraint::Fixed(30.0),
        ));

        run_layout(&mut stack, 200.0, 600.0).unwrap();

        // height = 30 + 2*20 = 70
        assert_eq!(stack.size, (200.0, 70.0));
        // child starts at (padding, padding) = (20, 20), width = 200 - 40 = 160 (Stretch)
        assert_eq!(stack.children[0].offset, (20.0, 20.0));
        assert_eq!(stack.children[0].size, (160.0, 30.0));
    }

    #[test]
    fn vstack_fill_child_takes_remaining() {
        // One fixed rect (40px) + one Fill rect inside a 200px VStack.
        let mut stack = LayoutNode::vstack(0.0, 0.0, Alignment::Stretch);
        stack.height = SizeConstraint::Fill; // override to fill window height

        stack.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(100.0),
            SizeConstraint::Fixed(40.0),
        ));
        let fill_rect = LayoutNode::rectangle(SizeConstraint::Fixed(100.0), SizeConstraint::Fill);
        stack.children.push(fill_rect);

        run_layout(&mut stack, 200.0, 200.0).unwrap();

        assert_eq!(stack.size, (200.0, 200.0));
        assert_eq!(stack.children[0].size.1, 40.0);
        assert_eq!(stack.children[1].size.1, 160.0); // 200 - 40
    }

    #[test]
    fn hstack_three_fixed_rects() {
        let mut stack = LayoutNode::hstack(8.0, 0.0, Alignment::Stretch);
        for _ in 0..3 {
            stack.children.push(LayoutNode::rectangle(
                SizeConstraint::Fixed(50.0),
                SizeConstraint::Fixed(30.0),
            ));
        }

        run_layout(&mut stack, 600.0, 200.0).unwrap();

        // HStack width=Shrink → 3*50 + 2*8 = 166; height=Fill → 200
        assert_eq!(stack.size, (166.0, 200.0));
        assert_eq!(stack.children[0].offset, (0.0, 0.0));
        assert_eq!(stack.children[1].offset, (58.0, 0.0)); // 50 + 8
        assert_eq!(stack.children[2].offset, (116.0, 0.0)); // 50 + 8 + 50 + 8
    }

    #[test]
    fn vstack_center_alignment() {
        let mut stack = LayoutNode::vstack(0.0, 0.0, Alignment::Center);
        stack.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(60.0),
            SizeConstraint::Fixed(30.0),
        ));

        run_layout(&mut stack, 200.0, 600.0).unwrap();

        // child width = 60, centered in 200 → x = (200 - 60) / 2 = 70
        assert_eq!(stack.children[0].offset.0, 70.0);
        assert_eq!(stack.children[0].size.0, 60.0);
    }

    #[test]
    fn degenerate_fill_in_shrink_parent_clamps_to_zero() {
        // Fill child inside a Shrink parent → remaining = 0, child height = 0
        let mut stack = LayoutNode::vstack(0.0, 0.0, Alignment::Stretch);
        // height stays Shrink (default)
        let fill_child = LayoutNode::rectangle(SizeConstraint::Fixed(50.0), SizeConstraint::Fill);
        stack.children.push(fill_child);

        run_layout(&mut stack, 200.0, 600.0).unwrap();

        assert_eq!(stack.children[0].size.1, 0.0);
    }

    // M3-Phase 4 T6: pin the Shrink-VStack-root + Fill-ScrollView-child
    // collapse using a gallery-shaped fixture (mixed Shrink-height
    // WrapPanel + Fixed-height Buttons + Fill-height ScrollView), and
    // verify that pre-setting the same root to Fill height (the override
    // applied by `WidgetNode::run_layout`) flips the Fill child's
    // allocated height to non-zero. Documents the basal trap at the
    // layer where the runtime boundary's policy is enacted; pairs with
    // the mock-free `WidgetNode::run_layout` integration test in
    // `tests/scroll_view_layout_integration.rs::scroll_path_vstack_root_*`
    // which exercises the same shape through the production Composition
    // path. See progress doc Decisions log "T6 smoke failure mode A
    // disposition (2026-05-25)".
    #[test]
    fn shrink_vstack_root_with_fill_scroll_view_child_collapses() {
        let mut root = LayoutNode::vstack(12.0, 12.0, Alignment::Stretch);
        // root keeps default (width: Fill, height: Shrink), matching
        // `WidgetNode::vstack` defaults the IR loader hands out for an
        // authored `VStack { ... }`.

        // Phase 3 standalone WrapPanel slice approximation: Shrink height,
        // Fixed inner main extent. The WrapPanel's measured height does
        // not matter for the assertion below; we only need a non-Fill
        // sibling that consumes some of the VStack's Shrink desired_h.
        let mut wrap = LayoutNode::wrap_panel(Some(88.0), 12.0, 12.0);
        wrap.width = SizeConstraint::Fill;
        wrap.height = SizeConstraint::Shrink;
        for _ in 0..4 {
            let b = LayoutNode::rectangle(SizeConstraint::Fixed(88.0), SizeConstraint::Fixed(88.0));
            wrap.children.push(b);
        }
        root.children.push(wrap);

        // Two Fixed-height Buttons (Rectangle stand-ins).
        for _ in 0..2 {
            root.children.push(LayoutNode::rectangle(
                SizeConstraint::Fixed(160.0),
                SizeConstraint::Fixed(32.0),
            ));
        }

        // Fill-height ScrollView with one Fill-width Shrink-height child.
        let mut sv = LayoutNode::scroll_view(0);
        sv.width = SizeConstraint::Fill;
        sv.height = SizeConstraint::Fill;
        let mut inner = LayoutNode::wrap_panel(Some(64.0), 8.0, 8.0);
        inner.width = SizeConstraint::Fill;
        inner.height = SizeConstraint::Shrink;
        for _ in 0..32 {
            inner.children.push(LayoutNode::rectangle(
                SizeConstraint::Fixed(64.0),
                SizeConstraint::Fixed(64.0),
            ));
        }
        sv.children.push(inner);
        root.children.push(sv);

        // Reproduce the original bug at the pure-layout level: with the
        // root still in its default Shrink height, the Fill ScrollView
        // child collapses to a zero outer rect.
        run_layout(&mut root, 1000.0, 740.0).unwrap();
        let sv_idx = root.children.len() - 1;
        assert_eq!(
            root.children[sv_idx].size.1, 0.0,
            "Shrink-VStack-root with Fill-ScrollView child must collapse \
             the ScrollView to height 0 — pinned to mirror \
             degenerate_fill_in_shrink_parent_clamps_to_zero for the \
             gallery-shaped fixture (T6 failure mode A)",
        );

        // Apply the runtime-boundary override and re-run: the ScrollView
        // now receives the remaining height after non-Fill siblings,
        // and arrange_scroll_view assigns it a non-zero viewport.
        root.height = SizeConstraint::Fill;
        run_layout(&mut root, 1000.0, 740.0).unwrap();
        assert!(
            root.children[sv_idx].size.1 > 0.0,
            "with the root forced to Fill height (mirroring \
             WidgetNode::run_layout's override), the Fill ScrollView \
             child must receive a non-zero viewport allocation; got {}",
            root.children[sv_idx].size.1,
        );
    }

    // ── M3-Phase 2 T8: Box aspect measure-arrange (DD-M3-P2-005) ───────────

    fn box_with_aspect(num: i32, den: i32) -> LayoutNode {
        LayoutNode::box_(Some(Ratio { num, den }))
    }

    fn box_no_aspect() -> LayoutNode {
        LayoutNode::box_(None)
    }

    #[test]
    fn box_aspect_inscribed_width_constrained() {
        // 16:9 in 800×800 → width-constrained → 800 × 450 (touches width).
        let mut root = box_with_aspect(16, 9);
        run_layout(&mut root, 800.0, 800.0).unwrap();
        assert_eq!(root.offset, (0.0, 0.0));
        assert_eq!(root.size, (800.0, 450.0));
    }

    #[test]
    fn box_aspect_inscribed_height_constrained() {
        // 16:9 in 100×100 → height-constrained → ≈177.78 × 100. Use 100x100
        // would give equal-on-touch (100*9=900 vs 100*16=1600 → width-branch
        // by ≤; for a height-constrained case we need W*den > H*num).
        // Use 200×50: 200*9=1800 > 50*16=800 → height-constrained → (50*16/9, 50)
        let mut root = box_with_aspect(16, 9);
        run_layout(&mut root, 200.0, 50.0).unwrap();
        assert_eq!(root.size, (50.0 * 16.0 / 9.0, 50.0));
    }

    #[test]
    fn box_aspect_equal_touch_takes_width_branch() {
        // 16:9 in 1600×900 → W*den = H*num exactly. The contract picks the
        // width branch on equality (`<=`).
        let mut root = box_with_aspect(16, 9);
        run_layout(&mut root, 1600.0, 900.0).unwrap();
        assert_eq!(root.size, (1600.0, 900.0));
    }

    #[test]
    fn box_aspect_unbounded_height_uses_bounded_axis_wins() {
        // Box(aspect=16:9) inside an HStack child whose height is Fixed and
        // whose parent leaves width unbounded (HStack passes width=INFINITY
        // to children). Drive it directly by calling measure() with
        // (W=400, H=INF) → expect (400, 225).
        let bx = box_with_aspect(16, 9);
        let (w, h) = measure(&bx, 400.0, f32::INFINITY).unwrap();
        assert_eq!((w, h), (400.0, 400.0 * 9.0 / 16.0));
    }

    #[test]
    fn box_aspect_unbounded_width_uses_bounded_axis_wins() {
        // Symmetric: (W=INF, H=300) → derive width from height.
        let bx = box_with_aspect(16, 9);
        let (w, h) = measure(&bx, f32::INFINITY, 300.0).unwrap();
        assert_eq!((w, h), (300.0 * 16.0 / 9.0, 300.0));
    }

    #[test]
    fn box_aspect_unbounded_both_axes_is_runtime_error() {
        let bx = box_with_aspect(16, 9);
        let err = measure(&bx, f32::INFINITY, f32::INFINITY).unwrap_err();
        assert_eq!(err, LayoutError::BoxAspectUnboundedBoth);
    }

    #[test]
    fn box_no_aspect_empty_matches_parent_bounds() {
        let mut root = box_no_aspect();
        run_layout(&mut root, 640.0, 480.0).unwrap();
        assert_eq!(root.size, (640.0, 480.0));
        assert_eq!(root.offset, (0.0, 0.0));
    }

    #[test]
    fn box_no_aspect_empty_unbounded_both_is_runtime_error() {
        let bx = box_no_aspect();
        let err = measure(&bx, f32::INFINITY, f32::INFINITY).unwrap_err();
        assert_eq!(err, LayoutError::BoxNoExtent);
    }

    #[test]
    fn box_no_aspect_empty_one_axis_unbounded_collapses_to_zero() {
        // Scrim-only Box in an intrinsic-sizing context paints a
        // zero-thickness strip on the unbounded axis.
        let bx = box_no_aspect();
        let (w, h) = measure(&bx, 200.0, f32::INFINITY).unwrap();
        assert_eq!((w, h), (200.0, 0.0));
        let (w, h) = measure(&bx, f32::INFINITY, 120.0).unwrap();
        assert_eq!((w, h), (0.0, 120.0));
    }

    #[test]
    fn box_no_aspect_shrinks_to_fit_child() {
        // Box (no aspect) with a 40×20 child inside a 400×300 parent →
        // shrink-to-fit to the child's intrinsic measure.
        let mut root = box_no_aspect();
        root.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(40.0),
            SizeConstraint::Fixed(20.0),
        ));
        run_layout(&mut root, 400.0, 300.0).unwrap();
        assert_eq!(root.size, (40.0, 20.0));
        // Child is centred inside the shrink-to-fit Box (rw=40, cw=40 → cx=0).
        assert_eq!(root.children[0].offset, (0.0, 0.0));
        assert_eq!(root.children[0].size, (40.0, 20.0));
    }

    #[test]
    fn box_aspect_child_measured_centred_and_intrinsic_kept() {
        // Box(aspect=16:9) in 800×800 → resolved 800×450. Child 100×40
        // (well within bounds) → centred at ((800-100)/2, (450-40)/2).
        let mut root = box_with_aspect(16, 9);
        root.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(100.0),
            SizeConstraint::Fixed(40.0),
        ));
        run_layout(&mut root, 800.0, 800.0).unwrap();
        assert_eq!(root.size, (800.0, 450.0));
        assert_eq!(root.children[0].size, (100.0, 40.0));
        assert_eq!(root.children[0].offset, (350.0, 205.0));
    }

    #[test]
    fn box_aspect_oversize_child_clipped_to_box_bounds() {
        // Box(aspect=16:9) in 200×200 → resolved 200×112.5. Child 400×400
        // (overflows on both axes) → clipped to (200, 112.5), centred → (0,0).
        let mut root = box_with_aspect(16, 9);
        root.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(400.0),
            SizeConstraint::Fixed(400.0),
        ));
        run_layout(&mut root, 200.0, 200.0).unwrap();
        assert_eq!(root.size, (200.0, 112.5));
        assert_eq!(root.children[0].size, (200.0, 112.5));
        assert_eq!(root.children[0].offset, (0.0, 0.0));
    }

    #[test]
    fn box_aspect_in_vstack_uses_inscribed_via_bounded_axis_wins() {
        // VStack(width=Fill, height=Shrink) with a Box(aspect=16:9) child
        // inside a 400×600 window. VStack passes (inner_w=400, INF) to the
        // Box, so it bounded-axis-wins to (400, 225); VStack shrinks to that.
        let mut stack = LayoutNode::vstack(0.0, 0.0, Alignment::Stretch);
        stack.children.push(box_with_aspect(16, 9));
        run_layout(&mut stack, 400.0, 600.0).unwrap();
        assert_eq!(stack.size, (400.0, 225.0));
        assert_eq!(stack.children[0].size, (400.0, 225.0));
        assert_eq!(stack.children[0].offset, (0.0, 0.0));
    }

    #[test]
    fn box_zero_child_still_has_size() {
        // The "filled with `fill` or transparent" rendering layer is on
        // the SpriteVisual brush (widget.rs); the layout-level guarantee
        // is that the rectangle has the resolved aspect-derived size.
        let mut root = box_with_aspect(4, 3);
        run_layout(&mut root, 600.0, 400.0).unwrap();
        // 4:3 in 600×400: 600*3 = 1800 ≤ 400*4 = 1600? 1800 > 1600 → height-branch.
        // height-constrained → (400*4/3, 400) ≈ (533.33, 400)
        assert_eq!(root.size, (400.0 * 4.0 / 3.0, 400.0));
    }

    // ── M3-Phase 3 T7: WrapPanel line-breaker and arrange (DD-M3-P3-005) ───

    fn wrap_panel_with_boxes(
        ics: Option<f32>,
        item_spacing: f32,
        line_spacing: f32,
        boxes: &[(i32, i32)],
    ) -> LayoutNode {
        let mut wp = LayoutNode::wrap_panel(ics, item_spacing, line_spacing);
        for &(num, den) in boxes {
            wp.children.push(LayoutNode::box_(Some(Ratio { num, den })));
        }
        wp
    }

    #[test]
    fn wrap_panel_zero_children_measures_zero() {
        // DD-M3-P3-001 0-child shape: line set is empty, cross extent is 0.
        let wp = LayoutNode::wrap_panel(Some(88.0), 12.0, 12.0);
        let (_, h) = measure(&wp, 800.0, f32::INFINITY).unwrap();
        assert_eq!(h, 0.0);
    }

    #[test]
    fn wrap_panel_bounded_single_line_no_wrap() {
        // Three 1:1 Boxes at 50×50 inside a 200-wide WrapPanel with
        // item_spacing=10: 50 + 10 + 50 + 10 + 50 = 170 ≤ 200 → all fit
        // on one line. Cross extent = 50 (uniform per DD-004 Option A).
        let mut wp = wrap_panel_with_boxes(Some(50.0), 10.0, 8.0, &[(1, 1), (1, 1), (1, 1)]);
        run_layout(&mut wp, 200.0, 200.0).unwrap();
        assert_eq!(wp.size, (200.0, 50.0));
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
        assert_eq!(wp.children[1].offset, (60.0, 0.0)); // 50 + 10
        assert_eq!(wp.children[2].offset, (120.0, 0.0)); // 50 + 10 + 50 + 10
        assert_eq!(wp.children[0].size, (50.0, 50.0));
    }

    #[test]
    fn wrap_panel_bounded_multi_line_wraps() {
        // 130-wide parent, three 50×50 Boxes, item_spacing=10, line_spacing=8.
        // Line 1: child0(50), child1(50+10+50=110) — child2 would be
        // 110+10+50=170 > 130 → new line. Line 2: child2(50).
        // Outer: main=130, cross=50+50+8=108.
        let mut wp = wrap_panel_with_boxes(Some(50.0), 10.0, 8.0, &[(1, 1), (1, 1), (1, 1)]);
        run_layout(&mut wp, 130.0, 300.0).unwrap();
        assert_eq!(wp.size, (130.0, 108.0));
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
        assert_eq!(wp.children[1].offset, (60.0, 0.0));
        assert_eq!(wp.children[2].offset, (0.0, 58.0)); // 50 + 8
    }

    #[test]
    fn wrap_panel_spacing_aware_inequality_uses_less_equal() {
        // 110-wide parent, two 50×50 Boxes, item_spacing=10:
        // 50 + 10 + 50 = 110 == 110 → fits per `<=` (DD-001 / DD-005).
        let mut wp = wrap_panel_with_boxes(Some(50.0), 10.0, 0.0, &[(1, 1), (1, 1)]);
        run_layout(&mut wp, 110.0, 200.0).unwrap();
        assert_eq!(wp.size, (110.0, 50.0));
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
        assert_eq!(wp.children[1].offset, (60.0, 0.0));
    }

    #[test]
    fn wrap_panel_no_trailing_item_spacing() {
        // 200-wide parent, two 50×50 Boxes, item_spacing=20.
        // Cumulative content extent is 50+20+50=120; the WrapPanel's
        // outer main equals parent main bound (200), not 120 — that's
        // a Fill-width convention. The check here is that the second
        // child does *not* have trailing item_spacing accruing past it
        // (line_main == 120, not 140).
        let mut wp = wrap_panel_with_boxes(Some(50.0), 20.0, 0.0, &[(1, 1), (1, 1)]);
        run_layout(&mut wp, 200.0, 200.0).unwrap();
        assert_eq!(wp.size, (200.0, 50.0));
        // Children sit flush-left; no trailing margin would shift child[1]
        // outside the expected (50 + 20) = 70 offset.
        assert_eq!(wp.children[1].offset.0, 70.0);
    }

    #[test]
    fn wrap_panel_unbounded_main_axis_one_line_flow() {
        // DD-M3-P3-005 unbounded-main Option A: parent gives INFINITY
        // main; the line breaker degenerates to all-children-on-one-line.
        // Three 50×50 Boxes, item_spacing=10 → one line, cumulative
        // main = 50+10+50+10+50 = 170.
        let wp = wrap_panel_with_boxes(Some(50.0), 10.0, 0.0, &[(1, 1), (1, 1), (1, 1)]);
        let (main, cross) = measure(&wp, f32::INFINITY, 200.0).unwrap();
        // Width=Fill on unbounded main reports the cumulative anchor.
        assert_eq!(main, 170.0);
        assert_eq!(cross, 50.0);
    }

    #[test]
    fn wrap_panel_oversized_first_child_placed_unconditionally() {
        // Box(aspect=4:1) with item_cross_size=50 → measured (200, 50).
        // Parent main bound = 100; the oversized first child is placed
        // anyway (DD-M3-P3-005 oversized-first-child Option A).
        let mut wp = wrap_panel_with_boxes(Some(50.0), 0.0, 0.0, &[(4, 1)]);
        run_layout(&mut wp, 100.0, 100.0).unwrap();
        // WrapPanel outer main stays at parent_main_bound (= 100); the
        // oversized line does NOT grow the WrapPanel rectangle.
        assert_eq!(wp.size, (100.0, 50.0));
        // The child paints at its measured extent — width 200, x = 0.
        assert_eq!(wp.children[0].size, (200.0, 50.0));
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
    }

    #[test]
    fn wrap_panel_arrange_visible_overflow_for_oversized_child() {
        // Same fixture as above — the arrange-pass observable form of
        // "child paints past the WrapPanel rectangle" is
        // child.offset.0 + child.size.0 > wp.offset.0 + wp.size.0.
        // ADR verification closure evidence item 2 (oversized-first-child
        // arrange evidence).
        let mut wp = wrap_panel_with_boxes(Some(50.0), 0.0, 0.0, &[(4, 1)]);
        run_layout(&mut wp, 100.0, 100.0).unwrap();
        let child_main_end = wp.children[0].offset.0 + wp.children[0].size.0;
        let wp_main_end = wp.offset.0 + wp.size.0;
        assert!(
            child_main_end > wp_main_end,
            "expected visible overflow: child end {} should exceed WrapPanel end {}",
            child_main_end,
            wp_main_end
        );
    }

    #[test]
    fn wrap_panel_oversized_first_child_then_normal_children() {
        // Oversized child closes a line on its own; subsequent children
        // start a new line where the unconditional-placement rule
        // re-applies. Parent main=100, item_spacing=0:
        //   child0(4:1 → 200 wide) on line 0 (oversized)
        //   child1(1:1 → 50 wide), child2(1:1 → 50 wide) on line 1
        //     (50+0+50 = 100 ≤ 100 fits)
        let mut wp = wrap_panel_with_boxes(Some(50.0), 0.0, 0.0, &[(4, 1), (1, 1), (1, 1)]);
        run_layout(&mut wp, 100.0, 200.0).unwrap();
        // 2 lines × 50 cross + 0 line-spacing = 100 cross
        assert_eq!(wp.size, (100.0, 100.0));
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
        assert_eq!(wp.children[1].offset, (0.0, 50.0));
        assert_eq!(wp.children[2].offset, (50.0, 50.0));
    }

    #[test]
    fn wrap_panel_cross_axis_uniform_when_item_cross_size_set() {
        // item_cross_size=50; Box aspect 1:1 → child cross == 50.
        // Line cross extent equals item_cross_size uniformly per
        // DD-M3-P3-004 per-line sizing Option A.
        let mut wp = wrap_panel_with_boxes(Some(50.0), 0.0, 10.0, &[(1, 1), (1, 1), (1, 1)]);
        run_layout(&mut wp, 100.0, 300.0).unwrap();
        // 100 parent main: child0(50) + child1(50) = 100 fits, child2 wraps.
        // Lines: 2; cross = 50 + 10 + 50 = 110.
        assert_eq!(wp.size, (100.0, 110.0));
    }

    #[test]
    fn wrap_panel_cross_axis_max_of_children_when_item_cross_size_unset() {
        // No item_cross_size; each child measured with parent cross
        // (200 here). Use plain Rectangles (no aspect) so children have
        // independent intrinsic cross sizes.
        let mut wp = LayoutNode::wrap_panel(None, 0.0, 0.0);
        wp.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(40.0),
            SizeConstraint::Fixed(30.0),
        ));
        wp.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(50.0),
            SizeConstraint::Fixed(80.0),
        ));
        run_layout(&mut wp, 200.0, 200.0).unwrap();
        // Both fit on one line (40 + 50 = 90 ≤ 200).
        // Line cross = max(30, 80) = 80.
        assert_eq!(wp.size, (200.0, 80.0));
    }

    #[test]
    fn wrap_panel_cross_axis_center_alignment_within_line() {
        // item_cross_size=80; child cross=20 (Fixed rectangle). Centred
        // within line: child y = 0 + (80 - 20) / 2 = 30.
        let mut wp = LayoutNode::wrap_panel(Some(80.0), 0.0, 0.0);
        wp.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(40.0),
            SizeConstraint::Fixed(20.0),
        ));
        run_layout(&mut wp, 200.0, 200.0).unwrap();
        assert_eq!(wp.size, (200.0, 80.0));
        assert_eq!(wp.children[0].offset, (0.0, 30.0));
        assert_eq!(wp.children[0].size, (40.0, 20.0));
    }

    #[test]
    fn wrap_panel_zero_item_spacing_touching_items() {
        // DD-M3-P3-006 zero-handling: item_spacing=0 is valid; items
        // touch on the main axis.
        let mut wp = wrap_panel_with_boxes(Some(40.0), 0.0, 0.0, &[(1, 1), (1, 1), (1, 1)]);
        run_layout(&mut wp, 200.0, 200.0).unwrap();
        assert_eq!(wp.children[0].offset.0, 0.0);
        assert_eq!(wp.children[1].offset.0, 40.0);
        assert_eq!(wp.children[2].offset.0, 80.0);
    }

    #[test]
    fn wrap_panel_zero_line_spacing_touching_lines() {
        // 80-wide parent forces a wrap; line_spacing=0 makes lines touch.
        let mut wp = wrap_panel_with_boxes(Some(50.0), 0.0, 0.0, &[(1, 1), (1, 1)]);
        run_layout(&mut wp, 80.0, 200.0).unwrap();
        // 50 + 0 + 50 = 100 > 80 → wrap.
        assert_eq!(wp.size, (80.0, 100.0)); // 50 + 0 + 50 = 100
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
        assert_eq!(wp.children[1].offset, (0.0, 50.0));
    }

    #[test]
    fn wrap_panel_zero_item_cross_size_degenerate_layout() {
        // DD-M3-P3-006 author-requested degenerate: item_cross_size=0 →
        // each line collapses to zero cross extent; line count still
        // computed. Three Boxes, aspect 1:1, parent main=200, ics=0:
        // children measured with (INF, 0) → bounded-axis-wins → (0, 0).
        // All fit on one line (0 + 0 + 0 = 0 ≤ 200).
        let mut wp = wrap_panel_with_boxes(Some(0.0), 0.0, 0.0, &[(1, 1), (1, 1), (1, 1)]);
        run_layout(&mut wp, 200.0, 200.0).unwrap();
        assert_eq!(wp.size, (200.0, 0.0));
    }

    #[test]
    fn wrap_panel_unbounded_cross_with_aspect_child_propagates_box_error() {
        // DD-M3-P3-005 unbounded-cross Option A: no `item-cross-size` on
        // WrapPanel + parent cross unbounded → child Box(aspect) measured
        // with (INF, INF) → Phase 2 `LayoutError::BoxAspectUnboundedBoth`.
        // Drive directly via `measure` to keep the fixture small.
        let wp = wrap_panel_with_boxes(None, 0.0, 0.0, &[(1, 1)]);
        let err = measure(&wp, 200.0, f32::INFINITY).unwrap_err();
        assert_eq!(err, LayoutError::BoxAspectUnboundedBoth);
    }

    #[test]
    fn wrap_panel_unset_item_cross_size_measure_arrange_consistent() {
        // Regression for the measure→arrange drift caught in T7 review:
        // when `item-cross-size` is unset, the spec's DD-M3-P3-004
        // Option (a) routes the parent of WrapPanel's cross-axis bound
        // through to each child. measure receives `avail_h` from its
        // parent and passes it down. Under the default `height: Shrink`
        // the WrapPanel's own arrange-time `h` collapses to
        // `desired_h` (= sum of line cross extents), which is **not**
        // the same value. The fix caches the measure-time cross input
        // on the node and re-uses it at arrange so the line breaker
        // produces the same line layout in both passes.
        //
        // Fixture (reviewer's example): WrapPanel(no item-cross-size,
        // default height: Shrink), three Box{aspect:1:1}, parent
        // 250×100.
        //   measure(250, 100): child cross bound = 100 → children
        //     (100, 100). Line break 250 bound: 100 / 200 fit, 300 >
        //     250 wraps. 2 lines, outer (250, 200).
        //   arrange(250, 200): under the buggy code the cross bound
        //     would re-derive as h=200 → children (200, 200), 1 per
        //     line, 3 lines stacked to 600 cross — overflowing the
        //     allocated 200. With the cache, cross bound stays 100,
        //     the 2-line layout matches.
        let mut wp = wrap_panel_with_boxes(None, 0.0, 0.0, &[(1, 1), (1, 1), (1, 1)]);
        run_layout(&mut wp, 250.0, 100.0).unwrap();
        assert_eq!(wp.size, (250.0, 200.0));
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
        assert_eq!(wp.children[0].size, (100.0, 100.0));
        assert_eq!(wp.children[1].offset, (100.0, 0.0));
        assert_eq!(wp.children[1].size, (100.0, 100.0));
        assert_eq!(wp.children[2].offset, (0.0, 100.0));
        assert_eq!(wp.children[2].size, (100.0, 100.0));
    }

    #[test]
    fn wrap_panel_arrange_without_prior_measure_falls_back_to_h() {
        // Cache fallback contract: a direct `arrange` call without a
        // prior `measure` (uncommon outside tests) finds the cache at
        // its `f32::NAN` sentinel and uses the allocated `h` as the
        // cross bound. There is no measure result to be consistent
        // with in this case, so the fallback preserves a self-
        // consistent stand-alone arrange.
        let mut wp = wrap_panel_with_boxes(None, 0.0, 0.0, &[(1, 1)]);
        // Skip measure; call arrange directly with a known cell.
        arrange(&mut wp, 0.0, 0.0, 200.0, 60.0).unwrap();
        // Box(1:1) measured against (INF, 60) → (60, 60). One line,
        // line cross = 60, centred at 0.
        assert_eq!(wp.size, (200.0, 60.0));
        assert_eq!(wp.children[0].size, (60.0, 60.0));
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
    }

    #[test]
    fn wrap_panel_gallery_subscreen_shape() {
        // ADR verification closure evidence item 4 wrap-path fixture
        // dimensions, exercised as a pure-data shape sanity:
        //   item-cross-size: 88; item-spacing: 12; line-spacing: 12;
        //   5 Boxes aspect 1:1; parent main 250.
        // Per-line: 88 + 12 + 88 = 188; +12+88 = 288 > 250 → 2 per line.
        // 5 children → 3 lines (2 + 2 + 1).
        // outer: main=250, cross=88*3 + 12*2 = 264 + 24 = 288.
        let mut wp = wrap_panel_with_boxes(
            Some(88.0),
            12.0,
            12.0,
            &[(1, 1), (1, 1), (1, 1), (1, 1), (1, 1)],
        );
        run_layout(&mut wp, 250.0, 600.0).unwrap();
        assert_eq!(wp.size, (250.0, 288.0));
        assert_eq!(wp.children[0].offset, (0.0, 0.0));
        assert_eq!(wp.children[1].offset, (100.0, 0.0)); // 88 + 12
        assert_eq!(wp.children[2].offset, (0.0, 100.0)); // 88 + 12
        assert_eq!(wp.children[3].offset, (100.0, 100.0));
        assert_eq!(wp.children[4].offset, (0.0, 200.0));
        for c in &wp.children {
            assert_eq!(c.size, (88.0, 88.0));
        }
    }

    // ── M3-Phase 4 T2: ScrollView measure-arrange (DD-M3-P4-005) ────────────

    fn scroll_view_with_rect(offset_y: i32, content_w: f32, content_h: f32) -> LayoutNode {
        let mut sv = LayoutNode::scroll_view(offset_y);
        sv.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(content_w),
            SizeConstraint::Fixed(content_h),
        ));
        sv
    }

    #[test]
    fn scroll_view_content_smaller_than_viewport_anchors_top_leading() {
        // DD-M3-P4-005 "content smaller than viewport" sub-issue: content
        // paints at its measured size at the viewport's top-leading
        // corner; max_offset = 0; outer = viewport.
        let mut sv = scroll_view_with_rect(0, 100.0, 50.0);
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.size, (200.0, 300.0));
        assert_eq!(sv.children[0].size, (100.0, 50.0));
        assert_eq!(sv.children[0].offset, (0.0, 0.0));
        assert_eq!(sv.applied_offset_y.get(), 0.0);
    }

    #[test]
    fn scroll_view_content_equal_to_viewport_max_offset_zero() {
        // DD-M3-P4-005 boundary case: content_h == viewport_h → max_offset
        // = max(0, 0) = 0; applied = 0 regardless of `offset-y` value.
        let mut sv = scroll_view_with_rect(50, 200.0, 300.0);
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.size, (200.0, 300.0));
        assert_eq!(sv.applied_offset_y.get(), 0.0);
        assert_eq!(sv.children[0].offset, (0.0, 0.0));
    }

    #[test]
    fn scroll_view_content_larger_than_viewport_zero_offset_anchors_top() {
        // DD-M3-P4-005 "content exceeds viewport" sub-issue, offset 0
        // case: content paints at top-leading; viewport clip is the
        // Visual-layer concern (T4), not observable in pure layout.
        let mut sv = scroll_view_with_rect(0, 200.0, 500.0);
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.size, (200.0, 300.0));
        assert_eq!(sv.applied_offset_y.get(), 0.0);
        assert_eq!(sv.children[0].offset, (0.0, 0.0));
        assert_eq!(sv.children[0].size, (200.0, 500.0));
    }

    #[test]
    fn scroll_view_offset_clamp_negative_pins_to_zero() {
        // DD-M3-P4-005 clamp lower bound: `offset-y < 0` → applied = 0.
        let mut sv = scroll_view_with_rect(-50, 200.0, 500.0);
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.applied_offset_y.get(), 0.0);
        assert_eq!(sv.children[0].offset, (0.0, 0.0));
    }

    #[test]
    fn scroll_view_offset_clamp_zero_passes_through() {
        // DD-M3-P4-005 clamp at lower edge: `offset-y = 0` is in range
        // when content exceeds viewport; applied = 0.
        let mut sv = scroll_view_with_rect(0, 200.0, 500.0);
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.applied_offset_y.get(), 0.0);
        assert_eq!(sv.children[0].offset, (0.0, 0.0));
    }

    #[test]
    fn scroll_view_offset_clamp_mid_range_passes_through() {
        // DD-M3-P4-005 clamp mid-range: `0 < offset-y < max_offset`
        // applied unchanged; content translates upward by that amount.
        // max_offset = 500 - 300 = 200; offset-y = 100 → applied = 100.
        let mut sv = scroll_view_with_rect(100, 200.0, 500.0);
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.applied_offset_y.get(), 100.0);
        assert_eq!(sv.children[0].offset, (0.0, -100.0));
    }

    #[test]
    fn scroll_view_offset_clamp_at_max_holds() {
        // DD-M3-P4-005 clamp upper edge: `offset-y == max_offset` → applied
        // = max_offset.
        let mut sv = scroll_view_with_rect(200, 200.0, 500.0);
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.applied_offset_y.get(), 200.0);
        assert_eq!(sv.children[0].offset, (0.0, -200.0));
    }

    #[test]
    fn scroll_view_offset_clamp_above_max_pins_to_max() {
        // DD-M3-P4-005 clamp upper bound: `offset-y > max_offset` → applied
        // = max_offset.
        let mut sv = scroll_view_with_rect(500, 200.0, 500.0);
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.applied_offset_y.get(), 200.0);
        assert_eq!(sv.children[0].offset, (0.0, -200.0));
    }

    #[test]
    fn scroll_view_outer_size_equals_viewport_regardless_of_content() {
        // DD-M3-P4-005 outer-size invariant: ScrollView outer size = viewport
        // size, independent of content size. Covers tall (overflowing) and
        // short (under-filling) content in a single test.
        let mut sv_tall = scroll_view_with_rect(0, 200.0, 1000.0);
        run_layout(&mut sv_tall, 200.0, 300.0).unwrap();
        assert_eq!(sv_tall.size, (200.0, 300.0));
        let mut sv_short = scroll_view_with_rect(0, 50.0, 80.0);
        run_layout(&mut sv_short, 200.0, 300.0).unwrap();
        assert_eq!(sv_short.size, (200.0, 300.0));
    }

    #[test]
    fn scroll_view_unbounded_scroll_axis_parent_is_runtime_error() {
        // DD-M3-P4-002 unbounded-scroll-axis case: arrange called with
        // `h = INFINITY` fires `LayoutError::ScrollViewUnboundedAxis`. The
        // gate fires before children are measured so the error names the
        // structural problem rather than surfacing a child error first.
        let mut sv = scroll_view_with_rect(0, 200.0, 500.0);
        let err = arrange(&mut sv, 0.0, 0.0, 200.0, f32::INFINITY).unwrap_err();
        assert_eq!(err, LayoutError::ScrollViewUnboundedAxis);
    }

    #[test]
    fn scroll_view_measures_content_with_viewport_width_unbounded_height() {
        // DD-M3-P4-005 asymmetric measure: content receives
        // `(viewport_w, +∞)`. A Box(aspect 1:1) inside a (100, 50)
        // viewport bounded-axis-wins via the cross axis (viewport width)
        // and derives height = 100 — distinguishable from a symmetric
        // (100, 50) measure which would inscribed-fit to (50, 50).
        let mut sv = LayoutNode::scroll_view(0);
        sv.children
            .push(LayoutNode::box_(Some(Ratio { num: 1, den: 1 })));
        run_layout(&mut sv, 100.0, 50.0).unwrap();
        assert_eq!(sv.children[0].size, (100.0, 100.0));
        assert_eq!(sv.size, (100.0, 50.0));
    }

    #[test]
    fn scroll_view_fill_width_child_expands_to_viewport_width() {
        // Cross-axis child resolution: a `Fill`-width content child
        // expands to the viewport width (mirrors VStack's Fill-cross
        // convention). A 200-wide viewport with a Fill-width / Fixed-50
        // child arranges to (200, 50) at the viewport's top-leading
        // corner.
        let mut sv = LayoutNode::scroll_view(0);
        sv.children.push(LayoutNode::rectangle(
            SizeConstraint::Fill,
            SizeConstraint::Fixed(50.0),
        ));
        run_layout(&mut sv, 200.0, 300.0).unwrap();
        assert_eq!(sv.children[0].size, (200.0, 50.0));
        assert_eq!(sv.children[0].offset, (0.0, 0.0));
    }

    #[test]
    fn scroll_view_rounding_contract_no_pixel_snap() {
        // DD-M3-P4-005 rounding contract: `offset-y` is `i32` promoted to
        // `f32`; clamp arithmetic preserves the value without snapping.
        // content_h = 333, viewport_h = 200 → max_offset = 133;
        // offset-y = 33 → applied = 33.0 (no rounding to a pixel grid).
        let mut sv = scroll_view_with_rect(33, 200.0, 333.0);
        run_layout(&mut sv, 200.0, 200.0).unwrap();
        assert_eq!(sv.applied_offset_y.get(), 33.0);
        assert_eq!(sv.children[0].offset, (0.0, -33.0));
    }

    // ── M3-Phase 5 Grid (DD-M3-P5-004 / DD-M3-P5-005) — ADR evidence (2) ───────

    fn cell(
        row: u32,
        column: u32,
        row_span: u32,
        column_span: u32,
        h_align: Alignment,
        v_align: Alignment,
    ) -> CellPlacement {
        CellPlacement {
            row,
            column,
            row_span,
            column_span,
            h_align,
            v_align,
        }
    }

    // Stretch/stretch placement (the DD-M3-P5-005 default) at a single cell.
    fn stretch_cell(row: u32, column: u32, row_span: u32, column_span: u32) -> CellPlacement {
        cell(
            row,
            column,
            row_span,
            column_span,
            Alignment::Stretch,
            Alignment::Stretch,
        )
    }

    // A Fill/Fill content rectangle (stretches to the cell extent).
    fn fill_child() -> LayoutNode {
        LayoutNode::rectangle(SizeConstraint::Fill, SizeConstraint::Fill)
    }

    // ── Track resolution (DD-M3-P5-004 resolve_axis_tracks) ────────────────────

    #[test]
    fn grid_resolve_fixed_only() {
        let resolved = resolve_axis_tracks(
            &[TrackSize::Fixed(100), TrackSize::Fixed(200)],
            AxisBound::Bounded(500.0),
        )
        .unwrap();
        // Fixed tracks keep their declared size regardless of the bound;
        // trailing space stays inside the Grid.
        assert_eq!(resolved, vec![100.0, 200.0]);
    }

    #[test]
    fn grid_resolve_weighted_star_proportional() {
        // 1* : 2* over 300px → 100 : 200.
        let resolved = resolve_axis_tracks(
            &[TrackSize::Star(1), TrackSize::Star(2)],
            AxisBound::Bounded(300.0),
        )
        .unwrap();
        assert_eq!(resolved, vec![100.0, 200.0]);
    }

    #[test]
    fn grid_resolve_mixed_fixed_first_then_star() {
        // 180 1* 2* over 480px → fixed 180 consumed first, 300 remains,
        // split 1:2 → 100 : 200.
        let resolved = resolve_axis_tracks(
            &[
                TrackSize::Fixed(180),
                TrackSize::Star(1),
                TrackSize::Star(2),
            ],
            AxisBound::Bounded(480.0),
        )
        .unwrap();
        assert_eq!(resolved, vec![180.0, 100.0, 200.0]);
    }

    #[test]
    fn grid_resolve_negative_remaining_star_collapses_to_zero() {
        // Fixed sum 400 exceeds bound 300; star tracks resolve to 0 while
        // fixed tracks retain their declared size (DD-M3-P5-004 negative
        // remaining).
        let resolved = resolve_axis_tracks(
            &[
                TrackSize::Fixed(200),
                TrackSize::Fixed(200),
                TrackSize::Star(1),
            ],
            AxisBound::Bounded(300.0),
        )
        .unwrap();
        assert_eq!(resolved, vec![200.0, 200.0, 0.0]);
    }

    #[test]
    fn grid_resolve_unbounded_star_axis_errors() {
        // A star track on an unbounded axis has no finite space to divide.
        let err = resolve_axis_tracks(
            &[TrackSize::Fixed(100), TrackSize::Star(1)],
            AxisBound::Unbounded,
        )
        .unwrap_err();
        assert_eq!(err, LayoutError::GridUnboundedStarAxis);
    }

    #[test]
    fn grid_resolve_unbounded_fixed_only_resolves_to_fixed_sum() {
        // No star track: an unbounded axis resolves each fixed track to its
        // declared size (the axis extent is the fixed sum).
        let resolved = resolve_axis_tracks(
            &[TrackSize::Fixed(120), TrackSize::Fixed(80)],
            AxisBound::Unbounded,
        )
        .unwrap();
        assert_eq!(resolved, vec![120.0, 80.0]);
    }

    #[test]
    #[should_panic(expected = "zero total")]
    fn grid_resolve_star_zero_weight_panics_defensively() {
        // `Star(0)` is rejected at wasamoc check / validate() (DD-M3-P5-006);
        // reaching layout with it is a defended-against bug. The guard
        // prevents a divide-by-zero in star distribution.
        let _ = resolve_axis_tracks(&[TrackSize::Star(0)], AxisBound::Bounded(100.0));
    }

    #[test]
    fn grid_prefix_boundaries_are_cumulative() {
        let b = prefix_boundaries(&[100.0, 200.0, 50.0]);
        assert_eq!(b, vec![0.0, 100.0, 300.0, 350.0]);
        // Empty track list still yields the leading 0 boundary.
        assert_eq!(prefix_boundaries(&[]), vec![0.0]);
    }

    // ── Arrange (DD-M3-P5-004 cell rects + DD-M3-P5-005 placement) ─────────────

    #[test]
    fn grid_arrange_outer_rect_is_parent_allocation() {
        // DD-M3-P5-004 Grid outer rect on a bounded axis = parent allocation.
        let mut g = LayoutNode::grid(
            vec![TrackSize::Star(1)],
            vec![TrackSize::Star(1)],
            vec![stretch_cell(0, 0, 1, 1)],
        );
        g.children.push(fill_child());
        run_layout(&mut g, 400.0, 250.0).unwrap();
        assert_eq!(g.offset, (0.0, 0.0));
        assert_eq!(g.size, (400.0, 250.0));
    }

    #[test]
    fn grid_arrange_fixed_cell_rectangles() {
        // 2×2 fixed grid; each stretch cell fills its track intersection.
        let mut g = LayoutNode::grid(
            vec![TrackSize::Fixed(100), TrackSize::Fixed(200)],
            vec![TrackSize::Fixed(50), TrackSize::Fixed(80)],
            vec![
                stretch_cell(0, 0, 1, 1),
                stretch_cell(0, 1, 1, 1),
                stretch_cell(1, 0, 1, 1),
                stretch_cell(1, 1, 1, 1),
            ],
        );
        for _ in 0..4 {
            g.children.push(fill_child());
        }
        run_layout(&mut g, 300.0, 130.0).unwrap();
        assert_eq!(g.children[0].offset, (0.0, 0.0));
        assert_eq!(g.children[0].size, (100.0, 50.0));
        assert_eq!(g.children[1].offset, (100.0, 0.0));
        assert_eq!(g.children[1].size, (200.0, 50.0));
        assert_eq!(g.children[2].offset, (0.0, 50.0));
        assert_eq!(g.children[2].size, (100.0, 80.0));
        assert_eq!(g.children[3].offset, (100.0, 50.0));
        assert_eq!(g.children[3].size, (200.0, 80.0));
    }

    #[test]
    fn grid_arrange_weighted_star_cells() {
        // columns 1* 2* over 300 → 100 : 200; single full-height row.
        let mut g = LayoutNode::grid(
            vec![TrackSize::Star(1), TrackSize::Star(2)],
            vec![TrackSize::Star(1)],
            vec![stretch_cell(0, 0, 1, 1), stretch_cell(0, 1, 1, 1)],
        );
        g.children.push(fill_child());
        g.children.push(fill_child());
        run_layout(&mut g, 300.0, 90.0).unwrap();
        assert_eq!(g.children[0].offset, (0.0, 0.0));
        assert_eq!(g.children[0].size, (100.0, 90.0));
        assert_eq!(g.children[1].offset, (100.0, 0.0));
        assert_eq!(g.children[1].size, (200.0, 90.0));
    }

    #[test]
    fn grid_arrange_both_axis_spanning_cell() {
        // 3×3 fixed grid; one cell spans (row 1, col 1) over 2×2 tracks.
        let mut g = LayoutNode::grid(
            vec![
                TrackSize::Fixed(100),
                TrackSize::Fixed(100),
                TrackSize::Fixed(100),
            ],
            vec![
                TrackSize::Fixed(50),
                TrackSize::Fixed(50),
                TrackSize::Fixed(50),
            ],
            vec![
                stretch_cell(0, 0, 1, 3), // header spanning all columns
                stretch_cell(1, 1, 2, 2), // 2×2 spanning block
            ],
        );
        g.children.push(fill_child());
        g.children.push(fill_child());
        run_layout(&mut g, 300.0, 150.0).unwrap();
        // Header: columns 0..3 → x 0..300, row 0..1 → y 0..50.
        assert_eq!(g.children[0].offset, (0.0, 0.0));
        assert_eq!(g.children[0].size, (300.0, 50.0));
        // Spanning block: columns 1..3 → x 100..300, rows 1..3 → y 50..150.
        assert_eq!(g.children[1].offset, (100.0, 50.0));
        assert_eq!(g.children[1].size, (200.0, 100.0));
    }

    #[test]
    fn grid_arrange_alignment_within_cell() {
        // One 200×100 cell; a 50×40 fixed content rect under each alignment.
        let run = |h: Alignment, v: Alignment| -> ((f32, f32), (f32, f32)) {
            let mut g = LayoutNode::grid(
                vec![TrackSize::Fixed(200)],
                vec![TrackSize::Fixed(100)],
                vec![cell(0, 0, 1, 1, h, v)],
            );
            g.children.push(LayoutNode::rectangle(
                SizeConstraint::Fixed(50.0),
                SizeConstraint::Fixed(40.0),
            ));
            run_layout(&mut g, 200.0, 100.0).unwrap();
            (g.children[0].offset, g.children[0].size)
        };

        // start/start → anchored at the cell origin, natural size.
        assert_eq!(
            run(Alignment::Leading, Alignment::Leading),
            ((0.0, 0.0), (50.0, 40.0))
        );
        // center/center → centred, natural size.
        assert_eq!(
            run(Alignment::Center, Alignment::Center),
            ((75.0, 30.0), (50.0, 40.0))
        );
        // end/end → anchored at the far corner, natural size.
        assert_eq!(
            run(Alignment::Trailing, Alignment::Trailing),
            ((150.0, 60.0), (50.0, 40.0))
        );
        // stretch/stretch → fills the whole cell.
        assert_eq!(
            run(Alignment::Stretch, Alignment::Stretch),
            ((0.0, 0.0), (200.0, 100.0))
        );
        // mixed: center-h, stretch-v → centred horizontally, full height.
        assert_eq!(
            run(Alignment::Center, Alignment::Stretch),
            ((75.0, 0.0), (50.0, 100.0))
        );
    }

    #[test]
    fn grid_arrange_negative_remaining_overflows_outer_rect() {
        // Fixed columns 200+200 exceed the 300px parent allocation; Grid's
        // outer rect stays at the parent allocation (300) and the second
        // column's cell overflows past it (clipped at the outer-bounds clip
        // in T4; here we only assert the layout-side overflow).
        let mut g = LayoutNode::grid(
            vec![TrackSize::Fixed(200), TrackSize::Fixed(200)],
            vec![TrackSize::Fixed(100)],
            vec![stretch_cell(0, 0, 1, 1), stretch_cell(0, 1, 1, 1)],
        );
        g.children.push(fill_child());
        g.children.push(fill_child());
        run_layout(&mut g, 300.0, 100.0).unwrap();
        // Grid does not grow to 400; outer rect = parent allocation.
        assert_eq!(g.size, (300.0, 100.0));
        // Second cell starts at x=200 and extends to x=400 — past the outer
        // rect's right edge (300).
        assert_eq!(g.children[1].offset, (200.0, 0.0));
        assert_eq!(g.children[1].size, (200.0, 100.0));
        let right_edge = g.children[1].offset.0 + g.children[1].size.0;
        assert!(right_edge > g.offset.0 + g.size.0);
    }

    #[test]
    fn grid_arrange_unbounded_star_axis_errors() {
        // A Grid with a star column arranged against an unbounded width
        // surfaces the layout error (mirrors the ScrollView unbounded gate).
        let mut g = LayoutNode::grid(
            vec![TrackSize::Star(1)],
            vec![TrackSize::Fixed(100)],
            vec![stretch_cell(0, 0, 1, 1)],
        );
        g.children.push(fill_child());
        let err = arrange(&mut g, 0.0, 0.0, f32::INFINITY, 100.0).unwrap_err();
        assert_eq!(err, LayoutError::GridUnboundedStarAxis);
    }

    #[test]
    fn grid_arrange_preserves_document_order() {
        // `cell_placements[i]` always governs `children[i]`; the arrange loop
        // visits children in declared (document) order, which is the
        // DD-M3-P5-005 paint / z-order. Declare cells out of row-major order
        // and assert each child lands in its own declared cell (not reordered
        // by position).
        let mut g = LayoutNode::grid(
            vec![TrackSize::Fixed(100), TrackSize::Fixed(100)],
            vec![TrackSize::Fixed(50)],
            vec![
                stretch_cell(0, 1, 1, 1), // first child → right column
                stretch_cell(0, 0, 1, 1), // second child → left column
            ],
        );
        g.children.push(fill_child());
        g.children.push(fill_child());
        run_layout(&mut g, 200.0, 50.0).unwrap();
        // child[0] declared first → right column (x=100), regardless of
        // having a higher column index than child[1].
        assert_eq!(g.children[0].offset, (100.0, 0.0));
        assert_eq!(g.children[1].offset, (0.0, 0.0));
    }

    #[test]
    fn grid_arrange_nonstretch_axis_measures_natural_extent() {
        // DD-M3-P5-005: a non-stretch axis measures the content at its
        // natural extent, not against the cell bound. An aspect Box is the
        // representative bound-dependent content: in a 40-wide × 100-tall
        // cell with h-align center (non-stretch) + v-align stretch, the
        // square (1:1) Box derives its natural size from the *stretched*
        // height (100) — width 100 — and overflows the narrow 40px column,
        // centred. Measuring the non-stretch width against the cell (40)
        // would instead shrink the Box to 40×40 (the pre-fix behaviour),
        // so this asserts the natural-extent measure.
        let mut g = LayoutNode::grid(
            vec![TrackSize::Fixed(40)],
            vec![TrackSize::Fixed(100)],
            vec![cell(0, 0, 1, 1, Alignment::Center, Alignment::Stretch)],
        );
        g.children
            .push(LayoutNode::box_(Some(Ratio { num: 1, den: 1 })));
        run_layout(&mut g, 40.0, 100.0).unwrap();
        // Natural square sized off the stretched height: 100×100.
        assert_eq!(g.children[0].size, (100.0, 100.0));
        // Centred horizontally in the 40px cell → x = (40 - 100) / 2 = -30
        // (overflow past both cell edges); top-anchored by stretch-v at y=0.
        assert_eq!(g.children[0].offset, (-30.0, 0.0));
    }

    #[test]
    fn grid_arrange_overflowing_cells_overlap_in_document_order() {
        // Layout-side substrate for the DD-M3-P5-005 document-order z-order:
        // two adjacent cells whose centred natural content overflows into the
        // neighbour produce overlapping rectangles, and the arrange loop
        // emits them in declared order (children[0] before children[1]).
        // The *paint precedence* (later child on top under overlap) is a
        // Visual-tree insertion-order property asserted by the T6 smoke
        // ("document-order paint order is observed when overlapping content
        // occurs"); T2 only proves the layout produces the overlapping
        // geometry in document order.
        let mut g = LayoutNode::grid(
            vec![TrackSize::Fixed(40), TrackSize::Fixed(40)],
            vec![TrackSize::Fixed(40)],
            vec![
                cell(0, 0, 1, 1, Alignment::Center, Alignment::Center),
                cell(0, 1, 1, 1, Alignment::Center, Alignment::Center),
            ],
        );
        // Two 60-wide natural-size rects, each wider than its 40px cell.
        g.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(60.0),
            SizeConstraint::Fixed(20.0),
        ));
        g.children.push(LayoutNode::rectangle(
            SizeConstraint::Fixed(60.0),
            SizeConstraint::Fixed(20.0),
        ));
        run_layout(&mut g, 80.0, 40.0).unwrap();
        // child[0] in column 0 cell (0..40), centred: x = (40-60)/2 = -10 → spans -10..50.
        // child[1] in column 1 cell (40..80), centred: x = 40 + (40-60)/2 = 30 → spans 30..70.
        let c0 = &g.children[0];
        let c1 = &g.children[1];
        assert_eq!(c0.offset.0, -10.0);
        assert_eq!(c1.offset.0, 30.0);
        // The two painted rectangles overlap horizontally (50 > 30).
        let c0_right = c0.offset.0 + c0.size.0;
        assert!(
            c0_right > c1.offset.0,
            "expected overflow overlap between cells"
        );
        // Document order is children-vector order (= sync_visuals paint order).
        assert_eq!(c0.size, (60.0, 20.0));
        assert_eq!(c1.size, (60.0, 20.0));
    }
}
