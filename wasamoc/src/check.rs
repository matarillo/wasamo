use std::collections::HashMap;

use crate::ast::{ComponentDef, Expr, Member, QualifiedName, Span, TypeName};
use crate::diagnostic::Diagnostic;

const KNOWN_WIDGET_TYPES: &[&str] = &[
    "VStack",
    "HStack",
    "Text",
    "Button",
    "Rectangle",
    "Box",
    "WrapPanel",
    "ScrollView",
];

/// WrapPanel's three constant-only `i32` attributes per dsl_spec §4.10
/// (DD-M3-P3-003 / DD-M3-P3-004). Listed in a single table so the
/// rejection paths (`bind`-style state-ident, non-`IntLit` literal,
/// attribute-outside-WrapPanel) share one source of truth.
const WRAPPANEL_INT_ATTRS: &[&str] = &["item-cross-size", "item-spacing", "line-spacing"];

/// Flat namespace of declared state names → their types.
pub type Namespace = HashMap<String, TypeName>;

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
    match expr {
        Expr::IntLit { .. } | Expr::Measurement { .. } => Some(TypeName::Int),
        Expr::FloatLit { .. } => Some(TypeName::Float),
        Expr::StringLit { .. } => Some(TypeName::Str),
        Expr::BoolLit { .. } => Some(TypeName::Bool),
        Expr::Ident { name, .. } => ns.get(name).cloned(),
        // Ratio / Color are Box-internal value types (DD-M3-P2-002 /
        // DD-M3-P2-003 Option A); they have no `TypeName` entry. Position
        // and value validity for these literals are checked at the
        // property-bind layer (T3), not via the state-type compatibility
        // table.
        Expr::RatioLit { .. } | Expr::ColorLit { .. } => None,
    }
}

fn types_compatible(a: &TypeName, b: &TypeName) -> bool {
    matches!(
        (a, b),
        (TypeName::Int, TypeName::Int)
            | (TypeName::Str, TypeName::Str)
            | (TypeName::Bool, TypeName::Bool)
            | (TypeName::Float, TypeName::Float)
    )
}

fn type_name_display(ty: &TypeName) -> &'static str {
    match ty {
        TypeName::Int => "i32",
        TypeName::Str => "string",
        TypeName::Float => "float",
        TypeName::Bool => "bool",
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
        Expr::Ident { name, span: ident_span } => match ns.get(name) {
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
fn check_scrollview_writable_offset_y(
    span: &Span,
    filename: &str,
    diags: &mut Vec<Diagnostic>,
) {
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
    let child_count = members
        .iter()
        .filter(|m| matches!(m, Member::WidgetDecl { .. }))
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

/// Second pass: validate widget types, property-bind types, and name references.
fn check_members(members: &[Member], filename: &str, ns: &Namespace, diags: &mut Vec<Diagnostic>) {
    check_members_inner(members, None, filename, ns, diags);
}

fn check_members_inner(
    members: &[Member],
    enclosing_widget: Option<&str>,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
) {
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
                // Box.aspect and Box.fill are constant-only per DD-M3-P2-004:
                // the RHS must be the matching literal kind, not a state-
                // backed ident or any other expression form. Validate here
                // and skip the generic `check_expr_type` path, which would
                // otherwise re-reject the literal positionally (the Ratio /
                // Color arm rejects every appearance outside this site).
                if enclosing_widget == Some("ScrollView") && name == "offset-y" {
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
                } else if enclosing_widget == Some("Box")
                    && (name.as_str() == "aspect" || name.as_str() == "fill")
                {
                    check_box_const_only_bind(name, value, span, filename, diags);
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
                    check_expr_type(value, span, filename, ns, diags);
                    check_property_bind_target(
                        enclosing_widget,
                        name,
                        value,
                        span,
                        filename,
                        ns,
                        diags,
                    );
                }
            }

            Member::WidgetDecl {
                type_name,
                members: children,
                span,
            } => {
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
                check_members_inner(children, Some(type_name), filename, ns, diags);
            }

            Member::SignalHandler { body, .. } => {
                for stmt in &body.statements {
                    check_qualified_name(&stmt.target, filename, ns, diags);
                    check_expr_type(&stmt.value, &stmt.span, filename, ns, diags);
                }
            }
        }
    }
}

/// Type-check a property binding's RHS against the target property's
/// declared type (if known via the widget catalog). Soft when either the
/// enclosing widget context or the property entry is unknown.
fn check_property_bind_target(
    enclosing_widget: Option<&str>,
    prop_name: &str,
    value: &Expr,
    span: &Span,
    filename: &str,
    ns: &Namespace,
    diags: &mut Vec<Diagnostic>,
) {
    let Some(widget) = enclosing_widget else {
        return;
    };
    let Some(target_ty) = widget_prop_type(widget, prop_name) else {
        return;
    };
    let Some(source_ty) = expr_static_type(value, ns) else {
        return;
    };
    if matches!(source_ty, TypeName::Float) {
        // FloatLit is rejected separately; avoid the redundant mismatch.
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
                // Keyword-valued idents (e.g. mica, system, accent, title) are not state refs.
                // State refs must resolve; plain keyword values pass through.
                // We treat single-segment idents that are NOT in the namespace as widget property
                // keyword values (e.g. `mica`, `system`). Only flag if it looks like a state ref
                // (i.e. matches a declared name or fails to match anything known).
                // Conservative: only reject if the namespace is non-empty and the name is
                // clearly a reference (starts with `root.` etc.). Plain single idents are
                // ambiguous (could be enum/keyword value); we don't reject them here.
                let _ = (name, span);
            }
            // String interpolation parts: check that Interp segments resolve
            // to declared state and stay within the currently supported
            // interpolation value types.
            if let Expr::StringLit { parts, .. } = expr {
                for part in parts {
                    if let crate::ast::StringPart::Interp(qn) = part {
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
    fn bind_component_level_no_type_check() {
        // Component-level prop binds (`title:`, `backdrop:`) have no
        // enclosing widget catalog — pass through.
        let result =
            check_src(r#"component C inherits W { title: "Counter" backdrop: mica VStack {} }"#);
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
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
        let errs = errors(
            r#"component C inherits W { ScrollView { VStack {} VStack {} } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("found 2") && errs[0].contains("`ScrollView`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_three_children_rejected() {
        let errs = errors(
            r#"component C inherits W { ScrollView { VStack {} HStack {} Box {} } }"#,
        );
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
        let result = check_src(
            r#"component C inherits W { ScrollView { offset-y: 0 VStack {} } }"#,
        );
        // The offset-y handling lands in the next commit; for now, the
        // child-count gate alone must not produce an error.
        let child_count_err = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == crate::diagnostic::Severity::Error)
            .any(|d| d.message.contains("`ScrollView` requires exactly one child"));
        assert!(
            !child_count_err,
            "child-count gate misfired: {:?}",
            result.diagnostics
        );
    }

    // --- T1: ScrollView offset-y literal accept (DD-M3-P4-003) ---

    #[test]
    fn scrollview_offset_y_positive_int_literal_accepted() {
        let result = check_src(
            r#"component C inherits W { ScrollView { offset-y: 42 VStack {} } }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn scrollview_offset_y_zero_literal_accepted() {
        let result = check_src(
            r#"component C inherits W { ScrollView { offset-y: 0 VStack {} } }"#,
        );
        assert!(!result.has_errors(), "{:?}", result.diagnostics);
    }

    #[test]
    fn scrollview_offset_y_negative_literal_accepted() {
        // DD-M3-P4-005 / DD-M3-P4-006: negative offsets are layout-time-
        // clamped to 0, not compile-time-rejected (explicitly distinct
        // from Phase 3 WrapPanel negative-literal rejection).
        let result = check_src(
            r#"component C inherits W { ScrollView { offset-y: -5 VStack {} } }"#,
        );
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
        let errs = errors(
            r#"component C inherits W { ScrollView { offset-y: "hello" VStack {} } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`")
                && errs[0].contains("`i32` literal"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_float_literal_rejected() {
        let errs = errors(
            r#"component C inherits W { ScrollView { offset-y: 1.5 VStack {} } }"#,
        );
        // FloatLit is rejected by `check_expr_type` globally; we don't
        // see the ScrollView-named diagnostic for floats because the
        // ScrollView path matches `_ =>` and the float arm is taken
        // before. Either error wording is acceptable as long as the
        // float is rejected.
        assert!(
            errs.iter().any(|e| e.contains("`ScrollView.offset-y`")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_color_literal_rejected() {
        let errs = errors(
            r#"component C inherits W { ScrollView { offset-y: #336699 VStack {} } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_ratio_literal_rejected() {
        let errs = errors(
            r#"component C inherits W { ScrollView { offset-y: 16:9 VStack {} } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_bool_literal_rejected() {
        let errs = errors(
            r#"component C inherits W { ScrollView { offset-y: true VStack {} } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`"),
            "{:?}",
            errs
        );
    }

    #[test]
    fn scrollview_offset_y_measurement_rejected() {
        // `12px` is `Token::Measurement` — not an `IntLit`. Reject per
        // dsl_spec §4.11 "i32 literal" surface contract.
        let errs = errors(
            r#"component C inherits W { ScrollView { offset-y: 12px VStack {} } }"#,
        );
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`ScrollView.offset-y`"),
            "{:?}",
            errs
        );
    }

    // --- T1: ScrollView offset-y bind-to-wrong-type-state reject ---

    #[test]
    fn scrollview_offset_y_undeclared_state_rejected() {
        let errs = errors(
            r#"component C inherits W { ScrollView { offset-y: scroll_y VStack {} } }"#,
        );
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
            errs[0].contains("`ScrollView.offset-y`")
                && errs[0].contains("declared `bool`"),
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
            errs[0].contains("`ScrollView.offset-y`")
                && errs[0].contains("declared `string`"),
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
            errs.iter()
                .any(|e| e.contains("`ScrollView.offset-y` is bindable read-only")
                    && e.contains("in-out")
                    && e.contains("M4")),
            "{:?}",
            errs
        );
    }

    #[test]
    fn wrappanel_attr_at_component_level_rejected() {
        let errs = errors("component C inherits W { line-spacing: 12 WrapPanel {} }");
        assert_eq!(errs.len(), 1, "{:?}", errs);
        assert!(
            errs[0].contains("`line-spacing` is a WrapPanel attribute")
                && errs[0].contains("component-level property"),
            "{:?}",
            errs
        );
    }
}
