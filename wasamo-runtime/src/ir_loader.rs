//! IR loader (DD-M2-P6-006) — parses the normative Wasamo IR text grammar
//! (DD-M2-P6-002) and constructs the runtime widget tree.
//!
//! The module is split in two: a pure-logic parser (`parse_ir`, testable
//! without any Win32/WinRT dependency) and a builder (`build_widget_tree`,
//! requires a live `Compositor` and `TextRenderer`). The C ABI front-end
//! (`wasamo_load_ui`) is wired in DD-M2-P6-005 — this module exposes the
//! Rust-level entry points only.

use std::rc::Rc;

use wasamo_ir::{
    CompoundOp, ControlFlowBranch, ControlFlowNode, HandlerExpr, InterpolationPart, IrBinding,
    IrComponent, IrHandler, IrLiteral, IrMember, IrNode, IrProp, IrState, IrType, KindPayload,
    TrackSize,
};

use crate::box_values;
use crate::layout::{Alignment, CellPlacement, TrackSize as LayoutTrackSize, ZStackPlacement};
use crate::reactive::{
    register_binding, register_bool_binding, set_active_registry, BindingTarget, PropertyKey,
    Signal, SignalRegistry, WidgetId,
};
use crate::text::{TextRenderer, TypographyStyle};
use crate::widget::{
    widget_write_property, widget_write_property_bool, ButtonStyle, WidgetNode,
    PROP_BUTTON_ENABLED, PROP_BUTTON_LABEL, PROP_BUTTON_STYLE, PROP_SCROLLVIEW_OFFSET_Y,
    PROP_TEXT_CONTENT, PROP_TEXT_STYLE,
};

use windows::UI::Composition::Compositor;

/// Errors surfaced by the IR loader.
///
/// `InvalidHeader`, `Parse`, `UnknownWidget`, and `Validate` are all
/// "malformed IR" failures — DD-M2-P6-005 maps them to
/// `WASAMO_ERR_IR_MALFORMED` at the C ABI boundary, with the `Display`
/// rendering surfaced through `wasamo_last_error_message`. `Build`
/// failures originate from the Win32/WinRT side and are not part of the
/// defense-in-depth surface targeted by DD-M2-P6-009.
#[derive(Debug, Clone, PartialEq)]
pub enum IrLoadError {
    InvalidHeader(String),
    Parse(String),
    UnknownWidget(String),
    Validate(String),
    Build(String),
}

impl IrLoadError {
    /// Whether this variant represents a "malformed IR" failure
    /// (DD-M2-P6-009). Useful for the C ABI translation in DD-M2-P6-005.
    pub fn is_malformed(&self) -> bool {
        matches!(
            self,
            IrLoadError::InvalidHeader(_)
                | IrLoadError::Parse(_)
                | IrLoadError::UnknownWidget(_)
                | IrLoadError::Validate(_)
        )
    }
}

impl std::fmt::Display for IrLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrLoadError::InvalidHeader(msg) => write!(f, "invalid IR header: {msg}"),
            IrLoadError::Parse(msg) => write!(f, "IR parse error: {msg}"),
            IrLoadError::UnknownWidget(name) => write!(f, "unknown widget type: {name}"),
            IrLoadError::Validate(msg) => write!(f, "IR validation error: {msg}"),
            IrLoadError::Build(msg) => write!(f, "IR build error: {msg}"),
        }
    }
}

const HEADER_MAGIC: &str = ";wasamo-ir v0";

/// A widget tree built from an IR component, paired with the SignalRegistry
/// for the component's `state` declarations. The registry is also installed
/// as the runtime's active registry (see `reactive::set_active_registry`) so
/// click-handler dispatch can reach it; the field here keeps an extra Rc
/// around so the registry's lifetime is tied to the BuiltUi (and therefore
/// to whatever owns the widget tree), independent of the thread-local.
pub struct BuiltUi {
    pub root: Box<WidgetNode>,
    #[allow(dead_code)]
    pub(crate) registry: Rc<SignalRegistry>,
}

impl BuiltUi {
    /// Test-only mutator for an `i32`-typed state declared on the
    /// component (per `state <name>: i32 = <default>`). Writes through
    /// the underlying `Signal<i32>::set`, so the reactive engine
    /// re-fires every binding effect that read this state — exactly
    /// the path a handler-driven mutation (`compound-assign += scroll_y
    /// 100`) takes at runtime. Returns `true` when the state name
    /// exists, `false` otherwise. Hidden from rustdoc and named with
    /// the project's `__*_for_test` convention.
    ///
    /// Used by `wasamo-runtime/tests/scroll_view_layout_integration.rs`
    /// to drive the ADR Phase 4 verification closure item 4
    /// "mutate `state.scroll_y`" assertions without rewiring the
    /// `SignalRegistry` visibility.
    #[doc(hidden)]
    pub fn __set_i32_state_for_test(&self, name: &str, value: i32) -> bool {
        match self.registry.i32s.get(name) {
            Some(signal) => {
                signal.set(value);
                true
            }
            None => false,
        }
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Parse the normative IR text (DD-M2-P6-002) into an `IrComponent`. Pure
/// logic — testable without any Win32/WinRT dependency.
///
/// On success the returned `IrComponent` has passed defense-in-depth
/// validation (DD-M2-P6-009): header magic + version, top-level document
/// structure (enforced by the parser), unique `state` names, and
/// resolution of every name referenced by a binding/handler expression
/// to a declared `state`. Per-node value-type integrity is trusted from
/// the emitter (`wasamoc`) and is **not** re-validated here.
pub fn parse_ir(text: &str) -> Result<IrComponent, IrLoadError> {
    let body = check_and_strip_header(text)?;
    let tokens = tokenize(body)?;
    let mut p = Parser {
        tokens: &tokens,
        pos: 0,
    };
    let comp = p.parse_component()?;
    if p.pos < p.tokens.len() {
        return Err(IrLoadError::Parse(format!(
            "unexpected trailing tokens after component (token #{})",
            p.pos
        )));
    }
    validate(&comp)?;
    Ok(comp)
}

/// Defense-in-depth validation pass (DD-M2-P6-009).
///
/// Currently checks:
/// 1. `state` declarations have unique names (flat namespace per
///    DD-M2-P6-004's name-resolution rules).
/// 2. Every name referenced by a binding/handler expression resolves to
///    a declared `state`. Widget-instance references are not yet part of
///    the IR — see `docs/notes/dsl-grammar.md` Q1 for the open question.
fn validate(comp: &IrComponent) -> Result<(), IrLoadError> {
    let mut declared = std::collections::HashMap::new();
    for state in &comp.states {
        if declared
            .insert(state.name.as_str(), state.ty.clone())
            .is_some()
        {
            return Err(IrLoadError::Validate(format!(
                "duplicate `state` name: `{}`",
                state.name
            )));
        }
    }
    validate_node_references(&comp.root, &declared)?;
    // M3-Phase 2 T7 defense-in-depth gates (DD-M3-P2-001 / DD-M3-P2-002 /
    // DD-M3-P2-003). The shared mapping at the C ABI boundary is
    // `WASAMO_ERR_IR_MALFORMED` (DD-M2-P6-005 / DD-M2-P6-009). These
    // checks live in `validate` so they exercise without a live
    // `Compositor`; `build_node` would otherwise have had to repeat them
    // before construction. wasamoc emits IR that already respects both
    // invariants — these gates exist for IR not produced by wasamoc
    // (e.g. via `wasamo_load_ui` directly).
    validate_phase2_node_invariants(&comp.root)?;
    // M3-Phase 3 T6 defense-in-depth gate (DD-M3-P3-006 runtime half).
    // `wasamoc check` (T1) rejects negative literals on WrapPanel's three
    // attribute names at compile time; this is the last-line-of-defence
    // for memory-IR that reaches the runtime via `wasamo_load_ui`
    // without traversing `wasamoc`.
    validate_phase3_node_invariants(&comp.root)?;
    // M3-Phase 4 T3 defense-in-depth gate (DD-M3-P4-006 structural half).
    // `wasamoc check` (T1) diagnoses 0-child / >1-child ScrollView at
    // compile time; this is the runtime gate for memory-IR that reaches
    // the runtime via `wasamo_load_ui` without traversing `wasamoc`. The
    // value-range half (negative / out-of-range `offset-y`) deliberately
    // does **not** reject — DD-M3-P4-005's layout-time clamp is the
    // runtime gate per DD-M3-P4-006's compound-shape decision, so a
    // bound `state.scroll_y` can legitimately transition through
    // negative / out-of-range intermediate values without becoming a
    // load-time error.
    validate_phase4_node_invariants(&comp.root)?;
    // M3-Phase 5 T3 defense-in-depth gate (DD-M3-P5-006). `wasamoc check`
    // (T1) dual-gates every Grid / Cell structural invariant at compile
    // time; this is the runtime gate for memory IR that reaches the
    // loader via `wasamo_load_ui` without traversing `wasamoc`. All Grid
    // invariants are reject-at-validate (no clamp-at-arrange — Grid has
    // no runtime-clamp analogue to ScrollView's `offset-y`); the only
    // Grid layout-time gate is `LayoutError::GridUnboundedStarAxis`
    // (DD-M3-P5-004), which depends on the parent axis bound and so is
    // not a `validate()`-time concern. A top-level / non-Grid-nested
    // `Cell` is rejected as Cell-outside-Grid (the recursion descends
    // into a Grid's Cell *content* children, never treating `Cell` as a
    // standalone node, so a `Cell` reached by the generic walk is
    // necessarily misplaced).
    validate_phase5_node_invariants(&comp.root)?;
    // M3-Phase 6 T3 defense-in-depth gate (DD-M3-P6-001 /
    // DD-M3-P6-002). `wasamoc check` (T1) rejects the same malformed
    // ZStack surface before emit; this gate covers memory/textual IR that
    // reaches the runtime loader directly. ZStack has no kind payload, no
    // ZStack-level attrs, no bindings/handlers, and only its direct children
    // may carry `h-align` / `v-align` placement annotations.
    validate_phase6_zstack_node_invariants(&comp.root, ParentKind::Root)?;
    validate_phase6_control_flow_invariants(&comp.root)
}

fn validate_phase6_control_flow_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    for member in &node.children {
        match member {
            IrMember::Widget(child) => validate_phase6_control_flow_invariants(child)?,
            IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
                if branches.len() != 1 {
                    return Err(IrLoadError::Validate(format!(
                        "`if` control flow supports exactly one branch in M3-Phase 6, got {}",
                        branches.len()
                    )));
                }
                let branch = &branches[0];
                if branch.body.len() != 1 {
                    return Err(IrLoadError::Validate(format!(
                        "`if` body supports exactly one widget member in M3-Phase 6, got {}",
                        branch.body.len()
                    )));
                }
                match &branch.body[0] {
                    IrMember::Widget(body) => validate_phase6_control_flow_invariants(body)?,
                    IrMember::ControlFlow(_) => {
                        return Err(IrLoadError::Validate(
                            "a nested control-flow member is not valid directly in an `if` body in M3-Phase 6".into(),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_phase2_member_invariants(member: &IrMember) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(child) => validate_phase2_node_invariants(child),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase2_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
    }
}

fn validate_phase2_node_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    // Box single-child invariant (DD-M3-P2-001). wasamoc check (T3)
    // diagnoses the same condition at compile time; this is the runtime
    // defense for IR not produced by wasamoc.
    let widget_child_count = node.widget_children().count();
    if node.widget_type == "Box" && widget_child_count > 1 {
        return Err(IrLoadError::Validate(format!(
            "`Box` node accepts at most one child, got {} (use `VStack` / `HStack` / `ZStack` for multi-child layouts)",
            widget_child_count
        )));
    }
    // Ratio / Color literal placement (DD-M3-P2-002 / DD-M3-P2-003,
    // variant strategy Option A). These literals materialise directly
    // into Box-internal `Ratio` / `Color` at `build_node` and never
    // travel as `PropertyValue`; appearing in any other prop position
    // would imply a `PropertyValue` boundary that does not exist in
    // Phase 2.
    for prop in &node.props {
        match &prop.value {
            IrLiteral::Ratio { .. } => {
                let valid = node.widget_type == "Box" && prop.name == "aspect";
                if !valid {
                    return Err(IrLoadError::Validate(format!(
                        "ratio literal valid only on `Box.aspect`, found on `{}.{}`",
                        node.widget_type, prop.name
                    )));
                }
            }
            IrLiteral::Color(_) => {
                let valid = node.widget_type == "Box" && prop.name == "fill";
                if !valid {
                    return Err(IrLoadError::Validate(format!(
                        "color literal valid only on `Box.fill`, found on `{}.{}`",
                        node.widget_type, prop.name
                    )));
                }
            }
            _ => {}
        }
    }
    for member in &node.children {
        validate_phase2_member_invariants(member)?;
    }
    Ok(())
}

// M3-Phase 3 T6 defense-in-depth: reject negative literal values on
// WrapPanel's three attribute names (`item-cross-size` /
// `item-spacing` / `line-spacing`). The DSL surface spec invariant is
// non-negative `i32` per DD-M3-P3-006 (zero is *valid*; the rejection
// threshold is `< 0`, not `<= 0`). Scoped to `widget_type == "WrapPanel"`
// to match `wasamoc check` T1 — attribute-position rejection on other
// widgets is the compile-time half's responsibility, not this runtime
// gate.
// M3-Phase 4 T3 defense-in-depth: enforce ScrollView's exactly-1-child
// contract (DD-M3-P4-001 / DD-M3-P4-006). The DSL surface invariant
// is "exactly one content child"; both 0-child and >1-child surface as
// `WASAMO_ERR_IR_MALFORMED` at the C ABI boundary. Symmetric with
// Phase 2's `Box`-child-count gate in shape, distinct in cardinality
// (Box admits 0 or 1; ScrollView demands exactly 1). The value-range
// half (`offset-y`) intentionally has no gate here — DD-M3-P4-005's
// arrange-time clamp is the runtime gate so bindings can transition
// through negative / out-of-range intermediates.
fn validate_phase4_node_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    let widget_child_count = node.widget_children().count();
    if node.widget_type == "ScrollView" && widget_child_count != 1 {
        return Err(IrLoadError::Validate(format!(
            "`ScrollView` requires exactly one content child, got {}",
            widget_child_count
        )));
    }
    for member in &node.children {
        validate_phase4_member_invariants(member)?;
    }
    Ok(())
}

fn validate_phase4_member_invariants(member: &IrMember) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(child) => validate_phase4_node_invariants(child),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase4_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
    }
}

fn validate_phase3_node_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    if node.widget_type == "WrapPanel" {
        for prop in &node.props {
            let is_wrap_attr = matches!(
                prop.name.as_str(),
                "item-cross-size" | "item-spacing" | "line-spacing"
            );
            if !is_wrap_attr {
                continue;
            }
            if let IrLiteral::Int(n) = &prop.value {
                if *n < 0 {
                    return Err(IrLoadError::Validate(format!(
                        "`WrapPanel.{}` must be non-negative, got {}",
                        prop.name, n
                    )));
                }
            }
        }
    }
    for member in &node.children {
        validate_phase3_member_invariants(member)?;
    }
    Ok(())
}

fn validate_phase3_member_invariants(member: &IrMember) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(child) => validate_phase3_node_invariants(child),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase3_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
    }
}

// M3-Phase 5 T3 defense-in-depth (DD-M3-P5-006). Mirrors the
// `wasamoc check` Grid / Cell gate (`wasamoc/src/check.rs`
// `check_grid` / `check_cell`) against post-lowering memory IR. Routes
// by widget kind: a `Grid` is validated as a unit (its `Cell` children
// are validated here, not as standalone nodes), and recursion descends
// only into each Cell's content child. A `Cell` reached by the generic
// walk is therefore necessarily outside a `Grid` and is rejected.
fn validate_phase5_node_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    // Grid `kind_payload` (carrier c1) is Grid-only — the wasamo-ir
    // invariant is "non-Grid kind → kind_payload None" (DD-M3-P5-001).
    // The textual parser already restricts `tracks` to Grid nodes; this
    // is the defense-in-depth gate for IR constructed programmatically
    // (e.g. directly via `IrNode { .. }` and handed to `validate`).
    reject_non_grid_kind_payload(node)?;
    match node.widget_type.as_str() {
        "Grid" => {
            validate_grid_invariants(node)?;
            // Descend into each Cell's content child (the Cell wrapper
            // itself is validated above and is IR-only). A bad child-count
            // was already rejected by `validate_grid_invariants`.
            for cell in node.widget_children() {
                for member in &cell.children {
                    validate_phase5_member_invariants(member)?;
                }
            }
        }
        "Cell" => {
            return Err(IrLoadError::Validate(
                "`Cell` is only valid as a direct child of a `Grid` (DD-M3-P5-001)".into(),
            ));
        }
        _ => {
            for member in &node.children {
                validate_phase5_member_invariants(member)?;
            }
        }
    }
    Ok(())
}

fn validate_phase5_member_invariants(member: &IrMember) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(child) => validate_phase5_node_invariants(child),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase5_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParentKind {
    Root,
    Grid,
    Cell,
    ZStack,
    Other,
}

fn parent_kind_for(node: &IrNode) -> ParentKind {
    match node.widget_type.as_str() {
        "Grid" => ParentKind::Grid,
        "Cell" => ParentKind::Cell,
        "ZStack" => ParentKind::ZStack,
        _ => ParentKind::Other,
    }
}

// M3-Phase 6 T3 defense-in-depth (DD-M3-P6-001 / DD-M3-P6-002). Mirrors
// the ZStack surface from `wasamoc check`: ZStack is a direct-child
// container with no `KindPayload`, no ZStack-level props, and no bindable
// properties; `h-align` / `v-align` are parent-owned placement props valid
// only on a ZStack direct child (or Grid Cell, owned by Phase 5).
fn validate_phase6_zstack_node_invariants(
    node: &IrNode,
    parent: ParentKind,
) -> Result<(), IrLoadError> {
    if node.widget_type == "ZStack" {
        if node.kind_payload.is_some() {
            return Err(IrLoadError::Validate(
                "`ZStack` must not carry a `kind_payload` (DD-M3-P6-001)".into(),
            ));
        }
        if !node.props.is_empty() {
            return Err(IrLoadError::Validate(format!(
                "`ZStack` accepts no Phase-6 attributes; found `{}`",
                node.props[0].name
            )));
        }
        if !node.bindings.is_empty() {
            return Err(IrLoadError::Validate(
                "`ZStack` accepts no Phase-6 bindings".into(),
            ));
        }
        if !node.handlers.is_empty() {
            return Err(IrLoadError::Validate(
                "`ZStack` accepts no Phase-6 handlers".into(),
            ));
        }
    }

    for prop in &node.props {
        if matches!(prop.name.as_str(), "h-align" | "v-align") {
            let allowed = parent == ParentKind::ZStack
                || (parent == ParentKind::Grid && node.widget_type == "Cell");
            if !allowed {
                return Err(IrLoadError::Validate(format!(
                    "`{}` is valid only on a ZStack direct child or Grid `Cell`",
                    prop.name
                )));
            }
            validate_alignment_literal(
                &prop.value,
                &format!("{}.{}", node.widget_type, prop.name),
            )?;
        }
    }

    let current = parent_kind_for(node);
    for member in &node.children {
        validate_phase6_zstack_member_invariants(member, current)?;
    }
    Ok(())
}

fn validate_phase6_zstack_member_invariants(
    member: &IrMember,
    parent: ParentKind,
) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(child) => validate_phase6_zstack_node_invariants(child, parent),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase6_zstack_member_invariants(body_member, parent)?;
                }
            }
            Ok(())
        }
    }
}

fn validate_alignment_literal(value: &IrLiteral, label: &str) -> Result<(), IrLoadError> {
    match value {
        IrLiteral::Ident(id) if matches!(id.as_str(), "start" | "center" | "end" | "stretch") => {
            Ok(())
        }
        _ => Err(IrLoadError::Validate(format!(
            "`{label}` must be one of start, center, end, stretch"
        ))),
    }
}

/// Reject a non-`Grid` node carrying a Grid `kind_payload` (carrier c1
/// is Grid-only — DD-M3-P5-001; wasamo-ir "non-Grid → None" invariant).
fn reject_non_grid_kind_payload(node: &IrNode) -> Result<(), IrLoadError> {
    if node.widget_type != "Grid" && node.kind_payload.is_some() {
        return Err(IrLoadError::Validate(format!(
            "Grid track-list payload (`kind_payload`) is only valid on a `Grid` node, found on `{}` (DD-M3-P5-001)",
            node.widget_type
        )));
    }
    Ok(())
}

/// DD-M3-P5-002 star-weight cap (inclusive). Mirrors `wasamoc`'s
/// `STAR_WEIGHT_MAX`.
const GRID_STAR_WEIGHT_MAX: u32 = 1024;

/// A resolved Cell rectangle in track coordinates, used for the pairwise
/// overlap check. Built only after placement / span all validate.
struct GridCellRect {
    row: i64,
    column: i64,
    row_span: i64,
    column_span: i64,
}

// Validate one `Grid` node's body (DD-M3-P5-006 invariant table). Track
// value ranges, minimum row / column count, per-`Cell` child-count +
// placement / span range + alignment vocabulary, and pairwise
// same-cell / overlapping-rectangle conflict. All violations surface
// `IrLoadError::Validate` → `WASAMO_ERR_IR_MALFORMED`.
fn validate_grid_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    let (columns, rows) = match &node.kind_payload {
        Some(KindPayload::Grid { columns, rows }) => (columns, rows),
        None => {
            return Err(IrLoadError::Validate(
                "`Grid` requires `columns:` and `rows:` track lists (DD-M3-P5-001)".into(),
            ));
        }
    };

    // Minimum shape: at least one row and one column track (DD-M3-P5-001).
    if columns.is_empty() {
        return Err(IrLoadError::Validate(
            "`Grid` requires at least one column track (DD-M3-P5-001)".into(),
        ));
    }
    if rows.is_empty() {
        return Err(IrLoadError::Validate(
            "`Grid` requires at least one row track (DD-M3-P5-001)".into(),
        ));
    }

    // Track value ranges (DD-M3-P5-002): fixed `>= 1`; star weight in
    // `[1, 1024]`.
    for axis in [columns, rows] {
        for t in axis {
            match t {
                TrackSize::Fixed(v) => {
                    if *v < 1 {
                        return Err(IrLoadError::Validate(format!(
                            "`Grid` fixed track size must be a positive integer, got {v} (DD-M3-P5-002)"
                        )));
                    }
                }
                TrackSize::Star(w) => {
                    if *w < 1 || *w > GRID_STAR_WEIGHT_MAX {
                        return Err(IrLoadError::Validate(format!(
                            "`Grid` star weight must be in [1, {GRID_STAR_WEIGHT_MAX}], got {w} (DD-M3-P5-002)"
                        )));
                    }
                }
            }
        }
    }

    let columns_len = columns.len() as i64;
    let rows_len = rows.len() as i64;

    // Per-Cell validation + rectangle collection (DD-M3-P5-003 /
    // DD-M3-P5-005 / DD-M3-P5-006).
    let mut rects: Vec<GridCellRect> = Vec::new();
    for member in &node.children {
        match member {
            IrMember::Widget(cell) if cell.widget_type == "Cell" => {
                // A `Cell` is a non-Grid node and must not carry a Grid payload.
                reject_non_grid_kind_payload(cell)?;
                rects.push(validate_grid_cell(cell, columns_len, rows_len)?);
            }
            IrMember::Widget(child) => {
                return Err(IrLoadError::Validate(format!(
                    "`Grid` children must be wrapped in `Cell`, found `{}` (DD-M3-P5-001)",
                    child.widget_type
                )));
            }
            IrMember::ControlFlow(_) => {
                return Err(IrLoadError::Validate(
                    "`Grid` children must be wrapped in `Cell`; conditional members are not valid directly in runtime Grid IR".into(),
                ));
            }
        }
    }

    // Same-cell / overlapping-rectangle conflict (DD-M3-P5-003): no two
    // Cells share any resolved cell. `O(n_cells^2)` pairwise (trivial for
    // practical Grid sizes per DD-M3-P5-006).
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            if grid_rects_overlap(&rects[i], &rects[j]) {
                return Err(IrLoadError::Validate(format!(
                    "`Grid` Cell at (row {}, column {}) overlaps an earlier Cell's rectangle; same-cell and overlapping placements are rejected (DD-M3-P5-003)",
                    rects[j].row, rects[j].column
                )));
            }
        }
    }

    Ok(())
}

// Validate one `Cell` node's invariants and return its resolved
// rectangle (DD-M3-P5-001 / DD-M3-P5-003 / DD-M3-P5-005 / DD-M3-P5-006).
// Placement / span defaults match `extract_cell_placement` and
// `wasamoc lower`'s placement-default Option A (`row` / `column` absent →
// `0`; `row-span` / `column-span` absent → `1`). The multi-Cell
// placement-presence rule is compile-time-only (DD-M3-P5-006 marks it
// `(n/a)` at runtime); a multi-Cell Grid that omits placement is caught
// by the overlap check (two Cells both defaulting to `(0, 0)`).
fn validate_grid_cell(
    cell: &IrNode,
    columns_len: i64,
    rows_len: i64,
) -> Result<GridCellRect, IrLoadError> {
    // Cell single content child (DD-M3-P5-001).
    let widget_child_count = cell.widget_children().count();
    if widget_child_count != 1 {
        return Err(IrLoadError::Validate(format!(
            "`Cell` requires exactly one content child, got {} (DD-M3-P5-001)",
            widget_child_count
        )));
    }
    if cell
        .children
        .iter()
        .any(|m| matches!(m, IrMember::ControlFlow(_)))
    {
        return Err(IrLoadError::Validate(
            "`Cell` admits exactly one direct widget content child; put conditional members inside that content widget".into(),
        ));
    }

    // Placement / span values (Int literal positions). `wasamoc lower`
    // emits these as `IrLiteral::Int`; a non-Int literal is malformed.
    let row = grid_cell_int(cell, "row", 0)?;
    let column = grid_cell_int(cell, "column", 0)?;
    let row_span = grid_cell_int(cell, "row-span", 1)?;
    let column_span = grid_cell_int(cell, "column-span", 1)?;

    // Alignment vocabulary (DD-M3-P5-005): `h-align` / `v-align`, when
    // present, are idents in `{ start, center, end, stretch }`.
    validate_cell_alignment(cell, "h-align")?;
    validate_cell_alignment(cell, "v-align")?;

    // Placement value range (DD-M3-P5-003): row in `[0, rows.len())`,
    // column in `[0, columns.len())`.
    if row < 0 || row >= rows_len {
        return Err(IrLoadError::Validate(format!(
            "`Cell.row` {row} is out of range [0, {rows_len}) (DD-M3-P5-003)"
        )));
    }
    if column < 0 || column >= columns_len {
        return Err(IrLoadError::Validate(format!(
            "`Cell.column` {column} is out of range [0, {columns_len}) (DD-M3-P5-003)"
        )));
    }

    // Span value range (DD-M3-P5-003): spans `>= 1` and the resolved
    // rectangle fits within the declared track count.
    if row_span < 1 {
        return Err(IrLoadError::Validate(format!(
            "`Cell.row-span` must be a positive integer (>= 1), got {row_span} (DD-M3-P5-003)"
        )));
    }
    if column_span < 1 {
        return Err(IrLoadError::Validate(format!(
            "`Cell.column-span` must be a positive integer (>= 1), got {column_span} (DD-M3-P5-003)"
        )));
    }
    if row + row_span > rows_len {
        return Err(IrLoadError::Validate(format!(
            "`Cell` row span exceeds the grid: row {row} + row-span {row_span} = {} > {rows_len} declared row tracks (DD-M3-P5-003)",
            row + row_span
        )));
    }
    if column + column_span > columns_len {
        return Err(IrLoadError::Validate(format!(
            "`Cell` column span exceeds the grid: column {column} + column-span {column_span} = {} > {columns_len} declared column tracks (DD-M3-P5-003)",
            column + column_span
        )));
    }

    Ok(GridCellRect {
        row,
        column,
        row_span,
        column_span,
    })
}

/// Read a `Cell` placement / span attribute as an `i64`, defaulting when
/// the prop is absent. A present-but-non-`Int` literal is malformed.
fn grid_cell_int(cell: &IrNode, name: &str, default: i64) -> Result<i64, IrLoadError> {
    match cell.props.iter().find(|p| p.name == name) {
        Some(prop) => match &prop.value {
            IrLiteral::Int(n) => Ok(*n as i64),
            other => Err(IrLoadError::Validate(format!(
                "`Cell.{name}` must be an integer literal, got {other:?} (DD-M3-P5-003)"
            ))),
        },
        None => Ok(default),
    }
}

/// Validate a `Cell` alignment attribute against the DD-M3-P5-005
/// vocabulary when present. Absent is valid (defaults to `stretch`).
fn validate_cell_alignment(cell: &IrNode, name: &str) -> Result<(), IrLoadError> {
    let Some(prop) = cell.props.iter().find(|p| p.name == name) else {
        return Ok(());
    };
    match &prop.value {
        IrLiteral::Ident(v) if matches!(v.as_str(), "start" | "center" | "end" | "stretch") => {
            Ok(())
        }
        other => Err(IrLoadError::Validate(format!(
            "`Cell.{name}` must be one of start, center, end, stretch, got {other:?} (DD-M3-P5-005)"
        ))),
    }
}

/// Half-open rectangle overlap in track coordinates (DD-M3-P5-003).
fn grid_rects_overlap(a: &GridCellRect, b: &GridCellRect) -> bool {
    fn ranges_overlap(s1: i64, len1: i64, s2: i64, len2: i64) -> bool {
        s1 < s2 + len2 && s2 < s1 + len1
    }
    ranges_overlap(a.row, a.row_span, b.row, b.row_span)
        && ranges_overlap(a.column, a.column_span, b.column, b.column_span)
}

fn validate_node_references(
    node: &IrNode,
    declared: &std::collections::HashMap<&str, IrType>,
) -> Result<(), IrLoadError> {
    for binding in &node.bindings {
        validate_expr_references(&binding.expr, declared, &|name| {
            format!(
                "binding `{}` references undeclared name `{}`",
                binding.prop_name, name
            )
        })?;
    }
    for handler in &node.handlers {
        validate_expr_references(&handler.expr, declared, &|name| {
            format!(
                "handler `on {}` references undeclared name `{}`",
                handler.signal, name
            )
        })?;
    }
    for member in &node.children {
        validate_member_references(member, declared)?;
    }
    Ok(())
}

fn validate_member_references(
    member: &IrMember,
    declared: &std::collections::HashMap<&str, IrType>,
) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(node) => validate_node_references(node, declared),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                validate_condition_expr(&branch.condition, declared)?;
                for body_member in &branch.body {
                    validate_member_references(body_member, declared)?;
                }
            }
            Ok(())
        }
    }
}

fn validate_expr_references(
    expr: &HandlerExpr,
    declared: &std::collections::HashMap<&str, IrType>,
    err_msg: &dyn Fn(&str) -> String,
) -> Result<(), IrLoadError> {
    match expr {
        HandlerExpr::IntLit(_) | HandlerExpr::StrLit(_) | HandlerExpr::BoolLit(_) => Ok(()),
        HandlerExpr::PropRead { path }
        | HandlerExpr::StrPropRead { path }
        | HandlerExpr::BoolPropRead { path } => {
            if !declared.contains_key(path.as_str()) {
                Err(IrLoadError::Validate(err_msg(path)))
            } else {
                Ok(())
            }
        }
        HandlerExpr::Assign { lhs, rhs } => {
            if !declared.contains_key(lhs.as_str()) {
                return Err(IrLoadError::Validate(err_msg(lhs)));
            }
            validate_expr_references(rhs, declared, err_msg)
        }
        HandlerExpr::CompoundAssign { lhs, rhs, .. } => {
            if !declared.contains_key(lhs.as_str()) {
                return Err(IrLoadError::Validate(err_msg(lhs)));
            }
            validate_expr_references(rhs, declared, err_msg)
        }
        HandlerExpr::Interpolation(parts) => {
            for part in parts {
                if let InterpolationPart::Expr(inner) = part {
                    validate_expr_references(inner, declared, err_msg)?;
                }
            }
            Ok(())
        }
        HandlerExpr::Block(exprs) => {
            for inner in exprs {
                validate_expr_references(inner, declared, err_msg)?;
            }
            Ok(())
        }
    }
}

fn validate_condition_expr(
    expr: &HandlerExpr,
    declared: &std::collections::HashMap<&str, IrType>,
) -> Result<(), IrLoadError> {
    match expr {
        HandlerExpr::BoolLit(_) => Ok(()),
        HandlerExpr::BoolPropRead { path } => match declared.get(path.as_str()) {
            Some(IrType::Bool) => Ok(()),
            Some(other) => Err(IrLoadError::Validate(format!(
                "`if` condition `{path}` must resolve to bool, got {other:?}"
            ))),
            None => Err(IrLoadError::Validate(format!(
                "`if` condition references undeclared name `{path}`"
            ))),
        },
        HandlerExpr::PropRead { path } | HandlerExpr::StrPropRead { path } => {
            Err(IrLoadError::Validate(format!(
                "`if` condition `{path}` must use a bool condition expression"
            )))
        }
        other => Err(IrLoadError::Validate(format!(
            "`if` condition must be a bool literal or bool state read, got {other:?}"
        ))),
    }
}

fn check_and_strip_header(text: &str) -> Result<&str, IrLoadError> {
    let trimmed = text.trim_start();
    let line_end = trimmed.find('\n').unwrap_or(trimmed.len());
    let line = trimmed[..line_end].trim_end();
    if line != HEADER_MAGIC {
        return Err(IrLoadError::InvalidHeader(format!(
            "expected `{HEADER_MAGIC}`, got `{line}`"
        )));
    }
    if line_end < trimmed.len() {
        Ok(&trimmed[line_end + 1..])
    } else {
        Ok("")
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LBrace,
    RBrace,
    LParen,
    RParen,
    Eq,
    Colon,
    Ident(String),
    Int(i32),
    Str(String),
    AssignOp(CompoundOp),
    // M3-Phase 2 T7: ratio / color literal terminals (DD-M3-P2-002 /
    // DD-M3-P2-003). Both reach `parse_literal` only — they are not
    // valid in handler / binding expression position (no new
    // `HandlerExpr` variant per DD-M3-P2-004).
    Ratio { num: i32, den: i32 },
    Color(u32),
    // M3-Phase 5 T3: payload-less star terminal for Grid `tracks` lines
    // (DD-M3-P5-002). Mirrors `wasamoc`'s lexer decision (T1 R-A): the
    // lexer learns nothing about track lists — `Star` is recombined with
    // a preceding `Int` into a weighted-star track by `parse_track_list`.
    // Only reached in `tracks <axis> = …` position; a bare `*` elsewhere
    // surfaces a parser-level `expected …` diagnostic. `*=` stays
    // `AssignOp(Mul)`.
    Star,
}

fn tokenize(text: &str) -> Result<Vec<Token>, IrLoadError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == ';' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        match c {
            '{' => {
                tokens.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '=' => {
                tokens.push(Token::Eq);
                i += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
                i += 1;
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        let esc = chars[i + 1];
                        let ch = match esc {
                            '"' => '"',
                            '\\' => '\\',
                            'n' => '\n',
                            't' => '\t',
                            other => {
                                return Err(IrLoadError::Parse(format!(
                                    "unknown escape: \\{other}"
                                )));
                            }
                        };
                        s.push(ch);
                        i += 2;
                    } else {
                        s.push(chars[i]);
                        i += 1;
                    }
                }
                if i >= chars.len() {
                    return Err(IrLoadError::Parse("unterminated string literal".into()));
                }
                i += 1;
                tokens.push(Token::Str(s));
            }
            '+' | '/' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    let op = match c {
                        '+' => CompoundOp::Add,
                        '/' => CompoundOp::Div,
                        _ => unreachable!(),
                    };
                    tokens.push(Token::AssignOp(op));
                    i += 2;
                } else {
                    return Err(IrLoadError::Parse(format!("unexpected character: '{c}'")));
                }
            }
            // M3-Phase 5 T3: `*=` stays a compound-assign op; a bare `*`
            // emits the payload-less `Token::Star` (Grid `tracks` star
            // terminal, DD-M3-P5-002) instead of erroring.
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::AssignOp(CompoundOp::Mul));
                    i += 2;
                } else {
                    tokens.push(Token::Star);
                    i += 1;
                }
            }
            '-' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::AssignOp(CompoundOp::Sub));
                    i += 2;
                } else if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                    i += 1;
                    let start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let s: String = chars[start..i].iter().collect();
                    let n: i32 = s
                        .parse()
                        .map_err(|_| IrLoadError::Parse(format!("invalid integer: -{s}")))?;
                    tokens.push(Token::Int(-n));
                } else {
                    return Err(IrLoadError::Parse("unexpected character: '-'".into()));
                }
            }
            d if d.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let num_s: String = chars[start..i].iter().collect();
                let num: i32 = num_s
                    .parse()
                    .map_err(|_| IrLoadError::Parse(format!("invalid integer: {num_s}")))?;
                // M3-Phase 2 T7: ratio literal `<digits>:<digits>` (DD-M3-P2-002,
                // surface form Option A). Triggered only when `:` immediately
                // follows the integer with no intervening whitespace and a
                // digit immediately follows the `:`; otherwise the colon is
                // left for the `Colon` arm (e.g. `state name: type`). Mirrors
                // `wasamoc`'s lexer disambiguation in `scan_int_or_float`.
                if i + 1 < chars.len() && chars[i] == ':' && chars[i + 1].is_ascii_digit() {
                    i += 1; // consume ':'
                    let den_start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let den_s: String = chars[den_start..i].iter().collect();
                    let den: i32 = den_s.parse().map_err(|_| {
                        IrLoadError::Parse(format!("invalid ratio denominator: {den_s}"))
                    })?;
                    tokens.push(Token::Ratio { num, den });
                } else {
                    tokens.push(Token::Int(num));
                }
            }
            // M3-Phase 2 T7: color literal `#RRGGBB` / `#RRGGBBAA`
            // (DD-M3-P2-003, surface form Option A). Packed `0xAARRGGBB`
            // per dsl_spec §8.2 — `#RRGGBB` materialises with implicit
            // alpha `0xFF`. Mirrors `wasamoc::lexer::scan_color`.
            '#' => {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i].is_ascii_hexdigit() {
                    i += 1;
                }
                let hex: String = chars[start..i].iter().collect();
                let packed = match hex.len() {
                    6 => {
                        let rgb = u32::from_str_radix(&hex, 16).map_err(|_| {
                            IrLoadError::Parse(format!("invalid color literal: #{hex}"))
                        })?;
                        0xFF00_0000 | rgb
                    }
                    8 => {
                        let rgba = u32::from_str_radix(&hex, 16).map_err(|_| {
                            IrLoadError::Parse(format!("invalid color literal: #{hex}"))
                        })?;
                        ((rgba & 0xFF) << 24) | (rgba >> 8)
                    }
                    n => {
                        return Err(IrLoadError::Parse(format!(
                            "color literal `#{hex}` must have 6 or 8 hex digits, got {n}"
                        )));
                    }
                };
                tokens.push(Token::Color(packed));
            }
            l if l.is_ascii_alphabetic() || l == '_' => {
                let start = i;
                while i < chars.len() {
                    let cc = chars[i];
                    if cc.is_ascii_alphanumeric() || cc == '_' || cc == '-' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(s));
            }
            other => {
                return Err(IrLoadError::Parse(format!(
                    "unexpected character: '{other}'"
                )));
            }
        }
    }
    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), IrLoadError> {
        let actual = self
            .advance()
            .ok_or_else(|| IrLoadError::Parse(format!("expected {expected:?}, got EOF")))?;
        if actual != expected {
            return Err(IrLoadError::Parse(format!(
                "expected {expected:?}, got {actual:?}"
            )));
        }
        Ok(())
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), IrLoadError> {
        match self.advance() {
            Some(Token::Ident(s)) if s == kw => Ok(()),
            other => Err(IrLoadError::Parse(format!(
                "expected keyword `{kw}`, got {other:?}"
            ))),
        }
    }

    fn expect_ident(&mut self) -> Result<String, IrLoadError> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            other => Err(IrLoadError::Parse(format!(
                "expected identifier, got {other:?}"
            ))),
        }
    }

    fn parse_component(&mut self) -> Result<IrComponent, IrLoadError> {
        self.expect_keyword("component")?;
        let name = self.expect_ident()?;
        self.expect_keyword("inherits")?;
        let base = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut states = Vec::new();
        let mut root: Option<IrNode> = None;

        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::Ident(s)) if s == "state" => {
                    states.push(self.parse_state()?);
                }
                Some(Token::Ident(s)) if s == "node" => {
                    if root.is_some() {
                        return Err(IrLoadError::Parse(
                            "multiple root nodes in component".into(),
                        ));
                    }
                    root = Some(self.parse_node()?);
                }
                Some(other) => {
                    return Err(IrLoadError::Parse(format!(
                        "unexpected token in component body: {other:?}"
                    )));
                }
                None => {
                    return Err(IrLoadError::Parse(
                        "unexpected EOF in component body".into(),
                    ));
                }
            }
        }

        let root = root.ok_or_else(|| IrLoadError::Parse("component has no root node".into()))?;
        Ok(IrComponent {
            name,
            base,
            states,
            root,
        })
    }

    fn parse_state(&mut self) -> Result<IrState, IrLoadError> {
        self.expect_keyword("state")?;
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let ty_str = self.expect_ident()?;
        let ty = match ty_str.as_str() {
            "i32" => IrType::I32,
            "string" => IrType::Str,
            "bool" => IrType::Bool,
            other => {
                return Err(IrLoadError::Parse(format!("unknown state type: {other}")));
            }
        };
        self.expect(&Token::Eq)?;
        let default = self.parse_literal()?;
        Ok(IrState { name, ty, default })
    }

    fn parse_node(&mut self) -> Result<IrNode, IrLoadError> {
        self.expect_keyword("node")?;
        let widget_type = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut props = Vec::new();
        let mut bindings = Vec::new();
        let mut handlers = Vec::new();
        let mut children = Vec::new();
        // M3-Phase 5 T3: Grid `tracks <axis> = …` lines lower into
        // `KindPayload::Grid` (carrier c1), kept out of `props` so
        // `IrProp.value` stays strictly `IrLiteral`.
        let mut grid_columns: Option<Vec<TrackSize>> = None;
        let mut grid_rows: Option<Vec<TrackSize>> = None;

        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::Ident(s)) if s == "prop" => props.push(self.parse_prop()?),
                Some(Token::Ident(s)) if s == "bind" => bindings.push(self.parse_binding()?),
                Some(Token::Ident(s)) if s == "on" => handlers.push(self.parse_handler()?),
                Some(Token::Ident(s)) if s == "node" => {
                    children.push(IrMember::Widget(self.parse_node()?))
                }
                Some(Token::Ident(s)) if s == "if" => {
                    children.push(IrMember::ControlFlow(self.parse_if_member()?))
                }
                Some(Token::Ident(s)) if s == "tracks" => {
                    // `tracks` lines are Grid-only (DD-M3-P5-001 carrier
                    // c1 is a Grid-specific payload). Reject them on any
                    // other node at parse time, so `kind_payload` can only
                    // become `Some` on a `Grid` node — keeping the
                    // wasamo-ir "non-Grid → kind_payload None" invariant
                    // intact for textual IR. `validate()` is the
                    // defense-in-depth gate for IR built programmatically
                    // rather than parsed (see `validate_phase5`).
                    if widget_type != "Grid" {
                        return Err(IrLoadError::Parse(format!(
                            "`tracks` track list is only valid on a `Grid` node, found on `{widget_type}`"
                        )));
                    }
                    let (axis, tracks) = self.parse_tracks_line()?;
                    let slot = match axis.as_str() {
                        "columns" => &mut grid_columns,
                        "rows" => &mut grid_rows,
                        other => {
                            return Err(IrLoadError::Parse(format!(
                                "unknown track axis `{other}` (expected `columns` or `rows`)"
                            )));
                        }
                    };
                    if slot.is_some() {
                        return Err(IrLoadError::Parse(format!(
                            "duplicate `tracks {axis}` line on node"
                        )));
                    }
                    *slot = Some(tracks);
                }
                Some(other) => {
                    return Err(IrLoadError::Parse(format!(
                        "unexpected token in node body: {other:?}"
                    )));
                }
                None => {
                    return Err(IrLoadError::Parse("unexpected EOF in node body".into()));
                }
            }
        }

        // The Grid `kind_payload` is present iff at least one `tracks`
        // line was seen — i.e. iff this is a Grid node. A one-sided /
        // empty track list is left for `validate()` to reject (memory IR
        // reaching the loader via `wasamo_load_ui` is untrusted), unlike
        // `wasamoc lower` which panics on a both-or-neither violation
        // because `wasamoc check` has already guaranteed both lists.
        let kind_payload = if grid_columns.is_some() || grid_rows.is_some() {
            Some(KindPayload::Grid {
                columns: grid_columns.unwrap_or_default(),
                rows: grid_rows.unwrap_or_default(),
            })
        } else {
            None
        };

        Ok(IrNode {
            widget_type,
            props,
            bindings,
            handlers,
            children,
            kind_payload,
        })
    }

    fn parse_if_member(&mut self) -> Result<ControlFlowNode, IrLoadError> {
        self.expect_keyword("if")?;
        let condition = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let mut body = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::Ident(s)) if s == "node" => {
                    body.push(IrMember::Widget(self.parse_node()?));
                }
                Some(Token::Ident(s)) if s == "if" => {
                    body.push(IrMember::ControlFlow(self.parse_if_member()?));
                }
                Some(other) => {
                    return Err(IrLoadError::Parse(format!(
                        "unexpected token in if body: {other:?}"
                    )));
                }
                None => return Err(IrLoadError::Parse("unexpected EOF in if body".into())),
            }
        }
        Ok(ControlFlowNode::If {
            branches: vec![ControlFlowBranch { condition, body }],
        })
    }

    /// Parse a Grid `tracks <axis> = <track-list>` line (DD-M3-P5-002,
    /// carrier c1). The runtime IR is the canonical machine format
    /// emitted by `wasamoc` (`tracks columns = 180 1* 2*`), so the grammar
    /// is whitespace-insensitive: an `Int` immediately preceding a `Star`
    /// is a weighted-star track, a standalone `Int` is a fixed track, and
    /// a standalone `Star` is a unit `Star(1)` (the `1*`-vs-`1 *`
    /// author-surface adjacency distinction is already resolved at
    /// `wasamoc` compile time — see log.md T3 R-B Decision 3). The
    /// track-list reader stops at the next keyword / `RBrace`. Value
    /// ranges are enforced by `validate()`, not here.
    fn parse_tracks_line(&mut self) -> Result<(String, Vec<TrackSize>), IrLoadError> {
        self.expect_keyword("tracks")?;
        let axis = self.expect_ident()?;
        self.expect(&Token::Eq)?;
        let mut tracks = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Int(n)) => {
                    let n = *n;
                    self.advance();
                    if matches!(self.peek(), Some(Token::Star)) {
                        self.advance();
                        tracks.push(TrackSize::Star(n as u32));
                    } else {
                        tracks.push(TrackSize::Fixed(n));
                    }
                }
                Some(Token::Star) => {
                    self.advance();
                    tracks.push(TrackSize::Star(1));
                }
                _ => break,
            }
        }
        Ok((axis, tracks))
    }

    fn parse_prop(&mut self) -> Result<IrProp, IrLoadError> {
        self.expect_keyword("prop")?;
        let name = self.expect_ident()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_literal()?;
        Ok(IrProp { name, value })
    }

    fn parse_binding(&mut self) -> Result<IrBinding, IrLoadError> {
        self.expect_keyword("bind")?;
        let prop_name = self.expect_ident()?;
        self.expect(&Token::Eq)?;
        let expr = self.parse_expr()?;
        Ok(IrBinding { prop_name, expr })
    }

    fn parse_handler(&mut self) -> Result<IrHandler, IrLoadError> {
        self.expect_keyword("on")?;
        let signal = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::RBrace)?;
        Ok(IrHandler { signal, expr })
    }

    fn parse_literal(&mut self) -> Result<IrLiteral, IrLoadError> {
        match self.advance() {
            Some(Token::Int(n)) => Ok(IrLiteral::Int(*n)),
            Some(Token::Str(s)) => Ok(IrLiteral::Str(s.clone())),
            Some(Token::Ident(s)) if s == "true" => Ok(IrLiteral::Bool(true)),
            Some(Token::Ident(s)) if s == "false" => Ok(IrLiteral::Bool(false)),
            Some(Token::Ident(s)) => Ok(IrLiteral::Ident(s.clone())),
            // M3-Phase 2 T7: ratio / color literals reach this arm only —
            // there is no `HandlerExpr::RatioLit` / `ColorLit` (DD-M3-P2-004),
            // so binding / handler position cannot accept them. The
            // placement-level rejection (literal only valid on Box
            // `aspect` / `fill`) is enforced by `validate` below.
            Some(Token::Ratio { num, den }) => Ok(IrLiteral::Ratio {
                num: *num,
                den: *den,
            }),
            Some(Token::Color(value)) => Ok(IrLiteral::Color(*value)),
            other => Err(IrLoadError::Parse(format!(
                "expected literal, got {other:?}"
            ))),
        }
    }

    fn parse_expr(&mut self) -> Result<HandlerExpr, IrLoadError> {
        match self.peek() {
            Some(Token::Int(n)) => {
                let v = *n;
                self.advance();
                Ok(HandlerExpr::IntLit(v))
            }
            Some(Token::Str(s)) => {
                let v = s.clone();
                self.advance();
                Ok(HandlerExpr::StrLit(v))
            }
            Some(Token::Ident(s)) if s == "true" => {
                self.advance();
                Ok(HandlerExpr::BoolLit(true))
            }
            Some(Token::Ident(s)) if s == "false" => {
                self.advance();
                Ok(HandlerExpr::BoolLit(false))
            }
            Some(Token::LParen) => self.parse_sexpr(),
            other => Err(IrLoadError::Parse(format!(
                "expected expression, got {other:?}"
            ))),
        }
    }

    fn parse_sexpr(&mut self) -> Result<HandlerExpr, IrLoadError> {
        self.expect(&Token::LParen)?;
        let tag = self.expect_ident()?;
        let result = match tag.as_str() {
            "prop-read" => {
                let path = self.expect_ident()?;
                HandlerExpr::PropRead { path }
            }
            "str-prop-read" => {
                let path = self.expect_ident()?;
                HandlerExpr::StrPropRead { path }
            }
            "bool-prop-read" => {
                let path = self.expect_ident()?;
                HandlerExpr::BoolPropRead { path }
            }
            "assign" => {
                let lhs = self.expect_ident()?;
                let rhs = self.parse_expr()?;
                HandlerExpr::Assign {
                    lhs,
                    rhs: Box::new(rhs),
                }
            }
            "compound-assign" => {
                let op = match self.advance() {
                    Some(Token::AssignOp(op)) => *op,
                    other => {
                        return Err(IrLoadError::Parse(format!(
                            "expected compound op, got {other:?}"
                        )));
                    }
                };
                let lhs = self.expect_ident()?;
                let rhs = self.parse_expr()?;
                HandlerExpr::CompoundAssign {
                    op,
                    lhs,
                    rhs: Box::new(rhs),
                }
            }
            "interp" => {
                let mut parts = Vec::new();
                while !matches!(self.peek(), Some(Token::RParen)) {
                    parts.push(self.parse_interp_part()?);
                }
                HandlerExpr::Interpolation(parts)
            }
            "block" => {
                let mut exprs = Vec::new();
                while !matches!(self.peek(), Some(Token::RParen)) {
                    exprs.push(self.parse_expr()?);
                }
                HandlerExpr::Block(exprs)
            }
            other => {
                return Err(IrLoadError::Parse(format!("unknown S-expr tag: {other}")));
            }
        };
        self.expect(&Token::RParen)?;
        Ok(result)
    }

    fn parse_interp_part(&mut self) -> Result<InterpolationPart, IrLoadError> {
        match self.peek() {
            Some(Token::Str(s)) => {
                let s = s.clone();
                self.advance();
                Ok(InterpolationPart::Literal(s))
            }
            Some(Token::LParen) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(InterpolationPart::Expr(inner))
            }
            other => Err(IrLoadError::Parse(format!(
                "expected interp part, got {other:?}"
            ))),
        }
    }
}

// ── Builder (Win32/WinRT) ─────────────────────────────────────────────────────

/// Build a runtime widget tree from a parsed `IrComponent`, plus the
/// `SignalRegistry` populated from `state` declarations.
pub fn build_widget_tree(
    comp: &IrComponent,
    compositor: &Compositor,
    renderer: &TextRenderer,
) -> Result<BuiltUi, IrLoadError> {
    let registry = build_signal_registry(&comp.states);
    let registry = Rc::new(registry);
    // Install before building children so binding closures captured during
    // `build_node` see the registry — and so click-handler dispatch (which
    // happens after `wasamo::run()` enters the message loop) can find it.
    set_active_registry(Rc::clone(&registry));
    let root = build_node(&comp.root, compositor, renderer, &registry)?;
    Ok(BuiltUi { root, registry })
}

fn build_signal_registry(states: &[IrState]) -> SignalRegistry {
    let mut registry = SignalRegistry::new();
    for state in states {
        match state.ty {
            IrType::I32 => {
                let initial = match &state.default {
                    IrLiteral::Int(n) => *n,
                    _ => 0,
                };
                registry
                    .i32s
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrType::Str => {
                let initial = match &state.default {
                    IrLiteral::Str(s) => s.clone(),
                    _ => String::new(),
                };
                registry
                    .strings
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrType::Bool => {
                let initial = match &state.default {
                    IrLiteral::Bool(b) => *b,
                    _ => false,
                };
                registry
                    .bools
                    .insert(state.name.clone(), Signal::new(initial));
            }
        }
    }
    registry
}

fn build_node(
    node: &IrNode,
    compositor: &Compositor,
    renderer: &TextRenderer,
    registry: &Rc<SignalRegistry>,
) -> Result<Box<WidgetNode>, IrLoadError> {
    let mut widget = construct_widget(node, compositor, renderer, registry)?;

    // Bindings: register each `bind` as a reactive Effect targeting the widget property.
    for binding in &node.bindings {
        let Some((prop_key, prop_ty)) = resolve_prop_key(&node.widget_type, &binding.prop_name)
        else {
            // Unknown property name on this widget type — silently skip in M2.
            // M3 will surface this through the diagnostic system.
            continue;
        };
        let widget_id = WidgetId(widget.as_mut() as *mut WidgetNode as *mut ());
        let target = BindingTarget::WidgetProperty {
            node: widget_id,
            prop: prop_key,
        };
        // Per-type writer dispatch (DD-M3-P1-007 Option A + DD-M3-P1-009):
        // the loader selects the evaluator/writer pair matching the target
        // property's declared `IrType`. The reactive engine itself stays
        // type-agnostic; the seam lives here at the call site.
        let handle = match prop_ty {
            IrType::Bool => register_bool_binding(
                target,
                binding.expr.clone(),
                Rc::clone(registry),
                widget_write_property_bool,
            ),
            // I32 and Str properties continue through the M2 string-baked
            // writer (stringified by `evaluate_binding`, parsed at the
            // per-widget setter — typed-i32 writer lands when its use case
            // arrives).
            IrType::I32 | IrType::Str => register_binding(
                target,
                binding.expr.clone(),
                Rc::clone(registry),
                widget_write_property,
            ),
        };
        widget.bindings.push(handle);
    }

    // Handlers: attach each `on` body via Phase 3's set_inline_handler path.
    for handler in &node.handlers {
        widget.set_inline_handler(handler.signal.clone(), handler.expr.clone());
    }

    // Children: recurse and attach via the Phase 4 internal mutation API.
    //
    // M3-Phase 5 T3 (R-B): Grid bypasses the generic child loop. A Grid's
    // IR children are `Cell` wrappers — IR-only nodes that never
    // materialise as a `WidgetNode` (DD-M3-P5-001), so feeding them to
    // `build_node` / `construct_widget` would `UnknownWidget`. Instead each
    // Cell's single content child is built and appended directly, in
    // document order, so `WidgetNode.children` stays parallel to the
    // `cell_placements` that `construct_widget` extracted from the same
    // `node.children` in the same order (log.md T3 R-B Decision 2). The
    // single-content-child invariant was enforced by `validate()`
    // (DD-M3-P5-006); the `first()` guard is the defensive fallback.
    if node.widget_type == "Grid" {
        for cell in node.widget_children() {
            let content = cell.widget_children().next().ok_or_else(|| {
                IrLoadError::Build(format!(
                    "Grid `Cell` requires exactly one content child, got {}",
                    cell.widget_children().count()
                ))
            })?;
            let content_widget = build_node(content, compositor, renderer, registry)?;
            widget
                .append_child(content_widget)
                .map_err(|e| IrLoadError::Build(format!("append_child failed: {e:?}")))?;
        }
    } else {
        for member in &node.children {
            append_static_member(member, &mut widget, compositor, renderer, registry)?;
        }
    }

    Ok(widget)
}

fn append_static_member(
    member: &IrMember,
    parent: &mut WidgetNode,
    compositor: &Compositor,
    renderer: &TextRenderer,
    registry: &Rc<SignalRegistry>,
) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(child) => {
            let child_widget = build_node(child, compositor, renderer, registry)?;
            parent
                .append_child(child_widget)
                .map_err(|e| IrLoadError::Build(format!("append_child failed: {e:?}")))?;
        }
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            let branch = branches
                .first()
                .ok_or_else(|| IrLoadError::Build("`if` control flow has no branch".into()))?;
            if evaluate_static_condition(&branch.condition, registry)? {
                let body = match branch.body.first() {
                    Some(IrMember::Widget(node)) => node,
                    _ => {
                        return Err(IrLoadError::Build(
                            "`if` body must contain one widget member".into(),
                        ));
                    }
                };
                let child_widget = build_node(body, compositor, renderer, registry)?;
                parent
                    .append_child(child_widget)
                    .map_err(|e| IrLoadError::Build(format!("append_child failed: {e:?}")))?;
            }
        }
    }
    Ok(())
}

fn evaluate_static_condition(
    expr: &HandlerExpr,
    registry: &SignalRegistry,
) -> Result<bool, IrLoadError> {
    match expr {
        HandlerExpr::BoolLit(value) => Ok(*value),
        HandlerExpr::BoolPropRead { path } => registry
            .bools
            .get(path)
            .map(|signal| signal.get())
            .ok_or_else(|| IrLoadError::Build(format!("unknown bool state `{path}`"))),
        other => Err(IrLoadError::Build(format!(
            "`if` condition must be bool at build time, got {other:?}"
        ))),
    }
}

fn construct_widget(
    node: &IrNode,
    compositor: &Compositor,
    renderer: &TextRenderer,
    registry: &SignalRegistry,
) -> Result<Box<WidgetNode>, IrLoadError> {
    match node.widget_type.as_str() {
        "VStack" => {
            let spacing = extract_int_prop(&node.props, "spacing").unwrap_or(0) as f32;
            let padding = extract_int_prop(&node.props, "padding").unwrap_or(0) as f32;
            WidgetNode::vstack(compositor, spacing, padding, Alignment::Center)
                .map_err(|e| IrLoadError::Build(format!("vstack: {e}")))
        }
        "HStack" => {
            let spacing = extract_int_prop(&node.props, "spacing").unwrap_or(0) as f32;
            let padding = extract_int_prop(&node.props, "padding").unwrap_or(0) as f32;
            WidgetNode::hstack(compositor, spacing, padding, Alignment::Center)
                .map_err(|e| IrLoadError::Build(format!("hstack: {e}")))
        }
        "Text" => {
            let text = extract_str_prop(&node.props, "text").unwrap_or_default();
            let style = extract_typography(&node.props, "font");
            // If `text` will be filled by a binding, construct with empty content;
            // the binding's initial run writes the actual value during build.
            let initial = if has_binding(&node.bindings, "text") {
                String::new()
            } else {
                text
            };
            WidgetNode::text(compositor, renderer, &initial, style)
                .map_err(|e| IrLoadError::Build(format!("text: {e}")))
        }
        "Button" => {
            let label = extract_str_prop(&node.props, "text").unwrap_or_default();
            let style = extract_button_style(&node.props, "style");
            let initial = if has_binding(&node.bindings, "text") {
                String::new()
            } else {
                label
            };
            WidgetNode::button(compositor, renderer, &initial, style)
                .map_err(|e| IrLoadError::Build(format!("button: {e}")))
        }
        // M3-Phase 2 T7: Box materialisation. `IrLiteral::Ratio` and
        // `IrLiteral::Color` are unpacked directly into Box-internal
        // domain types (DD-M3-P2-002 / DD-M3-P2-003 Option A) — they
        // never round through `PropertyValue`. `validate` has already
        // refused any Ratio / Color outside Box `aspect` / `fill` and
        // any Box with >1 children, so the extracts here are total over
        // their accept set.
        "Box" => {
            let aspect = extract_ratio_prop(&node.props, "aspect");
            let fill = extract_color_prop(&node.props, "fill");
            WidgetNode::box_(compositor, aspect, fill)
                .map_err(|e| IrLoadError::Build(format!("box: {e}")))
        }
        // M3-Phase 3 T6: WrapPanel materialisation. The three kebab-case
        // attribute names from dsl_spec §4.10 carry through as `Option<i32>`
        // (presence-preserving) so the catalog can apply the absent-to-
        // default policy via `apply_wrap_panel_defaults` (DD-M3-P3-003 /
        // DD-M3-P3-004 Option (a)) — the loader stays default-knowledge-
        // free. `validate()` has already rejected negative IntLits on
        // these prop names (DD-M3-P3-006 runtime gate).
        "WrapPanel" => {
            let item_cross_size = extract_int_prop(&node.props, "item-cross-size");
            let item_spacing = extract_int_prop(&node.props, "item-spacing");
            let line_spacing = extract_int_prop(&node.props, "line-spacing");
            WidgetNode::wrap_panel(compositor, item_cross_size, item_spacing, line_spacing)
                .map_err(|e| IrLoadError::Build(format!("wrap_panel: {e}")))
        }
        // M3-Phase 4 T3: ScrollView materialisation. `offset-y` is the
        // sole DSL-surface attribute (DD-M3-P4-003 `i32` pixels), carried
        // through as `Option<i32>` so the catalog can apply the
        // absent-to-default policy (`unwrap_or(0)` per DD-M3-P4-003).
        // The runtime layer (`WidgetNode::scroll_view`) owns the default,
        // mirroring the Phase 3 WrapPanel `apply_*_defaults` discipline.
        // `validate()` has rejected 0-child and >1-child ScrollView IR
        // before this arm runs (DD-M3-P4-006 structural gate); negative
        // and out-of-range `offset-y` literals are layout-time-clamped
        // (DD-M3-P4-005), not loader-rejected. A binding on `offset-y`
        // appears on `node.bindings` and is wired through the generic
        // `build_node` binding loop; `resolve_prop_key`'s ScrollView
        // entry lands in T4 alongside the per-widget `set_property` arm.
        "ScrollView" => {
            let offset_y = extract_int_prop(&node.props, "offset-y");
            WidgetNode::scroll_view(compositor, offset_y)
                .map_err(|e| IrLoadError::Build(format!("scroll_view: {e}")))
        }
        // M3-Phase 5 T3: Grid materialisation (DD-M3-P5-001 carrier c1).
        // The track lists live on `node.kind_payload` (not `node.props` —
        // `IrProp.value` stays strictly `IrLiteral`); the per-Cell
        // placements are extracted from each `Cell` child's standard
        // `IrProp` entries here, so `WidgetData::Grid` carries the
        // layout-engine mirror types and `build_layout_tree` stays a
        // structural copy (log.md T3 R-B Decision 1). `validate()` has
        // already rejected malformed track lists / placements / overlaps
        // before this arm runs (DD-M3-P5-006). The `Cell` flattening
        // (appending each Cell's single content child) is the `build_node`
        // special case; this arm only builds the shell + placement vector.
        "Grid" => {
            let (columns, rows) = match &node.kind_payload {
                Some(KindPayload::Grid { columns, rows }) => (
                    columns.iter().map(to_layout_track_size).collect(),
                    rows.iter().map(to_layout_track_size).collect(),
                ),
                None => {
                    return Err(IrLoadError::Build(
                        "Grid node has no track-list payload (kind_payload)".into(),
                    ));
                }
            };
            let cell_placements = node.widget_children().map(extract_cell_placement).collect();
            WidgetNode::grid(compositor, columns, rows, cell_placements)
                .map_err(|e| IrLoadError::Build(format!("grid: {e}")))
        }
        // M3-Phase 6 T3: ZStack materialisation. Per-child placement
        // annotations are parent-owned and carried as a vector parallel to
        // direct children; document order is preserved by the generic child
        // append loop below.
        "ZStack" => {
            let placements = collect_static_zstack_placements(&node.children, registry)?;
            WidgetNode::zstack(compositor, placements)
                .map_err(|e| IrLoadError::Build(format!("zstack: {e}")))
        }
        other => Err(IrLoadError::UnknownWidget(other.to_string())),
    }
}

/// Convert an IR `TrackSize` (`wasamo_ir`) to the layout-engine mirror
/// (`layout::TrackSize`). Structural one-to-one (log.md T3 R-B
/// Decision 1); value ranges were already enforced by `validate()`.
fn to_layout_track_size(t: &TrackSize) -> LayoutTrackSize {
    match t {
        TrackSize::Fixed(n) => LayoutTrackSize::Fixed(*n),
        TrackSize::Star(w) => LayoutTrackSize::Star(*w),
    }
}

/// Extract a `Cell` IR node's placement into the layout-engine
/// `CellPlacement` (DD-M3-P5-003 / DD-M3-P5-005). Reads the standard
/// `IrProp` entries (`row` / `column` Int; `row-span` / `column-span`
/// Int; `h-align` / `v-align` Ident). Defaults match `wasamoc lower`'s
/// placement-default Option A and the runtime `validate()` gate: `row` /
/// `column` absent → `0`, `row-span` / `column-span` absent → `1`,
/// alignment absent → `Stretch` (DD-M3-P5-005 stretch default). Negative
/// / out-of-range values were already rejected by `validate()`; the
/// `as u32` casts here are total over the validated accept set.
fn extract_cell_placement(cell: &IrNode) -> CellPlacement {
    CellPlacement {
        row: extract_int_prop(&cell.props, "row").unwrap_or(0).max(0) as u32,
        column: extract_int_prop(&cell.props, "column").unwrap_or(0).max(0) as u32,
        row_span: extract_int_prop(&cell.props, "row-span")
            .unwrap_or(1)
            .max(1) as u32,
        column_span: extract_int_prop(&cell.props, "column-span")
            .unwrap_or(1)
            .max(1) as u32,
        h_align: extract_alignment_prop_or(&cell.props, "h-align", Alignment::Stretch),
        v_align: extract_alignment_prop_or(&cell.props, "v-align", Alignment::Stretch),
    }
}

fn extract_zstack_placement(child: &IrNode) -> ZStackPlacement {
    ZStackPlacement {
        h_align: extract_alignment_prop_or(&child.props, "h-align", Alignment::Center),
        v_align: extract_alignment_prop_or(&child.props, "v-align", Alignment::Center),
    }
}

fn collect_static_zstack_placements(
    members: &[IrMember],
    registry: &SignalRegistry,
) -> Result<Vec<ZStackPlacement>, IrLoadError> {
    let mut placements = Vec::new();
    for member in members {
        match member {
            IrMember::Widget(child) => placements.push(extract_zstack_placement(child)),
            IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
                let branch = branches
                    .first()
                    .ok_or_else(|| IrLoadError::Build("`if` control flow has no branch".into()))?;
                if evaluate_static_condition(&branch.condition, registry)? {
                    let body = match branch.body.first() {
                        Some(IrMember::Widget(node)) => node,
                        _ => {
                            return Err(IrLoadError::Build(
                                "`if` body must contain one widget member".into(),
                            ));
                        }
                    };
                    placements.push(extract_zstack_placement(body));
                }
            }
        }
    }
    Ok(placements)
}

/// Map a `Cell` alignment `IrProp` (`h-align` / `v-align`) to the layout
/// `Alignment`, defaulting to `Stretch` when absent (DD-M3-P5-005). The
/// vocabulary (`start` / `center` / `end` / `stretch`) was validated by
/// `validate()`; an unrecognised ident here falls back to `Stretch`
/// rather than failing, since this runs after the validate gate.
fn extract_alignment_prop_or(props: &[IrProp], name: &str, default: Alignment) -> Alignment {
    let ident = props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Ident(id) => Some(id.as_str()),
            _ => None,
        });
    match ident {
        Some("start") => Alignment::Leading,
        Some("center") => Alignment::Center,
        Some("end") => Alignment::Trailing,
        Some("stretch") => Alignment::Stretch,
        _ => default,
    }
}

// Widget catalog: `(widget_type, prop_name) → (PROP_* id, declared IrType)`.
// DD-M3-P1-009 widens the return shape so the binding loader can pick the
// per-type writer that matches the target property. The catalog mirrors the
// soft `wasamoc::check` widget-property table (kept independently so the
// compiler stays self-contained — see m3-phase-1-progress.md T3 Notes).
fn resolve_prop_key(widget_type: &str, prop_name: &str) -> Option<(PropertyKey, IrType)> {
    match (widget_type, prop_name) {
        ("Text", "text") => Some((PROP_TEXT_CONTENT, IrType::Str)),
        ("Text", "font") => Some((PROP_TEXT_STYLE, IrType::I32)),
        ("Button", "text") => Some((PROP_BUTTON_LABEL, IrType::Str)),
        ("Button", "style") => Some((PROP_BUTTON_STYLE, IrType::I32)),
        ("Button", "enabled") => Some((PROP_BUTTON_ENABLED, IrType::Bool)),
        // M3-Phase 4 T4 / DD-M3-P4-003: ScrollView's `offset-y` is `i32`
        // (DSL surface storage type). The `I32` selection here routes
        // the binding through the string-baked `register_binding` +
        // `widget_write_property` pair (per the `IrType::I32 |
        // IrType::Str` arm in `build_node`); the narrow string-to-`i32`
        // parse lives on the ScrollView arm of `set_property`. The
        // general typed-`i32` evaluator / writer pair from
        // architecture.md §6.8 *Per-type seam* stays deferred to M4+.
        ("ScrollView", "offset-y") => Some((PROP_SCROLLVIEW_OFFSET_Y, IrType::I32)),
        _ => None,
    }
}

fn extract_int_prop(props: &[IrProp], name: &str) -> Option<i32> {
    props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Int(n) => Some(*n),
            _ => None,
        })
}

fn extract_str_prop(props: &[IrProp], name: &str) -> Option<String> {
    props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Str(s) => Some(s.clone()),
            _ => None,
        })
}

fn extract_typography(props: &[IrProp], name: &str) -> TypographyStyle {
    let ident = props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Ident(id) => Some(id.as_str()),
            _ => None,
        });
    match ident {
        Some("caption") => TypographyStyle::Caption,
        Some("subtitle") => TypographyStyle::Subtitle,
        Some("title") => TypographyStyle::Title,
        _ => TypographyStyle::Body,
    }
}

fn extract_ratio_prop(props: &[IrProp], name: &str) -> Option<box_values::Ratio> {
    props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Ratio { num, den } => Some(box_values::Ratio {
                num: *num,
                den: *den,
            }),
            _ => None,
        })
}

fn extract_color_prop(props: &[IrProp], name: &str) -> Option<box_values::Color> {
    props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Color(value) => Some(box_values::Color(*value)),
            _ => None,
        })
}

fn extract_button_style(props: &[IrProp], name: &str) -> ButtonStyle {
    let ident = props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Ident(id) => Some(id.as_str()),
            _ => None,
        });
    match ident {
        Some("accent") => ButtonStyle::Accent,
        _ => ButtonStyle::Default,
    }
}

fn has_binding(bindings: &[IrBinding], name: &str) -> bool {
    bindings.iter().any(|b| b.prop_name == name)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_prop_key / binding dispatch (M3-Phase 1 T8 / DD-M3-P1-009) ──
    //
    // resolve_prop_key drives the per-type writer dispatch in `build_node`:
    // its returned `IrType` selects between `register_bool_binding`
    // (+ widget_write_property_bool) and the string-baked `register_binding`
    // path. End-to-end exercise lives in the Windows-bound integration test
    // for `Button.enabled` (T6); these tests cover the pure-logic seam.

    #[test]
    fn resolve_prop_key_button_enabled_is_bool() {
        let (key, ty) = resolve_prop_key("Button", "enabled").expect("Button.enabled exists");
        assert_eq!(key, PROP_BUTTON_ENABLED);
        assert_eq!(ty, IrType::Bool);
    }

    #[test]
    fn resolve_prop_key_text_text_is_string() {
        let (key, ty) = resolve_prop_key("Text", "text").expect("Text.text exists");
        assert_eq!(key, PROP_TEXT_CONTENT);
        assert_eq!(ty, IrType::Str);
    }

    #[test]
    fn resolve_prop_key_button_text_is_string() {
        let (key, ty) = resolve_prop_key("Button", "text").expect("Button.text exists");
        assert_eq!(key, PROP_BUTTON_LABEL);
        assert_eq!(ty, IrType::Str);
    }

    #[test]
    fn resolve_prop_key_button_style_is_i32() {
        let (key, ty) = resolve_prop_key("Button", "style").expect("Button.style exists");
        assert_eq!(key, PROP_BUTTON_STYLE);
        assert_eq!(ty, IrType::I32);
    }

    #[test]
    fn resolve_prop_key_text_font_is_i32() {
        let (key, ty) = resolve_prop_key("Text", "font").expect("Text.font exists");
        assert_eq!(key, PROP_TEXT_STYLE);
        assert_eq!(ty, IrType::I32);
    }

    #[test]
    fn static_condition_reducer_maps_bool_to_presence() {
        let mut registry = SignalRegistry::new();
        registry.bools.insert("open".into(), Signal::new(true));
        assert_eq!(
            evaluate_static_condition(
                &HandlerExpr::BoolPropRead {
                    path: "open".into()
                },
                &registry,
            ),
            Ok(true)
        );
        assert_eq!(
            evaluate_static_condition(&HandlerExpr::BoolLit(false), &registry),
            Ok(false)
        );
    }

    #[test]
    fn zstack_static_placements_follow_materialized_member_order() {
        fn text_with_align(h_align: &str, v_align: &str) -> IrNode {
            IrNode {
                widget_type: "Text".into(),
                props: vec![
                    IrProp {
                        name: "h-align".into(),
                        value: IrLiteral::Ident(h_align.into()),
                    },
                    IrProp {
                        name: "v-align".into(),
                        value: IrLiteral::Ident(v_align.into()),
                    },
                ],
                bindings: vec![],
                handlers: vec![],
                children: vec![],
                kind_payload: None,
            }
        }

        let mut registry = SignalRegistry::new();
        registry.bools.insert("open".into(), Signal::new(true));
        registry.bools.insert("closed".into(), Signal::new(false));
        let members = vec![
            IrMember::Widget(text_with_align("start", "start")),
            IrMember::ControlFlow(ControlFlowNode::If {
                branches: vec![ControlFlowBranch {
                    condition: HandlerExpr::BoolPropRead {
                        path: "open".into(),
                    },
                    body: vec![IrMember::Widget(text_with_align("end", "stretch"))],
                }],
            }),
            IrMember::ControlFlow(ControlFlowNode::If {
                branches: vec![ControlFlowBranch {
                    condition: HandlerExpr::BoolPropRead {
                        path: "closed".into(),
                    },
                    body: vec![IrMember::Widget(text_with_align("stretch", "end"))],
                }],
            }),
            IrMember::Widget(text_with_align("center", "center")),
        ];

        let placements = collect_static_zstack_placements(&members, &registry).unwrap();
        assert_eq!(placements.len(), 3);
        assert_eq!(placements[0].h_align, Alignment::Leading);
        assert_eq!(placements[0].v_align, Alignment::Leading);
        assert_eq!(placements[1].h_align, Alignment::Trailing);
        assert_eq!(placements[1].v_align, Alignment::Stretch);
        assert_eq!(placements[2].h_align, Alignment::Center);
        assert_eq!(placements[2].v_align, Alignment::Center);
    }

    // M3-Phase 4 T4 / DD-M3-P4-003: ScrollView's `offset-y` is `i32`.
    // The `I32` selection routes the binding through the string-baked
    // `register_binding` + `widget_write_property` pair (the `IrType::I32
    // | IrType::Str` arm in `build_node`); the narrow string-to-`i32`
    // parse lives on the ScrollView arm of `set_property`. Pins the
    // catalog row that closes the loop between the bound `Signal<i32>`
    // (declared in `state scroll_y: i32 = 0`) and the runtime
    // `WidgetData::ScrollView::offset_y` field.
    #[test]
    fn resolve_prop_key_scrollview_offset_y_is_i32() {
        let (key, ty) =
            resolve_prop_key("ScrollView", "offset-y").expect("ScrollView.offset-y exists");
        assert_eq!(key, PROP_SCROLLVIEW_OFFSET_Y);
        assert_eq!(ty, IrType::I32);
    }

    #[test]
    fn resolve_prop_key_unknown_pair_is_none() {
        assert!(resolve_prop_key("Button", "nonsuch").is_none());
        assert!(resolve_prop_key("Nonsuch", "enabled").is_none());
        // M3-Phase 4 T4: only `offset-y` resolves on ScrollView; any
        // other attribute name returns None and the binding loop's
        // "silently skip unknown property" branch fires (the
        // attribute-scope rejection itself is `wasamoc check`'s
        // responsibility — T1 already enforces it at compile time).
        assert!(resolve_prop_key("ScrollView", "scroll-axis").is_none());
        assert!(resolve_prop_key("ScrollView", "viewport-width").is_none());
    }

    fn parse_ok(src: &str) -> IrComponent {
        match parse_ir(src) {
            Ok(c) => c,
            Err(e) => panic!("parse failed: {e}\nsrc:\n{src}"),
        }
    }

    fn child_widget<'a>(node: &'a IrNode, index: usize) -> &'a IrNode {
        match &node.children[index] {
            IrMember::Widget(child) => child,
            other => panic!("expected widget child at {index}, got {other:?}"),
        }
    }

    #[test]
    fn header_required() {
        let err = parse_ir("component C inherits W { node V {} }").unwrap_err();
        assert!(matches!(err, IrLoadError::InvalidHeader(_)));
    }

    #[test]
    fn header_wrong_version() {
        let err = parse_ir(";wasamo-ir v9\ncomponent C inherits W { node V {} }").unwrap_err();
        assert!(matches!(err, IrLoadError::InvalidHeader(_)));
    }

    #[test]
    fn empty_component_with_root() {
        let c = parse_ok(";wasamo-ir v0\ncomponent C inherits W { node V {} }");
        assert_eq!(c.name, "C");
        assert_eq!(c.base, "W");
        assert!(c.states.is_empty());
        assert_eq!(c.root.widget_type, "V");
    }

    #[test]
    fn state_i32_with_int_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V {}\n}",
        );
        assert_eq!(c.states.len(), 1);
        assert_eq!(c.states[0].name, "count");
        assert_eq!(c.states[0].ty, IrType::I32);
        assert_eq!(c.states[0].default, IrLiteral::Int(0));
    }

    #[test]
    fn state_string_with_str_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state msg: string = \"hi\"\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].ty, IrType::Str);
        assert_eq!(c.states[0].default, IrLiteral::Str("hi".into()));
    }

    #[test]
    fn prop_int_and_ident_and_str() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V {\n\
               prop spacing = 12\n\
               prop theme = system\n\
               prop title = \"Hi\"\n\
             }\n}",
        );
        let props = &c.root.props;
        assert_eq!(props.len(), 3);
        assert_eq!(
            props[0],
            IrProp {
                name: "spacing".into(),
                value: IrLiteral::Int(12)
            }
        );
        assert_eq!(
            props[1],
            IrProp {
                name: "theme".into(),
                value: IrLiteral::Ident("system".into())
            }
        );
        assert_eq!(
            props[2],
            IrProp {
                name: "title".into(),
                value: IrLiteral::Str("Hi".into())
            }
        );
    }

    #[test]
    fn nested_nodes() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V {\n\
               node Text {}\n\
               node Button {}\n\
             }\n}",
        );
        assert_eq!(c.root.widget_type, "V");
        assert_eq!(c.root.children.len(), 2);
        assert_eq!(child_widget(&c.root, 0).widget_type, "Text");
        assert_eq!(child_widget(&c.root, 1).widget_type, "Button");
    }

    #[test]
    fn control_flow_if_parses_as_member_with_single_widget_body() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node VStack {\n\
               if (bool-prop-read ready) { node Text { prop text = \"Shown\" } }\n\
             }\n}",
        );
        match &c.root.children[0] {
            IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
                assert_eq!(branches.len(), 1);
                assert_eq!(
                    branches[0].condition,
                    HandlerExpr::BoolPropRead {
                        path: "ready".into()
                    }
                );
                assert_eq!(branches[0].body.len(), 1);
                assert_eq!(
                    match &branches[0].body[0] {
                        IrMember::Widget(node) => node.widget_type.as_str(),
                        other => panic!("expected widget body, got {other:?}"),
                    },
                    "Text"
                );
            }
            other => panic!("expected control-flow member, got {other:?}"),
        }
    }

    #[test]
    fn control_flow_roundtrip_preserves_condition_and_body() {
        let original = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = false\n\
             node VStack { if (bool-prop-read ready) { node Text {} } }\n\
             }",
        );
        let reparsed = parse_ok(&render(&original));
        assert_eq!(reparsed, original);
    }

    #[test]
    fn binding_with_prop_read() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V { bind text = (prop-read count) }\n}",
        );
        assert_eq!(c.root.bindings.len(), 1);
        let b = &c.root.bindings[0];
        assert_eq!(b.prop_name, "text");
        assert_eq!(
            b.expr,
            HandlerExpr::PropRead {
                path: "count".into()
            }
        );
    }

    #[test]
    fn binding_with_str_prop_read() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state label: string = \"hi\"\n\
             node V { bind text = (str-prop-read label) }\n}",
        );
        assert_eq!(c.root.bindings.len(), 1);
        let b = &c.root.bindings[0];
        assert_eq!(b.prop_name, "text");
        assert_eq!(
            b.expr,
            HandlerExpr::StrPropRead {
                path: "label".into()
            }
        );
    }

    #[test]
    fn binding_with_interpolation() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V { bind text = (interp \"Count: \" ((prop-read count))) }\n}",
        );
        let b = &c.root.bindings[0];
        assert_eq!(
            b.expr,
            HandlerExpr::Interpolation(vec![
                InterpolationPart::Literal("Count: ".into()),
                InterpolationPart::Expr(HandlerExpr::PropRead {
                    path: "count".into()
                }),
            ])
        );
    }

    #[test]
    fn handler_with_compound_assign() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V { on clicked { (compound-assign += count 1) } }\n}",
        );
        assert_eq!(c.root.handlers.len(), 1);
        let h = &c.root.handlers[0];
        assert_eq!(h.signal, "clicked");
        assert_eq!(
            h.expr,
            HandlerExpr::CompoundAssign {
                op: CompoundOp::Add,
                lhs: "count".into(),
                rhs: Box::new(HandlerExpr::IntLit(1)),
            }
        );
    }

    #[test]
    fn handler_with_assign_and_block() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state x: i32 = 0\n\
             state y: i32 = 0\n\
             node V { on clicked { (block (assign x 1) (assign y 2)) } }\n}",
        );
        let h = &c.root.handlers[0];
        let HandlerExpr::Block(exprs) = &h.expr else {
            panic!("expected block, got {:?}", h.expr);
        };
        assert_eq!(exprs.len(), 2);
        assert_eq!(
            exprs[0],
            HandlerExpr::Assign {
                lhs: "x".into(),
                rhs: Box::new(HandlerExpr::IntLit(1))
            }
        );
    }

    #[test]
    fn state_bool_with_false_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = false\n\
             node V {}\n}",
        );
        assert_eq!(c.states.len(), 1);
        assert_eq!(c.states[0].name, "ready");
        assert_eq!(c.states[0].ty, IrType::Bool);
        assert_eq!(c.states[0].default, IrLiteral::Bool(false));
    }

    #[test]
    fn state_bool_with_true_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].ty, IrType::Bool);
        assert_eq!(c.states[0].default, IrLiteral::Bool(true));
    }

    #[test]
    fn prop_bool_literal() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Button {\n\
               prop enabled = true\n\
             }\n}",
        );
        let props = &c.root.props;
        assert_eq!(
            props[0],
            IrProp {
                name: "enabled".into(),
                value: IrLiteral::Bool(true)
            }
        );
    }

    #[test]
    fn binding_with_bool_prop_read() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = false\n\
             node Button { bind enabled = (bool-prop-read ready) }\n}",
        );
        assert_eq!(c.root.bindings.len(), 1);
        let b = &c.root.bindings[0];
        assert_eq!(b.prop_name, "enabled");
        assert_eq!(
            b.expr,
            HandlerExpr::BoolPropRead {
                path: "ready".into()
            }
        );
    }

    #[test]
    fn binding_with_bool_literal() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Button { bind enabled = true }\n}",
        );
        assert_eq!(c.root.bindings[0].expr, HandlerExpr::BoolLit(true));
    }

    #[test]
    fn handler_assign_bool_literal() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node Button { on clicked { (assign ready false) } }\n}",
        );
        let h = &c.root.handlers[0];
        assert_eq!(
            h.expr,
            HandlerExpr::Assign {
                lhs: "ready".into(),
                rhs: Box::new(HandlerExpr::BoolLit(false))
            }
        );
    }

    #[test]
    fn handler_assign_bool_prop_read() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state a: bool = false\n\
             state b: bool = true\n\
             node V { on clicked { (assign a (bool-prop-read b)) } }\n}",
        );
        let h = &c.root.handlers[0];
        assert_eq!(
            h.expr,
            HandlerExpr::Assign {
                lhs: "a".into(),
                rhs: Box::new(HandlerExpr::BoolPropRead { path: "b".into() })
            }
        );
    }

    #[test]
    fn negative_int_literal() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state x: i32 = -5\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].default, IrLiteral::Int(-5));
    }

    #[test]
    fn parse_full_counter_round_trip() {
        // Build an IrComponent identical in shape to what wasamoc emits for
        // counter.ui, render it through a minimal hand-rolled emit (mirroring
        // wasamoc::emit's grammar §8), and assert parse_ir yields back the
        // same structure. This covers the full grammar surface (header,
        // state, prop, bind+interp, on+compound-assign, nested nodes).
        let original = IrComponent {
            name: "Counter".into(),
            base: "Window".into(),
            states: vec![IrState {
                name: "count".into(),
                ty: IrType::I32,
                default: IrLiteral::Int(0),
            }],
            root: IrNode {
                widget_type: "VStack".into(),
                props: vec![
                    IrProp {
                        name: "spacing".into(),
                        value: IrLiteral::Int(12),
                    },
                    IrProp {
                        name: "padding".into(),
                        value: IrLiteral::Int(24),
                    },
                ],
                bindings: vec![],
                handlers: vec![],
                children: vec![
                    IrMember::Widget(IrNode {
                        widget_type: "Text".into(),
                        props: vec![IrProp {
                            name: "font".into(),
                            value: IrLiteral::Ident("title".into()),
                        }],
                        bindings: vec![IrBinding {
                            prop_name: "text".into(),
                            expr: HandlerExpr::Interpolation(vec![
                                InterpolationPart::Literal("Count: ".into()),
                                InterpolationPart::Expr(HandlerExpr::PropRead {
                                    path: "count".into(),
                                }),
                            ]),
                        }],
                        handlers: vec![],
                        children: vec![],
                        kind_payload: None,
                    }),
                    IrMember::Widget(IrNode {
                        widget_type: "Button".into(),
                        props: vec![
                            IrProp {
                                name: "text".into(),
                                value: IrLiteral::Str("Increment".into()),
                            },
                            IrProp {
                                name: "style".into(),
                                value: IrLiteral::Ident("accent".into()),
                            },
                        ],
                        bindings: vec![],
                        handlers: vec![IrHandler {
                            signal: "clicked".into(),
                            expr: HandlerExpr::CompoundAssign {
                                op: CompoundOp::Add,
                                lhs: "count".into(),
                                rhs: Box::new(HandlerExpr::IntLit(1)),
                            },
                        }],
                        children: vec![],
                        kind_payload: None,
                    }),
                ],
                kind_payload: None,
            },
        };

        let text = render(&original);
        let parsed = parse_ok(&text);
        assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");
    }

    /// Minimal IR text renderer mirroring `wasamoc::emit` (§8 normative grammar).
    /// Kept in the test module so the parser test does not require `wasamoc` as
    /// a dev-dependency; correctness vs. the compiler's emitter is enforced by
    /// the cross-crate round-trip integration test in
    /// `wasamo-runtime/tests/ir_loader_roundtrip.rs`.
    fn render(c: &IrComponent) -> String {
        let mut out = String::new();
        out.push_str(";wasamo-ir v0\n\n");
        out.push_str(&format!("component {} inherits {} {{\n", c.name, c.base));
        for s in &c.states {
            let ty = match s.ty {
                IrType::I32 => "i32",
                IrType::Str => "string",
                IrType::Bool => "bool",
            };
            out.push_str(&format!(
                "    state {}: {} = {}\n",
                s.name,
                ty,
                render_lit(&s.default)
            ));
        }
        if !c.states.is_empty() {
            out.push('\n');
        }
        render_node(&mut out, &c.root, 1);
        out.push_str("}\n");
        out
    }

    fn render_node(out: &mut String, n: &IrNode, depth: usize) {
        let i = "    ".repeat(depth);
        out.push_str(&format!("{i}node {} {{\n", n.widget_type));
        for p in &n.props {
            out.push_str(&format!(
                "{i}    prop {} = {}\n",
                p.name,
                render_lit(&p.value)
            ));
        }
        for b in &n.bindings {
            out.push_str(&format!(
                "{i}    bind {} = {}\n",
                b.prop_name,
                render_expr(&b.expr)
            ));
        }
        for h in &n.handlers {
            out.push_str(&format!("{i}    on {} {{\n", h.signal));
            out.push_str(&format!("{i}        {}\n", render_expr(&h.expr)));
            out.push_str(&format!("{i}    }}\n"));
        }
        for child in &n.children {
            match child {
                IrMember::Widget(node) => render_node(out, node, depth + 1),
                IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
                    let i = "  ".repeat(depth + 1);
                    for branch in branches {
                        out.push_str(&format!("{}if {} {{\n", i, render_expr(&branch.condition)));
                        for body_member in &branch.body {
                            if let IrMember::Widget(node) = body_member {
                                render_node(out, node, depth + 2);
                            }
                        }
                        out.push_str(&format!("{}}}\n", i));
                    }
                }
            }
        }
        out.push_str(&format!("{i}}}\n"));
    }

    fn render_lit(l: &IrLiteral) -> String {
        match l {
            IrLiteral::Int(n) => n.to_string(),
            IrLiteral::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            IrLiteral::Ident(id) => id.clone(),
            IrLiteral::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
            IrLiteral::Ratio { num, den } => format!("{}:{}", num, den),
            IrLiteral::Color(value) => {
                let alpha = (*value >> 24) & 0xFF;
                let rgb = *value & 0x00FF_FFFF;
                if alpha == 0xFF {
                    format!("#{:06x}", rgb)
                } else {
                    format!("#{:06x}{:02x}", rgb, alpha)
                }
            }
        }
    }

    fn render_expr(e: &HandlerExpr) -> String {
        match e {
            HandlerExpr::IntLit(n) => n.to_string(),
            HandlerExpr::StrLit(s) => format!("\"{}\"", s),
            HandlerExpr::BoolLit(b) => (if *b { "true" } else { "false" }).to_string(),
            HandlerExpr::PropRead { path } => format!("(prop-read {})", path),
            HandlerExpr::StrPropRead { path } => format!("(str-prop-read {})", path),
            HandlerExpr::BoolPropRead { path } => format!("(bool-prop-read {})", path),
            HandlerExpr::Assign { lhs, rhs } => format!("(assign {} {})", lhs, render_expr(rhs)),
            HandlerExpr::CompoundAssign { lhs, op, rhs } => {
                let op_str = match op {
                    CompoundOp::Add => "+=",
                    CompoundOp::Sub => "-=",
                    CompoundOp::Mul => "*=",
                    CompoundOp::Div => "/=",
                };
                format!("(compound-assign {} {} {})", op_str, lhs, render_expr(rhs))
            }
            HandlerExpr::Interpolation(parts) => {
                let inner: Vec<String> = parts
                    .iter()
                    .map(|p| match p {
                        InterpolationPart::Literal(s) => format!("\"{}\"", s),
                        InterpolationPart::Expr(e) => format!("({})", render_expr(e)),
                    })
                    .collect();
                format!("(interp {})", inner.join(" "))
            }
            HandlerExpr::Block(exprs) => {
                if exprs.is_empty() {
                    "(block)".to_string()
                } else {
                    let inner: Vec<String> = exprs.iter().map(render_expr).collect();
                    format!("(block {})", inner.join(" "))
                }
            }
        }
    }

    // ── DD-M2-P6-009 defense-in-depth validation tests ──────────────────

    fn parse_err(src: &str) -> IrLoadError {
        match parse_ir(src) {
            Ok(_) => panic!("expected parse error, but parse succeeded:\n{src}"),
            Err(e) => e,
        }
    }

    fn assert_malformed_display_nonempty(err: &IrLoadError) {
        assert!(
            err.is_malformed(),
            "expected malformed-class error, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            !msg.is_empty(),
            "Display impl must produce non-empty message for {err:?}"
        );
    }

    // Top-level structure ----------------------------------------------------

    #[test]
    fn malformed_no_root_node() {
        let err = parse_err(";wasamo-ir v0\ncomponent C inherits W { }");
        assert!(matches!(err, IrLoadError::Parse(ref m) if m.contains("no root node")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_multiple_root_nodes() {
        let err = parse_err(";wasamo-ir v0\ncomponent C inherits W { node V {} node W {} }");
        assert!(matches!(err, IrLoadError::Parse(ref m) if m.contains("multiple root nodes")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_missing_component_keyword() {
        let err = parse_err(";wasamo-ir v0\nfoo C inherits W { node V {} }");
        assert!(matches!(err, IrLoadError::Parse(_)));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_missing_inherits_keyword() {
        let err = parse_err(";wasamo-ir v0\ncomponent C foo W { node V {} }");
        assert!(matches!(err, IrLoadError::Parse(_)));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_trailing_tokens() {
        let err = parse_err(";wasamo-ir v0\ncomponent C inherits W { node V {} } stray");
        assert!(matches!(err, IrLoadError::Parse(ref m) if m.contains("trailing tokens")));
        assert_malformed_display_nonempty(&err);
    }

    // Reference resolution ---------------------------------------------------

    #[test]
    fn malformed_propread_undeclared() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { bind text = (prop-read missing) }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("missing")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_bool_prop_read_undeclared() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { bind enabled = (bool-prop-read missing) }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("missing")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_assign_undeclared() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { on clicked { (assign nope 1) } }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("nope")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_compound_assign_undeclared() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { on clicked { (compound-assign += counter 1) } }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("counter")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_undeclared_inside_interpolation() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { bind text = (interp \"x: \" ((prop-read ghost))) }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("ghost")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_undeclared_inside_block() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state x: i32 = 0\n\
             node V { on clicked { (block (assign x 1) (assign y 2)) } }\n}",
        );
        // `x` is fine; `y` is not.
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("y")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_undeclared_in_child_node() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node Root {\n\
               node Inner { bind text = (prop-read missing) }\n\
             }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("missing")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_duplicate_state_name() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             state count: string = \"hi\"\n\
             node V {}\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("duplicate")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn validate_passes_with_all_references_declared() {
        // Sanity-check: every reference resolves, so parse_ir succeeds.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             state label: string = \"hi\"\n\
             node Root {\n\
               node Text { bind text = (interp \"n=\" ((prop-read count))) }\n\
               node Button { on clicked { (compound-assign += count 1) } }\n\
             }\n}",
        );
        assert_eq!(c.states.len(), 2);
    }

    #[test]
    fn header_error_is_malformed_class() {
        let err = parse_err("not-a-header\ncomponent C inherits W { node V {} }");
        assert!(matches!(err, IrLoadError::InvalidHeader(_)));
        assert_malformed_display_nonempty(&err);
    }

    // ── M3-Phase 2 T7: ratio / color literal lex + parse + placement ─────
    //
    // These tests cover the pure-logic surface of T7. The `build_node`
    // materialisation path (IR → `WidgetData::Box`) needs a live
    // `Compositor` and is exercised end-to-end by T10's Box round-trip
    // integration test. The accept-shape lex / parse / placement / single-
    // child invariants are testable without a Compositor and live here.

    #[test]
    fn ratio_literal_in_prop_position() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { prop aspect = 16:9 }\n}",
        );
        assert_eq!(c.root.widget_type, "Box");
        assert_eq!(c.root.props.len(), 1);
        assert_eq!(c.root.props[0].name, "aspect");
        assert_eq!(c.root.props[0].value, IrLiteral::Ratio { num: 16, den: 9 });
    }

    #[test]
    fn color_literal_short_form_packs_implicit_alpha_ff() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { prop fill = #cccccc }\n}",
        );
        // `#cccccc` materialises with implicit alpha `0xFF` in the MSB
        // (dsl_spec §8.2 packing).
        assert_eq!(c.root.props[0].value, IrLiteral::Color(0xFF_CC_CC_CC));
    }

    #[test]
    fn color_literal_long_form_carries_explicit_alpha() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { prop fill = #00000080 }\n}",
        );
        // `#00000080`: RR=GG=BB=0x00, AA=0x80 → packed `0x80_00_00_00`.
        assert_eq!(c.root.props[0].value, IrLiteral::Color(0x80_00_00_00));
    }

    #[test]
    fn color_literal_long_form_with_full_rgba() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { prop fill = #11223344 }\n}",
        );
        // `#11223344`: RR=0x11, GG=0x22, BB=0x33, AA=0x44 →
        // packed `0x44_11_22_33`.
        assert_eq!(c.root.props[0].value, IrLiteral::Color(0x44_11_22_33));
    }

    #[test]
    fn box_phase2_load_side_fixture() {
        // ADR §Phase 2 verification closure item 2 (load-side gate at the
        // parse level — the build_node materialisation half lands in T10's
        // Windows-only `wasamo-runtime/tests/box_round_trip.rs`). For the
        // fixture
        // `Box { aspect: 16:9; fill: #00000080; Text { text: "Photo 12" } }`,
        // assert the post-parse `IrLiteral` variants match the emit-side
        // fixture `box_phase2_ir_text_emit_fixture` in `wasamoc::emit`.
        // The two halves together establish that the literal types
        // survive both directions of the IR text grammar.
        let src = ";wasamo-ir v0\n\
                   component C inherits W {\n\
                       node Box {\n\
                           prop aspect = 16:9\n\
                           prop fill = #00000080\n\
                           node Text { prop text = \"Photo 12\" }\n\
                       }\n\
                   }";
        let c = parse_ok(src);
        assert_eq!(c.root.widget_type, "Box");
        let aspect = c
            .root
            .props
            .iter()
            .find(|p| p.name == "aspect")
            .expect("aspect prop");
        let fill = c
            .root
            .props
            .iter()
            .find(|p| p.name == "fill")
            .expect("fill prop");
        assert_eq!(aspect.value, IrLiteral::Ratio { num: 16, den: 9 });
        assert_eq!(fill.value, IrLiteral::Color(0x80_00_00_00));
        assert_eq!(c.root.children.len(), 1);
        assert_eq!(child_widget(&c.root, 0).widget_type, "Text");
    }

    #[test]
    fn color_must_be_six_or_eight_hex_digits() {
        // 5 hex digits — neither short nor long form.
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { prop fill = #aabbc }\n}",
        );
        assert!(matches!(err, IrLoadError::Parse(ref m) if m.contains("6 or 8 hex digits")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_ratio_outside_box_aspect_on_vstack() {
        // DD-M3-P2-002: Ratio literal valid only on Box.aspect.
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { prop aspect = 16:9 }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("ratio") && m.contains("VStack"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_ratio_on_box_wrong_prop_name() {
        // Ratio on Box but in a prop slot other than `aspect`.
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { prop spacing = 16:9 }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("ratio")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_color_outside_box_fill_on_text() {
        // DD-M3-P2-003: Color literal valid only on Box.fill.
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Text { prop fill = #cccccc }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("color") && m.contains("Text"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_color_on_box_wrong_prop_name() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { prop aspect = #cccccc }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("color")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_ratio_in_nested_node() {
        // The walk recurses — a Ratio on a child VStack inside Box is
        // still rejected.
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box {\n\
               node VStack { prop aspect = 4:3 }\n\
             }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("ratio")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_box_with_two_children() {
        // DD-M3-P2-001: Box accepts at most one child.
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box {\n\
               node Text {}\n\
               node Text {}\n\
             }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("Box") && m.contains("at most one child"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn box_with_zero_children_is_valid() {
        // Box without a child is valid (per DD-M3-P2-005 no-aspect bounded
        // Box / aspect-only scrim use case).
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { prop fill = #cccccc }\n}",
        );
        assert!(c.root.children.is_empty());
    }

    #[test]
    fn box_with_single_child_is_valid() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { node Text { prop text = \"hi\" } }\n}",
        );
        assert_eq!(c.root.children.len(), 1);
        assert_eq!(child_widget(&c.root, 0).widget_type, "Text");
    }

    // ── M3-Phase 3 T6: WrapPanel validate() defense-in-depth ─────────────
    //
    // These tests cover the pure-logic `validate()` half of T6. The
    // `construct_widget` materialisation half needs a live `Compositor`
    // and is exercised end-to-end by T8's Windows-only integration test.
    // DD-M3-P3-006 runtime gate: negative values on `item-cross-size` /
    // `item-spacing` / `line-spacing` are rejected; zero is *valid*
    // (rejection threshold is `< 0`).

    #[test]
    fn wrap_panel_zero_children_is_valid() {
        // DD-M3-P3-001 no-lower-bound: 0+ children, no upper bound.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel {}\n}",
        );
        assert_eq!(c.root.widget_type, "WrapPanel");
        assert!(c.root.children.is_empty());
    }

    #[test]
    fn wrap_panel_single_child_is_valid() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel { node Text { prop text = \"hi\" } }\n}",
        );
        assert_eq!(c.root.children.len(), 1);
        assert_eq!(child_widget(&c.root, 0).widget_type, "Text");
    }

    #[test]
    fn wrap_panel_multi_child_is_valid() {
        // DD-M3-P3-001: no upper bound on child count.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel {\n\
               node Text {}\n\
               node Text {}\n\
               node Text {}\n\
               node Text {}\n\
             }\n}",
        );
        assert_eq!(c.root.children.len(), 4);
    }

    #[test]
    fn wrap_panel_rejects_negative_item_cross_size() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel { prop item-cross-size = -1 }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("item-cross-size") && m.contains("non-negative"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn wrap_panel_rejects_negative_item_spacing() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel { prop item-spacing = -5 }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("item-spacing") && m.contains("non-negative"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn wrap_panel_rejects_negative_line_spacing() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel { prop line-spacing = -10 }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("line-spacing") && m.contains("non-negative"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn wrap_panel_accepts_zero_on_all_three_attributes() {
        // DD-M3-P3-006 zero-handling: the rejection threshold is `< 0`,
        // not `<= 0`. Zero is a *valid* setting on every WrapPanel
        // integer attribute — `Some(0)` for `item-cross-size` means
        // uniform zero per-line cross-axis size, `0` spacings mean
        // touching items / lines.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel {\n\
               prop item-cross-size = 0\n\
               prop item-spacing = 0\n\
               prop line-spacing = 0\n\
             }\n}",
        );
        assert_eq!(c.root.widget_type, "WrapPanel");
    }

    #[test]
    fn wrap_panel_accepts_positive_values_on_all_three_attributes() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel {\n\
               prop item-cross-size = 96\n\
               prop item-spacing = 8\n\
               prop line-spacing = 12\n\
             }\n}",
        );
        assert_eq!(c.root.props.len(), 3);
    }

    #[test]
    fn wrap_panel_negative_value_in_nested_node_is_rejected() {
        // The walk recurses — a negative attribute on a child WrapPanel
        // nested inside another container is still rejected.
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack {\n\
               node WrapPanel { prop item-spacing = -3 }\n\
             }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("item-spacing")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn ratio_lex_requires_colon_immediately_after_digits() {
        // `prop spacing = 12` followed by `node Text` must tokenize the
        // `12` as Int, not snare the next colon (there isn't one here —
        // this guards against the digit lookahead misfiring across
        // whitespace).
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { prop spacing = 12 node Text {} }\n}",
        );
        assert_eq!(c.root.props[0].value, IrLiteral::Int(12));
        assert_eq!(c.root.children.len(), 1);
    }

    // ── M3-Phase 4 T3: ScrollView validate() defense-in-depth ───────────
    //
    // Pure-logic coverage of DD-M3-P4-006's compound-shape gate:
    //   - structural child-count rejection (exactly-1-child contract from
    //     DD-M3-P4-001) at validate() time;
    //   - value-range pass-through for `offset-y` (negative and very
    //     large values reach the layout engine, which clamps them per
    //     DD-M3-P4-005).
    // The `construct_widget` "ScrollView" arm needs a live `Compositor`
    // and is exercised end-to-end by T4's Windows-only integration test.

    #[test]
    fn scroll_view_with_single_child_is_valid() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView { node Box {} }\n}",
        );
        assert_eq!(c.root.widget_type, "ScrollView");
        assert_eq!(c.root.children.len(), 1);
        assert_eq!(child_widget(&c.root, 0).widget_type, "Box");
    }

    #[test]
    fn scroll_view_with_zero_children_rejected() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView {}\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("ScrollView") && m.contains("exactly one"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn scroll_view_with_two_children_rejected() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView {\n\
               node Box {}\n\
               node Box {}\n\
             }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("ScrollView") && m.contains("exactly one"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn scroll_view_with_three_children_rejected() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView {\n\
               node Box {}\n\
               node Box {}\n\
               node Box {}\n\
             }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("ScrollView") && m.contains("exactly one"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn scroll_view_nested_zero_child_is_rejected() {
        // The walk recurses — a 0-child ScrollView nested inside a
        // structural parent is still rejected.
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack {\n\
               node ScrollView {}\n\
             }\n}",
        );
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("ScrollView") && m.contains("exactly one"))
        );
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn scroll_view_accepts_negative_offset_y_literal() {
        // DD-M3-P4-006 compound-shape: value-range invariants are
        // layout-time-clamped, *not* validate()-rejected. A negative
        // `offset-y` literal must reach the layout engine for the
        // clamp to run.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView { prop offset-y = -5 node Box {} }\n}",
        );
        let offset = c
            .root
            .props
            .iter()
            .find(|p| p.name == "offset-y")
            .expect("offset-y prop");
        assert_eq!(offset.value, IrLiteral::Int(-5));
    }

    #[test]
    fn scroll_view_accepts_very_large_offset_y_literal() {
        // Far past any plausible content extent — must still pass
        // validate(). The arrange-time clamp narrows it down to
        // `[0, max_offset]` per DD-M3-P4-005.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView { prop offset-y = 2000000 node Box {} }\n}",
        );
        let offset = c
            .root
            .props
            .iter()
            .find(|p| p.name == "offset-y")
            .expect("offset-y prop");
        assert_eq!(offset.value, IrLiteral::Int(2_000_000));
    }

    #[test]
    fn scroll_view_accepts_offset_y_state_binding() {
        // `offset-y` may be bound to a state identifier (DD-M3-P4-003
        // bindable read-only). validate() resolves the reference
        // against the declared `state` table; layout consumes the
        // bound value at run time.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state scroll_y: i32 = 0\n\
             node ScrollView { bind offset-y = (prop-read scroll_y) node Box {} }\n}",
        );
        assert_eq!(c.root.widget_type, "ScrollView");
        assert_eq!(c.root.bindings.len(), 1);
        assert_eq!(c.root.bindings[0].prop_name, "offset-y");
    }

    #[test]
    fn ratio_lex_does_not_capture_state_colon() {
        // `state count: i32 = 0` — the `count` ident is followed by `:`,
        // but the digit lookahead is anchored at the digit side, so `count:`
        // remains Ident + Colon and parsing proceeds normally.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V {}\n}",
        );
        assert_eq!(c.states.len(), 1);
        assert_eq!(c.states[0].default, IrLiteral::Int(0));
    }

    // ── M3-Phase 5 T3: Grid `tracks` parse + validate() invariants ──────
    //
    // These cover ADR Phase 5 verification closure item (3) — the
    // IR-loader `validate()` invariant evidence (DD-M3-P5-006). Pure
    // logic: `parse_ir` tokenises the carrier-c1 textual IR (the
    // `tracks <axis> = …` lines T1's `wasamoc emit` produces), parses
    // into `KindPayload::Grid`, and runs the Phase 5 `validate()` gate,
    // all without a `Compositor`.

    fn assert_validate_err(src: &str, needle: &str) {
        match parse_err(src) {
            IrLoadError::Validate(msg) => assert!(
                msg.contains(needle),
                "validate message `{msg}` did not contain `{needle}`"
            ),
            other => panic!("expected Validate error, got {other:?}"),
        }
    }

    // ── tracks parse (carrier c1) ───────────────────────────────────────

    #[test]
    fn grid_tracks_parse_into_kind_payload() {
        // `tracks columns = 180 1* 2*` — fixed + two weighted stars;
        // `tracks rows = 1* 1*`. Unit star arrives canonicalised as `1*`.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 180 1* 2*\n\
               tracks rows = 1* 1*\n\
               node Cell { prop row = 0 prop column = 0 node Text {} }\n\
             }\n}",
        );
        assert_eq!(c.root.widget_type, "Grid");
        match c.root.kind_payload.as_ref().expect("Grid kind_payload") {
            KindPayload::Grid { columns, rows } => {
                assert_eq!(
                    columns,
                    &[
                        TrackSize::Fixed(180),
                        TrackSize::Star(1),
                        TrackSize::Star(2)
                    ]
                );
                assert_eq!(rows, &[TrackSize::Star(1), TrackSize::Star(1)]);
            }
        }
        // Track lists never leak into `props` (carrier-c1 invariant).
        assert!(c
            .root
            .props
            .iter()
            .all(|p| p.name != "columns" && p.name != "rows"));
    }

    #[test]
    fn grid_bare_star_parses_as_unit_weight() {
        // A standalone `*` (no preceding integer) is a unit `Star(1)`.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = *\n\
               tracks rows = *\n\
               node Cell { prop row = 0 prop column = 0 node Text {} }\n\
             }\n}",
        );
        match c.root.kind_payload.as_ref().unwrap() {
            KindPayload::Grid { columns, rows } => {
                assert_eq!(columns, &[TrackSize::Star(1)]);
                assert_eq!(rows, &[TrackSize::Star(1)]);
            }
        }
    }

    #[test]
    fn non_grid_node_has_no_kind_payload() {
        let c = parse_ok(";wasamo-ir v0\ncomponent C inherits W { node VStack {} }");
        assert!(c.root.kind_payload.is_none());
    }

    #[test]
    fn star_compound_assign_still_lexes() {
        // `*=` must stay `AssignOp(Mul)` after the bare-`*` → `Token::Star`
        // split, so handler bodies keep compiling.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node Button { on click { (compound-assign *= count 2) } }\n}",
        );
        assert_eq!(c.root.handlers.len(), 1);
    }

    // A minimal valid 2×2 Grid wrapper used as the positive control and
    // the base for negative mutations.
    fn valid_grid_src(cell_body: &str) -> String {
        format!(
            ";wasamo-ir v0\ncomponent C inherits W {{\n\
             node Grid {{\n\
               tracks columns = 1* 1*\n\
               tracks rows = 1* 1*\n\
               {cell_body}\n\
             }}\n}}"
        )
    }

    #[test]
    fn grid_positive_control_validates() {
        // Fixed + weighted-star tracks, a spanning Cell and three
        // single-cell Cells — all placements distinct, all in range.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 180 1* 2*\n\
               tracks rows = 1* 1*\n\
               node Cell { prop row = 0 prop column = 0 prop column-span = 3 node Text {} }\n\
               node Cell { prop row = 1 prop column = 0 node Text {} }\n\
               node Cell { prop row = 1 prop column = 1 prop h-align = center node Text {} }\n\
               node Cell { prop row = 1 prop column = 2 node Text {} }\n\
             }\n}",
        );
        assert_eq!(c.root.children.len(), 4);
    }

    // ── min row / column count ──────────────────────────────────────────

    #[test]
    fn grid_missing_column_track_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks rows = 1*\n\
               node Cell { prop row = 0 prop column = 0 node Text {} }\n\
             }\n}",
            "at least one column track",
        );
    }

    #[test]
    fn grid_missing_row_track_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 1*\n\
               node Cell { prop row = 0 prop column = 0 node Text {} }\n\
             }\n}",
            "at least one row track",
        );
    }

    // ── track value range ───────────────────────────────────────────────

    #[test]
    fn grid_zero_fixed_track_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 0\n\
               tracks rows = 1*\n\
               node Cell { prop row = 0 prop column = 0 node Text {} }\n\
             }\n}",
            "fixed track size must be a positive integer",
        );
    }

    #[test]
    fn grid_star_weight_over_cap_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 1025*\n\
               tracks rows = 1*\n\
               node Cell { prop row = 0 prop column = 0 node Text {} }\n\
             }\n}",
            "star weight must be in [1, 1024]",
        );
    }

    // ── placement value range ───────────────────────────────────────────

    #[test]
    fn grid_cell_column_out_of_range_rejected() {
        assert_validate_err(
            &valid_grid_src("node Cell { prop row = 0 prop column = 2 node Text {} }"),
            "`Cell.column` 2 is out of range [0, 2)",
        );
    }

    #[test]
    fn grid_cell_row_out_of_range_rejected() {
        assert_validate_err(
            &valid_grid_src("node Cell { prop row = 5 prop column = 0 node Text {} }"),
            "`Cell.row` 5 is out of range [0, 2)",
        );
    }

    // ── span value range ────────────────────────────────────────────────

    #[test]
    fn grid_cell_zero_span_rejected() {
        assert_validate_err(
            &valid_grid_src(
                "node Cell { prop row = 0 prop column = 0 prop column-span = 0 node Text {} }",
            ),
            "`Cell.column-span` must be a positive integer",
        );
    }

    #[test]
    fn grid_cell_span_exceeds_grid_rejected() {
        assert_validate_err(
            &valid_grid_src(
                "node Cell { prop row = 0 prop column = 1 prop column-span = 2 node Text {} }",
            ),
            "column span exceeds the grid",
        );
    }

    // ── Cell child-count ────────────────────────────────────────────────

    #[test]
    fn grid_cell_zero_content_children_rejected() {
        assert_validate_err(
            &valid_grid_src("node Cell { prop row = 0 prop column = 0 }"),
            "`Cell` requires exactly one content child, got 0",
        );
    }

    #[test]
    fn grid_cell_two_content_children_rejected() {
        assert_validate_err(
            &valid_grid_src("node Cell { prop row = 0 prop column = 0 node Text {} node Text {} }"),
            "`Cell` requires exactly one content child, got 2",
        );
    }

    // ── same-cell / overlapping-rectangle conflict ──────────────────────

    #[test]
    fn grid_same_cell_conflict_rejected() {
        assert_validate_err(
            &valid_grid_src(
                "node Cell { prop row = 0 prop column = 0 node Text {} }\n\
                 node Cell { prop row = 0 prop column = 0 node Text {} }",
            ),
            "overlaps an earlier Cell's rectangle",
        );
    }

    #[test]
    fn grid_overlapping_span_conflict_rejected() {
        // A 1×2 spanning Cell at (0,0)-(0,1) overlaps a single Cell at
        // (0,1).
        assert_validate_err(
            &valid_grid_src(
                "node Cell { prop row = 0 prop column = 0 prop column-span = 2 node Text {} }\n\
                 node Cell { prop row = 0 prop column = 1 node Text {} }",
            ),
            "overlaps an earlier Cell's rectangle",
        );
    }

    #[test]
    fn grid_multi_cell_omitted_placement_collides_at_origin() {
        // Runtime validate() does not enforce the compile-time-only
        // multi-Cell placement-presence rule; two Cells omitting `row` /
        // `column` both default to (0, 0) and are caught by the overlap
        // gate (DD-M3-P5-006 defense-in-depth).
        assert_validate_err(
            &valid_grid_src(
                "node Cell { node Text {} }\n\
                 node Cell { node Text {} }",
            ),
            "overlaps an earlier Cell's rectangle",
        );
    }

    // ── alignment vocabulary ────────────────────────────────────────────

    #[test]
    fn grid_cell_unknown_alignment_rejected() {
        assert_validate_err(
            &valid_grid_src(
                "node Cell { prop row = 0 prop column = 0 prop h-align = middle node Text {} }",
            ),
            "`Cell.h-align` must be one of start, center, end, stretch",
        );
    }

    // ── non-Cell Grid child / Cell outside Grid ─────────────────────────

    #[test]
    fn grid_non_cell_child_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 1*\n\
               tracks rows = 1*\n\
               node Text {}\n\
             }\n}",
            "children must be wrapped in `Cell`",
        );
    }

    #[test]
    fn cell_outside_grid_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { node Cell { prop row = 0 prop column = 0 node Text {} } }\n}",
            "`Cell` is only valid as a direct child of a `Grid`",
        );
    }

    #[test]
    fn grid_node_without_tracks_rejected() {
        // A `Grid` node with no `tracks` line carries no kind_payload and
        // is rejected (min-shape defense-in-depth).
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node Grid {} }",
            "requires `columns:` and `rows:` track lists",
        );
    }

    // ── non-Grid kind_payload invariant (carrier c1 is Grid-only) ───────

    #[test]
    fn tracks_on_non_grid_node_rejected_at_parse() {
        // The textual parser restricts `tracks` to `Grid` nodes, so a
        // `kind_payload` can never become `Some` on a non-Grid node via
        // parsing (keeps the wasamo-ir "non-Grid → None" invariant).
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Text { tracks columns = 1* tracks rows = 1* }\n}",
        );
        match err {
            IrLoadError::Parse(msg) => assert!(
                msg.contains("`tracks` track list is only valid on a `Grid` node"),
                "got: {msg}"
            ),
            other => panic!("expected Parse error, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_non_grid_kind_payload() {
        // Defense-in-depth gate for IR built programmatically rather than
        // parsed: a non-Grid node carrying a Grid `kind_payload` violates
        // the carrier-c1 invariant (DD-M3-P5-001) and `validate()` rejects
        // it directly, independent of the textual parser's `tracks`
        // restriction.
        let comp = IrComponent {
            name: "C".into(),
            base: "W".into(),
            states: vec![],
            root: IrNode {
                widget_type: "Text".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![],
                kind_payload: Some(KindPayload::Grid {
                    columns: vec![TrackSize::Star(1)],
                    rows: vec![TrackSize::Star(1)],
                }),
            },
        };
        match validate(&comp) {
            Err(IrLoadError::Validate(msg)) => {
                assert!(msg.contains("only valid on a `Grid` node"), "got: {msg}")
            }
            other => panic!("expected Validate error, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_cell_with_kind_payload() {
        // A `Cell` is also a non-Grid node; a Grid payload on a Cell is
        // rejected by the per-Cell defense-in-depth check inside
        // `validate_grid_invariants`.
        let cell = IrNode {
            widget_type: "Cell".into(),
            props: vec![
                IrProp {
                    name: "row".into(),
                    value: IrLiteral::Int(0),
                },
                IrProp {
                    name: "column".into(),
                    value: IrLiteral::Int(0),
                },
            ],
            bindings: vec![],
            handlers: vec![],
            children: vec![IrMember::Widget(IrNode {
                widget_type: "Text".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![],
                kind_payload: None,
            })],
            kind_payload: Some(KindPayload::Grid {
                columns: vec![TrackSize::Star(1)],
                rows: vec![TrackSize::Star(1)],
            }),
        };
        let comp = IrComponent {
            name: "C".into(),
            base: "W".into(),
            states: vec![],
            root: IrNode {
                widget_type: "Grid".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![IrMember::Widget(cell)],
                kind_payload: Some(KindPayload::Grid {
                    columns: vec![TrackSize::Star(1)],
                    rows: vec![TrackSize::Star(1)],
                }),
            },
        };
        match validate(&comp) {
            Err(IrLoadError::Validate(msg)) => {
                assert!(msg.contains("only valid on a `Grid` node"), "got: {msg}")
            }
            other => panic!("expected Validate error, got {other:?}"),
        }
    }

    // ── M3-Phase 6 T3: ZStack validate() defense-in-depth ──────────────

    #[test]
    fn zstack_positive_control_validates_direct_children() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack {\n\
               node Box { prop fill = #336699cc }\n\
               node Text { prop h-align = end prop v-align = start prop text = \"caption\" }\n\
             }\n}",
        );
        assert_eq!(c.root.widget_type, "ZStack");
        assert_eq!(c.root.children.len(), 2);
    }

    #[test]
    fn zstack_zero_children_validates() {
        let c = parse_ok(";wasamo-ir v0\ncomponent C inherits W { node ZStack {} }");
        assert_eq!(c.root.widget_type, "ZStack");
        assert!(c.root.children.is_empty());
    }

    #[test]
    fn validate_rejects_if_with_non_bool_condition() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node VStack { if (prop-read count) { node Text {} } }\n\
             }",
            "must use a bool condition",
        );
    }

    #[test]
    fn validate_rejects_if_with_bool_read_resolving_to_non_bool_state() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node VStack { if (bool-prop-read count) { node Text {} } }\n\
             }",
            "must resolve to bool",
        );
    }

    #[test]
    fn validate_rejects_if_with_unresolved_condition() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { if (bool-prop-read missing) { node Text {} } }\n\
             }",
            "undeclared name",
        );
    }

    #[test]
    fn validate_rejects_if_with_empty_body() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { if true { } }\n\
             }",
            "exactly one widget member",
        );
    }

    #[test]
    fn validate_rejects_if_with_multi_child_body() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { if true { node Text {} node Button {} } }\n\
             }",
            "exactly one widget member",
        );
    }

    #[test]
    fn validate_rejects_if_with_nested_control_flow_body() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { if true { if false { node Text {} } } }\n\
             }",
            "nested control-flow",
        );
    }

    #[test]
    fn validate_rejects_invalid_subtree_inside_if_body() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { if true { node Box { node Text {} node Text {} } } }\n\
             }",
            "`Box` node accepts at most one child",
        );
    }

    #[test]
    fn validate_rejects_direct_conditional_grid_member() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid { tracks columns = 1* tracks rows = 1* if true { node Cell { node Text {} } } }\n\
             }",
            "conditional members are not valid directly in runtime Grid IR",
        );
    }

    #[test]
    fn validate_rejects_direct_conditional_cell_member() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid { tracks columns = 1* tracks rows = 1* node Cell { node VStack {} if true { node Text {} } } }\n\
             }",
            "put conditional members inside that content widget",
        );
    }

    #[test]
    fn zstack_attribute_rejected_at_validate() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { prop spacing = 8 node Text {} }\n}",
            "`ZStack` accepts no Phase-6 attributes",
        );
    }

    #[test]
    fn zstack_binding_rejected_at_validate() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node ZStack { bind h-align = (bool-prop-read ready) node Text {} }\n}",
            "`ZStack` accepts no Phase-6 bindings",
        );
    }

    #[test]
    fn zstack_handler_rejected_at_validate() {
        // The handler-rejection arm of the Phase-6 ZStack gate is distinct
        // from the binding arm above; pin it so a ZStack carrying an inline
        // `on` handler surfaces the dedicated diagnostic.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node ZStack { on clicked { (assign ready false) } node Text {} }\n}",
            "`ZStack` accepts no Phase-6 handlers",
        );
    }

    #[test]
    fn zstack_child_unknown_alignment_rejected_at_validate() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { node Text { prop h-align = middle } }\n}",
            "`Text.h-align` must be one of start, center, end, stretch",
        );
    }

    #[test]
    fn placement_prop_outside_zstack_child_or_grid_cell_rejected_at_validate() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { node Text { prop h-align = center } }\n}",
            "valid only on a ZStack direct child or Grid `Cell`",
        );
    }

    #[test]
    fn validate_rejects_zstack_with_kind_payload() {
        let comp = IrComponent {
            name: "C".into(),
            base: "W".into(),
            states: vec![],
            root: IrNode {
                widget_type: "ZStack".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![],
                kind_payload: Some(KindPayload::Grid {
                    columns: vec![TrackSize::Star(1)],
                    rows: vec![TrackSize::Star(1)],
                }),
            },
        };
        match validate(&comp) {
            Err(IrLoadError::Validate(msg)) => {
                assert!(msg.contains("only valid on a `Grid` node"), "got: {msg}")
            }
            other => panic!("expected Validate error, got {other:?}"),
        }
    }
}
