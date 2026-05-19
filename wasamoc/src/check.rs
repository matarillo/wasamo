use std::collections::HashMap;

use crate::ast::{ComponentDef, Expr, Member, QualifiedName, Span, TypeName};
use crate::diagnostic::Diagnostic;

const M1_WIDGET_TYPES: &[&str] = &["VStack", "HStack", "Text", "Button", "Rectangle"];

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
        _ => None,
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

            Member::PropertyDecl { .. } => {}

            Member::PropertyBind { name, value, span } => {
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

            Member::WidgetDecl {
                type_name,
                members: children,
                span,
            } => {
                if !M1_WIDGET_TYPES.contains(&type_name.as_str()) {
                    diags.push(Diagnostic::warning(
                        filename,
                        span.line,
                        span.col,
                        format!(
                            "unknown widget type `{}`; known M1 types: {}",
                            type_name,
                            M1_WIDGET_TYPES.join(", ")
                        ),
                    ));
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
            // String interpolation parts: check that Interp segments resolve to declared state.
            if let Expr::StringLit { parts, .. } = expr {
                for part in parts {
                    if let crate::ast::StringPart::Interp(qn) = part {
                        check_qualified_name(qn, filename, ns, diags);
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
        let errs = errors(
            "component C inherits W { state ready: bool = true Text { text: ready } }",
        );
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
        let errs = errors(
            "component C inherits W { state x: i32 = 5 Button { enabled: x } }",
        );
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
}
