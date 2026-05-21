// Pure layout engine — no Win32/WinRT dependencies; all logic here is unit-testable.

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    BoxAspectUnboundedBoth,
    BoxNoExtent,
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
    pub children: Vec<LayoutNode>,
    // Written by arrange():
    pub offset: (f32, f32),
    pub size: (f32, f32),
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
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
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
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
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
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
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
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
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
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
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
    let lines = compute_wrap_lines(node, avail_w, avail_h)?;
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
/// DD-M3-P3-001 unbounded-main + DD-M3-P3-004 cross constraint, then
/// applies the DD-M3-P3-005 greedy line breaker:
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
    cross_bound: f32,
) -> Result<Vec<WrapLine>, LayoutError> {
    let child_cross_input = node.item_cross_size.unwrap_or(cross_bound);

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

    let lines = compute_wrap_lines(node, w, h)?;
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
}
