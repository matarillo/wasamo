use crate::box_values;
use crate::dip_scale::DipScale;
use crate::handler::{self, EvalContext, EvalError, HandlerExpr};
use crate::hit::{self, DipPoint, HitTree};
use crate::layout::{
    self, Alignment, ChildSlots, LayoutChildSlot, LayoutError, LayoutNode, SizeConstraint,
    SlotData, TrackSize,
};
use crate::reactive::EffectHandle;
use crate::text::{TextRenderer, TypographyStyle};
use std::ops::{Deref, DerefMut};
use windows::{
    Foundation::{
        Numerics::{Vector2, Vector3},
        TimeSpan,
    },
    UI::{
        Color,
        Composition::{
            AnimationIterationBehavior, ColorKeyFrameAnimation, CompositionAnimation,
            CompositionColorBrush, CompositionDrawingSurface, CompositionObject,
            CompositionStretch, CompositionSurfaceBrush, Compositor, ContainerVisual, InsetClip,
            SpriteVisual, Visual,
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

// Standard Button-family sizing: label + horizontal/vertical padding.
// One declaration, read by the two sites that derive the node's
// `SizeConstraint` from a fresh measurement (`button_family`,
// `update_button_label`) and by the sync pass that places the label
// Visual inside the background Visual.
const BUTTON_PAD_H: f32 = 16.0;
const BUTTON_PAD_V: f32 = 8.0;

struct ButtonData {
    style: ButtonStyle,
    state: ButtonState,
    checked: bool,
    // Background brush retained for in-place color animation (DD-P5-005).
    bg_brush: CompositionColorBrush,
    label_visual: SpriteVisual,
    label_text: String,
    label_style: TypographyStyle,
    // The extent `TextRenderer::measure` returned for `label_text` at
    // `label_style`, in DIP. Written by the same two primitives that write
    // `label_text` and that derive this node's `SizeConstraint::Fixed` pair
    // from the same measurement; read by `sync_visuals`, which places the
    // label Visual. It is retained rather than re-measured because the sync
    // pass holds no `TextRenderer`, and rather than recovered from the
    // arranged size because a stretched button's arranged size is no longer
    // the measured label plus padding.
    label_size: (f32, f32),
    clicked_fn: Option<Box<dyn Fn()>>,
    // Accent color for ButtonStyle::Accent (read from UISettings at creation).
    accent: Color,
    // Phase 1 narrow `Button.enabled` contract (M3-Phase 1 DD-M3-P1-005):
    // when false, click-handler dispatch is suppressed and the background is
    // greyed with no animation. Focus / a11y / hover-state semantics are
    // deferred to M4–M5.
    enabled: bool,
    /// Whether this node currently paints the focus indicator (M4-Phase 2
    /// T5; DD-M4-P2-003 "the indicator is presentation state on the
    /// node... and not a new visual written at focus-change time").
    /// Paired with the window's `WindowFocus` record; written only by
    /// [`WidgetNode::set_button_focused_at`], whose only caller is
    /// `crate::focus::move_focus`.
    focused: bool,
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
    ToggleButton(Box<ButtonData>),
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
        fill: Option<box_values::Color>,
    },
    // M3-Phase 3 DD-M3-P3-001 per-kind tag for the WrapPanel layout
    // primitive. The three attributes are stored as `i32` per DD-M3-P3-003
    // / DD-M3-P3-004 (constant-only `i32` plumbing; no new `PropertyValue`
    // variant). Defaults — applied at this layer, not at the IR layer —
    // are `item_cross_size: None` (parent-cross passthrough per
    // DD-M3-P3-004 Option (a)) and `item_spacing: 0` / `line_spacing: 0`
    // (touching items / lines per DD-M3-P3-003). Children live on
    // `WidgetNode.children` per the existing per-widget convention,
    // mirroring Phase 2's `Box` shape (0+ children, no upper bound;
    // single-child invariant intentionally absent — DD-M3-P3-001).
    //
    // T7 layout reads these fields via the per-kind dispatch on
    // `WidgetData` and the `WidgetKind::WrapPanel` arm in `layout.rs`.
    // T6 wired the IR-loader path (`ir_loader::construct_widget`
    // "WrapPanel" arm) so this variant is constructed in production —
    // the T5-era `#[allow(dead_code)]` forward-pointer is no longer
    // needed. No `PropertyValue` / binding / ABI paths touch these
    // fields per DD-M3-P3-003 / DD-M3-P3-004 constant-only invariants.
    WrapPanel {
        item_cross_size: Option<i32>,
        item_spacing: i32,
        line_spacing: i32,
    },
    // M3-Phase 4 DD-M3-P4-001 per-kind tag for the ScrollView layout
    // primitive. `offset_y` is stored as `i32` per DD-M3-P4-003 (`i32`
    // pixels, bindable read-only); the field default of `0` (DD-M3-P4-003
    // absent-attribute default) is applied at the widget-catalog
    // constructor layer (`WidgetNode::scroll_view`), not at the IR
    // loader. The single content child lives on `WidgetNode.children`
    // per the existing per-widget convention (mirrors Phase 2 Box); the
    // exactly-1-child invariant is enforced by `wasamoc check` (T1) and
    // `ir_loader::validate()` (T3), not by this data shape.
    //
    // T4 layout consumes `offset_y` via `build_layout_tree`'s
    // `LayoutNode::scroll_view(offset_y)` boundary; the clamp arithmetic
    // lives in `layout::arrange_scroll_view` (T2). No `PropertyValue`
    // variant carries `offset_y` — the read-only binding path stringifies
    // through `widget_write_property`, and the narrow string-to-`i32`
    // parse lives on the ScrollView arm of `set_property`
    // (`update_scroll_view_offset_y`).
    //
    // `content_visual` is the ScrollView-owned intermediate Visual
    // (DD-M3-P4-004): a SpriteVisual that sits between `WidgetNode.visual`
    // (the outer clipped Visual) and the single content child's widget
    // Visual. It carries the scroll translation
    // `Visual.Offset = (0, -applied_y, 0)` (the clamped applied offset
    // recorded on `LayoutNode.applied_offset_y` by `arrange_scroll_view`
    // in T2). `WidgetNode::append_child` / `insert_child` / `remove_child`
    // / `replace_child` route ScrollView's children into this Visual
    // rather than the outer one, so the outer Visual carries only the
    // viewport clip (`Visual.Clip = InsetClip{0,0,0,0}`) and the
    // intermediate carries only the scroll translation — neither
    // conflates with the other or with the child widget's own
    // layout-derived `Visual.Offset`.
    ScrollView {
        offset_y: i32,
        content_visual: SpriteVisual,
    },
    // M3-Phase 5 DD-M3-P5-001 per-kind tag for the Grid layout primitive.
    // The track lists are stored as layout-engine mirror types. Per-child
    // placement rides the runtime child slot as `SlotData::Grid`, so this
    // variant no longer carries parent metadata parallel to `children`.
    // `Cell` is IR-only (DD-M3-P5-001) and never materialises as a
    // `WidgetData` variant. No `PropertyValue` / binding / ABI path touches
    // these fields (Phase 5 constant-only, DD-M3-P5-001 / DD-M3-P5-006).
    //
    // The outer Visual carries the DD-M3-P5-005 outer-bounds clip
    // (`Visual.Clip = InsetClip{0,0,0,0}`, installed in `WidgetNode::grid`,
    // the same zero-inset auto-tracking clip ScrollView's outer Visual
    // uses); Grid paints no background brush (it is a pure layout
    // container). T4 asserts the clip presence on the live Visual.
    Grid {
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
    },
    // M3-Phase 6 DD-M3-P6-001 / DD-M3-P6-002 per-kind tag for the ZStack
    // layout primitive. Children are direct real widgets in document order
    // (first = bottom, last = top). Per-child `h-align` / `v-align` rides
    // the runtime child slot as `SlotData::ZStack`, so placement travels
    // with the child through mutation and staging. The outer Visual carries
    // the zero-inset clip; children deliberately do not get per-child clips.
    ZStack,
}

pub struct ChildSlot {
    node: Box<WidgetNode>,
    slot_data: Option<SlotData>,
}

impl ChildSlot {
    fn new(node: Box<WidgetNode>, slot_data: Option<SlotData>) -> Self {
        Self { node, slot_data }
    }

    fn into_node(self) -> Box<WidgetNode> {
        self.node
    }
}

impl Deref for ChildSlot {
    type Target = WidgetNode;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for ChildSlot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

impl AsRef<WidgetNode> for ChildSlot {
    fn as_ref(&self) -> &WidgetNode {
        &self.node
    }
}

impl AsMut<WidgetNode> for ChildSlot {
    fn as_mut(&mut self) -> &mut WidgetNode {
        &mut self.node
    }
}

// ── Property dispatch (M1 experimental property IDs from wasamo.h §5) ─────────

pub const PROP_BUTTON_LABEL: u32 = 1;
pub const PROP_BUTTON_STYLE: u32 = 2;
pub const PROP_TEXT_CONTENT: u32 = 3;
pub const PROP_TEXT_STYLE: u32 = 4;
pub const PROP_BUTTON_ENABLED: u32 = 5;
// M3-Phase 4 T4 / DD-M3-P4-003 narrow per-widget i32 parse bridge.
// `offset-y` is an `i32` DSL surface attribute on ScrollView; the
// existing reactive engine stringifies the bound `Signal<i32>`'s value
// through `evaluate_binding` and `widget_write_property`, and this
// per-widget `set_property` arm parses the string back into the `i32`
// `offset_y` field on `WidgetData::ScrollView`. No new `PropertyValue`
// variant is introduced (the general typed-`i32` evaluator / writer
// pair stays deferred to M4+ per ADR §M4 hand-off item 2).
pub const PROP_SCROLLVIEW_OFFSET_Y: u32 = 6;
pub const PROP_TOGGLEBUTTON_CHECKED: u32 = 7;

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

// M3-Phase 3 T5: WrapPanel absent-to-default mapping (DD-M3-P3-003 /
// DD-M3-P3-004). The runtime catalog owns this policy per the T3
// progress note "defaults are applied at the runtime layer in T5, not
// at the IR layer", so the IR loader (T6) only forwards presence /
// absence and the constructor / pure-logic tests share one
// authoritative mapping site.
//
// - `item_cross_size` absent → `None` (parent-cross passthrough per
//   DD-M3-P3-004 Option (a); the storage is already `Option<i32>`, so
//   absence flows through unchanged).
// - `item_spacing` absent → `0` (touching items per DD-M3-P3-003).
// - `line_spacing` absent → `0` (touching lines per DD-M3-P3-003).
//
// T6 wires the IR-loader path that calls `WidgetNode::wrap_panel`,
// which in turn calls this helper — the T5-era `#[allow(dead_code)]`
// forward-pointer is no longer needed.
pub(crate) fn apply_wrap_panel_defaults(
    item_cross_size: Option<i32>,
    item_spacing: Option<i32>,
    line_spacing: Option<i32>,
) -> (Option<i32>, i32, i32) {
    (
        item_cross_size,
        item_spacing.unwrap_or(0),
        line_spacing.unwrap_or(0),
    )
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
        LayoutError::ScrollViewUnboundedAxis => {
            "ScrollView has no bounded scroll-axis viewport (DD-M3-P4-002)"
        }
        LayoutError::GridUnboundedStarAxis => {
            "Grid has a star track on an unbounded parent axis (DD-M3-P5-004)"
        }
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

/// Create the one-to-one surface mapping required by DD-M4-P1-006.
///
/// A drawing surface is allocated at `ceil(dip * scale)` whole pixels while
/// the Visual keeps the exact fractional physical extent. The default brush
/// is `Uniform` and centred; relying on it scales and displaces the surface.
/// `None` with zero alignment keeps unit scale and clips excess storage at the
/// right and bottom Visual bounds.
fn create_text_surface_brush(
    compositor: &Compositor,
    surface: &CompositionDrawingSurface,
) -> windows::core::Result<CompositionSurfaceBrush> {
    let brush = compositor.CreateSurfaceBrushWithSurface(surface)?;
    brush.SetStretch(CompositionStretch::None)?;
    brush.SetHorizontalAlignmentRatio(0.0)?;
    brush.SetVerticalAlignmentRatio(0.0)?;
    Ok(brush)
}

/// Read the retained natural DIP extent of a Text node without re-measuring.
fn fixed_extent(width: &SizeConstraint, height: &SizeConstraint) -> Option<(f32, f32)> {
    match (width, height) {
        (SizeConstraint::Fixed(width), SizeConstraint::Fixed(height)) => Some((*width, *height)),
        _ => None,
    }
}

// ── WidgetNode ────────────────────────────────────────────────────────────────

/// An absolute rectangle, in DIP, within the window's client area.
///
/// The geometry source for hit testing from M4-Phase 2 onward
/// (DD-M4-P2-002 "one walk, two stores"): `WidgetNode::arranged_rect`
/// retains this value, derived from the same `LayoutNode.offset` /
/// `.size` the Composition `Visual.Offset` / `Visual.Size` writes come
/// from, in the same `sync_visuals` pass.
///
/// No methods beyond the derives here: containment / hit-testing (M4-Phase
/// 2 T2) is `hit::contains` / `hit::resolve_topmost`, free functions over
/// this type rather than methods on it, so the resolver stays pure logic
/// with no Compositor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DipRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// The M4-Phase 2 authored focus annotations carried by a container node
/// (`focus-group: true` / `modal-scope: true`, dsl_spec §4.19;
/// DD-M4-P2-005 A1). Built-time state with **exactly one writer** — the
/// IR loader (`ir_loader::build_node_with_loop_context`), through
/// [`WidgetNode::set_focus_annotation`]. It is not window state, does not
/// participate in layout, and creates no `Visual` and writes no geometry:
/// it exists only to be read back by [`WidgetNode::focus_role`], which
/// widens the widget-kind-only derivation (`FocusRole::Stop` /
/// `FocusRole::Container`) into the two additional roles a container can
/// carry (`FocusRole::Group`, `FocusRole::ModalScope`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FocusAnnotation {
    group: bool,
    modal_scope: bool,
}

pub struct WidgetNode {
    data: WidgetData,
    width: SizeConstraint,
    height: SizeConstraint,
    pub visual: SpriteVisual,
    pub children: Vec<ChildSlot>,
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
    /// This node's cached copy of its window's DIP -> physical conversion
    /// factor (M4-Phase 1; the carrier decision recorded in
    /// `implementation/log.md` §T1).
    ///
    /// **A cache with exactly one writer** — the attach / scale-change walk —
    /// while `WindowState::scale` holds the authoritative value. It exists
    /// because hit testing and property-write re-rasterization stand on a node
    /// with no window in hand. Production `sync_visuals` does not infer its
    /// target from this copy: the window layout caller supplies the
    /// authoritative value, and the successful geometry pass commits it here
    /// before returning. Standalone Rust layout entries use the root copy as
    /// their target because they are not attached through `WindowState`.
    ///
    /// `DipScale::default()` is the identity, so a tree that has not been
    /// attached to a window converts as 1 — which is also what every node holds
    /// until the walk lands, and is why introducing the field changes no
    /// rendered output.
    ///
    /// Not `pub` and not `pub(crate)`: every reader is in this module.
    scale: DipScale,
    /// DPI of the text surface most recently installed for this node.
    ///
    /// This is deliberately independent of `scale`: geometry can advance
    /// after a recoverable WinRT rasterization failure, and retryability must
    /// describe the brush that actually exists rather than the geometry that
    /// should exist. Non-text nodes carry the same marker so the recursive
    /// freshness walk has one uniform node shape; exhaustive `WidgetData`
    /// matching keeps a future text-bearing variant from becoming a silent
    /// no-op.
    raster_scale: DipScale,
    /// The layout-derived hit geometry for this node (DD-M4-P2-002 "one walk,
    /// two stores"): an absolute DIP rectangle within the window's client
    /// area.
    ///
    /// **Exactly one writer** — the lockstep walk in `sync_visuals`, which
    /// derives it from the same `LayoutNode.offset` / `.size` value the
    /// Composition `Visual.Offset` / `Visual.Size` writes come from, in the
    /// same pass.
    ///
    /// `None` means this node has not been through a layout pass, which
    /// makes it not hit-testable — the intended failure rather than being
    /// hit-testable at a stale rectangle.
    ///
    /// Not `pub` and not `pub(crate)`: every reader is in this module.
    arranged_rect: Option<DipRect>,
    /// The M4-Phase 2 authored focus annotation for this node (see
    /// [`FocusAnnotation`]'s doc comment for the writer / reader
    /// contract). **Built-time state with exactly one writer** — the IR
    /// loader, through [`Self::set_focus_annotation`] — not window
    /// state: it does not participate in layout and creates no `Visual`,
    /// so every constructor below sets it to `FocusAnnotation::default()`
    /// (both flags `false`, matching an un-annotated container) and only
    /// the IR loader ever writes a non-default value.
    focus_annotation: FocusAnnotation,
}

// ── Tree-mutation errors ──────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum MutationError {
    IndexOutOfBounds,
    AlreadyAttached,
}

fn mutation_error_to_winerr(err: MutationError) -> windows::core::Error {
    use windows::core::{Error, HRESULT};
    const E_FAIL: HRESULT = HRESULT(0x80004005_u32 as i32);
    let msg = match err {
        MutationError::IndexOutOfBounds => "Widget child mutation index out of bounds",
        MutationError::AlreadyAttached => "Widget child is already attached",
    };
    Error::new(E_FAIL, msg)
}

// ── Hover state (M4-Phase 2 T4) ────────────────────────────────────────────

/// A window's retained record of which node currently paints a non-`Normal`
/// `ButtonState` (`docs/architecture.md` §13.2 "hover and pressed... computed
/// as enter / leave transitions against the resolved target"). One
/// `HoverState` lives on `WindowState` (`window.rs`); `WidgetNode::update_hover`
/// / `WidgetNode::clear_hover` are its only writers.
///
/// `target` is the path of child indices, from the **window's root
/// widget**, of the one node that currently paints a non-`Normal`
/// `ButtonState`; `None` when no node is entered.
///
/// **Invariant: at most one node in the tree has `state != ButtonState::Normal`,
/// and it is exactly the node this path names.** `update_hover` and
/// `clear_hover` write the record and the painted state together, in the
/// same function call, so the pair can never be edited apart from those two
/// entry points (T4 start gate trap #3).
///
/// **`update_button_enabled` is a third writer of `ButtonData::state`** — a
/// binding write that disables a Button-family widget resets `state` to
/// `ButtonState::Normal` directly, with no `HoverState` in reach. When
/// `target` still names that now-disabled node, the next leave
/// (`update_hover` or `clear_hover`) resolves to a node whose `state` is
/// already `Normal`; `set_button_state_at`'s `if new_state != btn.state`
/// guard then makes that leave a no-op, so the flat disabled grey
/// `update_button_enabled` painted (`docs/dsl_spec.md` §4.8) survives
/// instead of being unconditionally re-animated.
#[derive(Default)]
pub(crate) struct HoverState {
    target: Option<Vec<usize>>,
}

impl HoverState {
    /// Read-back for the FFI test seam
    /// (`lib.rs::ffi::__hover_target_for_test`). `target` itself stays
    /// private — every writer is `WidgetNode::update_hover` /
    /// `WidgetNode::clear_hover`, both in this module — so a fixture across
    /// the crate boundary can assert the retained path agrees with the
    /// painted state without this accessor becoming a second writer.
    pub(crate) fn target(&self) -> Option<&[usize]> {
        self.target.as_deref()
    }
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
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
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
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
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
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
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
        let surface = renderer.draw_text_at_dpi(
            text,
            style,
            w,
            h,
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
            DipScale::IDENTITY.dpi(),
        )?;
        let brush = create_text_surface_brush(compositor, &surface)?;
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
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
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
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
        }))
    }

    // M3-Phase 3 T5: WrapPanel constructor. All three attributes arrive as
    // `Option<i32>` so callers (chiefly the T6 IR loader's
    // `construct_widget` "WrapPanel" arm) can pass through DSL presence /
    // absence verbatim — the runtime catalog itself owns the
    // absent-to-default policy per the T3 progress note "defaults are
    // applied at the runtime layer in T5, not at the IR layer". The
    // default mapping lives in the pure-logic `apply_wrap_panel_defaults`
    // free function below so unit tests can pin it without a Compositor.
    // The constructor does not paint a background brush — WrapPanel is a
    // layout container (mirrors VStack / HStack which also leave the
    // visual brush unset). Children are appended via the existing
    // tree-mutation API.
    //
    // T6 wires the IR-loader path that constructs this widget via
    // `ir_loader::construct_widget` — the T5-era `#[allow(dead_code)]`
    // forward-pointer is no longer needed.
    pub(crate) fn wrap_panel(
        compositor: &Compositor,
        item_cross_size: Option<i32>,
        item_spacing: Option<i32>,
        line_spacing: Option<i32>,
    ) -> windows::core::Result<Box<Self>> {
        let (item_cross_size, item_spacing, line_spacing) =
            apply_wrap_panel_defaults(item_cross_size, item_spacing, line_spacing);
        let visual = compositor.CreateSpriteVisual()?;
        Ok(Box::new(Self {
            data: WidgetData::WrapPanel {
                item_cross_size,
                item_spacing,
                line_spacing,
            },
            width: SizeConstraint::Fill,
            height: SizeConstraint::Shrink,
            visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
        }))
    }

    // M3-Phase 4 T3: ScrollView constructor. `offset_y` arrives as
    // `Option<i32>` so the IR loader (`ir_loader::construct_widget`
    // "ScrollView" arm) can pass through DSL presence / absence
    // verbatim — the runtime catalog owns the absent-to-default policy
    // per DD-M3-P4-003 (default `offset-y: 0` at the widget-catalog
    // layer, not the IR loader). The constructor does not paint a
    // background brush — ScrollView is a layout container that owns a
    // clip surface (the InsetClip install lands in T4 with the
    // intermediate content Visual). The single content child is
    // appended via the existing tree-mutation API; the child-count
    // invariant is enforced upstream by `wasamoc check` (T1) and
    // `ir_loader::validate()` (T3).
    pub(crate) fn scroll_view(
        compositor: &Compositor,
        offset_y: Option<i32>,
    ) -> windows::core::Result<Box<Self>> {
        use windows::core::Interface;
        let offset_y = offset_y.unwrap_or(0);
        let visual = compositor.CreateSpriteVisual()?;
        // DD-M3-P4-004 Option A: outer Visual carries the viewport clip
        // (`Visual.Clip = InsetClip{0,0,0,0}`). Zero insets means the
        // clip tracks the Visual's `Size` automatically, so the
        // sync_visuals size write at every layout pass keeps the
        // clipped region in sync with the viewport.
        let clip: InsetClip = compositor.CreateInsetClip()?;
        let outer_visual: Visual = visual.cast()?;
        outer_visual.SetClip(&clip)?;
        // DD-M3-P4-004 Option A: ScrollView-owned intermediate content
        // Visual sits between the outer Visual and the single content
        // child's widget Visual; carries the scroll translation
        // `Visual.Offset = (0, -applied_y, 0)` written by `sync_visuals`.
        // Children inserted into this WidgetNode are routed beneath
        // `content_visual` by `content_container_visual`, so the outer
        // Visual's child collection contains exactly the intermediate
        // Visual and the intermediate's child collection contains the
        // single content child widget Visual.
        let content_visual = compositor.CreateSpriteVisual()?;
        let outer_container: ContainerVisual = visual.cast()?;
        let content_as_visual: Visual = content_visual.cast()?;
        outer_container
            .Children()?
            .InsertAtTop(&content_as_visual)?;
        Ok(Box::new(Self {
            data: WidgetData::ScrollView {
                offset_y,
                content_visual,
            },
            width: SizeConstraint::Fill,
            height: SizeConstraint::Fill,
            visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
        }))
    }

    // M3-Phase 5 T3 / M3-Phase 7b T3: Grid constructor. The track lists
    // arrive already converted to the layout-engine mirror types; per-child
    // placement is carried by child slots inserted after construction. The
    // constructor installs the DD-M3-P5-005 outer-bounds
    // clip on the outer Visual (`Visual.Clip = InsetClip{0,0,0,0}`, the
    // same zero-inset auto-tracking clip ScrollView's outer Visual uses,
    // so the `sync_visuals` size write keeps the clipped region in sync
    // with the Grid rect each layout pass). Grid paints no background
    // brush. Width / height default to `Fill` / `Fill` (mirrors
    // `LayoutNode::grid`); the child-count / placement invariants are
    // enforced upstream by `wasamoc check` and `ir_loader::validate()`
    // (T3), not by this data shape.
    pub(crate) fn grid(
        compositor: &Compositor,
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
    ) -> windows::core::Result<Box<Self>> {
        use windows::core::Interface;
        let visual = compositor.CreateSpriteVisual()?;
        let clip: InsetClip = compositor.CreateInsetClip()?;
        let outer_visual: Visual = visual.cast()?;
        outer_visual.SetClip(&clip)?;
        Ok(Box::new(Self {
            data: WidgetData::Grid { columns, rows },
            width: SizeConstraint::Fill,
            height: SizeConstraint::Fill,
            visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
        }))
    }

    // M3-Phase 6 T3 / M3-Phase 7 T5: ZStack materialisation. Width /
    // height default to Fill / Fill (overlay-first). Per-child placement is
    // carried by each child slot and consumed by the ZStack parent during
    // layout (DD-M3-P7-006), so the container stores no parallel metadata.
    pub(crate) fn zstack(compositor: &Compositor) -> windows::core::Result<Box<Self>> {
        use windows::core::Interface;
        let visual = compositor.CreateSpriteVisual()?;
        let clip: InsetClip = compositor.CreateInsetClip()?;
        let outer_visual: Visual = visual.cast()?;
        outer_visual.SetClip(&clip)?;
        Ok(Box::new(Self {
            data: WidgetData::ZStack,
            width: SizeConstraint::Fill,
            height: SizeConstraint::Fill,
            visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
        }))
    }

    /// Test-only accessor for the ScrollView-owned intermediate content
    /// Visual (DD-M3-P4-004 Option A). Returns the `SpriteVisual` whose
    /// `Visual.Offset` carries the scroll translation `(0, -applied_y, 0)`
    /// and beneath which the single content child's widget Visual is
    /// attached. Returns `None` for non-ScrollView widgets. Hidden from
    /// rustdoc and named with the project's `__*_for_test` convention.
    /// Used by `wasamo-runtime/tests/scroll_view_layout_integration.rs`
    /// to assert (b)–(g) of ADR Phase 4 verification closure item 4.
    #[doc(hidden)]
    pub fn __scroll_view_intermediate_for_test(&self) -> Option<SpriteVisual> {
        match &self.data {
            WidgetData::ScrollView { content_visual, .. } => Some(content_visual.clone()),
            _ => None,
        }
    }

    /// Returns the Visual that ScrollView's child widget Visuals are
    /// attached beneath: the intermediate content Visual for ScrollView
    /// (DD-M3-P4-004 Option A); `self.visual` for every other widget.
    /// Used by the tree-mutation primitives (`append_child` /
    /// `insert_child` / `remove_child` / `replace_child`) so the scroll
    /// translation `Visual.Offset = (0, -applied_y, 0)` on the
    /// intermediate Visual stays separated from the child widget's own
    /// layout-derived `Visual.Offset`.
    fn content_container_visual(&self) -> &SpriteVisual {
        match &self.data {
            WidgetData::ScrollView { content_visual, .. } => content_visual,
            _ => &self.visual,
        }
    }

    // Test-only accessor for `WidgetData::Box` (M3-Phase 2 ADR §Phase 2
    // verification closure item 2, build_node materialisation half). Returns
    // the Box-internal `aspect` / `fill` as primitives so cross-crate
    // integration tests can assert that `IrLiteral::Ratio` / `IrLiteral::Color`
    // materialised into the Box-internal domain types per DD-M3-P2-002 /
    // DD-M3-P2-003 variant strategy Option A — without leaking the
    // `box_values::Ratio` / `box_values::Color` `pub(crate)` types past
    // crate boundaries. `None` for non-Box widgets. Hidden from rustdoc
    // and named with the project's `__*_for_test` convention (cf.
    // `lib.rs::ffi::__install_owning_thread_for_test`); production callers
    // must use `wasamo_get_property` for the M3-Phase 2 `PropertyValue`-
    // mediated paths (which `aspect` / `fill` deliberately are not part
    // of, per DD-M3-P2-004 keeping both constant-only).
    #[doc(hidden)]
    pub fn __box_state_for_test(&self) -> Option<(Option<(i32, i32)>, Option<u32>)> {
        match &self.data {
            WidgetData::Box { aspect, fill } => {
                Some((aspect.map(|r| (r.num, r.den)), fill.map(|c| c.0)))
            }
            _ => None,
        }
    }

    pub fn button(
        compositor: &Compositor,
        renderer: &TextRenderer,
        label: &str,
        style: ButtonStyle,
    ) -> windows::core::Result<Box<Self>> {
        Self::button_family(compositor, renderer, label, style, true, false, false)
    }

    /// Same as [`Self::button`], but with an explicit initial `enabled`
    /// state (M4-Phase 2 T8, CF-2 disposition). `button` keeps its
    /// hard-coded `enabled: true` so every other caller — the C ABI
    /// `wasamo_button_create`, and the pre-existing `WidgetNode::button`
    /// call sites in `tests/arranged_rect_integration.rs` and
    /// `tests/button_enabled.rs` — stays source-compatible; only the IR
    /// loader's `"Button"` materialisation arm needs a literal
    /// `enabled: false` to reach construction, matching `toggle_button`'s
    /// `enabled` parameter.
    pub fn button_with_enabled(
        compositor: &Compositor,
        renderer: &TextRenderer,
        label: &str,
        style: ButtonStyle,
        enabled: bool,
    ) -> windows::core::Result<Box<Self>> {
        Self::button_family(compositor, renderer, label, style, enabled, false, false)
    }

    pub fn toggle_button(
        compositor: &Compositor,
        renderer: &TextRenderer,
        label: &str,
        style: ButtonStyle,
        enabled: bool,
        checked: bool,
    ) -> windows::core::Result<Box<Self>> {
        Self::button_family(compositor, renderer, label, style, enabled, checked, true)
    }

    fn button_family(
        compositor: &Compositor,
        renderer: &TextRenderer,
        label: &str,
        style: ButtonStyle,
        enabled: bool,
        checked: bool,
        toggle: bool,
    ) -> windows::core::Result<Box<Self>> {
        let label_style = TypographyStyle::Body;
        let (lw, lh) = renderer.measure(label, label_style)?;

        let btn_w = lw + BUTTON_PAD_H * 2.0;
        let btn_h = lh + BUTTON_PAD_V * 2.0;

        let accent = read_accent_color();

        // Root visual: background.
        let bg_visual = compositor.CreateSpriteVisual()?;
        let initial_color =
            effective_button_color(style, ButtonState::Normal, accent, enabled, checked, false);
        let bg_brush = compositor.CreateColorBrushWithColor(initial_color)?;
        bg_visual.SetBrush(&bg_brush)?;

        // Child visual: text label.
        let label_visual = compositor.CreateSpriteVisual()?;
        let surface = renderer.draw_text_at_dpi(
            label,
            label_style,
            lw,
            lh,
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
            DipScale::IDENTITY.dpi(),
        )?;
        let label_brush = create_text_surface_brush(compositor, &surface)?;
        label_visual.SetBrush(&label_brush)?;

        // Parent the label Visual under the background Visual. Its offset and
        // size are *not* written here: construction precedes attachment to any
        // window — through the IR loader or through the Rust-native API — so no
        // scale factor exists at this moment (DD-M4-P1-002 §Row 6 detail). They
        // are written by `sync_visuals`, alongside every other Composition
        // geometry write in the runtime.
        //
        // The consequence is that a Button-family widget shows its label only
        // once a layout pass has run over it, and **not every path that puts a
        // widget on screen runs one**. `window::set_root` and the `WM_SIZE` arm
        // do; the tree-mutation API (`append_child` / `insert_child` /
        // `replace_child`, whether called directly or through their ABI
        // wrappers) does not, and neither does `lib.rs::window_add_widget`.
        // Those paths rely on a later `WM_SIZE` or size-affecting property
        // write, except `window_add_widget`, which has no later pass at all.
        // A new mutation entry that omits `emit::mark_layout_dirty_for` will
        // reproduce the missing label.
        use windows::core::Interface;
        let label_vis: Visual = label_visual.cast()?;
        let bg_container: ContainerVisual = bg_visual.cast()?;
        bg_container.Children()?.InsertAtTop(&label_vis)?;

        let btn_data = Box::new(ButtonData {
            style,
            state: ButtonState::Normal,
            checked,
            bg_brush,
            label_visual,
            label_text: label.to_owned(),
            label_style,
            label_size: (lw, lh),
            clicked_fn: None,
            accent,
            enabled,
            focused: false,
        });
        let data = if toggle {
            WidgetData::ToggleButton(btn_data)
        } else {
            WidgetData::Button(btn_data)
        };

        Ok(Box::new(Self {
            data,
            width: SizeConstraint::Fixed(btn_w),
            height: SizeConstraint::Fixed(btn_h),
            visual: bg_visual,
            children: Vec::new(),
            inline_handlers: Vec::new(),
            attached: false,
            bindings: Vec::new(),
            scale: DipScale::default(),
            raster_scale: DipScale::default(),
            arranged_rect: None,
            focus_annotation: FocusAnnotation::default(),
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

    /// Register a callback invoked when this Button-family widget is clicked.
    ///
    /// **Residual, not introduced by M4-Phase 2 T3.** `clicked_fn` is a
    /// `Box<dyn Fn()>` stored inline on this node and is not `Clone`, so
    /// click dispatch cannot snapshot it out the way it snapshots inline DSL
    /// handler expressions before running anything (see
    /// `run_clicked_handlers` in `hit_test_click`'s neighbourhood). If the
    /// closure registered here destroys this very node — e.g. it detaches
    /// the node and drops the owning `Box<WidgetNode>` — it frees the
    /// `Box<dyn Fn()>` it is executing inside. That is a pre-existing
    /// property of this Rust-native test/example API, not something this
    /// task fixes.
    pub fn set_clicked<F: Fn() + 'static>(&mut self, f: F) {
        if let Some(btn) = self.button_data_mut() {
            btn.clicked_fn = Some(Box::new(f));
        }
    }

    fn button_data_mut(&mut self) -> Option<&mut ButtonData> {
        match &mut self.data {
            WidgetData::Button(btn) | WidgetData::ToggleButton(btn) => Some(btn),
            _ => None,
        }
    }

    /// Immutable counterpart to [`Self::button_data_mut`]. M4-Phase 2 T3's
    /// read-only click dispositioning (`click_disposition_for`) needs to
    /// inspect a Button-family node's `enabled` flag and `clicked_fn`
    /// presence without taking a mutable borrow, so that peeking at every
    /// node on the dispatch chain cannot itself conflict with anything.
    fn button_data(&self) -> Option<&ButtonData> {
        match &self.data {
            WidgetData::Button(btn) | WidgetData::ToggleButton(btn) => Some(btn),
            _ => None,
        }
    }

    /// Write this node's authored focus annotation (`focus-group: true` /
    /// `modal-scope: true`, dsl_spec §4.19). **The only writer**: called
    /// once, from the IR loader
    /// (`ir_loader::build_node_with_loop_context`), immediately after
    /// construction. No binding or property-write path calls this —
    /// both attributes are constant-only (DD-M4-P2-005 A1), so there is
    /// no second writer to keep in step with this one.
    pub(crate) fn set_focus_annotation(&mut self, group: bool, modal_scope: bool) {
        self.focus_annotation = FocusAnnotation { group, modal_scope };
    }

    /// The focus role and enabled state derivable from this node's widget
    /// kind, its `enabled` flag (Button family), and — from M4-Phase 2 T6
    /// onward — its authored [`FocusAnnotation`] (containers only).
    /// `crate::focus::FocusProjection::project` calls this once per node
    /// in its pre-order walk over the window's live tree.
    ///
    /// Deliberately total over `WidgetData` rather than a `_` arm, so a
    /// later widget kind cannot become a silent `Container`.
    ///
    /// **The Button-family arm is unchanged** (DD-M4-P2-003 F3): neither
    /// `focus-group` nor `modal-scope` is admitted on a Button-family
    /// widget (`ir_loader::validate_focus_annotation_invariants` — Button
    /// is not one of the seven `FOCUS_ANNOTATION_CONTAINERS`), so
    /// consulting an annotation there would be an unreachable branch.
    ///
    /// **The container arm consults the annotation**:
    ///
    /// - `modal_scope` set → `FocusRole::ModalScope`
    /// - else `group` set → `FocusRole::Group`
    /// - else → `FocusRole::Container` (the M4-Phase 2 T5 baseline, and
    ///   the only role `Rectangle` / `Text` can ever show — the loader
    ///   never lets either carry a non-default annotation).
    ///
    /// **`modal-scope` wins when a node carries both**, because
    /// `focus_core::FocusRole` is one-of-six: a container cannot
    /// literally hold two roles at once. DD-M4-P2-005's forward-compat
    /// note records the both-at-once case as "expressible under A1 and
    /// untested in M4"; confinement is the stronger property — it
    /// changes which subtree is reachable at all, where a group only
    /// changes how Tab moves within one — so `modal-scope` takes
    /// precedence here. Whether the combination should instead mean
    /// something else is tracked in the pre-1.0 candidate pool with no
    /// milestone claimed (`process/candidate-pool.md` §"Focus-annotation
    /// surface", disposition recorded in
    /// `process/milestone-4/phase-2/implementation/log.md` §"Owner
    /// disposition of CF-T6-2"); this function's own answer —
    /// `modal-scope` wins — is the landed rule regardless of how that pool
    /// item resolves.
    ///
    /// **Reachability caveat — read before widening this function's value
    /// set.** `focus_role` has exactly one production caller, the `walk`
    /// helper in `focus.rs`, itself reached only from
    /// `FocusProjection::project`'s pre-order walk. `FocusProjection::project`
    /// in turn has six call sites in `focus.rs` — `sync_scopes_to_tree`,
    /// `traverse_on_key`, `focus_on_click`, `arrow_on_key`, `dismiss_on_key`,
    /// and `focused_path` — every production and test-seam path that reads
    /// the focus tree at all, so a wider value set here changes what all of
    /// them can reach, not only what this function returns.
    ///
    /// Widening this function's value set changes what the keyboard and
    /// pointer paths can reach, and each has its own production consumer:
    /// `focus_core::FocusTree::collect_stops` decides which roles Tab
    /// enumerates, `focus_core::FocusTree::tab` (through `resolve_stop`)
    /// decides where Tab lands inside a `Group`, and
    /// `focus_core::FocusTree::focus_landing` decides where a click lands,
    /// including landing directly on a `Group`'s clicked member rather than
    /// the group container (M4-Phase 2 T7, closing CF-T6-5). A
    /// `modal-scope` container is entered by its own presence:
    /// `focus::sync_scopes_to_tree`'s entry step enters every
    /// `FocusRole::ModalScope` node not already on the stack at the seam
    /// that materialises it, so a scope's subtree is reachable by both Tab
    /// and click-to-focus from the moment it exists in the tree.
    pub(crate) fn focus_role(&self) -> (crate::focus_core::FocusRole, bool) {
        use crate::focus_core::FocusRole;
        match &self.data {
            WidgetData::Button(btn) | WidgetData::ToggleButton(btn) => {
                (FocusRole::Stop, btn.enabled)
            }
            WidgetData::Rectangle
            | WidgetData::VStack { .. }
            | WidgetData::HStack { .. }
            | WidgetData::Text { .. }
            | WidgetData::Box { .. }
            | WidgetData::WrapPanel { .. }
            | WidgetData::ScrollView { .. }
            | WidgetData::Grid { .. }
            | WidgetData::ZStack { .. } => {
                let role = if self.focus_annotation.modal_scope {
                    FocusRole::ModalScope
                } else if self.focus_annotation.group {
                    FocusRole::Group
                } else {
                    FocusRole::Container
                };
                (role, true)
            }
        }
    }

    // ── Property R/W (wasamo.h §4.3 + §5 experimental property IDs) ───────────
    //
    // Dispatch is enum-on-`WidgetData`: each variant accepts only the IDs that
    // belong to it; everything else returns `UnknownId`. Types that do not
    // match the property's declared type return `TypeMismatch`.

    pub fn get_property(&self, id: u32) -> Result<PropertyValue, PropertyError> {
        match (&self.data, id) {
            (WidgetData::Button(btn) | WidgetData::ToggleButton(btn), PROP_BUTTON_LABEL) => {
                Ok(PropertyValue::String(btn.label_text.clone()))
            }
            (WidgetData::Button(btn) | WidgetData::ToggleButton(btn), PROP_BUTTON_STYLE) => {
                Ok(PropertyValue::I32(button_style_to_i32(btn.style)))
            }
            (WidgetData::Button(btn) | WidgetData::ToggleButton(btn), PROP_BUTTON_ENABLED) => {
                Ok(PropertyValue::Bool(btn.enabled))
            }
            (WidgetData::ToggleButton(btn), PROP_TOGGLEBUTTON_CHECKED) => {
                Ok(PropertyValue::Bool(btn.checked))
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

    pub fn set_property(&mut self, id: u32, value: &PropertyValue) -> Result<(), PropertyError> {
        // Track whether this property write requires a layout pass
        // (DD-P8-002 mark_layout_dirty hook). For Button / Text label
        // and font, the intrinsic size changes; for ScrollView
        // `offset-y` the intrinsic size is unchanged but the
        // `arrange_scroll_view` clamp and child placement re-run, so
        // the same dirty hook drives a re-layout in the next
        // drain_if_outermost cycle. The DSL surface property name
        // "size-affecting" is read loosely here as
        // "needs a layout pass to take effect on screen".
        let size_affecting = matches!(
            (&self.data, id),
            (
                WidgetData::Button(_) | WidgetData::ToggleButton(_),
                PROP_BUTTON_LABEL
            ) | (WidgetData::Text { .. }, PROP_TEXT_CONTENT)
                | (WidgetData::Text { .. }, PROP_TEXT_STYLE)
                | (WidgetData::ScrollView { .. }, PROP_SCROLLVIEW_OFFSET_Y)
        );
        let result = match (&mut self.data, id) {
            (WidgetData::Button(_) | WidgetData::ToggleButton(_), PROP_BUTTON_LABEL) => {
                let s = match value {
                    PropertyValue::String(s) => s.clone(),
                    _ => return Err(PropertyError::TypeMismatch),
                };
                self.update_button_label(&s)
            }
            (WidgetData::Button(_) | WidgetData::ToggleButton(_), PROP_BUTTON_STYLE) => {
                let v = match value {
                    PropertyValue::I32(v) => *v,
                    _ => return Err(PropertyError::TypeMismatch),
                };
                let new_style = button_style_from_i32(v).ok_or(PropertyError::TypeMismatch)?;
                self.update_button_style(new_style)
            }
            (WidgetData::Button(_) | WidgetData::ToggleButton(_), PROP_BUTTON_ENABLED) => {
                let v = match value {
                    PropertyValue::Bool(b) => *b,
                    _ => return Err(PropertyError::TypeMismatch),
                };
                self.update_button_enabled(v)
            }
            (WidgetData::ToggleButton(_), PROP_TOGGLEBUTTON_CHECKED) => {
                let v = match value {
                    PropertyValue::Bool(b) => *b,
                    _ => return Err(PropertyError::TypeMismatch),
                };
                self.update_toggle_button_checked(v)
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
            // M3-Phase 4 T4 / DD-M3-P4-003: narrow string-to-`i32` parse
            // bridge for the `offset-y` binding. The reactive engine
            // stringifies the `Signal<i32>` value via `evaluate_binding`
            // and routes it through `widget_write_property`, so the
            // value arrives here as `PropertyValue::String`. Parsing
            // failure is a `TypeMismatch` (the producer is the string
            // baker; non-integer content would mean the upstream
            // contract is broken). The general typed-`i32` evaluator /
            // writer pair from architecture.md §6.8 *Per-type seam*
            // stays deferred per ADR §M4 hand-off item 2.
            (WidgetData::ScrollView { .. }, PROP_SCROLLVIEW_OFFSET_Y) => {
                let s = match value {
                    PropertyValue::String(s) => s.as_str(),
                    _ => return Err(PropertyError::TypeMismatch),
                };
                let parsed: i32 = s.parse().map_err(|_| PropertyError::TypeMismatch)?;
                self.update_scroll_view_offset_y(parsed)
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
        let dpi = self.scale.dpi();

        let Some(btn) = self.button_data_mut() else {
            return Err(PropertyError::UnknownId);
        };
        let label_style = btn.label_style;
        let (lw, lh) = renderer.measure(new_label, label_style)?;
        let surface = renderer.draw_text_at_dpi(
            new_label,
            label_style,
            lw,
            lh,
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
            dpi,
        )?;
        let label_brush = create_text_surface_brush(compositor, &surface)?;
        btn.label_visual.SetBrush(&label_brush)?;

        btn.label_text = new_label.to_owned();
        // The label Visual's offset and size are written by `sync_visuals`
        // from `label_size`, not here — every Composition geometry write in
        // the runtime happens in that one pass (DD-M4-P1-002 §Row 6 detail).
        // The `size_affecting` clause in `set_property` already marks the
        // owning window's layout dirty for this property, so the drain's
        // layout phase runs the sync pass before the ABI call returns.
        btn.label_size = (lw, lh);
        // Natural size updates; takes effect on the next layout pass.
        self.width = SizeConstraint::Fixed(lw + BUTTON_PAD_H * 2.0);
        self.height = SizeConstraint::Fixed(lh + BUTTON_PAD_V * 2.0);
        self.raster_scale = self.scale;
        Ok(())
    }

    fn update_button_style(&mut self, new_style: ButtonStyle) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let visual = self.visual.clone();
        let Some(btn) = self.button_data_mut() else {
            return Err(PropertyError::UnknownId);
        };
        if btn.style == new_style {
            return Ok(());
        }
        btn.style = new_style;
        let target = effective_button_color(
            btn.style,
            btn.state,
            btn.accent,
            btn.enabled,
            btn.checked,
            btn.focused,
        );
        let new_brush = compositor.CreateColorBrushWithColor(target)?;
        visual.SetBrush(&new_brush)?;
        btn.bg_brush = new_brush;
        Ok(())
    }

    fn update_button_enabled(&mut self, new_enabled: bool) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let visual = self.visual.clone();
        let Some(btn) = self.button_data_mut() else {
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
        let target = effective_button_color(
            btn.style,
            btn.state,
            btn.accent,
            btn.enabled,
            btn.checked,
            btn.focused,
        );
        let new_brush = compositor.CreateColorBrushWithColor(target)?;
        visual.SetBrush(&new_brush)?;
        btn.bg_brush = new_brush;
        Ok(())
    }

    fn update_toggle_button_checked(&mut self, new_checked: bool) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let visual = self.visual.clone();
        let WidgetData::ToggleButton(ref mut btn) = self.data else {
            return Err(PropertyError::UnknownId);
        };
        if btn.checked == new_checked {
            return Ok(());
        }
        btn.checked = new_checked;
        let target = effective_button_color(
            btn.style,
            btn.state,
            btn.accent,
            btn.enabled,
            btn.checked,
            btn.focused,
        );
        let new_brush = compositor.CreateColorBrushWithColor(target)?;
        visual.SetBrush(&new_brush)?;
        btn.bg_brush = new_brush;
        Ok(())
    }

    // M3-Phase 4 T4 / DD-M3-P4-003: update the ScrollView's `i32`
    // `offset_y` field. Pure-data mutation — no Win32/WinRT side
    // effect at this point. The new value takes effect on the next
    // layout pass: `arrange_scroll_view` re-clamps it against the
    // current `(content_h - viewport_h)` upper bound and writes the
    // clamped applied offset onto `LayoutNode.applied_offset_y`,
    // which `sync_visuals` then writes onto the intermediate
    // content Visual's `Visual.Offset`. The set_property caller
    // marks the host window's layout dirty (size_affecting clause
    // above) so the existing `drain_if_outermost` re-layout path
    // picks up the change without a bespoke trigger.
    fn update_scroll_view_offset_y(&mut self, new_offset_y: i32) -> Result<(), PropertyError> {
        let WidgetData::ScrollView {
            ref mut offset_y, ..
        } = self.data
        else {
            return Err(PropertyError::UnknownId);
        };
        *offset_y = new_offset_y;
        Ok(())
    }

    fn update_text_content(&mut self, new_content: &str) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let renderer = &rt.text_renderer;
        let dpi = self.scale.dpi();

        let WidgetData::Text {
            ref mut content,
            style,
        } = self.data
        else {
            return Err(PropertyError::UnknownId);
        };
        let (w, h) = renderer.measure(new_content, style)?;
        let surface = renderer.draw_text_at_dpi(
            new_content,
            style,
            w,
            h,
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
            dpi,
        )?;
        let brush = create_text_surface_brush(compositor, &surface)?;
        self.visual.SetBrush(&brush)?;

        *content = new_content.to_owned();
        self.width = SizeConstraint::Fixed(w);
        self.height = SizeConstraint::Fixed(h);
        self.raster_scale = self.scale;
        Ok(())
    }

    fn update_text_style(&mut self, new_style: TypographyStyle) -> Result<(), PropertyError> {
        let rt = crate::runtime::get();
        let compositor = &rt.compositor;
        let renderer = &rt.text_renderer;
        let dpi = self.scale.dpi();

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
        let surface = renderer.draw_text_at_dpi(
            content,
            new_style,
            w,
            h,
            Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
            },
            dpi,
        )?;
        let brush = create_text_surface_brush(compositor, &surface)?;
        self.visual.SetBrush(&brush)?;

        self.width = SizeConstraint::Fixed(w);
        self.height = SizeConstraint::Fixed(h);
        self.raster_scale = self.scale;
        Ok(())
    }

    // ── Inline handler registration ───────────────────────────────────────────

    /// Attach a DSL inline handler for `signal_name` to this widget.
    /// Called by the IR loader (Phase 6) when building the widget tree.
    ///
    /// **`signal_name` is the canonical storage spelling, not the bare IR
    /// signal name** (M4-Phase 2 T8): a signal carrying an argument is
    /// stored under `wasamo_ir::signal_key`'s composed form —
    /// `key-down("ArrowLeft")` rather than `key-down` — so that two
    /// handlers for different keys on the same node are distinguishable by
    /// name alone. `signal_key` returns the bare name unchanged when there
    /// is no argument, so `clicked` and `dismiss` are stored exactly as
    /// before. The loader is the only caller, and every lookup side
    /// ([`signal_handlers_for`]'s callers) composes its key through the
    /// same function — nothing else builds this string
    /// (DD-M4-P2-005).
    pub fn set_inline_handler(&mut self, signal_name: impl Into<String>, expr: HandlerExpr) {
        self.inline_handlers.push((signal_name.into(), expr));
    }

    // ── Hit testing ───────────────────────────────────────────────────────────

    /// Resolve the topmost widget under `(x, y)` and dispatch its `clicked`
    /// event: the event fires at the resolved target and then walks its
    /// ancestors, stopping at the first node that actually runs a handler
    /// (DD-M4-P2-001 "target then bubble; a handler that runs consumes the
    /// event"; DD-M4-P2-002 "`clicked` is one signal on every widget", no
    /// longer Button-family-only).
    ///
    /// **`(x, y)` are DIP** (M4-Phase 1, DD-M4-P1-002 option H2): the window
    /// procedure divides the pointer message's physical coordinates by the
    /// window's scale before calling in, so hit-testing runs in the same space
    /// as layout. `f32` rather than `i32` because a DIP pointer position is not
    /// an integer — physical 50 at 150% is 33.33 — and truncating it would make
    /// hit-test edges depend on the scale factor for no benefit.
    ///
    /// **Entering on a subtree is now well-defined for geometry.**
    /// `arranged_rect` is absolute DIP within the window's client area
    /// regardless of where the traversal starts, so there is no per-tree
    /// divisor to get wrong — the precondition the pre-T2 Visual-readback
    /// path carried is gone (DD-M4-P2-002 "H2 deletes the row"). What a
    /// subtree entry still does **not** get is the clip bound of any
    /// ancestor *above* the entry point: a clipping container that would
    /// have confined `self`'s subtree in a full-tree walk plays no part
    /// here, because resolution starts at `self`.
    pub fn hit_test_click(&mut self, x: f32, y: f32) {
        let point = DipPoint { x, y };
        let Some(path) = hit::resolve_topmost(self, point) else {
            return;
        };

        // Resolve every node of the dispatch chain to a raw pointer up
        // front, before any handler runs. DD-M4-P2-001 "the ancestor chain
        // is captured when the event is dispatched" (docs/architecture.md
        // §13.2): a handler's state write can rebuild or remove subtrees
        // synchronously (see the safety note on `run_clicked_handlers`
        // below), so resolving an ancestor lazily — after an earlier step
        // has already run user code — could walk into a tree the same
        // event has already invalidated. `hit::dispatch_chain` needs no
        // second tree walk to produce this: every ancestor's path is a
        // prefix of the path `resolve_topmost` already returned.
        let mut chain: Vec<*mut WidgetNode> = Vec::new();
        for node_path in hit::dispatch_chain(&path) {
            // Reborrow `self` fresh for each node_path instead of moving it,
            // so the outer loop can walk down from the root again for the
            // next ancestor.
            let mut node: &mut WidgetNode = &mut *self;
            for index in node_path {
                node = node.children[index].as_mut();
            }
            chain.push(node as *mut WidgetNode);
        }

        for widget_ptr in chain {
            // Safety: every pointer in `chain` was resolved above, before
            // any handler in this dispatch ran, so it is still live the
            // first time the walk reaches it. No "is this node still
            // attached?" check is needed beyond that: the loop returns as
            // soon as a node actually runs something (the `Handlers` arm
            // below), so no ancestor is ever visited after user code has
            // run — a defensive re-check here would guard a branch this
            // code path can never exercise.
            match unsafe { click_disposition_for(widget_ptr) } {
                // Phase 1 `Button.enabled` (DD-M3-P1-005), generalised by
                // §4.19: a disabled Button-family widget still occludes
                // whatever is beneath it — it is the resolved target or an
                // ancestor of it, so it is on the chain regardless — but
                // "the button suppresses click-handler dispatch... [and]
                // does not stop propagation" (docs/dsl_spec.md §4.8), so
                // the walk continues to its ancestor exactly as it would
                // from a node with no handler at all (§4.19 "A disabled
                // Button still occludes... Having run no handler, it also
                // does not end propagation").
                ClickDisposition::Suppressed => continue,
                ClickDisposition::NoHandler => continue,
                ClickDisposition::Handlers(handlers) => {
                    run_clicked_handlers(widget_ptr, handlers);
                    return;
                }
            }
        }
    }

    /// Deliver a `dismiss` request to the node at `path` — the single
    /// recipient the request is addressed to (M4-Phase 2 T7;
    /// `docs/dsl_spec.md` §4.19 "The request is addressed to the innermost
    /// scope and stops there"; `docs/architecture.md` §13.5). The caller
    /// (`focus::dismiss_on_key`) has already resolved which node that is
    /// from `focus_core::FocusTree::esc_target`; this function's only job
    /// is to run whatever handlers that one node carries.
    ///
    /// **Unlike [`Self::hit_test_click`], there is no propagation walk and
    /// no suppression check.** A dismissal request has exactly one
    /// addressee, decided before this call, not discovered by walking
    /// ancestors — `docs/architecture.md` §13.5 "addressed to the
    /// innermost scope... rather than walked to from the focused widget".
    ///
    /// A `path` that does not resolve (the node has already left the
    /// tree) is a silent no-op, mirroring [`Self::node_at_path_mut`]'s
    /// bounds-checked contract.
    ///
    /// **Safety argument, inherited from [`click_disposition_for`] /
    /// [`run_clicked_handlers`] rather than restated.** A `dismiss`
    /// handler's state write drains synchronously, exactly like a
    /// `clicked` handler's, and can remove this very node's subtree while
    /// this call is still running (`run_signal_handlers`'s doc comment).
    /// The snapshot-then-run split below is what that hazard requires:
    /// [`signal_handlers_for`] clones every inline body out of the node
    /// *before* any of them runs, and [`run_signal_handlers`]'s
    /// host-enqueue step only compares `widget_ptr`, never dereferences
    /// it — the same two properties `hit_test_click` /
    /// `run_clicked_handlers` already rest on, applied here to a single
    /// resolved node instead of a captured ancestor chain.
    pub(crate) fn deliver_dismiss_at(&mut self, path: &[usize]) {
        let Some(node) = self.node_at_path_mut(path) else {
            return;
        };
        let widget_ptr = node as *mut WidgetNode;
        // Safety: `widget_ptr` was just resolved from `self`, which is
        // live, and no handler has run yet in this delivery.
        let handlers = unsafe { signal_handlers_for(widget_ptr, "dismiss") };
        run_signal_handlers(widget_ptr, "dismiss", handlers);
    }

    /// Deliver an authored `key-down("<key>")` event, walking from
    /// `start_path` through its ancestors until a handler runs (M4-Phase 2
    /// T8; `docs/dsl_spec.md` §4.19 "Keyboard events start at the focused
    /// widget... and walk the same way" as a pointer event; "the first
    /// matching handler runs and consumes the key"). The caller
    /// (`focus::key_down_on_key`) has already resolved `start_path` — the
    /// focused widget, or the innermost modal focus scope when nothing is
    /// focused — against the same live tree this call runs against, so no
    /// staleness check is needed on `start_path` itself, the same
    /// precondition [`Self::hit_test_click`] relies on for the path
    /// `hit::resolve_topmost` just returned.
    ///
    /// Mirrors [`Self::hit_test_click`]'s two-pass shape: resolve every
    /// node of the dispatch chain to a raw pointer up front, before any
    /// handler runs (a handler's synchronous state write can rebuild or
    /// remove subtrees — see `run_clicked_handlers`'s safety note, which
    /// this inherits rather than restates), then walk front-to-back
    /// looking up `wasamo_ir::signal_key("key-down", Some(key))` — the one
    /// string the IR loader and this dispatcher both compose it from
    /// (DD-M4-P2-005; "Nothing else may compose this string").
    ///
    /// Returns `true` as soon as some node in the chain actually runs a
    /// handler; `false` if the walk reaches the root with nothing run —
    /// the caller reads that as "not consumed" and lets the key continue
    /// toward the host key slot and then `DefWindowProcW`
    /// (`docs/architecture.md` §13.2).
    ///
    /// **No `enabled` suppression arm, deliberately — unlike
    /// [`ClickDisposition::Suppressed`].** `docs/dsl_spec.md` §4.8's
    /// disabled contract is written over *clicks* ("suppresses
    /// click-handler dispatch") and says nothing about `key-down`. The
    /// case is also unreachable from any authored tree today, so a
    /// suppression branch here would be a branch no test could ever fire
    /// (implementation-gates trap #4): a disabled Button is excluded from
    /// Tab-stop enumeration and click landing
    /// (`focus_core::FocusTree::collect_stops` / `focus_landing`, both
    /// gated on `node.enabled`), so it can never become the *focused*
    /// widget and therefore never `start_path`'s target; and a Button can
    /// carry no children at all (the M4-Phase 2 T8 loader gate,
    /// `ir_loader.rs`'s Button-family child rejection), so it can never be
    /// a *non-start* node on any chain either — a Button only ever appears
    /// on a `key-down` chain as the chain's own first (start) entry, never
    /// as an ancestor reached afterwards.
    pub(crate) fn deliver_key_down(&mut self, start_path: &[usize], key: &str) -> bool {
        let signal = wasamo_ir::signal_key("key-down", Some(key));

        // Same two-step split as `hit_test_click`: resolve every chain
        // node to a raw pointer first, before any handler runs.
        let mut chain: Vec<*mut WidgetNode> = Vec::new();
        for node_path in hit::dispatch_chain(start_path) {
            // Reborrow `self` fresh for each node_path instead of moving
            // it, so the outer loop can walk down from the root again for
            // the next ancestor — the same pattern `hit_test_click` uses.
            let mut node: &mut WidgetNode = &mut *self;
            for index in node_path {
                node = node.children[index].as_mut();
            }
            chain.push(node as *mut WidgetNode);
        }

        for widget_ptr in chain {
            // Safety: every pointer in `chain` was resolved above, before
            // any handler in this dispatch ran, so it is still live the
            // first time the walk reaches it — the same argument
            // `hit_test_click` makes for its own chain.
            let handlers = unsafe { signal_handlers_for(widget_ptr, &signal) };
            if handlers.inline.is_empty() && !handlers.has_host_listener {
                continue;
            }
            run_signal_handlers(widget_ptr, &signal, handlers);
            return true;
        }
        false
    }

    /// Update hover/press state as an enter/leave transition against the
    /// single resolved hit-test target (M4-Phase 2 T4; replaces the pre-T4
    /// whole-tree walk per `docs/architecture.md` §13.2 "computed as enter /
    /// leave transitions against the resolved target rather than by a
    /// whole-tree walk"). `down` is true while the left mouse button is
    /// held.
    ///
    /// **`pub(crate)`, narrowed from `pub`.** Grep over `wasamo-runtime`,
    /// `wasamo-dll`, `bindings`, and `examples` shows `update_hover` has no
    /// caller outside `window.rs`'s four pointer arms (the T4 start gate's
    /// scope-re-decision fact 1), and [`HoverState`] is itself crate-only,
    /// so nothing outside this crate could pass the second argument anyway.
    ///
    /// **`(x, y)` are DIP.** See [`Self::hit_test_click`].
    pub(crate) fn update_hover(
        &mut self,
        compositor: &Compositor,
        hover: &mut HoverState,
        x: f32,
        y: f32,
        down: bool,
    ) -> windows::core::Result<()> {
        let resolved = hit::resolve_topmost(self, DipPoint { x, y });

        // The paint target: `resolved` only when it names an *enabled*
        // Button-family node. A disabled Button is still `resolved` — it
        // still occludes, T2's rule and unchanged here — but paints nothing
        // (`docs/dsl_spec.md` §4.8 "Hover / press visual transitions are
        // frozen; the background paints a flat disabled grey directly"), so
        // nothing is retained for it. This is the one and only place
        // `enabled` is consulted by this algorithm; `set_button_state_at`
        // deliberately repeats no such check (see its doc comment).
        let paint_target: Option<Vec<usize>> = match resolved {
            Some(path) => {
                let is_enabled_button = self
                    .node_at_path_mut(&path)
                    .and_then(|node| node.button_data())
                    .is_some_and(|btn| btn.enabled);
                if is_enabled_button {
                    Some(path)
                } else {
                    None
                }
            }
            None => None,
        };

        // Leave, then enter. `hit::hover_leave_target` is `None` exactly
        // when `hover.target == paint_target`, so a mouse move that stays
        // inside the same button's rectangle never leaves-then-re-enters it
        // (see that function's doc comment for why the guard is
        // load-bearing).
        if let Some(path) =
            hit::hover_leave_target(hover.target.as_deref(), paint_target.as_deref())
        {
            self.leave_hover_at(compositor, path)?;
        }
        if let Some(path) = paint_target.as_deref() {
            let entering = if down {
                ButtonState::Pressed
            } else {
                ButtonState::Hovered
            };
            self.set_button_state_at(compositor, path, entering)?;
        }

        // The record and the painted state are written together, in this
        // same function, so [`HoverState`]'s pairing invariant can never be
        // edited apart (trap #3, T4 start gate).
        hover.target = paint_target;
        Ok(())
    }

    /// Reset the retained hover/press target to `Normal` (called on
    /// `WM_MOUSELEAVE`). Same primitive as [`Self::update_hover`]'s leave
    /// half, applied unconditionally.
    pub(crate) fn clear_hover(
        &mut self,
        compositor: &Compositor,
        hover: &mut HoverState,
    ) -> windows::core::Result<()> {
        if let Some(path) = hover.target.as_deref() {
            self.leave_hover_at(compositor, path)?;
        }
        hover.target = None;
        Ok(())
    }

    /// Leave the node at `path`: drive it back to `ButtonState::Normal`
    /// through the shared [`Self::set_button_state_at`] primitive. Shared by
    /// [`Self::update_hover`] and [`Self::clear_hover`] so both write the
    /// leave half identically.
    fn leave_hover_at(
        &mut self,
        compositor: &Compositor,
        path: &[usize],
    ) -> windows::core::Result<()> {
        self.set_button_state_at(compositor, path, ButtonState::Normal)
    }

    /// Descend `path` from `self`, one child index at a time, returning the
    /// node it names or `None` if any index is out of range.
    ///
    /// **Bounds-checked (`children.get_mut`), never indexed
    /// (`children[i]`).** A [`HoverState::target`] path can outlive the node
    /// it named: a click handler's synchronous state write (see
    /// `run_clicked_handlers`'s safety note) can rebuild or remove subtrees
    /// between `hit_test_click` and the `update_hover` call that follows it
    /// in the same `WM_LBUTTONUP` arm (`window.rs`), so a stale path segment
    /// must resolve to `None` rather than panic.
    fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut WidgetNode> {
        let mut node = self;
        for &index in path {
            node = node.children.get_mut(index)?.as_mut();
        }
        Some(node)
    }

    /// The shared state-writing primitive for both enter and leave
    /// transitions (M4-Phase 2 T4). Resolves `path`; if it names a
    /// Button-family node whose current state differs from `new_state`,
    /// starts the same guarded colour-animation transition the pre-T4
    /// whole-tree walk ran per node.
    ///
    /// A stale or non-Button `path` (see [`Self::node_at_path_mut`]) is a
    /// silent no-op, not an error: there is nothing left to paint.
    ///
    /// **No `enabled` check here, deliberately.** `update_button_enabled`
    /// already resets `state` to `Normal` when a Button-family node is
    /// disabled, so a leave that targets a now-disabled node finds
    /// `new_state == btn.state` (both `Normal`) and the guard below already
    /// makes it a no-op — the disabled grey `update_button_enabled` painted
    /// survives untouched. An explicit `enabled` arm here would be a branch
    /// whose two arms are observationally identical to the guard's existing
    /// no-op, which is exactly what implementation-gates trap #4 forbids.
    /// `enabled` is decided exactly once, in `update_hover`'s paint-target
    /// computation.
    fn set_button_state_at(
        &mut self,
        compositor: &Compositor,
        path: &[usize],
        new_state: ButtonState,
    ) -> windows::core::Result<()> {
        let Some(node) = self.node_at_path_mut(path) else {
            return Ok(());
        };
        let Some(btn) = node.button_data_mut() else {
            return Ok(());
        };
        if new_state != btn.state {
            let old_state = btn.state;
            btn.state = new_state;
            let target = effective_button_color(
                btn.style,
                new_state,
                btn.accent,
                true,
                btn.checked,
                btn.focused,
            );
            let ticks = transition_duration(old_state, new_state);
            start_color_anim(compositor, &btn.bg_brush, target, ticks)?;
        }
        Ok(())
    }

    /// Paint (or unpaint) this node's focus indicator (M4-Phase 2 T5;
    /// DD-M4-P2-003 "the indicator is presentation state on the node...
    /// not a new visual written at focus-change time"). Shares
    /// [`Self::node_at_path_mut`]'s bounds-checked descent with
    /// [`Self::set_button_state_at`]: a stale `path` is a silent no-op,
    /// for the reason recorded there.
    ///
    /// **The only caller is `crate::focus::move_focus`.** That is what
    /// keeps `WindowFocus`'s focused id and this painted flag written by
    /// one primitive — the same pairing discipline [`HoverState`] carries
    /// for hover (T4 trap #3), and the shape DD-M4-P2-003 fixes for focus
    /// ("the focus pointer... is written by one primitive").
    ///
    /// **`enabled` is read off the node here, where
    /// [`Self::set_button_state_at`] passes a constant `true`.** That
    /// sibling can afford the constant because `update_button_enabled`
    /// forces `state` back to `Normal` when it disables a node, so its own
    /// guard makes every later hover transition on a disabled node a no-op.
    /// No equivalent invariant holds for `focused`: a node can be disabled
    /// by a binding write while it still holds focus (DD-M4-P2-003's
    /// successor rule applies at the next traversal, not at the write), so
    /// the leave that eventually runs here would animate a disabled node
    /// away from its flat grey if this call claimed it was enabled.
    pub(crate) fn set_button_focused_at(
        &mut self,
        compositor: &Compositor,
        path: &[usize],
        focused: bool,
    ) -> windows::core::Result<()> {
        let Some(node) = self.node_at_path_mut(path) else {
            return Ok(());
        };
        let Some(btn) = node.button_data_mut() else {
            return Ok(());
        };
        if focused != btn.focused {
            btn.focused = focused;
            let target = effective_button_color(
                btn.style,
                btn.state,
                btn.accent,
                btn.enabled,
                btn.checked,
                btn.focused,
            );
            start_color_anim(compositor, &btn.bg_brush, target, FOCUS_TRANSITION_TICKS)?;
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

    pub fn append_child(&mut self, child: Box<WidgetNode>) -> windows::core::Result<()> {
        self.insert_child_inner(self.children.len(), child, None)
            .map_err(mutation_error_to_winerr)
    }

    // ── Tree-mutation primitives (DD-M2-P4-001/002 = Option A) ───────────────

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    #[doc(hidden)]
    pub fn __text_content_for_test(&self) -> Option<&str> {
        match &self.data {
            WidgetData::Text { content, .. } => Some(content.as_str()),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn __button_enabled_for_test(&self) -> Option<bool> {
        match &self.data {
            WidgetData::Button(button) | WidgetData::ToggleButton(button) => Some(button.enabled),
            _ => None,
        }
    }

    /// Test-only accessor for this node's transient hover/press paint state
    /// (M4-Phase 2 T4). `None` for non-Button-family widgets.
    /// `ButtonState` stays private to this module — a `&'static str` is
    /// returned instead of widening its visibility, so a cross-crate
    /// integration test can assert the painted state without this crate
    /// exporting `ButtonState`.
    #[doc(hidden)]
    pub fn __button_state_for_test(&self) -> Option<&'static str> {
        self.button_data().map(|btn| match btn.state {
            ButtonState::Normal => "normal",
            ButtonState::Hovered => "hovered",
            ButtonState::Pressed => "pressed",
        })
    }

    /// Test-only accessor for this node's painted focus indicator
    /// (M4-Phase 2 T5). `None` for non-Button-family widgets, matching
    /// [`Self::__button_state_for_test`]'s shape.
    #[doc(hidden)]
    pub fn __button_focused_for_test(&self) -> Option<bool> {
        self.button_data().map(|btn| btn.focused)
    }

    /// Test-only accessor for this node's [`focus_role`](Self::focus_role)
    /// (M4-Phase 2 T6). `focus_core::FocusRole` is not exported from this
    /// crate, so — matching [`Self::__button_state_for_test`]'s shape — a
    /// `&'static str` is returned instead of widening a private type's
    /// visibility: `"stop"`, `"container"`, `"group"`, or `"modal-scope"`.
    #[doc(hidden)]
    pub fn __focus_role_for_test(&self) -> &'static str {
        use crate::focus_core::FocusRole;
        match self.focus_role().0 {
            FocusRole::Stop => "stop",
            FocusRole::Container => "container",
            FocusRole::Group => "group",
            FocusRole::ModalScope => "modal-scope",
            FocusRole::ActiveItemList | FocusRole::ActiveItem => {
                unreachable!("focus_role never returns ActiveItemList/ActiveItem in M4-Phase 2")
            }
        }
    }

    #[doc(hidden)]
    pub fn __togglebutton_checked_for_test(&self) -> Option<bool> {
        match &self.data {
            WidgetData::ToggleButton(button) => Some(button.checked),
            _ => None,
        }
    }

    pub(crate) fn is_zstack(&self) -> bool {
        matches!(self.data, WidgetData::ZStack)
    }

    pub(crate) fn is_grid(&self) -> bool {
        matches!(self.data, WidgetData::Grid { .. })
    }

    /// Restates, as a match, what three widget constructors already state by
    /// calling `outer_visual.SetClip(&clip)`: `ScrollView`, `Grid`, and
    /// `ZStack` each install a zero-inset `InsetClip` at construction, and no
    /// other kind does. The two facts cannot be made atomic — one lives in a
    /// match here, the other in three separate constructor bodies — so the
    /// agreement between them is pinned by an integration test
    /// (`wasamo-runtime/tests/arranged_rect_integration.rs`) that compares
    /// this predicate against the live `Visual.Clip()` for one instance of
    /// every `WidgetData` kind.
    ///
    /// The match is exhaustive with no `_` wildcard arm: a new `WidgetData`
    /// variant fails to compile here until it is explicitly classified as
    /// clipping or not.
    fn clips_children(&self) -> bool {
        match &self.data {
            WidgetData::ScrollView { .. } | WidgetData::Grid { .. } | WidgetData::ZStack => true,
            WidgetData::Rectangle
            | WidgetData::VStack { .. }
            | WidgetData::HStack { .. }
            | WidgetData::Text { .. }
            | WidgetData::Button(_)
            | WidgetData::ToggleButton(_)
            | WidgetData::Box { .. }
            | WidgetData::WrapPanel { .. } => false,
        }
    }

    /// Test-only accessor for `arranged_rect` (DD-M4-P2-002). `None` until a
    /// layout pass has run over this node at least once.
    #[doc(hidden)]
    pub fn __arranged_rect_for_test(&self) -> Option<DipRect> {
        self.arranged_rect
    }

    /// Test-only accessor for `clips_children`.
    ///
    /// `hit::resolve_topmost` (via `HitTree::hit_clips_children`) is the
    /// predicate's production caller as of T2, so this accessor is no
    /// longer the predicate's *only* entry point — it is retained as the
    /// evidence seam for the eight non-`ScrollView`/`Grid`/`ZStack` widget
    /// kinds, which cannot have children and therefore give hit resolution
    /// no way to exercise their (non-clipping) arm: no production path
    /// drives a click through a childless widget's clip predicate, so
    /// `the_clip_predicate_agrees_with_the_live_visual_for_every_widget_kind`
    /// (T1) still needs a direct reader to keep pinning the per-kind
    /// agreement with the live `Visual.Clip()` for those eight kinds.
    #[doc(hidden)]
    pub fn __clips_children_for_test(&self) -> bool {
        self.clips_children()
    }

    /// Test-only accessor naming this node's `WidgetData` kind.
    ///
    /// Exists so an integration test can assert *which* kind an instance
    /// actually is, rather than trusting the position it was constructed at
    /// or pulled from a tree by (e.g. after a sequence of `remove_child(0)`
    /// calls whose ordering assumption could silently drift). The match is
    /// exhaustive with no `_` wildcard arm, so a new `WidgetData` variant
    /// fails to compile here until it is given a name.
    #[doc(hidden)]
    pub fn __kind_name_for_test(&self) -> &'static str {
        match &self.data {
            WidgetData::Rectangle => "Rectangle",
            WidgetData::VStack { .. } => "VStack",
            WidgetData::HStack { .. } => "HStack",
            WidgetData::Text { .. } => "Text",
            WidgetData::Button(_) => "Button",
            WidgetData::ToggleButton(_) => "ToggleButton",
            WidgetData::Box { .. } => "Box",
            WidgetData::WrapPanel { .. } => "WrapPanel",
            WidgetData::ScrollView { .. } => "ScrollView",
            WidgetData::Grid { .. } => "Grid",
            WidgetData::ZStack => "ZStack",
        }
    }

    pub fn insert_child(
        &mut self,
        index: usize,
        child: Box<WidgetNode>,
    ) -> Result<(), MutationError> {
        self.insert_child_inner(index, child, None)
    }

    pub(crate) fn insert_child_with_slot_data(
        &mut self,
        index: usize,
        child: Box<WidgetNode>,
        slot_data: Option<SlotData>,
    ) -> Result<(), MutationError> {
        self.insert_child_inner(index, child, slot_data)
    }

    fn insert_child_inner(
        &mut self,
        index: usize,
        mut child: Box<WidgetNode>,
        slot_data: Option<SlotData>,
    ) -> Result<(), MutationError> {
        if index > self.children.len() {
            return Err(MutationError::IndexOutOfBounds);
        }
        if child.attached {
            return Err(MutationError::AlreadyAttached);
        }
        use windows::core::Interface;
        let parent_container: ContainerVisual = self
            .content_container_visual()
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let child_visual: Visual = child
            .visual
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let children_col = parent_container
            .Children()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        if index == self.children.len() {
            children_col
                .InsertAtTop(&child_visual)
                .map_err(|_| MutationError::IndexOutOfBounds)?;
        } else {
            let sibling_visual: Visual = self.children[index]
                .visual
                .cast()
                .map_err(|_| MutationError::IndexOutOfBounds)?;
            children_col
                .InsertBelow(&child_visual, &sibling_visual)
                .map_err(|_| MutationError::IndexOutOfBounds)?;
        }
        child.attached = true;
        self.children
            .insert(index, ChildSlot::new(child, slot_data));
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
            .content_container_visual()
            .cast()
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        parent_container
            .Children()
            .and_then(|c| c.Remove(&child_visual))
            .map_err(|_| MutationError::IndexOutOfBounds)?;
        let mut removed = self.children.remove(index).into_node();
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
        let replacement_slot_data = self.children[index].slot_data;
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
            .content_container_visual()
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
        let mut old = std::mem::replace(
            &mut self.children[index],
            ChildSlot::new(new_child, replacement_slot_data),
        )
        .into_node();
        old.attached = false;
        Ok(old)
    }

    // ── Window-scale preparation ─────────────────────────────────────────────

    /// Refresh text surfaces whose last successful rasterization differs from
    /// `target`.
    ///
    /// Raster freshness is independent of the geometry cache. Each node's
    /// marker advances only after its replacement brush is installed, so a
    /// partial WinRT failure leaves precisely the unfinished suffix retryable
    /// without holding back layout or target-scale geometry.
    pub(crate) fn refresh_text_surfaces_recursive(
        &mut self,
        compositor: &Compositor,
        renderer: &TextRenderer,
        target: DipScale,
    ) -> windows::core::Result<()> {
        if self.raster_scale != target {
            match &self.data {
                WidgetData::Text { content, style } => {
                    let (width, height) =
                        fixed_extent(&self.width, &self.height).ok_or_else(|| {
                            windows::core::Error::new(
                                windows::core::HRESULT(0x8000FFFF_u32 as i32),
                                "Text node does not retain a fixed DIP extent",
                            )
                        })?;
                    let surface = renderer.draw_text_at_dpi(
                        content,
                        *style,
                        width,
                        height,
                        Color {
                            A: 255,
                            R: 255,
                            G: 255,
                            B: 255,
                        },
                        target.dpi(),
                    )?;
                    let brush = create_text_surface_brush(compositor, &surface)?;
                    self.visual.SetBrush(&brush)?;
                }
                WidgetData::Button(button) | WidgetData::ToggleButton(button) => {
                    let (width, height) = button.label_size;
                    let surface = renderer.draw_text_at_dpi(
                        &button.label_text,
                        button.label_style,
                        width,
                        height,
                        Color {
                            A: 255,
                            R: 255,
                            G: 255,
                            B: 255,
                        },
                        target.dpi(),
                    )?;
                    let brush = create_text_surface_brush(compositor, &surface)?;
                    button.label_visual.SetBrush(&brush)?;
                }
                WidgetData::Rectangle
                | WidgetData::VStack { .. }
                | WidgetData::HStack { .. }
                | WidgetData::Box { .. }
                | WidgetData::WrapPanel { .. }
                | WidgetData::ScrollView { .. }
                | WidgetData::Grid { .. }
                | WidgetData::ZStack => {}
            }
            self.raster_scale = target;
        }

        for child in &mut self.children {
            child.refresh_text_surfaces_recursive(compositor, renderer, target)?;
        }
        Ok(())
    }

    pub(crate) fn commit_scale_recursive(&mut self, target: DipScale) {
        self.scale = target;
        for child in &mut self.children {
            child.commit_scale_recursive(target);
        }
    }

    // ── Layout ────────────────────────────────────────────────────────────────

    /// Builds a LayoutNode tree, runs layout, then syncs results back to SpriteVisuals.
    ///
    /// M3-Phase 2 T8: `layout::run_layout` is fallible — it surfaces
    /// `LayoutError::BoxAspectUnboundedBoth` / `BoxNoExtent` from
    /// DD-M3-P2-005. We translate those into `windows::core::Error` so
    /// the existing window layout call sites (which already swallow the
    /// Result with `let _ = …`) keep their current shape. A dedicated C ABI
    /// surface for layout-time runtime errors
    /// is out of Phase 2 scope and tracked alongside the ABI work in
    /// later phases.
    pub fn run_layout(&mut self, window_w: f32, window_h: f32) -> windows::core::Result<()> {
        let target = self.scale;
        self.run_layout_at_scale(window_w, window_h, target)?;
        let runtime = crate::runtime::get();
        self.refresh_text_surfaces_recursive(&runtime.compositor, &runtime.text_renderer, target)
    }

    fn run_layout_at_scale(
        &mut self,
        window_w: f32,
        window_h: f32,
        target: DipScale,
    ) -> windows::core::Result<()> {
        let mut layout_tree = self.build_layout_tree();
        layout::run_layout(&mut layout_tree, window_w, window_h).map_err(layout_error_to_winerr)?;
        self.sync_visuals(&layout_tree, (0.0, 0.0), target)?;
        self.commit_scale_recursive(target);
        Ok(())
    }

    /// Layout entry for the **window-root** WidgetNode (the one
    /// `window.rs::set_root` attaches as the topmost child of
    /// `state.root` and re-lays out on `WM_SIZE`). Forces the root
    /// LayoutNode's `width` / `height` to `Fill` before delegating to
    /// `layout::run_layout`, so the window client rect determines the
    /// root viewport regardless of the root container's declared
    /// sizing constraints.
    ///
    /// M3-Phase 4 T6 fix. Without this override, a root container with
    /// `height: Shrink` (the default for `VStack`; DSL-authored `.ui`
    /// cannot set width/height since `dsl_spec.md` §4 does not yet
    /// expose those attributes) holding a `height: Fill` child (e.g.
    /// `ScrollView`) collapses the Fill child to zero via the
    /// convention pinned by
    /// `layout::tests::degenerate_fill_in_shrink_parent_clamps_to_zero`,
    /// making the ScrollView's outer Visual size `(w, 0)` and clipping
    /// its content to a zero-height rect. Phase 2 / Phase 3 examples
    /// (counter, bool-demo) implicitly relied on the "window client
    /// rect determines root viewport" contract because their root
    /// containers had no Fill children; the gallery sub-screen is the
    /// first `.ui` to surface the latent collapse.
    ///
    /// The plain [`Self::run_layout`] keeps its current semantics so
    /// existing mock-free integration tests that drive `WidgetNode`s
    /// directly (e.g. `tests/wrap_panel_layout_integration.rs`)
    /// continue to exercise the declared sizing constraints. See
    /// `m3-phase-4-progress.md` Decisions log "T6 smoke failure mode A
    /// disposition (2026-05-25)" for the observation that drove this
    /// split.
    pub fn run_layout_as_window_root(
        &mut self,
        window_w: f32,
        window_h: f32,
    ) -> windows::core::Result<()> {
        let target = self.scale;
        self.run_layout_as_window_root_at_scale(window_w, window_h, target)?;
        let runtime = crate::runtime::get();
        self.refresh_text_surfaces_recursive(&runtime.compositor, &runtime.text_renderer, target)
    }

    /// Run window-root geometry at an authoritative window scale.
    ///
    /// This entry deliberately does not refresh text. Window callers compose
    /// fallible geometry projection followed by an infallible cache commit
    /// with the independent, fallible raster pass. T7 can therefore defer the
    /// latter out of the nested `WM_SIZE` while preserving
    /// DD-M4-P1-003's fixed ordering.
    pub(crate) fn run_layout_as_window_root_at_scale(
        &mut self,
        window_w: f32,
        window_h: f32,
        target: DipScale,
    ) -> windows::core::Result<()> {
        let mut layout_tree = self.build_layout_tree();
        layout_tree.width = SizeConstraint::Fill;
        layout_tree.height = SizeConstraint::Fill;
        layout::run_layout(&mut layout_tree, window_w, window_h).map_err(layout_error_to_winerr)?;
        self.sync_visuals(&layout_tree, (0.0, 0.0), target)?;
        self.commit_scale_recursive(target);
        Ok(())
    }

    /// Exercise authoritative window-scale geometry without refreshing text.
    ///
    /// This test seam keeps the private `DipScale` carrier private while a
    /// mock-free integration test verifies that geometry cache and raster
    /// freshness can diverge and subsequently reconcile.
    #[doc(hidden)]
    pub fn __run_layout_as_window_root_at_dpi_for_test(
        &mut self,
        window_w: f32,
        window_h: f32,
        dpi: u32,
    ) -> windows::core::Result<()> {
        self.run_layout_as_window_root_at_scale(window_w, window_h, DipScale::from_dpi(dpi))
    }

    /// Leave **this one node's** cached geometry scale behind the window's,
    /// without touching its children, its Visual geometry or its raster
    /// marker (M4-Phase 1 T8; T5 finding F-37).
    ///
    /// **Pre-M4-Phase 2 T2, the property under test was that a hit-test
    /// traversal divided every `visual_rect` readback by the traversal
    /// root's scale rather than by each node's own**, so a descendant whose
    /// cache was stale still resolved to the rectangle it was actually
    /// composited at. T2 removed that reader entirely: hit-testing now reads
    /// `arranged_rect` — absolute DIP, written once by `sync_visuals` — and
    /// never touches `self.scale` or the Visual at all, so there is no
    /// divisor left to be one or many of. The seam and the test that drives
    /// it (`dpi_scale_matrix_integration.rs::
    /// a_stale_descendant_scale_still_hit_tests_where_the_widget_is`) stay,
    /// re-purposed: they now demonstrate that poking this node-local field
    /// has **no** effect on hit resolution, which is the stronger
    /// (structural, not just numerical) form of the same guarantee.
    ///
    /// Constructing a stale-`scale` node still needs this seam rather than a
    /// legitimate path: `commit_scale_recursive` writes the whole subtree,
    /// and the incremental attach paths F-32 enumerated leave a fresh node
    /// with no geometry to hit-test at all.
    ///
    /// **This is not a production entry and must not become one.** It writes
    /// the derived copy without the projection that owns it, which is exactly
    /// the drift trap #3 exists to prevent — deliberately, because the test's
    /// subject is what survives that drift. A `u32` DPI keeps `DipScale`
    /// crate-private, in the same shape as the entry above.
    #[doc(hidden)]
    pub fn __set_geometry_scale_dpi_for_test(&mut self, dpi: u32) {
        self.scale = DipScale::from_dpi(dpi);
    }

    fn build_layout_child_slots(&self) -> ChildSlots {
        self.children
            .iter()
            .map(|slot| LayoutChildSlot::new(slot.build_layout_tree(), slot.slot_data))
            .collect::<Vec<_>>()
            .into()
    }

    fn build_layout_tree(&self) -> LayoutNode {
        match &self.data {
            WidgetData::Rectangle
            | WidgetData::Text { .. }
            | WidgetData::Button(_)
            | WidgetData::ToggleButton(_) => {
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
                node.children = self.build_layout_child_slots();
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
                node.children = self.build_layout_child_slots();
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
                node.children = self.build_layout_child_slots();
                node
            }
            // M3-Phase 3 T5: thread the WrapPanel attribute set into the
            // pure-logic layout engine. The conversion from `i32` (the
            // DSL surface storage type per DD-M3-P3-003 / DD-M3-P3-004)
            // to `f32` (the layout engine's numeric domain) lives at this
            // build boundary — the same shape as how VStack / HStack
            // hand `spacing` / `padding` to `LayoutNode` (loader-side
            // i32-to-f32 cast at `ir_loader::construct_widget`). The
            // measure-arrange itself lands in T7.
            WidgetData::WrapPanel {
                item_cross_size,
                item_spacing,
                line_spacing,
            } => {
                let mut node = LayoutNode::wrap_panel(
                    item_cross_size.map(|v| v as f32),
                    *item_spacing as f32,
                    *line_spacing as f32,
                );
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self.build_layout_child_slots();
                node
            }
            // M3-Phase 4 T3: thread the ScrollView `offset_y` into the
            // pure-logic layout engine. The `i32` DSL surface storage
            // (DD-M3-P4-003) is handed to `LayoutNode::scroll_view` here
            // unchanged; `arrange_scroll_view` (T2) promotes it to `f32`
            // for clamp arithmetic per the rounding contract.
            WidgetData::ScrollView { offset_y, .. } => {
                let mut node = LayoutNode::scroll_view(*offset_y);
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self.build_layout_child_slots();
                node
            }
            // M3-Phase 5 T3 / M3-Phase 7b T3: thread the Grid track lists
            // and child-slot placement into the pure-logic layout engine.
            // `arrange_grid` writes each content child's resolved offset /
            // size directly onto its `LayoutNode`, read back by
            // `sync_visuals`.
            WidgetData::Grid { columns, rows } => {
                let mut node = LayoutNode::grid(columns.clone(), rows.clone());
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self.build_layout_child_slots();
                node
            }
            WidgetData::ZStack => {
                let mut node = LayoutNode::zstack();
                node.width = self.width.clone();
                node.height = self.height.clone();
                node.children = self.build_layout_child_slots();
                node
            }
        }
    }

    // `computed.offset` is the absolute offset the layout engine assigns in
    // `arrange` (cumulative through parents). The WinRT Composition
    // `Visual.Offset` is parent-relative, so the parent's absolute offset
    // is subtracted here before writing. Before M3-Phase 3 the Phase 2
    // gallery sub-screen had its single Box rooted at the Window origin
    // (parent absolute offset `(0, 0)`), so the absolute-as-relative write
    // happened to render correctly; WrapPanel-arranged Boxes have non-zero
    // offsets and exposed the latent bug as visibly mis-placed Text
    // labels — see the M3-Phase 3 T9 step-end retrospective.
    //
    // DD-M4-P2-002 "one walk, two stores": this walk now writes two things
    // per node — the physical Composition geometry below, and (after it)
    // `self.arranged_rect`, the same `computed` value retained as absolute
    // DIP on the node itself. A node-side field write is not a Composition
    // geometry write, so the single-pass property DD-M4-P1-002's audit
    // closed — every geometry write happens once, in this walk — is
    // preserved; the second store rides the same traversal rather than
    // adding one.
    fn sync_visuals(
        &mut self,
        computed: &LayoutNode,
        parent_abs_offset: (f32, f32),
        target: DipScale,
    ) -> windows::core::Result<()> {
        use windows::core::Interface;
        let visual: Visual = self.visual.cast()?;
        // DD-M4-P1-002 audit row 4, the outbound seam. `computed` is DIP and
        // the Composition visual tree is physical, so every write below
        // multiplies by the authoritative window target — through the named
        // operations, not
        // by hand: `dip * scale.factor()` satisfies a prose reading of the rule
        // and is wrong only at non-dyadic scales, where two of the phase's
        // three test factors cannot see it (T2 findings F-13 / F-15).
        //
        // `relative_offset_to_physical` is what makes **convert once, on the
        // difference** the natural call rather than a discipline: it is handed
        // the two absolute DIP positions, so subtracting in DIP and multiplying
        // the result — one rounding instead of two — is the only thing a caller
        // can express without converting each operand itself.
        let (offset_x, offset_y) =
            target.relative_offset_to_physical(computed.offset, parent_abs_offset);
        let (size_x, size_y) = target.extent_to_physical(computed.size);
        visual.SetOffset(Vector3 {
            X: offset_x,
            Y: offset_y,
            Z: 0.0,
        })?;
        visual.SetSize(Vector2 {
            X: size_x,
            Y: size_y,
        })?;
        // DD-M4-P2-002 second store: `computed.offset` retained **un-subtracted**.
        // The Composition write above needs a parent-relative physical offset
        // (hence the subtraction into `offset_x` / `offset_y`), while this node
        // store needs the absolute DIP one — and both come from the same
        // `computed.offset` value in the same pass, which is what makes them
        // lockstep rather than two independent derivations that could drift.
        //
        // The ordering is deliberate: this store runs after the two Composition
        // writes above have already succeeded (the `?`s did not early-return),
        // so "this node has a rectangle" implies its Visual was written from the
        // same layout result. A pass that fails part-way through the tree
        // therefore leaves the already-visited nodes consistent between the two
        // stores, rather than a node claiming a rectangle for a Visual that was
        // never actually moved.
        self.arranged_rect = Some(DipRect {
            x: computed.offset.0,
            y: computed.offset.1,
            width: computed.size.0,
            height: computed.size.1,
        });
        // DD-M4-P1-002 §The conversion sites row 6: the Button-family
        // label's placement is written here rather than at construction,
        // where no scale factor exists. Like the ScrollView intermediate
        // Visual below, the label Visual is not a child `WidgetNode` — it
        // lives in `ButtonData.label_visual` — so it is reached through a
        // per-kind arm rather than the `children` / `computed.children` zip.
        //
        // The offset is a constant inside the background Visual, and the
        // size is the label's measured extent, not `computed.size`: a parent
        // that stretches the button changes the arranged size but not the
        // extent of the text drawn into the label surface.
        //
        // Audit row 6. **The offset here is already parent-relative**, so it is
        // not the difference-taking case row 4 is: there is no absolute pair to
        // subtract, and `relative_offset_to_physical` cannot be applied without
        // inventing one (T3 finding F-19). Each component goes through the
        // scalar `to_physical`, which is one multiplication and therefore
        // exactly the one rounding the rule asks for.
        if let WidgetData::Button(btn) | WidgetData::ToggleButton(btn) = &self.data {
            let label_visual: Visual = btn.label_visual.cast()?;
            label_visual.SetOffset(Vector3 {
                X: target.to_physical(BUTTON_PAD_H),
                Y: target.to_physical(BUTTON_PAD_V),
                Z: 0.0,
            })?;
            let (label_w, label_h) = target.extent_to_physical(btn.label_size);
            label_visual.SetSize(Vector2 {
                X: label_w,
                Y: label_h,
            })?;
        }
        // DD-M3-P4-004 Option A: when self is a ScrollView, the
        // intermediate content Visual carries the scroll translation
        // `Visual.Offset = (0, -applied_y, 0)` (T2's clamped
        // `applied_offset_y` cache). The child widget Visual sits
        // beneath the intermediate, so its sync_visuals parent_abs
        // is the intermediate's absolute offset
        // `(computed.offset.0, computed.offset.1 - applied_y)` — the
        // layout engine arranged the child at `(x, y - applied)`
        // absolute (per `arrange_scroll_view`), so the child
        // Visual.Offset resolves to (0, 0) parent-relative and the
        // scroll position is contributed exactly once by the
        // intermediate Visual. The intermediate Visual itself carries
        // no clip (the outer Visual's InsetClip clips the translated
        // content); its size mirrors the viewport for hit-testing
        // consistency with the outer Visual.
        //
        // Audit row 5, and the same already-parent-relative case as row 6:
        // `(0, -applied_y)` is the intermediate's offset inside its own parent,
        // so each component takes the scalar `to_physical`. **The recursion
        // itself stays entirely in DIP** — `child_parent_abs` below is the
        // value the layout engine arranged against, and only the two
        // Composition writes above it multiply.
        let child_parent_abs = if let WidgetData::ScrollView { content_visual, .. } = &self.data {
            let applied = computed.applied_offset_y.get();
            let int_visual: Visual = content_visual.cast()?;
            int_visual.SetOffset(Vector3 {
                X: target.to_physical(0.0),
                Y: target.to_physical(-applied),
                Z: 0.0,
            })?;
            let (int_w, int_h) = target.extent_to_physical(computed.size);
            int_visual.SetSize(Vector2 { X: int_w, Y: int_h })?;
            (computed.offset.0, computed.offset.1 - applied)
        } else {
            computed.offset
        };
        // T1 re-audit / T2 carry-forward: `self.children` and
        // `computed.children` are built 1:1 by `build_layout_child_slots` in
        // the same pass, so a length mismatch is not currently reachable —
        // but `zip` would absorb a truncation silently, leaving a trailing
        // `WidgetNode` with no `arranged_rect` written this pass. Since T2
        // makes that field the hit-testing source, the silent failure mode
        // is a widget that is present but permanently unhittable rather than
        // a panic, which is worse than a debug-time assertion.
        debug_assert_eq!(
            self.children.len(),
            computed.children.len(),
            "sync_visuals: WidgetNode/LayoutNode child count mismatch — zip would silently \
             leave a trailing WidgetNode with no arranged_rect, i.e. a silently unhittable widget"
        );
        for (child, child_computed) in self.children.iter_mut().zip(computed.children.iter()) {
            child.sync_visuals(child_computed, child_parent_abs, target)?;
        }
        Ok(())
    }
}

// ── Click dispatch (M4-Phase 2 T3) ──────────────────────────────────────────

/// What `hit_test_click` should do at one node of the captured dispatch
/// chain, decided by [`click_disposition_for`] with no side effects.
///
/// **One snapshot, read at both the consumption check and the invocation
/// site.** `hit_test_click` branches on this enum to decide whether to
/// continue to the ancestor, and the `Handlers` arm carries everything
/// `run_clicked_handlers` needs to actually run them. If those two
/// decisions — "does this node consume the event?" and "what do I run?" —
/// were derived independently from the node's live state instead of from
/// one snapshot, they could drift: a node could consume the event without
/// running anything, or run something without having been counted as
/// consuming it.
enum ClickDisposition {
    /// A Button-family widget with `enabled == false`. It is on the
    /// dispatch chain only because it was the hit-test target or an
    /// ancestor of it, and it occludes as usual, but it runs nothing and
    /// does not consume — the walk continues to its ancestor
    /// (docs/dsl_spec.md §4.8, §4.19).
    Suppressed,
    /// None of the three `clicked` producers (native closure, inline DSL
    /// handler, host signal listener) is present on this node. Runs
    /// nothing and does not consume.
    NoHandler,
    /// At least one producer is present. Running any of them consumes the
    /// event (DD-M4-P2-001 "a handler that runs consumes the event").
    Handlers(ClickedHandlers),
}

/// The `clicked` producers found on one node, snapshotted before any of
/// them runs.
///
/// **Non-empty by construction.** [`click_disposition_for`] returns
/// [`ClickDisposition::NoHandler`] instead of an empty `ClickedHandlers`
/// when no producer is present, so every value `run_clicked_handlers`
/// receives has at least one thing to do.
struct ClickedHandlers {
    /// `ButtonData::clicked_fn` is present (Button-family only — the
    /// Rust-native `WidgetNode::set_clicked` API). `clicked` is the only
    /// signal with a native producer, so this flag has no counterpart in
    /// [`SignalHandlers`] — see that type's own doc comment.
    has_native: bool,
    /// The inline-plus-host half: DSL inline `clicked` bodies and whether
    /// a host listener is connected. Shared shape with `dismiss`
    /// (M4-Phase 2 T7) through [`signal_handlers_for`] /
    /// [`run_signal_handlers`].
    signal: SignalHandlers,
}

/// Read-only: what would happen if the click walk reached `widget_ptr`,
/// without running anything. `hit_test_click` calls this once per node of
/// its captured dispatch chain, in walk order, strictly before any earlier
/// node in that chain has run a handler — which is what makes it sound to
/// dereference `widget_ptr` here even though the whole chain was captured
/// before the walk began: nothing has had a chance to invalidate it yet.
///
/// # Safety
/// `widget_ptr` must point to a still-live `WidgetNode`.
unsafe fn click_disposition_for(widget_ptr: *mut WidgetNode) -> ClickDisposition {
    // Safety: see the function doc comment; the caller's dispatch-order
    // invariant (no handler has run yet) is what makes this sound.
    let node = unsafe { &*widget_ptr };

    // Suppression is checked ahead of the producer scan: a disabled
    // Button-family node dispatches nothing even if it happens to carry a
    // native, inline, or host handler (docs/dsl_spec.md §4.8 "the button
    // suppresses click-handler dispatch"), so its own handlers must never
    // reach `ClickDisposition::Handlers`.
    if let Some(false) = node.button_data().map(|btn| btn.enabled) {
        return ClickDisposition::Suppressed;
    }

    let has_native = node
        .button_data()
        .is_some_and(|btn| btn.clicked_fn.is_some());
    // Safety: see `signal_handlers_for`'s doc comment — `widget_ptr` is
    // still live and no handler in this dispatch chain has run yet, the
    // same two facts this function's own doc comment already establishes
    // for `node` above.
    let signal = unsafe { signal_handlers_for(widget_ptr, "clicked") };

    if has_native || !signal.inline.is_empty() || signal.has_host_listener {
        ClickDisposition::Handlers(ClickedHandlers { has_native, signal })
    } else {
        ClickDisposition::NoHandler
    }
}

/// The handlers found for one named signal at one node, snapshotted before
/// any of them runs (M4-Phase 2 T7) — the inline-plus-host half of what
/// [`ClickedHandlers`] carries for `clicked`, generalised so `dismiss` and
/// `key-down("<key>")` (`docs/dsl_spec.md` §4.19) can share it rather than
/// each growing a parallel handler dispatcher (implementation-gates trap
/// #3: several handler-running code paths could drift on ordering, or on
/// the synchronous-rebuild hazard [`run_signal_handlers`]'s doc comment
/// records).
///
/// **No native-closure flag here**, unlike `ClickedHandlers`: `clicked` is
/// the only signal with a Rust-native producer (`ButtonData::clicked_fn`),
/// so `run_clicked_handlers` keeps that step in its own body and hands
/// only this inline-plus-host half to [`run_signal_handlers`]. Neither
/// `dismiss` nor `key-down` has a native producer at all — every recipient
/// of this type either has no native step
/// ([`WidgetNode::deliver_dismiss_at`], [`WidgetNode::deliver_key_down`])
/// or runs it separately (`run_clicked_handlers`).
struct SignalHandlers {
    /// DSL inline handler bodies for this signal name, already cloned out
    /// of `WidgetNode::inline_handlers` — any widget kind can carry these.
    inline: Vec<HandlerExpr>,
    /// `crate::registry::signal_tokens_for(widget_ptr, signal)` was
    /// non-empty: a host listener is connected via
    /// `wasamo_signal_connect` — any widget kind.
    has_host_listener: bool,
}

/// Read-only: what would run for `signal` at `widget_ptr`, without running
/// anything. Mirrors [`click_disposition_for`]'s soundness argument,
/// narrowed to the inline/host half that argument is actually about.
///
/// # Safety
/// `widget_ptr` must point to a still-live `WidgetNode`, and this must be
/// called before any handler has run in this delivery — the same
/// dispatch-order invariant `click_disposition_for`'s callers already
/// guarantee (`hit_test_click`'s pre-captured chain, and
/// [`WidgetNode::deliver_key_down`]'s, which is that chain's shape applied
/// to a key) and [`WidgetNode::deliver_dismiss_at`] guarantees by
/// construction (it resolves `widget_ptr` itself, immediately before
/// calling this).
unsafe fn signal_handlers_for(widget_ptr: *mut WidgetNode, signal: &str) -> SignalHandlers {
    // Safety: see the function doc comment.
    let node = unsafe { &*widget_ptr };
    let inline: Vec<HandlerExpr> = node
        .inline_handlers
        .iter()
        .filter(|(sig, _)| sig == signal)
        .map(|(_, expr)| expr.clone())
        .collect();
    let has_host_listener = !crate::registry::signal_tokens_for(widget_ptr, signal).is_empty();
    SignalHandlers {
        inline,
        has_host_listener,
    }
}

/// Run `handlers`' inline bodies, then enqueue the host listener — the
/// inline-plus-host half of the DD-M2-P3-002 Option B order
/// (`run_clicked_handlers` runs the native-closure step, when there is
/// one, before calling this).
///
/// **Why this is safe despite what user code can do to the node.** A
/// handler's state write (`reactive::Signal::set`) drains its reactive
/// effects **synchronously**, at zero batch depth — nothing in this call
/// wraps the dispatch in `with_batched_writes` — so a conditional or `for`
/// effect can rebuild or remove subtrees, including this very node's,
/// *while this function is still running* (the same hazard
/// `run_clicked_handlers` was already built around; this is that
/// function's argument, factored out rather than restated). The two
/// properties it rests on hold here too:
///
/// 1. `handlers.inline` was already cloned out of the node by
///    [`signal_handlers_for`], before any handler ran, so evaluating it
///    here touches no widget memory at all.
/// 2. `enqueue_signal` takes `widget_ptr` **only to compare it** against
///    registry entries (`std::ptr::eq`) — it never dereferences it. A node
///    whose subtree was removed by an earlier step in this same function
///    has already had its registrations severed (`widget_destroy` calls
///    `registry::remove_for_widget`), so the comparison simply finds no
///    tokens and enqueues nothing; it does not need `widget_ptr` to still
///    point at live memory.
///
/// **Unlike `run_clicked_handlers`, this function itself never
/// dereferences `widget_ptr`** — the native-closure step is `clicked`-only
/// and stays in that caller, so there is no "last dereference" here to
/// call out.
fn run_signal_handlers(widget_ptr: *mut WidgetNode, signal: &str, handlers: SignalHandlers) {
    // DD-M2-P6-006: dispatch handler bodies against the SignalRegistry
    // installed by the IR loader. If no registry is active (e.g. tests
    // building widgets directly) fall back to a no-op context — the
    // evaluator runs but property reads/writes report "unknown property".
    let registry = crate::reactive::active_registry();
    let location = format!("?.{signal}");
    for expr in &handlers.inline {
        // DD-M2-P3-003: catch_unwind wrapper logs errors and continues the
        // event loop; location is a coarse identifier (Phase 6 supplies
        // the component name prefix).
        if let Some(reg) = registry.as_deref() {
            let mut ctx = crate::reactive::HandlerEvalContext::new(reg);
            handler::invoke_handler(expr, &mut ctx, &location);
        } else {
            let mut ctx = NullEvalContext;
            handler::invoke_handler(expr, &mut ctx, &location);
        }
    }
    if handlers.has_host_listener {
        // Route the signal through the C-ABI signal registry. The
        // emission is queued and fires after the current call returns to
        // wasamo_run's message-loop drain (abi_spec §6).
        crate::emit::enqueue_signal(widget_ptr, signal, Vec::new());
    }
}

/// Run every producer in `handlers` at `widget_ptr`, in the DD-M2-P3-002
/// Option B order this runtime has always used: the native closure first,
/// then the inline DSL handlers, then the host signal enqueue.
///
/// **Why this is safe despite what user code can do to the node.** See
/// [`run_signal_handlers`]'s doc comment for the argument this function
/// shares with it (a handler's state write can rebuild or remove
/// subtrees, including this node's own, synchronously). This function's
/// own remaining step — the native closure — carries the identical
/// property: `widget_ptr` is dereferenced one further time, immediately
/// below, to fetch `clicked_fn` right before calling it — the last
/// dereference of `widget_ptr` in this function. If that closure destroys
/// its own node, it frees the `Box<dyn Fn()>` it is executing inside;
/// that is `set_clicked`'s documented pre-existing residual, not
/// something introduced here.
///
/// No node-still-attached check is needed here either, for the same
/// reason `hit_test_click`'s walk needs none: this is the one node in the
/// chain that runs anything, and nothing after it re-reads `widget_ptr`'s
/// pointee.
fn run_clicked_handlers(widget_ptr: *mut WidgetNode, handlers: ClickedHandlers) {
    if handlers.has_native {
        // Safety: see the doc comment above — this is the one remaining
        // dereference of widget_ptr before user code (the closure call
        // below) begins running.
        let node = unsafe { &mut *widget_ptr };
        if let Some(f) = node
            .button_data_mut()
            .and_then(|btn| btn.clicked_fn.as_ref())
        {
            f();
        }
    }
    run_signal_handlers(widget_ptr, "clicked", handlers.signal);
}

impl HitTree for WidgetNode {
    fn hit_rect(&self) -> Option<DipRect> {
        self.arranged_rect
    }

    fn hit_clips_children(&self) -> bool {
        self.clips_children()
    }

    fn hit_child_count(&self) -> usize {
        self.children.len()
    }

    fn hit_child(&self, index: usize) -> &Self {
        self.children[index].as_ref()
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

// Minimal disabled-state colour (DD-M3-P1-005 Phase 1 contract): a flat grey
// fill, independent of style and hover/press state, deliberately animation-
// free. M4/M5 own the richer disabled visuals.
const BUTTON_DISABLED_COLOR: Color = Color {
    A: 0x40,
    R: 0x80,
    G: 0x80,
    B: 0x80,
};

/// M4-Phase 2 T5's focus colour (DD-M4-P2-003 "shares its only means — a
/// background change — with hover and the selected state, so the three
/// must remain visually distinct"). Until M5's theming surface there is no
/// border and no focus ring to draw instead (`docs/dsl_spec.md` §4.18), so
/// a background blend is the only means available, and the colour itself
/// is provisional — M5's theming surface may absorb it.
const FOCUS_INDICATOR_COLOR: Color = Color {
    A: 0xF0,
    R: 0xFF,
    G: 0xC4,
    B: 0x4D,
};

fn blend_half(a: u8, b: u8) -> u8 {
    ((a as u16 + b as u16) / 2) as u8
}

/// Blend `base` halfway toward [`FOCUS_INDICATOR_COLOR`], every channel
/// including alpha. Blending rather than replacing is what keeps the
/// hover / selected colour underneath legible — DD-M4-P2-003 requires the
/// focused state to be visibly distinct from *both*, not to erase either.
fn focus_indicator_color(base: Color) -> Color {
    Color {
        A: blend_half(base.A, FOCUS_INDICATOR_COLOR.A),
        R: blend_half(base.R, FOCUS_INDICATOR_COLOR.R),
        G: blend_half(base.G, FOCUS_INDICATOR_COLOR.G),
        B: blend_half(base.B, FOCUS_INDICATOR_COLOR.B),
    }
}

fn effective_button_color(
    style: ButtonStyle,
    state: ButtonState,
    accent: Color,
    enabled: bool,
    checked: bool,
    focused: bool,
) -> Color {
    if !enabled {
        // A disabled widget is not a stop at all (DD-M4-P2-003; `docs/dsl_spec.md`
        // §4.8 / §4.19), so its flat disabled grey must survive regardless
        // of `focused` — checked *before* `focused` is even read.
        BUTTON_DISABLED_COLOR
    } else {
        let base = if checked {
            toggle_checked_color(style, state, accent)
        } else {
            button_state_color(style, state, accent)
        };
        if focused {
            focus_indicator_color(base)
        } else {
            base
        }
    }
}

fn toggle_checked_color(style: ButtonStyle, state: ButtonState, accent: Color) -> Color {
    match style {
        ButtonStyle::Default => match state {
            ButtonState::Normal => Color {
                A: 0xE6,
                R: 0x2F,
                G: 0x80,
                B: 0xED,
            },
            ButtonState::Hovered => Color {
                A: 0xF0,
                R: 0x4B,
                G: 0x93,
                B: 0xF0,
            },
            ButtonState::Pressed => Color {
                A: 0xF0,
                R: 0x1F,
                G: 0x66,
                B: 0xCC,
            },
        },
        ButtonStyle::Accent => match state {
            ButtonState::Normal => darken(accent, 28),
            ButtonState::Hovered => lighten(accent, 16),
            ButtonState::Pressed => darken(accent, 44),
        },
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

/// Duration for the focus-indicator colour transition (M4-Phase 2 T5).
/// `transition_duration` above takes an (old, new) *hover/press* state
/// pair; the focus indicator is a second, independent axis on the same
/// brush (DD-M4-P2-003), so reusing that function's two-argument shape
/// here would be a meaningless pair rather than a shared rule. "fast"
/// (83 ms) matches the enter-state duration hover/press already use, so a
/// focus change reads as similarly immediate.
const FOCUS_TRANSITION_TICKS: i64 = 830_000;

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
    use super::{
        effective_button_color, fixed_extent, ButtonState, ButtonStyle, MutationError,
        BUTTON_DISABLED_COLOR,
    };
    use crate::layout::{Alignment, CellPlacement, SizeConstraint, SlotData, ZStackPlacement};
    use windows::UI::Color;

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

    #[test]
    fn fixed_extent_accepts_only_two_fixed_axes() {
        assert_eq!(
            fixed_extent(&SizeConstraint::Fixed(12.5), &SizeConstraint::Fixed(7.25)),
            Some((12.5, 7.25))
        );
        assert_eq!(
            fixed_extent(&SizeConstraint::Fill, &SizeConstraint::Fixed(7.25)),
            None
        );
        assert_eq!(
            fixed_extent(&SizeConstraint::Fixed(12.5), &SizeConstraint::Shrink),
            None
        );
    }

    struct StoredSlot {
        slot: Slot,
        slot_data: Option<SlotData>,
    }

    struct Children(Vec<StoredSlot>);

    impl Children {
        fn new() -> Self {
            Children(Vec::new())
        }

        fn len(&self) -> usize {
            self.0.len()
        }

        fn insert(&mut self, index: usize, slot: Slot) -> Result<(), MutationError> {
            self.insert_with_slot_data(index, slot, None)
        }

        fn insert_with_slot_data(
            &mut self,
            index: usize,
            mut slot: Slot,
            slot_data: Option<SlotData>,
        ) -> Result<(), MutationError> {
            if index > self.0.len() {
                return Err(MutationError::IndexOutOfBounds);
            }
            if slot.attached {
                return Err(MutationError::AlreadyAttached);
            }
            slot.attached = true;
            self.0.insert(index, StoredSlot { slot, slot_data });
            Ok(())
        }

        fn remove(&mut self, index: usize) -> Result<Slot, MutationError> {
            if index >= self.0.len() {
                return Err(MutationError::IndexOutOfBounds);
            }
            let mut stored = self.0.remove(index);
            stored.slot.attached = false;
            Ok(stored.slot)
        }

        fn replace(&mut self, index: usize, mut new: Slot) -> Result<Slot, MutationError> {
            if index >= self.0.len() {
                return Err(MutationError::IndexOutOfBounds);
            }
            if new.attached {
                return Err(MutationError::AlreadyAttached);
            }
            new.attached = true;
            let slot_data = self.0[index].slot_data;
            let mut old = std::mem::replace(
                &mut self.0[index],
                StoredSlot {
                    slot: new,
                    slot_data,
                },
            );
            old.slot.attached = false;
            Ok(old.slot)
        }

        fn slot_data_at(&self, index: usize) -> Option<SlotData> {
            self.0[index].slot_data
        }
    }

    fn zplace(h_align: Alignment, v_align: Alignment) -> ZStackPlacement {
        ZStackPlacement { h_align, v_align }
    }

    fn grid_place() -> CellPlacement {
        CellPlacement::default_grid()
    }

    fn color(a: u8, r: u8, g: u8, b: u8) -> Color {
        Color {
            A: a,
            R: r,
            G: g,
            B: b,
        }
    }

    #[test]
    fn togglebutton_disabled_color_wins_over_checked_and_pressed_state() {
        let accent = color(0xFF, 0x20, 0x80, 0xD0);
        for style in [ButtonStyle::Default, ButtonStyle::Accent] {
            for state in [
                ButtonState::Normal,
                ButtonState::Hovered,
                ButtonState::Pressed,
            ] {
                for focused in [false, true] {
                    assert_eq!(
                        effective_button_color(style, state, accent, false, true, focused),
                        BUTTON_DISABLED_COLOR,
                        "DD-M4-P2-003: a disabled widget is not a stop at all, so its flat \
                         disabled grey must survive regardless of the focus flag \
                         (style={style:?}, state={state:?}, focused={focused})"
                    );
                }
            }
        }
    }

    #[test]
    fn togglebutton_checked_hover_press_color_matrix_is_pinned() {
        let accent = color(0xFF, 0x20, 0x80, 0xD0);
        assert_eq!(
            effective_button_color(
                ButtonStyle::Default,
                ButtonState::Normal,
                accent,
                true,
                true,
                false,
            ),
            color(0xE6, 0x2F, 0x80, 0xED)
        );
        assert_eq!(
            effective_button_color(
                ButtonStyle::Default,
                ButtonState::Hovered,
                accent,
                true,
                true,
                false,
            ),
            color(0xF0, 0x4B, 0x93, 0xF0)
        );
        assert_eq!(
            effective_button_color(
                ButtonStyle::Default,
                ButtonState::Pressed,
                accent,
                true,
                true,
                false,
            ),
            color(0xF0, 0x1F, 0x66, 0xCC)
        );
        assert_eq!(
            effective_button_color(
                ButtonStyle::Accent,
                ButtonState::Normal,
                accent,
                true,
                true,
                false,
            ),
            color(0xFF, 0x04, 0x64, 0xB4)
        );
        assert_eq!(
            effective_button_color(
                ButtonStyle::Accent,
                ButtonState::Hovered,
                accent,
                true,
                true,
                false,
            ),
            color(0xFF, 0x30, 0x90, 0xE0)
        );
        assert_eq!(
            effective_button_color(
                ButtonStyle::Accent,
                ButtonState::Pressed,
                accent,
                true,
                true,
                false,
            ),
            color(0xFF, 0x00, 0x54, 0xA4)
        );
    }

    // ── Focus indicator distinctness (M4-Phase 2 T5, DD-M4-P2-003) ────────

    /// Do `a` and `b` differ by at least `min` in at least one channel
    /// (A/R/G/B)? `u8::abs_diff` is not used so the comparison reads the
    /// same shape as the ADR's wording ("distinct... by at least N").
    fn channel_delta_at_least(a: Color, b: Color, min: u8) -> bool {
        fn delta(x: u8, y: u8) -> u8 {
            x.max(y) - x.min(y)
        }
        delta(a.A, b.A) >= min
            || delta(a.R, b.R) >= min
            || delta(a.G, b.G) >= min
            || delta(a.B, b.B) >= min
    }

    #[test]
    fn focused_normal_is_distinct_from_every_unfocused_hover_state() {
        let accent = color(0xFF, 0x20, 0x80, 0xD0);
        for style in [ButtonStyle::Default, ButtonStyle::Accent] {
            for checked in [false, true] {
                let focused_normal =
                    effective_button_color(style, ButtonState::Normal, accent, true, checked, true);
                for (label, state) in [
                    ("Normal", ButtonState::Normal),
                    ("Hovered", ButtonState::Hovered),
                    ("Pressed", ButtonState::Pressed),
                ] {
                    let unfocused =
                        effective_button_color(style, state, accent, true, checked, false);
                    assert!(
                        channel_delta_at_least(focused_normal, unfocused, 8),
                        "DD-M4-P2-003: the focused state must be visibly distinct from \
                         unfocused {label} ({style:?}, checked={checked}) by at least 8 in \
                         at least one channel, or a captured frame cannot say which stop \
                         holds focus"
                    );
                }
            }
        }
    }

    #[test]
    fn focused_unchecked_is_distinct_from_unfocused_checked_the_selected_confusion() {
        // The gallery's first Tab stop is a *checked* ToggleButton (T5 scope
        // fact 5), so this is not a constructed edge case: it is the first
        // frame any traversal evidence produces.
        let accent = color(0xFF, 0x20, 0x80, 0xD0);
        for style in [ButtonStyle::Default, ButtonStyle::Accent] {
            let focused_unchecked =
                effective_button_color(style, ButtonState::Normal, accent, true, false, true);
            let unfocused_checked =
                effective_button_color(style, ButtonState::Normal, accent, true, true, false);
            assert!(
                channel_delta_at_least(focused_unchecked, unfocused_checked, 8),
                "DD-M4-P2-003 names this confusion directly: focused-but-unchecked \
                 ({style:?}) must not read as unfocused-but-checked (selected)"
            );
        }
    }

    #[test]
    fn insert_at_zero() {
        let mut ch = Children::new();
        assert!(ch.insert(0, Slot::new()).is_ok());
        assert_eq!(ch.len(), 1);
        assert!(ch.0[0].slot.attached);
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
        assert!(ch.0[0].slot.attached);
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
        assert!(ch.0[0].slot.attached);
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
        assert!(ch.0[0].slot.attached);
    }

    #[test]
    fn already_attached_cannot_reattach() {
        let mut ch = Children::new();
        ch.insert(0, Slot::new()).unwrap();
        let already = Slot { attached: true };
        assert_eq!(ch.insert(0, already), Err(MutationError::AlreadyAttached));
    }

    #[test]
    fn insert_stores_zstack_slot_data_on_the_slot() {
        let mut ch = Children::new();
        let slot_data = Some(SlotData::ZStack(zplace(
            Alignment::Trailing,
            Alignment::Stretch,
        )));

        ch.insert_with_slot_data(0, Slot::new(), slot_data).unwrap();

        assert_eq!(ch.slot_data_at(0), slot_data);
    }

    #[test]
    fn insert_stores_grid_slot_data_on_the_slot() {
        let mut ch = Children::new();
        let slot_data = Some(SlotData::Grid(grid_place()));

        ch.insert_with_slot_data(0, Slot::new(), slot_data).unwrap();

        assert_eq!(ch.slot_data_at(0), slot_data);
    }

    #[test]
    fn non_placement_parent_insert_normalizes_slot_data_to_none() {
        let mut ch = Children::new();

        ch.insert(0, Slot::new()).unwrap();

        assert_eq!(ch.slot_data_at(0), None);
    }

    #[test]
    fn remove_returns_detached_subtree_without_slot_metadata() {
        let mut ch = Children::new();
        ch.insert_with_slot_data(
            0,
            Slot::new(),
            Some(SlotData::ZStack(ZStackPlacement::centered())),
        )
        .unwrap();

        let removed = ch.remove(0).unwrap();

        assert!(!removed.attached);
        assert_eq!(ch.len(), 0);
    }

    #[test]
    fn replace_preserves_existing_slot_data_on_new_child() {
        let mut ch = Children::new();
        let slot_data = Some(SlotData::ZStack(zplace(
            Alignment::Leading,
            Alignment::Stretch,
        )));
        ch.insert_with_slot_data(0, Slot::new(), slot_data).unwrap();

        let old = ch.replace(0, Slot::new()).unwrap();

        assert!(!old.attached);
        assert_eq!(ch.slot_data_at(0), slot_data);
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
    // needs a Compositor; the build_node materialisation half of ADR §Phase 2
    // verification closure item 2 is exercised by the Windows-only integration
    // test landed in T10 (`wasamo-runtime/tests/box_round_trip.rs`), which
    // reads the resulting `WidgetData::Box` through `__box_state_for_test`.

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

    // ── M3-Phase 3 T5: WrapPanel widget data shape ───────────────────────────
    //
    // The variant has no Win32/WinRT field — its data shape is verifiable
    // as pure logic without a Compositor. The `WidgetNode::wrap_panel`
    // constructor itself needs a Compositor; T8 (Windows-runtime
    // integration test) will exercise the build_node materialisation half
    // alongside the IR loader path landed in T6.

    #[test]
    fn wrap_panel_variant_carries_three_attributes() {
        let data = WidgetData::WrapPanel {
            item_cross_size: Some(96),
            item_spacing: 8,
            line_spacing: 12,
        };
        match &data {
            WidgetData::WrapPanel {
                item_cross_size,
                item_spacing,
                line_spacing,
            } => {
                assert_eq!(*item_cross_size, Some(96));
                assert_eq!(*item_spacing, 8);
                assert_eq!(*line_spacing, 12);
            }
            _ => panic!("expected WidgetData::WrapPanel variant"),
        }
    }

    #[test]
    fn wrap_panel_variant_defaults_match_constructor_defaults() {
        // Mirrors the data shape that `WidgetNode::wrap_panel(..,
        // None, None, None)` produces after `apply_wrap_panel_defaults`
        // resolves absences — DD-M3-P3-004 Option (a) parent-cross
        // passthrough (`None`) and DD-M3-P3-003 touching items / lines
        // (`0` / `0`). The `apply_wrap_panel_defaults_*` tests below
        // exercise that resolution directly; this test pins only the
        // post-resolution data carrier.
        let data = WidgetData::WrapPanel {
            item_cross_size: None,
            item_spacing: 0,
            line_spacing: 0,
        };
        if let WidgetData::WrapPanel {
            item_cross_size,
            item_spacing,
            line_spacing,
        } = &data
        {
            assert!(item_cross_size.is_none());
            assert_eq!(*item_spacing, 0);
            assert_eq!(*line_spacing, 0);
        } else {
            panic!("expected WidgetData::WrapPanel variant");
        }
    }

    #[test]
    fn wrap_panel_variant_accepts_zero_item_cross_size() {
        // DD-M3-P3-006 zero-handling: zero is a *valid* setting on every
        // WrapPanel integer attribute (the rejection threshold is `< 0`,
        // not `<= 0`). `wasamoc check` (T1) already pins this on the
        // diagnostic side; the data shape is symmetric — `Some(0)` is a
        // distinct, legal carrier from `None`.
        let data = WidgetData::WrapPanel {
            item_cross_size: Some(0),
            item_spacing: 0,
            line_spacing: 0,
        };
        if let WidgetData::WrapPanel {
            item_cross_size, ..
        } = &data
        {
            assert_eq!(*item_cross_size, Some(0));
        } else {
            panic!("expected WidgetData::WrapPanel variant");
        }
    }

    // ── M3-Phase 3 T5: WrapPanel absent-to-default mapping ──────────────────
    //
    // `apply_wrap_panel_defaults` is the single authoritative site for
    // DD-M3-P3-003 / DD-M3-P3-004 default policy at the runtime catalog
    // layer. The T6 IR loader forwards presence / absence verbatim and
    // these tests pin the absent→default mapping the constructor performs.

    use super::apply_wrap_panel_defaults;

    #[test]
    fn apply_wrap_panel_defaults_maps_all_absent_to_runtime_defaults() {
        // DD-M3-P3-004 Option (a): `item_cross_size` absent → `None`
        // (parent-cross passthrough). DD-M3-P3-003: `item_spacing` and
        // `line_spacing` absent → `0` (touching items / lines).
        let (item_cross_size, item_spacing, line_spacing) =
            apply_wrap_panel_defaults(None, None, None);
        assert_eq!(item_cross_size, None);
        assert_eq!(item_spacing, 0);
        assert_eq!(line_spacing, 0);
    }

    #[test]
    fn apply_wrap_panel_defaults_passes_through_present_values() {
        // When every attribute is present in the IR, the mapping is the
        // identity (modulo the `Option<i32> → i32` unwrap for the two
        // spacing attributes). Phase 3 has no clamping at this layer —
        // `wasamoc check` T1 and the T6 `validate()` gate both reject
        // negative values before they reach the constructor.
        let (item_cross_size, item_spacing, line_spacing) =
            apply_wrap_panel_defaults(Some(96), Some(8), Some(12));
        assert_eq!(item_cross_size, Some(96));
        assert_eq!(item_spacing, 8);
        assert_eq!(line_spacing, 12);
    }

    #[test]
    fn apply_wrap_panel_defaults_handles_each_attribute_independently() {
        // Mixed presence: only `item_spacing` is set. The other two
        // attributes must still receive their per-attribute defaults
        // (None / 0) — the mapping does not couple attributes.
        let (item_cross_size, item_spacing, line_spacing) =
            apply_wrap_panel_defaults(None, Some(5), None);
        assert_eq!(item_cross_size, None);
        assert_eq!(item_spacing, 5);
        assert_eq!(line_spacing, 0);
    }

    #[test]
    fn apply_wrap_panel_defaults_preserves_some_zero_distinct_from_none() {
        // DD-M3-P3-006 zero-handling at the default boundary: `Some(0)`
        // for `item_cross_size` is a legal, intentional setting (uniform
        // zero per-line cross-axis size) and must NOT collapse to `None`.
        // `Some(0)` for the two spacings is indistinguishable from the
        // absent default at the field-value layer (both yield `0`) but
        // the helper still threads `Some(0)` through unwrap-or-default
        // unchanged.
        let (item_cross_size, item_spacing, line_spacing) =
            apply_wrap_panel_defaults(Some(0), Some(0), Some(0));
        assert_eq!(item_cross_size, Some(0));
        assert_eq!(item_spacing, 0);
        assert_eq!(line_spacing, 0);
    }
}
