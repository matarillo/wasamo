// Pure layout engine — no Win32/WinRT dependencies; all logic here is unit-testable.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetKind {
    Rectangle,
    VStack,
    HStack,
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

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub kind: WidgetKind,
    pub width: SizeConstraint,
    pub height: SizeConstraint,
    pub spacing: f32,
    pub padding: f32,
    pub alignment: Alignment,
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
            children: Vec::new(),
            offset: (0.0, 0.0),
            size: (0.0, 0.0),
        }
    }
}

/// Returns the desired (width, height) of a node given available space.
/// Pass f32::INFINITY for unconstrained axes.
/// Fill children return 0.0 — they take whatever the parent allocates in arrange().
pub fn measure(node: &LayoutNode, avail_w: f32, avail_h: f32) -> (f32, f32) {
    match node.kind {
        WidgetKind::Rectangle => measure_leaf(node),
        WidgetKind::VStack => measure_vstack(node, avail_w),
        WidgetKind::HStack => measure_hstack(node, avail_h),
    }
}

fn measure_leaf(node: &LayoutNode) -> (f32, f32) {
    let w = if let SizeConstraint::Fixed(v) = node.width { v } else { 0.0 };
    let h = if let SizeConstraint::Fixed(v) = node.height { v } else { 0.0 };
    (w, h)
}

fn measure_vstack(node: &LayoutNode, avail_w: f32) -> (f32, f32) {
    let inner_w = (avail_w - 2.0 * node.padding).max(0.0);
    let child_desired: Vec<(f32, f32)> = node.children.iter()
        .map(|c| measure(c, inner_w, f32::INFINITY))
        .collect();

    let n = node.children.len();
    let spacing_total = if n > 0 { node.spacing * (n as f32 - 1.0) } else { 0.0 };

    let desired_w = match &node.width {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => {
            let max_cw = child_desired.iter().map(|&(w, _)| w).fold(0.0_f32, f32::max);
            max_cw + 2.0 * node.padding
        }
    };

    let non_fill_h: f32 = node.children.iter()
        .zip(child_desired.iter())
        .filter(|(c, _)| c.height != SizeConstraint::Fill)
        .map(|(_, &(_, h))| h)
        .sum();

    let desired_h = match &node.height {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => non_fill_h + spacing_total + 2.0 * node.padding,
    };

    (desired_w, desired_h)
}

fn measure_hstack(node: &LayoutNode, avail_h: f32) -> (f32, f32) {
    let inner_h = (avail_h - 2.0 * node.padding).max(0.0);
    let child_desired: Vec<(f32, f32)> = node.children.iter()
        .map(|c| measure(c, f32::INFINITY, inner_h))
        .collect();

    let n = node.children.len();
    let spacing_total = if n > 0 { node.spacing * (n as f32 - 1.0) } else { 0.0 };

    let desired_h = match &node.height {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => {
            let max_ch = child_desired.iter().map(|&(_, h)| h).fold(0.0_f32, f32::max);
            max_ch + 2.0 * node.padding
        }
    };

    let non_fill_w: f32 = node.children.iter()
        .zip(child_desired.iter())
        .filter(|(c, _)| c.width != SizeConstraint::Fill)
        .map(|(_, &(w, _))| w)
        .sum();

    let desired_w = match &node.width {
        SizeConstraint::Fixed(v) => *v,
        SizeConstraint::Fill => 0.0,
        SizeConstraint::Shrink => non_fill_w + spacing_total + 2.0 * node.padding,
    };

    (desired_w, desired_h)
}

/// Assigns final offset and size, recurses into children.
pub fn arrange(node: &mut LayoutNode, x: f32, y: f32, w: f32, h: f32) {
    node.offset = (x, y);
    node.size = (w, h);

    let kind = node.kind;
    let padding = node.padding;
    let spacing = node.spacing;
    let alignment = node.alignment;

    match kind {
        WidgetKind::Rectangle => {}
        WidgetKind::VStack => arrange_vstack(&mut node.children, x, y, w, h, padding, spacing, alignment),
        WidgetKind::HStack => arrange_hstack(&mut node.children, x, y, w, h, padding, spacing, alignment),
    }
}

fn arrange_vstack(
    children: &mut [LayoutNode],
    x: f32, y: f32, w: f32, h: f32,
    padding: f32, spacing: f32, alignment: Alignment,
) {
    let inner_x = x + padding;
    let inner_y = y + padding;
    let inner_w = (w - 2.0 * padding).max(0.0);
    let inner_h = (h - 2.0 * padding).max(0.0);

    let child_desired: Vec<(f32, f32)> = children.iter()
        .map(|c| measure(c, inner_w, f32::INFINITY))
        .collect();

    let n = children.len();
    let fill_count = children.iter().filter(|c| c.height == SizeConstraint::Fill).count();
    let non_fill_h: f32 = children.iter()
        .zip(child_desired.iter())
        .filter(|(c, _)| c.height != SizeConstraint::Fill)
        .map(|(_, &(_, h))| h)
        .sum();
    let spacing_total = if n > 0 { spacing * (n as f32 - 1.0) } else { 0.0 };
    let remaining = (inner_h - non_fill_h - spacing_total).max(0.0);
    let fill_child_h = if fill_count > 0 { remaining / fill_count as f32 } else { 0.0 };

    let mut cur_y = inner_y;
    for (i, child) in children.iter_mut().enumerate() {
        let (desired_w, desired_h) = child_desired[i];

        let child_h = if child.height == SizeConstraint::Fill { fill_child_h } else { desired_h };

        let (child_x, child_w) = cross_axis_position(
            &child.width, desired_w, inner_x, inner_w, alignment,
        );

        arrange(child, child_x, cur_y, child_w, child_h);
        cur_y += child_h;
        if i < n - 1 {
            cur_y += spacing;
        }
    }
}

fn arrange_hstack(
    children: &mut [LayoutNode],
    x: f32, y: f32, w: f32, h: f32,
    padding: f32, spacing: f32, alignment: Alignment,
) {
    let inner_x = x + padding;
    let inner_y = y + padding;
    let inner_w = (w - 2.0 * padding).max(0.0);
    let inner_h = (h - 2.0 * padding).max(0.0);

    let child_desired: Vec<(f32, f32)> = children.iter()
        .map(|c| measure(c, f32::INFINITY, inner_h))
        .collect();

    let n = children.len();
    let fill_count = children.iter().filter(|c| c.width == SizeConstraint::Fill).count();
    let non_fill_w: f32 = children.iter()
        .zip(child_desired.iter())
        .filter(|(c, _)| c.width != SizeConstraint::Fill)
        .map(|(_, &(w, _))| w)
        .sum();
    let spacing_total = if n > 0 { spacing * (n as f32 - 1.0) } else { 0.0 };
    let remaining = (inner_w - non_fill_w - spacing_total).max(0.0);
    let fill_child_w = if fill_count > 0 { remaining / fill_count as f32 } else { 0.0 };

    let mut cur_x = inner_x;
    for (i, child) in children.iter_mut().enumerate() {
        let (desired_w, desired_h) = child_desired[i];

        let child_w = if child.width == SizeConstraint::Fill { fill_child_w } else { desired_w };

        let (child_y, child_h) = cross_axis_position(
            &child.height, desired_h, inner_y, inner_h, alignment,
        );

        arrange(child, cur_x, child_y, child_w, child_h);
        cur_x += child_w;
        if i < n - 1 {
            cur_x += spacing;
        }
    }
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
pub fn run_layout(root: &mut LayoutNode, window_w: f32, window_h: f32) {
    let (desired_w, desired_h) = measure(root, window_w, window_h);
    let final_w = resolve_axis(&root.width, desired_w, window_w);
    let final_h = resolve_axis(&root.height, desired_h, window_h);
    arrange(root, 0.0, 0.0, final_w, final_h);
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
        let mut rect = LayoutNode::rectangle(SizeConstraint::Fixed(100.0), SizeConstraint::Fixed(50.0));
        arrange(&mut rect, 10.0, 20.0, 100.0, 50.0);
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

        run_layout(&mut stack, 400.0, 600.0);

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

        run_layout(&mut stack, 200.0, 600.0);

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

        run_layout(&mut stack, 200.0, 200.0);

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

        run_layout(&mut stack, 600.0, 200.0);

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

        run_layout(&mut stack, 200.0, 600.0);

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

        run_layout(&mut stack, 200.0, 600.0);

        assert_eq!(stack.children[0].size.1, 0.0);
    }
}
