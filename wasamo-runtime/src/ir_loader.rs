//! IR loader (DD-M2-P6-006) — parses the normative Wasamo IR text grammar
//! (DD-M2-P6-002) and constructs the runtime widget tree.
//!
//! The module is split in two: a pure-logic parser (`parse_ir`, testable
//! without any Win32/WinRT dependency) and a builder (`build_widget_tree`,
//! requires a live `Compositor` and `TextRenderer`). The C ABI front-end
//! (`wasamo_load_ui`) is wired in DD-M2-P6-005 — this module exposes the
//! Rust-level entry points only.

use std::cell::RefCell;
use std::rc::Rc;

use wasamo_ir::{
    CompoundOp, ControlFlowBranch, ControlFlowNode, HandlerExpr, InterpolationPart, IrAlignment,
    IrBinding, IrChildSlot, IrComponent, IrHandler, IrLiteral, IrMember, IrNode, IrProp,
    IrSlotData, IrState, IrStateType, IrType, KindPayload, TrackSize,
};

use crate::box_values;
use crate::layout::{
    Alignment, CellPlacement, SlotData, TrackSize as LayoutTrackSize, ZStackPlacement,
};
use crate::reactive::{
    register_binding, register_bool_binding, register_conditional_binding,
    register_for_item_binding, register_for_item_bool_binding, register_for_loop_binding,
    set_active_registry, BindingTarget, ForItemContext, PropertyKey, Signal, SignalRegistry,
    WidgetId,
};
use crate::text::{TextRenderer, TypographyStyle};
use crate::widget::{
    widget_write_property, widget_write_property_bool, ButtonStyle, WidgetNode,
    PROP_BUTTON_ENABLED, PROP_BUTTON_LABEL, PROP_BUTTON_STYLE, PROP_SCROLLVIEW_OFFSET_Y,
    PROP_TEXT_CONTENT, PROP_TEXT_STYLE, PROP_TOGGLEBUTTON_CHECKED,
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

#[derive(Clone)]
enum DeclaredMemberSlot {
    Widget,
    Conditional(Rc<RefCell<ConditionalRuntimeState>>),
    ForLoop(Rc<RefCell<ForLoopRuntimeState>>),
}

struct ConditionalRuntimeState {
    live_child: bool,
}

struct ForLoopRuntimeState {
    live_children: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TailRangePlan {
    Insert { start: usize, count: usize },
    Remove { tail_first_indices: Vec<usize> },
    NoOp,
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

    #[doc(hidden)]
    pub fn __set_bool_state_for_test(&self, name: &str, value: bool) -> bool {
        match self.registry.bools.get(name) {
            Some(signal) => {
                signal.set(value);
                true
            }
            None => false,
        }
    }

    #[doc(hidden)]
    pub fn __set_string_list_state_for_test(&self, name: &str, value: Vec<String>) -> bool {
        match self.registry.string_lists.get(name) {
            Some(signal) => signal.set_if_changed(value),
            None => false,
        }
    }

    #[doc(hidden)]
    pub fn __set_i32_list_state_for_test(&self, name: &str, value: Vec<i32>) -> bool {
        match self.registry.i32_lists.get(name) {
            Some(signal) => signal.set_if_changed(value),
            None => false,
        }
    }

    #[doc(hidden)]
    pub fn __set_bool_list_state_for_test(&self, name: &str, value: Vec<bool>) -> bool {
        match self.registry.bool_lists.get(name) {
            Some(signal) => signal.set_if_changed(value),
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
    let mut comp = p.parse_component()?;
    if p.pos < p.tokens.len() {
        return Err(IrLoadError::Parse(format!(
            "unexpected trailing tokens after component (token #{})",
            p.pos
        )));
    }
    annotate_collection_expr_types(&mut comp)?;
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
        validate_state_default(state)?;
    }
    validate_host_surface(comp)?;
    reject_root_squatted_host_attrs(comp)?;
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
    // M4-Phase 2 T6 defense-in-depth gate (dsl_spec §4.19, DD-M4-P2-005
    // A1). `wasamoc check` (T6 stage 1) rejects the same `focus-group` /
    // `modal-scope` / `dismiss` shapes at compile time; this is the
    // runtime half for memory IR that reaches the loader via
    // `wasamo_load_ui` without traversing `wasamoc`. Runs early — before
    // the per-kind gates below (ZStack, ToggleButton) — so a
    // focus-annotation violation surfaces its own admission diagnostic
    // rather than being swallowed by a per-kind "unknown attribute"
    // catch-all.
    validate_focus_annotation_invariants(&comp.root)?;
    // M4-Phase 2 T8 defense-in-depth gate (dsl_spec §4.19, DD-M4-P2-005).
    // `wasamoc check` (T8) rejects the same `key-down` argument shapes
    // at compile time; this is the runtime half for memory IR that
    // reaches the loader via `wasamo_load_ui` without traversing
    // `wasamoc`. See `validate_key_down_invariants`'s doc comment for
    // why this is a sibling pass rather than folded into
    // `validate_focus_annotation_invariants` above.
    validate_key_down_invariants(&comp.root)?;
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
    validate_phase6_control_flow_invariants(&comp.root)?;
    validate_phase7_iteration_invariants(&comp.root, false)?;
    validate_phase8_togglebutton_node_invariants(&comp.root, &declared, None)
}

fn validate_state_default(state: &IrState) -> Result<(), IrLoadError> {
    match (&state.ty, &state.default) {
        (IrStateType::Scalar(IrType::I32), IrLiteral::Int(_))
        | (IrStateType::Scalar(IrType::Str), IrLiteral::Str(_))
        | (IrStateType::Scalar(IrType::Bool), IrLiteral::Bool(_)) => Ok(()),
        (IrStateType::Collection(elem), IrLiteral::List(items)) => {
            for item in items {
                validate_list_literal_item(elem, item).map_err(|msg| {
                    IrLoadError::Validate(format!(
                        "collection state `{}` default {}",
                        state.name, msg
                    ))
                })?;
            }
            Ok(())
        }
        (IrStateType::Scalar(_), IrLiteral::List(_)) => Err(IrLoadError::Validate(format!(
            "scalar state `{}` cannot use a list literal default",
            state.name
        ))),
        (IrStateType::Collection(_), other) => Err(IrLoadError::Validate(format!(
            "collection state `{}` default must be a list literal, got {other:?}",
            state.name
        ))),
        (expected, other) => Err(IrLoadError::Validate(format!(
            "state `{}` default does not match declared type {expected:?}: {other:?}",
            state.name
        ))),
    }
}

fn validate_list_literal_item(elem: &IrType, item: &IrLiteral) -> Result<(), String> {
    match (elem, item) {
        (IrType::I32, IrLiteral::Int(_))
        | (IrType::Str, IrLiteral::Str(_))
        | (IrType::Bool, IrLiteral::Bool(_)) => Ok(()),
        (_, IrLiteral::List(_)) => Err("cannot contain a nested list literal".into()),
        (expected, other) => Err(format!(
            "element must match `{}`, got {other:?}",
            scalar_type_name(expected)
        )),
    }
}

fn scalar_type_name(ty: &IrType) -> &'static str {
    match ty {
        IrType::I32 => "i32",
        IrType::Str => "string",
        IrType::Bool => "bool",
    }
}

fn validate_phase8_togglebutton_node_invariants(
    node: &IrNode,
    declared: &std::collections::HashMap<&str, IrStateType>,
    loop_scope: Option<LoopReadScope<'_>>,
) -> Result<(), IrLoadError> {
    if node.widget_type == "ToggleButton" {
        for prop in &node.props {
            match prop.name.as_str() {
                "text" => validate_literal_type(&prop.value, IrType::Str, "ToggleButton.text")?,
                "style" => {
                    if !matches!(prop.value, IrLiteral::Ident(_)) {
                        return Err(IrLoadError::Validate(
                            "ToggleButton.style must be a keyword identifier".into(),
                        ));
                    }
                }
                "enabled" => {
                    validate_literal_type(&prop.value, IrType::Bool, "ToggleButton.enabled")?
                }
                "checked" => {
                    validate_literal_type(&prop.value, IrType::Bool, "ToggleButton.checked")?
                }
                other => {
                    return Err(IrLoadError::Validate(format!(
                        "unknown ToggleButton attribute `{other}`; valid attributes: text, style, enabled, checked"
                    )));
                }
            }
        }
        for binding in &node.bindings {
            if binding.prop_name == "style" {
                return Err(IrLoadError::Validate(
                    "ToggleButton.style is not bindable in M3-Phase 8".into(),
                ));
            }
            let Some((_, target_ty)) = resolve_prop_key("ToggleButton", &binding.prop_name) else {
                return Err(IrLoadError::Validate(format!(
                    "unknown ToggleButton binding `{}`; valid bindable attributes: text, enabled, checked",
                    binding.prop_name
                )));
            };
            validate_scalar_binding_expr_type(
                &binding.expr,
                &target_ty,
                declared,
                loop_scope,
                &format!("ToggleButton.{}", binding.prop_name),
            )?;
        }
    } else {
        if node.props.iter().any(|p| p.name == "checked") {
            return Err(IrLoadError::Validate(format!(
                "`checked` is only valid on ToggleButton, not `{}`",
                node.widget_type
            )));
        }
        if node.bindings.iter().any(|b| b.prop_name == "checked") {
            return Err(IrLoadError::Validate(format!(
                "`checked` binding is only valid on ToggleButton, not `{}`",
                node.widget_type
            )));
        }
    }

    for member in &node.children {
        validate_phase8_togglebutton_member_invariants(member, declared, loop_scope)?;
    }
    Ok(())
}

fn validate_phase8_togglebutton_member_invariants(
    member: &IrMember,
    declared: &std::collections::HashMap<&str, IrStateType>,
    loop_scope: Option<LoopReadScope<'_>>,
) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(slot) => {
            validate_phase8_togglebutton_node_invariants(&slot.node, declared, loop_scope)
        }
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase8_togglebutton_member_invariants(
                        body_member,
                        declared,
                        loop_scope,
                    )?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For {
            binder,
            index_binder,
            collection,
            body,
        }) => {
            let child_scope = LoopReadScope {
                binder,
                index_binder: index_binder.as_deref(),
                elem: match collection {
                    HandlerExpr::ListPropRead { elem, .. } => elem,
                    _ => &IrType::I32,
                },
            };
            for body_member in body {
                validate_phase8_togglebutton_member_invariants(
                    body_member,
                    declared,
                    Some(child_scope),
                )?;
            }
            Ok(())
        }
    }
}

fn validate_literal_type(
    value: &IrLiteral,
    expected: IrType,
    label: &str,
) -> Result<(), IrLoadError> {
    let ok = matches!(
        (&expected, value),
        (IrType::I32, IrLiteral::Int(_))
            | (IrType::Str, IrLiteral::Str(_))
            | (IrType::Bool, IrLiteral::Bool(_))
    );
    if ok {
        Ok(())
    } else {
        Err(IrLoadError::Validate(format!(
            "{label} must be a `{}` literal",
            scalar_type_name(&expected)
        )))
    }
}

fn validate_scalar_binding_expr_type(
    expr: &HandlerExpr,
    expected: &IrType,
    declared: &std::collections::HashMap<&str, IrStateType>,
    loop_scope: Option<LoopReadScope<'_>>,
    label: &str,
) -> Result<(), IrLoadError> {
    match (expected, expr) {
        (IrType::I32, HandlerExpr::IntLit(_))
        | (IrType::Str, HandlerExpr::StrLit(_))
        | (IrType::Bool, HandlerExpr::BoolLit(_)) => Ok(()),
        (IrType::I32, HandlerExpr::PropRead { path })
        | (IrType::Str, HandlerExpr::StrPropRead { path })
        | (IrType::Bool, HandlerExpr::BoolPropRead { path }) => {
            validate_scalar_binding_read_type(path, expected, declared, label)
        }
        (IrType::Str, HandlerExpr::Interpolation(_)) => {
            validate_expr_references(expr, declared, loop_scope, &|name| {
                format!("binding `{label}` references undeclared name `{name}`")
            })
        }
        (_, HandlerExpr::ItemRead { .. } | HandlerExpr::IndexRead { .. })
            if loop_scope.is_some() =>
        {
            validate_loop_local_binding_type(expr, expected, loop_scope.expect("checked above"))
        }
        _ => Err(IrLoadError::Validate(format!(
            "binding `{label}` must resolve to `{}`",
            scalar_type_name(expected)
        ))),
    }
}

fn validate_scalar_binding_read_type(
    path: &str,
    expected: &IrType,
    declared: &std::collections::HashMap<&str, IrStateType>,
    label: &str,
) -> Result<(), IrLoadError> {
    match declared.get(path) {
        Some(IrStateType::Scalar(found)) if found == expected => Ok(()),
        Some(IrStateType::Scalar(found)) => Err(IrLoadError::Validate(format!(
            "binding `{label}` reads `{path}` with type `{}`, expected `{}`",
            scalar_type_name(found),
            scalar_type_name(expected)
        ))),
        Some(IrStateType::Collection(_)) => Err(IrLoadError::Validate(format!(
            "binding `{label}` references collection state `{path}`"
        ))),
        None => Err(IrLoadError::Validate(format!(
            "binding `{label}` references undeclared name `{path}`"
        ))),
    }
}

fn annotate_collection_expr_types(comp: &mut IrComponent) -> Result<(), IrLoadError> {
    let declared: std::collections::HashMap<String, IrStateType> = comp
        .states
        .iter()
        .map(|state| (state.name.clone(), state.ty.clone()))
        .collect();
    annotate_node_collection_expr_types(&mut comp.root, &declared)
}

fn annotate_node_collection_expr_types(
    node: &mut IrNode,
    declared: &std::collections::HashMap<String, IrStateType>,
) -> Result<(), IrLoadError> {
    for binding in &mut node.bindings {
        annotate_expr_collection_types(&mut binding.expr, declared)?;
    }
    for handler in &mut node.handlers {
        annotate_expr_collection_types(&mut handler.expr, declared)?;
    }
    for member in &mut node.children {
        annotate_member_collection_expr_types(member, declared)?;
    }
    Ok(())
}

fn annotate_member_collection_expr_types(
    member: &mut IrMember,
    declared: &std::collections::HashMap<String, IrStateType>,
) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(slot) => annotate_node_collection_expr_types(&mut slot.node, declared),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                annotate_expr_collection_types(&mut branch.condition, declared)?;
                for body_member in &mut branch.body {
                    annotate_member_collection_expr_types(body_member, declared)?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For {
            collection, body, ..
        }) => {
            annotate_expr_collection_types(collection, declared)?;
            for body_member in body {
                annotate_member_collection_expr_types(body_member, declared)?;
            }
            Ok(())
        }
    }
}

fn annotate_expr_collection_types(
    expr: &mut HandlerExpr,
    declared: &std::collections::HashMap<String, IrStateType>,
) -> Result<(), IrLoadError> {
    match expr {
        HandlerExpr::ListPropRead { path, elem }
        | HandlerExpr::ListAppend { path, elem, .. }
        | HandlerExpr::ListDropLast { path, elem } => {
            if let Some(IrStateType::Collection(found)) = declared.get(path) {
                *elem = found.clone();
            }
        }
        _ => {}
    }
    match expr {
        HandlerExpr::Assign { rhs, .. } | HandlerExpr::CompoundAssign { rhs, .. } => {
            annotate_expr_collection_types(rhs, declared)?;
        }
        HandlerExpr::ListAppend { value, .. } => {
            annotate_expr_collection_types(value, declared)?;
        }
        HandlerExpr::Interpolation(parts) => {
            for part in parts {
                if let InterpolationPart::Expr(inner) = part {
                    annotate_expr_collection_types(inner, declared)?;
                }
            }
        }
        HandlerExpr::Block(exprs) => {
            for inner in exprs {
                annotate_expr_collection_types(inner, declared)?;
            }
        }
        HandlerExpr::IntLit(_)
        | HandlerExpr::StrLit(_)
        | HandlerExpr::BoolLit(_)
        | HandlerExpr::PropRead { .. }
        | HandlerExpr::StrPropRead { .. }
        | HandlerExpr::BoolPropRead { .. }
        | HandlerExpr::ListPropRead { .. }
        | HandlerExpr::ItemRead { .. }
        | HandlerExpr::IndexRead { .. }
        | HandlerExpr::ListDropLast { .. }
        | HandlerExpr::ListLit(_) => {}
    }
    Ok(())
}

const HOST_STATIC_ATTRS: &[&str] = &["title", "backdrop", "theme"];

fn validate_host_surface(comp: &IrComponent) -> Result<(), IrLoadError> {
    // The runtime is a *defensive reader* of textual IR (DD-M3-P6-008), so it
    // mirrors the compiler host catalog on both the attribute *name* and the
    // per-attribute *value shape* — not just the name. `wasamoc check` rejects
    // a non-string `title` and a typed-scalar literal on `backdrop` / `theme`
    // (those take a keyword identifier such as `mica` / `system`), so a
    // hand-crafted textual IR that skips the compiler must be rejected here
    // identically rather than leaving a direct-textual-IR hole.
    for prop in &comp.host_props {
        match prop.name.as_str() {
            "title" => {
                if !matches!(prop.value, IrLiteral::Str(_)) {
                    return Err(IrLoadError::Validate(
                        "host `title` prop must be a string literal".into(),
                    ));
                }
            }
            "backdrop" | "theme" => {
                if matches!(
                    prop.value,
                    IrLiteral::Int(_) | IrLiteral::Str(_) | IrLiteral::Bool(_)
                ) {
                    return Err(IrLoadError::Validate(format!(
                        "host `{}` prop must be a keyword identifier, not a typed literal",
                        prop.name
                    )));
                }
            }
            _ => {
                return Err(IrLoadError::Validate(format!(
                    "unknown host attribute `{}`; M3-Phase 6 host attributes are: {}",
                    prop.name,
                    HOST_STATIC_ATTRS.join(", ")
                )));
            }
        }
    }
    if let Some(binding) = comp.host_bindings.first() {
        return Err(IrLoadError::Validate(format!(
            "host attribute `{}` is not bindable in M3-Phase 6",
            binding.prop_name
        )));
    }
    Ok(())
}

fn reject_root_squatted_host_attrs(comp: &IrComponent) -> Result<(), IrLoadError> {
    if let Some(prop) = comp
        .root
        .props
        .iter()
        .find(|prop| HOST_STATIC_ATTRS.contains(&prop.name.as_str()))
    {
        return Err(IrLoadError::Validate(format!(
            "host attribute `{}` must live on `host_props`, not on the content root",
            prop.name
        )));
    }
    if let Some(binding) = comp
        .root
        .bindings
        .iter()
        .find(|binding| HOST_STATIC_ATTRS.contains(&binding.prop_name.as_str()))
    {
        return Err(IrLoadError::Validate(format!(
            "host attribute `{}` must live on `host_bindings`, not on the content root",
            binding.prop_name
        )));
    }
    Ok(())
}

pub(crate) fn resolve_static_window_title<'a>(
    comp: &'a IrComponent,
    default_title: &'a str,
) -> &'a str {
    let Some(prop) = comp.host_props.iter().find(|prop| prop.name == "title") else {
        return default_title;
    };
    match &prop.value {
        IrLiteral::Str(title) if !title.is_empty() => title.as_str(),
        _ => default_title,
    }
}

fn is_child_placement_prop(name: &str) -> bool {
    matches!(name, "h-align" | "v-align")
}

fn validate_phase6_control_flow_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    for member in &node.children {
        match member {
            IrMember::Widget(slot) => validate_phase6_control_flow_invariants(&slot.node)?,
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
                    IrMember::Widget(slot) => validate_phase6_control_flow_invariants(&slot.node)?,
                    IrMember::ControlFlow(_) => {
                        return Err(IrLoadError::Validate(
                            "a nested control-flow member is not valid directly in an `if` body in M3-Phase 6".into(),
                        ));
                    }
                }
            }
            IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
                if body.len() != 1 {
                    return Err(IrLoadError::Validate(format!(
                        "`for` body supports exactly one widget member in M3-Phase 7, got {}",
                        body.len()
                    )));
                }
                match &body[0] {
                    IrMember::Widget(slot) => validate_phase6_control_flow_invariants(&slot.node)?,
                    IrMember::ControlFlow(_) => {
                        return Err(IrLoadError::Validate(
                            "a nested control-flow member is not valid directly in a `for` body in M3-Phase 7".into(),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_phase7_iteration_invariants(
    node: &IrNode,
    inside_for_template: bool,
) -> Result<(), IrLoadError> {
    if inside_for_template && !node.handlers.is_empty() {
        return Err(IrLoadError::Validate(
            "handlers inside a `for` body template are deferred in M3-Phase 7".into(),
        ));
    }

    let current = parent_kind_for(node);
    for member in &node.children {
        match member {
            IrMember::Widget(slot) => {
                validate_phase7_iteration_invariants(&slot.node, inside_for_template)?;
            }
            IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
                for branch in branches {
                    for body_member in &branch.body {
                        validate_phase7_iteration_member_invariants(
                            body_member,
                            current,
                            inside_for_template,
                        )?;
                    }
                }
            }
            IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
                validate_direct_for_parent(node)?;
                if inside_for_template {
                    return Err(IrLoadError::Validate(
                        "nested `for` is deferred in M3-Phase 7".into(),
                    ));
                }
                if body.len() != 1 || !matches!(body.first(), Some(IrMember::Widget(_))) {
                    return Err(IrLoadError::Validate(
                        "`for` body admits exactly one widget child in M3-Phase 7".into(),
                    ));
                }
                if let Some(IrMember::Widget(body_slot)) = body.first() {
                    validate_phase7_iteration_invariants(&body_slot.node, true)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_phase7_iteration_member_invariants(
    member: &IrMember,
    parent: ParentKind,
    inside_for_template: bool,
) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(slot) => {
            validate_phase7_iteration_invariants(&slot.node, inside_for_template)
        }
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase7_iteration_member_invariants(
                        body_member,
                        parent,
                        inside_for_template,
                    )?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
            if inside_for_template {
                return Err(IrLoadError::Validate(
                    "nested `for` is deferred in M3-Phase 7".into(),
                ));
            }
            match parent {
                ParentKind::Grid | ParentKind::Cell => Err(IrLoadError::Validate(
                    "direct `for` is not valid in Grid placement contexts in M3-Phase 7".into(),
                )),
                _ => {
                    if body.len() != 1 || !matches!(body.first(), Some(IrMember::Widget(_))) {
                        return Err(IrLoadError::Validate(
                            "`for` body admits exactly one widget child in M3-Phase 7".into(),
                        ));
                    }
                    if let Some(IrMember::Widget(body_slot)) = body.first() {
                        validate_phase7_iteration_invariants(&body_slot.node, true)?;
                    }
                    Ok(())
                }
            }
        }
    }
}

fn validate_direct_for_parent(parent_node: &IrNode) -> Result<(), IrLoadError> {
    match parent_node.widget_type.as_str() {
        "ScrollView" => Err(IrLoadError::Validate(
            "direct `for` is not valid in ScrollView; wrap it in a content widget such as `WrapPanel`".into(),
        )),
        "Box" => Err(IrLoadError::Validate(
            "direct `for` is not valid in Box because Box admits at most one child".into(),
        )),
        "Grid" | "Cell" => Err(IrLoadError::Validate(
            "direct `for` is not valid in Grid placement contexts in M3-Phase 7".into(),
        )),
        _ => Ok(()),
    }
}

fn validate_phase2_member_invariants(member: &IrMember) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(slot) => validate_phase2_node_invariants(&slot.node),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase2_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
            for body_member in body {
                validate_phase2_member_invariants(body_member)?;
            }
            Ok(())
        }
    }
}

fn validate_phase2_node_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    // Box single-child invariant (DD-M3-P2-001). wasamoc check (T3)
    // diagnoses the same condition at compile time; this is the runtime
    // defense for IR not produced by wasamoc.
    //
    // T4 review follow-up: count every member that can materialise a
    // child, not widget children only. A conditional member materialises
    // at most one child, so a conditional sibling counts toward the limit
    // (`Box { Text  if c { … } }` could become two children). The prior
    // `widget_children()` count under-counted the conditional sibling and
    // let it slip past both gates (see log.md T4 migration audit).
    let child_member_count = node.children.len();
    if node.widget_type == "Box" && child_member_count > 1 {
        return Err(IrLoadError::Validate(format!(
            "`Box` node accepts at most one child, got {} (use `VStack` / `HStack` / `ZStack` for multi-child layouts)",
            child_member_count
        )));
    }
    // M4-Phase 2 T8 (CF-1, owner disposition 2026-08-07; widened from
    // Button/ToggleButton to all four `wasamo_ir::LAYOUT_CHILDLESS_WIDGET_KINDS`
    // 2026-08-08): a layout-childless node (`Rectangle` / `Text` /
    // `Button` / `ToggleButton`) carrying any child member is rejected
    // here too. `wasamoc check`'s `check_layout_childless_widget_children`
    // rejects the same shape at compile time (defense in depth); this is
    // the runtime gate for memory IR that reaches `wasamo_load_ui` without
    // traversing `wasamoc`. Unlike Box's "at most one", every child member
    // (widget, conditional, or `for`) is unknown to `build_layout_tree`
    // (widget.rs), which maps every kind in the table to a childless
    // `LayoutNode::rectangle` — so the admitted count here is zero, not
    // one. Neither this condition nor the message below names a widget
    // kind: both read `wasamo_ir::layout_treats_as_childless` / the
    // offending `node.widget_type`, so widening or narrowing the rule is a
    // single edit to `wasamo_ir::LAYOUT_CHILDLESS_WIDGET_KINDS`.
    if wasamo_ir::layout_treats_as_childless(&node.widget_type) && child_member_count > 0 {
        return Err(IrLoadError::Validate(format!(
            "`{}` node accepts no children, got {} (layout arranges it as a single rectangle, so a child would never be arranged, painted, or hit-tested — wrap it in a container widget instead; dsl_spec §4.4)",
            node.widget_type, child_member_count
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
    if node.widget_type == "ScrollView" {
        // DD-M3-P6-007 accepted (a): a conditional is not a valid *direct*
        // ScrollView content member. Its presence is dynamic, so it cannot
        // satisfy "exactly one content child" (`ScrollView { Content if c }`
        // could become two; `ScrollView { if c { ... } }` could become zero).
        // Wrap it in the content widget (`ScrollView { Box { if c { ... } } }`).
        // The conditionally-empty direction is deferred, not rejected.
        if node
            .children
            .iter()
            .any(|m| matches!(m, IrMember::ControlFlow(_)))
        {
            return Err(IrLoadError::Validate(
                "`ScrollView` content child must be a single widget; a conditional member is not valid directly in ScrollView (wrap it in the content widget)".into(),
            ));
        }
        let widget_child_count = node.widget_children().count();
        if widget_child_count != 1 {
            return Err(IrLoadError::Validate(format!(
                "`ScrollView` requires exactly one content child, got {}",
                widget_child_count
            )));
        }
    }
    for member in &node.children {
        validate_phase4_member_invariants(member)?;
    }
    Ok(())
}

fn validate_phase4_member_invariants(member: &IrMember) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(slot) => validate_phase4_node_invariants(&slot.node),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase4_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
            for body_member in body {
                validate_phase4_member_invariants(body_member)?;
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
        IrMember::Widget(slot) => validate_phase3_node_invariants(&slot.node),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase3_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
            for body_member in body {
                validate_phase3_member_invariants(body_member)?;
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
            for slot in node.widget_child_slots() {
                validate_phase5_node_invariants(&slot.node)?;
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
        IrMember::Widget(slot) => validate_phase5_node_invariants(&slot.node),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase5_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
            for body_member in body {
                validate_phase5_member_invariants(body_member)?;
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
        // Let stale child-placement props on a nested ZStack fall through to
        // the global legacy-placement diagnostic below instead of reporting a
        // generic ZStack attribute error.
        //
        // M4-Phase 2 T6 relaxation (dsl_spec §4.19, DD-M4-P2-005 A1):
        // `wasamoc check` now accepts `focus-group` / `modal-scope` on a
        // ZStack — it is one of the seven `FOCUS_ANNOTATION_CONTAINERS`
        // — and a `dismiss` handler beside `modal-scope: true`. Without
        // this widening the loader would refuse to load a `.ui` the
        // compiler already accepted. `validate_focus_annotation_invariants`
        // has already run (see `validate`'s ordering) and rejected a
        // malformed shape of either attribute or of `dismiss`, so by the
        // time this gate sees them they are known-good. Every other
        // Phase-6 attribute stays rejected. The bindings rule is
        // untouched: both focus annotations are constant-only and never
        // travel the binding path.
        //
        // M4-Phase 2 T8: signal-handler admission on a ZStack is not
        // gated here at all — it is the generic name-keyed rule in
        // `validate_focus_annotation_invariants` (`dismiss` needs a
        // `modal-scope: true` sibling on the same node; every other
        // signal name, e.g. `clicked`, is admitted unconditionally), the
        // same rule every other widget kind is subject to. This gate has
        // no per-kind handler rule of its own.
        let zstack_widget_prop = node.props.iter().find(|prop| {
            !(parent == ParentKind::ZStack && is_child_placement_prop(&prop.name))
                && !matches!(prop.name.as_str(), "focus-group" | "modal-scope")
        });
        if let Some(prop) = zstack_widget_prop {
            return Err(IrLoadError::Validate(format!(
                "`ZStack` accepts no Phase-6 attributes; found `{}`",
                prop.name
            )));
        }
        if !node.bindings.is_empty() {
            return Err(IrLoadError::Validate(
                "`ZStack` accepts no Phase-6 bindings".into(),
            ));
        }
    }

    for prop in &node.props {
        if matches!(prop.name.as_str(), "h-align" | "v-align") {
            return Err(legacy_placement_ir_form(
                "bare `h-align` / `v-align` props are stale textual IR; regenerate to child-slot placement",
            ));
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
        IrMember::Widget(slot) => {
            validate_slot_data_parent(parent, slot)?;
            validate_phase6_zstack_node_invariants(&slot.node, parent)
        }
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_phase6_zstack_member_invariants(body_member, parent)?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
            for body_member in body {
                validate_phase6_zstack_member_invariants(body_member, parent)?;
            }
            Ok(())
        }
    }
}

fn validate_slot_data_parent(parent: ParentKind, slot: &IrChildSlot) -> Result<(), IrLoadError> {
    match (parent, &slot.slot_data) {
        (ParentKind::Grid, Some(IrSlotData::Grid { .. }) | None) => Ok(()),
        (ParentKind::Grid, Some(IrSlotData::ZStack { .. })) => Err(IrLoadError::Validate(
            "invalid-placement-ir: `placement zstack` is not valid on a Grid child".into(),
        )),
        (ParentKind::ZStack, Some(IrSlotData::ZStack { .. }) | None) => Ok(()),
        (ParentKind::ZStack, Some(IrSlotData::Grid { .. })) => Err(IrLoadError::Validate(
            "invalid-placement-ir: `placement grid` is not valid on a ZStack child".into(),
        )),
        (_, Some(_)) => Err(IrLoadError::Validate(
            "invalid-placement-ir: placement data is valid only on Grid or ZStack child slots"
                .into(),
        )),
        (_, None) => Ok(()),
    }
}

// ── M4-Phase 2 T6: focus-group / modal-scope / dismiss ────────────────

/// Container widget kinds that admit the M4-Phase 2 focus annotations
/// (`focus-group` / `modal-scope`), per `docs/dsl_spec.md` §4.19
/// "admitted on any container" (DD-M4-P2-005 A1). Mirrors
/// `wasamoc::check::FOCUS_ANNOTATION_CONTAINERS` one-for-one — kept as a
/// separate const because the two live in different crates, not because
/// the list differs. `Text`, `Button`, `ToggleButton`, and `Rectangle`
/// are leaf/content widgets, not containers, and are excluded; `Cell` is
/// an IR-only Grid wrapper, not a runtime container, and is excluded
/// too. Both `focus-group` and `modal-scope` read from this one const in
/// `validate_focus_annotation_invariants`, so the seven-name list has a
/// single source of truth rather than appearing once per attribute.
const FOCUS_ANNOTATION_CONTAINERS: &[&str] = &[
    "VStack",
    "HStack",
    "Box",
    "WrapPanel",
    "ScrollView",
    "Grid",
    "ZStack",
];

/// M4-Phase 2 T6 defense-in-depth gate (dsl_spec §4.19, DD-M4-P2-005
/// A1) — the runtime half of the `wasamoc check` gate for `focus-group`
/// / `modal-scope` / `dismiss`. `wasamoc check` (T6 stage 1) rejects the
/// same four shapes at compile time
/// (`check_focus_annotation_admission`,
/// `check_focus_annotation_const_only_bind`, the constant-only binding
/// rule, and the `dismiss`/`carries_modal_scope` predicate); this gate
/// exists for memory IR that reaches the runtime loader through
/// `wasamo_load_ui` without traversing `wasamoc`, the same reason every
/// earlier `validate_phaseN_*` gate in this file exists.
fn validate_focus_annotation_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    for prop in &node.props {
        if !matches!(prop.name.as_str(), "focus-group" | "modal-scope") {
            continue;
        }
        if !FOCUS_ANNOTATION_CONTAINERS.contains(&node.widget_type.as_str()) {
            return Err(IrLoadError::Validate(format!(
                "`{}` is admitted on any container (dsl_spec §4.19) and is not valid on widget `{}`",
                prop.name, node.widget_type
            )));
        }
        // Constant-only (dsl_spec §4.19, the `Box.fill` / `WrapPanel`
        // precedent): the runtime half of `check_focus_annotation_const_only_bind`.
        // A non-`Bool` literal reaching here means the IR was not
        // produced by `wasamoc`.
        if !matches!(prop.value, IrLiteral::Bool(_)) {
            return Err(IrLoadError::Validate(format!(
                "`{}` is constant-only (dsl_spec §4.19); expected a `true` or `false` literal",
                prop.name
            )));
        }
    }
    // Constant-only also means "never travels the binding path" — a
    // `bind focus-group = …` / `bind modal-scope = …` is rejected
    // outright, independent of widget kind (mirrors `wasamoc check`'s
    // rejection of a bare state-ident RHS, but a runtime `IrBinding` of
    // either name is rejected unconditionally rather than re-parsing the
    // expression shape).
    if let Some(binding) = node
        .bindings
        .iter()
        .find(|b| matches!(b.prop_name.as_str(), "focus-group" | "modal-scope"))
    {
        return Err(IrLoadError::Validate(format!(
            "`{}` is constant-only (dsl_spec §4.19) and cannot be bound",
            binding.prop_name
        )));
    }
    // `dismiss` is admitted only on a node that itself carries
    // `prop modal-scope = true`. An absent prop, a `false` value, and a
    // non-container widget (which cannot carry a `true` value past the
    // admission check above) all collapse to the same "no true
    // `modal-scope` sibling" test — there is no separate container check
    // here because the admission rule above has already ruled out a
    // non-container ever reaching this point with `modal-scope: true`.
    if node.handlers.iter().any(|h| h.signal == "dismiss") {
        let carries_modal_scope_true = node
            .props
            .iter()
            .any(|p| p.name == "modal-scope" && matches!(p.value, IrLiteral::Bool(true)));
        if !carries_modal_scope_true {
            return Err(IrLoadError::Validate(
                "`dismiss` handler can never be raised: a dismissal request is addressed to a modal scope; write `modal-scope: true` on the same container or remove the handler (dsl_spec §4.19)".into(),
            ));
        }
    }
    for member in &node.children {
        validate_focus_annotation_member_invariants(member)?;
    }
    Ok(())
}

fn validate_focus_annotation_member_invariants(member: &IrMember) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(slot) => validate_focus_annotation_invariants(&slot.node),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_focus_annotation_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
            for body_member in body {
                validate_focus_annotation_member_invariants(body_member)?;
            }
            Ok(())
        }
    }
}

// ── M4-Phase 2 T8: `key-down("<key>")` handler argument ────────────────

/// M4-Phase 2 T8 defense-in-depth gate (dsl_spec §4.19 "Keyboard input",
/// DD-M4-P2-005) — the runtime half of the `wasamoc check` gate for
/// `key-down`'s parenthesised argument. `wasamoc check` rejects the same
/// three shapes at compile time (in the `Member::SignalHandler` arm of
/// `check_members_inner`); this gate exists for memory IR that reaches
/// the runtime loader through `wasamo_load_ui` without traversing
/// `wasamoc`, the same reason every earlier `validate_phaseN_*` gate in
/// this file exists.
///
/// Kept as its own pass rather than folded into
/// `validate_focus_annotation_invariants`: the two features are
/// unrelated beyond sharing dsl_spec §4.19. `key-down` is admitted on
/// **every** widget kind (no container gate, unlike `focus-group` /
/// `modal-scope`), so this pass needs neither `FOCUS_ANNOTATION_CONTAINERS`
/// nor an `enclosing_widget`-shaped parameter — folding it in would have
/// made that function's own doc comment ("the gate for focus-group /
/// modal-scope / dismiss") inaccurate for no shared logic.
fn validate_key_down_invariants(node: &IrNode) -> Result<(), IrLoadError> {
    for handler in &node.handlers {
        if handler.signal == "key-down" {
            match &handler.arg {
                // A bare `key-down` (no argument) can never fire — the
                // same "silently never fires" class `dismiss` guards
                // against above.
                None => {
                    return Err(IrLoadError::Validate(
                        "`key-down` handler can never be raised: the key must be named in the declaration, e.g. `key-down(\"ArrowLeft\")` (dsl_spec §4.19)".into(),
                    ));
                }
                Some(key) if !wasamo_ir::is_recognised_key_name(key) => {
                    return Err(IrLoadError::Validate(format!(
                        "`key-down(\"{key}\")` names an unrecognised key; recognised keys are the named non-character keys per dsl_spec §4.19 (`Escape`, the arrow / Home / End / Page keys, `Enter`, `F1`-`F12`)"
                    )));
                }
                Some(_) => {}
            }
        } else if handler.arg.is_some() {
            // `key-down` is the only signal dsl_spec §4.19 defines with
            // an argument.
            return Err(IrLoadError::Validate(format!(
                "`{}` does not take an argument; only `key-down` does (dsl_spec §4.19)",
                handler.signal
            )));
        }
    }
    for member in &node.children {
        validate_key_down_member_invariants(member)?;
    }
    Ok(())
}

fn validate_key_down_member_invariants(member: &IrMember) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(slot) => validate_key_down_invariants(&slot.node),
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                for body_member in &branch.body {
                    validate_key_down_member_invariants(body_member)?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
            for body_member in body {
                validate_key_down_member_invariants(body_member)?;
            }
            Ok(())
        }
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

    // Per-child-slot validation + rectangle collection. M3-Phase 7b T2
    // normalises the old `Cell` textual form to `IrSlotData::Grid`; a
    // surviving `Cell` node is stale IR and must be regenerated.
    let mut rects: Vec<GridCellRect> = Vec::new();
    for member in &node.children {
        match member {
            IrMember::Widget(slot) if slot.node.widget_type == "Cell" => {
                return Err(legacy_placement_ir_form(
                    "Grid `Cell` child wrapper is stale textual IR; regenerate to `child { placement grid ... node ... }`",
                ));
            }
            IrMember::Widget(slot) => {
                rects.push(validate_grid_child_slot(slot, columns_len, rows_len)?)
            }
            IrMember::ControlFlow(_) => {
                return Err(IrLoadError::Validate(
                    "`Grid` children must use child slots; conditional members are not valid directly in runtime Grid IR".into(),
                ));
            }
        }
    }

    // Same-cell / overlapping-rectangle conflict (DD-M3-P5-003): no two
    // Grid child placements share any resolved cell. `O(n_cells^2)` pairwise (trivial for
    // practical Grid sizes per DD-M3-P5-006).
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            if grid_rects_overlap(&rects[i], &rects[j]) {
                return Err(IrLoadError::Validate(format!(
                    "`Grid` child placement at (row {}, column {}) overlaps an earlier Grid child rectangle; same-cell and overlapping placements are rejected (DD-M3-P5-003)",
                    rects[j].row, rects[j].column
                )));
            }
        }
    }

    Ok(())
}

fn legacy_placement_ir_form(detail: &str) -> IrLoadError {
    IrLoadError::Validate(format!("legacy-placement-ir-form: {detail}"))
}

// Validate one Grid child slot and return its resolved rectangle. Missing
// slot data preserves the existing Grid defaults; wrong-kind slot data is
// malformed canonical IR.
fn validate_grid_child_slot(
    slot: &IrChildSlot,
    columns_len: i64,
    rows_len: i64,
) -> Result<GridCellRect, IrLoadError> {
    let (row, column, row_span, column_span) = match &slot.slot_data {
        Some(IrSlotData::Grid {
            row,
            column,
            row_span,
            column_span,
            ..
        }) => (
            *row as i64,
            *column as i64,
            *row_span as i64,
            *column_span as i64,
        ),
        Some(IrSlotData::ZStack { .. }) => {
            return Err(IrLoadError::Validate(
                "invalid-placement-ir: `placement zstack` is not valid on a Grid child".into(),
            ));
        }
        None => (0, 0, 1, 1),
    };

    // Placement value range (DD-M3-P5-003): row in `[0, rows.len())`,
    // column in `[0, columns.len())`.
    if row < 0 || row >= rows_len {
        return Err(IrLoadError::Validate(format!(
            "`Grid` child placement `row` {row} is out of range [0, {rows_len}) (DD-M3-P5-003)"
        )));
    }
    if column < 0 || column >= columns_len {
        return Err(IrLoadError::Validate(format!(
            "`Grid` child placement `column` {column} is out of range [0, {columns_len}) (DD-M3-P5-003)"
        )));
    }

    // Span value range (DD-M3-P5-003): spans `>= 1` and the resolved
    // rectangle fits within the declared track count.
    if row_span < 1 {
        return Err(IrLoadError::Validate(format!(
            "`Grid` child placement `row-span` must be a positive integer (>= 1), got {row_span} (DD-M3-P5-003)"
        )));
    }
    if column_span < 1 {
        return Err(IrLoadError::Validate(format!(
            "`Grid` child placement `column-span` must be a positive integer (>= 1), got {column_span} (DD-M3-P5-003)"
        )));
    }
    if row + row_span > rows_len {
        return Err(IrLoadError::Validate(format!(
            "`Grid` child row span exceeds the grid: row {row} + row-span {row_span} = {} > {rows_len} declared row tracks (DD-M3-P5-003)",
            row + row_span
        )));
    }
    if column + column_span > columns_len {
        return Err(IrLoadError::Validate(format!(
            "`Grid` child column span exceeds the grid: column {column} + column-span {column_span} = {} > {columns_len} declared column tracks (DD-M3-P5-003)",
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

/// Half-open rectangle overlap in track coordinates (DD-M3-P5-003).
fn grid_rects_overlap(a: &GridCellRect, b: &GridCellRect) -> bool {
    fn ranges_overlap(s1: i64, len1: i64, s2: i64, len2: i64) -> bool {
        s1 < s2 + len2 && s2 < s1 + len1
    }
    ranges_overlap(a.row, a.row_span, b.row, b.row_span)
        && ranges_overlap(a.column, a.column_span, b.column, b.column_span)
}

#[derive(Clone, Copy)]
struct LoopReadScope<'a> {
    binder: &'a str,
    index_binder: Option<&'a str>,
    elem: &'a IrType,
}

fn validate_node_references(
    node: &IrNode,
    declared: &std::collections::HashMap<&str, IrStateType>,
) -> Result<(), IrLoadError> {
    validate_node_references_in_scope(node, declared, None, false)
}

fn validate_node_references_in_scope(
    node: &IrNode,
    declared: &std::collections::HashMap<&str, IrStateType>,
    loop_scope: Option<LoopReadScope<'_>>,
    inside_for_template: bool,
) -> Result<(), IrLoadError> {
    for binding in &node.bindings {
        if let Some(scope) = loop_scope {
            if let Some((_, target_ty)) = resolve_prop_key(&node.widget_type, &binding.prop_name) {
                validate_loop_local_binding_type(&binding.expr, &target_ty, scope)?;
            }
        }
        validate_expr_references(&binding.expr, declared, loop_scope, &|name| {
            format!(
                "binding `{}` references undeclared name `{}`",
                binding.prop_name, name
            )
        })?;
    }
    for handler in &node.handlers {
        if inside_for_template {
            return Err(IrLoadError::Validate(
                "handlers inside a `for` body template are deferred in M3-Phase 7".into(),
            ));
        }
        validate_expr_references(&handler.expr, declared, None, &|name| {
            format!(
                "handler `on {}` references undeclared name `{}`",
                handler.signal, name
            )
        })?;
    }
    for member in &node.children {
        validate_member_references(member, declared, loop_scope, inside_for_template)?;
    }
    Ok(())
}

fn validate_member_references(
    member: &IrMember,
    declared: &std::collections::HashMap<&str, IrStateType>,
    loop_scope: Option<LoopReadScope<'_>>,
    inside_for_template: bool,
) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(slot) => {
            validate_node_references_in_scope(&slot.node, declared, loop_scope, inside_for_template)
        }
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            for branch in branches {
                validate_condition_expr(&branch.condition, declared)?;
                for body_member in &branch.body {
                    validate_member_references(
                        body_member,
                        declared,
                        loop_scope,
                        inside_for_template,
                    )?;
                }
            }
            Ok(())
        }
        IrMember::ControlFlow(ControlFlowNode::For {
            binder,
            index_binder,
            collection,
            body,
        }) => {
            validate_for_header(binder, index_binder.as_deref(), collection, declared)?;
            if inside_for_template {
                return Err(IrLoadError::Validate(
                    "nested `for` is deferred in M3-Phase 7".into(),
                ));
            }
            let child_scope = LoopReadScope {
                binder,
                index_binder: index_binder.as_deref(),
                elem: match collection {
                    HandlerExpr::ListPropRead { elem, .. } => elem,
                    _ => &IrType::I32,
                },
            };
            for body_member in body {
                validate_member_references(body_member, declared, Some(child_scope), true)?;
            }
            Ok(())
        }
    }
}

fn validate_loop_local_binding_type(
    expr: &HandlerExpr,
    target_ty: &IrType,
    loop_scope: LoopReadScope<'_>,
) -> Result<(), IrLoadError> {
    match expr {
        HandlerExpr::ItemRead { binder } if binder == loop_scope.binder => {
            match (target_ty, loop_scope.elem) {
                (IrType::Str, IrType::Bool) => Err(IrLoadError::Validate(
                    "bool loop binder cannot be used in string binding; bool formatting/display conversion is not defined in M3-Phase 7".into(),
                )),
                (IrType::I32, IrType::I32) | (IrType::Str, IrType::I32 | IrType::Str) | (IrType::Bool, IrType::Bool) => Ok(()),
                (expected, found) => Err(IrLoadError::Validate(format!(
                    "loop binder `{binder}` has element type `{}`, not `{}`",
                    scalar_type_name(found),
                    scalar_type_name(expected)
                ))),
            }
        }
        HandlerExpr::IndexRead { binder } if loop_scope.index_binder == Some(binder.as_str()) => {
            match target_ty {
                IrType::I32 | IrType::Str => Ok(()),
                IrType::Bool => Err(IrLoadError::Validate(
                    "loop index binder cannot be used in a bool binding".into(),
                )),
            }
        }
        HandlerExpr::Interpolation(parts) => {
            for part in parts {
                if let InterpolationPart::Expr(inner) = part {
                    validate_loop_local_binding_type(inner, &IrType::Str, loop_scope)?;
                }
            }
            Ok(())
        }
        HandlerExpr::Block(exprs) => {
            for inner in exprs {
                validate_loop_local_binding_type(inner, target_ty, loop_scope)?;
            }
            Ok(())
        }
        HandlerExpr::Assign { rhs, .. } | HandlerExpr::CompoundAssign { rhs, .. } => {
            validate_loop_local_binding_type(rhs, target_ty, loop_scope)
        }
        HandlerExpr::ListAppend { value, .. } => {
            validate_loop_local_binding_type(value, target_ty, loop_scope)
        }
        _ => Ok(()),
    }
}

fn validate_expr_references(
    expr: &HandlerExpr,
    declared: &std::collections::HashMap<&str, IrStateType>,
    loop_scope: Option<LoopReadScope<'_>>,
    err_msg: &dyn Fn(&str) -> String,
) -> Result<(), IrLoadError> {
    match expr {
        HandlerExpr::IntLit(_) | HandlerExpr::StrLit(_) | HandlerExpr::BoolLit(_) => Ok(()),
        HandlerExpr::ItemRead { binder } => match loop_scope {
            Some(scope) if binder == scope.binder => Ok(()),
            Some(_) => Err(IrLoadError::Validate(format!(
                "`item-read {binder}` is not in scope for the current `for` body"
            ))),
            None => Err(IrLoadError::Validate(format!(
                "loop-local binder `{binder}` may be read only inside its `for` body"
            ))),
        },
        HandlerExpr::IndexRead { binder } => match loop_scope.and_then(|scope| scope.index_binder) {
            Some(index) if binder == index => Ok(()),
            Some(_) => Err(IrLoadError::Validate(format!(
                "`index-read {binder}` is not in scope for the current `for` body"
            ))),
            None => Err(IrLoadError::Validate(format!(
                "loop-local index binder `{binder}` may be read only inside its `for` body"
            ))),
        },
        HandlerExpr::PropRead { path }
        | HandlerExpr::StrPropRead { path }
        | HandlerExpr::BoolPropRead { path } => validate_scalar_read_path(path, declared, err_msg),
        HandlerExpr::ListPropRead { .. } => Err(IrLoadError::Validate(
            "collection reads are valid only as a `for` collection header in M3-Phase 7".into(),
        )),
        HandlerExpr::Assign { lhs, rhs } => {
            match declared.get(lhs.as_str()) {
                Some(IrStateType::Scalar(_)) => {
                    validate_expr_references(rhs, declared, loop_scope, err_msg)
                }
                Some(IrStateType::Collection(_)) => {
                    validate_collection_assignment_rhs(lhs, rhs, declared, err_msg)
                }
                None => Err(IrLoadError::Validate(err_msg(lhs))),
            }
        }
        HandlerExpr::CompoundAssign { lhs, rhs, .. } => {
            match declared.get(lhs.as_str()) {
                Some(IrStateType::Scalar(_)) => {
                    validate_expr_references(rhs, declared, loop_scope, err_msg)
                }
                Some(IrStateType::Collection(_)) => Err(IrLoadError::Validate(format!(
                    "collection state `{lhs}` cannot use compound assignment"
                ))),
                None => Err(IrLoadError::Validate(err_msg(lhs))),
            }
        }
        HandlerExpr::ListAppend { .. } | HandlerExpr::ListDropLast { .. } => {
            Err(IrLoadError::Validate(
                "collection edit expressions are valid only as a collection assignment RHS in M3-Phase 7".into(),
            ))
        }
        HandlerExpr::ListLit(_) => Err(IrLoadError::Validate(
            "list literals are valid only as collection state defaults or collection assignment RHS in M3-Phase 7".into(),
        )),
        HandlerExpr::Interpolation(parts) => {
            for part in parts {
                if let InterpolationPart::Expr(inner) = part {
                    validate_expr_references(inner, declared, loop_scope, err_msg)?;
                }
            }
            Ok(())
        }
        HandlerExpr::Block(exprs) => {
            for inner in exprs {
                validate_expr_references(inner, declared, loop_scope, err_msg)?;
            }
            Ok(())
        }
    }
}

fn validate_condition_expr(
    expr: &HandlerExpr,
    declared: &std::collections::HashMap<&str, IrStateType>,
) -> Result<(), IrLoadError> {
    match expr {
        HandlerExpr::BoolLit(_) => Ok(()),
        HandlerExpr::BoolPropRead { path } => match declared.get(path.as_str()) {
            Some(IrStateType::Scalar(IrType::Bool)) => Ok(()),
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

fn validate_scalar_read_path(
    path: &str,
    declared: &std::collections::HashMap<&str, IrStateType>,
    err_msg: &dyn Fn(&str) -> String,
) -> Result<(), IrLoadError> {
    match declared.get(path) {
        Some(IrStateType::Scalar(_)) => Ok(()),
        Some(IrStateType::Collection(_)) => Err(IrLoadError::Validate(format!(
            "scalar expression references collection state `{path}`"
        ))),
        None => Err(IrLoadError::Validate(err_msg(path))),
    }
}

fn validate_collection_read_path(
    path: &str,
    elem: &IrType,
    declared: &std::collections::HashMap<&str, IrStateType>,
    err_msg: &dyn Fn(&str) -> String,
) -> Result<(), IrLoadError> {
    match declared.get(path) {
        Some(IrStateType::Collection(found)) if found == elem => Ok(()),
        Some(IrStateType::Collection(found)) => Err(IrLoadError::Validate(format!(
            "collection read `{path}` has element tag `{}`, but state is `{}`",
            scalar_type_name(elem),
            scalar_type_name(found)
        ))),
        Some(IrStateType::Scalar(_)) => Err(IrLoadError::Validate(format!(
            "collection expression references scalar state `{path}`"
        ))),
        None => Err(IrLoadError::Validate(err_msg(path))),
    }
}

fn validate_collection_assignment_rhs(
    lhs: &str,
    rhs: &HandlerExpr,
    declared: &std::collections::HashMap<&str, IrStateType>,
    err_msg: &dyn Fn(&str) -> String,
) -> Result<(), IrLoadError> {
    let elem = match declared.get(lhs) {
        Some(IrStateType::Collection(elem)) => elem,
        _ => return Err(IrLoadError::Validate(err_msg(lhs))),
    };
    match rhs {
        HandlerExpr::ListAppend {
            path,
            elem: rhs_elem,
            value,
        } if path == lhs && rhs_elem == elem => {
            validate_collection_element_expr(lhs, elem, value, declared, err_msg)?;
            Ok(())
        }
        HandlerExpr::ListDropLast {
            path,
            elem: rhs_elem,
        } if path == lhs && rhs_elem == elem => Ok(()),
        HandlerExpr::ListLit(items) => {
            for item in items {
                validate_list_literal_item(elem, item).map_err(IrLoadError::Validate)?;
            }
            Ok(())
        }
        HandlerExpr::ListAppend { path, .. } | HandlerExpr::ListDropLast { path, .. } => {
            Err(IrLoadError::Validate(format!(
                "collection assignment `{lhs}` RHS must use `{lhs}` as its receiver, got `{path}`"
            )))
        }
        other => Err(IrLoadError::Validate(format!(
            "collection assignment `{lhs}` requires list-append, list-drop-last, or list literal RHS, got {other:?}"
        ))),
    }
}

fn validate_collection_element_expr(
    lhs: &str,
    elem: &IrType,
    value: &HandlerExpr,
    declared: &std::collections::HashMap<&str, IrStateType>,
    err_msg: &dyn Fn(&str) -> String,
) -> Result<(), IrLoadError> {
    match (elem, value) {
        (IrType::I32, HandlerExpr::IntLit(_))
        | (IrType::Str, HandlerExpr::StrLit(_))
        | (IrType::Bool, HandlerExpr::BoolLit(_)) => Ok(()),
        (IrType::I32, HandlerExpr::PropRead { path }) => {
            validate_scalar_value_read(lhs, elem, path, declared, err_msg)
        }
        (IrType::Str, HandlerExpr::StrPropRead { path }) => {
            validate_scalar_value_read(lhs, elem, path, declared, err_msg)
        }
        (IrType::Bool, HandlerExpr::BoolPropRead { path }) => {
            validate_scalar_value_read(lhs, elem, path, declared, err_msg)
        }
        (
            _,
            HandlerExpr::PropRead { path }
            | HandlerExpr::StrPropRead { path }
            | HandlerExpr::BoolPropRead { path },
        ) => validate_scalar_value_read(lhs, elem, path, declared, err_msg),
        (IrType::Str, HandlerExpr::Interpolation(_)) => {
            validate_expr_references(value, declared, None, err_msg)
        }
        (_, HandlerExpr::ItemRead { .. } | HandlerExpr::IndexRead { .. }) => {
            validate_expr_references(value, declared, None, err_msg)
        }
        _ => Err(IrLoadError::Validate(format!(
            "collection assignment `{lhs}` appends a value that does not match element type `{}`",
            scalar_type_name(elem)
        ))),
    }
}

fn validate_scalar_value_read(
    lhs: &str,
    elem: &IrType,
    path: &str,
    declared: &std::collections::HashMap<&str, IrStateType>,
    err_msg: &dyn Fn(&str) -> String,
) -> Result<(), IrLoadError> {
    match declared.get(path) {
        Some(IrStateType::Scalar(found)) if found == elem => Ok(()),
        Some(IrStateType::Scalar(found)) => Err(IrLoadError::Validate(format!(
            "collection assignment `{lhs}` appends `{path}` with type `{}`, expected `{}`",
            scalar_type_name(found),
            scalar_type_name(elem)
        ))),
        Some(IrStateType::Collection(_)) => Err(IrLoadError::Validate(format!(
            "collection assignment `{lhs}` cannot append collection state `{path}` as one element"
        ))),
        None => Err(IrLoadError::Validate(err_msg(path))),
    }
}

fn validate_for_header(
    binder: &str,
    index_binder: Option<&str>,
    collection: &HandlerExpr,
    declared: &std::collections::HashMap<&str, IrStateType>,
) -> Result<(), IrLoadError> {
    if binder.is_empty() {
        return Err(IrLoadError::Validate(
            "`for` binder must not be empty".into(),
        ));
    }
    if declared.contains_key(binder) {
        return Err(IrLoadError::Validate(format!(
            "`for` binder `{binder}` collides with a declared state"
        )));
    }
    if let Some(index) = index_binder {
        if index.is_empty() {
            return Err(IrLoadError::Validate(
                "`for` index binder must not be empty".into(),
            ));
        }
        if index == binder {
            return Err(IrLoadError::Validate(
                "`for` binder and index binder must be distinct".into(),
            ));
        }
        if declared.contains_key(index) {
            return Err(IrLoadError::Validate(format!(
                "`for` index binder `{index}` collides with a declared state"
            )));
        }
    }
    match collection {
        HandlerExpr::ListPropRead { path, elem } => {
            validate_collection_read_path(path, elem, declared, &|name| {
                format!("`for` references undeclared collection `{name}`")
            })
        }
        other => Err(IrLoadError::Validate(format!(
            "`for` collection must be a collection state read, got {other:?}"
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
    LBracket,
    RBracket,
    Comma,
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
            '[' => {
                tokens.push(Token::LBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
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

        let mut host_props = Vec::new();
        let mut host_bindings = Vec::new();
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
                Some(Token::Ident(s)) if s == "host" => {
                    let (props, bindings) = self.parse_host_member()?;
                    host_props.extend(props);
                    host_bindings.extend(bindings);
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
            host_props,
            host_bindings,
            states,
            root,
        })
    }

    fn parse_host_member(&mut self) -> Result<(Vec<IrProp>, Vec<IrBinding>), IrLoadError> {
        self.expect_keyword("host")?;
        match self.peek() {
            Some(Token::Ident(s)) if s == "prop" => Ok((vec![self.parse_prop()?], Vec::new())),
            Some(Token::Ident(s)) if s == "bind" => Ok((Vec::new(), vec![self.parse_binding()?])),
            other => Err(IrLoadError::Parse(format!(
                "expected `host prop` or `host bind`, got {other:?}"
            ))),
        }
    }

    fn parse_state(&mut self) -> Result<IrState, IrLoadError> {
        self.expect_keyword("state")?;
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let ty_str = self.expect_ident()?;
        let elem = match ty_str.as_str() {
            "i32" => IrType::I32,
            "string" => IrType::Str,
            "bool" => IrType::Bool,
            other => {
                return Err(IrLoadError::Parse(format!("unknown state type: {other}")));
            }
        };
        let ty = if matches!(self.peek(), Some(Token::LBracket)) {
            self.advance();
            self.expect(&Token::RBracket)?;
            IrStateType::Collection(elem)
        } else {
            IrStateType::Scalar(elem)
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
                Some(Token::Ident(s)) if s == "child" => {
                    children.push(IrMember::Widget(self.parse_child_slot()?))
                }
                Some(Token::Ident(s)) if s == "node" => {
                    children.push(IrMember::Widget(IrChildSlot {
                        node: self.parse_node()?,
                        slot_data: None,
                    }))
                }
                Some(Token::Ident(s)) if s == "if" => {
                    children.push(IrMember::ControlFlow(self.parse_if_member()?))
                }
                Some(Token::Ident(s)) if s == "for" => {
                    children.push(IrMember::ControlFlow(self.parse_for_member()?))
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

    fn parse_child_slot(&mut self) -> Result<IrChildSlot, IrLoadError> {
        self.expect_keyword("child")?;
        self.expect(&Token::LBrace)?;
        let mut slot_data = None;
        let mut node = None;

        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::Ident(s)) if s == "placement" => {
                    if slot_data.is_some() {
                        return Err(IrLoadError::Parse(
                            "malformed-placement-ir: duplicate `placement` block in child slot"
                                .into(),
                        ));
                    }
                    slot_data = Some(self.parse_slot_data()?);
                }
                Some(Token::Ident(s)) if s == "node" => {
                    if node.is_some() {
                        return Err(IrLoadError::Parse(
                            "malformed-placement-ir: duplicate `node` block in child slot".into(),
                        ));
                    }
                    node = Some(self.parse_node()?);
                }
                Some(other) => {
                    return Err(IrLoadError::Parse(format!(
                        "malformed-placement-ir: unexpected token in child slot: {other:?}"
                    )));
                }
                None => return Err(IrLoadError::Parse("unexpected EOF in child slot".into())),
            }
        }

        let node = node.ok_or_else(|| {
            IrLoadError::Parse("malformed-placement-ir: child slot missing `node` block".into())
        })?;
        Ok(IrChildSlot { node, slot_data })
    }

    fn parse_slot_data(&mut self) -> Result<IrSlotData, IrLoadError> {
        self.expect_keyword("placement")?;
        let kind = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        match kind.as_str() {
            "grid" => self.parse_grid_slot_data(),
            "zstack" => self.parse_zstack_slot_data(),
            other => Err(IrLoadError::Parse(format!(
                "malformed-placement-ir: unknown placement kind `{other}`"
            ))),
        }
    }

    fn parse_grid_slot_data(&mut self) -> Result<IrSlotData, IrLoadError> {
        let mut row = 0;
        let mut column = 0;
        let mut row_span = 1;
        let mut column_span = 1;
        let mut h_align = IrAlignment::Stretch;
        let mut v_align = IrAlignment::Stretch;
        let mut seen = std::collections::HashSet::new();

        while !matches!(self.peek(), Some(Token::RBrace)) {
            let key = self.expect_ident()?;
            if !seen.insert(key.clone()) {
                return Err(IrLoadError::Parse(format!(
                    "malformed-placement-ir: duplicate grid placement key `{key}`"
                )));
            }
            self.expect(&Token::Colon)?;
            match key.as_str() {
                "row" => row = self.expect_nonnegative_u32("grid.row")?,
                "column" => column = self.expect_nonnegative_u32("grid.column")?,
                "row-span" => row_span = self.expect_positive_u32("grid.row-span")?,
                "column-span" => {
                    column_span = self.expect_positive_u32("grid.column-span")?;
                }
                "h-align" => h_align = self.expect_alignment("grid.h-align")?,
                "v-align" => v_align = self.expect_alignment("grid.v-align")?,
                other => {
                    return Err(IrLoadError::Parse(format!(
                        "malformed-placement-ir: unknown grid placement key `{other}`"
                    )));
                }
            }
            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(IrSlotData::Grid {
            row,
            column,
            row_span,
            column_span,
            h_align,
            v_align,
        })
    }

    fn parse_zstack_slot_data(&mut self) -> Result<IrSlotData, IrLoadError> {
        let mut h_align = IrAlignment::Center;
        let mut v_align = IrAlignment::Center;
        let mut seen = std::collections::HashSet::new();

        while !matches!(self.peek(), Some(Token::RBrace)) {
            let key = self.expect_ident()?;
            if !seen.insert(key.clone()) {
                return Err(IrLoadError::Parse(format!(
                    "malformed-placement-ir: duplicate zstack placement key `{key}`"
                )));
            }
            self.expect(&Token::Colon)?;
            match key.as_str() {
                "h-align" => h_align = self.expect_alignment("zstack.h-align")?,
                "v-align" => v_align = self.expect_alignment("zstack.v-align")?,
                other => {
                    return Err(IrLoadError::Parse(format!(
                        "malformed-placement-ir: unknown zstack placement key `{other}`"
                    )));
                }
            }
            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(IrSlotData::ZStack { h_align, v_align })
    }

    fn expect_nonnegative_u32(&mut self, label: &str) -> Result<u32, IrLoadError> {
        match self.advance() {
            Some(Token::Int(n)) if *n >= 0 => Ok(*n as u32),
            other => Err(IrLoadError::Parse(format!(
                "malformed-placement-ir: `{label}` must be a non-negative integer, got {other:?}"
            ))),
        }
    }

    fn expect_positive_u32(&mut self, label: &str) -> Result<u32, IrLoadError> {
        match self.advance() {
            Some(Token::Int(n)) if *n >= 1 => Ok(*n as u32),
            other => Err(IrLoadError::Parse(format!(
                "malformed-placement-ir: `{label}` must be a positive integer, got {other:?}"
            ))),
        }
    }

    fn expect_alignment(&mut self, label: &str) -> Result<IrAlignment, IrLoadError> {
        match self.advance() {
            Some(Token::Ident(s)) => match s.as_str() {
                "start" => Ok(IrAlignment::Start),
                "center" => Ok(IrAlignment::Center),
                "end" => Ok(IrAlignment::End),
                "stretch" => Ok(IrAlignment::Stretch),
                other => Err(IrLoadError::Parse(format!(
                    "malformed-placement-ir: `{label}` must be one of start, center, end, stretch, got `{other}`"
                ))),
            },
            other => Err(IrLoadError::Parse(format!(
                "malformed-placement-ir: `{label}` must be an alignment keyword, got {other:?}"
            ))),
        }
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
                Some(Token::Ident(s)) if s == "child" => {
                    body.push(IrMember::Widget(self.parse_child_slot()?));
                }
                Some(Token::Ident(s)) if s == "node" => {
                    body.push(IrMember::Widget(IrChildSlot {
                        node: self.parse_node()?,
                        slot_data: None,
                    }));
                }
                Some(Token::Ident(s)) if s == "if" => {
                    body.push(IrMember::ControlFlow(self.parse_if_member()?));
                }
                Some(Token::Ident(s)) if s == "for" => {
                    body.push(IrMember::ControlFlow(self.parse_for_member()?));
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

    fn parse_for_member(&mut self) -> Result<ControlFlowNode, IrLoadError> {
        self.expect_keyword("for")?;
        let binder = self.expect_ident()?;
        let index_binder = if matches!(self.peek(), Some(Token::Comma)) {
            self.advance();
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.expect_keyword("in")?;
        let collection_name = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let mut body = Vec::new();
        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::Ident(s)) if s == "child" => {
                    body.push(IrMember::Widget(self.parse_child_slot()?));
                }
                Some(Token::Ident(s)) if s == "node" => {
                    body.push(IrMember::Widget(IrChildSlot {
                        node: self.parse_node()?,
                        slot_data: None,
                    }));
                }
                Some(Token::Ident(s)) if s == "if" => {
                    body.push(IrMember::ControlFlow(self.parse_if_member()?));
                }
                Some(Token::Ident(s)) if s == "for" => {
                    body.push(IrMember::ControlFlow(self.parse_for_member()?));
                }
                Some(other) => {
                    return Err(IrLoadError::Parse(format!(
                        "unexpected token in for body: {other:?}"
                    )));
                }
                None => return Err(IrLoadError::Parse("unexpected EOF in for body".into())),
            }
        }
        Ok(ControlFlowNode::For {
            binder,
            index_binder,
            collection: HandlerExpr::ListPropRead {
                path: collection_name,
                elem: IrType::I32,
            },
            body,
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
        // Optional parenthesised string argument (DD-M4-P2-005), e.g.
        // `on key-down("ArrowLeft") { ... }`. The IR text grammar's
        // canonical machine format always writes a plain string literal
        // here — no interpolation, mirroring `wasamoc`'s parser.
        let arg = if matches!(self.peek(), Some(Token::LParen)) {
            self.advance();
            let value = match self.advance() {
                Some(Token::Str(s)) => s.clone(),
                other => {
                    return Err(IrLoadError::Parse(format!(
                        "expected string literal in handler argument, got {other:?}"
                    )));
                }
            };
            self.expect(&Token::RParen)?;
            Some(value)
        } else {
            None
        };
        self.expect(&Token::LBrace)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::RBrace)?;
        Ok(IrHandler { signal, arg, expr })
    }

    fn parse_literal(&mut self) -> Result<IrLiteral, IrLoadError> {
        match self.peek() {
            Some(Token::LBracket) => return self.parse_list_literal(),
            _ => {}
        }
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

    fn parse_list_literal(&mut self) -> Result<IrLiteral, IrLoadError> {
        self.expect(&Token::LBracket)?;
        let mut items = Vec::new();
        if matches!(self.peek(), Some(Token::RBracket)) {
            self.advance();
            return Ok(IrLiteral::List(items));
        }
        loop {
            let item = match self.advance() {
                Some(Token::Int(n)) => IrLiteral::Int(*n),
                Some(Token::Str(s)) => IrLiteral::Str(s.clone()),
                Some(Token::Ident(s)) if s == "true" => IrLiteral::Bool(true),
                Some(Token::Ident(s)) if s == "false" => IrLiteral::Bool(false),
                other => {
                    return Err(IrLoadError::Parse(format!(
                        "expected scalar list literal element, got {other:?}"
                    )));
                }
            };
            items.push(item);
            match self.peek() {
                Some(Token::Comma) => {
                    self.advance();
                }
                Some(Token::RBracket) => {
                    self.advance();
                    break;
                }
                other => {
                    return Err(IrLoadError::Parse(format!(
                        "expected `,` or `]` in list literal, got {other:?}"
                    )));
                }
            }
        }
        Ok(IrLiteral::List(items))
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
            Some(Token::LBracket) => match self.parse_list_literal()? {
                IrLiteral::List(items) => Ok(HandlerExpr::ListLit(items)),
                _ => unreachable!(),
            },
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
            "list-prop-read" => {
                let path = self.expect_ident()?;
                HandlerExpr::ListPropRead {
                    path,
                    elem: IrType::I32,
                }
            }
            "item-read" => {
                let binder = self.expect_ident()?;
                HandlerExpr::ItemRead { binder }
            }
            "index-read" => {
                let binder = self.expect_ident()?;
                HandlerExpr::IndexRead { binder }
            }
            "list-append" => {
                let path = self.expect_ident()?;
                let value = self.parse_expr()?;
                HandlerExpr::ListAppend {
                    path,
                    elem: IrType::I32,
                    value: Box::new(value),
                }
            }
            "list-drop-last" => {
                let path = self.expect_ident()?;
                HandlerExpr::ListDropLast {
                    path,
                    elem: IrType::I32,
                }
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
        match &state.ty {
            IrStateType::Scalar(IrType::I32) => {
                let initial = match &state.default {
                    IrLiteral::Int(n) => *n,
                    _ => 0,
                };
                registry
                    .i32s
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrStateType::Scalar(IrType::Str) => {
                let initial = match &state.default {
                    IrLiteral::Str(s) => s.clone(),
                    _ => String::new(),
                };
                registry
                    .strings
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrStateType::Scalar(IrType::Bool) => {
                let initial = match &state.default {
                    IrLiteral::Bool(b) => *b,
                    _ => false,
                };
                registry
                    .bools
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrStateType::Collection(IrType::I32) => {
                let initial = match &state.default {
                    IrLiteral::List(items) => items
                        .iter()
                        .filter_map(|item| match item {
                            IrLiteral::Int(n) => Some(*n),
                            _ => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                registry
                    .i32_lists
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrStateType::Collection(IrType::Str) => {
                let initial = match &state.default {
                    IrLiteral::List(items) => items
                        .iter()
                        .filter_map(|item| match item {
                            IrLiteral::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                registry
                    .string_lists
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrStateType::Collection(IrType::Bool) => {
                let initial = match &state.default {
                    IrLiteral::List(items) => items
                        .iter()
                        .filter_map(|item| match item {
                            IrLiteral::Bool(b) => Some(*b),
                            _ => None,
                        })
                        .collect(),
                    _ => Vec::new(),
                };
                registry
                    .bool_lists
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
    build_node_with_loop_context(node, compositor, renderer, registry, None)
}

fn build_node_with_loop_context(
    node: &IrNode,
    compositor: &Compositor,
    renderer: &TextRenderer,
    registry: &Rc<SignalRegistry>,
    loop_context: Option<&ForItemContext>,
) -> Result<Box<WidgetNode>, IrLoadError> {
    let mut widget = construct_widget(node, compositor, renderer, registry)?;

    // M4-Phase 2 T6: write the authored focus annotation (dsl_spec §4.19,
    // DD-M4-P2-005 A1) onto the freshly constructed node. One
    // kind-independent site rather than scattered through
    // `construct_widget`'s per-kind arms, because the annotation is not
    // part of any widget kind's own shape — it is read back only by
    // `WidgetNode::focus_role`. `validate` has already rejected a
    // non-`Bool` value and a non-container carrier (`FOCUS_ANNOTATION_CONTAINERS`
    // above), so the extracts are total over the accept set: an absent
    // prop and an explicit `false` both correctly yield `false`.
    let focus_group = extract_bool_prop(&node.props, "focus-group").unwrap_or(false);
    let modal_scope = extract_bool_prop(&node.props, "modal-scope").unwrap_or(false);
    widget.set_focus_annotation(focus_group, modal_scope);

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
        let handle = match (prop_ty, loop_context) {
            (IrType::Bool, Some(item)) => register_for_item_bool_binding(
                target,
                binding.expr.clone(),
                Rc::clone(registry),
                item.clone(),
                widget_write_property_bool,
            ),
            (IrType::Bool, None) => register_bool_binding(
                target,
                binding.expr.clone(),
                Rc::clone(registry),
                widget_write_property_bool,
            ),
            // I32 and Str properties continue through the M2 string-baked
            // writer (stringified by `evaluate_binding`, parsed at the
            // per-widget setter — typed-i32 writer lands when its use case
            // arrives).
            (IrType::I32 | IrType::Str, Some(item)) => register_for_item_binding(
                target,
                binding.expr.clone(),
                Rc::clone(registry),
                item.clone(),
                widget_write_property,
            ),
            (IrType::I32 | IrType::Str, None) => register_binding(
                target,
                binding.expr.clone(),
                Rc::clone(registry),
                widget_write_property,
            ),
        };
        widget.bindings.push(handle);
    }

    // Handlers: attach each `on` body via Phase 3's set_inline_handler
    // path. The storage key is the DD-M4-P2-005 canonical spelling
    // (`wasamo_ir::signal_key`) — `clicked` for a no-argument handler,
    // `key-down("ArrowLeft")` for an argument-carrying one — so a later
    // stage's dispatcher can look handlers up by the same composed key.
    // Nothing else may compose this string.
    for handler in &node.handlers {
        widget.set_inline_handler(
            wasamo_ir::signal_key(&handler.signal, handler.arg.as_deref()),
            handler.expr.clone(),
        );
    }

    // Children: recurse and attach via the Phase 4 internal mutation API.
    let declared_slots = Rc::new(RefCell::new(Vec::with_capacity(node.children.len())));
    for (declared_member_index, member) in node.children.iter().enumerate() {
        append_static_member(
            member,
            declared_member_index,
            Rc::clone(&declared_slots),
            &mut widget,
            compositor,
            renderer,
            registry,
            loop_context,
        )?;
    }

    Ok(widget)
}

fn append_static_member(
    member: &IrMember,
    declared_member_index: usize,
    declared_slots: Rc<RefCell<Vec<DeclaredMemberSlot>>>,
    parent: &mut WidgetNode,
    compositor: &Compositor,
    renderer: &TextRenderer,
    registry: &Rc<SignalRegistry>,
    loop_context: Option<&ForItemContext>,
) -> Result<(), IrLoadError> {
    match member {
        IrMember::Widget(child) => {
            declared_slots.borrow_mut().push(DeclaredMemberSlot::Widget);
            let child_widget = build_node_with_loop_context(
                &child.node,
                compositor,
                renderer,
                registry,
                loop_context,
            )?;
            parent
                .insert_child_with_slot_data(
                    parent.child_count(),
                    child_widget,
                    slot_data_for_parent(parent, child),
                )
                .map_err(|e| IrLoadError::Build(format!("insert_child failed: {e:?}")))?;
        }
        IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
            let branch = branches
                .first()
                .ok_or_else(|| IrLoadError::Build("`if` control flow has no branch".into()))?;
            let body = match branch.body.first() {
                Some(IrMember::Widget(slot)) => slot.clone(),
                _ => {
                    return Err(IrLoadError::Build(
                        "`if` body must contain one widget member".into(),
                    ));
                }
            };
            let state = Rc::new(RefCell::new(ConditionalRuntimeState { live_child: false }));
            declared_slots
                .borrow_mut()
                .push(DeclaredMemberSlot::Conditional(Rc::clone(&state)));
            let parent_id = WidgetId(parent as *mut WidgetNode as *mut ());
            let target = BindingTarget::ConditionalSubtree {
                parent: parent_id,
                declared_member_index,
            };
            let slots_for_effect = Rc::clone(&declared_slots);
            let registry_for_effect = Rc::clone(registry);
            let handle = register_conditional_binding(
                target,
                branch.condition.clone(),
                Rc::clone(registry),
                move |parent_id, declared_member_index, present| {
                    mutate_conditional_subtree(
                        parent_id,
                        declared_member_index,
                        present,
                        &body,
                        &slots_for_effect,
                        &registry_for_effect,
                    );
                },
            );
            parent.bindings.push(handle);
        }
        IrMember::ControlFlow(ControlFlowNode::For {
            binder,
            index_binder,
            collection,
            body,
        }) => {
            let (collection_name, elem, live_children) =
                static_collection_cardinality(collection, registry)?;
            let state = Rc::new(RefCell::new(ForLoopRuntimeState { live_children }));
            declared_slots
                .borrow_mut()
                .push(DeclaredMemberSlot::ForLoop(Rc::clone(&state)));
            let body = match body.first() {
                Some(IrMember::Widget(slot)) => slot,
                _ => {
                    return Err(IrLoadError::Build(
                        "`for` body must contain one widget member".into(),
                    ));
                }
            };
            let base_index = {
                let slots = declared_slots.borrow();
                materialized_offset_for_declared_slot(declared_member_index, &slots)
            };
            for position in 0..live_children {
                let item_context = ForItemContext {
                    collection: collection_name.clone(),
                    elem: elem.clone(),
                    binder: binder.clone(),
                    index_binder: index_binder.clone(),
                    position,
                };
                let child_widget = build_node_with_loop_context(
                    &body.node,
                    compositor,
                    renderer,
                    registry,
                    Some(&item_context),
                )?;
                let insert_index = base_index + position;
                parent
                    .insert_child_with_slot_data(
                        insert_index,
                        child_widget,
                        slot_data_for_parent(parent, body),
                    )
                    .map_err(|e| IrLoadError::Build(format!("insert_child failed: {e:?}")))?;
            }
            let parent_id = WidgetId(parent as *mut WidgetNode as *mut ());
            let target = BindingTarget::ForLoopSubtree {
                parent: parent_id,
                declared_member_index,
            };
            let slots_for_effect = Rc::clone(&declared_slots);
            let registry_for_effect = Rc::clone(registry);
            let body_for_effect = body.clone();
            let binder_for_effect = binder.clone();
            let index_binder_for_effect = index_binder.clone();
            let collection_for_effect = collection_name.clone();
            let elem_for_effect = elem.clone();
            let handle = register_for_loop_binding(
                target,
                collection.clone(),
                Rc::clone(registry),
                move |parent_id, declared_member_index, new_len| {
                    mutate_for_loop_subtree(
                        parent_id,
                        declared_member_index,
                        new_len,
                        &body_for_effect,
                        &binder_for_effect,
                        &index_binder_for_effect,
                        &collection_for_effect,
                        &elem_for_effect,
                        &slots_for_effect,
                        &registry_for_effect,
                    );
                },
            );
            parent.bindings.push(handle);
        }
    }
    Ok(())
}

fn static_collection_cardinality(
    collection: &HandlerExpr,
    registry: &SignalRegistry,
) -> Result<(String, IrType, usize), IrLoadError> {
    let HandlerExpr::ListPropRead { path, elem } = collection else {
        return Err(IrLoadError::Build(format!(
            "`for` collection must be a collection state read, got {collection:?}"
        )));
    };
    let len = match elem {
        IrType::I32 => registry
            .i32_lists
            .get(path)
            .ok_or_else(|| IrLoadError::Build(format!("unknown i32 collection `{path}`")))?
            .get_untracked()
            .len(),
        IrType::Str => registry
            .string_lists
            .get(path)
            .ok_or_else(|| IrLoadError::Build(format!("unknown string collection `{path}`")))?
            .get_untracked()
            .len(),
        IrType::Bool => registry
            .bool_lists
            .get(path)
            .ok_or_else(|| IrLoadError::Build(format!("unknown bool collection `{path}`")))?
            .get_untracked()
            .len(),
    };
    Ok((path.clone(), elem.clone(), len))
}

fn declared_slot_live_cardinality(slot: &DeclaredMemberSlot) -> usize {
    match slot {
        DeclaredMemberSlot::Widget => 1,
        DeclaredMemberSlot::Conditional(state) => usize::from(state.borrow().live_child),
        DeclaredMemberSlot::ForLoop(state) => state.borrow().live_children,
    }
}

fn materialized_offset_for_declared_slot(
    declared_member_index: usize,
    declared_slots: &[DeclaredMemberSlot],
) -> usize {
    declared_slots
        .iter()
        .take(declared_member_index)
        .map(declared_slot_live_cardinality)
        .sum()
}

#[allow(dead_code)]
fn total_materialized_children(declared_slots: &[DeclaredMemberSlot]) -> usize {
    declared_slots
        .iter()
        .map(declared_slot_live_cardinality)
        .sum()
}

fn plan_tail_range_change(old_len: usize, new_len: usize) -> TailRangePlan {
    if new_len > old_len {
        TailRangePlan::Insert {
            start: old_len,
            count: new_len - old_len,
        }
    } else if new_len < old_len {
        TailRangePlan::Remove {
            tail_first_indices: (new_len..old_len).rev().collect(),
        }
    } else {
        TailRangePlan::NoOp
    }
}

fn mutate_conditional_subtree(
    parent_id: WidgetId,
    declared_member_index: usize,
    present: bool,
    body: &IrChildSlot,
    declared_slots: &Rc<RefCell<Vec<DeclaredMemberSlot>>>,
    registry: &Rc<SignalRegistry>,
) {
    let parent_ptr = parent_id.0 as *mut WidgetNode;
    if parent_ptr.is_null() {
        return;
    }
    let state = {
        let slots = declared_slots.borrow();
        match slots.get(declared_member_index) {
            Some(DeclaredMemberSlot::Conditional(state)) => Rc::clone(state),
            _ => {
                eprintln!(
                    "wasamo: conditional binding declared slot {declared_member_index} is missing"
                );
                return;
            }
        }
    };
    let currently_present = state.borrow().live_child;
    if present == currently_present {
        return;
    }
    let live_index = {
        let slots = declared_slots.borrow();
        materialized_offset_for_declared_slot(declared_member_index, &slots)
    };
    unsafe {
        let parent = &mut *parent_ptr;
        if present {
            let compositor = crate::get_compositor();
            let renderer = crate::get_text_renderer();
            match build_node(&body.node, compositor, renderer, registry) {
                Ok(child) => {
                    let result = insert_structural_child(
                        parent,
                        live_index,
                        child,
                        slot_data_for_parent(parent, body),
                    );
                    match result {
                        Ok(()) => {
                            state.borrow_mut().live_child = true;
                            crate::emit::mark_layout_dirty_for(parent_ptr);
                        }
                        Err(e) => eprintln!("wasamo: conditional insert_child failed: {e:?}"),
                    }
                }
                Err(e) => eprintln!("wasamo: conditional subtree build failed: {e}"),
            }
        } else {
            match remove_structural_child(parent, live_index) {
                Ok(removed) => {
                    crate::widget::widget_destroy(removed);
                    state.borrow_mut().live_child = false;
                    crate::emit::mark_layout_dirty_for(parent_ptr);
                }
                Err(e) => eprintln!("wasamo: conditional remove_child failed: {e:?}"),
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn mutate_for_loop_subtree(
    parent_id: WidgetId,
    declared_member_index: usize,
    new_len: usize,
    body: &IrChildSlot,
    binder: &str,
    index_binder: &Option<String>,
    collection_name: &str,
    elem: &IrType,
    declared_slots: &Rc<RefCell<Vec<DeclaredMemberSlot>>>,
    registry: &Rc<SignalRegistry>,
) {
    let parent_ptr = parent_id.0 as *mut WidgetNode;
    if parent_ptr.is_null() {
        return;
    }
    let state = {
        let slots = declared_slots.borrow();
        match slots.get(declared_member_index) {
            Some(DeclaredMemberSlot::ForLoop(state)) => Rc::clone(state),
            _ => {
                eprintln!("wasamo: for binding declared slot {declared_member_index} is missing");
                return;
            }
        }
    };
    let old_len = state.borrow().live_children;
    if old_len == new_len {
        return;
    }
    let live_index = {
        let slots = declared_slots.borrow();
        materialized_offset_for_declared_slot(declared_member_index, &slots)
    };
    let plan = plan_tail_range_change(old_len, new_len);
    unsafe {
        let parent = &mut *parent_ptr;
        match plan {
            TailRangePlan::Insert { start, count } => {
                let compositor = crate::get_compositor();
                let renderer = crate::get_text_renderer();
                let mut staged = Vec::with_capacity(count);
                for position in start..(start + count) {
                    let item_context = ForItemContext {
                        collection: collection_name.to_string(),
                        elem: elem.clone(),
                        binder: binder.to_string(),
                        index_binder: index_binder.clone(),
                        position,
                    };
                    match build_node_with_loop_context(
                        &body.node,
                        compositor,
                        renderer,
                        registry,
                        Some(&item_context),
                    ) {
                        Ok(child) => staged.push(child),
                        Err(e) => {
                            for child in staged {
                                crate::widget::widget_destroy(child);
                            }
                            eprintln!(
                                "wasamo: for range insert build failed at position {position}: {e}"
                            );
                            return;
                        }
                    }
                }
                let slot_data = slot_data_for_parent(parent, body);
                let mut inserted = 0usize;
                // `while let` (not `for`) so the staged iterator stays usable
                // inside the failure branch to dispose the not-yet-committed
                // remainder.
                let mut staged_iter = staged.into_iter().enumerate();
                while let Some((offset, child)) = staged_iter.next() {
                    let insert_index = live_index + start + offset;
                    // Production inserts directly. A `debug_assertions`-gated
                    // test-only fault seam (see `__arm_structural_insert_fault_for_test`;
                    // Rust-side fault injection, NOT a WinRT mock) can force the
                    // Nth commit insert to fail so the rollback branch below is
                    // directly exercised. Absent from release builds.
                    #[cfg(debug_assertions)]
                    let insert_result = if structural_insert_fault_armed(inserted) {
                        // The staged child is not inserted; dispose it (mirrors a
                        // real insert failure, where `insert_child` consumes the
                        // child) and report the error to drive rollback.
                        crate::widget::widget_destroy(child);
                        Err(crate::widget::MutationError::IndexOutOfBounds)
                    } else {
                        insert_structural_child(parent, insert_index, child, slot_data)
                    };
                    #[cfg(not(debug_assertions))]
                    let insert_result =
                        insert_structural_child(parent, insert_index, child, slot_data);
                    match insert_result {
                        Ok(()) => inserted += 1,
                        Err(e) => {
                            // Roll back so the tree and registry return to the
                            // pre-write baseline (review finding #4). `WidgetNode`
                            // has no `Drop`, so a bare drop skips `widget_destroy`'s
                            // `remove_for_widget`; any child holding a `registry`
                            // entry would leak. Today's handler-free `for`-body
                            // children hold none (per-item `EffectHandle`s
                            // self-dispose on `Drop`), so this branch's disposal is
                            // a *defensive* symmetry with the staging-failure branch
                            // and the no-`Drop` ⇒ explicit-disposal invariant — not
                            // an active leak fix for current bodies, but required
                            // for any future body shape that registers entries.
                            //   (a) remove + destroy the committed prefix, tail-first;
                            for rollback in (0..inserted).rev() {
                                if let Ok(removed) =
                                    remove_structural_child(parent, live_index + start + rollback)
                                {
                                    crate::widget::widget_destroy(removed);
                                }
                            }
                            //   (b) destroy the staged children not yet committed.
                            for (_, leftover) in staged_iter.by_ref() {
                                crate::widget::widget_destroy(leftover);
                            }
                            // The faulting child itself was already consumed: by
                            // `widget_destroy` in the test seam, or by `insert_child`
                            // in production (its by-value contract drops the child on
                            // a WinRT failure — a near-unreachable path here, as the
                            // index is always valid and the child freshly unattached;
                            // recovering it would mean changing `insert_child`'s
                            // signature across the conditional / ABI / static callers,
                            // which is out of this task's scope).
                            eprintln!("wasamo: for range insert_child failed: {e:?}");
                            return;
                        }
                    }
                }
                state.borrow_mut().live_children = new_len;
                crate::emit::mark_layout_dirty_for(parent_ptr);
            }
            TailRangePlan::Remove { tail_first_indices } => {
                for position in tail_first_indices {
                    match remove_structural_child(parent, live_index + position) {
                        Ok(removed) => crate::widget::widget_destroy(removed),
                        Err(e) => {
                            eprintln!("wasamo: for range remove_child failed: {e:?}");
                            return;
                        }
                    }
                }
                state.borrow_mut().live_children = new_len;
                crate::emit::mark_layout_dirty_for(parent_ptr);
            }
            TailRangePlan::NoOp => {}
        }
    }
}

fn insert_structural_child(
    parent: &mut WidgetNode,
    index: usize,
    child: Box<WidgetNode>,
    slot_data: Option<SlotData>,
) -> Result<(), crate::widget::MutationError> {
    parent.insert_child_with_slot_data(index, child, slot_data)
}

fn remove_structural_child(
    parent: &mut WidgetNode,
    index: usize,
) -> Result<Box<WidgetNode>, crate::widget::MutationError> {
    parent.remove_child(index)
}

// ── Test-only structural-insert fault seam (review finding #2) ──────────────
//
// Rust-side fault injection — NOT a WinRT/OS API mock — that forces the Nth
// commit-stage `insert_structural_child` in `mutate_for_loop_subtree` to fail,
// so the partial-insert rollback branch is directly exercised by a mock-free
// Windows integration test. Gated on `debug_assertions`, so the seam, the arm
// helpers, and the cost of the per-insert check are all absent from release
// builds (`cargo build --release` disables `debug_assertions`); the project's
// CI runs `cargo test` in the dev profile, where `debug_assertions` is on.
#[cfg(debug_assertions)]
thread_local! {
    static FAIL_STRUCTURAL_INSERT_AT: std::cell::Cell<Option<usize>> =
        std::cell::Cell::new(None);
}

#[cfg(debug_assertions)]
fn structural_insert_fault_armed(inserted_so_far: usize) -> bool {
    FAIL_STRUCTURAL_INSERT_AT.with(|cell| cell.get() == Some(inserted_so_far))
}

/// Arm the for-range commit loop to fail its `inserted_index`-th structural
/// insert (0-based count of successful inserts so far). Test-only; absent from
/// release. Call under the integration `test_lock` and disarm before any
/// assertion that may panic, to avoid leaking the armed state onto the
/// reused runtime thread.
#[cfg(debug_assertions)]
#[doc(hidden)]
pub fn __arm_structural_insert_fault_for_test(inserted_index: usize) {
    FAIL_STRUCTURAL_INSERT_AT.with(|cell| cell.set(Some(inserted_index)));
}

#[cfg(debug_assertions)]
#[doc(hidden)]
pub fn __disarm_structural_insert_fault_for_test() {
    FAIL_STRUCTURAL_INSERT_AT.with(|cell| cell.set(None));
}

fn slot_data_for_parent(parent: &WidgetNode, slot: &IrChildSlot) -> Option<SlotData> {
    slot_data_for_parent_kind(parent.is_zstack(), parent.is_grid(), slot)
}

fn slot_data_for_parent_kind(
    parent_is_zstack: bool,
    parent_is_grid: bool,
    slot: &IrChildSlot,
) -> Option<SlotData> {
    if parent_is_zstack {
        Some(SlotData::ZStack(zstack_payload_from_ir_slot(slot)))
    } else if parent_is_grid {
        Some(SlotData::Grid(grid_placement_from_slot(slot)))
    } else {
        None
    }
}

#[cfg(test)]
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
    _registry: &SignalRegistry,
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
            // CF-2 disposition (2026-08-07): read `enabled` exactly the way
            // the `ToggleButton` arm below does. A literal `enabled: false`
            // was previously silently dropped here — this arm read only
            // `text` / `style`, and `WidgetNode::button` hard-coded
            // `enabled: true` — so only a *state-bound* `enabled` could
            // disable a plain Button. `has_binding` still defers to the
            // binding's initial run, same as ToggleButton.
            let enabled = extract_bool_prop(&node.props, "enabled").unwrap_or(true);
            let initial = if has_binding(&node.bindings, "text") {
                String::new()
            } else {
                label
            };
            let initial_enabled = if has_binding(&node.bindings, "enabled") {
                true
            } else {
                enabled
            };
            WidgetNode::button_with_enabled(compositor, renderer, &initial, style, initial_enabled)
                .map_err(|e| IrLoadError::Build(format!("button: {e}")))
        }
        "ToggleButton" => {
            let label = extract_str_prop(&node.props, "text").unwrap_or_default();
            let style = extract_button_style(&node.props, "style");
            let enabled = extract_bool_prop(&node.props, "enabled").unwrap_or(true);
            let checked = extract_bool_prop(&node.props, "checked").unwrap_or(false);
            let initial_label = if has_binding(&node.bindings, "text") {
                String::new()
            } else {
                label
            };
            let initial_enabled = if has_binding(&node.bindings, "enabled") {
                true
            } else {
                enabled
            };
            let initial_checked = if has_binding(&node.bindings, "checked") {
                false
            } else {
                checked
            };
            WidgetNode::toggle_button(
                compositor,
                renderer,
                &initial_label,
                style,
                initial_enabled,
                initial_checked,
            )
            .map_err(|e| IrLoadError::Build(format!("toggle_button: {e}")))
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
        // M3-Phase 5 T3 / M3-Phase 7b T3: Grid materialisation.
        // The track lists live on `node.kind_payload` (not `node.props` —
        // `IrProp.value` stays strictly `IrLiteral`). Per-child placement
        // is converted from each `IrChildSlot.slot_data` during child
        // insertion, so this arm builds only the Grid shell + track lists.
        // `validate()` has already rejected malformed track lists /
        // placements / overlaps before this arm runs (DD-M3-P5-006).
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
            WidgetNode::grid(compositor, columns, rows)
                .map_err(|e| IrLoadError::Build(format!("grid: {e}")))
        }
        // M3-Phase 6 T3 / M3-Phase 7 T5: ZStack materialisation. Per-child
        // placement annotations are carried on child slots; document order
        // is preserved by the generic child append loop below.
        "ZStack" => {
            WidgetNode::zstack(compositor).map_err(|e| IrLoadError::Build(format!("zstack: {e}")))
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

fn zstack_payload_from_ir_slot(slot: &IrChildSlot) -> ZStackPlacement {
    match &slot.slot_data {
        Some(IrSlotData::ZStack { h_align, v_align }) => ZStackPlacement {
            h_align: to_layout_alignment(*h_align),
            v_align: to_layout_alignment(*v_align),
        },
        _ => ZStackPlacement {
            h_align: Alignment::Center,
            v_align: Alignment::Center,
        },
    }
}

fn grid_placement_from_slot(slot: &IrChildSlot) -> CellPlacement {
    match &slot.slot_data {
        Some(IrSlotData::Grid {
            row,
            column,
            row_span,
            column_span,
            h_align,
            v_align,
        }) => CellPlacement {
            row: *row,
            column: *column,
            row_span: *row_span,
            column_span: *column_span,
            h_align: to_layout_alignment(*h_align),
            v_align: to_layout_alignment(*v_align),
        },
        _ => CellPlacement {
            row: 0,
            column: 0,
            row_span: 1,
            column_span: 1,
            h_align: Alignment::Stretch,
            v_align: Alignment::Stretch,
        },
    }
}

fn to_layout_alignment(alignment: IrAlignment) -> Alignment {
    match alignment {
        IrAlignment::Start => Alignment::Leading,
        IrAlignment::Center => Alignment::Center,
        IrAlignment::End => Alignment::Trailing,
        IrAlignment::Stretch => Alignment::Stretch,
    }
}

#[cfg(test)]
fn collect_static_zstack_child_placement_slots(
    members: &[IrMember],
    registry: &SignalRegistry,
) -> Result<Vec<ZStackPlacement>, IrLoadError> {
    let mut placements = Vec::new();
    for member in members {
        match member {
            IrMember::Widget(slot) => placements.push(zstack_payload_from_ir_slot(slot)),
            IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
                let branch = branches
                    .first()
                    .ok_or_else(|| IrLoadError::Build("`if` control flow has no branch".into()))?;
                if evaluate_static_condition(&branch.condition, registry)? {
                    let body = match branch.body.first() {
                        Some(IrMember::Widget(slot)) => slot,
                        _ => {
                            return Err(IrLoadError::Build(
                                "`if` body must contain one widget member".into(),
                            ));
                        }
                    };
                    placements.push(zstack_payload_from_ir_slot(body));
                }
            }
            IrMember::ControlFlow(ControlFlowNode::For {
                collection, body, ..
            }) => {
                let (_, _, live_children) = static_collection_cardinality(collection, registry)?;
                let body = match body.first() {
                    Some(IrMember::Widget(slot)) => slot,
                    _ => {
                        return Err(IrLoadError::Build(
                            "`for` body must contain one widget member".into(),
                        ));
                    }
                };
                for _ in 0..live_children {
                    placements.push(zstack_payload_from_ir_slot(body));
                }
            }
        }
    }
    Ok(placements)
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
        ("ToggleButton", "text") => Some((PROP_BUTTON_LABEL, IrType::Str)),
        ("ToggleButton", "style") => Some((PROP_BUTTON_STYLE, IrType::I32)),
        ("ToggleButton", "enabled") => Some((PROP_BUTTON_ENABLED, IrType::Bool)),
        ("ToggleButton", "checked") => Some((PROP_TOGGLEBUTTON_CHECKED, IrType::Bool)),
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

fn extract_bool_prop(props: &[IrProp], name: &str) -> Option<bool> {
    props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Bool(b) => Some(*b),
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

    fn child_slot(node: IrNode) -> IrChildSlot {
        IrChildSlot {
            node,
            slot_data: None,
        }
    }

    fn text_node() -> IrNode {
        IrNode {
            widget_type: "Text".into(),
            props: Vec::new(),
            bindings: Vec::new(),
            handlers: Vec::new(),
            children: Vec::new(),
            kind_payload: None,
        }
    }

    fn zstack_slot(node: IrNode, h_align: IrAlignment, v_align: IrAlignment) -> IrChildSlot {
        IrChildSlot {
            node,
            slot_data: Some(IrSlotData::ZStack { h_align, v_align }),
        }
    }

    fn grid_slot(
        node: IrNode,
        row: u32,
        column: u32,
        h_align: IrAlignment,
        v_align: IrAlignment,
    ) -> IrChildSlot {
        IrChildSlot {
            node,
            slot_data: Some(IrSlotData::Grid {
                row,
                column,
                row_span: 2,
                column_span: 3,
                h_align,
                v_align,
            }),
        }
    }

    #[test]
    fn slot_data_for_parent_kind_maps_zstack_slot_payload() {
        let slot = zstack_slot(text_node(), IrAlignment::End, IrAlignment::Start);

        assert_eq!(
            slot_data_for_parent_kind(true, false, &slot),
            Some(SlotData::ZStack(ZStackPlacement {
                h_align: Alignment::Trailing,
                v_align: Alignment::Leading,
            }))
        );
    }

    #[test]
    fn slot_data_for_parent_kind_maps_grid_slot_payload() {
        let slot = grid_slot(text_node(), 4, 5, IrAlignment::Center, IrAlignment::Stretch);

        assert_eq!(
            slot_data_for_parent_kind(false, true, &slot),
            Some(SlotData::Grid(CellPlacement {
                row: 4,
                column: 5,
                row_span: 2,
                column_span: 3,
                h_align: Alignment::Center,
                v_align: Alignment::Stretch,
            }))
        );
    }

    #[test]
    fn slot_data_for_parent_kind_normalizes_non_placement_parent_to_none() {
        let slot = zstack_slot(text_node(), IrAlignment::End, IrAlignment::Start);

        assert_eq!(slot_data_for_parent_kind(false, false, &slot), None);
    }

    #[test]
    fn slot_data_for_parent_kind_defaults_missing_zstack_payload_to_center() {
        let slot = child_slot(text_node());

        assert_eq!(
            slot_data_for_parent_kind(true, false, &slot),
            Some(SlotData::ZStack(ZStackPlacement::centered()))
        );
    }

    #[test]
    fn slot_data_for_parent_kind_defaults_missing_grid_payload_to_origin_stretch() {
        let slot = child_slot(text_node());

        assert_eq!(
            slot_data_for_parent_kind(false, true, &slot),
            Some(SlotData::Grid(CellPlacement::default_grid()))
        );
    }

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
    fn resolve_prop_key_togglebutton_checked_is_bool() {
        let (key, ty) =
            resolve_prop_key("ToggleButton", "checked").expect("ToggleButton.checked exists");
        assert_eq!(key, PROP_TOGGLEBUTTON_CHECKED);
        assert_eq!(ty, IrType::Bool);
    }

    #[test]
    fn resolve_prop_key_togglebutton_button_family_attrs() {
        assert_eq!(
            resolve_prop_key("ToggleButton", "text"),
            Some((PROP_BUTTON_LABEL, IrType::Str))
        );
        assert_eq!(
            resolve_prop_key("ToggleButton", "enabled"),
            Some((PROP_BUTTON_ENABLED, IrType::Bool))
        );
    }

    #[test]
    fn resolve_prop_key_text_font_is_i32() {
        let (key, ty) = resolve_prop_key("Text", "font").expect("Text.font exists");
        assert_eq!(key, PROP_TEXT_STYLE);
        assert_eq!(ty, IrType::I32);
    }

    fn conditional_slot(live_child: bool) -> DeclaredMemberSlot {
        DeclaredMemberSlot::Conditional(Rc::new(RefCell::new(ConditionalRuntimeState {
            live_child,
        })))
    }

    fn for_loop_slot(live_children: usize) -> DeclaredMemberSlot {
        DeclaredMemberSlot::ForLoop(Rc::new(RefCell::new(ForLoopRuntimeState { live_children })))
    }

    #[test]
    fn expansion_seam_counts_interleaved_widgets_conditionals_and_for_loops() {
        let toggled = Rc::new(RefCell::new(ConditionalRuntimeState { live_child: true }));
        let generated = Rc::new(RefCell::new(ForLoopRuntimeState { live_children: 3 }));
        let slots = vec![
            DeclaredMemberSlot::Widget,
            DeclaredMemberSlot::Widget,
            conditional_slot(false),
            DeclaredMemberSlot::Conditional(Rc::clone(&toggled)),
            DeclaredMemberSlot::ForLoop(Rc::clone(&generated)),
            DeclaredMemberSlot::Widget,
        ];

        assert_eq!(
            materialized_offset_for_declared_slot(2, &slots),
            2,
            "preceding widgets contribute one materialised child each"
        );
        assert_eq!(
            materialized_offset_for_declared_slot(3, &slots),
            2,
            "absent preceding conditional contributes no materialised child"
        );
        assert_eq!(
            materialized_offset_for_declared_slot(4, &slots),
            3,
            "present preceding conditional contributes one materialised child"
        );
        assert_eq!(
            materialized_offset_for_declared_slot(5, &slots),
            6,
            "preceding for-loop contributes its live cardinality"
        );
        assert_eq!(
            materialized_offset_for_declared_slot(6, &slots),
            7,
            "mixed widget / absent conditional / present conditional / for-loop prefix"
        );
        assert_eq!(
            total_materialized_children(&slots),
            7,
            "total materialised count is the sum of live slot cardinalities"
        );

        generated.borrow_mut().live_children = 0;
        assert_eq!(
            materialized_offset_for_declared_slot(5, &slots),
            3,
            "zero-cardinality for-loop contributes no materialised children"
        );
        assert_eq!(
            total_materialized_children(&slots),
            4,
            "total count recomputes after for-loop cardinality changes"
        );

        toggled.borrow_mut().live_child = false;
        assert_eq!(
            materialized_offset_for_declared_slot(4, &slots),
            2,
            "removing a preceding conditional shifts later materialised indices"
        );
    }

    #[test]
    fn expansion_seam_handles_boundaries_and_total_count() {
        let slots = vec![
            for_loop_slot(0),
            DeclaredMemberSlot::Widget,
            conditional_slot(true),
            for_loop_slot(2),
        ];

        assert_eq!(
            materialized_offset_for_declared_slot(0, &slots),
            0,
            "first declared slot starts at materialised offset zero"
        );
        assert_eq!(
            materialized_offset_for_declared_slot(1, &slots),
            0,
            "leading zero-cardinality slot does not move the offset"
        );
        assert_eq!(
            materialized_offset_for_declared_slot(4, &slots),
            4,
            "offset after the final declared slot equals total materialised children"
        );
        assert_eq!(total_materialized_children(&slots), 4);
    }

    #[test]
    fn tail_range_plan_derives_insert_remove_and_noop_cases() {
        assert_eq!(
            plan_tail_range_change(2, 5),
            TailRangePlan::Insert { start: 2, count: 3 },
            "tail growth inserts the new suffix"
        );
        assert_eq!(
            plan_tail_range_change(5, 2),
            TailRangePlan::Remove {
                tail_first_indices: vec![4, 3, 2],
            },
            "tail shrink removes from the old tail toward the retained prefix"
        );
        assert_eq!(
            plan_tail_range_change(3, 3),
            TailRangePlan::NoOp,
            "same-length reset is a no-op for structural range planning"
        );
        assert_eq!(
            plan_tail_range_change(0, 0),
            TailRangePlan::NoOp,
            "empty boundary is stable"
        );
        assert_eq!(
            plan_tail_range_change(0, 1),
            TailRangePlan::Insert { start: 0, count: 1 },
            "empty-to-one inserts at the beginning of the range"
        );
        assert_eq!(
            plan_tail_range_change(1, 0),
            TailRangePlan::Remove {
                tail_first_indices: vec![0],
            },
            "one-to-empty removes the only materialised child"
        );
    }

    #[test]
    fn expansion_seam_composes_for_slot_offset_with_tail_plan() {
        let slots = vec![
            DeclaredMemberSlot::Widget,
            for_loop_slot(2),
            DeclaredMemberSlot::Widget,
        ];
        let for_slot_base = materialized_offset_for_declared_slot(1, &slots);

        let insert_start = match plan_tail_range_change(2, 3) {
            TailRangePlan::Insert { start, count: 1 } => start,
            other => panic!("expected single tail insert, got {other:?}"),
        };

        assert_eq!(
            for_slot_base + insert_start,
            3,
            "absolute insertion index composes declared-slot base offset with for-local tail plan"
        );
    }

    #[test]
    fn static_condition_reducer_maps_bool_to_presence() {
        // Reducer logic pin only: after T5, production initial presence is
        // materialised by the same eager conditional Effect used for dynamic
        // toggles, not by this helper.
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
    fn collection_signal_registry_populates_all_collection_types() {
        let registry = build_signal_registry(&[
            IrState {
                name: "ints".into(),
                ty: IrStateType::Collection(IrType::I32),
                default: IrLiteral::List(vec![IrLiteral::Int(1), IrLiteral::Int(2)]),
            },
            IrState {
                name: "labels".into(),
                ty: IrStateType::Collection(IrType::Str),
                default: IrLiteral::List(vec![
                    IrLiteral::Str("a".into()),
                    IrLiteral::Str("b".into()),
                ]),
            },
            IrState {
                name: "flags".into(),
                ty: IrStateType::Collection(IrType::Bool),
                default: IrLiteral::List(vec![IrLiteral::Bool(true), IrLiteral::Bool(false)]),
            },
        ]);

        assert_eq!(registry.i32_lists["ints"].get_untracked(), vec![1, 2],);
        assert_eq!(
            registry.string_lists["labels"].get_untracked(),
            vec!["a".to_string(), "b".to_string()],
        );
        assert_eq!(
            registry.bool_lists["flags"].get_untracked(),
            vec![true, false],
        );
    }

    #[test]
    fn zstack_static_placements_follow_materialized_member_order() {
        // Reducer logic pin only: after T5, production ZStack placement is
        // accumulated through append_static_member's per-child insert path.
        fn text_with_align(h_align: IrAlignment, v_align: IrAlignment) -> IrChildSlot {
            zstack_slot(
                IrNode {
                    widget_type: "Text".into(),
                    props: vec![],
                    bindings: vec![],
                    handlers: vec![],
                    children: vec![],
                    kind_payload: None,
                },
                h_align,
                v_align,
            )
        }

        let mut registry = SignalRegistry::new();
        registry.bools.insert("open".into(), Signal::new(true));
        registry.bools.insert("closed".into(), Signal::new(false));
        let members = vec![
            IrMember::Widget(text_with_align(IrAlignment::Start, IrAlignment::Start)),
            IrMember::ControlFlow(ControlFlowNode::If {
                branches: vec![ControlFlowBranch {
                    condition: HandlerExpr::BoolPropRead {
                        path: "open".into(),
                    },
                    body: vec![IrMember::Widget(text_with_align(
                        IrAlignment::End,
                        IrAlignment::Stretch,
                    ))],
                }],
            }),
            IrMember::ControlFlow(ControlFlowNode::If {
                branches: vec![ControlFlowBranch {
                    condition: HandlerExpr::BoolPropRead {
                        path: "closed".into(),
                    },
                    body: vec![IrMember::Widget(text_with_align(
                        IrAlignment::Stretch,
                        IrAlignment::End,
                    ))],
                }],
            }),
            IrMember::Widget(text_with_align(IrAlignment::Center, IrAlignment::Center)),
        ];

        let placements = collect_static_zstack_child_placement_slots(&members, &registry).unwrap();
        assert_eq!(placements.len(), 3);
        assert_eq!(placements[0].h_align, Alignment::Leading);
        assert_eq!(placements[0].v_align, Alignment::Leading);
        assert_eq!(placements[1].h_align, Alignment::Trailing);
        assert_eq!(placements[1].v_align, Alignment::Stretch);
        assert_eq!(placements[2].h_align, Alignment::Center);
        assert_eq!(placements[2].v_align, Alignment::Center);
    }

    #[test]
    fn zstack_static_placement_reducer_expands_for_cardinality_after_t6() {
        let mut registry = SignalRegistry::new();
        registry
            .i32_lists
            .insert("xs".into(), Signal::new(vec![1, 2, 3]));
        let members = vec![IrMember::ControlFlow(ControlFlowNode::For {
            binder: "x".into(),
            index_binder: None,
            collection: HandlerExpr::ListPropRead {
                path: "xs".into(),
                elem: IrType::I32,
            },
            body: vec![IrMember::Widget(child_slot(IrNode {
                widget_type: "Text".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![],
                kind_payload: None,
            }))],
        })];

        let placements = collect_static_zstack_child_placement_slots(&members, &registry).unwrap();
        assert_eq!(placements.len(), 3);
        assert!(placements
            .iter()
            .all(|placement| placement.h_align == Alignment::Center
                && placement.v_align == Alignment::Center));
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
            IrMember::Widget(slot) => &slot.node,
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
        assert_eq!(c.states[0].ty, IrStateType::Scalar(IrType::I32));
        assert_eq!(c.states[0].default, IrLiteral::Int(0));
    }

    #[test]
    fn state_string_with_str_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state msg: string = \"hi\"\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].ty, IrStateType::Scalar(IrType::Str));
        assert_eq!(c.states[0].default, IrLiteral::Str("hi".into()));
    }

    #[test]
    fn collection_state_i32_list_default_parses() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state thumbs: i32[] = [1, 2, 3]\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].ty, IrStateType::Collection(IrType::I32));
        assert_eq!(
            c.states[0].default,
            IrLiteral::List(vec![
                IrLiteral::Int(1),
                IrLiteral::Int(2),
                IrLiteral::Int(3)
            ])
        );
    }

    #[test]
    fn collection_state_string_and_bool_list_defaults_parse() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state labels: string[] = [\"a\", \"b\"]\n\
             state flags: bool[] = [true, false]\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].ty, IrStateType::Collection(IrType::Str));
        assert_eq!(
            c.states[0].default,
            IrLiteral::List(vec![IrLiteral::Str("a".into()), IrLiteral::Str("b".into())])
        );
        assert_eq!(c.states[1].ty, IrStateType::Collection(IrType::Bool));
        assert_eq!(
            c.states[1].default,
            IrLiteral::List(vec![IrLiteral::Bool(true), IrLiteral::Bool(false)])
        );
    }

    #[test]
    fn collection_state_rejects_mismatched_list_element() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state thumbs: i32[] = [1, \"two\"]\n\
             node V {}\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("element must match `i32`")),
            "{err:?}"
        );
    }

    #[test]
    fn collection_state_rejects_nested_list_default() {
        let state = IrState {
            name: "nested".into(),
            ty: IrStateType::Collection(IrType::I32),
            default: IrLiteral::List(vec![IrLiteral::List(vec![IrLiteral::Int(1)])]),
        };
        let err = validate_state_default(&state).unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("cannot contain a nested list literal")),
            "{err:?}"
        );
    }

    #[test]
    fn scalar_state_rejects_list_default() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = [1]\n\
             node V {}\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("scalar state `count` cannot use a list literal default")),
            "{err:?}"
        );
    }

    #[test]
    fn scalar_state_rejects_type_mismatched_default() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = true\n\
             node V {}\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("default does not match declared type")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_parses_binders_collection_and_body() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state thumbs: i32[] = [1, 2]\n\
             node WrapPanel {\n\
               for thumb, i in thumbs {\n\
                 node Text { bind text = (interp \"#\" ((index-read i)) \":\" ((item-read thumb))) }\n\
               }\n\
             }\n}",
        );
        match &c.root.children[0] {
            IrMember::ControlFlow(ControlFlowNode::For {
                binder,
                index_binder,
                collection,
                body,
            }) => {
                assert_eq!(binder, "thumb");
                assert_eq!(index_binder.as_deref(), Some("i"));
                assert_eq!(
                    *collection,
                    HandlerExpr::ListPropRead {
                        path: "thumbs".into(),
                        elem: IrType::I32,
                    }
                );
                assert_eq!(body.len(), 1);
            }
            other => panic!("expected for member, got {other:?}"),
        }
    }

    #[test]
    fn for_member_rejects_scalar_collection_target() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node WrapPanel { for x in count { node Text {} } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("collection expression references scalar state `count`")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_rejects_undeclared_collection_target() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node WrapPanel { for x in missing { node Text {} } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("undeclared collection `missing`")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_rejects_binder_state_collision() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state x: i32 = 0\n\
             state xs: i32[] = []\n\
             node WrapPanel { for x in xs { node Text {} } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("binder `x` collides")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_rejects_same_binder_and_index() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node WrapPanel { for x, x in xs { node Text {} } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("binder and index binder must be distinct")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_rejects_index_state_collision() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state i: i32 = 0\n\
             state xs: i32[] = []\n\
             node WrapPanel { for x, i in xs { node Text {} } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("index binder `i` collides")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_rejects_multi_child_body() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node WrapPanel { for x in xs { node Text {} node Text {} } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("`for` body supports exactly one widget member")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_rejects_nested_control_flow_body() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node WrapPanel { for x in xs { if true { node Text {} } } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("nested control-flow")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_rejects_direct_disallowed_containers() {
        let cases = [
            (
                ";wasamo-ir v0\ncomponent C inherits W {\n\
                 state xs: i32[] = []\n\
                 node ScrollView { for x in xs { node Text {} } }\n}",
                "ScrollView",
            ),
            (
                ";wasamo-ir v0\ncomponent C inherits W {\n\
                 state xs: i32[] = []\n\
                 node Box { for x in xs { node Text {} } }\n}",
                "Box",
            ),
            (
                ";wasamo-ir v0\ncomponent C inherits W {\n\
                 state xs: i32[] = []\n\
                 node Grid { tracks columns = 1* tracks rows = 1* for x in xs { node Cell { node Text {} } } }\n}",
                "Grid",
            ),
        ];
        for (src, needle) in cases {
            let err = parse_ir(src).unwrap_err();
            assert!(
                matches!(err, IrLoadError::Validate(ref m) if m.contains(needle)),
                "{needle}: {err:?}"
            );
        }
    }

    #[test]
    fn for_member_rejects_component_body_surface_at_parse() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             for x in xs { node Text {} }\n\
             node Window {}\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Parse(ref m) if m.contains("unexpected token in component body")),
            "{err:?}"
        );
    }

    #[test]
    fn for_member_rejects_handler_and_nested_for_inside_template() {
        let handler = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node WrapPanel { for x in xs { node Button { on clicked { 1 } } } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(handler, IrLoadError::Validate(ref m) if m.contains("handlers inside a `for` body")),
            "{handler:?}"
        );

        let nested = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node WrapPanel { for x in xs { node VStack { for y in xs { node Text {} } } } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(nested, IrLoadError::Validate(ref m) if m.contains("nested `for`")),
            "{nested:?}"
        );
    }

    #[test]
    fn loop_local_reads_are_scoped_to_for_body() {
        let outside_item = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: string[] = []\n\
             node VStack { node Text { bind text = (item-read x) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(outside_item, IrLoadError::Validate(ref m) if m.contains("may be read only inside")),
            "{outside_item:?}"
        );

        let missing_index = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: string[] = []\n\
             node WrapPanel { for x in xs { node Text { bind text = (index-read i) } } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(missing_index, IrLoadError::Validate(ref m) if m.contains("may be read only inside")),
            "{missing_index:?}"
        );

        let bool_interp = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state flags: bool[] = []\n\
             node WrapPanel { for flag in flags { node Text { bind text = (interp \"v=\" ((item-read flag))) } } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(bool_interp, IrLoadError::Validate(ref m) if m.contains("bool loop binder")),
            "{bool_interp:?}"
        );

        let string_item_to_i32_target = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state labels: string[] = []\n\
             node WrapPanel { for label in labels { node Text { bind font = (item-read label) } } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(string_item_to_i32_target, IrLoadError::Validate(ref m) if m.contains("element type `string`, not `i32`")),
            "{string_item_to_i32_target:?}"
        );

        let index_read_to_bool_target = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state nums: i32[] = []\n\
             node WrapPanel { for n, i in nums { node Button { bind enabled = (index-read i) } } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(index_read_to_bool_target, IrLoadError::Validate(ref m) if m.contains("index binder cannot be used in a bool binding")),
            "{index_read_to_bool_target:?}"
        );
    }

    #[test]
    fn collection_assignment_drop_last_rhs_parses_and_validates() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state thumbs: i32[] = [1]\n\
             node Button { on clicked { (assign thumbs (list-drop-last thumbs)) } }\n}",
        );
        let handler = &c.root.handlers[0];
        assert_eq!(
            handler.expr,
            HandlerExpr::Assign {
                lhs: "thumbs".into(),
                rhs: Box::new(HandlerExpr::ListDropLast {
                    path: "thumbs".into(),
                    elem: IrType::I32,
                }),
            }
        );
    }

    #[test]
    fn collection_assignment_append_rhs_parses_and_validates() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state thumbs: i32[] = [1]\n\
             state next: i32 = 2\n\
             node Button { on clicked { (assign thumbs (list-append thumbs (prop-read next))) } }\n}",
        );
        let handler = &c.root.handlers[0];
        assert_eq!(
            handler.expr,
            HandlerExpr::Assign {
                lhs: "thumbs".into(),
                rhs: Box::new(HandlerExpr::ListAppend {
                    path: "thumbs".into(),
                    elem: IrType::I32,
                    value: Box::new(HandlerExpr::PropRead {
                        path: "next".into(),
                    }),
                }),
            }
        );
    }

    #[test]
    fn collection_assignment_append_literal_type_mismatch_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node Button { on clicked { (assign xs (list-append xs \"bad\")) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("does not match element type `i32`")),
            "{err:?}"
        );
    }

    #[test]
    fn collection_assignment_append_scalar_read_type_mismatch_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             state label: string = \"bad\"\n\
             node Button { on clicked { (assign xs (list-append xs (str-prop-read label))) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("appends `label` with type `string`, expected `i32`")),
            "{err:?}"
        );
    }

    #[test]
    fn append_collection_state_as_element_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             state ys: i32[] = []\n\
             node Button { on clicked { (assign xs (list-append xs (prop-read ys))) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("cannot append collection state")),
            "{err:?}"
        );
    }

    #[test]
    fn collection_assignment_list_literal_rhs_parses_and_validates() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state thumbs: i32[] = [1]\n\
             node Button { on clicked { (assign thumbs [2, 3]) } }\n}",
        );
        let handler = &c.root.handlers[0];
        assert_eq!(
            handler.expr,
            HandlerExpr::Assign {
                lhs: "thumbs".into(),
                rhs: Box::new(HandlerExpr::ListLit(vec![
                    IrLiteral::Int(2),
                    IrLiteral::Int(3)
                ])),
            }
        );
    }

    #[test]
    fn collection_state_rejects_scalar_default() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = 1\n\
             node V {}\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("default must be a list literal")),
            "{err:?}"
        );
    }

    #[test]
    fn collection_compound_assignment_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node Button { on clicked { (compound-assign += xs 1) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("cannot use compound assignment")),
            "{err:?}"
        );
    }

    #[test]
    fn collection_edit_outside_assignment_rhs_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node Button { on clicked { (list-drop-last xs) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("collection edit expressions are valid only")),
            "{err:?}"
        );
    }

    #[test]
    fn collection_assignment_wrong_rhs_kind_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             state n: i32 = 1\n\
             node Button { on clicked { (assign xs (prop-read n)) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("requires list-append, list-drop-last, or list literal RHS")),
            "{err:?}"
        );
    }

    #[test]
    fn collection_assignment_wrong_receiver_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             state ys: i32[] = []\n\
             node Button { on clicked { (assign xs (list-drop-last ys)) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("RHS must use `xs` as its receiver")),
            "{err:?}"
        );
    }

    #[test]
    fn scalar_assignment_list_rhs_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node Button { on clicked { (assign count [1]) } }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("list literals are valid only")),
            "{err:?}"
        );
    }

    #[test]
    fn scalar_read_of_collection_state_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node Text { bind text = (prop-read xs) }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("scalar expression references collection state")),
            "{err:?}"
        );
    }

    #[test]
    fn bare_collection_read_outside_for_header_rejected() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node Text { bind text = (list-prop-read xs) }\n}",
        )
        .unwrap_err();
        assert!(
            matches!(err, IrLoadError::Validate(ref m) if m.contains("collection reads are valid only")),
            "{err:?}"
        );
    }

    #[test]
    fn prop_int_and_ident_and_str() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V {\n\
               prop spacing = 12\n\
               prop style = system\n\
               prop font = \"Hi\"\n\
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
                name: "style".into(),
                value: IrLiteral::Ident("system".into())
            }
        );
        assert_eq!(
            props[2],
            IrProp {
                name: "font".into(),
                value: IrLiteral::Str("Hi".into())
            }
        );
    }

    #[test]
    fn host_prop_parses_on_component_surface() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop title = \"Hi\"\n\
             host prop backdrop = mica\n\
             node V {}\n}",
        );
        assert!(c.root.props.is_empty());
        assert_eq!(c.host_props.len(), 2);
        assert_eq!(c.host_props[0].name, "title");
        assert_eq!(c.host_props[0].value, IrLiteral::Str("Hi".into()));
        assert_eq!(resolve_static_window_title(&c, "Wasamo"), "Hi");
    }

    #[test]
    fn host_attribute_catalog_mirrors_wasamoc() {
        assert_eq!(HOST_STATIC_ATTRS, wasamoc::check::HOST_STATIC_ATTRS);
    }

    #[test]
    fn static_window_title_resolves_string_or_default() {
        let absent = parse_ok(";wasamo-ir v0\ncomponent C inherits W {\nnode V {}\n}");
        assert_eq!(resolve_static_window_title(&absent, "Wasamo"), "Wasamo");

        let empty = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop title = \"\"\n\
             node V {}\n}",
        );
        assert_eq!(resolve_static_window_title(&empty, "Wasamo"), "Wasamo");

        let custom = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop title = \"Gallery\"\n\
             node V {}\n}",
        );
        assert_eq!(resolve_static_window_title(&custom, "Wasamo"), "Gallery");
    }

    #[test]
    fn static_window_title_rejects_non_string_host_prop() {
        let err = parse_ir(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop title = 3\n\
             node V {}\n}",
        )
        .unwrap_err();
        assert!(matches!(err, IrLoadError::Validate(_)));
        assert!(
            err.to_string()
                .contains("host `title` prop must be a string"),
            "{err}"
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
                        IrMember::Widget(slot) => slot.node.widget_type.as_str(),
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
        assert_eq!(c.states[0].ty, IrStateType::Scalar(IrType::Bool));
        assert_eq!(c.states[0].default, IrLiteral::Bool(false));
    }

    #[test]
    fn state_bool_with_true_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].ty, IrStateType::Scalar(IrType::Bool));
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
            host_props: vec![IrProp {
                name: "title".into(),
                value: IrLiteral::Str("Counter".into()),
            }],
            host_bindings: vec![],
            states: vec![IrState {
                name: "count".into(),
                ty: IrStateType::Scalar(IrType::I32),
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
                    IrMember::Widget(child_slot(IrNode {
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
                    })),
                    IrMember::Widget(child_slot(IrNode {
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
                            arg: None,
                            expr: HandlerExpr::CompoundAssign {
                                op: CompoundOp::Add,
                                lhs: "count".into(),
                                rhs: Box::new(HandlerExpr::IntLit(1)),
                            },
                        }],
                        children: vec![],
                        kind_payload: None,
                    })),
                ],
                kind_payload: None,
            },
        };

        let text = render(&original);
        let parsed = parse_ok(&text);
        assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");
    }

    #[test]
    fn grid_slot_emit_then_parse_preserves_payload_values() {
        let original = IrComponent {
            name: "GridRoundTrip".into(),
            base: "Window".into(),
            host_props: vec![],
            host_bindings: vec![],
            states: vec![],
            root: IrNode {
                widget_type: "Grid".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![IrMember::Widget(IrChildSlot {
                    node: IrNode {
                        widget_type: "Text".into(),
                        props: vec![IrProp {
                            name: "text".into(),
                            value: IrLiteral::Str("cell".into()),
                        }],
                        bindings: vec![],
                        handlers: vec![],
                        children: vec![],
                        kind_payload: None,
                    },
                    slot_data: Some(IrSlotData::Grid {
                        row: 1,
                        column: 2,
                        row_span: 1,
                        column_span: 2,
                        h_align: IrAlignment::Center,
                        v_align: IrAlignment::End,
                    }),
                })],
                kind_payload: Some(KindPayload::Grid {
                    columns: vec![
                        TrackSize::Fixed(120),
                        TrackSize::Star(1),
                        TrackSize::Star(2),
                        TrackSize::Star(1),
                    ],
                    rows: vec![TrackSize::Fixed(40), TrackSize::Star(1)],
                }),
            },
        };

        let text = render(&original);
        assert!(
            text.contains(
                "placement grid { row: 1, column: 2, row-span: 1, column-span: 2, h-align: center, v-align: end }"
            ),
            "IR text did not contain the non-default Grid placement:\n{text}"
        );
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
            let ty = match &s.ty {
                IrStateType::Scalar(IrType::I32) => "i32",
                IrStateType::Scalar(IrType::Str) => "string",
                IrStateType::Scalar(IrType::Bool) => "bool",
                IrStateType::Collection(IrType::I32) => "i32[]",
                IrStateType::Collection(IrType::Str) => "string[]",
                IrStateType::Collection(IrType::Bool) => "bool[]",
            };
            out.push_str(&format!(
                "    state {}: {} = {}\n",
                s.name,
                ty,
                render_lit(&s.default)
            ));
        }
        for p in &c.host_props {
            out.push_str(&format!(
                "    host prop {} = {}\n",
                p.name,
                render_lit(&p.value)
            ));
        }
        for b in &c.host_bindings {
            out.push_str(&format!(
                "    host bind {} = {}\n",
                b.prop_name,
                render_expr(&b.expr)
            ));
        }
        if !c.states.is_empty() || !c.host_props.is_empty() || !c.host_bindings.is_empty() {
            out.push('\n');
        }
        render_node(&mut out, &c.root, 1);
        out.push_str("}\n");
        out
    }

    fn render_node(out: &mut String, n: &IrNode, depth: usize) {
        let i = "    ".repeat(depth);
        out.push_str(&format!("{i}node {} {{\n", n.widget_type));
        if let Some(KindPayload::Grid { columns, rows }) = &n.kind_payload {
            out.push_str(&format!(
                "{i}    tracks columns = {}\n",
                render_tracks(columns)
            ));
            out.push_str(&format!("{i}    tracks rows = {}\n", render_tracks(rows)));
        }
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
            match &h.arg {
                Some(arg) => out.push_str(&format!("{i}    on {}(\"{}\") {{\n", h.signal, arg)),
                None => out.push_str(&format!("{i}    on {} {{\n", h.signal)),
            }
            out.push_str(&format!("{i}        {}\n", render_expr(&h.expr)));
            out.push_str(&format!("{i}    }}\n"));
        }
        for child in &n.children {
            match child {
                IrMember::Widget(slot) => render_child_slot(out, slot, depth + 1),
                IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
                    let i = "  ".repeat(depth + 1);
                    for branch in branches {
                        out.push_str(&format!("{}if {} {{\n", i, render_expr(&branch.condition)));
                        for body_member in &branch.body {
                            if let IrMember::Widget(slot) = body_member {
                                render_child_slot(out, slot, depth + 2);
                            }
                        }
                        out.push_str(&format!("{}}}\n", i));
                    }
                }
                IrMember::ControlFlow(ControlFlowNode::For {
                    binder,
                    index_binder,
                    collection,
                    body,
                }) => {
                    let i = "  ".repeat(depth + 1);
                    let collection_name = match collection {
                        HandlerExpr::ListPropRead { path, .. } => path.as_str(),
                        other => unreachable!("For.collection must be ListPropRead, got {other:?}"),
                    };
                    match index_binder {
                        Some(index) => out.push_str(&format!(
                            "{i}for {binder}, {index} in {collection_name} {{\n"
                        )),
                        None => out.push_str(&format!("{i}for {binder} in {collection_name} {{\n")),
                    }
                    for body_member in body {
                        if let IrMember::Widget(slot) = body_member {
                            render_child_slot(out, slot, depth + 2);
                        }
                    }
                    out.push_str(&format!("{}}}\n", i));
                }
            }
        }
        out.push_str(&format!("{i}}}\n"));
    }

    fn render_child_slot(out: &mut String, slot: &IrChildSlot, depth: usize) {
        let i = "    ".repeat(depth);
        out.push_str(&format!("{i}child {{\n"));
        if let Some(slot_data) = &slot.slot_data {
            render_slot_data(out, slot_data, depth + 1);
        }
        render_node(out, &slot.node, depth + 1);
        out.push_str(&format!("{i}}}\n"));
    }

    fn render_slot_data(out: &mut String, slot_data: &IrSlotData, depth: usize) {
        let i = "    ".repeat(depth);
        match slot_data {
            IrSlotData::Grid {
                row,
                column,
                row_span,
                column_span,
                h_align,
                v_align,
            } => out.push_str(&format!(
                "{i}placement grid {{ row: {row}, column: {column}, row-span: {row_span}, column-span: {column_span}, h-align: {}, v-align: {} }}\n",
                render_alignment(*h_align),
                render_alignment(*v_align)
            )),
            IrSlotData::ZStack { h_align, v_align } => out.push_str(&format!(
                "{i}placement zstack {{ h-align: {}, v-align: {} }}\n",
                render_alignment(*h_align),
                render_alignment(*v_align)
            )),
        }
    }

    fn render_tracks(tracks: &[TrackSize]) -> String {
        tracks
            .iter()
            .map(|track| match track {
                TrackSize::Fixed(px) => px.to_string(),
                TrackSize::Star(weight) => format!("{weight}*"),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn render_alignment(alignment: IrAlignment) -> &'static str {
        match alignment {
            IrAlignment::Start => "start",
            IrAlignment::Center => "center",
            IrAlignment::End => "end",
            IrAlignment::Stretch => "stretch",
        }
    }

    fn render_lit(l: &IrLiteral) -> String {
        match l {
            IrLiteral::Int(n) => n.to_string(),
            IrLiteral::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            IrLiteral::Ident(id) => id.clone(),
            IrLiteral::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
            IrLiteral::List(items) => {
                let inner: Vec<String> = items.iter().map(render_lit).collect();
                format!("[{}]", inner.join(", "))
            }
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
            HandlerExpr::ListPropRead { path, .. } => format!("(list-prop-read {})", path),
            HandlerExpr::ItemRead { binder } => format!("(item-read {})", binder),
            HandlerExpr::IndexRead { binder } => format!("(index-read {})", binder),
            HandlerExpr::ListAppend { path, value, .. } => {
                format!("(list-append {} {})", path, render_expr(value))
            }
            HandlerExpr::ListDropLast { path, .. } => format!("(list-drop-last {})", path),
            HandlerExpr::ListLit(items) => {
                let inner: Vec<String> = items.iter().map(render_lit).collect();
                format!("[{}]", inner.join(", "))
            }
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

    // ── M4-Phase 2 T8: layout-childless widget child rejection (CF-1,
    // owner disposition 2026-08-07; widened from Button/ToggleButton to
    // all four `wasamo_ir::LAYOUT_CHILDLESS_WIDGET_KINDS` 2026-08-08)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn validate_rejects_button_with_widget_child() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Button { node Text {} }\n\
             }",
            "`Button` node accepts no children, got 1 (layout arranges it as a single rectangle",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_with_widget_child() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ToggleButton { node Text {} }\n\
             }",
            "`ToggleButton` node accepts no children, got 1 (layout arranges it as a single rectangle",
        );
    }

    #[test]
    fn validate_rejects_text_with_widget_child() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Text { node Text {} }\n\
             }",
            "`Text` node accepts no children, got 1 (layout arranges it as a single rectangle",
        );
    }

    #[test]
    fn validate_rejects_rectangle_with_widget_child() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Rectangle { node Text {} }\n\
             }",
            "`Rectangle` node accepts no children, got 1 (layout arranges it as a single rectangle",
        );
    }

    #[test]
    fn childless_button_is_valid() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Button { prop text = \"hi\" }\n}",
        );
        assert!(c.root.children.is_empty());
    }

    #[test]
    fn vstack_with_widget_child_is_valid() {
        // Control: a container kind is untouched by the layout-childless
        // rule — `VStack` is not in `wasamo_ir::LAYOUT_CHILDLESS_WIDGET_KINDS`,
        // so a widget child must still parse and validate.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { node Text {} }\n}",
        );
        assert_eq!(c.root.children.len(), 1);
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

    fn assert_validate_err_not_contains(src: &str, needle: &str) {
        match parse_err(src) {
            IrLoadError::Validate(msg) => assert!(
                !msg.contains(needle),
                "validate message `{msg}` unexpectedly contained `{needle}`"
            ),
            other => panic!("expected Validate error, got {other:?}"),
        }
    }

    fn assert_parse_err(src: &str, needle: &str) {
        match parse_err(src) {
            IrLoadError::Parse(msg) => {
                assert!(
                    msg.contains(needle),
                    "parse message `{msg}` did not contain `{needle}`"
                )
            }
            other => panic!("expected Parse error, got {other:?}"),
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
               child { placement grid { row: 0, column: 0 } node Text {} }\n\
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
               child { placement grid { row: 0, column: 0 } node Text {} }\n\
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

    fn grid_child(placement: &str) -> String {
        format!("child {{ placement grid {{ {placement} }} node Text {{}} }}")
    }

    #[test]
    fn child_slot_missing_node_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src("child { placement grid { row: 0, column: 0 } }"),
            "child slot missing `node` block",
        );
    }

    #[test]
    fn child_slot_duplicate_node_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src("child { node Text {} node Text {} }"),
            "duplicate `node` block in child slot",
        );
    }

    #[test]
    fn child_slot_duplicate_placement_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src(
                "child { placement grid { row: 0, column: 0 } placement grid { row: 0, column: 0 } node Text {} }",
            ),
            "duplicate `placement` block in child slot",
        );
    }

    #[test]
    fn child_slot_unknown_placement_kind_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src("child { placement overlay {} node Text {} }"),
            "unknown placement kind `overlay`",
        );
    }

    #[test]
    fn child_slot_unexpected_token_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src("child { prop text = \"x\" node Text {} }"),
            "unexpected token in child slot",
        );
    }

    #[test]
    fn grid_slot_unknown_key_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src(&grid_child("row: 0, column: 0, layer: 1")),
            "unknown grid placement key `layer`",
        );
    }

    #[test]
    fn grid_slot_duplicate_key_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src(&grid_child("row: 0, row: 1, column: 0")),
            "duplicate grid placement key `row`",
        );
    }

    #[test]
    fn grid_positive_control_validates() {
        // Fixed + weighted-star tracks, a spanning slot and three
        // single-cell slots — all placements distinct, all in range.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 180 1* 2*\n\
               tracks rows = 1* 1*\n\
               child { placement grid { row: 0, column: 0, column-span: 3 } node Text {} }\n\
               child { placement grid { row: 1, column: 0 } node Text {} }\n\
               child { placement grid { row: 1, column: 1, h-align: center } node Text {} }\n\
               child { placement grid { row: 1, column: 2 } node Text {} }\n\
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
               child { placement grid { row: 0, column: 0 } node Text {} }\n\
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
               child { placement grid { row: 0, column: 0 } node Text {} }\n\
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
               child { placement grid { row: 0, column: 0 } node Text {} }\n\
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
               child { placement grid { row: 0, column: 0 } node Text {} }\n\
             }\n}",
            "star weight must be in [1, 1024]",
        );
    }

    // ── placement value range ───────────────────────────────────────────

    #[test]
    fn grid_cell_column_out_of_range_rejected() {
        assert_validate_err(
            &valid_grid_src(&grid_child("row: 0, column: 2")),
            "`Grid` child placement `column` 2 is out of range [0, 2)",
        );
    }

    #[test]
    fn grid_cell_row_out_of_range_rejected() {
        assert_validate_err(
            &valid_grid_src(&grid_child("row: 5, column: 0")),
            "`Grid` child placement `row` 5 is out of range [0, 2)",
        );
    }

    // ── span value range ────────────────────────────────────────────────

    #[test]
    fn grid_cell_zero_span_rejected() {
        assert_parse_err(
            &valid_grid_src(
                "child { placement grid { row: 0, column: 0, column-span: 0 } node Text {} }",
            ),
            "grid.column-span",
        );
    }

    #[test]
    fn grid_slot_negative_row_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src(
                "child { placement grid { row: -1, column: 0, column-span: 1 } node Text {} }",
            ),
            "grid.row",
        );
    }

    #[test]
    fn grid_cell_span_exceeds_grid_rejected() {
        assert_validate_err(
            &valid_grid_src(
                "child { placement grid { row: 0, column: 1, column-span: 2 } node Text {} }",
            ),
            "column span exceeds the grid",
        );
    }

    // ── stale Cell textual IR ───────────────────────────────────────────

    #[test]
    fn grid_legacy_cell_zero_content_children_rejected_as_stale_ir() {
        assert_validate_err(
            &valid_grid_src("node Cell { prop row = 0 prop column = 0 }"),
            "legacy-placement-ir-form",
        );
    }

    #[test]
    fn grid_legacy_cell_two_content_children_rejected_as_stale_ir() {
        assert_validate_err(
            &valid_grid_src("node Cell { prop row = 0 prop column = 0 node Text {} node Text {} }"),
            "legacy-placement-ir-form",
        );
    }

    // ── same-cell / overlapping-rectangle conflict ──────────────────────

    #[test]
    fn grid_same_cell_conflict_rejected() {
        assert_validate_err(
            &valid_grid_src(
                "child { placement grid { row: 0, column: 0 } node Text {} }\n\
                 child { placement grid { row: 0, column: 0 } node Text {} }",
            ),
            "overlaps an earlier Grid child rectangle",
        );
    }

    #[test]
    fn grid_overlapping_span_conflict_rejected() {
        // A 1×2 spanning Cell at (0,0)-(0,1) overlaps a single Cell at
        // (0,1).
        assert_validate_err(
            &valid_grid_src(
                "child { placement grid { row: 0, column: 0, column-span: 2 } node Text {} }\n\
                 child { placement grid { row: 0, column: 1 } node Text {} }",
            ),
            "overlaps an earlier Grid child rectangle",
        );
    }

    #[test]
    fn grid_multi_cell_omitted_placement_collides_at_origin() {
        // Two Grid child slots omitting explicit placement both default
        // to (0, 0) and are caught by the overlap gate.
        assert_validate_err(
            &valid_grid_src(
                "child { node Text {} }\n\
                 child { node Text {} }",
            ),
            "overlaps an earlier Grid child rectangle",
        );
    }

    // ── alignment vocabulary ────────────────────────────────────────────

    #[test]
    fn grid_cell_unknown_alignment_rejected() {
        assert_parse_err(
            &valid_grid_src(
                "child { placement grid { row: 0, column: 0, h-align: middle } node Text {} }",
            ),
            "grid.h-align",
        );
    }

    #[test]
    fn grid_slot_non_keyword_alignment_rejected_at_parse() {
        assert_parse_err(
            &valid_grid_src(
                "child { placement grid { row: 0, column: 0, h-align: 5 } node Text {} }",
            ),
            "grid.h-align",
        );
    }

    // ── legacy Cell outside Grid ────────────────────────────────────────

    #[test]
    fn grid_direct_child_without_placement_defaults_to_origin() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 1*\n\
               tracks rows = 1*\n\
               node Text {}\n\
             }\n}",
        );
        assert_eq!(c.root.children.len(), 1);
    }

    #[test]
    fn grid_rejects_zstack_slot_data() {
        assert_validate_err(
            &valid_grid_src(
                "child { placement zstack { h-align: center, v-align: center } node Text {} }",
            ),
            "`placement zstack` is not valid on a Grid child",
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
            host_props: vec![],
            host_bindings: vec![],
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
            children: vec![IrMember::Widget(child_slot(IrNode {
                widget_type: "Text".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![],
                kind_payload: None,
            }))],
            kind_payload: Some(KindPayload::Grid {
                columns: vec![TrackSize::Star(1)],
                rows: vec![TrackSize::Star(1)],
            }),
        };
        let comp = IrComponent {
            name: "C".into(),
            base: "W".into(),
            host_props: vec![],
            host_bindings: vec![],
            states: vec![],
            root: IrNode {
                widget_type: "Grid".into(),
                props: vec![],
                bindings: vec![],
                handlers: vec![],
                children: vec![IrMember::Widget(child_slot(cell))],
                kind_payload: Some(KindPayload::Grid {
                    columns: vec![TrackSize::Star(1)],
                    rows: vec![TrackSize::Star(1)],
                }),
            },
        };
        match validate(&comp) {
            Err(IrLoadError::Validate(msg)) => {
                assert!(msg.contains("legacy-placement-ir-form"), "got: {msg}")
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
               child { placement zstack { h-align: end, v-align: start } node Text { prop text = \"caption\" } }\n\
             }\n}",
        );
        assert_eq!(c.root.widget_type, "ZStack");
        assert_eq!(c.root.children.len(), 2);
    }

    #[test]
    fn zstack_slot_unknown_key_rejected_at_parse() {
        assert_parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { child { placement zstack { h-align: center, layer: 1 } node Text {} } }\n\
             }",
            "unknown zstack placement key `layer`",
        );
    }

    // ── M3-Phase 8 T4: ToggleButton runtime catalog defense ───────────

    #[test]
    fn togglebutton_checked_literal_validates() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ToggleButton { prop text = \"All\" prop checked = true }\n\
             }",
        );
        validate(&c).expect("ToggleButton checked literal must validate");
    }

    #[test]
    fn togglebutton_checked_binding_validates() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state selected: bool = true\n\
             node ToggleButton { bind checked = (bool-prop-read selected) }\n\
             }",
        );
        validate(&c).expect("ToggleButton checked bool binding must validate");
    }

    #[test]
    fn validate_rejects_checked_on_button_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Button { prop checked = true }\n\
             }",
            "`checked` is only valid on ToggleButton",
        );
    }

    #[test]
    fn validate_rejects_checked_on_text_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Text { prop checked = true }\n\
             }",
            "`checked` is only valid on ToggleButton",
        );
    }

    #[test]
    fn validate_rejects_checked_binding_on_button_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state selected: bool = true\n\
             node Button { bind checked = (bool-prop-read selected) }\n\
             }",
            "`checked` binding is only valid on ToggleButton",
        );
    }

    #[test]
    fn validate_rejects_checked_binding_on_text_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state selected: bool = true\n\
             node Text { bind checked = (bool-prop-read selected) }\n\
             }",
            "`checked` binding is only valid on ToggleButton",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_unknown_attr_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ToggleButton { prop selected = true }\n\
             }",
            "unknown ToggleButton attribute `selected`",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_unknown_binding_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state selected: bool = true\n\
             node ToggleButton { bind selected = (bool-prop-read selected) }\n\
             }",
            "unknown ToggleButton binding `selected`",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_style_binding_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state style_id: i32 = 1\n\
             node ToggleButton { bind style = (prop-read style_id) }\n\
             }",
            "ToggleButton.style is not bindable",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_text_non_str_literal_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ToggleButton { prop text = true }\n\
             }",
            "ToggleButton.text must be a `string` literal",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_style_non_ident_literal_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ToggleButton { prop style = 1 }\n\
             }",
            "ToggleButton.style must be a keyword identifier",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_enabled_non_bool_literal_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ToggleButton { prop enabled = 1 }\n\
             }",
            "ToggleButton.enabled must be a `bool` literal",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_checked_non_bool_literal_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ToggleButton { prop checked = 1 }\n\
             }",
            "ToggleButton.checked must be a `bool` literal",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_checked_non_bool_binding_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state index: i32 = 0\n\
             node ToggleButton { bind checked = (prop-read index) }\n\
             }",
            "binding `ToggleButton.checked` must resolve to `bool`",
        );
    }

    #[test]
    fn validate_rejects_togglebutton_checked_wrong_read_tag_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state selected: bool = true\n\
             node ToggleButton { bind checked = (str-prop-read selected) }\n\
             }",
            "binding `ToggleButton.checked` must resolve to `bool`",
        );
    }

    #[test]
    fn validate_accepts_togglebutton_checked_loop_item_binding_runtime_ir() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state flags: bool[] = [true, false]\n\
             node WrapPanel { for flag in flags { node ToggleButton { bind checked = (item-read flag) } } }\n\
             }",
        );
        validate(&c).expect("ToggleButton.checked may bind a bool loop item");
    }

    #[test]
    fn validate_accepts_togglebutton_text_loop_item_binding_runtime_ir() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state labels: string[] = [\"All\", \"Albums\"]\n\
             node WrapPanel { for label in labels { node ToggleButton { bind text = (item-read label) } } }\n\
             }",
        );
        validate(&c).expect("ToggleButton.text may bind a string loop item");
    }

    #[test]
    fn validate_accepts_togglebutton_text_loop_item_interpolation_runtime_ir() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state labels: string[] = [\"All\", \"Albums\"]\n\
             node WrapPanel { for label in labels { node ToggleButton { bind text = (interp \"Tab \" ((item-read label))) } } }\n\
             }",
        );
        validate(&c).expect("ToggleButton.text interpolation may read a string loop item");
    }

    #[test]
    fn validate_rejects_togglebutton_checked_loop_index_binding_runtime_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state flags: bool[] = [true, false]\n\
             node WrapPanel { for flag, i in flags { node ToggleButton { bind checked = (index-read i) } } }\n\
             }",
            "loop index binder cannot be used in a bool binding",
        );
    }

    #[test]
    fn zstack_slot_duplicate_key_rejected_at_parse() {
        assert_parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { child { placement zstack { h-align: center, h-align: end } node Text {} } }\n\
             }",
            "duplicate zstack placement key `h-align`",
        );
    }

    #[test]
    fn zstack_rejects_grid_slot_data() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { child { placement grid { row: 0, column: 0 } node Text {} } }\n\
             }",
            "`placement grid` is not valid on a ZStack child",
        );
    }

    #[test]
    fn zstack_legacy_bare_child_placement_rejected_as_stale_ir() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { node Text { prop h-align = end } }\n\
             }",
            "legacy-placement-ir-form",
        );
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
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node Box { if true { for x in xs { node Text {} } } }\n\
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
            "legacy-placement-ir-form",
        );
    }

    // T4 review follow-up: single-child container counts must include a
    // conditional sibling (it materialises at most one child). Box's
    // at-most-one and ScrollView's exactly-one gates previously counted
    // widget children only and let `Box { Content  if c }` /
    // `ScrollView { Content  if c }` slip through (see log.md T4 audit +
    // DD-M3-P6-007).
    #[test]
    fn validate_rejects_box_with_widget_and_conditional_sibling() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { node Text {} if true { node Text {} } }\n\
             }",
            "`Box` node accepts at most one child",
        );
    }

    #[test]
    fn validate_accepts_box_with_conditional_only_child() {
        // A lone conditional is one potential child (≤ 1) — valid.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state c: bool = true\n\
             node Box { if (bool-prop-read c) { node Text {} } }\n\
             }",
        );
        assert_eq!(c.root.widget_type, "Box");
        assert_eq!(c.root.children.len(), 1);
    }

    #[test]
    fn validate_rejects_box_with_multiple_conditional_siblings() {
        // Two conditionals = two potential children: the shortest reject
        // proving `node.children.len()` counts conditionals, not just a
        // widget+conditional pair.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Box { if true { node Text {} } if true { node Button {} } }\n\
             }",
            "`Box` node accepts at most one child",
        );
    }

    #[test]
    fn validate_rejects_scrollview_with_conditional_member() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView { node Box {} if true { node Text {} } }\n\
             }",
            "a conditional member is not valid directly in ScrollView",
        );
        assert_validate_err_not_contains(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView { node Box {} if true { node Text {} } }\n\
             }",
            "DD-M3-P6-007",
        );
    }

    #[test]
    fn validate_rejects_scrollview_with_conditional_only_member() {
        // DD-M3-P6-007 centre case: conditional-only ScrollView content is
        // the interim (a) rejection — pins the value a (b) relaxation flips.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView { if true { node Text {} } }\n\
             }",
            "a conditional member is not valid directly in ScrollView",
        );
        assert_validate_err_not_contains(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ScrollView { if true { node Text {} } }\n\
             }",
            "DD-M3-P6-007",
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
    fn root_zstack_accepts_host_props_on_component_surface() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop title = \"Gallery\"\n\
             host prop backdrop = mica\n\
             host prop theme = system\n\
             node ZStack { node Text {} }\n\
             }",
        );
        validate(&c).expect("host props are separate from the ZStack content root");
        assert_eq!(resolve_static_window_title(&c, "Wasamo"), "Gallery");
    }

    #[test]
    fn root_zstack_still_rejects_widget_attribute() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop title = \"Gallery\"\n\
             node ZStack { prop spacing = 8 node Text {} }\n}",
            "`ZStack` accepts no Phase-6 attributes; found `spacing`",
        );
    }

    #[test]
    fn zstack_child_zstack_accepts_placement_props() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { child { placement zstack { h-align: stretch, v-align: stretch } node ZStack { node Text {} } } }\n\
             }",
        );
        validate(&c).expect("ZStack direct-child placement applies even when the child is ZStack");
    }

    // ── DD-M3-P6-008 A2a canonical host surface ─────────────────────────
    // Host attributes now live on `IrComponent.host_props` / `host_bindings`;
    // the content root is pure widget content again. The runtime remains a
    // defensive textual-IR reader, so it validates the host catalog and
    // rejects old root-squatted host attributes rather than silently accepting
    // both shapes.

    #[test]
    fn nested_zstack_rejects_component_window_prop() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { node ZStack { prop title = \"x\" node Text {} } }\n}",
            "`ZStack` accepts no Phase-6 attributes; found `title`",
        );
    }

    #[test]
    fn host_surface_rejects_unknown_host_prop() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop foo = bar\n\
             node ZStack { node Text {} }\n}",
            "unknown host attribute `foo`",
        );
    }

    #[test]
    fn host_surface_rejects_typed_literal_backdrop() {
        // Defensive-reader mirror of `wasamoc check`: `backdrop` / `theme`
        // take a keyword identifier (e.g. `mica` / `system`), so a typed
        // scalar literal in hand-crafted textual IR must be rejected at the
        // runtime catalog too — not left as a direct-textual-IR hole.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop backdrop = 3\n\
             node ZStack { node Text {} }\n}",
            "host `backdrop` prop must be a keyword identifier",
        );
    }

    #[test]
    fn host_surface_rejects_typed_literal_theme() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop theme = 3\n\
             node ZStack { node Text {} }\n}",
            "host `theme` prop must be a keyword identifier",
        );
    }

    #[test]
    fn host_surface_accepts_keyword_backdrop_and_theme() {
        // Positive control: the keyword-identifier forms the compiler emits
        // (`mica` / `system`) validate, so the typed-literal rejection above
        // is not blanket-rejecting `backdrop` / `theme`.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             host prop backdrop = mica\n\
             host prop theme = system\n\
             node ZStack { node Text {} }\n}",
        );
        validate(&c).expect("keyword-identifier backdrop/theme must validate");
    }

    #[test]
    fn root_zstack_rejects_placement_prop() {
        // A placement prop has no meaning on a root widget (no parent
        // placement context); the root ZStack rejects `h-align`.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { prop h-align = stretch node Text {} }\n}",
            "`ZStack` accepts no Phase-6 attributes; found `h-align`",
        );
    }

    #[test]
    fn host_surface_rejects_host_binding() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state s: string = \"x\"\n\
             host bind title = (str-prop-read s)\n\
             node ZStack { node Text {} }\n}",
            "host attribute `title` is not bindable in M3-Phase 6",
        );
    }

    #[test]
    fn old_root_squatted_host_prop_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { prop title = \"Gallery\" node Text {} }\n}",
            "host attribute `title` must live on `host_props`",
        );
    }

    #[test]
    fn old_root_squatted_host_binding_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state s: string = \"x\"\n\
             node ZStack { bind title = (str-prop-read s) node Text {} }\n}",
            "host attribute `title` must live on `host_bindings`",
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
    fn zstack_clicked_handler_validates() {
        // T8: the Phase-6 ZStack gate no longer carries a per-kind
        // handler rejection arm (it used to reject every signal name but
        // `dismiss`, asymmetrically with the checker side, which never
        // gated ZStack handlers). `clicked` is admitted on any widget
        // per dsl_spec §4.19, so a ZStack carrying an inline `on clicked`
        // handler now loads successfully.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node ZStack { on clicked { (assign ready false) } node Text {} }\n}",
        );
        assert_eq!(c.root.widget_type, "ZStack");
        assert_eq!(c.root.handlers.len(), 1);
        assert_eq!(c.root.handlers[0].signal, "clicked");
    }

    #[test]
    fn zstack_child_unknown_alignment_rejected_at_validate() {
        assert_parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { child { placement zstack { h-align: middle } node Text {} } }\n}",
            "zstack.h-align",
        );
    }

    #[test]
    fn placement_prop_outside_zstack_child_or_grid_cell_rejected_at_validate() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { child { placement zstack { h-align: center } node Text {} } }\n}",
            "placement data is valid only on Grid or ZStack child slots",
        );
    }

    #[test]
    fn validate_rejects_zstack_with_kind_payload() {
        let comp = IrComponent {
            name: "C".into(),
            base: "W".into(),
            host_props: vec![],
            host_bindings: vec![],
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

    // ── M4-Phase 2 T6: focus-group / modal-scope / dismiss ──────────────
    //
    // The runtime half of `wasamoc check`'s T6 stage 1 gate (dsl_spec
    // §4.19, DD-M4-P2-005 A1): `validate_focus_annotation_invariants`'s
    // four rejects, plus the ZStack relaxation (`validate_phase6_zstack_node_invariants`)
    // and the ToggleButton dispatch-ordering control
    // (`validate_phase8_togglebutton_node_invariants`).

    /// Body fixture for each of the seven `FOCUS_ANNOTATION_CONTAINERS`
    /// kinds, with `{ATTR}` standing in for the prop line under test.
    /// `ScrollView` carries its required single content child; `Grid`
    /// carries its required `tracks` lines; `Box` stays within its
    /// at-most-one-child limit.
    const FOCUS_ANNOTATION_CONTAINER_FIXTURES: &[(&str, &str)] = &[
        ("VStack", "node VStack { {ATTR} }"),
        ("HStack", "node HStack { {ATTR} }"),
        ("Box", "node Box { {ATTR} }"),
        ("WrapPanel", "node WrapPanel { {ATTR} }"),
        ("ScrollView", "node ScrollView { {ATTR} node Box {} }"),
        (
            "Grid",
            "node Grid { tracks columns = 1* tracks rows = 1* {ATTR} }",
        ),
        ("ZStack", "node ZStack { {ATTR} }"),
    ];

    fn assert_focus_annotation_accepted_everywhere(attr_line: &str) {
        for (kind, body) in FOCUS_ANNOTATION_CONTAINER_FIXTURES {
            let src = format!(
                ";wasamo-ir v0\ncomponent C inherits W {{ {} }}",
                body.replace("{ATTR}", attr_line)
            );
            let c = parse_ok(&src);
            validate(&c).unwrap_or_else(|e| {
                panic!("{kind} accepting `{attr_line}` failed: {e}\nsrc:\n{src}")
            });
        }
    }

    #[test]
    fn focus_group_true_accepted_on_every_admitting_container() {
        assert_focus_annotation_accepted_everywhere("prop focus-group = true");
    }

    #[test]
    fn modal_scope_true_accepted_on_every_admitting_container() {
        assert_focus_annotation_accepted_everywhere("prop modal-scope = true");
    }

    #[test]
    fn focus_group_false_accepted() {
        // `false` is a valid constant, not just `true` (dsl_spec §4.19).
        assert_focus_annotation_accepted_everywhere("prop focus-group = false");
    }

    #[test]
    fn modal_scope_false_accepted() {
        assert_focus_annotation_accepted_everywhere("prop modal-scope = false");
    }

    #[test]
    fn focus_group_and_modal_scope_together_on_one_container_accepted() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node VStack { prop focus-group = true prop modal-scope = true }\n}",
        );
        validate(&c).expect("a container may carry both annotations at once (DD-M4-P2-005)");
    }

    #[test]
    fn dismiss_handler_accepted_beside_modal_scope_true() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node Box { prop modal-scope = true on dismiss { (assign open false) } }\n}",
        );
        validate(&c).expect("dismiss beside modal-scope: true must validate");
    }

    #[test]
    fn dismiss_handler_accepted_on_zstack_carrying_modal_scope() {
        // T6 regression pin, unaffected by T8's removal of the
        // ZStack-specific handler gate: `dismiss` is admitted here via
        // the generic dsl_spec §4.19 rule because the ZStack carries
        // `modal-scope: true`.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node ZStack { prop modal-scope = true on dismiss { (assign open false) } node Text {} }\n}",
        );
        validate(&c).expect("dismiss beside modal-scope: true must validate on ZStack");
    }

    #[test]
    fn dismiss_handler_accepted_on_grid_carrying_modal_scope() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node Grid {\n\
               tracks columns = 1*\n\
               tracks rows = 1*\n\
               prop modal-scope = true\n\
               on dismiss { (assign open false) }\n\
             }\n}",
        );
        validate(&c).expect("dismiss beside modal-scope: true must validate on Grid");
    }

    #[test]
    fn focus_group_true_on_text_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node Text { prop focus-group = true } }",
            "`focus-group` is admitted on any container",
        );
    }

    #[test]
    fn focus_group_true_on_button_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node Button { prop focus-group = true } }",
            "`focus-group` is admitted on any container",
        );
    }

    #[test]
    fn focus_group_true_on_togglebutton_rejected_as_admission_not_unknown_attr() {
        // Dispatch-ordering control: `validate_focus_annotation_invariants`
        // runs before `validate_phase8_togglebutton_node_invariants`
        // (wired in `validate`), so the diagnostic is the admission one,
        // not "unknown ToggleButton attribute".
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W { node ToggleButton { prop focus-group = true } }",
        );
        match err {
            IrLoadError::Validate(msg) => {
                assert!(
                    msg.contains("`focus-group` is admitted on any container"),
                    "got: {msg}"
                );
                assert!(
                    !msg.contains("unknown ToggleButton attribute"),
                    "got: {msg}"
                );
            }
            other => panic!("expected Validate error, got {other:?}"),
        }
    }

    #[test]
    fn focus_group_true_on_rectangle_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node Rectangle { prop focus-group = true } }",
            "`focus-group` is admitted on any container",
        );
    }

    #[test]
    fn focus_group_true_inside_cell_rejected() {
        // Loader-side equivalent of `check.rs`'s
        // `focus_group_true_inside_cell_rejected`: `Cell` is an IR-only
        // Grid wrapper, not a runtime container, and is excluded from
        // `FOCUS_ANNOTATION_CONTAINERS`. Built as a `Grid` child so
        // `Cell` is in a legal position and the admission diagnostic is
        // what fires, not a Cell-outside-Grid one.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Grid {\n\
               tracks columns = 1*\n\
               tracks rows = 1*\n\
               node Cell { prop focus-group = true node Text {} }\n\
             }\n}",
            "`focus-group` is admitted on any container",
        );
    }

    #[test]
    fn modal_scope_true_on_text_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node Text { prop modal-scope = true } }",
            "`modal-scope` is admitted on any container",
        );
    }

    #[test]
    fn focus_group_non_bool_literal_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node VStack { prop focus-group = 1 } }",
            "`focus-group` is constant-only",
        );
    }

    #[test]
    fn modal_scope_non_bool_literal_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node VStack { prop modal-scope = \"yes\" } }",
            "`modal-scope` is constant-only",
        );
    }

    #[test]
    fn focus_group_binding_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node VStack { bind focus-group = true } }",
            "`focus-group` is constant-only",
        );
    }

    #[test]
    fn modal_scope_binding_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node VStack { bind modal-scope = true } }",
            "`modal-scope` is constant-only",
        );
    }

    #[test]
    fn dismiss_handler_without_modal_scope_prop_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node Box { on dismiss { (assign open false) } }\n}",
            "`dismiss` handler can never be raised",
        );
    }

    #[test]
    fn dismiss_handler_beside_modal_scope_false_rejected() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node Box { prop modal-scope = false on dismiss { (assign open false) } }\n}",
            "`dismiss` handler can never be raised",
        );
    }

    #[test]
    fn dismiss_handler_on_non_container_rejected() {
        // A non-container can never carry a `true` `modal-scope` prop
        // (the admission check above already refuses that combination),
        // so its `dismiss` handler always fails the same "no true
        // `modal-scope` sibling" test as the absent/`false` cases above.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node Text { on dismiss { (assign open false) } }\n}",
            "`dismiss` handler can never be raised",
        );
    }

    // Every test above puts `dismiss` as a flat sibling of `modal-scope`
    // directly on a node's own `prop` list. These five exercise the
    // `ControlFlow::If` / `ControlFlow::For` arms of
    // `validate_focus_annotation_member_invariants`, which recurse into
    // an `if` / `for` member's body — the `wasamoc` `check.rs` mirror
    // group above tests the same shapes at compile time; this is the
    // runtime half for memory IR that reaches the loader without
    // traversing `wasamoc`. The `for`-plus-`dismiss` combination cannot
    // reach the `ControlFlow::For` arm's own `dismiss` check at all (see
    // that test's comment); the admission-only `for` test right after it
    // is what actually fires the arm.

    #[test]
    fn dismiss_handler_accepted_inside_if_wrapped_modal_scope() {
        // §4.19's own worked shape: the annotated node is the `if`'s
        // branch body, not a flat sibling of the enclosing widget. Fires
        // the `ControlFlow::If` recursion arm.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node VStack { if true { node Box { prop modal-scope = true on dismiss { (assign open false) } } } }\n}",
        );
        validate(&c).expect("dismiss inside an if-wrapped modal-scope container must validate");
    }

    #[test]
    fn dismiss_handler_inside_if_wrapped_container_without_modal_scope_rejected() {
        // Same shape as the accept test above, minus `prop modal-scope =
        // true`. Proves the `ControlFlow::If` recursion actually
        // re-validates the inner node rather than short-circuiting.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node VStack { if true { node Box { on dismiss { (assign open false) } } } }\n}",
            "`dismiss` handler can never be raised",
        );
    }

    #[test]
    fn dismiss_handler_inside_for_wrapped_container_hits_the_pre_existing_handler_gate_first() {
        // This is *not* the `ControlFlow::For` dismiss-check arm firing —
        // it is unreachable for this shape. `validate()` runs
        // `validate_node_references` (which unconditionally rejects any
        // handler found while `inside_for_template`) before
        // `validate_focus_annotation_invariants`, and each gate in
        // `validate()` short-circuits via `?` on its own first error. So a
        // `dismiss` handler inside a `for` body always surfaces this
        // earlier, handler-name-agnostic message; `validate_focus_annotation_invariants`
        // never runs for this node at all. (Confirmed directly: calling
        // `validate_focus_annotation_invariants` on the parsed tree in
        // isolation *does* return the `dismiss`-specific error — the gate
        // itself is correct, it is simply never reached through the public
        // `parse_ir` entry point for this shape.) This differs from
        // `wasamoc check`'s `check_members_inner`, which accumulates
        // diagnostics into one `Vec` in a single pass instead of
        // short-circuiting per gate, so the checker's `for` counterpart
        // (see `check.rs`) surfaces both messages together.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             state xs: i32[] = []\n\
             node VStack { for x in xs { node Box { on dismiss { (assign open false) } } } }\n}",
            "handlers inside a `for` body template are deferred in M3-Phase 7",
        );
    }

    #[test]
    fn focus_group_true_on_text_inside_for_body_rejected() {
        // The reachable way to fire the `ControlFlow::For` recursion arm:
        // an admission violation carries no handler, so it never trips the
        // earlier, handler-only `validate_node_references` for-template
        // gate (see the test above) and reaches
        // `validate_focus_annotation_invariants` inside the `for` body.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state xs: i32[] = []\n\
             node VStack { for x in xs { node Text { prop focus-group = true } } }\n}",
            "`focus-group` is admitted on any container",
        );
    }

    #[test]
    fn focus_group_true_on_text_inside_if_body_rejected() {
        // The `ControlFlow::If` recursion must reach the admission check
        // too, not only the `dismiss` check, for a node nested inside an
        // `if` body.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W { node VStack { if true { node Text { prop focus-group = true } } } }",
            "`focus-group` is admitted on any container",
        );
    }

    #[test]
    fn zstack_spacing_prop_still_rejected_after_relaxation() {
        // Control proving the ZStack relaxation stayed narrow: an
        // ordinary Phase-6-rejected attribute is still rejected.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node ZStack { prop spacing = 4 node Text {} }\n}",
            "`ZStack` accepts no Phase-6 attributes; found `spacing`",
        );
    }

    #[test]
    fn grid_clicked_handler_validates() {
        // T8: `Grid` never had a per-kind handler gate in the loader
        // (only `wasamoc check` did, asymmetrically with ZStack); pin
        // the accept case explicitly, symmetric with
        // `zstack_clicked_handler_validates` above.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node Grid {\n\
               tracks columns = 1*\n\
               tracks rows = 1*\n\
               on clicked { (assign count 1) }\n\
             }\n}",
        );
        assert_eq!(c.root.widget_type, "Grid");
        assert_eq!(c.root.handlers.len(), 1);
        assert_eq!(c.root.handlers[0].signal, "clicked");
    }

    #[test]
    fn dismiss_handler_on_zstack_without_modal_scope_prop_rejected() {
        // T8: proves the generic dsl_spec §4.19 `dismiss` rule
        // (`validate_focus_annotation_invariants`) still owns ZStack
        // admission now that the ZStack-specific handler gate is gone —
        // the diagnostic is the generic "can never be raised" message,
        // the same one every other widget kind produces (see
        // `dismiss_handler_without_modal_scope_prop_rejected` above for
        // the Box case), not a ZStack-specific rejection.
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state open: bool = true\n\
             node ZStack { on dismiss { (assign open false) } node Text {} }\n}",
            "`dismiss` handler can never be raised",
        );
    }

    // ── M4-Phase 2 T8: `key-down("<key>")` argument (dsl_spec §4.19
    // "Keyboard input", DD-M4-P2-005) ───────────────────────────────────
    //
    // Parse half: the optional `( STRING )` after the signal name
    // (`parse_handler`). Second-gate half: `validate_key_down_invariants`
    // re-checks the same three shapes `wasamoc check` rejects at compile
    // time, for memory IR that reaches the loader without traversing
    // `wasamoc`.

    #[test]
    fn key_down_handler_parses_with_string_argument() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state selected_index: i32 = 0\n\
             node Box { on key-down(\"ArrowLeft\") { (compound-assign -= selected_index 1) } }\n}",
        );
        assert_eq!(c.root.handlers.len(), 1);
        let h = &c.root.handlers[0];
        assert_eq!(h.signal, "key-down");
        assert_eq!(h.arg.as_deref(), Some("ArrowLeft"));
    }

    #[test]
    fn clicked_handler_still_parses_with_arg_none() {
        // Regression guard: a plain `on clicked { ... }` (no parenthesised
        // argument) keeps parsing with `arg == None` after the optional
        // `( STRING )` production is added.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node Button { on clicked { (compound-assign += count 1) } }\n}",
        );
        let h = &c.root.handlers[0];
        assert_eq!(h.signal, "clicked");
        assert_eq!(h.arg, None);
    }

    #[test]
    fn key_down_without_argument_rejected_at_validate() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state selected_index: i32 = 0\n\
             node Box { on key-down { (compound-assign -= selected_index 1) } }\n}",
            "`key-down` handler can never be raised",
        );
    }

    #[test]
    fn key_down_unrecognised_key_name_rejected_at_validate() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state selected_index: i32 = 0\n\
             node Box { on key-down(\"Tab\") { (compound-assign -= selected_index 1) } }\n}",
            "unrecognised key",
        );
    }

    #[test]
    fn argument_on_clicked_rejected_at_validate() {
        assert_validate_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node Button { on clicked(\"x\") { (compound-assign += count 1) } }\n}",
            "`clicked` does not take an argument",
        );
    }
}
