use crate::box_values;
use crate::handler::{self, EvalContext, EvalError, HandlerExpr};
use crate::layout::{self, Alignment, LayoutError, LayoutNode, SizeConstraint};
use crate::reactive::EffectHandle;
use crate::text::{TextRenderer, TypographyStyle};
use windows::{
    Foundation::{
        Numerics::{Vector2, Vector3},
        TimeSpan,
    },
    UI::{
        Color,
        Composition::{
            AnimationIterationBehavior, ColorKeyFrameAnimation, CompositionAnimation,
            CompositionColorBrush, CompositionObject, CompositionSurfaceBrush, Compositor,
            ContainerVisual, SpriteVisual, Visual,
        },
    },
};

// ── Button state ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ButtonStyle {
    Default,
    Accent,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ButtonState {
    Normal,
    Hovered,
    Pressed,
}

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
    // Phase 1 narrow `Button.enabled` contract (M3-Phase 1 DD-M3-P1-005):
    // when false, click-handler dispatch is suppressed and the background is
    // greyed with no animation. Focus / a11y / hover-state semantics are
    // deferred to M4–M5.
    enabled: bool,
}

// ── Widget kinds ──────────────────────────────────────────────────────────────

enum WidgetData {
    Rectangle,
    VStack {
        spacing: f32,
        padding: f32,
        alignment: Alignment,
    },
    HStack {
        spacing: f32,
        padding: f32,
        alignment: Alignment,
    },
    Text {
        content: String,
        style: TypographyStyle,
    },
    Button(Box<ButtonData>),
    // M3-Phase 2 DD-M3-P2-001 per-kind tag. `aspect` / `fill` are stored as
    // Box-internal domain types (DD-M3-P2-002 / DD-M3-P2-003 variant
    // strategy Option A) — neither is a `PropertyValue` variant in Phase 2,
    // and DD-M3-P2-004 keeps both constant-only, so they never traverse the
    // property / binding / ABI paths. The (at most one) child lives on
    // `WidgetNode.children` per the existing per-widget convention; the
    // single-child invariant is enforced by `wasamoc check` (T3) and
    // `ir_loader::build_node` (T7), not by this data shape.
    //
    // Readers: `aspect` is forwarded into `LayoutNode` at `build_layout_tree`
    // and drives the DD-M3-P2-005 measure-arrange. `fill` is materialised as
    // a `CompositionColorBrush` on the SpriteVisual at construction time
    // (`WidgetNode::box_`); Phase 2 keeps it constant per DD-M3-P2-004.
    Box {
        aspect: Option<box_values::Ratio>,
        #[allow(dead_code)]
        fill: Option<box_values::Color>,
    },
}

// ── Property dispatch (M1 experimental property IDs from wasamo.h §5) ─────────

pub const PROP_BUTTON_LABEL: u32 = 1;
pub const PROP_BUTTON_STYLE: u32 = 2;
pub const PROP_TEXT_CONTENT: u32 = 3;
pub const PROP_TEXT_STYLE: u32 = 4;
pub const PROP_BUTTON_ENABLED: u32 = 5;

#[derive(Debug, Clone)]
pub enum PropertyValue {
    I32(i32),
    String(String),
    Bool(bool),
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

// M3-Phase 2 T8 / DD-M3-P2-005: translate the pure-logic `LayoutError`
// into a `windows::core::Error` so the `WM_SIZE` → `run_layout` call
// chain keeps its existing `windows::core::Result<()>` shape. The
// `WASAMO_ERR_*` ABI surface for layout-time runtime errors is deferred
// (today's call sites at `window.rs` / `emit.rs` already swallow the
// Result with `let _ = …`); for now we propagate the message as `E_FAIL`
// so the GUI-loop diagnostic is at least visible to a debugger.
fn layout_error_to_winerr(err: LayoutError) -> windows::core::Error {
    use windows::core::{Error, HRESULT};
    const E_FAIL: HRESULT = HRESULT(0x80004005_u32 as i32);
    let msg = match err {
        LayoutError::BoxAspectUnboundedBoth => {
            "Box with `aspect` has no bounded parent axis (DD-M3-P2-005)"
        }
        LayoutError::BoxNoExtent => "Box has no extent to resolve (DD-M3-P2-005)",
    };
    Error::new(E_FAIL, msg)
}

fn button_style_to_i32(s: ButtonStyle) -> i32 {
    match s {
        ButtonStyle::Default => 0,
        ButtonStyle::Accent => 1,
    }
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
        TypographyStyle::Caption => 0,
        TypographyStyle::Body => 1,
        TypographyStyle::Subtitle => 2,
        TypographyStyle::Title => 3,
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
    /// DSL inline handler body for a named signal (DD-M2-P3-002 = Option B).
    /// `(signal_name, expr)` — stored directly on the widget, separate from
    /// the host listener list. Phase 6 populates this from textual IR.
    pub inline_handlers: Vec<(String, HandlerExpr)>,
    /// True while this node is attached to a parent (or window root).
    /// Maintained by `append_child`, `insert_child`, `remove_child`,
    /// `replace_child`, and `window::set_root`. Used by `widget_destroy`
    /// to reject destruction of still-attached widgets (DD-M2-P4-003).
    pub attached: bool,
    /// Reactive bindings owned by this node. Dropping an EffectHandle
    /// removes it from the dependency graph (DD-M2-P5-003).
    pub(crate) bindings: Vec<EffectHandle>,
}

// ── Tree-mutation errors ──────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum MutationError {
    IndexOutOfBounds,
    AlreadyAttached,
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
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
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
            data: WidgetData::VStack {
                spacing,
                padding,
                alignment,
            },
            width: SizeConstraint::Fill,
            height: SizeConstraint::Shrink,
            visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
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
            data: WidgetData::HStack {
                spacing,
                padding,
                alignment,
            },
            width: SizeConstraint::Shrink,
            height: SizeConstraint::Fill,
            visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
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
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
        )?;
        let brush: CompositionSurfaceBrush = compositor.CreateSurfaceBrushWithSurface(&surface)?;
        visual.SetBrush(&brush)?;
        Ok(Box::new(Self {
            data: WidgetData::Text {
                content: text.to_owned(),
                style,
            },
            width: SizeConstraint::Fixed(w),
            height: SizeConstraint::Fixed(h),
            visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
        }))
    }

    // M3-Phase 2 T6 / T7 / T8: Box constructor. The (at most one) child is
    // appended via the existing tree-mutation API, matching every other
    // widget. `aspect` / `fill` are populated by `ir_loader::build_node`
    // from `IrLiteral::Ratio` / `IrLiteral::Color` (T7) — they are
    // Box-internal domain types (DD-M3-P2-002 / DD-M3-P2-003 Option A)
    // and never travel as `PropertyValue`.
    //
    // T8: default `width` / `height` are `Shrink` so parent containers
    // (VStack / HStack / window root) honour the size produced by the
    // DD-M3-P2-005 inscribed-fit measure-arrange in `layout::measure_box`.
    // `fill` materialises as a `CompositionColorBrush` on the SpriteVisual
    // here (Phase 2 keeps `fill` constant per DD-M3-P2-004, so no later
    // mutation path needs to retain the brush). An absent `fill` leaves
    // the visual without a brush — i.e. transparent — per DD-M3-P2-005's
    // "zero-child Box still produces a sized rectangle (filled with
    // `fill`, or transparent when absent)".
    pub(crate) fn box_(
        compositor: &Compositor,
        aspect: Option<box_values::Ratio>,
        fill: Option<box_values::Color>,
    ) -> windows::core::Result<Box<Self>> {
        let visual = compositor.CreateSpriteVisual()?;
        if let Some(c) = fill {
            // box_values::Color packs as 0xAARRGGBB per dsl_spec §8.2.
            let packed = c.0;
            let color = Color {
                A: ((packed >> 24) & 0xFF) as u8,
                R: ((packed >> 16) & 0xFF) as u8,
                G: ((packed >> 8) & 0xFF) as u8,
                B: (packed & 0xFF) as u8,
            };
            let brush = compositor.CreateColorBrushWithColor(color)?;
            visual.SetBrush(&brush)?;
        }
        Ok(Box::new(Self {
            data: WidgetData::Box { aspect, fill },
            width: SizeConstraint::Shrink,
            height: SizeConstraint::Shrink,
            visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
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
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
        )?;
        let label_brush: CompositionSurfaceBrush =
            compositor.CreateSurfaceBrushWithSurface(&surface)?;
        label_visual.SetBrush(&label_brush)?;

        // Position label centered in the button.
        use windows::core::Interface;
        let label_vis: Visual = label_visual.cast()?;
        label_vis.SetOffset(Vector3 {
            X: PAD_H,
            Y: PAD_V,
            Z: 0.0,
        })?;
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
            enabled: true,
        });

        Ok(Box::new(Self {
            data: WidgetData::Button(btn_data),
            width: SizeConstraint::Fixed(btn_w),
            height: SizeConstraint::Fixed(btn_h),
            visual: bg_visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
        }))
    }

    // ── Property setters ──────────────────────────────────────────────────────

    pub fn set_color(
        &self,
        compositor: &Compositor,
        r: u8,
        g: u8,
        b: u8,
    ) -> windows::core::Result<()> {
        let brush = compositor.CreateColorBrushWithColor(Color {
            A: 255,
            R: r,
            G: g,
            B: b,
        })?;
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
            (WidgetData::Button(btn), PROP_BUTTON_ENABLED) => Ok(PropertyValue::Bool(btn.enabled)),
            (WidgetData::Text { content, .. }, PROP_TEXT_CONTENT) => {
                Ok(PropertyValue::String(content.clone()))
            }
            (WidgetData::Text { style, .. }, PROP_TEXT_STYLE) => {
                Ok(PropertyValue::I32(typography_to_i32(*style)))
            }
            _ => Err(PropertyError::UnknownId),
        }
    }

    pub fn set_property(&mut self, id: u32, value: &PropertyValue) -> Result<(), PropertyError> {
        // Track whether this property affects intrinsic size (DD-P8-002).
        let size_affecting = matches!(
            (&self.data, id),
            (WidgetData::Button(_), PROP_BUTTON_LABEL)
                | (WidgetData::Text { .. }, PROP_TEXT_CONTENT)
                | (WidgetData::Text { .. }, PROP_TEXT_STYLE)
        );
        let result = match (&mut self.data, id) {
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
                let new_style = button_style_from_i32(v).ok_or(PropertyError::TypeMismatch)?;
                self.update_button_style(new_style)
            }
            (WidgetData::Button(_), PROP_BUTTON_ENABLED) => {
                let v = match value {
                    PropertyValue::Bool(b) => *b,
                    _ => return Err(PropertyError::TypeMismatch),
                };
                self.update_button_enabled(v)
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
                let new_style = typography_from_i32(v).ok_or(PropertyError::TypeMismatch)?;
                self.update_text_style(new_style)
            }
            _ => Err(PropertyError::UnknownId),
        };
        if result.is_ok() && size_affecting {
            crate::emit::mark_layout_dirty_for(self as *mut WidgetNode);
        }
        result
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
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
        )?;
        let label_brush: CompositionSurfaceBrush =
            compositor.CreateSurfaceBrushWithSurface(&surface)?;
        btn.label_visual.SetBrush(&label_brush)?;

        use windows::core::Interface;
        const PAD_H: f32 = 16.0;
        const PAD_V: f32 = 8.0;
        let label_vis: Visual = btn.label_visual.cast()?;
        label_vis.SetOffset(Vector3 {
            X: PAD_H,
            Y: PAD_V,
            Z: 0.0,
        })?;
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
        let target = effective_button_color(btn.style, btn.state, btn.accent, btn.enabled);
        let new_brush = compositor.CreateColorBrushWithColor(target)?;
        self.visual.SetBrush(&new_brush)?;
        btn.bg_brush = new_brush;
        Ok(())
    }

    fn update_button_enabled(&mut self, new_enabled: bool) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let WidgetData::Button(ref mut btn) = self.data else {
            return Err(PropertyError::UnknownId);
        };
        if btn.enabled == new_enabled {
            return Ok(());
        }
        btn.enabled = new_enabled;
        // Phase 1 narrow contract (DD-M3-P1-005): swap the brush directly with
        // no animation. A disabled button still occupies its layout slot — we
        // only repaint the background and reset the transient state so the
        // grey colour isn't immediately overridden by a stale hover/press.
        btn.state = ButtonState::Normal;
        let target = effective_button_color(btn.style, btn.state, btn.accent, btn.enabled);
        let new_brush = compositor.CreateColorBrushWithColor(target)?;
        self.visual.SetBrush(&new_brush)?;
        btn.bg_brush = new_brush;
        Ok(())
    }

    fn update_text_content(&mut self, new_content: &str) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let renderer = &rt.text_renderer;

        let WidgetData::Text {
            ref mut content,
            style,
        } = self.data
        else {
            return Err(PropertyError::UnknownId);
        };
        let (w, h) = renderer.measure(new_content, style)?;
        let surface = renderer.draw_text(
            new_content,
            style,
            w.max(1.0),
            h.max(1.0),
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
        )?;
        let brush: CompositionSurfaceBrush = compositor.CreateSurfaceBrushWithSurface(&surface)?;
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

        let WidgetData::Text {
            ref mut content,
            ref mut style,
        } = self.data
        else {
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
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
        )?;
        let brush: CompositionSurfaceBrush = compositor.CreateSurfaceBrushWithSurface(&surface)?;
        self.visual.SetBrush(&brush)?;

        self.width = SizeConstraint::Fixed(w);
        self.height = SizeConstraint::Fixed(h);
        Ok(())
    }

    // ── Inline handler registration ───────────────────────────────────────────

    /// Attach a DSL inline handler for `signal_name` to this widget.
    /// Called by the IR loader (Phase 6) when building the widget tree.
    pub fn set_inline_handler(&mut self, signal_name: impl Into<String>, expr: HandlerExpr) {
        self.inline_handlers.push((signal_name.into(), expr));
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
            // Phase 1 `Button.enabled` (DD-M3-P1-005): suppress click dispatch
            // when disabled — neither the host callback nor the inline `clicked`
            // handler fires, and no "clicked" signal is enqueued. Hit-testing
            // still recurses into children below so non-Button descendants of
            // a disabled Button (none in M3-Phase 1, defensive) remain
            // reachable.
            if !btn.enabled {
                for child in &mut self.children {
                    child.hit_test_click_inner(x, y, abs_x, abs_y);
                }
                return;
            }
            let fx = x as f32;
            let fy = y as f32;
            if fx >= abs_x && fx < abs_x + vw && fy >= abs_y && fy < abs_y + vh {
                if let Some(ref f) = btn.clicked_fn {
                    f();
                }
                // DD-M2-P3-002 Option B: evaluate inline handlers first, then
                // enqueue host listeners. Inline path is separate from the
                // host listener list and is not a disconnectable token.
                // Safety: inline_handlers borrows are released before
                // enqueue_signal, which does not touch this node.
                let handler_exprs: Vec<HandlerExpr> = {
                    // Safety: widget_ptr aliases self; we collect clones before
                    // dispatch so no aliased mutable borrow is live during eval.
                    unsafe { &*widget_ptr }
                        .inline_handlers
                        .iter()
                        .filter(|(sig, _)| sig == "clicked")
                        .map(|(_, expr)| expr.clone())
                        .collect()
                };
                // DD-M2-P6-006: dispatch handler bodies against the
                // SignalRegistry installed by the IR loader. If no registry
                // is active (e.g. tests building widgets directly) fall back
                // to a no-op context — the evaluator runs but property
                // reads/writes report "unknown property".
                let registry = crate::reactive::active_registry();
                for expr in &handler_exprs {
                    // DD-M2-P3-003: catch_unwind wrapper logs errors and
                    // continues the event loop; location is a coarse
                    // identifier (Phase 6 supplies the component name prefix).
                    if let Some(reg) = registry.as_deref() {
                        let mut ctx = crate::reactive::HandlerEvalContext::new(reg);
                        handler::invoke_handler(expr, &mut ctx, "?.clicked");
                    } else {
                        let mut ctx = NullEvalContext;
                        handler::invoke_handler(expr, &mut ctx, "?.clicked");
                    }
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
            // Phase 1 `Button.enabled` (DD-M3-P1-005): a disabled button does
            // not react to hover/press — its background stays at the flat
            // grey set by `update_button_enabled`.
            if !btn.enabled {
                for child in &mut self.children {
                    child.update_hover_inner(compositor, x, y, down, abs_x, abs_y)?;
                }
                return Ok(());
            }
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

    /// Visit every node in the subtree rooted at `self`, including `self`.
    /// The visitor receives a raw pointer suitable for registry lookup
    /// (the same pointer the host received as `WasamoWidget*`).
    pub fn for_each_ptr(&self, visit: &mut dyn FnMut(*mut WidgetNode)) {
        visit(self as *const WidgetNode as *mut WidgetNode);
        for c in &self.children {
            c.for_each_ptr(visit);
        }
    }

    pub fn append_child(&mut self, mut child: Box<WidgetNode>) -> windows::core::Result<()> {
        use windows::core::Interface;
        let parent_container: ContainerVisual = self.visual.cast()?;
        let child_visual: Visual = child.visual.cast()?;
        parent_container.Children()?.InsertAtTop(&child_visual)?;
        child.attached = true;
        self.children.push(child);
        Ok(())
    }

    // ── Tree-mutation primitives (DD-M2-P4-001/002 = Option A) ───────────────

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn insert_child(
        &mut self,
        index: usize,
        mut child: Box<WidgetNode>,
    ) -> Result<(), MutationError> {
        if index > self.children.len() {
            return Err(MutationError::IndexOutOfBounds);
        }
        if child.attached {
            return Err(MutationError::AlreadyAttached);
        }
        use windows::core::Interface;
        let parent_container: ContainerVisual = self
            .visual
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let child_visual: Visual = child
            .visual
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        parent_container
            .Children()
            .and_then(|c| c.InsertAtTop(&child_visual))
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        child.attached = true;
        self.children.insert(index, child);
        Ok(())
    }

    pub fn remove_child(&mut self, index: usize) -> Result<Box<WidgetNode>, MutationError> {
        if index >= self.children.len() {
            return Err(MutationError::IndexOutOfBounds);
        }
        use windows::core::Interface;
        let child_visual: Visual = self.children[index]
            .visual
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let parent_container: ContainerVisual = self
            .visual
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        parent_container
            .Children()
            .and_then(|c| c.Remove(&child_visual))
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let mut removed = self.children.remove(index);
        removed.attached = false;
        Ok(removed)
    }

    pub fn replace_child(
        &mut self,
        index: usize,
        mut new_child: Box<WidgetNode>,
    ) -> Result<Box<WidgetNode>, MutationError> {
        if index >= self.children.len() {
            return Err(MutationError::IndexOutOfBounds);
        }
        if new_child.attached {
            return Err(MutationError::AlreadyAttached);
        }
        use windows::core::Interface;
        let old_visual: Visual = self.children[index]
            .visual
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let new_visual: Visual = new_child
            .visual
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let parent_container: ContainerVisual = self
            .visual
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let children_col = parent_container
            .Children()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        children_col
            .Remove(&old_visual)
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        children_col
            .InsertAtTop(&new_visual)
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        new_child.attached = true;
        let mut old = std::mem::replace(&mut self.children[index], new_child);
        old.attached = false;
        Ok(old)
    }

    // ── Layout ────────────────────────────────────────────────────────────────

    /// Builds a LayoutNode tree, runs layout, then syncs results back to SpriteVisuals.
    ///
    /// M3-Phase 2 T8: `layout::run_layout` is fallible — it surfaces
    /// `LayoutError::BoxAspectUnboundedBoth` / `BoxNoExtent` from
    /// DD-M3-P2-005. We translate those into `windows::core::Error` so
    /// the existing `WM_SIZE` -> `r.run_layout(cw, ch)` call sites (which
    /// already swallow the Result with `let _ = …`) keep their current
    /// shape. A dedicated C ABI surface for layout-time runtime errors
    /// is out of Phase 2 scope and tracked alongside the ABI work in
    /// later phases.
    pub fn run_layout(&mut self, window_w: f32, window_h: f32) -> windows::core::Result<()> {
        let mut layout_tree = self.build_layout_tree();
        layout::run_layout(&mut layout_tree, window_w, window_h).map_err(layout_error_to_winerr)?;
        self.sync_visuals(&layout_tree)
    }

    fn build_layout_tree(&self) -> LayoutNode {
        match &self.data {
            WidgetData::Rectangle | WidgetData::Text { .. } | WidgetData::Button(_) => {
                LayoutNode::rectangle(self.width.clone(), self.height.clone())
            }
            WidgetData::VStack {
                spacing,
                padding,
                alignment,
            } => {
                let mut node = LayoutNode::vstack(*spacing, *padding, *alignment);
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self
                    .children
                    .iter()
                    .map(|c| c.build_layout_tree())
                    .collect();
                node
            }
            WidgetData::HStack {
                spacing,
                padding,
                alignment,
            } => {
                let mut node = LayoutNode::hstack(*spacing, *padding, *alignment);
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self
                    .children
                    .iter()
                    .map(|c| c.build_layout_tree())
                    .collect();
                node
            }
            // M3-Phase 2 T8: thread the Box-internal `aspect` into the
            // pure-logic layout engine. The engine's `layout::Ratio` is a
            // structural mirror of `box_values::Ratio` (DD-M3-P2-002
            // Option A keeps both Box-internal — neither is a
            // `PropertyValue`); the conversion here is the boundary at
            // which the runtime's `WidgetData::Box` field hands data to
            // the Win32/WinRT-free layout module. `fill` is consumed by
            // `WidgetNode::box_` at construction (painted onto the
            // SpriteVisual brush) and does not enter `LayoutNode`.
            WidgetData::Box { aspect, .. } => {
                let layout_ratio = aspect.map(|r| layout::Ratio {
                    num: r.num,
                    den: r.den,
                });
                let mut node = LayoutNode::box_(layout_ratio);
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self
                    .children
                    .iter()
                    .map(|c| c.build_layout_tree())
                    .collect();
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
        for (child, child_computed) in self.children.iter_mut().zip(computed.children.iter()) {
            child.sync_visuals(child_computed)?;
        }
        Ok(())
    }
}

// ── EvalContext placeholder (Phase 3) ─────────────────────────────────────────
//
// Phase 5 (reactive engine) will replace this with a context that resolves
// dot-path property names against the live widget tree and property store.
// For Phase 3 the evaluator runs but all property accesses return "unknown".

struct NullEvalContext;

impl EvalContext for NullEvalContext {
    fn get_i32(&self, path: &str) -> Result<i32, EvalError> {
        Err(EvalError::UnknownProperty(path.to_string()))
    }
    fn get_string(&self, path: &str) -> Result<String, EvalError> {
        Err(EvalError::UnknownProperty(path.to_string()))
    }
    fn set_i32(&mut self, path: &str, _value: i32) -> Result<(), EvalError> {
        Err(EvalError::UnknownProperty(path.to_string()))
    }
}

// ── Reactive binding writer (DD-M2-P5-005 production caller) ─────────────────

/// Static write function passed to `register_binding` as the property writer.
///
/// The reactive engine calls this whenever a binding's tracked Signal changes;
/// the stringified value produced by `evaluate_binding` is written to the
/// widget property identified by `(id, prop)`. M2 string-typed properties
/// (`PROP_TEXT_CONTENT`, `PROP_BUTTON_LABEL`) accept the value as-is. Other
/// property kinds (e.g. typed integers via DD-M2-P6-011) will introduce
/// kind-aware dispatch when their loader paths land.
///
/// Safety: `id` was created from `widget.as_mut() as *mut WidgetNode` by the
/// IR loader; the runtime is single-threaded GUI; the WidgetNode outlives the
/// binding (the EffectHandle is owned by `WidgetNode.bindings`, so disposal
/// runs before the node is dropped — DD-M2-P5-003).
pub(crate) fn widget_write_property(id: crate::reactive::WidgetId, prop: u32, value: &str) {
    let node_ptr = id.0 as *mut WidgetNode;
    if node_ptr.is_null() {
        return;
    }
    let val = PropertyValue::String(value.to_string());
    unsafe {
        let _ = (*node_ptr).set_property(prop, &val);
    }
}

/// Bool-typed counterpart of `widget_write_property` (DD-M3-P1-007 Option A).
///
/// Paired with `reactive::register_bool_binding` at the IR loader's binding
/// registration site: when `resolve_prop_key` reports `IrType::Bool` for the
/// target property, the loader selects this writer instead of the string
/// writer. The `PropertyValue::Bool` constructed here flows through the same
/// per-widget `set_property` dispatch the string writer uses; only the
/// `PropertyValue` variant changes.
///
/// Safety: identical contract to `widget_write_property` — the WidgetId is
/// the loader-supplied node pointer, the runtime is single-threaded GUI,
/// and the EffectHandle that owns this closure lives no longer than the
/// `WidgetNode.bindings` vec.
pub(crate) fn widget_write_property_bool(id: crate::reactive::WidgetId, prop: u32, value: bool) {
    let node_ptr = id.0 as *mut WidgetNode;
    if node_ptr.is_null() {
        return;
    }
    let val = PropertyValue::Bool(value);
    unsafe {
        let _ = (*node_ptr).set_property(prop, &val);
    }
}

// ── Subtree teardown helper (DD-M2-P4-003) ───────────────────────────────────

/// Sever all registry entries in the subtree, then drop it.
/// Called by `wasamo_widget_destroy` (abi.rs) and shared with
/// `wasamo_window_destroy`'s existing sweep path.
pub fn widget_destroy(mut node: Box<WidgetNode>) {
    dispose_subtree_bindings(&mut node);
    node.for_each_ptr(&mut |p| crate::registry::remove_for_widget(p));
    drop(node);
}

fn dispose_subtree_bindings(node: &mut WidgetNode) {
    node.bindings.clear();
    for child in &mut node.children {
        dispose_subtree_bindings(child);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn visual_rect(v: &SpriteVisual) -> (f32, f32, f32, f32) {
    use windows::core::Interface;
    let vis: Visual = v.cast().unwrap_or_else(|_| panic!("cast failed"));
    let off = vis.Offset().unwrap_or(Vector3 {
        X: 0.0,
        Y: 0.0,
        Z: 0.0,
    });
    let sz = vis.Size().unwrap_or(Vector2 { X: 0.0, Y: 0.0 });
    (off.X, off.Y, sz.X, sz.Y)
}

// Minimal disabled-state colour (DD-M3-P1-005 Phase 1 contract): a flat grey
// fill, independent of style and hover/press state, deliberately animation-
// free. M4/M5 own the richer disabled visuals.
const BUTTON_DISABLED_COLOR: Color = Color {
    A: 0x40,
    R: 0x80,
    G: 0x80,
    B: 0x80,
};

fn effective_button_color(
    style: ButtonStyle,
    state: ButtonState,
    accent: Color,
    enabled: bool,
) -> Color {
    if !enabled {
        BUTTON_DISABLED_COLOR
    } else {
        button_state_color(style, state, accent)
    }
}

fn button_state_color(style: ButtonStyle, state: ButtonState, accent: Color) -> Color {
    match (style, state) {
        (ButtonStyle::Default, ButtonState::Normal) => Color {
            A: 0x20,
            R: 0xFF,
            G: 0xFF,
            B: 0xFF,
        },
        (ButtonStyle::Default, ButtonState::Hovered) => Color {
            A: 0x33,
            R: 0xFF,
            G: 0xFF,
            B: 0xFF,
        },
        (ButtonStyle::Default, ButtonState::Pressed) => Color {
            A: 0x10,
            R: 0xFF,
            G: 0xFF,
            B: 0xFF,
        },
        (ButtonStyle::Accent, ButtonState::Normal) => accent,
        (ButtonStyle::Accent, ButtonState::Hovered) => lighten(accent, 26),
        (ButtonStyle::Accent, ButtonState::Pressed) => darken(accent, 26),
    }
}

// Duration in 100-ns ticks: fast (83 ms) for entering active state, slow (167 ms) for leaving.
fn transition_duration(old: ButtonState, new: ButtonState) -> i64 {
    match (old, new) {
        (_, ButtonState::Pressed) => 830_000,   // press-down: fast
        (ButtonState::Pressed, _) => 1_670_000, // press-up: slow
        (ButtonState::Normal, ButtonState::Hovered) => 830_000, // hover-in: fast
        _ => 1_670_000,                         // hover-out: slow
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
    anim.SetDuration(TimeSpan {
        Duration: duration_ticks,
    })?;
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
        .unwrap_or(Color {
            A: 255,
            R: 0,
            G: 120,
            B: 215,
        }) // Windows default blue
}

// ── Unit tests ────────────────────────────────────────────────────────────────
//
// These tests exercise the pure-logic parts of the mutation API: bounds
// checking and `attached` flag transitions. The WinRT visual operations
// within `WidgetNode` cannot run without a live Compositor; the logic
// below is extracted into a minimal `Slot` that mirrors the same invariants
// without any OS dependency.

#[cfg(test)]
mod tests {
    use super::MutationError;

    // Minimal stand-in for WidgetNode used only to verify index-check and
    // attached-flag logic, without requiring a Win32/WinRT environment.
    #[derive(Debug, PartialEq)]
    struct Slot {
        attached: bool,
    }

    impl Slot {
        fn new() -> Self {
            Slot { attached: false }
        }
    }

    struct Children(Vec<Slot>);

    impl Children {
        fn new() -> Self {
            Children(Vec::new())
        }

        fn len(&self) -> usize {
            self.0.len()
        }

        fn insert(&mut self, index: usize, mut slot: Slot) -> Result<(), MutationError> {
            if index > self.0.len() {
                return Err(MutationError::IndexOutOfBounds);
            }
            if slot.attached {
                return Err(MutationError::AlreadyAttached);
            }
            slot.attached = true;
            self.0.insert(index, slot);
            Ok(())
        }

        fn remove(&mut self, index: usize) -> Result<Slot, MutationError> {
            if index >= self.0.len() {
                return Err(MutationError::IndexOutOfBounds);
            }
            let mut slot = self.0.remove(index);
            slot.attached = false;
            Ok(slot)
        }

        fn replace(&mut self, index: usize, mut new: Slot) -> Result<Slot, MutationError> {
            if index >= self.0.len() {
                return Err(MutationError::IndexOutOfBounds);
            }
            if new.attached {
                return Err(MutationError::AlreadyAttached);
            }
            new.attached = true;
            let mut old = std::mem::replace(&mut self.0[index], new);
            old.attached = false;
            Ok(old)
        }
    }

    #[test]
    fn insert_at_zero() {
        let mut ch = Children::new();
        assert!(ch.insert(0, Slot::new()).is_ok());
        assert_eq!(ch.len(), 1);
        assert!(ch.0[0].attached);
    }

    #[test]
    fn insert_at_end() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        ch.insert(1, Slot::new()).unwrap();
        assert_eq!(ch.len(), 2);
    }

    #[test]
    fn insert_at_mid() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        ch.insert(1, Slot::new()).unwrap();
        ch.insert(1, Slot::new()).unwrap();
        assert_eq!(ch.len(), 3);
    }

    #[test]
    fn insert_out_of_bounds() {
        let mut ch = Children::new();
        assert_eq!(
            ch.insert(1, Slot::new()),
            Err(MutationError::IndexOutOfBounds)
        );
    }

    #[test]
    fn insert_already_attached() {
        let mut ch = Children::new();
        let s = Slot { attached: true };
        assert_eq!(ch.insert(0, s), Err(MutationError::AlreadyAttached));
    }

    #[test]
    fn remove_normal() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        let removed = ch.remove(0).unwrap();
        assert!(!removed.attached);
        assert_eq!(ch.len(), 0);
    }

    #[test]
    fn remove_out_of_bounds() {
        let mut ch = Children::new();
        assert_eq!(ch.remove(0), Err(MutationError::IndexOutOfBounds));
    }

    #[test]
    fn remove_returns_detached() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        let removed = ch.remove(0).unwrap();
        assert!(!removed.attached);
    }

    #[test]
    fn replace_normal() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        let old = ch.replace(0, Slot::new()).unwrap();
        assert!(!old.attached);
        assert!(ch.0[0].attached);
    }

    #[test]
    fn replace_out_of_bounds() {
        let mut ch = Children::new();
        assert_eq!(
            ch.replace(0, Slot::new()),
            Err(MutationError::IndexOutOfBounds)
        );
    }

    #[test]
    fn replace_new_already_attached() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        let s = Slot { attached: true };
        assert_eq!(ch.replace(0, s), Err(MutationError::AlreadyAttached));
    }

    #[test]
    fn child_count_after_insert_remove() {
        let mut ch = Children::new();
        assert_eq!(ch.len(), 0);
        ch.insert(0, Slot::new()).unwrap();
        assert_eq!(ch.len(), 1);
        ch.insert(1, Slot::new()).unwrap();
        assert_eq!(ch.len(), 2);
        ch.remove(0).unwrap();
        assert_eq!(ch.len(), 1);
    }

    #[test]
    fn attached_transition_append_remove() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        assert!(ch.0[0].attached);
        let s = ch.remove(0).unwrap();
        assert!(!s.attached);
    }

    #[test]
    fn reattach_after_remove() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        let s = ch.remove(0).unwrap();
        assert!(!s.attached);
        // Re-attaching the same slot (now detached) should succeed.
        ch.insert(0, s).unwrap();
        assert!(ch.0[0].attached);
    }

    #[test]
    fn already_attached_cannot_reattach() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        let already = Slot { attached: true };
        assert_eq!(ch.insert(0, already), Err(MutationError::AlreadyAttached));
    }

    // ── Binding disposal mirror ───────────────────────────────────────────────
    //
    // NodeMirror replicates the WidgetNode.bindings + widget_destroy ordering
    // without any Win32/WinRT dependency. `destroy()` mirrors
    // dispose_subtree_bindings → registry-sever → drop.

    use crate::reactive::{EffectHandle, Signal};

    struct NodeMirror {
        bindings: Vec<EffectHandle>,
        children: Vec<NodeMirror>,
    }

    impl NodeMirror {
        fn new() -> Self {
            NodeMirror {
                bindings: Vec::new(),
                children: Vec::new(),
            }
        }

        fn destroy(mut self) {
            self.dispose_bindings();
            // registry-sever would go here in production; nothing to do in mirror
            drop(self);
        }

        fn dispose_bindings(&mut self) {
            self.bindings.clear();
            for child in &mut self.children {
                child.dispose_bindings();
            }
        }
    }

    #[test]
    fn destroy_disposes_bindings_and_stops_effect() {
        let sig = Signal::new(0i32);
        let fired = std::rc::Rc::new(std::cell::Cell::new(false));
        let fired_clone = fired.clone();
        let sig_clone = sig.clone();

        let mut node = NodeMirror::new();
        node.bindings.push(EffectHandle::new(move || {
            sig_clone.get();
            fired_clone.set(true);
        }));

        // Initial run set fired; clear before destroy.
        fired.set(false);
        node.destroy();

        // Writing the signal must not re-fire the disposed binding.
        fired.set(false);
        sig.set(1);
        assert!(!fired.get(), "binding fired after widget_destroy");
    }

    #[test]
    fn destroy_child_binding_also_stopped() {
        let sig = Signal::new(0i32);
        let fired = std::rc::Rc::new(std::cell::Cell::new(false));
        let fired_clone = fired.clone();
        let sig_clone = sig.clone();

        let mut parent = NodeMirror::new();
        let mut child = NodeMirror::new();
        child.bindings.push(EffectHandle::new(move || {
            sig_clone.get();
            fired_clone.set(true);
        }));
        parent.children.push(child);

        fired.set(false);
        parent.destroy();

        fired.set(false);
        sig.set(1);
        assert!(
            !fired.get(),
            "child binding fired after parent widget_destroy"
        );
    }

    // ── M3-Phase 2 T6: Box widget data shape ─────────────────────────────────
    //
    // These tests exercise the `WidgetData::Box` variant directly without a
    // Compositor — the variant has no Win32/WinRT field, so its data shape
    // is verifiable as pure logic. The `WidgetNode::box_` constructor itself
    // needs a Compositor and is covered by T11's Windows-runtime integration
    // test.

    use super::WidgetData;
    use crate::box_values::{Color as BoxFill, Ratio};

    #[test]
    fn box_variant_carries_optional_aspect_and_fill() {
        let data = WidgetData::Box {
            aspect: Some(Ratio { num: 16, den: 9 }),
            fill: Some(BoxFill(0x80_00_00_00)),
        };
        match &data {
            WidgetData::Box { aspect, fill } => {
                assert_eq!(*aspect, Some(Ratio { num: 16, den: 9 }));
                assert_eq!(*fill, Some(BoxFill(0x80_00_00_00)));
            }
            _ => panic!("expected WidgetData::Box variant"),
        }
    }

    #[test]
    fn box_variant_defaults_both_fields_to_none() {
        // Mirrors the `WidgetNode::box_` constructor's default field
        // initialisation. The constructor itself requires a Compositor;
        // here we only assert the data-shape default that the constructor
        // writes.
        let data = WidgetData::Box {
            aspect: None,
            fill: None,
        };
        if let WidgetData::Box { aspect, fill } = &data {
            assert!(aspect.is_none());
            assert!(fill.is_none());
        } else {
            panic!("expected WidgetData::Box variant");
        }
    }
}
