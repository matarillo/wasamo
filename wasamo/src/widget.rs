use crate::layout::{self, Alignment, LayoutNode, SizeConstraint, WidgetKind};
use windows::{
    Foundation::Numerics::{Vector2, Vector3},
    UI::{
        Color,
        Composition::{Compositor, ContainerVisual, SpriteVisual, Visual},
    },
};

pub struct WidgetNode {
    kind: WidgetKind,
    width: SizeConstraint,
    height: SizeConstraint,
    spacing: f32,
    padding: f32,
    alignment: Alignment,
    pub visual: SpriteVisual,
    pub children: Vec<Box<WidgetNode>>,
}

impl WidgetNode {
    pub fn rectangle(
        compositor: &Compositor,
        width: SizeConstraint,
        height: SizeConstraint,
    ) -> windows::core::Result<Box<Self>> {
        let visual = compositor.CreateSpriteVisual()?;
        Ok(Box::new(Self {
            kind: WidgetKind::Rectangle,
            width,
            height,
            spacing: 0.0,
            padding: 0.0,
            alignment: Alignment::Stretch,
            visual,
            children: Vec::new(),
        }))
    }

    pub fn vstack(
        compositor: &Compositor,
        spacing: f32,
        padding: f32,
        alignment: Alignment,
    ) -> windows::core::Result<Box<Self>> {
        let visual = compositor.CreateSpriteVisual()?;
        Ok(Box::new(Self {
            kind: WidgetKind::VStack,
            width: SizeConstraint::Fill,
            height: SizeConstraint::Shrink,
            spacing,
            padding,
            alignment,
            visual,
            children: Vec::new(),
        }))
    }

    pub fn hstack(
        compositor: &Compositor,
        spacing: f32,
        padding: f32,
        alignment: Alignment,
    ) -> windows::core::Result<Box<Self>> {
        let visual = compositor.CreateSpriteVisual()?;
        Ok(Box::new(Self {
            kind: WidgetKind::HStack,
            width: SizeConstraint::Shrink,
            height: SizeConstraint::Fill,
            spacing,
            padding,
            alignment,
            visual,
            children: Vec::new(),
        }))
    }

    pub fn set_color(&self, compositor: &Compositor, r: u8, g: u8, b: u8) -> windows::core::Result<()> {
        let brush = compositor.CreateColorBrushWithColor(Color { A: 255, R: r, G: g, B: b })?;
        self.visual.SetBrush(&brush)?;
        Ok(())
    }

    pub fn append_child(&mut self, child: Box<WidgetNode>) -> windows::core::Result<()> {
        use windows::core::Interface;
        let parent_container: ContainerVisual = self.visual.cast()?;
        let child_visual: Visual = child.visual.cast()?;
        parent_container.Children()?.InsertAtTop(&child_visual)?;
        self.children.push(child);
        Ok(())
    }

    /// Builds a LayoutNode tree, runs layout, then syncs results back to SpriteVisuals.
    pub fn run_layout(&mut self, window_w: f32, window_h: f32) -> windows::core::Result<()> {
        let mut layout_tree = self.build_layout_tree();
        layout::run_layout(&mut layout_tree, window_w, window_h);
        self.sync_visuals(&layout_tree)
    }

    fn build_layout_tree(&self) -> LayoutNode {
        match self.kind {
            WidgetKind::Rectangle => LayoutNode::rectangle(self.width.clone(), self.height.clone()),
            WidgetKind::VStack => {
                let mut node = LayoutNode::vstack(self.spacing, self.padding, self.alignment);
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self.children.iter().map(|c| c.build_layout_tree()).collect();
                node
            }
            WidgetKind::HStack => {
                let mut node = LayoutNode::hstack(self.spacing, self.padding, self.alignment);
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self.children.iter().map(|c| c.build_layout_tree()).collect();
                node
            }
        }
    }

    fn sync_visuals(&mut self, computed: &LayoutNode) -> windows::core::Result<()> {
        use windows::core::Interface;
        let visual: Visual = self.visual.cast()?;
        visual.SetOffset(Vector3 { X: computed.offset.0, Y: computed.offset.1, Z: 0.0 })?;
        visual.SetSize(Vector2 { X: computed.size.0, Y: computed.size.1 })?;
        for (child, child_computed) in self.children.iter_mut().zip(computed.children.iter()) {
            child.sync_visuals(child_computed)?;
        }
        Ok(())
    }
}
