use std::collections::HashMap;

use crate::ast::{
    BlockStatement, CollectionElemType, ComponentDef, Expr, Member, QualifiedName, Span, TrackAxis,
    TrackSize, TypeName,
};
use crate::diagnostic::Diagnostic;

const KNOWN_WIDGET_TYPES: &[&str] = &[
    "VStack",
    "HStack",
    "Text",
    "Button",
    "ToggleButton",
    "Rectangle",
    "Box",
    "WrapPanel",
    "ScrollView",
    "Grid",
    "ZStack",
];

/// Attribute names a `Cell` may carry (DD-M3-P5-001 / DD-M3-P5-005).
/// `row` / `column` are placement; `row-span` / `column-span` are span;
/// `h-align` / `v-align` are per-cell alignment. Any other PropertyBind
/// on a `Cell` is an unknown-attribute reject (DD-M3-P5-006).
const CELL_ATTRS: &[&str] = &[
    "row",
    "column",
    "row-span",
    "column-span",
    "h-align",
    "v-align",
];

/// Alignment vocabulary for `Cell.h-align` / `Cell.v-align`
/// (DD-M3-P5-005). `stretch` is the default; the other three position
/// the content within the resolved cell rectangle.
const ALIGN_VALUES: &[&str] = &["start", "center", "end", "stretch"];

const GRID_SLOT_KEYS: &[&str] = &[
    "row",
    "column",
    "row-span",
    "column-span",
    "h-align",
    "v-align",
];

const ZSTACK_SLOT_KEYS: &[&str] = &["h-align", "v-align"];

/// Parent-owned child-placement attributes (Grid `Cell` / ZStack direct
/// children). They are consumed by the parent context, not by the child widget.
const CHILD_PLACEMENT_ATTRS: &[&str] = &["h-align", "v-align"];

/// Per-axis weighted-star upper bound (DD-M3-P5-002 / DD-M3-P5-006).
const STAR_WEIGHT_MAX: i64 = 1024;

/// WrapPanel's three constant-only `i32` attributes per dsl_spec §4.10
/// (DD-M3-P3-003 / DD-M3-P3-004). Listed in a single table so the
/// rejection paths (`bind`-style state-ident, non-`IntLit` literal,
/// attribute-outside-WrapPanel) share one source of truth.
const WRAPPANEL_INT_ATTRS: &[&str] = &["item-cross-size", "item-spacing", "line-spacing"];

/// Container widget kinds that admit the M4-Phase 2 focus annotations
/// (`focus-group` / `modal-scope`), per dsl_spec §4.19 "admitted on any
/// container" (DD-M4-P2-005 A1). `Text`, `Button`, `ToggleButton`, and
/// `Rectangle` are leaf/content widgets, not containers, and are excluded.
/// `Cell` is an IR-only Grid wrapper (not a runtime container) and is
/// excluded too.
const FOCUS_ANNOTATION_CONTAINERS: &[&str] = &[
    "VStack",
    "HStack",
    "Box",
    "WrapPanel",
    "ScrollView",
    "Grid",
    "ZStack",
];

/// The two constant-only boolean focus annotations (dsl_spec §4.19).
const FOCUS_ANNOTATION_ATTRS: &[&str] = &["focus-group", "modal-scope"];

/// Host-owned attributes admitted at component level in M3-Phase 6.
/// The catalog is host-general in shape but contains only the Window entry
/// this phase (DD-M3-P6-008 A2a).
pub const HOST_STATIC_ATTRS: &[&str] = &["title", "backdrop", "theme"];

/// Flat namespace of declared state names → their types.
pub type Namespace = HashMap<String, TypeName>;

struct LoopContext<'a> {
    binder: &'a str,
    index_binder: Option<&'a str>,
    elem: CollectionElemType,
}

fn is_loop_local_ident(name: &str, loop_ctx: Option<&LoopContext<'_>>) -> bool {
    loop_ctx
        .map(|ctx| name == ctx.binder || ctx.index_binder == Some(name))
        .unwrap_or(false)
}

fn qualified_loop_local_segment<'a>(
    qn: &'a QualifiedName,
    loop_ctx: Option<&LoopContext<'_>>,
) -> Option<&'a str> {
    if qn.segments.len() <= 1 {
        return None;
    }
    qn.segments
        .iter()
        .map(String::as_str)
        .find(|segment| is_loop_local_ident(segment, loop_ctx))
}

pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
    pub namespace: Namespace,
}

impl CheckResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == crate::diagnostic::Severity::Error)
    }
}

pub fn check(ast: &ComponentDef, filename: &str) -> CheckResult {
    let mut diags = Vec::new();
    let namespace = collect_state_namespace(&ast.members, filename, &mut diags);
    check_state_defaults(&ast.members, filename, &namespace, &mut diags);
    check_members(&ast.members, filename, &namespace, &mut diags);
    CheckResult {
        diagnostics: diags,
        namespace,
    }
}

/// First pass: collect `state` names and detect duplicates.
fn collect_state_namespace(
    members: &[Member],
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) -> Namespace {
    let mut ns: Namespace = HashMap::new();
    for member in members {
        if let Member::StateMember { name, ty, span, .. } = member {
            if ns.contains_key(name) {
                diags.push(error(
                    filename,
                    span,
                    format!("duplicate state name `{}`", name),
                ));
            } else {
                ns.insert(name.clone(), ty.clone());
            }
        }
    }
    ns
}

/// Second pass (state-side): validate that each state's default expression
/// type matches the declared type. Runs before `check_members` so type
/// mismatches surface at the declaration site.
fn check_state_defaults(
    members: &[Member],
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
) {
    for member in members {
        if let Member::StateMember {
            name,
            ty,
            default,
            span,
        } = member
        {
            if let TypeName::Collection(elem) = ty {
                check_collection_literal(default, *elem, span, filename, "state default", diags);
                continue;
            }
            if matches!(default, Expr::ListLit { .. }) {
                diags.push(error(
                    filename,
                    default.span(),
                    format!(
                        "list literal default is only valid for collection states; scalar state `{}` is declared `{}`",
                        name,
                        type_name_display(ty),
                    ),
                ));
                continue;
            }
            // Validate the expression form first (rejects FloatLit, plus
            // positional Ratio / Color literals per dsl_spec §4.9). The
            // type-compatibility check below operates on the static type
            // only and silently skips Float / Ratio / Color, so without
            // this call those forms would slip through.
            check_expr_type(default, span, filename, ns, diags);
            let Some(source_ty) = expr_static_type(default, ns) else {
                continue;
            };
            if matches!(source_ty, TypeName::Float) {
                // Float literals are rejected separately in `check_expr_type`;
                // suppress the redundant type-mismatch diagnostic.
                continue;
            }
            if !types_compatible(ty, &source_ty) {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "type mismatch in state default for `{}`: declared `{}`, got `{}`",
                        name,
                        type_name_display(ty),
                        type_name_display(&source_ty),
                    ),
                ));
            }
        }
    }
}

/// Returns the static type of an expression, or `None` if the type cannot be
/// determined here (e.g. an identifier that does not resolve to a declared
/// state name — treated as a widget-property keyword value).
fn expr_static_type(expr: &Expr, ns: &Namespace) -> Option<TypeName> {
    expr_static_type_in_context(expr, ns, None)
}

fn check_collection_literal(
    expr: &Expr,
    elem: CollectionElemType,
    ctx_span: &Span,
    filename: &str,
    position: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let Expr::ListLit { items, .. } = expr else {
        diags.push(error(
            filename,
            ctx_span,
            format!(
                "collection {} for `{}` must be a list literal of `{}` elements",
                position,
                type_name_display(&TypeName::Collection(elem)),
                collection_elem_display(elem)
            ),
        ));
        return;
    };

    for item in items {
        match (elem, item) {
            (CollectionElemType::Int, Expr::IntLit { .. })
            | (CollectionElemType::Str, Expr::StringLit { .. })
            | (CollectionElemType::Bool, Expr::BoolLit { .. }) => {}
            (_, Expr::ListLit { .. }) => diags.push(error(
                filename,
                item.span(),
                "nested list literals are not supported in M3-Phase 7 collection values",
            )),
            (_, Expr::Ident { .. } | Expr::QualifiedRef { .. } | Expr::CollectionCall { .. }) => {
                diags.push(error(
                    filename,
                    item.span(),
                    "collection literal elements must be scalar literals; collection expressions are not yet supported",
                ));
            }
            _ => diags.push(error(
                filename,
                item.span(),
                format!(
                    "collection literal element type mismatch: expected `{}`, got `{}`",
                    collection_elem_display(elem),
                    expr_type_name_for_diagnostic(item)
                ),
            )),
        }
    }
}

fn expr_type_name_for_diagnostic(expr: &Expr) -> &'static str {
    match expr {
        Expr::IntLit { .. } | Expr::Measurement { .. } => "i32",
        Expr::StringLit { .. } => "string",
        Expr::BoolLit { .. } => "bool",
        Expr::FloatLit { .. } => "float",
        Expr::ListLit { .. } => "list",
        Expr::RatioLit { .. } => "ratio",
        Expr::ColorLit { .. } => "color",
        Expr::Ident { .. } | Expr::QualifiedRef { .. } | Expr::CollectionCall { .. } => {
            "expression"
        }
        Expr::UnsupportedOperator { .. } => "operator",
    }
}

fn expr_static_type_in_context(
    expr: &Expr,
    ns: &Namespace,
    loop_ctx: Option<&LoopContext<'_>>,
) -> Option<TypeName> {
    match expr {
        Expr::IntLit { .. } | Expr::Measurement { .. } => Some(TypeName::Int),
        Expr::FloatLit { .. } => Some(TypeName::Float),
        Expr::StringLit { .. } => Some(TypeName::Str),
        Expr::BoolLit { .. } => Some(TypeName::Bool),
        Expr::Ident { name, .. } => {
            if let Some(ctx) = loop_ctx {
                if name == ctx.binder {
                    return Some(collection_elem_as_type(ctx.elem));
                }
                if ctx.index_binder == Some(name.as_str()) {
                    return Some(TypeName::Int);
                }
            }
            ns.get(name).cloned()
        }
        Expr::QualifiedRef { name } => name
            .segments
            .last()
            .and_then(|state| ns.get(state).cloned()),
        Expr::ListLit { items, .. } => infer_list_type(items, ns, loop_ctx),
        Expr::CollectionCall { receiver, .. } => receiver
            .segments
            .last()
            .and_then(|state| ns.get(state).cloned())
            .filter(|ty| matches!(ty, TypeName::Collection(_))),
        // Ratio / Color are Box-internal value types (DD-M3-P2-002 /
        // DD-M3-P2-003 Option A); they have no `TypeName` entry. Position
        // and value validity for these literals are checked at the
        // property-bind layer (T3), not via the state-type compatibility
        // table.
        Expr::RatioLit { .. } | Expr::ColorLit { .. } | Expr::UnsupportedOperator { .. } => None,
    }
}

fn infer_list_type(
    items: &[Expr],
    ns: &Namespace,
    loop_ctx: Option<&LoopContext<'_>>,
) -> Option<TypeName> {
    let mut elem_ty: Option<TypeName> = None;
    for item in items {
        let ty = expr_static_type_in_context(item, ns, loop_ctx)?;
        if matches!(ty, TypeName::Collection(_)) {
            return None;
        }
        match &elem_ty {
            Some(existing) if !types_compatible(existing, &ty) => return None,
            None => elem_ty = Some(ty),
            _ => {}
        }
    }
    elem_ty.and_then(|ty| match ty {
        TypeName::Int => Some(TypeName::Collection(CollectionElemType::Int)),
        TypeName::Str => Some(TypeName::Collection(CollectionElemType::Str)),
        TypeName::Bool => Some(TypeName::Collection(CollectionElemType::Bool)),
        TypeName::Float | TypeName::Collection(_) => None,
    })
}

fn types_compatible(a: &TypeName, b: &TypeName) -> bool {
    matches!(
        (a, b),
        (TypeName::Int, TypeName::Int)
            | (TypeName::Str, TypeName::Str)
            | (TypeName::Bool, TypeName::Bool)
            | (TypeName::Float, TypeName::Float)
            | (
                TypeName::Collection(CollectionElemType::Int),
                TypeName::Collection(CollectionElemType::Int)
            )
            | (
                TypeName::Collection(CollectionElemType::Str),
                TypeName::Collection(CollectionElemType::Str)
            )
            | (
                TypeName::Collection(CollectionElemType::Bool),
                TypeName::Collection(CollectionElemType::Bool)
            )
    )
}

fn collection_elem_as_type(elem: CollectionElemType) -> TypeName {
    match elem {
        CollectionElemType::Int => TypeName::Int,
        CollectionElemType::Str => TypeName::Str,
        CollectionElemType::Bool => TypeName::Bool,
    }
}

fn collection_elem_display(elem: CollectionElemType) -> &'static str {
    match elem {
        CollectionElemType::Int => "i32",
        CollectionElemType::Str => "string",
        CollectionElemType::Bool => "bool",
    }
}

fn type_name_display(ty: &TypeName) -> String {
    match ty {
        TypeName::Int => "i32".to_string(),
        TypeName::Str => "string".to_string(),
        TypeName::Float => "float".to_string(),
        TypeName::Bool => "bool".to_string(),
        TypeName::Collection(elem) => format!("{}[]", collection_elem_display(*elem)),
    }
}

/// Widget property type catalog. Returns the declared `TypeName` for known
/// (widget, property) pairs and `None` for everything else — including
/// non-typed values like enum / keyword props (`Button.style: accent`,
/// `Text.font: title`) which the loader handles by ident name. The catalog
/// is soft: an entry is added only when the property has a meaningful
/// static type to check against. Mirrors `wasamo-runtime`'s
/// `resolve_prop_key` table (DD-M3-P1-009) but lives here so `wasamoc check`
/// is self-contained.
fn widget_prop_type(widget_type: &str, prop_name: &str) -> Option<TypeName> {
    match (widget_type, prop_name) {
        ("Text", "text") => Some(TypeName::Str),
        ("Button", "text") => Some(TypeName::Str),
        ("Button", "enabled") => Some(TypeName::Bool),
        ("ToggleButton", "text") => Some(TypeName::Str),
        ("ToggleButton", "enabled") => Some(TypeName::Bool),
        ("ToggleButton", "checked") => Some(TypeName::Bool),
        // `Box.aspect: Ratio` and `Box.fill: Color` are Box-internal value
        // types (DD-M3-P2-002 / DD-M3-P2-003 Option A); they are not
        // `TypeName` entries and bypass the type-compatibility table.
        // Validity is checked by `check_box_const_only_bind` instead. The
        // catalog row is named here so future maintainers can grep for it.
        // WrapPanel's three attributes are catalog-typed `i32` per
        // dsl_spec §4.10 so the type-mismatch diagnostic can name them in
        // bind contexts that survive the constant-only gate (none in
        // Phase 3 — the gate runs first — but the catalog row keeps the
        // attribute types reachable for future bindable phases).
        ("WrapPanel", "item-cross-size")
        | ("WrapPanel", "item-spacing")
        | ("WrapPanel", "line-spacing") => Some(TypeName::Int),
        _ => None,
    }
}

/// Box-internal property name used to name the rejected attribute in
/// `bind`-style diagnostics. Mirrors the spec's "Box.aspect: Ratio" /
/// "Box.fill: Color" surface (dsl_spec §4.9 attribute table).
fn box_prop_type_name(prop_name: &str) -> &'static str {
    match prop_name {
        "aspect" => "Ratio",
        "fill" => "Color",
        _ => unreachable!("box_prop_type_name called on non-Box-const-only prop"),
    }
}

/// Validate a property bind on `Box.aspect` or `Box.fill`. Both attributes
/// are constant-only per DD-M3-P2-004: the RHS must be the matching literal
/// kind exactly, not a state-backed ident or another literal form. Value
/// validity (positive ratio sides) is checked inline.
fn check_box_const_only_bind(
    prop_name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match (prop_name, value) {
        (
            "aspect",
            Expr::RatioLit {
                num,
                den,
                span: lit_span,
            },
        ) => {
            if *num <= 0 {
                diags.push(error(
                    filename,
                    lit_span,
                    format!(
                        "ratio literal numerator must be positive (got {}); `Box.aspect` requires `<num>:<den>` with both sides > 0",
                        num
                    ),
                ));
            }
            if *den <= 0 {
                diags.push(error(
                    filename,
                    lit_span,
                    format!(
                        "ratio literal denominator must be positive (got {}); `Box.aspect` requires `<num>:<den>` with both sides > 0",
                        den
                    ),
                ));
            }
        }
        ("fill", Expr::ColorLit { .. }) => {
            // ColorLit is syntactically validated at the lexer (6 or 8
            // hex digits). No additional value-validity check at the
            // check layer.
        }
        (prop, _) => {
            diags.push(error(
                filename,
                span,
                format!(
                    "`Box.{}` is constant-only in M3-Phase 2; expected a `{}` literal (`{}`), not a state-backed binding or other expression",
                    prop,
                    box_prop_type_name(prop),
                    box_prop_surface_form(prop),
                ),
            ));
        }
    }
}

/// Surface-form hint embedded in the diagnostic for `Box.aspect` /
/// `Box.fill` (dsl_spec §4.9 attribute table).
fn box_prop_surface_form(prop_name: &str) -> &'static str {
    match prop_name {
        "aspect" => "<num>:<den>",
        "fill" => "#RRGGBB or #RRGGBBAA",
        _ => unreachable!("box_prop_surface_form called on non-Box-const-only prop"),
    }
}

fn slot_key(name: &str) -> Option<&str> {
    name.strip_prefix("slot.")
}

fn slot_attr_label(key: &str) -> String {
    format!("slot.{key}")
}

/// Validate a property bind on `WrapPanel.item-cross-size`,
/// `WrapPanel.item-spacing`, or `WrapPanel.line-spacing`. All three are
/// constant-only `i32` per DD-M3-P3-003 / DD-M3-P3-004: the RHS must be
/// an `IntLit` (positive or zero — negatives are rejected by the
/// DD-M3-P3-006 compile-time gate). State-backed idents, ratio / color /
/// string / bool / measurement literals are all rejected here so the
/// diagnostic can name the attribute; the generic positional Ratio /
/// Color rejection in `check_expr_type` is bypassed.
fn check_wrappanel_const_only_bind(
    prop_name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match value {
        Expr::IntLit {
            value: v,
            span: lit_span,
        } => {
            if *v < 0 {
                diags.push(error(
                    filename,
                    lit_span,
                    format!(
                        "`WrapPanel.{}` must be a non-negative integer (got {}); \
                         negative spacing / cross-axis sizes are rejected per dsl_spec §4.10",
                        prop_name, v
                    ),
                ));
            }
        }
        _ => {
            diags.push(error(
                filename,
                span,
                format!(
                    "`WrapPanel.{}` is constant-only in M3-Phase 3; expected a non-negative `i32` literal, \
                     not a state-backed binding or other expression form",
                    prop_name
                ),
            ));
        }
    }
}

/// Reject WrapPanel attributes appearing outside a WrapPanel widget
/// (DD-M3-P3-003 / DD-M3-P3-004 attribute-position). Diagnostic names
/// the offending position (component-level vs other widget type) so
/// the author knows the attribute name was recognised but is misplaced.
fn check_wrappanel_attr_outside_wrappanel(
    prop_name: &str,
    enclosing_widget: Option<&str>,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let position = match enclosing_widget {
        Some(w) => format!("widget `{}`", w),
        None => "component-level property".to_string(),
    };
    diags.push(error(
        filename,
        span,
        format!(
            "`{}` is a WrapPanel attribute (dsl_spec §4.10) and is not valid on {}",
            prop_name, position
        ),
    ));
}

/// Validate a property bind on `focus-group` or `modal-scope`. Both are
/// constant-only `bool` per dsl_spec §4.19 (DD-M4-P2-005 A1): the RHS
/// must be a `BoolLit` literal exactly, not a state-backed ident or any
/// other expression form. One reject arm covers every non-`BoolLit`
/// shape (bare ident, int / string / ratio / color / measurement
/// literal) so the diagnostic can name the attribute; mirrors
/// `check_wrappanel_const_only_bind`'s constant-only shape.
fn check_focus_annotation_const_only_bind(
    prop_name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if !matches!(value, Expr::BoolLit { .. }) {
        diags.push(error(
            filename,
            span,
            format!(
                "`{}` is constant-only (dsl_spec §4.19); expected a `true` or `false` literal, \
                 not a state-backed binding or other expression form",
                prop_name
            ),
        ));
    }
}

/// Reject `focus-group` / `modal-scope` appearing on a widget kind that
/// is not in `FOCUS_ANNOTATION_CONTAINERS` (dsl_spec §4.19 "admitted on
/// any container"). Diagnostic names the attribute and the offending
/// widget kind so the author knows the attribute name was recognised but
/// is misplaced; mirrors `check_wrappanel_attr_outside_wrappanel`'s shape.
///
/// Takes the widget kind by value rather than as an `Option` because the
/// component-level position never reaches here — the
/// `enclosing_widget.is_none()` early return in `check_members_inner`
/// routes a bare `focus-group:` to `check_host_property_bind` first — so
/// a component-level arm would be a diagnostic no test could fire.
fn check_focus_annotation_admission(
    prop_name: &str,
    enclosing_widget: &str,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    diags.push(error(
        filename,
        span,
        format!(
            "`{}` is admitted on any container (dsl_spec §4.19) and is not valid on widget `{}`",
            prop_name, enclosing_widget
        ),
    ));
}

/// Emit a warning when a container carries both `focus-group: true` and
/// `modal-scope: true` (dsl_spec §4.19). The surface stays *accepted* —
/// DD-M4-P2-005 chose two separate constant-only attributes precisely so
/// a container could carry both — but `focus_core::FocusRole`
/// (`wasamo-runtime/src/widget.rs`) holds one role per node, and
/// `WidgetNode::focus_role` gives `modal-scope` precedence, so the
/// `focus-group` half of such a node is inert at runtime. That is worth
/// flagging even though it is not rejected.
///
/// Takes the already-computed `carries_modal_scope` rather than
/// rescanning for it, so this stays a cheap second pass over `members`
/// guarded by the same `FOCUS_ANNOTATION_CONTAINERS` test the caller
/// already applied. Anchored on the `focus-group` bind's own span, not
/// the enclosing widget's — `check_members_inner` has the member list
/// but not the widget's span, and the attribute with no effect is the
/// more useful thing to point at anyway.
fn check_focus_group_inert_beside_modal_scope_warning(
    members: &[Member],
    carries_modal_scope: bool,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if !carries_modal_scope {
        return;
    }
    let focus_group_span = members.iter().find_map(|m| match m {
        Member::PropertyBind {
            name,
            value: Expr::BoolLit { value: true, .. },
            span,
        } if name == "focus-group" => Some(span),
        _ => None,
    });
    let Some(span) = focus_group_span else {
        return;
    };
    diags.push(Diagnostic::warning(
        filename,
        span.line,
        span.col,
        "this container carries both `focus-group: true` and `modal-scope: true`; a container \
         can behave as one or the other, not both, and a modal scope wins, so `focus-group` \
         has no effect here. Remove it, or move the group to a child container \
         (dsl_spec §4.19)."
            .to_string(),
    ));
}

/// Emit a warning when a WrapPanel directly contains one or more
/// `Box { aspect: <ratio>; … }` children and `item-cross-size` is not
/// set on the WrapPanel itself (DD-M3-P3-004 Recommendation companion;
/// the "huge thumbnail" footgun documented in dsl_spec §4.10 Common
/// pitfalls). The guard scope is intentionally narrow: only direct-
/// child Boxes with an `aspect` PropertyBind are classified; nested
/// containers are not scanned, and other size-source shapes are not
/// enumerated. One warning per WrapPanel regardless of how many
/// matching children it contains.
fn check_wrappanel_aspect_only_box_warning(
    wrappanel_members: &[Member],
    wrappanel_span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let has_item_cross_size = wrappanel_members
        .iter()
        .any(|m| matches!(m, Member::PropertyBind { name, .. } if name == "item-cross-size"));
    if has_item_cross_size {
        return;
    }
    let any_aspect_only_box = wrappanel_members.iter().any(|m| match m {
        Member::WidgetDecl {
            type_name, members, ..
        } if type_name == "Box" => members
            .iter()
            .any(|cm| matches!(cm, Member::PropertyBind { name, .. } if name == "aspect")),
        _ => false,
    });
    if !any_aspect_only_box {
        return;
    }
    diags.push(Diagnostic::warning(
        filename,
        wrappanel_span.line,
        wrappanel_span.col,
        "`WrapPanel` directly contains an aspect-only `Box` child without `item-cross-size` set; \
         each child will inherit the parent's cross-axis constraint, producing the \"huge thumbnail\" \
         footgun. Set `item-cross-size` on the WrapPanel to bound the per-item cross-axis size \
         (dsl_spec §4.10 Common pitfalls).",
    ));
}

/// Reject any ScrollView attribute other than `offset-y` in Phase 4
/// (DD-M3-P4-001 / DD-M3-P4-002 scoping). `viewport-width`,
/// `viewport-height`, `scroll-axis`, and `padding` are explicitly
/// out of scope per dsl_spec §4.11 *Attributes*; this is the catch-
/// all rejection that fires for any other PropertyBind name a future
/// author or migration might attempt before the corresponding DD
/// opens that attribute. The diagnostic names the attribute and
/// points at §4.11 so authors can locate the scoping decision.
fn check_scrollview_unknown_attr(
    prop_name: &str,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    diags.push(error(
        filename,
        span,
        format!(
            "`{}` is not a recognised ScrollView attribute in M3-Phase 4; only `offset-y` is in scope (dsl_spec §4.11)",
            prop_name
        ),
    ));
}

fn check_togglebutton_property_name(
    prop_name: &str,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if !matches!(prop_name, "text" | "style" | "enabled" | "checked") {
        diags.push(error(
            filename,
            span,
            format!(
                "unknown ToggleButton attribute `{}`; valid attributes: text, style, enabled, checked (dsl_spec §4.17)",
                prop_name
            ),
        ));
    }
}

fn check_checked_attr_admission(
    widget: &str,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    diags.push(error(
        filename,
        span,
        format!(
            "`checked` is only valid on ToggleButton, not `{}` (dsl_spec §4.17)",
            widget
        ),
    ));
}

/// Validate a `ScrollView.offset-y` binding RHS (DD-M3-P4-003). The
/// attribute is bindable read-only in Phase 4: the RHS is either an
/// `IntLit` (any sign — negatives and out-of-range values are clamped
/// at layout time per DD-M3-P4-005, not rejected here) or a bare state
/// identifier whose namespace type is `i32` (per dsl_spec §4.3 bare-
/// ident binding form). All other expression shapes — non-integer
/// literals, undeclared idents, `bool` / `string` / `float` state
/// idents — are rejected with a diagnostic that names the rejected
/// surface.
fn check_scrollview_offset_y_bind(
    value: &Expr,
    span: &Span,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
) {
    match value {
        Expr::IntLit { .. } => {}
        Expr::Ident {
            name,
            span: ident_span,
        } => match ns.get(name) {
            Some(TypeName::Int) => {}
            Some(other) => {
                diags.push(error(
                    filename,
                    ident_span,
                    format!(
                        "`ScrollView.offset-y` binds to an `i32` state; state `{}` is declared `{}` (dsl_spec §4.11)",
                        name,
                        type_name_display(other),
                    ),
                ));
            }
            None => {
                diags.push(error(
                    filename,
                    ident_span,
                    format!(
                        "`ScrollView.offset-y` binds to a declared `i32` state; `{}` is not declared (declare it with `state {}: i32 = 0`)",
                        name, name,
                    ),
                ));
            }
        },
        _ => {
            diags.push(error(
                filename,
                span,
                "`ScrollView.offset-y` accepts an `i32` literal or a bare `i32` state identifier (dsl_spec §4.11); other expression forms are rejected",
            ));
        }
    }
}

/// Reject the writable (in-out) surface on `ScrollView.offset-y`. Phase
/// 4 binding is read-only per DD-M3-P4-003 Option B; the in-out form
/// (`in-out property<i32> offset-y: 0` inside a ScrollView body) would
/// declare a writable component-shaped surface, which would require the
/// general typed-`i32` writer pair deferred to M4 (per
/// architecture.md §6.8 *Per-type seam* paragraph). Naming the
/// rejection explicitly here keeps the M4 hand-off legible.
fn check_scrollview_writable_offset_y(span: &Span, filename: &str, diags: &mut Vec<Diagnostic>) {
    diags.push(error(
        filename,
        span,
        "`ScrollView.offset-y` is bindable read-only in M3-Phase 4 (dsl_spec §4.11); the writable `in-out property<i32> offset-y` surface is deferred to M4 or later",
    ));
}

/// Reject a ScrollView widget with anything other than exactly one
/// child widget (DD-M3-P4-001 / DD-M3-P4-006). The runtime IR loader
/// independently rejects the same shape at IR-load time (defense in
/// depth, per Phase 2 T7 / Phase 3 T6 pattern); the compile-time
/// diagnostic names the offending count and points authors at the
/// `ScrollView { VStack { … } }` wrapping pattern for multi-child
/// content.
fn check_scrollview_child_count(
    members: &[Member],
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    // T4 review follow-up / DD-M3-P6-007 (interim): a conditional is not a
    // valid direct ScrollView content member (its presence is dynamic, so
    // it cannot satisfy "exactly one content child"). Wrap it in the
    // content widget. The conditionally-empty relaxation is the open
    // DD-M3-P6-007 question; until decided this stays rejected. Reported as
    // the primary diagnostic so a conditional sibling (`ScrollView {
    // Content  if c { … } }`) does not slip past the widget-only count.
    let mut has_conditional = false;
    for m in members {
        match m {
            Member::Conditional { span, .. } => {
                has_conditional = true;
                diags.push(error(
                    filename,
                    span,
                    "`ScrollView` content child must be a single widget; a conditional member is not valid directly in ScrollView (wrap it in the content widget)",
                ));
            }
            Member::For { span, .. } => {
                has_conditional = true;
                diags.push(error(
                    filename,
                    span,
                    "`ScrollView` content child must be a single widget; a `for` member is not valid directly in ScrollView (wrap it in the content widget)",
                ));
            }
            _ => {}
        }
    }
    if has_conditional {
        return;
    }
    let child_count = members
        .iter()
        .filter(|m| matches!(m, Member::WidgetDecl { .. }))
        .count();
    if child_count != 1 {
        diags.push(error(
            filename,
            span,
            format!(
                "`ScrollView` requires exactly one child widget in M3-Phase 4 (found {}); wrap multiple children in an explicit container such as `ScrollView {{ VStack {{ … }} }}`",
                child_count
            ),
        ));
    }
}

/// Reject a Box widget with two or more child widgets (DD-M3-P2-001
/// multi-child). The runtime IR loader independently rejects the same
/// shape at IR-load time (defense in depth); the compile-time diagnostic
/// recommends ZStack / VStack / HStack for multi-child needs.
fn check_box_child_count(
    members: &[Member],
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    // T4 review follow-up: count every member that can materialise a child,
    // not widget declarations only. An `if` member materialises at most one
    // child, so a conditional sibling counts toward the at-most-one limit
    // (`Box { Text  if c { … } }` could become two children). A lone
    // conditional (`Box { if c { … } }`) is one potential child and stays
    // valid. The prior widget-only count under-counted the sibling (see
    // log.md T4 migration audit).
    let child_count = members
        .iter()
        .filter(|m| {
            matches!(
                m,
                Member::WidgetDecl { .. } | Member::Conditional { .. } | Member::For { .. }
            )
        })
        .count();
    if child_count > 1 {
        diags.push(error(
            filename,
            span,
            format!(
                "`Box` admits at most one child widget in M3-Phase 2 (found {}); use `ZStack` (overlay), `VStack` (vertical) or `HStack` (horizontal) for multi-child layouts",
                child_count
            ),
        ));
    }
}

/// Reject a Button-family widget (`Button` / `ToggleButton`) carrying any
/// child-materialising member (owner disposition CF-1, 2026-08-07).
/// `build_layout_tree` (`wasamo-runtime/src/widget.rs`) maps both widget
/// kinds to a childless `LayoutNode::rectangle`: an authored child is
/// accepted here by the generic widget-decl walk, built by the IR loader,
/// but unknown to layout — it renders nothing in release and aborts a
/// debug build during `wasamo_load_ui` on the `sync_visuals` child-count
/// assertion.
///
/// Mirrors `check_box_child_count`'s completeness: every member that can
/// materialise a child counts (`WidgetDecl`, `Conditional`, `For`), not
/// widget declarations only, so `Button { if c { Text {} } }` is caught
/// even though it authors no bare widget child. Property binds, signal
/// handlers, and `slot.*` placement binds are unaffected — those are not
/// `Member::WidgetDecl` / `Conditional` / `For` variants.
///
/// `Text` and `Rectangle` share the identical layout gap but are
/// deliberately out of scope for this rule; they are tracked as a
/// separate finding.
fn check_button_family_children(
    widget_kind: &str,
    members: &[Member],
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let child_count = members
        .iter()
        .filter(|m| {
            matches!(
                m,
                Member::WidgetDecl { .. } | Member::Conditional { .. } | Member::For { .. }
            )
        })
        .count();
    if child_count > 0 {
        diags.push(error(
            filename,
            span,
            format!(
                "`{}` admits no widget children (found {}); Button-family widgets take a `text:` label rather than authored content (dsl_spec §4.8)",
                widget_kind, child_count
            ),
        ));
    }
}

fn check_zstack_unknown_attr(name: &str, span: &Span, filename: &str, diags: &mut Vec<Diagnostic>) {
    diags.push(error(
        filename,
        span,
        format!(
            "unknown ZStack attribute `{}`; ZStack declares no Phase-6 attributes (dsl_spec §4.13)",
            name
        ),
    ));
}

fn check_unknown_slot_key(
    key: &str,
    parent: &str,
    allowed: &[&str],
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    diags.push(error(
        filename,
        span,
        format!(
            "unknown `{}` slot key `slot.{}`; valid keys: {}",
            parent,
            key,
            allowed
                .iter()
                .map(|k| format!("`slot.{k}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    ));
}

fn check_slot_property_outside_parent(
    key: &str,
    enclosing_widget: Option<&str>,
    parent_widget: Option<&str>,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let position = match (enclosing_widget, parent_widget) {
        (Some("Cell"), Some("Grid")) => "among `Cell` placement attributes".to_string(),
        (Some(widget), Some("Cell")) => {
            format!("on a widget inside a `Cell` content child (`{widget}`)")
        }
        (Some(widget), _) => format!("inside `{}`", widget),
        (None, _) => "at component level".to_string(),
    };
    diags.push(error(
        filename,
        span,
        format!(
            "`slot.{}` is parent-owned child placement data; it is only valid on a Grid direct child or a ZStack direct child; found {}",
            key, position
        ),
    ));
}

fn check_slot_mixing_in_cell(key: &str, span: &Span, filename: &str, diags: &mut Vec<Diagnostic>) {
    diags.push(error(
        filename,
        span,
        format!(
            "strict PM-2 mixing reject: `slot.{}` cannot appear among `Cell` attributes; use either `Cell {{ row: ... }}` grouped placement or direct `slot.*` on a Grid child, not both",
            key
        ),
    ));
}

fn check_child_placement_outside_parent(
    name: &str,
    enclosing_widget: Option<&str>,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let position = match enclosing_widget {
        Some("ZStack") => "on `ZStack` itself".to_string(),
        Some(widget) => format!("inside `{}`", widget),
        None => "at component level".to_string(),
    };
    diags.push(error(
        filename,
        span,
        format!(
            "`{}` is a parent-owned child placement attribute; it is only valid on a ZStack direct child or a Grid `Cell` (dsl_spec §4.13); found {}",
            name, position
        ),
    ));
}

fn check_zstack_bare_child_placement(
    name: &str,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    diags.push(error(
        filename,
        span,
        format!(
            "ZStack child bare `{}` is no longer accepted; use `slot.{}` for parent-owned placement",
            name, name
        ),
    ));
}

fn check_zstack_child_align(
    name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let allowed = ALIGN_VALUES
        .iter()
        .map(|v| format!("`{}`", v))
        .collect::<Vec<_>>()
        .join(", ");
    match value {
        Expr::Ident { name: value, .. } => {
            if !ALIGN_VALUES.contains(&value.as_str()) {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "ZStack child `{}` must be one of {} (got `{}`) (dsl_spec §4.13)",
                        name, allowed, value
                    ),
                ));
            }
        }
        _ => {
            diags.push(error(
                filename,
                span,
                format!(
                    "ZStack child `{}` expects an alignment keyword ({}) (dsl_spec §4.13)",
                    name, allowed
                ),
            ));
        }
    }
}

fn check_host_property_bind(
    name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
) {
    if !HOST_STATIC_ATTRS.contains(&name) {
        diags.push(error(
            filename,
            span,
            format!(
                "unknown host attribute `{}`; M3-Phase 6 host attributes are: {}",
                name,
                HOST_STATIC_ATTRS.join(", ")
            ),
        ));
        return;
    }

    // A state-backed identifier is a *dynamic* host binding. Dynamic host
    // attributes are deferred this phase (FD-D), so reject them with a
    // binding-specific message — distinct from a wrong-typed static literal,
    // which is a different mistake with a different fix.
    if matches!(value, Expr::Ident { name: ident, .. } if ns.contains_key(ident)) {
        diags.push(error(
            filename,
            span,
            format!(
                "host attribute `{}` is not bindable in M3-Phase 6; dynamic host attributes are deferred",
                name
            ),
        ));
        return;
    }

    if name == "title" {
        if !matches!(value, Expr::StringLit { .. }) {
            diags.push(error(
                filename,
                span,
                "host attribute `title` must be a string literal in M3-Phase 6",
            ));
        }
        return;
    }

    // `backdrop` / `theme` take keyword/enum identifiers this phase
    // (e.g. `mica`, `system`), which carry no static `TypeName`. A concrete
    // typed literal (int / float / bool / string) is not a valid host value.
    if expr_static_type(value, ns).is_some() {
        diags.push(error(
            filename,
            span,
            format!(
                "host attribute `{}` does not accept a literal value in M3-Phase 6; use a keyword (e.g. `mica`, `system`)",
                name
            ),
        ));
    }
}

fn check_if_condition(
    condition: &Expr,
    filename: &str,
    ns: &Namespace,
    loop_ctx: Option<&LoopContext<'_>>,
    diags: &mut Vec<Diagnostic>,
) {
    match condition {
        Expr::BoolLit { .. } => {}
        Expr::Ident { name, span } => {
            if loop_ctx
                .map(|ctx| name == ctx.binder || ctx.index_binder == Some(name.as_str()))
                .unwrap_or(false)
            {
                diags.push(error(
                    filename,
                    span,
                    "loop binders in `if` conditions are deferred in M3-Phase 7; conditions resolve only to bool state",
                ));
                return;
            }
            match ns.get(name) {
                Some(TypeName::Bool) => {}
                Some(other) => diags.push(error(
                    filename,
                    span,
                    format!(
                        "`if` condition must be `bool`; state `{}` is declared `{}` (dsl_spec §4.14)",
                        name,
                        type_name_display(other)
                    ),
                )),
                None => diags.push(error(
                    filename,
                    span,
                    format!(
                        "`if` condition identifier `{}` is not declared; declare it as `state {}: bool = false`",
                        name, name
                    ),
                )),
            }
        }
        Expr::QualifiedRef { name } => {
            check_qualified_name(name, filename, ns, diags);
        }
        Expr::UnsupportedOperator { op, span } => diags.push(error(
            filename,
            span,
            format!(
                "operators in `if` conditions are not yet supported in M3-Phase 6 (got {}); use a bool literal or declared bool state",
                op
            ),
        )),
        _ => {
            diags.push(error(
                filename,
                condition.span(),
                "`if` condition must be a bool literal or declared bool state identifier (dsl_spec §4.14)",
            ));
        }
    }
}

fn check_if_body(body: &[Member], span: &Span, filename: &str, diags: &mut Vec<Diagnostic>) {
    let widget_count = body
        .iter()
        .filter(|m| matches!(m, Member::WidgetDecl { .. }))
        .count();
    if body.len() != 1 || widget_count != 1 {
        diags.push(error(
            filename,
            span,
            "`if` body admits exactly one widget child in M3-Phase 6; wrap multiple widgets or nested control flow in a container",
        ));
    }
    for member in body {
        match member {
            Member::WidgetDecl { .. } => {}
            Member::Conditional { span, .. } => diags.push(error(
                filename,
                span,
                "a bare nested `if` is not admitted directly in an `if` body in M3-Phase 6; wrap it in a widget container",
            )),
            Member::For { span, .. } => diags.push(error(
                filename,
                span,
                "a bare nested `for` is not admitted directly in an `if` body in M3-Phase 7; wrap it in a widget container",
            )),
            Member::PropertyBind { span, .. }
            | Member::PropertyDecl { span, .. }
            | Member::SignalHandler { span, .. }
            | Member::StateMember { span, .. }
            | Member::GridTracks { span, .. } => diags.push(error(
                filename,
                span,
                "`if` body admits only a single widget child; properties, bindings, handlers, state declarations, and track lists are not structural body members",
            )),
        }
    }
}

/// Reject a `Cell` that appears outside a `Grid` parent (DD-M3-P5-001 /
/// DD-M3-P5-006). `Cell` is a Grid-owned IR-only wrapper with no
/// general-purpose use; the diagnostic names the offending position.
fn check_cell_outside_grid(
    enclosing_widget: Option<&str>,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let position = match enclosing_widget {
        Some(w) => format!("inside `{}`", w),
        None => "at component level".to_string(),
    };
    diags.push(error(
        filename,
        span,
        format!(
            "`Cell` is only valid as a direct child of a `Grid` (dsl_spec §4.12); found {}",
            position
        ),
    ));
}

/// A resolved Cell rectangle in track coordinates, used for overlap
/// detection. Populated only when placement and span all validate.
struct CellRect {
    row: i64,
    column: i64,
    row_span: i64,
    column_span: i64,
}

enum GridPlacementChild<'a> {
    Cell {
        members: &'a [Member],
        span: &'a Span,
    },
    Direct {
        members: &'a [Member],
        span: &'a Span,
    },
}

/// Validate a `Grid` widget body (DD-M3-P5-001 .. DD-M3-P5-006). This is
/// the Grid-level pass: Cell placement bounds depend on the declared
/// track counts and overlap detection compares all cells, so every
/// member is examined together. Per-cell intra-cell validation is
/// dispatched from here too — `Cell` is only valid inside a `Grid`, so
/// all Cell diagnostics live in one place.
fn check_grid(members: &[Member], grid_span: &Span, filename: &str, diags: &mut Vec<Diagnostic>) {
    // 1. Track lists — validate values; record declared track counts.
    let mut columns_len: Option<usize> = None;
    let mut rows_len: Option<usize> = None;
    for m in members {
        if let Member::GridTracks { axis, tracks, span } = m {
            check_grid_track_list(tracks, filename, diags);
            let slot = match axis {
                TrackAxis::Columns => &mut columns_len,
                TrackAxis::Rows => &mut rows_len,
            };
            if slot.is_some() {
                diags.push(error(
                    filename,
                    span,
                    format!("duplicate `{}:` track list on Grid", axis.attr_name()),
                ));
            } else {
                *slot = Some(tracks.len());
            }
        }
    }

    // 2. Minimum shape — both track lists present (DD-M3-P5-001).
    if columns_len.is_none() {
        diags.push(error(
            filename,
            grid_span,
            "`Grid` requires a `columns:` track list (dsl_spec §4.12)",
        ));
    }
    if rows_len.is_none() {
        diags.push(error(
            filename,
            grid_span,
            "`Grid` requires a `rows:` track list (dsl_spec §4.12)",
        ));
    }

    // 3. Collect placement-bearing children. Grid admits retained `Cell`
    // wrappers and direct child widgets with `slot.*` placement.
    let mut placement_children: Vec<GridPlacementChild<'_>> = Vec::new();
    for m in members {
        match m {
            Member::GridTracks { .. } => {}
            Member::WidgetDecl {
                type_name,
                members: cm,
                span,
            } if type_name == "Cell" => {
                placement_children.push(GridPlacementChild::Cell { members: cm, span });
            }
            Member::WidgetDecl {
                members: cm, span, ..
            } => {
                placement_children.push(GridPlacementChild::Direct { members: cm, span });
            }
            Member::PropertyBind { name, span, .. } => {
                if slot_key(name).is_some() {
                    // `slot.*` on the Grid node itself is parent-owned
                    // placement data for the Grid's *parent* (e.g. a ZStack
                    // that admits the Grid as a placed direct child), not a
                    // Grid-own attribute. It is validated by the parent via
                    // the generic member walk (`check_members_inner`'s
                    // `parent_widget` dispatch), so the Grid pass must not
                    // consume it. Consuming it here wrongly rejected a Grid
                    // placed in a ZStack as "inside `Grid`" (M3-Phase 7b T6b;
                    // DD-M3-P7b-001 intent: `slot.*` is valid on a ZStack
                    // direct child, and a Grid is a widget). A `slot.*` on a
                    // Grid under a non-admitting parent (or at component
                    // level) is still rejected by that same generic walk.
                } else if FOCUS_ANNOTATION_ATTRS.contains(&name.as_str()) {
                    // `focus-group` / `modal-scope` on the Grid node itself
                    // (dsl_spec §4.19) are validated by the generic
                    // `check_members_inner` dispatch, which runs separately
                    // on Grid's own children with `enclosing_widget ==
                    // Some("Grid")`. Consuming them here would produce a
                    // spurious second "unknown Grid attribute" diagnostic
                    // (same shape as the `slot.*` skip above).
                } else {
                    diags.push(error(
                        filename,
                        span,
                        format!(
                            "unknown Grid attribute `{}`; Grid declares only `columns:` and `rows:` (dsl_spec §4.12)",
                            name
                        ),
                    ));
                }
            }
            Member::Conditional { span, .. } => {
                diags.push(error(filename, span, "`Grid` children must be wrapped in `Cell`; conditional members may appear inside a Cell content widget, not directly in Grid"));
            }
            Member::For { span, .. } => {
                diags.push(error(
                    filename,
                    span,
                    "`Grid` children must be wrapped in `Cell`; direct `for` members are not valid in Grid",
                ));
            }
            // Handler admission on a `Grid` is the generic rule that
            // `check_members_inner` applies to every widget kind
            // (dsl_spec.md §4.19's admission table: `clicked` on any
            // widget; `dismiss` only beside `modal-scope: true`). This
            // `check_grid` pass holds no per-kind signal rule.
            Member::SignalHandler { .. }
            | Member::StateMember { .. }
            | Member::PropertyDecl { .. } => {}
        }
    }

    // 4. Per-child validation + rectangle collection. The retained `Cell`
    //    grouped form keeps its single-child placement escape. Direct
    //    `slot.*` children use child-slot defaults for omitted placement.
    let single_grid_child = placement_children.len() == 1;
    let mut rects: Vec<(CellRect, &Span)> = Vec::new();
    for child in &placement_children {
        match child {
            GridPlacementChild::Cell { members, span } => {
                if let Some(rect) = check_cell(
                    members,
                    span,
                    single_grid_child,
                    columns_len,
                    rows_len,
                    filename,
                    diags,
                ) {
                    rects.push((rect, span));
                }
            }
            GridPlacementChild::Direct { members, span } => {
                if let Some(rect) = check_grid_direct_child_slot(
                    members,
                    span,
                    columns_len,
                    rows_len,
                    filename,
                    diags,
                ) {
                    rects.push((rect, span));
                }
            }
        }
    }

    // 5. Same-cell / overlapping-rectangle conflict (DD-M3-P5-003).
    check_cell_overlaps(&rects, filename, diags);
}

/// Validate one Grid track list's values (DD-M3-P5-002 / DD-M3-P5-006).
/// Shape was settled by the parser; this pass enforces value ranges and
/// rejects the reserved-future `auto`, floats, and unknown words.
fn check_grid_track_list(tracks: &[TrackSize], filename: &str, diags: &mut Vec<Diagnostic>) {
    for t in tracks {
        match t {
            TrackSize::Fixed { value, span } => {
                if *value < 1 {
                    diags.push(error(
                        filename,
                        span,
                        format!(
                            "fixed track size must be a positive integer (got {}); 0 and negative tracks are rejected (dsl_spec §4.12)",
                            value
                        ),
                    ));
                } else if *value > i32::MAX as i64 {
                    diags.push(error(
                        filename,
                        span,
                        format!("fixed track size {} is out of range", value),
                    ));
                }
            }
            TrackSize::Star { weight, span } => {
                if *weight < 1 {
                    diags.push(error(
                        filename,
                        span,
                        format!(
                            "star weight must be >= 1 (got {}); `0*` and negative weights are rejected (dsl_spec §4.12)",
                            weight
                        ),
                    ));
                } else if *weight > STAR_WEIGHT_MAX {
                    diags.push(error(
                        filename,
                        span,
                        format!(
                            "star weight must not exceed {} (got {}); express larger proportions with additional tracks (dsl_spec §4.12)",
                            STAR_WEIGHT_MAX, weight
                        ),
                    ));
                }
            }
            TrackSize::InvalidFloat { span } => {
                diags.push(error(
                    filename,
                    span,
                    "floating-point track sizes are not valid in M3-Phase 5; use an integer (fixed px) or `n*` (weighted star) (dsl_spec §4.12)",
                ));
            }
            TrackSize::Word { name, span } => {
                if name == "auto" {
                    diags.push(error(
                        filename,
                        span,
                        "`auto` track sizing is reserved for a future phase and is not available in M3-Phase 5; use a fixed (integer px) or weighted-star (`n*`) track (dsl_spec §4.12)",
                    ));
                } else {
                    diags.push(error(
                        filename,
                        span,
                        format!(
                            "unknown track size token `{}`; expected an integer (fixed px) or `n*` (weighted star) (dsl_spec §4.12)",
                            name
                        ),
                    ));
                }
            }
        }
    }
}

/// Validate one `Cell` body (DD-M3-P5-001 / DD-M3-P5-003 / DD-M3-P5-005 /
/// DD-M3-P5-006). Returns the resolved rectangle when placement and span
/// all validate, so the Grid pass can detect overlaps.
#[allow(clippy::too_many_arguments)]
fn check_cell(
    members: &[Member],
    cell_span: &Span,
    single_cell: bool,
    columns_len: Option<usize>,
    rows_len: Option<usize>,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<CellRect> {
    // Single content child (DD-M3-P5-001).
    let child_count = members
        .iter()
        .filter(|m| matches!(m, Member::WidgetDecl { .. }))
        .count();
    if child_count != 1 {
        diags.push(error(
            filename,
            cell_span,
            format!(
                "`Cell` requires exactly one content child (found {}); wrap multiple widgets in a container such as `Cell {{ VStack {{ … }} }}` (dsl_spec §4.12)",
                child_count
            ),
        ));
    }

    // Placement / span / alignment attributes; unknown attributes rejected.
    let mut row_present = false;
    let mut column_present = false;
    let mut row: Option<i64> = None;
    let mut column: Option<i64> = None;
    let mut row_span: Option<i64> = Some(1);
    let mut column_span: Option<i64> = Some(1);
    for m in members {
        match m {
            Member::PropertyBind { name, value, span } => match name.as_str() {
                _ if slot_key(name).is_some() => {
                    check_slot_mixing_in_cell(slot_key(name).unwrap(), span, filename, diags)
                }
                "row" => {
                    row_present = true;
                    row = check_cell_index(name, value, span, filename, diags);
                }
                "column" => {
                    column_present = true;
                    column = check_cell_index(name, value, span, filename, diags);
                }
                "row-span" => row_span = check_cell_span(name, value, span, filename, diags),
                "column-span" => column_span = check_cell_span(name, value, span, filename, diags),
                "h-align" | "v-align" => check_cell_align(name, value, span, filename, diags),
                _ => diags.push(error(
                    filename,
                    span,
                    format!(
                        "unknown `Cell` attribute `{}`; valid attributes: {} (dsl_spec §4.12)",
                        name,
                        CELL_ATTRS.join(", ")
                    ),
                )),
            },
            Member::Conditional { span, .. } => diags.push(error(
                filename,
                span,
                "`Cell` admits exactly one direct widget content child; put conditional members inside that content widget",
            )),
            Member::For { span, .. } => diags.push(error(
                filename,
                span,
                "`Cell` admits exactly one direct widget content child; direct `for` members are not valid in Grid placement contexts",
            )),
            _ => {}
        }
    }

    // Placement presence (DD-M3-P5-001 placement-default Option A): a
    // multi-Cell Grid requires explicit `row:` / `column:`; a single-Cell
    // Grid defaults to (0, 0).
    let resolved_row = resolve_placement(
        row_present,
        row,
        single_cell,
        "row",
        cell_span,
        filename,
        diags,
    );
    let resolved_col = resolve_placement(
        column_present,
        column,
        single_cell,
        "column",
        cell_span,
        filename,
        diags,
    );

    // Bound checks against declared track counts (DD-M3-P5-003): the
    // half-open rectangle must lie within the track grid.
    if let (Some(r), Some(rs), Some(rl)) = (resolved_row, row_span, rows_len) {
        if r + rs > rl as i64 {
            diags.push(error(
                filename,
                cell_span,
                format!(
                    "`Cell` row span exceeds the grid: row {} + row-span {} = {} > {} declared row tracks (dsl_spec §4.12)",
                    r, rs, r + rs, rl
                ),
            ));
        }
    }
    if let (Some(c), Some(cs), Some(cl)) = (resolved_col, column_span, columns_len) {
        if c + cs > cl as i64 {
            diags.push(error(
                filename,
                cell_span,
                format!(
                    "`Cell` column span exceeds the grid: column {} + column-span {} = {} > {} declared column tracks (dsl_spec §4.12)",
                    c, cs, c + cs, cl
                ),
            ));
        }
    }

    match (resolved_row, resolved_col, row_span, column_span) {
        (Some(row), Some(column), Some(row_span), Some(column_span)) => Some(CellRect {
            row,
            column,
            row_span,
            column_span,
        }),
        _ => None,
    }
}

fn check_grid_direct_child_slot(
    members: &[Member],
    child_span: &Span,
    columns_len: Option<usize>,
    rows_len: Option<usize>,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<CellRect> {
    let mut row: Option<i64> = Some(0);
    let mut column: Option<i64> = Some(0);
    let mut row_span: Option<i64> = Some(1);
    let mut column_span: Option<i64> = Some(1);

    for m in members {
        if let Member::PropertyBind { name, value, span } = m {
            let Some(key) = slot_key(name) else {
                continue;
            };
            match key {
                "row" => row = check_grid_slot_index(key, value, span, filename, diags),
                "column" => column = check_grid_slot_index(key, value, span, filename, diags),
                "row-span" => row_span = check_grid_slot_span(key, value, span, filename, diags),
                "column-span" => {
                    column_span = check_grid_slot_span(key, value, span, filename, diags)
                }
                "h-align" | "v-align" => check_grid_slot_align(key, value, span, filename, diags),
                _ => check_unknown_slot_key(key, "Grid", GRID_SLOT_KEYS, span, filename, diags),
            }
        }
    }

    if let (Some(r), Some(rs), Some(rl)) = (row, row_span, rows_len) {
        if r + rs > rl as i64 {
            diags.push(error(
                filename,
                child_span,
                format!(
                    "`slot.row` placement exceeds the grid: row {} + row-span {} = {} > {} declared row tracks (dsl_spec §4.16)",
                    r, rs, r + rs, rl
                ),
            ));
        }
    }
    if let (Some(c), Some(cs), Some(cl)) = (column, column_span, columns_len) {
        if c + cs > cl as i64 {
            diags.push(error(
                filename,
                child_span,
                format!(
                    "`slot.column` placement exceeds the grid: column {} + column-span {} = {} > {} declared column tracks (dsl_spec §4.16)",
                    c, cs, c + cs, cl
                ),
            ));
        }
    }

    match (row, column, row_span, column_span) {
        (Some(row), Some(column), Some(row_span), Some(column_span)) => Some(CellRect {
            row,
            column,
            row_span,
            column_span,
        }),
        _ => None,
    }
}

fn check_grid_slot_index(
    key: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match value {
        Expr::IntLit { value: v, .. } => {
            if *v < 0 {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "`{}` must be a non-negative integer (got {}); placement is zero-based (dsl_spec §4.16)",
                        slot_attr_label(key),
                        v
                    ),
                ));
                None
            } else {
                Some(*v)
            }
        }
        _ => {
            diags.push(error(
                filename,
                span,
                format!(
                    "`{}` must be a non-negative integer literal; placement is constant per instance (dsl_spec §4.16)",
                    slot_attr_label(key)
                ),
            ));
            None
        }
    }
}

fn check_grid_slot_span(
    key: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match value {
        Expr::IntLit { value: v, .. } => {
            if *v < 1 {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "`{}` must be a positive integer (>= 1) (got {}) (dsl_spec §4.16)",
                        slot_attr_label(key),
                        v
                    ),
                ));
                None
            } else {
                Some(*v)
            }
        }
        _ => {
            diags.push(error(
                filename,
                span,
                format!(
                    "`{}` must be a positive integer literal; placement is constant per instance (dsl_spec §4.16)",
                    slot_attr_label(key)
                ),
            ));
            None
        }
    }
}

fn check_grid_slot_align(
    key: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match value {
        Expr::Ident { name: v, .. } => {
            if !ALIGN_VALUES.contains(&v.as_str()) {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "`{}` must be one of {} (got `{}`) (dsl_spec §4.16)",
                        slot_attr_label(key),
                        ALIGN_VALUES.join(", "),
                        v
                    ),
                ));
            }
        }
        _ => {
            diags.push(error(
                filename,
                span,
                format!(
                    "`{}` expects an alignment keyword ({}); placement is constant per instance (dsl_spec §4.16)",
                    slot_attr_label(key),
                    ALIGN_VALUES.join(", ")
                ),
            ));
        }
    }
}

/// Resolve a Cell's `row` / `column` placement, applying the single-Cell
/// (0, 0) default and rejecting a missing placement in a multi-Cell Grid.
/// Returns the resolved index only when it is present-and-valid or
/// defaulted; a present-but-invalid value returns `None` without a
/// redundant "must declare" diagnostic.
fn resolve_placement(
    present: bool,
    value: Option<i64>,
    single_cell: bool,
    axis: &str,
    cell_span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<i64> {
    if present {
        value
    } else if single_cell {
        Some(0)
    } else {
        diags.push(error(
            filename,
            cell_span,
            format!(
                "`Cell` in a multi-cell Grid must declare `{}:` (dsl_spec §4.12); only a single-Cell Grid may omit placement",
                axis
            ),
        ));
        None
    }
}

/// Validate a `Cell` placement index (`row` / `column`): a non-negative
/// integer literal. Returns the value when valid.
fn check_cell_index(
    name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match value {
        Expr::IntLit { value: v, .. } => {
            if *v < 0 {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "`Cell.{}` must be a non-negative integer (got {}); placement is zero-based (dsl_spec §4.12)",
                        name, v
                    ),
                ));
                None
            } else {
                Some(*v)
            }
        }
        _ => {
            diags.push(error(
                filename,
                span,
                format!(
                    "`Cell.{}` must be a non-negative integer literal (dsl_spec §4.12)",
                    name
                ),
            ));
            None
        }
    }
}

/// Validate a `Cell` span (`row-span` / `column-span`): a positive
/// integer literal (`>= 1`). Returns the value when valid.
fn check_cell_span(
    name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match value {
        Expr::IntLit { value: v, .. } => {
            if *v < 1 {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "`Cell.{}` must be a positive integer (>= 1) (got {}) (dsl_spec §4.12)",
                        name, v
                    ),
                ));
                None
            } else {
                Some(*v)
            }
        }
        _ => {
            diags.push(error(
                filename,
                span,
                format!(
                    "`Cell.{}` must be a positive integer literal (dsl_spec §4.12)",
                    name
                ),
            ));
            None
        }
    }
}

/// Validate a `Cell` alignment value (`h-align` / `v-align`): an
/// identifier from the alignment vocabulary (DD-M3-P5-005).
fn check_cell_align(
    name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
    match value {
        Expr::Ident { name: v, .. } => {
            if !ALIGN_VALUES.contains(&v.as_str()) {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "`Cell.{}` must be one of {} (got `{}`) (dsl_spec §4.12)",
                        name,
                        ALIGN_VALUES.join(", "),
                        v
                    ),
                ));
            }
        }
        _ => {
            diags.push(error(
                filename,
                span,
                format!(
                    "`Cell.{}` expects an alignment keyword ({}) (dsl_spec §4.12)",
                    name,
                    ALIGN_VALUES.join(", ")
                ),
            ));
        }
    }
}

/// Detect same-cell / overlapping-rectangle conflicts among a Grid's
/// resolved Cell rectangles (DD-M3-P5-003). Intentional overlay is
/// ZStack's responsibility (Phase 6), not Grid's.
fn check_cell_overlaps(rects: &[(CellRect, &Span)], filename: &str, diags: &mut Vec<Diagnostic>) {
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            let (a, _) = &rects[i];
            let (b, b_span) = &rects[j];
            if rects_overlap(a, b) {
                diags.push(error(
                    filename,
                    b_span,
                    format!(
                        "`Cell` at (row {}, column {}) overlaps an earlier Cell's rectangle; same-cell and overlapping placements are rejected — use `ZStack` for intentional overlay (dsl_spec §4.12)",
                        b.row, b.column
                    ),
                ));
            }
        }
    }
}

fn rects_overlap(a: &CellRect, b: &CellRect) -> bool {
    ranges_overlap(a.row, a.row_span, b.row, b.row_span)
        && ranges_overlap(a.column, a.column_span, b.column, b.column_span)
}

/// Half-open `[start, start + len)` interval overlap.
fn ranges_overlap(s1: i64, len1: i64, s2: i64, len2: i64) -> bool {
    s1 < s2 + len2 && s2 < s1 + len1
}

/// Second pass: validate widget types, property-bind types, and name references.
fn check_members(members: &[Member], filename: &str, ns: &Namespace, diags: &mut Vec<Diagnostic>) {
    let loop_binders = collect_loop_binders(members);
    check_members_inner(
        members,
        None,
        None,
        filename,
        ns,
        diags,
        None,
        false,
        &loop_binders,
    );
}

fn collect_loop_binders(members: &[Member]) -> Vec<String> {
    let mut binders = Vec::new();
    for member in members {
        match member {
            Member::For {
                binder,
                index_binder,
                body,
                ..
            } => {
                binders.push(binder.clone());
                if let Some(index) = index_binder {
                    binders.push(index.clone());
                }
                binders.extend(collect_loop_binders(body));
            }
            Member::WidgetDecl { members, .. } | Member::Conditional { body: members, .. } => {
                binders.extend(collect_loop_binders(members));
            }
            _ => {}
        }
    }
    binders
}

fn check_members_inner(
    members: &[Member],
    enclosing_widget: Option<&str>,
    parent_widget: Option<&str>,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
    loop_ctx: Option<&LoopContext<'_>>,
    inside_for_template: bool,
    all_loop_binders: &[String],
) {
    // `dismiss` is admitted only on **a container that carries
    // `modal-scope: true`** (dsl_spec §4.19) — both halves, not just the
    // sibling. The kind test is what keeps the rule from being satisfied
    // by an annotation that is itself rejected: `Text { modal-scope: true
    // dismiss => … }` gets two diagnostics (the attribute is not admitted
    // on a leaf widget, and the handler can never be raised), where a
    // sibling-only predicate would have suppressed the second. At
    // component level there is no container at all, so the answer is
    // `false` without scanning.
    //
    // Computed once per call so the `Member::SignalHandler` arm below can
    // check it without rescanning; recomputed fresh at each recursive
    // call (widget body, `if` body, `for` body) so a handler is checked
    // against its own immediate siblings, not an ancestor's.
    let carries_modal_scope = enclosing_widget
        .is_some_and(|widget| FOCUS_ANNOTATION_CONTAINERS.contains(&widget))
        && members.iter().any(|m| {
            matches!(
                m,
                Member::PropertyBind {
                    name,
                    value: Expr::BoolLit { value: true, .. },
                    ..
                } if name == "modal-scope"
            )
        });

    // Both halves accepted, but not silently — see the function doc for
    // why `focus-group` is inert once `modal-scope` is also present.
    check_focus_group_inert_beside_modal_scope_warning(
        members,
        carries_modal_scope,
        filename,
        diags,
    );

    for member in members {
        match member {
            Member::StateMember { .. } => {}

            Member::PropertyDecl { name, span, .. } => {
                // `in-out property<i32> offset-y: ...` inside a ScrollView
                // body is the writable surface DD-M3-P4-003 Option C
                // deferred to M4. Reject so the read-only contract from
                // Option B is enforced at compile time, not silently
                // dropped by the no-op PropertyDecl arm.
                if enclosing_widget == Some("ScrollView") && name == "offset-y" {
                    check_scrollview_writable_offset_y(span, filename, diags);
                }
            }

            Member::PropertyBind { name, value, span } => {
                if enclosing_widget.is_none() {
                    if let Some(key) = slot_key(name) {
                        check_slot_property_outside_parent(
                            key,
                            enclosing_widget,
                            parent_widget,
                            span,
                            filename,
                            diags,
                        );
                        continue;
                    }
                    check_host_property_bind(name, value, span, filename, ns, diags);
                    continue;
                }
                // Box.aspect and Box.fill are constant-only per DD-M3-P2-004:
                // the RHS must be the matching literal kind, not a state-
                // backed ident or any other expression form. Validate here
                // and skip the generic `check_expr_type` path, which would
                // otherwise re-reject the literal positionally (the Ratio /
                // Color arm rejects every appearance outside this site).
                if let Some(key) = slot_key(name) {
                    if enclosing_widget == Some("Cell") && parent_widget == Some("Grid") {
                        // Grid's enclosing pass emits the strict PM-2
                        // mixing diagnostic for Cell attributes.
                    } else if parent_widget == Some("Grid") {
                        // Grid's enclosing pass validates direct child
                        // slot keys, values, bounds, and overlap.
                    } else if parent_widget == Some("ZStack") {
                        if ZSTACK_SLOT_KEYS.contains(&key) {
                            check_zstack_child_align(name, value, span, filename, diags);
                        } else {
                            check_unknown_slot_key(
                                key,
                                "ZStack",
                                ZSTACK_SLOT_KEYS,
                                span,
                                filename,
                                diags,
                            );
                        }
                    } else {
                        check_slot_property_outside_parent(
                            key,
                            enclosing_widget,
                            parent_widget,
                            span,
                            filename,
                            diags,
                        );
                    }
                } else if CHILD_PLACEMENT_ATTRS.contains(&name.as_str()) {
                    if enclosing_widget == Some("Cell") {
                        // Grid's enclosing pass validates Cell placement.
                    } else if parent_widget == Some("ZStack") {
                        check_zstack_bare_child_placement(name, span, filename, diags);
                    } else {
                        check_child_placement_outside_parent(
                            name,
                            enclosing_widget,
                            span,
                            filename,
                            diags,
                        );
                    }
                } else if FOCUS_ANNOTATION_ATTRS.contains(&name.as_str()) {
                    // `focus-group` / `modal-scope` (dsl_spec §4.19) are
                    // admitted on any container kind (DD-M4-P2-005 A1) and
                    // are constant-only booleans. This dispatch must run
                    // before the ZStack / ScrollView / ToggleButton
                    // per-kind gates below, which would otherwise swallow
                    // the name as an unknown attribute on their own kind.
                    //
                    // `enclosing_widget` is `Some` here — the
                    // component-level early return above already routed a
                    // bare `focus-group:` to `check_host_property_bind`.
                    if let Some(widget) = enclosing_widget {
                        if FOCUS_ANNOTATION_CONTAINERS.contains(&widget) {
                            check_focus_annotation_const_only_bind(
                                name, value, span, filename, diags,
                            );
                        } else {
                            check_focus_annotation_admission(name, widget, span, filename, diags);
                        }
                    }
                } else if enclosing_widget == Some("ZStack") {
                    check_zstack_unknown_attr(name, span, filename, diags);
                } else if enclosing_widget == Some("ScrollView") && name == "offset-y" {
                    // ScrollView's only Phase 4 attribute (DD-M3-P4-003):
                    // i32-literal-or-bare-i32-state-ident, validated by the
                    // ScrollView-specific helper to produce a diagnostic
                    // that names the attribute. Skip the generic
                    // `check_expr_type` / `check_property_bind_target`
                    // path; the latter would pass through anyway because
                    // ScrollView has no widget_prop_type catalog entry,
                    // but the type-mismatch wording would not name the
                    // ScrollView-specific Phase 4 surface contract.
                    check_scrollview_offset_y_bind(value, span, filename, ns, diags);
                } else if enclosing_widget == Some("ScrollView")
                    && !WRAPPANEL_INT_ATTRS.contains(&name.as_str())
                {
                    // Any non-`offset-y` ScrollView attribute is out of
                    // Phase 4 scope (`viewport-*`, `scroll-axis`,
                    // `padding`, …). The WrapPanel-attribute-outside-
                    // WrapPanel branch below covers the WrapPanel attr
                    // names so the diagnostic stays attribute-specific;
                    // everything else falls into this catch-all.
                    check_scrollview_unknown_attr(name, span, filename, diags);
                } else if enclosing_widget == Some("Box")
                    && (name.as_str() == "aspect" || name.as_str() == "fill")
                {
                    check_box_const_only_bind(name, value, span, filename, diags);
                } else if name == "checked" && enclosing_widget != Some("ToggleButton") {
                    // Component-level `checked` is routed through
                    // `check_host_property_bind` above; this helper is only
                    // for widget-body admission.
                    if let Some(widget) = enclosing_widget {
                        check_checked_attr_admission(widget, span, filename, diags);
                    }
                } else if enclosing_widget == Some("ToggleButton") {
                    check_togglebutton_property_name(name, span, filename, diags);
                    check_expr_type_in_loop_context(
                        value,
                        span,
                        filename,
                        ns,
                        loop_ctx,
                        inside_for_template,
                        all_loop_binders,
                        diags,
                    );
                    check_property_bind_target_in_context(
                        enclosing_widget,
                        name,
                        value,
                        span,
                        filename,
                        ns,
                        loop_ctx,
                        inside_for_template,
                        diags,
                    );
                } else if WRAPPANEL_INT_ATTRS.contains(&name.as_str()) {
                    // WrapPanel's three attributes (DD-M3-P3-003 /
                    // DD-M3-P3-004) are constant-only `i32` per dsl_spec
                    // §4.10. Two-position dispatch: inside `WrapPanel`,
                    // validate the literal shape and non-negative value;
                    // anywhere else, reject the attribute by position.
                    if enclosing_widget == Some("WrapPanel") {
                        check_wrappanel_const_only_bind(name, value, span, filename, diags);
                    } else {
                        check_wrappanel_attr_outside_wrappanel(
                            name,
                            enclosing_widget,
                            span,
                            filename,
                            diags,
                        );
                    }
                } else {
                    check_expr_type_in_loop_context(
                        value,
                        span,
                        filename,
                        ns,
                        loop_ctx,
                        inside_for_template,
                        all_loop_binders,
                        diags,
                    );
                    check_property_bind_target_in_context(
                        enclosing_widget,
                        name,
                        value,
                        span,
                        filename,
                        ns,
                        loop_ctx,
                        inside_for_template,
                        diags,
                    );
                }
            }

            Member::WidgetDecl {
                type_name,
                members: children,
                span,
            } => {
                if type_name == "Cell" {
                    // `Cell` is an IR-only Grid wrapper (DD-M3-P5-001): it
                    // is not a runtime widget kind and is only valid as a
                    // direct child of a `Grid`. Its intra-cell validation
                    // (single child, placement / span / alignment, unknown
                    // attributes) needs Grid context (track counts, cell
                    // count) and is performed by the enclosing Grid's
                    // `check_grid` pass; here we only reject a `Cell` that
                    // appears outside a `Grid`. The unknown-widget warning
                    // is intentionally skipped (Cell is known, just
                    // IR-only). Recurse into the Cell's content so nested
                    // widgets are still checked.
                    if enclosing_widget != Some("Grid") {
                        check_cell_outside_grid(enclosing_widget, span, filename, diags);
                    }
                    check_members_inner(
                        children,
                        Some("Cell"),
                        enclosing_widget,
                        filename,
                        ns,
                        diags,
                        loop_ctx,
                        inside_for_template,
                        all_loop_binders,
                    );
                } else {
                    if !KNOWN_WIDGET_TYPES.contains(&type_name.as_str()) {
                        diags.push(Diagnostic::warning(
                            filename,
                            span.line,
                            span.col,
                            format!(
                                "unknown widget type `{}`; known types: {}",
                                type_name,
                                KNOWN_WIDGET_TYPES.join(", ")
                            ),
                        ));
                    }
                    if type_name == "Box" {
                        check_box_child_count(children, span, filename, diags);
                    }
                    if type_name == "WrapPanel" {
                        check_wrappanel_aspect_only_box_warning(children, span, filename, diags);
                    }
                    if type_name == "ScrollView" {
                        check_scrollview_child_count(children, span, filename, diags);
                    }
                    if type_name == "Grid" {
                        check_grid(children, span, filename, diags);
                    }
                    if type_name == "Button" || type_name == "ToggleButton" {
                        check_button_family_children(type_name, children, span, filename, diags);
                    }
                    check_members_inner(
                        children,
                        Some(type_name),
                        enclosing_widget,
                        filename,
                        ns,
                        diags,
                        loop_ctx,
                        inside_for_template,
                        all_loop_binders,
                    );
                }
            }

            Member::SignalHandler {
                signal,
                arg,
                body,
                span,
            } => {
                if inside_for_template {
                    diags.push(error(
                        filename,
                        span,
                        "handlers inside a `for` body template are deferred in M3-Phase 7; put mutation handlers outside the `for` body",
                    ));
                }
                // `dismiss` is admitted only on a container carrying
                // `modal-scope: true` (dsl_spec §4.19); written anywhere
                // else it could never be raised, so it is rejected here
                // rather than silently never firing. Fires at component
                // level too — a component member list never legitimately
                // carries `modal-scope`.
                if signal == "dismiss" && !carries_modal_scope {
                    diags.push(error(
                        filename,
                        span,
                        "`dismiss` handler can never be raised: a dismissal request is addressed to a modal scope; write `modal-scope: true` on the same container or remove the handler (dsl_spec §4.19)",
                    ));
                }
                // M4-Phase 2 T8: `key-down` argument rules (dsl_spec
                // §4.19 "Keyboard input"). A `key-down` handler with no
                // argument can never fire — the same "silently never
                // fires" class the `dismiss` rule above guards against —
                // and an argument naming an unrecognised key is rejected
                // here rather than reaching the runtime unfireable.
                // `key-down` is the only signal dsl_spec §4.19 defines
                // with an argument, so any other signal carrying one is
                // rejected too. Applies on every widget kind — there is
                // no per-kind signal admission left to gate against
                // (removed by the previous stage).
                if signal == "key-down" {
                    match arg {
                        None => diags.push(error(
                            filename,
                            span,
                            "`key-down` handler can never be raised: the key must be named in the declaration, e.g. `key-down(\"ArrowLeft\")` (dsl_spec §4.19)",
                        )),
                        Some(key) if !crate::ir::is_recognised_key_name(key) => {
                            diags.push(error(
                                filename,
                                span,
                                format!(
                                    "`key-down(\"{key}\")` names an unrecognised key; recognised keys are the named non-character keys per dsl_spec §4.19 (`Escape`, the arrow / Home / End / Page keys, `Enter`, `F1`-`F12`)"
                                ),
                            ));
                        }
                        Some(_) => {}
                    }
                } else if arg.is_some() {
                    diags.push(error(
                        filename,
                        span,
                        format!(
                            "`{signal}` does not take an argument; only `key-down` does (dsl_spec §4.19)"
                        ),
                    ));
                }
                for stmt in &body.statements {
                    check_block_statement(stmt, filename, ns, diags, loop_ctx, all_loop_binders);
                }
            }

            Member::Conditional {
                condition,
                body,
                span,
            } => {
                if enclosing_widget.is_none() {
                    diags.push(error(
                        filename,
                        span,
                        "component-level `if` is not supported in M3-Phase 6; put the `if` inside a widget body",
                    ));
                }
                check_if_condition(condition, filename, ns, loop_ctx, diags);
                check_if_body(body, span, filename, diags);
                check_members_inner(
                    body,
                    enclosing_widget,
                    parent_widget,
                    filename,
                    ns,
                    diags,
                    loop_ctx,
                    inside_for_template,
                    all_loop_binders,
                );
            }

            Member::For {
                binder,
                index_binder,
                collection,
                body,
                span,
            } => {
                check_for_member(
                    binder,
                    index_binder.as_deref(),
                    collection,
                    body,
                    span,
                    enclosing_widget,
                    filename,
                    ns,
                    diags,
                    inside_for_template,
                );
                let elem = match collection {
                    Expr::Ident { name, .. } => match ns.get(name) {
                        Some(TypeName::Collection(elem)) => *elem,
                        _ => CollectionElemType::Int,
                    },
                    _ => CollectionElemType::Int,
                };
                let child_ctx = LoopContext {
                    binder,
                    index_binder: index_binder.as_deref(),
                    elem,
                };
                check_members_inner(
                    body,
                    enclosing_widget,
                    parent_widget,
                    filename,
                    ns,
                    diags,
                    Some(&child_ctx),
                    true,
                    all_loop_binders,
                );
            }

            // Grid track-list members are validated by the enclosing
            // Grid's `check_grid` pass (which needs all of the Grid's
            // members together — track counts feed Cell placement bounds).
            // The narrow parser path only emits this variant inside a Grid
            // body, so reaching it here during the generic recursion is
            // always under `enclosing_widget == Some("Grid")` and is a
            // no-op to avoid double diagnostics.
            Member::GridTracks { .. } => {}
        }
    }
}

fn check_property_bind_target_in_context(
    enclosing_widget: Option<&str>,
    prop_name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    ns: &Namespace,
    loop_ctx: Option<&LoopContext<'_>>,
    inside_for_template: bool,
    diags: &mut Vec<Diagnostic>,
) {
    let Some(widget) = enclosing_widget else {
        return;
    };
    let Some(target_ty) = widget_prop_type(widget, prop_name) else {
        return;
    };
    let Some(source_ty) = expr_static_type_in_context(value, ns, loop_ctx) else {
        if inside_for_template {
            if let Expr::Ident { name, span } = value {
                if !is_loop_local_ident(name, loop_ctx) && !ns.contains_key(name) {
                    diags.push(error(
                        filename,
                        span,
                        format!(
                            "identifier `{}` is not a declared state or loop binder in this `for` body",
                            name
                        ),
                    ));
                }
            }
        }
        return;
    };
    if matches!(source_ty, TypeName::Float) {
        return;
    }
    if matches!(source_ty, TypeName::Collection(_)) {
        // A collection in a scalar property position is the loop-external
        // read deferral; `check_expr_type[_in_loop_context]` owns that
        // diagnostic. Avoid emitting a redundant type-mismatch for the same
        // expression.
        return;
    }
    if !types_compatible(&target_ty, &source_ty) {
        diags.push(error(
            filename,
            span,
            format!(
                "type mismatch in binding `{}.{}`: target is `{}`, source is `{}`",
                widget,
                prop_name,
                type_name_display(&target_ty),
                type_name_display(&source_ty),
            ),
        ));
    }
}

fn check_block_statement(
    stmt: &BlockStatement,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
    loop_ctx: Option<&LoopContext<'_>>,
    all_loop_binders: &[String],
) {
    match stmt {
        BlockStatement::Assignment(stmt) => {
            check_qualified_name(&stmt.target, filename, ns, diags);
            let lhs_name = stmt
                .target
                .segments
                .last()
                .map(String::as_str)
                .unwrap_or("");
            match ns.get(lhs_name) {
                Some(TypeName::Collection(elem)) => {
                    if stmt.target.segments.len() != 1 {
                        diags.push(error(
                            filename,
                            &stmt.target.span,
                            "collection mutation requires a local state name; qualified collection assignment is deferred",
                        ));
                    }
                    if !matches!(stmt.op, crate::ast::AssignOp::Eq) {
                        diags.push(error(
                            filename,
                            &stmt.span,
                            "compound assignment on collection states is not supported in M3-Phase 7; use `xs = xs.append(value)` or `xs = xs.drop-last()`",
                        ));
                        return;
                    }
                    check_collection_assignment_rhs(
                        lhs_name,
                        *elem,
                        &stmt.value,
                        &stmt.span,
                        filename,
                        ns,
                        loop_ctx,
                        diags,
                    );
                }
                Some(_) | None => {
                    if is_collection_expr(&stmt.value) {
                        diags.push(error(
                            filename,
                            stmt.value.span(),
                            "collection expressions are valid only as the RHS of an assignment to a collection state",
                        ));
                    } else {
                        check_expr_type_in_loop_context(
                            &stmt.value,
                            &stmt.span,
                            filename,
                            ns,
                            loop_ctx,
                            false,
                            all_loop_binders,
                            diags,
                        );
                    }
                }
            }
        }
        BlockStatement::Expr(expr_stmt) => {
            if is_collection_expr(&expr_stmt.value) {
                diags.push(error(
                    filename,
                    &expr_stmt.span,
                    "collection expressions are not statements in M3-Phase 7; assign them back to the collection state",
                ));
            } else {
                check_expr_type_in_loop_context(
                    &expr_stmt.value,
                    &expr_stmt.span,
                    filename,
                    ns,
                    loop_ctx,
                    false,
                    all_loop_binders,
                    diags,
                );
            }
        }
    }
}

fn is_collection_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::ListLit { .. } | Expr::CollectionCall { .. })
}

fn names_collection_state(name: &str, ns: &Namespace) -> bool {
    matches!(ns.get(name), Some(TypeName::Collection(_)))
}

/// DD-M3-P7-007 loop-external read row: a read that navigates a collection
/// state outside the `for` header / loop-local binder path (bare name,
/// whole-value qualified read, or member navigation such as `xs.length`) is a
/// recorded deferral, not a scalar read. Returns the offending collection
/// segment if any segment of the qualified name resolves to a collection
/// state. The `for` header collection and the collection-assignment RHS are
/// validated on their own paths and never reach this helper.
fn collection_external_read_segment<'a>(qn: &'a QualifiedName, ns: &Namespace) -> Option<&'a str> {
    qn.segments
        .iter()
        .map(String::as_str)
        .find(|seg| names_collection_state(seg, ns))
}

const COLLECTION_EXTERNAL_READ_HINT: &str = "collection reads outside iteration not yet supported";

#[allow(clippy::too_many_arguments)]
fn check_collection_assignment_rhs(
    lhs_name: &str,
    elem: CollectionElemType,
    rhs: &Expr,
    span: &Span,
    filename: &str,
    ns: &Namespace,
    loop_ctx: Option<&LoopContext<'_>>,
    diags: &mut Vec<Diagnostic>,
) {
    match rhs {
        Expr::ListLit { .. } => {
            check_collection_literal(rhs, elem, span, filename, "assignment RHS", diags);
        }
        Expr::CollectionCall {
            receiver,
            method,
            args,
            ..
        } => {
            if receiver.segments.len() != 1 || receiver.segments[0] != lhs_name {
                diags.push(error(
                    filename,
                    &receiver.span,
                    "collection assignment RHS must use the same local collection as its receiver; general collection expressions are deferred",
                ));
                return;
            }
            match method.as_str() {
                "append" => {
                    if args.len() != 1 {
                        diags.push(error(
                            filename,
                            rhs.span(),
                            "`append` takes exactly one element argument",
                        ));
                        return;
                    }
                    let Some(arg_ty) = expr_static_type_in_context(&args[0], ns, loop_ctx) else {
                        diags.push(error(
                            filename,
                            args[0].span(),
                            "append element must be a scalar expression with a known type",
                        ));
                        return;
                    };
                    if !types_compatible(&collection_elem_as_type(elem), &arg_ty) {
                        diags.push(error(
                            filename,
                            args[0].span(),
                            format!(
                                "`append` element type mismatch: collection `{}` expects `{}`, got `{}`",
                                lhs_name,
                                collection_elem_display(elem),
                                type_name_display(&arg_ty),
                            ),
                        ));
                    }
                }
                "drop-last" => {
                    if !args.is_empty() {
                        diags.push(error(
                            filename,
                            rhs.span(),
                            "`drop-last` takes no arguments in M3-Phase 7",
                        ));
                    }
                }
                _ => diags.push(error(
                    filename,
                    rhs.span(),
                    format!(
                        "unknown collection method `{}`; M3-Phase 7 supports `append` and `drop-last` only",
                        method
                    ),
                )),
            }
        }
        Expr::Ident { .. } | Expr::QualifiedRef { .. } => diags.push(error(
            filename,
            rhs.span(),
            "bare collection copies are deferred; collection assignment RHS must be a self-receiver `append` / `drop-last` call or a static list literal",
        )),
        _ => diags.push(error(
            filename,
            rhs.span(),
            "collection state assignment requires a collection expression RHS",
        )),
    }
}

/// Validate expression: reject unsupported types; resolve name references against namespace.
fn check_expr_type(
    expr: &Expr,
    ctx_span: &Span,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        Expr::IntLit { .. }
        | Expr::StringLit { .. }
        | Expr::BoolLit { .. }
        | Expr::Ident { .. } => {
            if let Expr::Ident { name, span } = expr {
                // A bare collection-state read outside the `for` header /
                // loop-local binder path is the DD-M3-P7-007 loop-external
                // read deferral, not a scalar keyword value (DD-002 / Q5).
                if names_collection_state(name, ns) {
                    diags.push(error(
                        filename,
                        span,
                        format!(
                            "{COLLECTION_EXTERNAL_READ_HINT}; `{}` is a collection — read its elements through a `for` binder",
                            name
                        ),
                    ));
                }
                // Otherwise: keyword-valued idents (e.g. mica, system, accent,
                // title) are not state refs and pass through. Plain single
                // idents are ambiguous (could be an enum/keyword value); we do
                // not reject them here.
            }
            // String interpolation parts: check that Interp segments resolve
            // to declared state and stay within the currently supported
            // interpolation value types.
            if let Expr::StringLit { parts, .. } = expr {
                for part in parts {
                    if let crate::ast::StringPart::Interp(qn) = part {
                        if let Some(seg) = collection_external_read_segment(qn, ns) {
                            diags.push(error(
                                filename,
                                &qn.span,
                                format!(
                                    "{COLLECTION_EXTERNAL_READ_HINT}; `{}` is a collection — read its elements through a `for` binder",
                                    seg
                                ),
                            ));
                            continue;
                        }
                        check_qualified_name(qn, filename, ns, diags);
                        check_string_interpolation_type(qn, filename, ns, diags);
                    }
                }
            }
        }
        Expr::FloatLit { span, .. } => {
            diags.push(error(
                filename,
                span,
                "float literals are not supported in M2 (only i32 and string)",
            ));
        }
        Expr::Measurement { span, .. } => {
            // Measurements (e.g. 12px) are static property values, not typed state — allowed.
            let _ = (ctx_span, span);
        }
        Expr::QualifiedRef { name } => {
            if let Some(seg) = collection_external_read_segment(name, ns) {
                diags.push(error(
                    filename,
                    &name.span,
                    format!(
                        "{COLLECTION_EXTERNAL_READ_HINT}; `{}` is a collection — read its elements through a `for` binder",
                        seg
                    ),
                ));
                return;
            }
            check_qualified_name(name, filename, ns, diags);
        }
        Expr::ListLit { span, .. } | Expr::CollectionCall { span, .. } => {
            diags.push(error(
                filename,
                span,
                "collection expressions are valid only in collection state defaults or collection-assignment RHS positions",
            ));
        }
        // Ratio / Color literals are only valid as `Box.aspect` / `Box.fill`
        // RHS (dsl_spec §4.9). Every other syntactic position — state
        // default, handler RHS, non-Box property assignment, nested
        // expression — is a compile-time reject per DD-M3-P2-002 /
        // DD-M3-P2-003 (literal plumbing, not bindable surface). The
        // accepted-position arm is dispatched in `check_members_inner`
        // before this path is taken, so reaching here means the literal
        // appeared outside its accepted position.
        Expr::RatioLit { span, .. } => {
            diags.push(error(
                filename,
                span,
                "ratio literal is only valid as the RHS of `Box.aspect` in M3-Phase 2",
            ));
        }
        Expr::ColorLit { span, .. } => {
            diags.push(error(
                filename,
                span,
                "color literal is only valid as the RHS of `Box.fill` in M3-Phase 2",
            ));
        }
        Expr::UnsupportedOperator { op, span } => {
            diags.push(error(
                filename,
                span,
                format!(
                    "operator {} is not part of the M3-Phase 6 expression surface",
                    op
                ),
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_expr_type_in_loop_context(
    expr: &Expr,
    ctx_span: &Span,
    filename: &str,
    ns: &Namespace,
    loop_ctx: Option<&LoopContext<'_>>,
    inside_for_template: bool,
    all_loop_binders: &[String],
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        Expr::Ident { name, span } => {
            if let Some(ctx) = loop_ctx {
                if name == ctx.binder || ctx.index_binder == Some(name.as_str()) {
                    return;
                }
            }
            if all_loop_binders.iter().any(|binder| binder == name) {
                diags.push(error(
                    filename,
                    span,
                    format!(
                        "loop binder `{}` may be read only inside its `for` body expression bindings",
                        name
                    ),
                ));
                return;
            }
            let _ = inside_for_template;
            check_expr_type(expr, ctx_span, filename, ns, diags);
        }
        Expr::QualifiedRef { name } => {
            if let Some(segment) = qualified_loop_local_segment(name, loop_ctx) {
                diags.push(error(
                    filename,
                    &name.span,
                    format!(
                        "qualified loop-local binder read `{}` is deferred in M3-Phase 7; `{}` is a scalar loop value, not a structured item field",
                        name.segments.join("."),
                        segment
                    ),
                ));
                return;
            }
            if let Some(seg) = collection_external_read_segment(name, ns) {
                diags.push(error(
                    filename,
                    &name.span,
                    format!(
                        "{COLLECTION_EXTERNAL_READ_HINT}; `{}` is a collection — read its elements through a `for` binder",
                        seg
                    ),
                ));
                return;
            }
            check_qualified_name(name, filename, ns, diags);
        }
        Expr::StringLit { parts, .. } => {
            for part in parts {
                if let crate::ast::StringPart::Interp(qn) = part {
                    if let Some(segment) = qualified_loop_local_segment(qn, loop_ctx) {
                        diags.push(error(
                            filename,
                            &qn.span,
                            format!(
                                "qualified loop-local binder read `{}` is deferred in M3-Phase 7; `{}` is a scalar loop value, not a structured item field",
                                qn.segments.join("."),
                                segment
                            ),
                        ));
                        continue;
                    }
                    let state_name = qn.segments.last().map(String::as_str).unwrap_or("");
                    if let Some(ctx) = loop_ctx {
                        if state_name == ctx.binder {
                            // A bool element rendered through interpolation has
                            // no defined display conversion, mirroring the
                            // scalar bool-in-interpolation reject. The index
                            // binder is always `i32` and stays admissible.
                            if ctx.elem == CollectionElemType::Bool {
                                diags.push(error(
                                    filename,
                                    &qn.span,
                                    format!(
                                        "bool loop binder `{}` cannot be used in string interpolation; \
                                         bool formatting/display conversion is not defined in M3-Phase 7",
                                        state_name
                                    ),
                                ));
                            }
                            continue;
                        }
                        if ctx.index_binder == Some(state_name) {
                            continue;
                        }
                    }
                    if let Some(seg) = collection_external_read_segment(qn, ns) {
                        diags.push(error(
                            filename,
                            &qn.span,
                            format!(
                                "{COLLECTION_EXTERNAL_READ_HINT}; `{}` is a collection — read its elements through a `for` binder",
                                seg
                            ),
                        ));
                        continue;
                    }
                    check_qualified_name(qn, filename, ns, diags);
                    check_string_interpolation_type(qn, filename, ns, diags);
                }
            }
        }
        Expr::ListLit { .. } | Expr::CollectionCall { .. } => {
            diags.push(error(
                filename,
                expr.span(),
                "collection expressions are valid only in collection state defaults or collection-assignment RHS positions",
            ));
        }
        _ => check_expr_type(expr, ctx_span, filename, ns, diags),
    }
}

#[allow(clippy::too_many_arguments)]
fn check_for_member(
    binder: &str,
    index_binder: Option<&str>,
    collection: &Expr,
    body: &[Member],
    span: &Span,
    enclosing_widget: Option<&str>,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
    inside_for_template: bool,
) {
    if inside_for_template {
        diags.push(error(
            filename,
            span,
            "nested `for` is deferred in M3-Phase 7; wrap or flatten the iteration source in a later phase",
        ));
    }
    match enclosing_widget {
        None => diags.push(error(
            filename,
            span,
            "component-level `for` is not admitted in M3-Phase 7; place it inside an admitted widget container",
        )),
        Some("ScrollView") => diags.push(error(
            filename,
            span,
            "direct `for` is not valid in ScrollView; wrap it in a content widget such as `WrapPanel`",
        )),
        Some("Box") => diags.push(error(
            filename,
            span,
            "direct `for` is not valid in Box because Box admits at most one child",
        )),
        Some("Grid") | Some("Cell") => diags.push(error(
            filename,
            span,
            "direct `for` is not valid in Grid placement contexts in M3-Phase 7",
        )),
        _ => {}
    }

    if ns.contains_key(binder) {
        diags.push(error(
            filename,
            span,
            format!(
                "loop binder `{}` collides with a declared state name",
                binder
            ),
        ));
    }
    if let Some(index) = index_binder {
        if index == binder {
            diags.push(error(
                filename,
                span,
                "loop element binder and index binder must be distinct",
            ));
        }
        if ns.contains_key(index) {
            diags.push(error(
                filename,
                span,
                format!(
                    "loop index binder `{}` collides with a declared state name",
                    index
                ),
            ));
        }
    }

    match collection {
        Expr::Ident { name, span } => match ns.get(name) {
            Some(TypeName::Collection(_)) => {}
            Some(other) => diags.push(error(
                filename,
                span,
                format!(
                    "`for` target `{}` must be a collection state; it is declared `{}`",
                    name,
                    type_name_display(other)
                ),
            )),
            None => diags.push(error(
                filename,
                span,
                format!("`for` target `{}` is not declared as a collection state", name),
            )),
        },
        Expr::QualifiedRef { name } => diags.push(error(
            filename,
            &name.span,
            "`for` collection must be a local state name; qualified collection references are deferred",
        )),
        _ => diags.push(error(
            filename,
            collection.span(),
            "`for` collection expressions are not yet supported; use a local collection state name after `in`",
        )),
    }

    check_for_body(body, span, filename, diags);
}

fn check_for_body(body: &[Member], span: &Span, filename: &str, diags: &mut Vec<Diagnostic>) {
    let widget_count = body
        .iter()
        .filter(|m| matches!(m, Member::WidgetDecl { .. }))
        .count();
    if body.len() != 1 || widget_count != 1 {
        diags.push(error(
            filename,
            span,
            "`for` body admits exactly one widget child in M3-Phase 7; wrap multiple widgets or control flow in a container",
        ));
    }
    for member in body {
        match member {
            Member::WidgetDecl { .. } => {}
            Member::Conditional { span, .. } | Member::For { span, .. } => diags.push(error(
                filename,
                span,
                "bare control flow is not admitted directly as a `for` body; wrap it in a widget container",
            )),
            Member::PropertyBind { span, .. }
            | Member::PropertyDecl { span, .. }
            | Member::SignalHandler { span, .. }
            | Member::StateMember { span, .. }
            | Member::GridTracks { span, .. } => diags.push(error(
                filename,
                span,
                "`for` body admits only a single widget child; properties, handlers, state declarations, and track lists are not structural body members",
            )),
        }
    }
}

fn check_string_interpolation_type(
    qn: &QualifiedName,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
) {
    if qn.segments.is_empty() {
        return;
    }
    let state_name = qn.segments.last().unwrap();
    if matches!(ns.get(state_name), Some(TypeName::Bool)) {
        diags.push(error(
            filename,
            &qn.span,
            format!(
                "bool state `{}` cannot be used in string interpolation; \
                 bool formatting/display conversion is not defined in M3-Phase 1",
                state_name
            ),
        ));
    }
}

/// Validate that every segment of a qualified name resolves to a declared state.
fn check_qualified_name(
    qn: &QualifiedName,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
) {
    // In the counter DSL, state references appear as `root.count` — the first
    // segment is `root` (the component root alias) and the second is the state name.
    // For M2 flat namespace we resolve the last segment as the state name.
    if qn.segments.is_empty() {
        return;
    }
    let state_name = qn.segments.last().unwrap();
    if !ns.contains_key(state_name) {
        diags.push(error(
            filename,
            &qn.span,
            format!(
                "undefined state `{}`; declare it with `state {}: <type> = <default>`",
                state_name, state_name
            ),
        ));
    }
}

fn error(filename: &str, span: &Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(filename, span.line, span.col, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn check_src(src: &str) -> CheckResult {
        let tokens = tokenize(src, "<test>").unwrap();
        let ast = parse(&tokens, "<test>").unwrap();
        check(&ast, "<test>")
    }

    fn errors(src: &str) -> Vec<String> {
        check_src(src)
            .diagnostics
            .into_iter()
            .filter(|d| d.severity == crate::diagnostic::Severity::Error)
            .map(|d| d.message)
            .collect()
    }

    fn warnings(src: &str) -> Vec<String> {
        check_src(src)
            .diagnostics
            .into_iter()
            .filter(|d| d.severity == crate::diagnostic::Severity::Warning)
            .map(|d| d.message)
            .collect()
    }

    #[test]
    fn duplicate_state_name() {
        let errs = errors("component C inherits W { state count: i32 = 0 state count: i32 = 1 }");
        assert_eq!(errs.len(), 1);
        assert!(
            errs[0].contains("duplicate state name `count`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn undefined_state_in_handler() {
        let errs = errors(
            "component C inherits W { state count: i32 = 0 clicked => { root.missing += 1; } }",
        );
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("undefined state `missing`"), "{:?}", errs);
    }

    #[test]
    fn undefined_state_in_string_interp() {
        let errs = errors(
            r#"component C inherits W { state count: i32 = 0 VStack { text: "val: \{root.missing}" } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(errs[0].contains("undefined state `missing`"), "{:?}", errs);
    }

    #[test]
    fn float_literal_rejected() {
        let errs = errors("component C inherits W { VStack { spacing: 1.5 } }");
        assert_eq!(errs.len(), 1);
        assert!(
            errs[0].contains("float literals are not supported"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn valid_state_and_handler_ok() {
        let result = check_src(
            "component C inherits W { state count: i32 = 0 clicked => { root.count += 1; } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn valid_string_interp_resolves() {
        let result = check_src(
            r#"component C inherits W { state count: i32 = 0 VStack { text: "x: \{root.count}" } }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn collection_state_default_and_for_body_accepted() {
        let result = check_src(
            r#"component C inherits W {
                state labels: string[] = ["a", "b"]
                WrapPanel { for label, i in labels { Text { text: label } } }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn collection_assignment_forms_accepted() {
        let result = check_src(
            r#"component C inherits W {
                state xs: i32[] = []
                state flags: bool[] = []
                Button { clicked => { xs = xs.append(2); xs = xs.drop-last(); xs = []; flags = []; } }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn gallery_like_for_shape_and_body_external_handlers_accepted() {
        let result = check_src(
            r##"component C inherits W {
                state labels: string[] = ["S01", "S02"]
                VStack {
                    ScrollView {
                        WrapPanel {
                            for label, i in labels {
                                Box { aspect: 1:1 fill: #334455 Text { text: "Thumb \{label} #\{i}" } }
                            }
                        }
                    }
                    Button { text: "Add" clicked => { labels = labels.append("S03"); } }
                    Button { text: "Remove" clicked => { labels = labels.drop-last(); } }
                }
            }"##,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn for_target_must_be_collection_state() {
        let errs = errors(
            "component C inherits W { state count: i32 = 0 WrapPanel { for x in count { Text {} } } }",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("must be a collection state")),
            "{errs:?}"
        );
    }

    #[test]
    fn for_target_must_be_declared() {
        let errs = errors("component C inherits W { WrapPanel { for x in missing { Text {} } } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("not declared as a collection")),
            "{errs:?}"
        );
    }

    #[test]
    fn for_target_rejects_qualified_reference() {
        let errs = errors(
            "component C inherits W { state xs: i32[] = [] WrapPanel { for x in root.xs { Text {} } } }",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("local state name") && e.contains("qualified")),
            "{errs:?}"
        );
    }

    #[test]
    fn for_target_rejects_collection_expression() {
        let cases = [
            "component C inherits W { state xs: i32[] = [] WrapPanel { for x in xs.append(1) { Text {} } } }",
            "component C inherits W { WrapPanel { for x in [1] { Text {} } } }",
        ];
        for src in cases {
            let errs = errors(src);
            assert!(
                errs.iter()
                    .any(|e| e.contains("collection expressions are not yet supported")),
                "{errs:?}"
            );
        }
    }

    #[test]
    fn for_binder_collisions_rejected() {
        let errs = errors(
            "component C inherits W { state xs: i32[] = [] state x: i32 = 0 WrapPanel { for x, x in xs { Text {} } } }",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("collides with a declared state")),
            "{errs:?}"
        );
        assert!(
            errs.iter().any(|e| e.contains("must be distinct")),
            "{errs:?}"
        );

        let index_state = errors(
            "component C inherits W { state xs: i32[] = [] state i: i32 = 0 WrapPanel { for x, i in xs { Text {} } } }",
        );
        assert!(
            index_state
                .iter()
                .any(|e| e.contains("loop index binder") && e.contains("collides")),
            "{index_state:?}"
        );
    }

    #[test]
    fn for_component_level_rejected() {
        let errs =
            errors("component C inherits W { state xs: i32[] = [] for x in xs { Text {} } }");
        assert!(
            errs.iter().any(|e| e.contains("component-level `for`")),
            "{errs:?}"
        );
    }

    #[test]
    fn for_disallowed_direct_containers_rejected() {
        let cases = [
            (
                "component C inherits W { state xs: i32[] = [] ScrollView { for x in xs { Text {} } } }",
                "ScrollView",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Box { for x in xs { Text {} } } }",
                "Box",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Grid { columns: 1* rows: 1* for x in xs { Cell { Text {} } } } }",
                "Grid",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 for x in xs { Text {} } } } }",
                "Grid placement contexts",
            ),
        ];
        for (src, needle) in cases {
            let errs = errors(src);
            assert!(
                errs.iter().any(|e| e.contains(needle)),
                "{needle}: {errs:?}"
            );
        }
    }

    #[test]
    fn for_is_admitted_inside_cell_descendant_container() {
        let result = check_src(
            "component C inherits W { state xs: i32[] = [] Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 WrapPanel { for x in xs { Text {} } } } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn for_body_shape_rejects_non_widget_multi_child_and_bare_control_flow() {
        let cases = [
            (
                "component C inherits W { state xs: i32[] = [] WrapPanel { for x in xs { text: \"x\" } } }",
                "only a single widget child",
            ),
            (
                "component C inherits W { state xs: i32[] = [] WrapPanel { for x in xs { Text {} Button {} } } }",
                "exactly one widget child",
            ),
            (
                "component C inherits W { state xs: i32[] = [] WrapPanel { for x in xs { if true { Text {} } } } }",
                "bare control flow",
            ),
        ];
        for (src, needle) in cases {
            let errs = errors(src);
            assert!(
                errs.iter().any(|e| e.contains(needle)),
                "{needle}: {errs:?}"
            );
        }
    }

    #[test]
    fn for_body_rejects_handler_and_nested_for_at_any_depth() {
        let handler_errs = errors(
            "component C inherits W { state xs: i32[] = [] WrapPanel { for x in xs { Button { clicked => { root.missing = 1; } } } } }",
        );
        assert!(
            handler_errs
                .iter()
                .any(|e| e.contains("handlers inside a `for` body")),
            "{handler_errs:?}"
        );
        let nested_errs = errors(
            "component C inherits W { state xs: i32[] = [] WrapPanel { for x in xs { VStack { for y in xs { Text {} } } } } }",
        );
        assert!(
            nested_errs.iter().any(|e| e.contains("nested `for`")),
            "{nested_errs:?}"
        );
    }

    #[test]
    fn loop_binder_reads_rejected_outside_handler_and_if_condition() {
        let outside = errors(
            "component C inherits W { state xs: i32[] = [] WrapPanel { for x in xs { Text {} } Text { text: x } } }",
        );
        assert!(
            outside
                .iter()
                .any(|e| e.contains("may be read only inside")),
            "{outside:?}"
        );
        let handler = errors(
            "component C inherits W { state xs: i32[] = [] Button { clicked => { root.count = x; } } WrapPanel { for x in xs { Text {} } } }",
        );
        assert!(
            handler
                .iter()
                .any(|e| e.contains("may be read only inside")),
            "{handler:?}"
        );
        let cond = errors(
            "component C inherits W { state xs: bool[] = [] WrapPanel { for x in xs { VStack { if x { Text {} } } } } }",
        );
        assert!(
            cond.iter()
                .any(|e| e.contains("loop binders in `if` conditions")),
            "{cond:?}"
        );
    }

    #[test]
    fn for_body_rejects_unknown_typed_binding_but_keeps_untyped_keyword_values() {
        let unknown = errors(
            "component C inherits W { state xs: string[] = [] WrapPanel { for x in xs { Text { text: missing } } } }",
        );
        assert!(
            unknown
                .iter()
                .any(|e| e.contains("not a declared state or loop binder")),
            "{unknown:?}"
        );

        let keyword_like = check_src(
            "component C inherits W { state xs: string[] = [] WrapPanel { for x in xs { Text { font: title text: x } } } }",
        );
        assert!(!keyword_like.has_errors(), "{:?}", keyword_like.diagnostics);
    }

    #[test]
    fn qualified_loop_local_reads_rejected_as_structured_item_deferral() {
        let cases = [
            r#"component C inherits W {
                state labels: string[] = []
                state field: string = ""
                WrapPanel { for label in labels { Text { text: label.field } } }
            }"#,
            r#"component C inherits W {
                state labels: string[] = []
                state field: string = ""
                WrapPanel { for label in labels { Text { text: "\{label.field}" } } }
            }"#,
            r#"component C inherits W {
                state labels: string[] = []
                WrapPanel { for label, i in labels { Text { text: "\{root.i}" } } }
            }"#,
        ];
        for src in cases {
            let errs = errors(src);
            assert!(
                errs.iter()
                    .any(|e| e.contains("qualified loop-local binder read")),
                "{errs:?}"
            );
        }
    }

    #[test]
    fn loop_external_collection_reads_rejected() {
        // DD-M3-P7-007 loop-external read row: bare name, whole-value
        // qualified read, member navigation, and a collection read in a
        // scalar assignment all reject with the named deferral instead of
        // being silently accepted or surfaced as a misleading "undefined
        // state" / type-mismatch diagnostic.
        let cases = [
            // bare collection ident in an (untyped) property position
            "component C inherits W { state xs: i32[] = [] Foo { bar: xs } }",
            // bare collection ident in a typed property position
            "component C inherits W { state xs: i32[] = [] Text { text: xs } }",
            // member navigation (`xs.length`) — previously "undefined state"
            "component C inherits W { state xs: i32[] = [] Text { text: xs.length } }",
            // whole-value qualified read (`root.xs`)
            "component C inherits W { state xs: i32[] = [] Text { text: root.xs } }",
            // bare collection ident inside string interpolation
            r#"component C inherits W { state xs: i32[] = [] Text { text: "\{xs}" } }"#,
            // member navigation inside string interpolation
            r#"component C inherits W { state xs: i32[] = [] Text { text: "\{xs.length}" } }"#,
            // collection read as a scalar handler RHS
            "component C inherits W { state n: i32 = 0 state xs: i32[] = [] Button { clicked => { n = xs; } } }",
            // loop-external read of the *iterated* collection inside the body
            "component C inherits W { state xs: i32[] = [] WrapPanel { for x in xs { Text { text: xs } } } }",
        ];
        for src in cases {
            let errs = errors(src);
            assert!(
                errs.iter()
                    .any(|e| e.contains("collection reads outside iteration")),
                "{src}: {errs:?}"
            );
        }
    }

    #[test]
    fn bool_loop_binder_in_interpolation_rejected() {
        // A bool element rendered through interpolation has no defined display
        // conversion — the same contract scalar bool states get, applied to
        // the loop binder so the bool surface cannot be smuggled in.
        let errs = errors(
            r#"component C inherits W { state flags: bool[] = [] WrapPanel { for f in flags { Text { text: "v=\{f}" } } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("bool loop binder") && e.contains("bool formatting")),
            "{errs:?}"
        );
        // Positive controls: a string element and the i32 index binder both
        // remain admissible in interpolation.
        let ok = check_src(
            r#"component C inherits W { state labels: string[] = [] WrapPanel { for label, i in labels { Text { text: "\{label} #\{i}" } } } }"#,
        );
        assert!(!ok.has_errors(), "{:?}", ok.diagnostics);
    }

    #[test]
    fn collection_declaration_literal_rejects_bad_shapes() {
        let cases = [
            (
                "component C inherits W { state xs: i32[] = 1 VStack {} }",
                "must be a list literal",
            ),
            (
                "component C inherits W { state x: i32 = [1] VStack {} }",
                "scalar state",
            ),
            (
                "component C inherits W { state xs: i32[] = [1, \"two\"] VStack {} }",
                "element type mismatch",
            ),
            (
                "component C inherits W { state xs: i32[] = [a] VStack {} }",
                "must be scalar literals",
            ),
            (
                "component C inherits W { state xs: i32[] = [[1]] VStack {} }",
                "nested list",
            ),
        ];
        for (src, needle) in cases {
            let errs = errors(src);
            assert!(
                errs.iter().any(|e| e.contains(needle)),
                "{needle}: {errs:?}"
            );
        }
    }

    #[test]
    fn collection_assignment_rejects_bad_shapes() {
        let cases = [
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { root.xs = xs.append(1); } } }",
                "local state name",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs += 1; } } }",
                "compound assignment",
            ),
            (
                "component C inherits W { state x: i32 = 0 Button { clicked => { x = [1]; } } }",
                "collection expressions are valid only",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { missing = xs.append(1); } } }",
                "collection expressions are valid only",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs = 1; } } }",
                "requires a collection expression RHS",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Text { text: xs.append(1) } }",
                "collection expressions are valid only",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs.append(1); } } }",
                "not statements",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs = xs.append(); } } }",
                "exactly one",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs = xs.append(missing); } } }",
                "known type",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs = xs.append(\"bad\"); } } }",
                "element type mismatch",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs = xs.drop-last(1); } } }",
                "takes no arguments",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs = xs.clear(); } } }",
                "unknown collection method",
            ),
            (
                "component C inherits W { state xs: i32[] = [] state ys: i32[] = [] Button { clicked => { xs = ys.append(1); } } }",
                "same local collection",
            ),
            (
                "component C inherits W { state xs: i32[] = [] Button { clicked => { xs = root.xs.append(1); } } }",
                "same local collection",
            ),
            (
                "component C inherits W { state xs: i32[] = [] state ys: i32[] = [] Button { clicked => { xs = ys; } } }",
                "bare collection copies",
            ),
        ];
        for (src, needle) in cases {
            let errs = errors(src);
            assert!(
                errs.iter().any(|e| e.contains(needle)),
                "{needle}: {errs:?}"
            );
        }
    }

    #[test]
    fn bool_state_in_string_interp_rejected() {
        let errs = errors(
            r#"component C inherits W { state ready: bool = true Text { text: "ready=\{root.ready}" } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("bool state `ready` cannot be used in string interpolation")
                && errs[0].contains("bool formatting/display conversion is not defined"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn namespace_collected() {
        let result =
            check_src("component C inherits W { state count: i32 = 0 state label: string = \"\" }");
        assert!(result.namespace.contains_key("count"));
        assert!(result.namespace.contains_key("label"));
        assert!(matches!(result.namespace["count"], TypeName::Int));
        assert!(matches!(result.namespace["label"], TypeName::Str));
    }

    #[test]
    fn bool_state_default_accepted() {
        let result = check_src("component C inherits W { state ready: bool = false }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn bool_state_default_int_literal_rejected() {
        let errs = errors("component C inherits W { state ready: bool = 0 }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("type mismatch in state default for `ready`")
                && errs[0].contains("declared `bool`")
                && errs[0].contains("got `i32`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn bool_state_default_string_literal_rejected() {
        let errs = errors(r#"component C inherits W { state ready: bool = "false" }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("declared `bool`") && errs[0].contains("got `string`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn i32_state_default_bool_literal_rejected() {
        let errs = errors("component C inherits W { state count: i32 = true }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("declared `i32`") && errs[0].contains("got `bool`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn string_state_default_bool_literal_rejected() {
        let errs = errors("component C inherits W { state label: string = true }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("declared `string`") && errs[0].contains("got `bool`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn bind_bool_target_bool_state_ident_accepted() {
        let result = check_src(
            "component C inherits W { state ready: bool = true Button { enabled: ready } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn bind_bool_target_bool_literal_accepted() {
        let result = check_src("component C inherits W { Button { enabled: true } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn bind_bool_target_int_literal_rejected() {
        let errs = errors("component C inherits W { Button { enabled: 1 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("type mismatch in binding `Button.enabled`")
                && errs[0].contains("target is `bool`")
                && errs[0].contains("source is `i32`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn bind_string_target_bool_literal_rejected() {
        let errs = errors("component C inherits W { Text { text: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("type mismatch in binding `Text.text`")
                && errs[0].contains("target is `string`")
                && errs[0].contains("source is `bool`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn bind_string_target_bool_state_ident_rejected() {
        let errs =
            errors("component C inherits W { state ready: bool = true Text { text: ready } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("type mismatch in binding `Text.text`")
                && errs[0].contains("target is `string`")
                && errs[0].contains("source is `bool`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn bind_bool_target_i32_state_ident_rejected() {
        let errs = errors("component C inherits W { state x: i32 = 5 Button { enabled: x } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("type mismatch in binding `Button.enabled`")
                && errs[0].contains("target is `bool`")
                && errs[0].contains("source is `i32`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn bind_unknown_property_no_type_check() {
        // `font: title` and `style: accent` are keyword-value idents on
        // properties not yet in the static catalog — must pass through.
        let result = check_src(
            r#"component C inherits W { Text { font: title } Button { style: accent } }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn component_level_host_attrs_accepted() {
        // Component-level host attributes are validated through the
        // host-attribute catalog, then lower to `IrComponent.host_props`.
        let result =
            check_src(r#"component C inherits W { title: "Counter" backdrop: mica VStack {} }"#);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn component_level_unknown_host_attr_rejected() {
        let errs = errors(r#"component C inherits W { foo: bar ZStack { Text {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("unknown host attribute `foo`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn component_level_host_binding_rejected() {
        let errs = errors(r#"component C inherits W { state s: string = "x" title: s ZStack {} }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("host attribute `title` is not bindable"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn component_level_host_title_non_string_literal_reports_string_requirement() {
        // A wrong-typed *static literal* on `title` is a different mistake
        // from a dynamic bind: it must report the string-literal requirement,
        // not the "not bindable" (dynamic-deferred) diagnostic.
        let errs = errors(r#"component C inherits W { title: 42 ZStack {} }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`title` must be a string literal")
                && !errs[0].contains("not bindable"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn component_level_host_backdrop_typed_literal_rejected() {
        // `backdrop` / `theme` take keyword/enum identifiers; a concrete typed
        // literal is rejected as a non-keyword value (not as a binding).
        let errs = errors(r#"component C inherits W { backdrop: 42 ZStack {} }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("does not accept a literal value")
                && !errs[0].contains("not bindable"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn bind_string_target_string_literal_accepted() {
        let result = check_src(r#"component C inherits W { Button { text: "Click" } }"#);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn unknown_widget_type_is_warning_not_error() {
        let result = check_src("component C inherits W { UnknownWidget {} }");
        assert!(!result.has_errors());
        let ws = warnings("component C inherits W { UnknownWidget {} }");
        assert_eq!(ws.len(), 1);
        assert!(ws[0].contains("unknown widget type"));
    }

    #[test]
    fn togglebutton_known_widget_and_attrs_accepted_without_warning() {
        let src = r#"component C inherits W {
            state on: bool = true
            ToggleButton {
                text: "Photos"
                style: accent
                enabled: true
                checked: on
                clicked => { on = false; }
            }
        }"#;
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(
            warnings(src).is_empty(),
            "ToggleButton must be known, warnings: {:?}",
            warnings(src)
        );
    }

    #[test]
    fn togglebutton_checked_absent_accepted() {
        let result = check_src(r#"component C inherits W { ToggleButton { text: "Albums" } }"#);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn component_level_checked_routes_to_host_attr_reject() {
        let errs = errors("component C inherits W { checked: true ToggleButton {} }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("unknown host attribute `checked`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn checked_on_button_rejected() {
        let errs = errors("component C inherits W { Button { checked: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`checked` is only valid on ToggleButton")
                && errs[0].contains("not `Button`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn checked_on_text_rejected() {
        let errs = errors("component C inherits W { Text { checked: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`checked` is only valid on ToggleButton")
                && errs[0].contains("not `Text`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn checked_on_other_widget_rejected() {
        let errs = errors("component C inherits W { Box { checked: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`checked` is only valid on ToggleButton")
                && errs[0].contains("not `Box`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn checked_on_scrollview_rejected_by_container_attr_gate() {
        let errs = errors("component C inherits W { ScrollView { checked: true Text {} } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("not a recognised ScrollView attribute"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn checked_on_zstack_rejected_by_container_attr_gate() {
        let errs = errors("component C inherits W { ZStack { checked: true Text {} } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("unknown ZStack attribute `checked`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn togglebutton_checked_non_bool_rhs_rejected() {
        let errs = errors("component C inherits W { ToggleButton { checked: 1 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("type mismatch in binding `ToggleButton.checked`")
                && errs[0].contains("target is `bool`")
                && errs[0].contains("source is `i32`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn togglebutton_checked_i32_state_rejected() {
        let errs = errors(
            "component C inherits W { state index: i32 = 0 ToggleButton { checked: index } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("type mismatch in binding `ToggleButton.checked`")
                && errs[0].contains("target is `bool`")
                && errs[0].contains("source is `i32`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn togglebutton_unknown_attr_rejected() {
        let errs = errors("component C inherits W { ToggleButton { selected: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("unknown ToggleButton attribute `selected`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn togglebutton_alpha_tab_band_shape_accepted() {
        let src = r#"component C inherits W {
            state all: bool = true
            state albums: bool = false
            state favorites: bool = false
            HStack {
                ToggleButton {
                    text: "All"
                    checked: all
                    clicked => { all = true; albums = false; favorites = false; }
                }
                ToggleButton {
                    text: "Albums"
                    checked: albums
                    clicked => { all = false; albums = true; favorites = false; }
                }
                ToggleButton {
                    text: "Favorites"
                    checked: favorites
                    clicked => { all = false; albums = false; favorites = true; }
                }
            }
        }"#;
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(warnings(src).is_empty(), "{:?}", warnings(src));
    }

    // --- M4-Phase 2 T8: Button-family child rejection (CF-1, owner
    // disposition 2026-08-07) ---

    #[test]
    fn button_with_widget_child_rejected() {
        let errs = errors(r#"component C inherits W { Button { Text { text: "ok" } } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`Button` admits no widget children")
                && errs[0].contains("text:")
                && errs[0].contains("dsl_spec §4.8"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn togglebutton_with_widget_child_rejected() {
        let errs = errors(r#"component C inherits W { ToggleButton { Text { text: "ok" } } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ToggleButton` admits no widget children")
                && errs[0].contains("dsl_spec §4.8"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn button_with_conditional_member_rejected() {
        // Measured (T8 start gate): a `Conditional` member's own position
        // inside a Button is not rejected by `check_if_body` (that pass
        // only validates the `if`'s own body) nor by any Button-specific
        // placement rule — before this rule, `Button { if c { Text {} } }`
        // passed unrejected. Mirrors `check_box_child_count`'s completeness
        // requirement: a conditional sibling must count toward the
        // child-materialising total, not just bare `WidgetDecl` members.
        let errs =
            errors("component C inherits W { state c: bool = true Button { if c { Text {} } } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("`Button` admits no widget children")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn button_with_for_member_rejected() {
        // Measured (T8 start gate): `check_for_member`'s enclosing-widget
        // match special-cases `None` / `ScrollView` / `Box` / `Grid` /
        // `Cell` only; `Some("Button")` falls through to its `_ => {}`
        // arm, so a direct `for` inside a Button was not rejected before
        // this rule.
        let errs = errors(
            "component C inherits W { state xs: i32[] = [] Button { for x in xs { Text {} } } }",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Button` admits no widget children")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn button_with_only_admitted_members_accepted() {
        // The control that bounds the narrowing: `text:` / `enabled:` /
        // a `clicked` handler / a `slot.*` bind are not child-materialising
        // members, so a legitimate Button must not be rejected.
        let src = r#"component C inherits W {
            state on: bool = true
            Grid {
                columns: 1*
                rows: 1*
                Button {
                    text: "Click"
                    enabled: on
                    slot.row: 0
                    slot.column: 0
                    clicked => { on = false; }
                }
            }
        }"#;
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    // --- T3: Box accept shapes (dsl_spec §4.9) ---

    #[test]
    fn box_empty_accepted() {
        let result = check_src("component C inherits W { Box {} }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(
            warnings("component C inherits W { Box {} }").is_empty(),
            "Box should be a known widget type, not warn"
        );
    }

    #[test]
    fn box_fill_only_accepted() {
        let result = check_src("component C inherits W { Box { fill: #cccccc } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn box_scrim_alpha_accepted() {
        let result = check_src("component C inherits W { Box { fill: #00000080 } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn box_aspect_only_accepted() {
        let result = check_src("component C inherits W { Box { aspect: 16:9 } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn box_placeholder_shape_accepted() {
        // dsl_spec §4.9 normative placeholder pattern (DD-M3-P2-006).
        // Members are separated by whitespace (no statement terminator
        // at member level), mirroring the M1 / M2 surface — see the
        // parser-side `box_image_placeholder_shape` test.
        let result = check_src(
            r#"component C inherits W {
                Box {
                    aspect: 1:1
                    fill: #cccccc
                    Text { text: "Photo 12" }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn box_one_child_accepted() {
        let result = check_src(
            r#"component C inherits W {
                Box {
                    aspect: 16:9
                    fill: #cccccc
                    Text { text: "x" }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    // --- T3: Box multi-child reject (DD-M3-P2-001) ---

    #[test]
    fn box_two_children_rejected() {
        let errs =
            errors(r#"component C inherits W { Box { Text { text: "a" } Text { text: "b" } } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`Box` admits at most one child")
                && errs[0].contains("found 2")
                && errs[0].contains("ZStack")
                && errs[0].contains("VStack")
                && errs[0].contains("HStack"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn box_three_children_rejected() {
        let errs = errors(
            r#"component C inherits W { Box { Text { text: "a" } Text { text: "b" } Text { text: "c" } } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(errs[0].contains("found 3"), "{:?}", errs);
    }

    #[test]
    fn box_attrs_do_not_count_as_children() {
        // aspect / fill PropertyBinds must not be miscounted as widget
        // children. A Box with two attrs and one child is still 1 child.
        let result = check_src(
            r#"component C inherits W {
                Box {
                    aspect: 1:1
                    fill: #cccccc
                    Text { text: "x" }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    // --- T3: aspect value-validity reject (DD-M3-P2-005) ---

    #[test]
    fn box_aspect_zero_numerator_rejected() {
        let errs = errors("component C inherits W { Box { aspect: 0:9 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("numerator must be positive")
                && errs[0].contains("got 0")
                && errs[0].contains("`Box.aspect`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn box_aspect_zero_denominator_rejected() {
        let errs = errors("component C inherits W { Box { aspect: 16:0 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("denominator must be positive")
                && errs[0].contains("got 0")
                && errs[0].contains("`Box.aspect`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn box_aspect_zero_both_sides_rejected() {
        let errs = errors("component C inherits W { Box { aspect: 0:0 } }");
        assert_eq!(errs.len(), 2, "{:?}", errs);
        assert!(
            errs.iter()
                .any(|e| e.contains("numerator must be positive")),
            "{:?}",
            errs
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("denominator must be positive")),
            "{:?}",
            errs
        );
    }

    // --- T3: `bind aspect:` / `bind fill:` reject (DD-M3-P2-004) ---

    #[test]
    fn box_aspect_state_ident_rejected() {
        // `aspect: <state-ident>` is the "bind aspect" surface that
        // DD-M3-P2-004 says is rejected (constant-only in Phase 2).
        let errs = errors("component C inherits W { state r: i32 = 0 Box { aspect: r } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`Box.aspect` is constant-only")
                && errs[0].contains("`Ratio`")
                && errs[0].contains("<num>:<den>"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn box_fill_state_ident_rejected() {
        let errs = errors("component C inherits W { state c: i32 = 0 Box { fill: c } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`Box.fill` is constant-only")
                && errs[0].contains("`Color`")
                && errs[0].contains("#RRGGBB"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn box_aspect_int_literal_rejected() {
        // Surface error: `aspect: 16` is not a ratio literal — must reject
        // with the constant-only diagnostic.
        let errs = errors("component C inherits W { Box { aspect: 16 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`Box.aspect` is constant-only"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn box_fill_string_literal_rejected() {
        let errs = errors(r##"component C inherits W { Box { fill: "#cccccc" } }"##);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`Box.fill` is constant-only"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn box_aspect_color_literal_rejected() {
        // Cross-type: aspect must be Ratio, not Color. The constant-only
        // diagnostic still fires because the literal kind does not match.
        let errs = errors("component C inherits W { Box { aspect: #cccccc } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`Box.aspect` is constant-only"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn box_fill_ratio_literal_rejected() {
        let errs = errors("component C inherits W { Box { fill: 16:9 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`Box.fill` is constant-only"),
            "{:?}",
            errs
        );
    }

    // --- T3: Ratio / Color literal positional reject (dsl_spec §4.9) ---

    #[test]
    fn ratio_literal_in_state_default_rejected() {
        let errs = errors("component C inherits W { state r: i32 = 16:9 }");
        assert!(
            errs.iter()
                .any(|e| e.contains("ratio literal is only valid as the RHS of `Box.aspect`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn color_literal_in_state_default_rejected() {
        let errs = errors("component C inherits W { state c: i32 = #cccccc }");
        assert!(
            errs.iter()
                .any(|e| e.contains("color literal is only valid as the RHS of `Box.fill`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn ratio_literal_in_handler_rejected() {
        let errs = errors(
            "component C inherits W { state count: i32 = 0 clicked => { root.count = 16:9; } }",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("ratio literal is only valid as the RHS of `Box.aspect`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn color_literal_in_non_box_prop_rejected() {
        let errs = errors("component C inherits W { Text { text: #cccccc } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("color literal is only valid as the RHS of `Box.fill`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn ratio_literal_on_non_box_widget_rejected() {
        // VStack does not have aspect/fill — RatioLit must be rejected
        // positionally even when its property name happens to be "aspect".
        let errs = errors("component C inherits W { VStack { aspect: 16:9 } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("ratio literal is only valid as the RHS of `Box.aspect`")),
            "{:?}",
            errs
        );
    }

    // Regression: T3 routes state-default expressions through
    // `check_expr_type` (so Ratio / Color positional reject fires in
    // that position). As a side effect, every `check_expr_type` branch
    // — including StringLit interpolation validation — applies to
    // state defaults. The two tests below pin the StringLit branch so a
    // future regression that narrows the state-default `check_expr_type`
    // call would also drop these cases.

    #[test]
    fn string_state_default_with_undefined_interp_rejected() {
        let errs =
            errors(r#"component C inherits W { state label: string = "val: \{root.missing}" }"#);
        assert!(
            errs.iter().any(|e| e.contains("undefined state `missing`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn string_state_default_with_bool_interp_rejected() {
        let errs = errors(
            r#"component C inherits W { state ready: bool = true state label: string = "v: \{root.ready}" }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("bool state `ready` cannot be used in string interpolation")),
            "{:?}",
            errs
        );
    }

    // --- T1: WrapPanel accept shapes (dsl_spec §4.10) ---

    #[test]
    fn wrappanel_known_widget_no_warning() {
        // WrapPanel is in KNOWN_WIDGET_TYPES — no "unknown widget" warning.
        let result = check_src("component C inherits W { WrapPanel {} }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(
            warnings("component C inherits W { WrapPanel {} }").is_empty(),
            "WrapPanel should be a known widget type, not warn"
        );
    }

    #[test]
    fn wrappanel_zero_child_accepted() {
        let result = check_src("component C inherits W { WrapPanel {} }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn wrappanel_one_child_accepted() {
        let result = check_src(
            r#"component C inherits W {
                WrapPanel {
                    Box { aspect: 1:1 fill: #cccccc }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn wrappanel_multi_child_accepted() {
        let result = check_src(
            r#"component C inherits W {
                WrapPanel {
                    Box { aspect: 1:1 fill: #cccccc }
                    Box { aspect: 1:1 fill: #cccccc }
                    Box { aspect: 1:1 fill: #cccccc }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn wrappanel_with_item_cross_size_accepted() {
        let result = check_src("component C inherits W { WrapPanel { item-cross-size: 88 } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn wrappanel_with_item_spacing_accepted() {
        let result = check_src("component C inherits W { WrapPanel { item-spacing: 12 } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn wrappanel_with_line_spacing_accepted() {
        let result = check_src("component C inherits W { WrapPanel { line-spacing: 12 } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn wrappanel_zero_values_accepted() {
        // Zero is a valid setting on all three attributes (DD-M3-P3-006
        // zero-handling); the rejection threshold is `< 0`, not `<= 0`.
        let result = check_src(
            "component C inherits W { WrapPanel { item-cross-size: 0 item-spacing: 0 line-spacing: 0 } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn wrappanel_full_accept_shape() {
        // dsl_spec §4.10 wireframe-fidelity shape: all three attributes
        // set, multi-child WrapPanel of 1:1 thumbnails.
        let result = check_src(
            r#"component C inherits W {
                WrapPanel {
                    item-cross-size: 88
                    item-spacing: 12
                    line-spacing: 12
                    Box { aspect: 1:1 fill: #cccccc }
                    Box { aspect: 1:1 fill: #cccccc }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    // --- T1: WrapPanel negative-literal reject (DD-M3-P3-006) ---

    #[test]
    fn wrappanel_negative_item_cross_size_rejected() {
        let errs = errors("component C inherits W { WrapPanel { item-cross-size: -1 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-cross-size`")
                && errs[0].contains("non-negative")
                && errs[0].contains("got -1"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_negative_item_spacing_rejected() {
        let errs = errors("component C inherits W { WrapPanel { item-spacing: -1 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-spacing`") && errs[0].contains("non-negative"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_negative_line_spacing_rejected() {
        let errs = errors("component C inherits W { WrapPanel { line-spacing: -42 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.line-spacing`")
                && errs[0].contains("non-negative")
                && errs[0].contains("got -42"),
            "{:?}",
            errs
        );
    }

    // --- T1: WrapPanel constant-only bind reject (DD-M3-P3-003 / 004) ---

    #[test]
    fn wrappanel_item_cross_size_state_ident_rejected() {
        // `item-cross-size: <state-ident>` — the "bind" surface that
        // DD-M3-P3-004 declares constant-only in Phase 3.
        let errs = errors(
            "component C inherits W { state size: i32 = 88 WrapPanel { item-cross-size: size } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-cross-size` is constant-only")
                && errs[0].contains("non-negative `i32` literal"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_item_spacing_state_ident_rejected() {
        let errs = errors(
            "component C inherits W { state gap: i32 = 12 WrapPanel { item-spacing: gap } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-spacing` is constant-only"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_line_spacing_state_ident_rejected() {
        let errs = errors(
            "component C inherits W { state gap: i32 = 12 WrapPanel { line-spacing: gap } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.line-spacing` is constant-only"),
            "{:?}",
            errs
        );
    }

    // --- T1: WrapPanel non-IntLit RHS shape reject ---

    #[test]
    fn wrappanel_item_cross_size_ratio_literal_rejected() {
        let errs = errors("component C inherits W { WrapPanel { item-cross-size: 16:9 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-cross-size` is constant-only"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_item_cross_size_string_literal_rejected() {
        let errs = errors(r#"component C inherits W { WrapPanel { item-cross-size: "88" } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-cross-size` is constant-only"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_item_cross_size_bool_literal_rejected() {
        let errs = errors("component C inherits W { WrapPanel { item-cross-size: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-cross-size` is constant-only"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_item_cross_size_color_literal_rejected() {
        let errs = errors("component C inherits W { WrapPanel { item-cross-size: #cccccc } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-cross-size` is constant-only"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_item_spacing_measurement_rejected() {
        // `12px` is `Token::Measurement` — not an `IntLit`. Reject per the
        // constant-only `i32` literal rule (dsl_spec §4.10 "bare integer
        // literal").
        let errs = errors("component C inherits W { WrapPanel { item-spacing: 12px } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`WrapPanel.item-spacing` is constant-only"),
            "{:?}",
            errs
        );
    }

    // --- T1: WrapPanel attribute outside WrapPanel reject ---

    #[test]
    fn wrappanel_attr_on_box_rejected() {
        let errs = errors("component C inherits W { Box { item-cross-size: 88 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`item-cross-size` is a WrapPanel attribute")
                && errs[0].contains("widget `Box`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_attr_on_vstack_rejected() {
        let errs = errors("component C inherits W { VStack { item-spacing: 12 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`item-spacing` is a WrapPanel attribute")
                && errs[0].contains("widget `VStack`"),
            "{:?}",
            errs
        );
    }

    // --- T2: WrapPanel aspect-only-Box warning (DD-M3-P3-004 Recommendation) ---

    #[test]
    fn wrappanel_aspect_only_box_without_item_cross_size_warns() {
        // Firing shape per dsl_spec §4.10 Common pitfalls: WrapPanel directly
        // contains an aspect-bearing Box child but no `item-cross-size`.
        let src = r#"component C inherits W {
            WrapPanel {
                Box { aspect: 1:1 fill: #cccccc }
            }
        }"#;
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let ws = warnings(src);
        assert_eq!(ws.len(), 1, "{:?}", ws);
        assert!(
            ws[0].contains("aspect-only `Box`")
                && ws[0].contains("`item-cross-size`")
                && ws[0].contains("§4.10"),
            "{:?}",
            ws
        );
    }

    #[test]
    fn wrappanel_aspect_only_box_with_item_cross_size_does_not_warn() {
        // Positive control: matches the Phase 3 gallery sub-screen — the
        // explicit `item-cross-size: 88` suppresses the warning.
        let src = r#"component C inherits W {
            WrapPanel {
                item-cross-size: 88
                Box { aspect: 1:1 fill: #cccccc }
                Box { aspect: 1:1 fill: #cccccc }
            }
        }"#;
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(warnings(src).is_empty(), "{:?}", warnings(src));
    }

    #[test]
    fn wrappanel_aspect_only_box_nested_does_not_warn() {
        // Non-direct-child shape: the aspect-only Box is nested inside
        // another container inside the WrapPanel. Per DD-M3-P3-004 the
        // guard does not scan into nested containers.
        let src = r#"component C inherits W {
            WrapPanel {
                VStack {
                    Box { aspect: 1:1 fill: #cccccc }
                }
            }
        }"#;
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(warnings(src).is_empty(), "{:?}", warnings(src));
    }

    #[test]
    fn wrappanel_box_without_aspect_does_not_warn() {
        // Box without `aspect:` is not the footgun shape — no warning.
        let src = r#"component C inherits W {
            WrapPanel {
                Box { fill: #cccccc }
            }
        }"#;
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(warnings(src).is_empty(), "{:?}", warnings(src));
    }

    #[test]
    fn wrappanel_multi_aspect_only_box_emits_single_warning() {
        // One warning per WrapPanel regardless of how many matching
        // children — the warning describes the WrapPanel-level setting,
        // not per-child.
        let src = r#"component C inherits W {
            WrapPanel {
                Box { aspect: 1:1 fill: #cccccc }
                Box { aspect: 1:1 fill: #cccccc }
                Box { aspect: 1:1 fill: #cccccc }
            }
        }"#;
        let ws = warnings(src);
        assert_eq!(ws.len(), 1, "{:?}", ws);
    }

    // --- T1: ScrollView known widget + child-count contract (DD-M3-P4-001) ---

    #[test]
    fn scrollview_known_widget_no_warning() {
        // ScrollView is in KNOWN_WIDGET_TYPES — no "unknown widget" warning
        // when the child-count contract is satisfied (one child).
        let src = r#"component C inherits W { ScrollView { VStack {} } }"#;
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(
            warnings(src).is_empty(),
            "ScrollView should be a known widget type, not warn"
        );
    }

    #[test]
    fn scrollview_zero_child_rejected() {
        let errs = errors("component C inherits W { ScrollView {} }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView` requires exactly one child")
                && errs[0].contains("found 0")
                && errs[0].contains("VStack"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_one_child_accepted() {
        let result = check_src("component C inherits W { ScrollView { VStack {} } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn scrollview_two_children_rejected() {
        let errs = errors(r#"component C inherits W { ScrollView { VStack {} VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("found 2") && errs[0].contains("`ScrollView`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_three_children_rejected() {
        let errs =
            errors(r#"component C inherits W { ScrollView { VStack {} HStack {} Box {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(errs[0].contains("found 3"), "{:?}", errs);
    }

    #[test]
    fn scrollview_attrs_do_not_count_as_children() {
        // `offset-y` PropertyBind must not be miscounted as a child.
        // (Accept rules for `offset-y` arrive in T1 sub-task 3; this test
        // pins only the child-count side here — the bare child-count
        // diagnostic must not fire when the single child + attribute
        // are present.)
        let result =
            check_src(r#"component C inherits W { ScrollView { offset-y: 0 VStack {} } }"#);
        // The offset-y handling lands in the next commit; for now, the
        // child-count gate alone must not produce an error.
        let child_count_err = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == crate::diagnostic::Severity::Error)
            .any(|d| {
                d.message
                    .contains("`ScrollView` requires exactly one child")
            });
        assert!(
            !child_count_err,
            "child-count gate misfired: {:?}",
            result.diagnostics
        );
    }

    // --- T1: ScrollView offset-y literal accept (DD-M3-P4-003) ---

    #[test]
    fn scrollview_offset_y_positive_int_literal_accepted() {
        let result =
            check_src(r#"component C inherits W { ScrollView { offset-y: 42 VStack {} } }"#);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn scrollview_offset_y_zero_literal_accepted() {
        let result =
            check_src(r#"component C inherits W { ScrollView { offset-y: 0 VStack {} } }"#);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn scrollview_offset_y_negative_literal_accepted() {
        // DD-M3-P4-005 / DD-M3-P4-006: negative offsets are layout-time-
        // clamped to 0, not compile-time-rejected (explicitly distinct
        // from Phase 3 WrapPanel negative-literal rejection).
        let result =
            check_src(r#"component C inherits W { ScrollView { offset-y: -5 VStack {} } }"#);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    // --- T1: ScrollView offset-y state-ident binding accept ---

    #[test]
    fn scrollview_offset_y_i32_state_ident_accepted() {
        let result = check_src(
            r#"component C inherits W {
                state scroll_y: i32 = 0
                ScrollView { offset-y: scroll_y VStack {} }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    // --- T1: ScrollView offset-y non-integer literal reject ---

    #[test]
    fn scrollview_offset_y_string_literal_rejected() {
        let errs =
            errors(r#"component C inherits W { ScrollView { offset-y: "hello" VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`") && errs[0].contains("`i32` literal"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_float_literal_rejected() {
        let errs = errors(r#"component C inherits W { ScrollView { offset-y: 1.5 VStack {} } }"#);
        // FloatLit falls into the `_` arm of
        // `check_scrollview_offset_y_bind` and receives the
        // ScrollView-specific wording; the dispatch site in
        // `check_members_inner` skips the generic `check_expr_type`
        // FloatLit reject because the ScrollView+offset-y branch is
        // taken first.
        assert!(
            errs.iter().any(|e| e.contains("`ScrollView.offset-y`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_color_literal_rejected() {
        let errs =
            errors(r#"component C inherits W { ScrollView { offset-y: #336699 VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(errs[0].contains("`ScrollView.offset-y`"), "{:?}", errs);
    }

    #[test]
    fn scrollview_offset_y_ratio_literal_rejected() {
        let errs = errors(r#"component C inherits W { ScrollView { offset-y: 16:9 VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(errs[0].contains("`ScrollView.offset-y`"), "{:?}", errs);
    }

    #[test]
    fn scrollview_offset_y_bool_literal_rejected() {
        let errs = errors(r#"component C inherits W { ScrollView { offset-y: true VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(errs[0].contains("`ScrollView.offset-y`"), "{:?}", errs);
    }

    #[test]
    fn scrollview_offset_y_measurement_rejected() {
        // `12px` is `Token::Measurement` — not an `IntLit`. Reject per
        // dsl_spec §4.11 "i32 literal" surface contract.
        let errs = errors(r#"component C inherits W { ScrollView { offset-y: 12px VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(errs[0].contains("`ScrollView.offset-y`"), "{:?}", errs);
    }

    // --- T1: ScrollView offset-y bind-to-wrong-type-state reject ---

    #[test]
    fn scrollview_offset_y_undeclared_state_rejected() {
        let errs =
            errors(r#"component C inherits W { ScrollView { offset-y: scroll_y VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`")
                && errs[0].contains("not declared")
                && errs[0].contains("scroll_y"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_bool_state_rejected() {
        let errs = errors(
            r#"component C inherits W {
                state ready: bool = true
                ScrollView { offset-y: ready VStack {} }
            }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`") && errs[0].contains("declared `bool`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_string_state_rejected() {
        let errs = errors(
            r#"component C inherits W {
                state label: string = ""
                ScrollView { offset-y: label VStack {} }
            }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`") && errs[0].contains("declared `string`"),
            "{:?}",
            errs
        );
    }

    // --- T1: ScrollView writable (in-out) offset-y reject ---

    #[test]
    fn scrollview_in_out_property_offset_y_rejected() {
        // `in-out property<i32> offset-y: 0` inside ScrollView is the
        // writable surface DD-M3-P4-003 Option C deferred to M4. The
        // generic PropertyDecl arm in `check_members_inner` is otherwise
        // a no-op; the ScrollView-specific path rejects.
        let errs = errors(
            r#"component C inherits W {
                ScrollView {
                    in-out property<i32> offset-y: 0
                    VStack {}
                }
            }"#,
        );
        assert!(
            errs.iter().any(
                |e| e.contains("`ScrollView.offset-y` is bindable read-only")
                    && e.contains("in-out")
                    && e.contains("M4")
            ),
            "{:?}",
            errs
        );
    }

    // --- T1: ScrollView unknown-attribute reject (DD-M3-P4-001 / 002) ---

    #[test]
    fn scrollview_viewport_width_rejected() {
        let errs =
            errors(r#"component C inherits W { ScrollView { viewport-width: 320 VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`viewport-width`")
                && errs[0].contains("not a recognised ScrollView attribute")
                && errs[0].contains("§4.11"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_viewport_height_rejected() {
        let errs =
            errors(r#"component C inherits W { ScrollView { viewport-height: 240 VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`viewport-height`")
                && errs[0].contains("not a recognised ScrollView attribute"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_scroll_axis_rejected() {
        let errs =
            errors(r#"component C inherits W { ScrollView { scroll-axis: vertical VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`scroll-axis`")
                && errs[0].contains("not a recognised ScrollView attribute"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_padding_rejected() {
        let errs = errors(r#"component C inherits W { ScrollView { padding: 8 VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`padding`")
                && errs[0].contains("not a recognised ScrollView attribute"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_wrappanel_attr_inside_routes_to_wrappanel_diag() {
        // A WrapPanel attribute name inside ScrollView is still rejected,
        // but the WrapPanel-attribute-outside-WrapPanel diagnostic takes
        // precedence so the author sees the WrapPanel-specific wording
        // (the attribute is recognised, just misplaced). This keeps the
        // ScrollView catch-all attribute-specific.
        let errs =
            errors(r#"component C inherits W { ScrollView { item-cross-size: 88 VStack {} } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`item-cross-size` is a WrapPanel attribute")
                && errs[0].contains("widget `ScrollView`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_attr_at_component_level_rejected() {
        let errs = errors("component C inherits W { line-spacing: 12 WrapPanel {} }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("unknown host attribute `line-spacing`"),
            "{:?}",
            errs
        );
    }

    // --- M3-Phase 5 T1: Grid / Cell Surface A2 diagnostics ---------------
    //
    // ADR Phase 5 verification closure evidence item (1), representative-
    // fixture / diagnostic half (DD-M3-P5-001 .. DD-M3-P5-006). Positive
    // controls plus a reject case per diagnostic on the surface.

    /// Representative valid Grid: mixed fixed + weighted-star tracks on
    /// both axes, a column-spanning header cell, and three middle cells.
    const VALID_GRID: &str = r#"component C inherits W {
        Grid {
            columns: 180 1* 2*
            rows: 1* 1*
            Cell { row: 0 column: 0 column-span: 3 Text { text: "header" } }
            Cell { row: 1 column: 0 Box { fill: #cccccc } }
            Cell { row: 1 column: 1 h-align: center v-align: end Text { text: "x" } }
            Cell { row: 1 column: 2 Box { fill: #cccccc } }
        }
    }"#;

    #[test]
    fn grid_known_widget_no_warning() {
        let result = check_src("component C inherits W { Grid { columns: 1* rows: 1* } }");
        assert!(
            warnings("component C inherits W { Grid { columns: 1* rows: 1* } }").is_empty(),
            "Grid should be a known widget type, not warn"
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_representative_fixture_accepted() {
        let result = check_src(VALID_GRID);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_single_cell_omits_placement_accepted() {
        // Single-Cell escape clause (DD-M3-P5-001): a lone Cell may omit
        // row: / column:.
        let result = check_src(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { Text { text: "x" } } } }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_row_span_accepted() {
        // Both-axis spanning positive control (DD-M3-P5-003).
        let result = check_src(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1*
                    rows: 1* 1*
                    Cell { row: 0 column: 0 row-span: 2 Box { fill: #cccccc } }
                    Cell { row: 0 column: 1 Text { text: "a" } }
                    Cell { row: 1 column: 1 Text { text: "b" } }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_missing_columns_rejected() {
        let errs = errors(r#"component C inherits W { Grid { rows: 1* Cell { Text {} } } }"#);
        assert!(
            errs.iter().any(|e| e.contains("requires a `columns:`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_missing_rows_rejected() {
        let errs = errors(r#"component C inherits W { Grid { columns: 1* Cell { Text {} } } }"#);
        assert!(
            errs.iter().any(|e| e.contains("requires a `rows:`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_fixed_track_zero_rejected() {
        let errs = errors("component C inherits W { Grid { columns: 0 rows: 1* } }");
        assert!(
            errs.iter().any(
                |e| e.contains("fixed track size must be a positive integer")
                    && e.contains("got 0")
            ),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_fixed_track_negative_rejected() {
        let errs = errors("component C inherits W { Grid { columns: -5 rows: 1* } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("fixed track size must be a positive integer")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_star_weight_zero_rejected() {
        let errs = errors("component C inherits W { Grid { columns: 0* rows: 1* } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("star weight must be >= 1") && e.contains("got 0")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_star_weight_over_cap_rejected() {
        let errs = errors("component C inherits W { Grid { columns: 2048* rows: 1* } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("must not exceed 1024") && e.contains("got 2048")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_auto_track_reserved_future_diagnostic() {
        // The `auto` diagnostic must name it reserved-future, not "unknown"
        // (DD-M3-P5-002).
        let errs = errors("component C inherits W { Grid { columns: auto rows: 1* } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("`auto`") && e.contains("reserved for a future phase")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_float_track_rejected() {
        let errs = errors("component C inherits W { Grid { columns: 1.5 rows: 1* } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("floating-point track sizes are not valid")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_unknown_track_token_rejected() {
        let errs = errors("component C inherits W { Grid { columns: wibble rows: 1* } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown track size token `wibble`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_unknown_attribute_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* gap: 4 Cell { Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown Grid attribute `gap`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn grid_direct_child_without_slot_uses_default_placement() {
        let result = check_src(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Text { text: "loose" } } }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn cell_outside_grid_rejected() {
        let errs = errors(r#"component C inherits W { VStack { Cell { Text { text: "x" } } } }"#);
        assert!(
            errs.iter().any(
                |e| e.contains("`Cell` is only valid as a direct child of a `Grid`")
                    && e.contains("inside `VStack`")
            ),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_at_component_level_rejected() {
        let errs = errors(r#"component C inherits W { Cell { Text { text: "x" } } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell` is only valid") && e.contains("at component level")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_zero_children_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell` requires exactly one content child")
                    && e.contains("found 0")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_two_children_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { Text {} Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell` requires exactly one content child")
                    && e.contains("found 2")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_missing_placement_in_multi_cell_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1* rows: 1*
                    Cell { row: 0 column: 0 Text {} }
                    Cell { Text {} }
                }
            }"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("must declare `row:`")),
            "{:?}",
            errs
        );
        assert!(
            errs.iter().any(|e| e.contains("must declare `column:`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_negative_row_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: -1 column: 0 Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell.row` must be a non-negative integer")
                    && e.contains("got -1")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_non_integer_row_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: "top" column: 0 Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell.row` must be a non-negative integer literal")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_non_integer_column_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: true Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell.column` must be a non-negative integer literal")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_row_out_of_range_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 1 column: 0 Text {} } } }"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("row span exceeds the grid")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_column_out_of_range_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: 1 Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("column span exceeds the grid")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_zero_span_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 column-span: 0 Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell.column-span` must be a positive integer")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_non_integer_row_span_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 row-span: "two" Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell.row-span` must be a positive integer literal")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_non_integer_column_span_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 column-span: false Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell.column-span` must be a positive integer literal")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_span_exceeds_grid_rejected() {
        // column 2 + column-span 2 = 4 > 3 declared column tracks.
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* 1* 1* rows: 1* Cell { row: 0 column: 2 column-span: 2 Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("column span exceeds the grid")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_unknown_attribute_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 weight: 3 Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown `Cell` attribute `weight`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_bad_alignment_value_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 h-align: middle Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`Cell.h-align` must be one of") && e.contains("`middle`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn cell_alignment_vocabulary_accepted() {
        let result = check_src(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { row: 0 column: 0 h-align: start v-align: stretch Text {} } } }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_same_cell_conflict_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1* rows: 1*
                    Cell { row: 0 column: 0 Text {} }
                    Cell { row: 0 column: 0 Text {} }
                }
            }"#,
        );
        assert!(errs.iter().any(|e| e.contains("overlaps")), "{:?}", errs);
    }

    #[test]
    fn grid_overlapping_span_rejected() {
        // A column-spanning cell overlapping a single cell in one of its
        // covered columns (DD-M3-P5-003).
        let errs = errors(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1* rows: 1*
                    Cell { row: 0 column: 0 column-span: 2 Text {} }
                    Cell { row: 0 column: 1 Text {} }
                }
            }"#,
        );
        assert!(errs.iter().any(|e| e.contains("overlaps")), "{:?}", errs);
    }

    #[test]
    fn grid_adjacent_non_overlapping_cells_accepted() {
        // Regression guard: adjacent (touching) rectangles must NOT be
        // reported as overlapping (half-open interval semantics).
        let result = check_src(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1* rows: 1*
                    Cell { row: 0 column: 0 Text {} }
                    Cell { row: 0 column: 1 Text {} }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    // --- M3-Phase 6 T1: ZStack check surface (DD-M3-P6-001 / 002) -------

    #[test]
    fn zstack_known_widget_no_warning() {
        let result = check_src("component C inherits W { ZStack { Text {} Box {} } }");
        assert!(
            warnings("component C inherits W { ZStack { Text {} Box {} } }").is_empty(),
            "ZStack should be a known widget type, not warn"
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn zstack_direct_child_alignment_accepted() {
        let result = check_src(
            r#"component C inherits W {
                ZStack {
                    Box { fill: #00000080 }
                    Text { slot.h-align: center slot.v-align: end text: "caption" }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_direct_slot_child_accepted() {
        let result = check_src(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1*
                    rows: 1*
                    Text { slot.row: 0 slot.column: 1 slot.h-align: center text: "direct" }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_cell_and_direct_slot_forms_can_coexist() {
        let result = check_src(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1*
                    rows: 1*
                    Cell { row: 0 column: 0 Text { text: "cell" } }
                    Text { slot.row: 0 slot.column: 1 text: "direct" }
                }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_direct_slot_unknown_key_rejected() {
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Text { slot.foo: 0 } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown `Grid` slot key `slot.foo`")),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_constant_rhs_rejected() {
        let errs = errors(
            r#"component C inherits W {
                state r: i32 = 0
                Grid { columns: 1* rows: 1* Text { slot.row: r } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`slot.row` must be a non-negative integer literal")),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_row_out_of_range_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid { columns: 1* rows: 1* Text { slot.row: 1 slot.column: 0 } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`slot.row` placement exceeds the grid")),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_column_out_of_range_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid { columns: 1* rows: 1* Text { slot.row: 0 slot.column: 1 } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`slot.column` placement exceeds the grid")),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_negative_index_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid { columns: 1* rows: 1* Text { slot.row: -1 slot.column: 0 } }
            }"#,
        );
        assert!(
            errs.iter().any(|e| {
                e.contains("`slot.row` must be a non-negative integer") && e.contains("got -1")
            }),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_zero_span_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid { columns: 1* rows: 1* Text { slot.row-span: 0 } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`slot.row-span` must be a positive integer")),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_non_literal_span_rejected() {
        let errs = errors(
            r#"component C inherits W {
                state span: i32 = 1
                Grid { columns: 1* rows: 1* Text { slot.row-span: span } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`slot.row-span` must be a positive integer literal")),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_bad_alignment_keyword_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid { columns: 1* rows: 1* Text { slot.h-align: middle } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`slot.h-align` must be one of") && e.contains("`middle`")),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_non_keyword_alignment_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid { columns: 1* rows: 1* Text { slot.h-align: 3 } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`slot.h-align` expects an alignment keyword")),
            "{errs:?}"
        );
    }

    #[test]
    fn grid_direct_slot_value_namespace_prefers_alignment_keyword() {
        let result = check_src(
            r#"component C inherits W {
                state end: i32 = 0
                Grid { columns: 1* rows: 1* Text { slot.h-align: end } }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn zstack_direct_slot_value_namespace_prefers_alignment_keyword() {
        let result = check_src(
            r#"component C inherits W {
                state end: i32 = 0
                ZStack { Text { slot.h-align: end } }
            }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn grid_direct_slot_overlaps_cell_rejected() {
        let errs = errors(
            r#"component C inherits W {
                Grid {
                    columns: 1* rows: 1*
                    Cell { row: 0 column: 0 Text {} }
                    Text { slot.row: 0 slot.column: 0 }
                }
            }"#,
        );
        assert!(errs.iter().any(|e| e.contains("overlaps")), "{errs:?}");
    }

    #[test]
    fn slot_property_inside_cell_attrs_is_mixing_reject() {
        let errs = errors(
            r#"component C inherits W {
                Grid { columns: 1* rows: 1* Cell { slot.row: 0 Text {} } }
            }"#,
        );
        assert!(
            errs.iter().any(|e| e.contains("strict PM-2 mixing reject")),
            "{errs:?}"
        );
    }

    #[test]
    fn slot_property_inside_cell_content_is_non_admitting_parent_reject() {
        let errs = errors(
            r#"component C inherits W {
                Grid { columns: 1* rows: 1* Cell { Text { slot.row: 0 } } }
            }"#,
        );
        assert!(
            errs.iter().any(|e| {
                e.contains("parent-owned child placement data") && e.contains("inside a `Cell`")
            }),
            "{errs:?}"
        );
        assert!(
            !errs.iter().any(|e| e.contains("strict PM-2 mixing reject")),
            "{errs:?}"
        );
    }

    #[test]
    fn slot_property_under_non_admitting_parent_rejected() {
        let errs = errors(r#"component C inherits W { VStack { Text { slot.h-align: center } } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("parent-owned child placement data")
                    && e.contains("inside `Text`")),
            "{errs:?}"
        );
    }

    #[test]
    fn slot_property_at_component_level_rejected() {
        let errs = errors(r#"component C inherits W { slot.h-align: center ZStack {} }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("parent-owned child placement data")
                    && e.contains("component level")),
            "{errs:?}"
        );
    }

    #[test]
    fn zstack_grid_child_slot_alignment_accepted() {
        // M3-Phase 7b T6b: a `Grid` that is a direct `ZStack` child MAY
        // carry both `slot.h-align` and `slot.v-align` to place itself, per
        // DD-M3-P7b-001 ("slot.* valid on a ZStack direct child"; a Grid is
        // a widget). Previously the Grid pass (`check_grid`) consumed the
        // `slot.*` among the Grid's own members and wrongly rejected it as
        // "inside `Grid`". The parent ZStack validates it through the generic
        // walk, so the whole component compiles with NO error (asserted on
        // `has_errors()`, not just the absence of one diagnostic string).
        // (The layout-side limitation — a Fill-default Grid collapses to 0×0
        // on a Shrink ancestor axis — is a separate carry-forward, not a
        // checker concern: docs/notes/author-controllable-sizing.md.)
        let result = check_src(
            r#"component C inherits W {
                ZStack { Grid { slot.h-align: end slot.v-align: start columns: 1* rows: 1* Cell { Text { text: "x" } } } }
            }"#,
        );
        assert!(
            !result.has_errors(),
            "Grid-as-ZStack-child slot.h-align/slot.v-align must compile, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn zstack_grid_child_unknown_slot_key_still_rejected() {
        // Positive control for `zstack_grid_child_slot_alignment_accepted`:
        // T6b skips *all* `slot.*` among a Grid's own members, not only the
        // alignment keys. A Grid-placement key (`slot.row`) on a Grid that is
        // a ZStack child is therefore delegated to the parent ZStack, which
        // rejects it as an unknown ZStack slot key.
        //
        // The discriminator vs the OLD behavior is that the Grid pass adds
        // NO "inside `Grid`" diagnostic: the old `check_grid` consumed
        // `slot.row` and emitted "inside `Grid`" *in addition to* the parent
        // walk's unknown-key error, so asserting only the unknown-key
        // presence would pass on the old code too. The unknown-key error must
        // also appear exactly once (no duplicate path).
        let errs = errors(
            r#"component C inherits W {
                ZStack { Grid { slot.row: 0 columns: 1* rows: 1* Cell { Text { text: "x" } } } }
            }"#,
        );
        let unknown_key = errs
            .iter()
            .filter(|e| e.contains("unknown `ZStack` slot key `slot.row`"))
            .count();
        assert_eq!(
            unknown_key, 1,
            "parent ZStack must reject the delegated key exactly once: {errs:?}"
        );
        assert!(
            !errs.iter().any(|e| {
                e.contains("parent-owned child placement data") && e.contains("inside `Grid`")
            }),
            "Grid pass must NOT consume slot.row (no \"inside `Grid`\"): {errs:?}"
        );
    }

    #[test]
    fn zstack_grid_child_slot_alignment_value_still_validated() {
        // Positive control for `zstack_grid_child_slot_alignment_accepted`:
        // the fix delegates validation to the parent ZStack, so the
        // `slot.h-align` VALUE is still checked. A blanket-accept (the wrong
        // implementation) would let a bogus alignment through; this pins that
        // the parent ZStack pass fires `check_zstack_child_align`.
        let errs = errors(
            r#"component C inherits W {
                ZStack { Grid { slot.h-align: bogus columns: 1* rows: 1* Cell { Text { text: "x" } } } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("ZStack child `slot.h-align` must be one of")),
            "{errs:?}"
        );
    }

    #[test]
    fn nonadmitting_parent_grid_child_slot_still_rejected() {
        // Positive control for `zstack_grid_child_slot_alignment_accepted`:
        // the fix must NOT blanket-accept `slot.*` on any Grid. A Grid under
        // a non-admitting parent (VStack) still has its `slot.*` rejected as
        // parent-owned placement data in the wrong position. Exactly one such
        // diagnostic fires (the Grid pass no longer adds a duplicate).
        let errs = errors(
            r#"component C inherits W {
                VStack { Grid { slot.h-align: end columns: 1* rows: 1* Cell { Text { text: "x" } } } }
            }"#,
        );
        let placement_errs = errs
            .iter()
            .filter(|e| {
                e.contains("parent-owned child placement data") && e.contains("inside `Grid`")
            })
            .count();
        assert_eq!(placement_errs, 1, "{errs:?}");
    }

    #[test]
    fn component_level_grid_slot_still_rejected_once() {
        // Completes `nonadmitting_parent_grid_child_slot_still_rejected` for
        // the parent=None branch named in the T6b start gate ("VStack /
        // component level"): a Grid at the component root (no admitting
        // parent) still has its `slot.*` rejected as parent-owned data in the
        // wrong position, exactly once (the Grid pass adds no duplicate).
        let errs = errors(
            r#"component C inherits W {
                Grid { slot.h-align: end columns: 1* rows: 1* Cell { Text { text: "x" } } }
            }"#,
        );
        let placement_errs = errs
            .iter()
            .filter(|e| {
                e.contains("parent-owned child placement data") && e.contains("inside `Grid`")
            })
            .count();
        assert_eq!(placement_errs, 1, "{errs:?}");
    }

    #[test]
    fn zstack_slot_unknown_key_rejected() {
        let errs = errors(r#"component C inherits W { ZStack { Text { slot.row: 0 } } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown `ZStack` slot key `slot.row`")),
            "{errs:?}"
        );
    }

    #[test]
    fn zstack_slot_constant_rhs_rejected() {
        let errs = errors(
            r#"component C inherits W {
                state align: string = "end"
                ZStack { Text { slot.h-align: align } }
            }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`slot.h-align` must be one of")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_bool_state_accepted() {
        let result = check_src(
            "component C inherits W { state ready: bool = true VStack { if ready { Text {} } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn conditional_bool_literal_accepted() {
        let result = check_src("component C inherits W { VStack { if true { Text {} } } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn conditional_non_bool_condition_rejected() {
        let errs = errors(
            "component C inherits W { state count: i32 = 0 VStack { if count { Text {} } } }",
        );
        assert!(
            errs.iter().any(|e| e.contains("condition must be `bool`")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_literal_condition_rejected() {
        let errs = errors("component C inherits W { VStack { if 3 { Text {} } } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("condition must be a bool literal or declared bool state")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_undeclared_condition_rejected() {
        let errs = errors("component C inherits W { VStack { if missing { Text {} } } }");
        assert!(errs.iter().any(|e| e.contains("not declared")), "{errs:?}");
    }

    #[test]
    fn conditional_operator_condition_rejected() {
        let errs = errors(
            "component C inherits W { state ready: bool = true VStack { if ! ready { Text {} } } }",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("operators in `if` conditions")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_component_level_rejected() {
        let errs = errors("component C inherits W { if true { Text {} } }");
        assert!(
            errs.iter().any(|e| e.contains("component-level `if`")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_non_structural_body_rejected() {
        let errs = errors("component C inherits W { VStack { if true { text: \"x\" } } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("body admits only a single widget child")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_multi_child_body_rejected() {
        let errs = errors("component C inherits W { VStack { if true { Text {} Button {} } } }");
        assert!(
            errs.iter().any(|e| e.contains("exactly one widget child")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_direct_nested_if_body_rejected() {
        let errs = errors("component C inherits W { VStack { if true { if false { Text {} } } } }");
        assert!(
            errs.iter().any(|e| e.contains("bare nested `if`")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_direct_grid_child_rejected() {
        let errs = errors(
            "component C inherits W { Grid { columns: 1* rows: 1* if true { Cell { Text {} } } } }",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("conditional members may appear inside a Cell content widget")),
            "{errs:?}"
        );
    }

    #[test]
    fn conditional_cell_sibling_rejected() {
        let errs = errors(
            "component C inherits W { Grid { columns: 1* rows: 1* Cell { VStack {} if true { Text {} } } } }",
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("put conditional members inside that content widget")),
            "{errs:?}"
        );
    }

    // T4 review follow-up: single-child container child-count gates must
    // count a conditional sibling (it materialises at most one child), or a
    // `Box { Content if c }` / `ScrollView { Content if c }` slips past the
    // widget-only count. See log.md T4 migration audit + DD-M3-P6-007.
    #[test]
    fn box_widget_and_conditional_sibling_rejected() {
        let errs = errors("component C inherits W { Box { Text {} if true { Text {} } } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("`Box` admits at most one child")),
            "{errs:?}"
        );
    }

    #[test]
    fn box_conditional_only_child_accepted() {
        // A lone conditional is one potential child (≤ 1), so it is valid.
        let result =
            check_src("component C inherits W { state c: bool = true Box { if c { Text {} } } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn box_multiple_conditional_siblings_rejected() {
        // Two conditionals are two potential children — the shortest proof
        // that the count counts conditionals, not just widget+conditional.
        let errs =
            errors("component C inherits W { Box { if true { Text {} } if true { Button {} } } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("`Box` admits at most one child")),
            "{errs:?}"
        );
    }

    #[test]
    fn scrollview_conditional_member_rejected() {
        let errs = errors("component C inherits W { ScrollView { Box {} if true { Text {} } } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("a conditional member is not valid directly in ScrollView")),
            "{errs:?}"
        );
        assert!(!errs.iter().any(|e| e.contains("DD-M3-P6-007")), "{errs:?}");
    }

    #[test]
    fn scrollview_conditional_only_member_rejected() {
        // DD-M3-P6-007 centre case: a conditional-only ScrollView content
        // (`ScrollView { if c { … } }`) is the interim (a) rejection — pins
        // the current value a future DD-M3-P6-007 (b) relaxation would flip.
        let errs = errors("component C inherits W { ScrollView { if true { Text {} } } }");
        assert!(
            errs.iter()
                .any(|e| e.contains("a conditional member is not valid directly in ScrollView")),
            "{errs:?}"
        );
        assert!(!errs.iter().any(|e| e.contains("DD-M3-P6-007")), "{errs:?}");
    }

    #[test]
    fn zstack_unknown_attribute_rejected() {
        let errs = errors(r#"component C inherits W { ZStack { spacing: 8 Text {} } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown ZStack attribute `spacing`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn zstack_reserved_layering_attribute_rejected() {
        let errs = errors(r#"component C inherits W { ZStack { z-index: 1 Text {} } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown ZStack attribute `z-index`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn zstack_grid_track_attribute_rejected() {
        let errs = errors(r#"component C inherits W { ZStack { columns: 1 Text {} } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown ZStack attribute `columns`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn zstack_child_bad_alignment_value_rejected() {
        let errs = errors(r#"component C inherits W { ZStack { Text { slot.h-align: middle } } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("ZStack child `slot.h-align` must be one of")
                    && e.contains("`middle`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn zstack_child_non_keyword_alignment_value_rejected() {
        // A non-identifier value (here an integer literal) must hit the
        // `expects an alignment keyword` arm of `check_zstack_child_align`,
        // distinct from the bad-identifier arm above.
        let errs = errors(r#"component C inherits W { ZStack { Text { slot.h-align: 3 } } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("ZStack child `slot.h-align` expects an alignment keyword")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn zstack_child_bare_alignment_rejected() {
        let errs = errors(r#"component C inherits W { ZStack { Text { h-align: end } } }"#);
        assert!(
            errs.iter().any(|e| {
                e.contains("ZStack child bare `h-align` is no longer accepted")
                    && e.contains("slot.h-align")
            }),
            "{errs:?}"
        );
    }

    #[test]
    fn placement_attr_outside_zstack_child_or_cell_rejected() {
        let errs = errors(r#"component C inherits W { VStack { Text { h-align: center } } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("parent-owned child placement attribute")
                    && e.contains("ZStack direct child")
                    && e.contains("Grid `Cell`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn placement_attr_on_zstack_itself_rejected_with_container_position() {
        let errs = errors(r#"component C inherits W { ZStack { h-align: center Text {} } }"#);
        assert!(
            errs.iter()
                .any(|e| e.contains("parent-owned child placement attribute")
                    && e.contains("on `ZStack` itself")),
            "{:?}",
            errs
        );
    }

    // --- M4-Phase 2 T6 stage 1: focus-group / modal-scope / dismiss ------
    //
    // Admission and constant-only rules for the two focus annotations,
    // plus `dismiss`'s modal-scope-sibling requirement (dsl_spec §4.19,
    // DD-M4-P2-005 A1). Accept side per admitting kind plus reject side
    // per rejection branch.

    /// Body template for each of the seven `FOCUS_ANNOTATION_CONTAINERS`
    /// kinds, with `{ATTR}` standing in for the focus-annotation property
    /// bind under test. ScrollView carries its required single child;
    /// Grid carries its required `columns:` / `rows:` track lists.
    const FOCUS_ANNOTATION_CONTAINER_FIXTURES: &[(&str, &str)] = &[
        ("VStack", "VStack { {ATTR} }"),
        ("HStack", "HStack { {ATTR} }"),
        ("Box", "Box { {ATTR} }"),
        ("WrapPanel", "WrapPanel { {ATTR} }"),
        ("ScrollView", "ScrollView { {ATTR} Text { text: \"x\" } }"),
        ("Grid", "Grid { columns: 1* rows: 1* {ATTR} }"),
        ("ZStack", "ZStack { {ATTR} }"),
    ];

    fn assert_focus_annotation_accepted_everywhere(attr_line: &str) {
        for (kind, body) in FOCUS_ANNOTATION_CONTAINER_FIXTURES {
            let src = format!(
                "component C inherits W {{ {} }}",
                body.replace("{ATTR}", attr_line)
            );
            let result = check_src(&src);
            // `is_empty`, not `!has_errors`: a warning would mean one of
            // the per-kind gates still reacts to the name.
            assert!(
                result.diagnostics.is_empty(),
                "{} accepting `{}`: {:?}",
                kind,
                attr_line,
                result.diagnostics
            );
        }
    }

    #[test]
    fn focus_group_true_accepted_on_every_admitting_container() {
        assert_focus_annotation_accepted_everywhere("focus-group: true");
    }

    #[test]
    fn modal_scope_true_accepted_on_every_admitting_container() {
        assert_focus_annotation_accepted_everywhere("modal-scope: true");
    }

    #[test]
    fn focus_group_and_modal_scope_together_on_one_container_accepted_with_warning() {
        // Both halves stay accepted — DD-M4-P2-005 A1 chose two separate
        // constant-only attributes precisely so a container could carry
        // both — but `WidgetNode::focus_role`
        // (wasamo-runtime/src/widget.rs) holds one `FocusRole` per node
        // and gives `modal-scope` precedence, so `focus-group` is inert
        // here. That is not silent: exactly one warning fires, naming the
        // ineffective `focus-group`.
        let src = "component C inherits W { VStack { focus-group: true modal-scope: true } }";
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let ws = warnings(src);
        assert_eq!(ws.len(), 1, "{:?}", ws);
        // Assert what the message must convey — both attribute names, and
        // that `focus-group` is the one with no effect — rather than its
        // exact wording, which is presentation.
        assert!(
            ws[0].contains("focus-group")
                && ws[0].contains("modal-scope")
                && ws[0].contains("no effect")
                && ws[0].contains("§4.19"),
            "{:?}",
            ws
        );
    }

    #[test]
    fn focus_group_true_modal_scope_false_no_warning() {
        // `focus-group: true` beside `modal-scope: false` is not the both-
        // true shape — the group half is not overridden.
        let src = "component C inherits W { VStack { focus-group: true modal-scope: false } }";
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(warnings(src).is_empty(), "{:?}", warnings(src));
    }

    #[test]
    fn focus_group_false_modal_scope_true_no_warning() {
        // The mirror case: `modal-scope: true` beside `focus-group: false`
        // is an ordinary, unremarked scope.
        let src = "component C inherits W { VStack { focus-group: false modal-scope: true } }";
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        assert!(warnings(src).is_empty(), "{:?}", warnings(src));
    }

    #[test]
    fn focus_group_or_modal_scope_alone_no_warning() {
        // Either attribute alone is the ordinary single-role case; the
        // warning requires both present and both `true`.
        let group_only = "component C inherits W { VStack { focus-group: true } }";
        assert!(
            warnings(group_only).is_empty(),
            "{:?}",
            warnings(group_only)
        );
        let scope_only = "component C inherits W { VStack { modal-scope: true } }";
        assert!(
            warnings(scope_only).is_empty(),
            "{:?}",
            warnings(scope_only)
        );
    }

    #[test]
    fn focus_group_and_modal_scope_together_inside_if_body_still_one_warning() {
        // Proves the scan runs at the `Member::Conditional` recursive call
        // in `check_members_inner`, the same as `carries_modal_scope`
        // itself (see the `if`-wrapped tests further below). The `if`
        // body admits exactly one widget child in M3-Phase 6, so an outer
        // `Box` wraps the `if`.
        let src = "component C inherits W { state open: bool = true Box { if open { VStack { focus-group: true modal-scope: true } } } }";
        let result = check_src(src);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
        let ws = warnings(src);
        assert_eq!(ws.len(), 1, "{:?}", ws);
    }

    #[test]
    fn focus_group_false_accepted() {
        // `false` is a valid constant, not just `true` (dsl_spec §4.19 default).
        let result = check_src("component C inherits W { VStack { focus-group: false } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn modal_scope_false_accepted() {
        let result = check_src("component C inherits W { VStack { modal-scope: false } }");
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn dismiss_handler_accepted_beside_modal_scope_true() {
        let result = check_src(
            "component C inherits W { state open: bool = true Box { modal-scope: true dismiss => { open = false; } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn dismiss_handler_accepted_on_grid_carrying_modal_scope() {
        // `dismiss` is admitted here because the Grid carries
        // `modal-scope: true` (the generic dsl_spec §4.19 rule; `Grid`
        // holds no per-kind signal rule of its own).
        let result = check_src(
            "component C inherits W { state open: bool = true Grid { columns: 1* rows: 1* modal-scope: true dismiss => { open = false; } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn focus_group_true_on_zstack_produces_no_diagnostic() {
        // Positive control: ZStack's own unknown-attribute catch-all
        // (`check_zstack_unknown_attr`) used to be reachable before the
        // focus-annotation dispatch and would have swallowed this name.
        let result = check_src("component C inherits W { ZStack { focus-group: true } }");
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    }

    #[test]
    fn modal_scope_true_on_scrollview_produces_no_diagnostic() {
        // Positive control: ScrollView's own attribute catch-all
        // (`check_scrollview_unknown_attr`) used to be reachable before
        // the focus-annotation dispatch and would have swallowed this name.
        let result = check_src(
            r#"component C inherits W { ScrollView { modal-scope: true Text { text: "x" } } }"#,
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    }

    #[test]
    fn focus_group_state_ident_rejected() {
        let errs = errors(
            "component C inherits W { state flag: bool = true VStack { focus-group: flag } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`focus-group` is constant-only") && errs[0].contains("§4.19"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn modal_scope_int_literal_rejected() {
        // Same constant-only branch, a different non-`BoolLit` input shape.
        let errs = errors("component C inherits W { VStack { modal-scope: 1 } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`modal-scope` is constant-only") && errs[0].contains("§4.19"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn focus_group_true_on_text_rejected() {
        let errs = errors("component C inherits W { Text { focus-group: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`focus-group` is admitted on any container")
                && errs[0].contains("widget `Text`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn focus_group_true_on_button_rejected() {
        let errs = errors("component C inherits W { Button { focus-group: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`focus-group` is admitted on any container")
                && errs[0].contains("widget `Button`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn focus_group_true_on_togglebutton_rejected_as_admission_not_unknown_attr() {
        // Proves dispatch ordering: the focus-annotation admission check
        // runs before `check_togglebutton_property_name`, so the
        // diagnostic is the admission one, not "unknown ToggleButton
        // attribute".
        let errs = errors("component C inherits W { ToggleButton { focus-group: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`focus-group` is admitted on any container")
                && errs[0].contains("widget `ToggleButton`"),
            "{:?}",
            errs
        );
        assert!(
            !errs[0].contains("unknown ToggleButton attribute"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn focus_group_true_on_rectangle_rejected() {
        let errs = errors("component C inherits W { Rectangle { focus-group: true } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`focus-group` is admitted on any container")
                && errs[0].contains("widget `Rectangle`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn focus_group_true_inside_cell_rejected() {
        // `Cell` is an IR-only Grid wrapper, not a runtime container, and
        // is excluded from `FOCUS_ANNOTATION_CONTAINERS`. `check_cell`
        // (called from `check_grid`) independently flags the same name as
        // an unknown `Cell` attribute, so this fires alongside a second
        // diagnostic — `.any()` locates the admission one specifically
        // (same dual-diagnostic shape already established for a WrapPanel
        // attribute misplaced on a non-admitting widget).
        let errs = errors(
            r#"component C inherits W { Grid { columns: 1* rows: 1* Cell { focus-group: true Text {} } } }"#,
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`focus-group` is admitted on any container")
                    && e.contains("widget `Cell`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn focus_group_true_at_component_level_rejected_as_unknown_host_attr() {
        // Component-level `focus-group:` never reaches the focus-
        // annotation dispatch: the `enclosing_widget.is_none()` early
        // return routes it to `check_host_property_bind` first, same as
        // any other unrecognised host attribute.
        let errs = errors("component C inherits W { focus-group: true ZStack { Text {} } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("unknown host attribute `focus-group`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn dismiss_handler_without_modal_scope_sibling_rejected() {
        let errs = errors(
            "component C inherits W { state open: bool = true Box { dismiss => { open = false; } } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`dismiss` handler can never be raised") && errs[0].contains("§4.19"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn dismiss_handler_beside_modal_scope_false_rejected() {
        let errs = errors(
            "component C inherits W { state open: bool = true Box { modal-scope: false dismiss => { open = false; } } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`dismiss` handler can never be raised"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn dismiss_handler_at_component_level_rejected() {
        let errs = errors(
            "component C inherits W { state open: bool = true dismiss => { open = false; } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`dismiss` handler can never be raised"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn dismiss_beside_a_modal_scope_on_a_non_container_is_still_rejected() {
        // The admission predicate is "a **container** that carries
        // `modal-scope: true`", both halves. A sibling-only predicate
        // would let the rejected annotation on a leaf widget suppress
        // this second diagnostic; both must fire.
        let errs = errors(
            "component C inherits W { state open: bool = true Text { modal-scope: true dismiss => { open = false; } } }",
        );
        assert_eq!(errs.len(), 2, "{:?}", errs);
        assert!(
            errs.iter()
                .any(|e| e.contains("`modal-scope` is admitted on any container")),
            "{:?}",
            errs
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`dismiss` handler can never be raised")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn dismiss_beside_a_component_level_modal_scope_is_still_rejected() {
        // The other half of the same predicate: a component member list
        // is not a container, so a (itself-rejected) component-level
        // `modal-scope: true` cannot admit a component-level `dismiss`.
        let errs = errors(
            "component C inherits W { state open: bool = true modal-scope: true dismiss => { open = false; } }",
        );
        assert_eq!(errs.len(), 2, "{:?}", errs);
        assert!(
            errs.iter()
                .any(|e| e.contains("unknown host attribute `modal-scope`")),
            "{:?}",
            errs
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`dismiss` handler can never be raised")),
            "{:?}",
            errs
        );
    }

    // Every test above writes `dismiss` as a flat sibling of
    // `modal-scope: true` directly in a widget body. §4.19's own worked
    // example — and the shape the feature exists for (the gallery
    // lightbox) — wraps the annotated container in an `if`. These four
    // tests exercise `carries_modal_scope`'s recomputation at the
    // `Member::Conditional` / `Member::For` recursive calls in
    // `check_members_inner`, which the flat tests above never reach.

    #[test]
    fn dismiss_handler_accepted_inside_if_wrapped_modal_scope() {
        // The `if` body admits exactly one widget child in M3-Phase 6, so
        // an outer `Box` wraps the `if` and the `if`'s branch body is the
        // annotated inner container — dsl_spec §4.19's own
        // `modal-scope` / `dismiss` example shape.
        let result = check_src(
            "component C inherits W { state lightbox_open: bool = true Box { if lightbox_open { Box { modal-scope: true dismiss => { lightbox_open = false; } } } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn dismiss_handler_inside_if_wrapped_container_without_modal_scope_rejected() {
        // Same shape as the accept test above, minus `modal-scope: true`.
        // Proves the recursion into the `if` body actually re-scans the
        // inner `Box`'s own children rather than short-circuiting or
        // inheriting an outer answer.
        let errs = errors(
            "component C inherits W { state lightbox_open: bool = true Box { if lightbox_open { Box { dismiss => { lightbox_open = false; } } } } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`dismiss` handler can never be raised") && errs[0].contains("§4.19"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn dismiss_inside_if_wrapped_container_not_admitted_by_ancestor_modal_scope() {
        // The discriminating case: the *enclosing* `Box`, outside the
        // `if`, carries `modal-scope: true`, but the `if`-wrapped inner
        // `Box` carrying `dismiss` does not. The admission rule is
        // same-node, not ancestor — this is what a `carries_modal_scope`
        // that leaked from the outer recursive call into the inner one
        // (instead of being recomputed against the inner container's own
        // siblings) would wrongly accept.
        let errs = errors(
            "component C inherits W { state lightbox_open: bool = true Box { modal-scope: true if lightbox_open { Box { dismiss => { lightbox_open = false; } } } } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`dismiss` handler can never be raised") && errs[0].contains("§4.19"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn dismiss_handler_inside_for_wrapped_container_without_modal_scope_rejected() {
        // The `for` counterpart of the `if`-wrapped reject case above.
        // The pre-existing "handlers inside a `for` body template are
        // deferred in M3-Phase 7" gate (`inside_for_template`) fires
        // unconditionally on every handler reached inside a `for` body,
        // so a `dismiss` handler in a `for` body can only ever be a
        // reject case, never an accept one — `inside_for_template` is
        // threaded unchanged through the `Member::WidgetDecl` recursion,
        // so it stays `true` all the way down to this handler. Both
        // diagnostics are asserted: the deferred one proves that
        // pre-existing gate still runs, and the modal-scope one proves
        // the `Member::For` recursive call still reaches the handler and
        // re-scans its own siblings for `carries_modal_scope` rather than
        // the check being skipped because the handler was already
        // rejected for the other reason.
        let errs = errors(
            "component C inherits W { state open: bool = true state items: i32[] = [] VStack { for it, idx in items { Box { dismiss => { open = false; } } } } }",
        );
        assert_eq!(errs.len(), 2, "{:?}", errs);
        assert!(
            errs.iter()
                .any(|e| e
                    .contains("handlers inside a `for` body template are deferred in M3-Phase 7")),
            "{:?}",
            errs
        );
        assert!(
            errs.iter()
                .any(|e| e.contains("`dismiss` handler can never be raised")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn clicked_handler_on_grid_accepted() {
        // T8: `check_grid` no longer carries a per-kind signal rule, so
        // `clicked` — admitted on any widget per dsl_spec §4.19 — is
        // accepted on a Grid, same as on every other widget kind.
        let result = check_src(
            "component C inherits W { state count: i32 = 0 Grid { columns: 1* rows: 1* clicked => { count += 1; } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn clicked_handler_on_zstack_accepted() {
        // Symmetric pin to the Grid case above: `wasamoc check` never had
        // a per-kind signal rule for ZStack (only the runtime loader
        // did), so this already passed; pinned explicitly so the
        // Grid/ZStack admission pair is documented as symmetric.
        let result = check_src(
            "component C inherits W { state count: i32 = 0 ZStack { clicked => { count += 1; } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn dismiss_handler_on_grid_without_modal_scope_rejected_by_generic_rule() {
        // Proves the generic dsl_spec §4.19 `dismiss` rule now solely
        // owns Grid admission: the diagnostic is the generic "can never
        // be raised" message, not the removed Grid-specific "takes no
        // signal handlers other than `dismiss`" message — a Grid-specific
        // rejection could not satisfy this assertion.
        let errs = errors(
            "component C inherits W { state open: bool = true Grid { columns: 1* rows: 1* dismiss => { open = false; } } }",
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`dismiss` handler can never be raised"),
            "{:?}",
            errs
        );
        assert!(!errs[0].contains("takes no signal handlers"), "{:?}", errs);
    }

    // `dismiss` on a Grid carrying `modal-scope: true` is pinned above by
    // `dismiss_handler_accepted_on_grid_carrying_modal_scope`, which
    // doubles as this pair's accept-side case.

    // ── M4-Phase 2 T8: `key-down("<key>")` argument rules (dsl_spec §4.19
    // "Keyboard input", DD-M4-P2-005) ───────────────────────────────────

    #[test]
    fn key_down_without_argument_rejected() {
        // A bare `key-down => { }` (no parenthesised key name) parses,
        // but can never fire — the same "silently never fires" class the
        // `dismiss` rule guards against.
        let errs = errors("component C inherits W { Box { key-down => { } } }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`key-down` handler can never be raised")
                && errs[0].contains("must be named")
                && errs[0].contains("key-down(\"ArrowLeft\")"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn key_down_unrecognised_key_name_rejected() {
        let errs = errors(r#"component C inherits W { Box { key-down("Tab") => { } } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("unrecognised key") && errs[0].contains("Tab"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn key_down_modifier_combo_rejected_as_unrecognised() {
        // Modifier combinations (`Ctrl+S`) are not in this surface — they
        // are simply an unrecognised name, no separate rule needed.
        let errs = errors(r#"component C inherits W { Box { key-down("Ctrl+S") => { } } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(errs[0].contains("unrecognised key"), "{:?}", errs);
    }

    #[test]
    fn argument_on_clicked_rejected() {
        let errs = errors(r#"component C inherits W { Box { clicked("x") => { } } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`clicked` does not take an argument") && errs[0].contains("key-down"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn argument_on_dismiss_rejected() {
        // `modal-scope: true` sibling present so only the argument rule
        // fires, not the `dismiss`/`carries_modal_scope` admission rule.
        let errs =
            errors(r#"component C inherits W { Box { modal-scope: true dismiss("x") => { } } }"#);
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`dismiss` does not take an argument"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn key_down_accepted_on_box() {
        let result = check_src(
            "component C inherits W { state selected_index: i32 = 0 Box { key-down(\"ArrowLeft\") => { root.selected_index -= 1; } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn key_down_accepted_on_button() {
        let result = check_src(
            "component C inherits W { state selected_index: i32 = 0 Button { key-down(\"ArrowLeft\") => { root.selected_index -= 1; } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn key_down_accepted_on_grid() {
        let result = check_src(
            "component C inherits W { state selected_index: i32 = 0 Grid { columns: 1* rows: 1* key-down(\"ArrowLeft\") => { root.selected_index -= 1; } } }",
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn key_down_recognised_key_names_spot_check_accepted() {
        // Representative sample across `RECOGNISED_KEY_NAMES`, not the
        // full 22 (that exhaustive coverage lives on the
        // `wasamo_ir::RECOGNISED_KEY_NAMES` table test).
        for key in ["Escape", "Enter", "F12", "PageDown"] {
            let src =
                format!(r#"component C inherits W {{ Box {{ key-down("{key}") => {{ }} }} }}"#);
            let result = check_src(&src);
            assert!(!result.has_errors(), "key={key}: {:?}", result.diagnostics);
        }
    }
}
