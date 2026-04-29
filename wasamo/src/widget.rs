use crate::layout::{self, Alignment, LayoutNode, SizeConstraint};
use crate::text::{TextRenderer, TypographyStyle};
use windows::{
    Foundation::{Numerics::{Vector2, Vector3}, TimeSpan},
    UI::{
        Color,
        Composition::{
            AnimationIterationBehavior,
            ColorKeyFrameAnimation,
            CompositionAnimation,
            CompositionColorBrush,
            CompositionObject,
            CompositionSurfaceBrush,
            Compositor,
            ContainerVisual,
            SpriteVisual,
            Visual,
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
    // Background brush retained for in-place color animation (DD-P5-005).
    bg_brush: CompositionColorBrush,
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
    Text { content: String, style: TypographyStyle },
    Button(Box<ButtonData>),
}

// ── Property dispatch (M1 experimental property IDs from wasamo.h §5) ─────────

pub const PROP_BUTTON_LABEL: u32 = 1;
pub const PROP_BUTTON_STYLE: u32 = 2;
pub const PROP_TEXT_CONTENT: u32 = 3;
pub const PROP_TEXT_STYLE: u32 = 4;

#[derive(Debug, Clone)]
pub enum PropertyValue {
    I32(i32),
    String(String),
}

#[derive(Debug)]
pub enum PropertyError {
    UnknownId,
    TypeMismatch,
    Runtime(String),
}

impl From<windows::core::Error> for PropertyError {
    fn from(e: windows::core::Error) -> Self {
        PropertyError::Runtime(format!("{e}"))
    }
}

fn button_style_to_i32(s: ButtonStyle) -> i32 {
    match s { ButtonStyle::Default => 0, ButtonStyle::Accent => 1 }
}

fn button_style_from_i32(v: i32) -> Option<ButtonStyle> {
    match v {
        0 => Some(ButtonStyle::Default),
        1 => Some(ButtonStyle::Accent),
        _ => None,
    }
}

fn typography_to_i32(s: TypographyStyle) -> i32 {
    match s {
        TypographyStyle::Caption  => 0,
        TypographyStyle::Body     => 1,
        TypographyStyle::Subtitle => 2,
        TypographyStyle::Title    => 3,
    }
}

fn typography_from_i32(v: i32) -> Option<TypographyStyle> {
    match v {
        0 => Some(TypographyStyle::Caption),
        1 => Some(TypographyStyle::Body),
        2 => Some(TypographyStyle::Subtitle),
        3 => Some(TypographyStyle::Title),
        _ => None,
    }
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
            data: WidgetData::Text { content: text.to_owned(), style },
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
        let initial_color = button_state_color(style, ButtonState::Normal, accent);
        let bg_brush = compositor.CreateColorBrushWithColor(initial_color)?;
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
            bg_brush,
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

    // ── Property R/W (wasamo.h §4.3 + §5 experimental property IDs) ───────────
    //
    // Dispatch is enum-on-`WidgetData`: each variant accepts only the IDs that
    // belong to it; everything else returns `UnknownId`. Types that do not
    // match the property's declared type return `TypeMismatch`.

    pub fn get_property(&self, id: u32) -> Result<PropertyValue, PropertyError> {
        match (&self.data, id) {
            (WidgetData::Button(btn), PROP_BUTTON_LABEL) => {
                Ok(PropertyValue::String(btn.label_text.clone()))
            }
            (WidgetData::Button(btn), PROP_BUTTON_STYLE) => {
                Ok(PropertyValue::I32(button_style_to_i32(btn.style)))
            }
            (WidgetData::Text { content, .. }, PROP_TEXT_CONTENT) => {
                Ok(PropertyValue::String(content.clone()))
            }
            (WidgetData::Text { style, .. }, PROP_TEXT_STYLE) => {
                Ok(PropertyValue::I32(typography_to_i32(*style)))
            }
            _ => Err(PropertyError::UnknownId),
        }
    }

    pub fn set_property(
        &mut self,
        id: u32,
        value: &PropertyValue,
    ) -> Result<(), PropertyError> {
        match (&mut self.data, id) {
            (WidgetData::Button(_), PROP_BUTTON_LABEL) => {
                let s = match value {
                    PropertyValue::String(s) => s.clone(),
                    _ => return Err(PropertyError::TypeMismatch),
                };
                self.update_button_label(&s)
            }
            (WidgetData::Button(_), PROP_BUTTON_STYLE) => {
                let v = match value {
                    PropertyValue::I32(v) => *v,
                    _ => return Err(PropertyError::TypeMismatch),
                };
                let new_style =
                    button_style_from_i32(v).ok_or(PropertyError::TypeMismatch)?;
                self.update_button_style(new_style)
            }
            (WidgetData::Text { .. }, PROP_TEXT_CONTENT) => {
                let s = match value {
                    PropertyValue::String(s) => s.clone(),
                    _ => return Err(PropertyError::TypeMismatch),
                };
                self.update_text_content(&s)
            }
            (WidgetData::Text { .. }, PROP_TEXT_STYLE) => {
                let v = match value {
                    PropertyValue::I32(v) => *v,
                    _ => return Err(PropertyError::TypeMismatch),
                };
                let new_style =
                    typography_from_i32(v).ok_or(PropertyError::TypeMismatch)?;
                self.update_text_style(new_style)
            }
            _ => Err(PropertyError::UnknownId),
        }
    }

    fn update_button_label(&mut self, new_label: &str) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let renderer = &rt.text_renderer;

        let WidgetData::Button(ref mut btn) = self.data else {
            return Err(PropertyError::UnknownId);
        };
        let label_style = btn.label_style;
        let (lw, lh) = renderer.measure(new_label, label_style)?;
        let surface = renderer.draw_text(
            new_label,
            label_style,
            lw.max(1.0),
            lh.max(1.0),
            Color { A: 255, R: 255, G: 255, B: 255 },
        )?;
        let label_brush: CompositionSurfaceBrush =
            compositor.CreateSurfaceBrushWithSurface(&surface)?;
        btn.label_visual.SetBrush(&label_brush)?;

        use windows::core::Interface;
        const PAD_H: f32 = 16.0;
        const PAD_V: f32 = 8.0;
        let label_vis: Visual = btn.label_visual.cast()?;
        label_vis.SetOffset(Vector3 { X: PAD_H, Y: PAD_V, Z: 0.0 })?;
        label_vis.SetSize(Vector2 { X: lw, Y: lh })?;

        btn.label_text = new_label.to_owned();
        // Natural size updates; takes effect on the next layout pass.
        self.width = SizeConstraint::Fixed(lw + PAD_H * 2.0);
        self.height = SizeConstraint::Fixed(lh + PAD_V * 2.0);
        Ok(())
    }

    fn update_button_style(&mut self, new_style: ButtonStyle) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let WidgetData::Button(ref mut btn) = self.data else {
            return Err(PropertyError::UnknownId);
        };
        if btn.style == new_style {
            return Ok(());
        }
        btn.style = new_style;
        let target = button_state_color(btn.style, btn.state, btn.accent);
        let new_brush = compositor.CreateColorBrushWithColor(target)?;
        self.visual.SetBrush(&new_brush)?;
        btn.bg_brush = new_brush;
        Ok(())
    }

    fn update_text_content(&mut self, new_content: &str) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let renderer = &rt.text_renderer;

        let WidgetData::Text { ref mut content, style } = self.data else {
            return Err(PropertyError::UnknownId);
        };
        let style = style;
        let (w, h) = renderer.measure(new_content, style)?;
        let surface = renderer.draw_text(
            new_content,
            style,
            w.max(1.0),
            h.max(1.0),
            Color { A: 255, R: 255, G: 255, B: 255 },
        )?;
        let brush: CompositionSurfaceBrush =
            compositor.CreateSurfaceBrushWithSurface(&surface)?;
        self.visual.SetBrush(&brush)?;

        *content = new_content.to_owned();
        self.width = SizeConstraint::Fixed(w);
        self.height = SizeConstraint::Fixed(h);
        Ok(())
    }

    fn update_text_style(&mut self, new_style: TypographyStyle) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let renderer = &rt.text_renderer;

        let WidgetData::Text { ref mut content, ref mut style } = self.data else {
            return Err(PropertyError::UnknownId);
        };
        if *style == new_style {
            return Ok(());
        }
        *style = new_style;
        let (w, h) = renderer.measure(content, new_style)?;
        let surface = renderer.draw_text(
            content,
            new_style,
            w.max(1.0),
            h.max(1.0),
            Color { A: 255, R: 255, G: 255, B: 255 },
        )?;
        let brush: CompositionSurfaceBrush =
            compositor.CreateSurfaceBrushWithSurface(&surface)?;
        self.visual.SetBrush(&brush)?;

        self.width = SizeConstraint::Fixed(w);
        self.height = SizeConstraint::Fixed(h);
        Ok(())
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

        // We need a stable pointer to `self` for the registry signal lookup
        // before we re-borrow `self.data` mutably below.
        let widget_ptr: *mut WidgetNode = self as *mut WidgetNode;

        if let WidgetData::Button(ref mut btn) = self.data {
            let fx = x as f32;
            let fy = y as f32;
            if fx >= abs_x && fx < abs_x + vw && fy >= abs_y && fy < abs_y + vh {
                if let Some(ref f) = btn.clicked_fn {
                    f();
                }
                // Route "clicked" through the C-ABI signal registry. The
                // emission is queued and fires after the current call
                // returns to wasamo_run's message-loop drain (abi_spec §6).
                crate::emit::enqueue_signal(widget_ptr, "clicked", Vec::new());
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
                let old_state = btn.state;
                btn.state = new_state;
                let target = button_state_color(btn.style, new_state, btn.accent);
                let ticks = transition_duration(old_state, new_state);
                start_color_anim(compositor, &btn.bg_brush, target, ticks)?;
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
                let target = button_state_color(btn.style, ButtonState::Normal, btn.accent);
                start_color_anim(compositor, &btn.bg_brush, target, 1_670_000)?;
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
            WidgetData::Rectangle | WidgetData::Text { .. } | WidgetData::Button(_) => {
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

fn button_state_color(style: ButtonStyle, state: ButtonState, accent: Color) -> Color {
    match (style, state) {
        (ButtonStyle::Default, ButtonState::Normal)  => Color { A: 0x20, R: 0xFF, G: 0xFF, B: 0xFF },
        (ButtonStyle::Default, ButtonState::Hovered) => Color { A: 0x33, R: 0xFF, G: 0xFF, B: 0xFF },
        (ButtonStyle::Default, ButtonState::Pressed) => Color { A: 0x10, R: 0xFF, G: 0xFF, B: 0xFF },
        (ButtonStyle::Accent,  ButtonState::Normal)  => accent,
        (ButtonStyle::Accent,  ButtonState::Hovered) => lighten(accent, 26),
        (ButtonStyle::Accent,  ButtonState::Pressed) => darken(accent, 26),
    }
}

// Duration in 100-ns ticks: fast (83 ms) for entering active state, slow (167 ms) for leaving.
fn transition_duration(old: ButtonState, new: ButtonState) -> i64 {
    match (old, new) {
        (_, ButtonState::Pressed)                         => 830_000,   // press-down: fast
        (ButtonState::Pressed, _)                         => 1_670_000, // press-up: slow
        (ButtonState::Normal,  ButtonState::Hovered)      => 830_000,   // hover-in: fast
        _                                                 => 1_670_000, // hover-out: slow
    }
}

fn start_color_anim(
    compositor: &Compositor,
    brush: &CompositionColorBrush,
    target: Color,
    duration_ticks: i64,
) -> windows::core::Result<()> {
    use windows::core::{Interface, HSTRING};
    let anim: ColorKeyFrameAnimation = compositor.CreateColorKeyFrameAnimation()?;
    anim.InsertKeyFrame(1.0_f32, target)?;
    anim.SetDuration(TimeSpan { Duration: duration_ticks })?;
    anim.SetIterationBehavior(AnimationIterationBehavior::Count)?;
    anim.SetIterationCount(1)?;
    let comp_anim: CompositionAnimation = anim.cast()?;
    let obj: CompositionObject = brush.cast()?;
    obj.StartAnimation(&HSTRING::from("Color"), &comp_anim)?;
    Ok(())
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
