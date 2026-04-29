use crate::layout::{self, Alignment, LayoutNode, SizeConstraint};
use crate::text::{TextRenderer, TypographyStyle};
use windows::{
    Foundation::Numerics::{Vector2, Vector3},
    UI::{
        Color,
        Composition::{
            CompositionColorBrush, CompositionSurfaceBrush, Compositor, ContainerVisual,
            SpriteVisual, Visual,
        },
    },
};

// ── Button state ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonStyle { Default, Accent }

#[derive(Clone, Copy, PartialEq, Debug)]
enum ButtonState { Normal, Hovered, Pressed }

// label_visual/text/style are retained for future set_label() support.
#[allow(dead_code)]
struct ButtonData {
    style: ButtonStyle,
    state: ButtonState,
    label_visual: SpriteVisual,
    label_text: String,
    label_style: TypographyStyle,
    clicked_fn: Option<Box<dyn Fn()>>,
    // Accent color for ButtonStyle::Accent (read from UISettings at creation).
    accent: Color,
}

// ── Widget kinds ──────────────────────────────────────────────────────────────

enum WidgetData {
    Rectangle,
    VStack { spacing: f32, padding: f32, alignment: Alignment },
    HStack { spacing: f32, padding: f32, alignment: Alignment },
    Text,
    Button(Box<ButtonData>),
}

// ── WidgetNode ────────────────────────────────────────────────────────────────

pub struct WidgetNode {
    data: WidgetData,
    width: SizeConstraint,
    height: SizeConstraint,
    pub visual: SpriteVisual,
    pub children: Vec<Box<WidgetNode>>,
}

impl WidgetNode {
    // ── Constructors ──────────────────────────────────────────────────────────

    pub fn rectangle(
        compositor: &Compositor,
        width: SizeConstraint,
        height: SizeConstraint,
    ) -> windows::core::Result<Box<Self>> {
        let visual = compositor.CreateSpriteVisual()?;
        Ok(Box::new(Self {
            data: WidgetData::Rectangle,
            width,
            height,
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
            data: WidgetData::VStack { spacing, padding, alignment },
            width: SizeConstraint::Fill,
            height: SizeConstraint::Shrink,
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
            data: WidgetData::HStack { spacing, padding, alignment },
            width: SizeConstraint::Shrink,
            height: SizeConstraint::Fill,
            visual,
            children: Vec::new(),
        }))
    }

    pub fn text(
        compositor: &Compositor,
        renderer: &TextRenderer,
        text: &str,
        style: TypographyStyle,
    ) -> windows::core::Result<Box<Self>> {
        let (w, h) = renderer.measure(text, style)?;
        let visual = compositor.CreateSpriteVisual()?;
        // Draw text onto a surface and apply it as a surface brush.
        let surface = renderer.draw_text(
            text,
            style,
            w.max(1.0),
            h.max(1.0),
            Color { A: 255, R: 255, G: 255, B: 255 },
        )?;
        let brush: CompositionSurfaceBrush = compositor.CreateSurfaceBrushWithSurface(&surface)?;
        visual.SetBrush(&brush)?;
        Ok(Box::new(Self {
            data: WidgetData::Text,
            width: SizeConstraint::Fixed(w),
            height: SizeConstraint::Fixed(h),
            visual,
            children: Vec::new(),
        }))
    }

    pub fn button(
        compositor: &Compositor,
        renderer: &TextRenderer,
        label: &str,
        style: ButtonStyle,
    ) -> windows::core::Result<Box<Self>> {
        let label_style = TypographyStyle::Body;
        let (lw, lh) = renderer.measure(label, label_style)?;

        // Standard button sizing: label + horizontal/vertical padding.
        const PAD_H: f32 = 16.0;
        const PAD_V: f32 = 8.0;
        let btn_w = lw + PAD_H * 2.0;
        let btn_h = lh + PAD_V * 2.0;

        let accent = read_accent_color();

        // Root visual: background.
        let bg_visual = compositor.CreateSpriteVisual()?;
        let bg_brush = make_button_brush(compositor, style, ButtonState::Normal, accent)?;
        bg_visual.SetBrush(&bg_brush)?;

        // Child visual: text label.
        let label_visual = compositor.CreateSpriteVisual()?;
        let surface = renderer.draw_text(
            label,
            label_style,
            lw.max(1.0),
            lh.max(1.0),
            Color { A: 255, R: 255, G: 255, B: 255 },
        )?;
        let label_brush: CompositionSurfaceBrush =
            compositor.CreateSurfaceBrushWithSurface(&surface)?;
        label_visual.SetBrush(&label_brush)?;

        // Position label centered in the button.
        use windows::core::Interface;
        let label_vis: Visual = label_visual.cast()?;
        label_vis.SetOffset(Vector3 { X: PAD_H, Y: PAD_V, Z: 0.0 })?;
        label_vis.SetSize(Vector2 { X: lw, Y: lh })?;
        let bg_container: ContainerVisual = bg_visual.cast()?;
        bg_container.Children()?.InsertAtTop(&label_vis)?;

        let btn_data = Box::new(ButtonData {
            style,
            state: ButtonState::Normal,
            label_visual,
            label_text: label.to_owned(),
            label_style,
            clicked_fn: None,
            accent,
        });

        Ok(Box::new(Self {
            data: WidgetData::Button(btn_data),
            width: SizeConstraint::Fixed(btn_w),
            height: SizeConstraint::Fixed(btn_h),
            visual: bg_visual,
            children: Vec::new(),
        }))
    }

    // ── Property setters ──────────────────────────────────────────────────────

    pub fn set_color(
        &self,
        compositor: &Compositor,
        r: u8, g: u8, b: u8,
    ) -> windows::core::Result<()> {
        let brush =
            compositor.CreateColorBrushWithColor(Color { A: 255, R: r, G: g, B: b })?;
        self.visual.SetBrush(&brush)?;
        Ok(())
    }

    /// Register a callback invoked when this Button is clicked.
    /// Panics if called on a non-Button widget.
    pub fn set_clicked<F: Fn() + 'static>(&mut self, f: F) {
        if let WidgetData::Button(ref mut btn) = self.data {
            btn.clicked_fn = Some(Box::new(f));
        }
    }

    // ── Hit testing ───────────────────────────────────────────────────────────

    /// Traverse the tree and fire the `clicked_fn` of the first Button whose
    /// computed visual rect contains `(x, y)` in window client coordinates.
    pub fn hit_test_click(&mut self, x: i32, y: i32) {
        self.hit_test_click_inner(x, y, 0.0, 0.0);
    }

    fn hit_test_click_inner(&mut self, x: i32, y: i32, off_x: f32, off_y: f32) {
        // The visual's current offset is available via computed layout stored on the Visual.
        // We read it back from the SpriteVisual to avoid tracking a separate state.
        let (vx, vy, vw, vh) = visual_rect(&self.visual);
        let abs_x = off_x + vx;
        let abs_y = off_y + vy;

        if let WidgetData::Button(ref mut btn) = self.data {
            let fx = x as f32;
            let fy = y as f32;
            if fx >= abs_x && fx < abs_x + vw && fy >= abs_y && fy < abs_y + vh {
                if let Some(ref f) = btn.clicked_fn {
                    f();
                }
                return;
            }
        }

        for child in &mut self.children {
            child.hit_test_click_inner(x, y, abs_x, abs_y);
        }
    }

    /// Update hover/press state for all Buttons based on mouse position.
    /// `down` is true while the left mouse button is held.
    pub fn update_hover(
        &mut self,
        compositor: &Compositor,
        x: i32,
        y: i32,
        down: bool,
    ) -> windows::core::Result<()> {
        self.update_hover_inner(compositor, x, y, down, 0.0, 0.0)
    }

    fn update_hover_inner(
        &mut self,
        compositor: &Compositor,
        x: i32,
        y: i32,
        down: bool,
        off_x: f32,
        off_y: f32,
    ) -> windows::core::Result<()> {
        let (vx, vy, vw, vh) = visual_rect(&self.visual);
        let abs_x = off_x + vx;
        let abs_y = off_y + vy;

        if let WidgetData::Button(ref mut btn) = self.data {
            let fx = x as f32;
            let fy = y as f32;
            let inside = fx >= abs_x && fx < abs_x + vw && fy >= abs_y && fy < abs_y + vh;
            let new_state = if inside && down {
                ButtonState::Pressed
            } else if inside {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            };
            if new_state != btn.state {
                btn.state = new_state;
                let brush =
                    make_button_brush(compositor, btn.style, new_state, btn.accent)?;
                self.visual.SetBrush(&brush)?;
            }
        }

        for child in &mut self.children {
            child.update_hover_inner(compositor, x, y, down, abs_x, abs_y)?;
        }
        Ok(())
    }

    /// Reset all Button states to Normal (called on WM_MOUSELEAVE).
    pub fn clear_hover(&mut self, compositor: &Compositor) -> windows::core::Result<()> {
        if let WidgetData::Button(ref mut btn) = self.data {
            if btn.state != ButtonState::Normal {
                btn.state = ButtonState::Normal;
                let brush =
                    make_button_brush(compositor, btn.style, ButtonState::Normal, btn.accent)?;
                self.visual.SetBrush(&brush)?;
            }
        }
        for child in &mut self.children {
            child.clear_hover(compositor)?;
        }
        Ok(())
    }

    // ── Tree building ─────────────────────────────────────────────────────────

    pub fn append_child(&mut self, child: Box<WidgetNode>) -> windows::core::Result<()> {
        use windows::core::Interface;
        let parent_container: ContainerVisual = self.visual.cast()?;
        let child_visual: Visual = child.visual.cast()?;
        parent_container.Children()?.InsertAtTop(&child_visual)?;
        self.children.push(child);
        Ok(())
    }

    // ── Layout ────────────────────────────────────────────────────────────────

    /// Builds a LayoutNode tree, runs layout, then syncs results back to SpriteVisuals.
    pub fn run_layout(&mut self, window_w: f32, window_h: f32) -> windows::core::Result<()> {
        let mut layout_tree = self.build_layout_tree();
        layout::run_layout(&mut layout_tree, window_w, window_h);
        self.sync_visuals(&layout_tree)
    }

    fn build_layout_tree(&self) -> LayoutNode {
        match &self.data {
            WidgetData::Rectangle | WidgetData::Text | WidgetData::Button(_) => {
                LayoutNode::rectangle(self.width.clone(), self.height.clone())
            }
            WidgetData::VStack { spacing, padding, alignment } => {
                let mut node =
                    LayoutNode::vstack(*spacing, *padding, *alignment);
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children =
                    self.children.iter().map(|c| c.build_layout_tree()).collect();
                node
            }
            WidgetData::HStack { spacing, padding, alignment } => {
                let mut node =
                    LayoutNode::hstack(*spacing, *padding, *alignment);
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children =
                    self.children.iter().map(|c| c.build_layout_tree()).collect();
                node
            }
        }
    }

    fn sync_visuals(&mut self, computed: &LayoutNode) -> windows::core::Result<()> {
        use windows::core::Interface;
        let visual: Visual = self.visual.cast()?;
        visual.SetOffset(Vector3 {
            X: computed.offset.0,
            Y: computed.offset.1,
            Z: 0.0,
        })?;
        visual.SetSize(Vector2 {
            X: computed.size.0,
            Y: computed.size.1,
        })?;
        // For Button, also resize the root SpriteVisual (already done above)
        // and keep the label visual's size/offset constant (set at creation).
        for (child, child_computed) in
            self.children.iter_mut().zip(computed.children.iter())
        {
            child.sync_visuals(child_computed)?;
        }
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn visual_rect(v: &SpriteVisual) -> (f32, f32, f32, f32) {
    use windows::core::Interface;
    let vis: Visual = v.cast().unwrap_or_else(|_| panic!("cast failed"));
    let off = vis.Offset().unwrap_or(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 });
    let sz = vis.Size().unwrap_or(Vector2 { X: 0.0, Y: 0.0 });
    (off.X, off.Y, sz.X, sz.Y)
}

fn make_button_brush(
    compositor: &Compositor,
    style: ButtonStyle,
    state: ButtonState,
    accent: Color,
) -> windows::core::Result<CompositionColorBrush> {
    let color = match (style, state) {
        (ButtonStyle::Default, ButtonState::Normal)  => Color { A: 0x20, R: 0xFF, G: 0xFF, B: 0xFF },
        (ButtonStyle::Default, ButtonState::Hovered) => Color { A: 0x33, R: 0xFF, G: 0xFF, B: 0xFF },
        (ButtonStyle::Default, ButtonState::Pressed) => Color { A: 0x10, R: 0xFF, G: 0xFF, B: 0xFF },
        (ButtonStyle::Accent,  ButtonState::Normal)  => accent,
        (ButtonStyle::Accent,  ButtonState::Hovered) => lighten(accent, 26),
        (ButtonStyle::Accent,  ButtonState::Pressed) => darken(accent, 26),
    };
    compositor.CreateColorBrushWithColor(color)
}

fn lighten(c: Color, amount: u8) -> Color {
    Color {
        A: c.A,
        R: c.R.saturating_add(amount),
        G: c.G.saturating_add(amount),
        B: c.B.saturating_add(amount),
    }
}

fn darken(c: Color, amount: u8) -> Color {
    Color {
        A: c.A,
        R: c.R.saturating_sub(amount),
        G: c.G.saturating_sub(amount),
        B: c.B.saturating_sub(amount),
    }
}

fn read_accent_color() -> Color {
    use windows::UI::ViewManagement::{UIColorType, UISettings};
    UISettings::new()
        .and_then(|s| s.GetColorValue(UIColorType::Accent))
        .unwrap_or(Color { A: 255, R: 0, G: 120, B: 215 }) // Windows default blue
}
