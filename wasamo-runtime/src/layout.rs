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
    /// `#[allow(dead_code)]` carries until T7's `measure_wrap_panel` /
    /// `arrange_wrap_panel` read this field (Phase 2 T6 used the same
    /// forward-pointer pattern on `aspect` until T8).
    #[allow(dead_code)]
    pub item_cross_size: Option<f32>,
    /// DD-M3-P3-003: WrapPanel-only gap (main-axis) between adjacent
    /// items on the same line. `0.0` on every other kind and on
    /// WrapPanel without `item-spacing` set (touching items).
    /// `#[allow(dead_code)]` carries until T7.
    #[allow(dead_code)]
    pub item_spacing: f32,
    /// DD-M3-P3-003: WrapPanel-only gap (cross-axis) between adjacent
    /// lines. `0.0` on every other kind and on WrapPanel without
    /// `line-spacing` set (touching lines).
    /// `#[allow(dead_code)]` carries until T7.
    #[allow(dead_code)]
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

// M3-Phase 3 T5 boundary placeholder. The DD-M3-P3-005 bounded /
// unbounded main-axis line-breaker lands in T7; until then the
// dispatch arm exists so the catalog wiring (variant + constructor +
// `build_layout_tree` arm) builds, but the WrapPanel reports `(0, 0)`
// extent and contributes no line geometry. Tests that exercise the
// actual line-breaker arrive in T7 alongside the real implementation.
fn measure_wrap_panel(
    _node: &LayoutNode,
    _avail_w: f32,
    _avail_h: f32,
) -> Result<(f32, f32), LayoutError> {
    Ok((0.0, 0.0))
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
        WidgetKind::WrapPanel => {
            // M3-Phase 3 T5 boundary placeholder. The DD-M3-P3-005 line
            // arrangement (per-line cross-axis stacking + spacing-aware
            // main-axis flow + unconditional first-child placement) is
            // T7's responsibility. For now we record the parent-allocated
            // cell as the WrapPanel's outer rectangle and leave children
            // un-arranged; T7 replaces this arm with the real arrange.
            node.offset = (x, y);
            node.size = (w, h);
            Ok(())
        }
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
}
