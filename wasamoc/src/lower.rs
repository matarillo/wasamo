use crate::ast::{AssignOp, Block, ComponentDef, Expr, Member, Statement, StringPart, TypeName};
use crate::check::Namespace;
use crate::ir::{
    CompoundOp, HandlerExpr, InterpolationPart, IrBinding, IrComponent, IrHandler, IrLiteral,
    IrNode, IrProp, IrState, IrType,
};

/// Lower a checked AST to the IR representation.
/// Panics if the component has no widget child (caller must ensure check passed).
pub fn lower(ast: &ComponentDef, ns: &Namespace) -> IrComponent {
    let mut states = Vec::new();
    let mut widget_members: Vec<&Member> = Vec::new();

    for member in &ast.members {
        match member {
            Member::StateMember {
                name, ty, default, ..
            } => {
                states.push(lower_state(name, ty, default));
            }
            Member::PropertyDecl { .. } => {}
            _ => widget_members.push(member),
        }
    }

    // The component body contains property-binds and exactly one root widget.
    // Collect component-level prop-binds, then find the root widget node.
    let mut comp_props = Vec::new();
    let mut comp_bindings = Vec::new();
    let mut root_opt: Option<IrNode> = None;

    for member in &widget_members {
        match member {
            Member::PropertyBind { name, value, .. } => match lower_expr(value, ns) {
                LoweredExpr::Static(lit) => comp_props.push(IrProp {
                    name: name.clone(),
                    value: lit,
                }),
                LoweredExpr::Dynamic(expr) => comp_bindings.push(IrBinding {
                    prop_name: name.clone(),
                    expr,
                }),
            },
            Member::WidgetDecl {
                type_name, members, ..
            } => {
                root_opt = Some(lower_node(type_name, members, ns));
            }
            Member::SignalHandler { .. } => {}
            _ => {}
        }
    }

    let mut root = root_opt.expect("component must contain a root widget node");
    // Component-level props/bindings (e.g. title, backdrop) belong on the root node.
    root.props.splice(0..0, comp_props);
    root.bindings.splice(0..0, comp_bindings);

    IrComponent {
        name: ast.name.clone(),
        base: ast.base.clone(),
        states,
        root,
    }
}

fn lower_state(name: &str, ty: &TypeName, default: &Expr) -> IrState {
    let ir_type = match ty {
        TypeName::Int => IrType::I32,
        TypeName::Str => IrType::Str,
        TypeName::Bool => IrType::Bool,
        _ => panic!("lower_state: unsupported type (check should have rejected this)"),
    };
    let ir_default = match default {
        Expr::IntLit { value, .. } => IrLiteral::Int(*value as i32),
        Expr::StringLit { parts, .. } => IrLiteral::Str(string_parts_to_static(parts)),
        Expr::BoolLit { value, .. } => IrLiteral::Bool(*value),
        // Ratio / Color literals in a `state` default are a positional
        // error rejected by `wasamoc check` (DD-M3-P2-002 / 003 confine
        // them to `Box.aspect` / `Box.fill` RHS). The positional reject
        // is wired up in T3; the catch-all below already covers them in
        // the meantime.
        _ => panic!("lower_state: unsupported default (check should have rejected this)"),
    };
    IrState {
        name: name.to_string(),
        ty: ir_type,
        default: ir_default,
    }
}

fn lower_node(widget_type: &str, members: &[Member], ns: &Namespace) -> IrNode {
    let mut props = Vec::new();
    let mut bindings = Vec::new();
    let mut handlers = Vec::new();
    let mut children = Vec::new();

    for member in members {
        match member {
            Member::PropertyBind { name, value, .. } => match lower_expr(value, ns) {
                LoweredExpr::Static(lit) => props.push(IrProp {
                    name: name.clone(),
                    value: lit,
                }),
                LoweredExpr::Dynamic(expr) => bindings.push(IrBinding {
                    prop_name: name.clone(),
                    expr,
                }),
            },
            Member::SignalHandler { signal, body, .. } => {
                handlers.push(IrHandler {
                    signal: signal.clone(),
                    expr: lower_block(body, ns),
                });
            }
            Member::WidgetDecl {
                type_name,
                members: child_members,
                ..
            } => {
                children.push(lower_node(type_name, child_members, ns));
            }
            Member::StateMember { .. } | Member::PropertyDecl { .. } => {}
        }
    }

    IrNode {
        widget_type: widget_type.to_string(),
        props,
        bindings,
        handlers,
        children,
    }
}

enum LoweredExpr {
    Static(IrLiteral),
    Dynamic(HandlerExpr),
}

fn lower_expr(expr: &Expr, ns: &Namespace) -> LoweredExpr {
    match expr {
        Expr::IntLit { value, .. } => LoweredExpr::Static(IrLiteral::Int(*value as i32)),
        Expr::BoolLit { value, .. } => LoweredExpr::Static(IrLiteral::Bool(*value)),
        Expr::Ident { name, .. } => {
            // Per DD-M3-P1-010: identifier resolution at lowering time
            // consults the state-type table. A name that matches a declared
            // `state` becomes a typed `*PropRead` (reactive binding). Names
            // not in `ns` (keyword-valued idents like `mica` / `system` /
            // `accent`) stay as static `IrLiteral::Ident`.
            match ns.get(name) {
                Some(TypeName::Bool) => {
                    LoweredExpr::Dynamic(HandlerExpr::BoolPropRead { path: name.clone() })
                }
                Some(TypeName::Str) => {
                    LoweredExpr::Dynamic(HandlerExpr::StrPropRead { path: name.clone() })
                }
                Some(TypeName::Int) => {
                    LoweredExpr::Dynamic(HandlerExpr::PropRead { path: name.clone() })
                }
                Some(TypeName::Float) | None => LoweredExpr::Static(IrLiteral::Ident(name.clone())),
            }
        }
        Expr::Measurement { value, .. } => LoweredExpr::Static(IrLiteral::Int(*value as i32)),
        Expr::StringLit { parts, .. } => {
            if is_static_string(parts) {
                LoweredExpr::Static(IrLiteral::Str(string_parts_to_static(parts)))
            } else {
                LoweredExpr::Dynamic(HandlerExpr::Interpolation(lower_string_parts(parts, ns)))
            }
        }
        Expr::FloatLit { .. } => {
            panic!("lower_expr: float not supported (check should have rejected this)");
        }
        // T4 will replace these stubs with `IrLiteral::Ratio { num, den }`
        // and `IrLiteral::Color(value)` lowering for `Box.aspect` /
        // `Box.fill`. The IR variants already exist (T1). Until T4 these
        // are unreachable through tests because T2 emits no lowering
        // exercises for ratio / color.
        Expr::RatioLit { .. } => {
            panic!("lower_expr: ratio literal lowering arrives in T4 (M3-Phase 2)");
        }
        Expr::ColorLit { .. } => {
            panic!("lower_expr: color literal lowering arrives in T4 (M3-Phase 2)");
        }
    }
}

fn is_static_string(parts: &[StringPart]) -> bool {
    parts.iter().all(|p| matches!(p, StringPart::Text(_)))
}

fn string_parts_to_static(parts: &[StringPart]) -> String {
    parts
        .iter()
        .map(|p| match p {
            StringPart::Text(s) => s.clone(),
            StringPart::Interp(_) => panic!("string_parts_to_static: called on dynamic string"),
        })
        .collect()
}

fn lower_string_parts(parts: &[StringPart], ns: &Namespace) -> Vec<InterpolationPart> {
    parts
        .iter()
        .map(|p| match p {
            StringPart::Text(s) => InterpolationPart::Literal(s.clone()),
            StringPart::Interp(qn) => {
                let path = qn.segments.last().cloned().unwrap_or_default();
                let expr = match ns.get(&path) {
                    Some(TypeName::Str) => HandlerExpr::StrPropRead { path },
                    Some(TypeName::Bool) => HandlerExpr::BoolPropRead { path },
                    _ => HandlerExpr::PropRead { path },
                };
                InterpolationPart::Expr(expr)
            }
        })
        .collect()
}

fn lower_block(block: &Block, ns: &Namespace) -> HandlerExpr {
    let mut exprs: Vec<HandlerExpr> = block
        .statements
        .iter()
        .map(|s| lower_statement(s, ns))
        .collect();
    if exprs.len() == 1 {
        exprs.remove(0)
    } else {
        HandlerExpr::Block(exprs)
    }
}

fn lower_statement(stmt: &Statement, ns: &Namespace) -> HandlerExpr {
    let lhs = stmt.target.segments.last().cloned().unwrap_or_default();
    let rhs = Box::new(lower_rhs_expr(&stmt.value, ns));
    match stmt.op {
        AssignOp::Eq => HandlerExpr::Assign { lhs, rhs },
        AssignOp::PlusEq => HandlerExpr::CompoundAssign {
            op: CompoundOp::Add,
            lhs,
            rhs,
        },
        AssignOp::MinusEq => HandlerExpr::CompoundAssign {
            op: CompoundOp::Sub,
            lhs,
            rhs,
        },
        AssignOp::MulEq => HandlerExpr::CompoundAssign {
            op: CompoundOp::Mul,
            lhs,
            rhs,
        },
        AssignOp::DivEq => HandlerExpr::CompoundAssign {
            op: CompoundOp::Div,
            lhs,
            rhs,
        },
    }
}

fn lower_rhs_expr(expr: &Expr, ns: &Namespace) -> HandlerExpr {
    match expr {
        Expr::IntLit { value, .. } => HandlerExpr::IntLit(*value as i32),
        Expr::BoolLit { value, .. } => HandlerExpr::BoolLit(*value),
        Expr::StringLit { parts, .. } => {
            if is_static_string(parts) {
                HandlerExpr::StrLit(string_parts_to_static(parts))
            } else {
                HandlerExpr::Interpolation(lower_string_parts(parts, ns))
            }
        }
        // Per DD-M3-P1-010: identifier resolution in handler RHS consults
        // the state-type table — `bool` → `BoolPropRead`, `String` →
        // `StrPropRead`, `i32` → `PropRead`. An ident not in `ns` keeps the
        // M2 i32-implicit `PropRead` shape (runtime will reject if the
        // surrounding assignment is bool-typed).
        Expr::Ident { name, .. } => match ns.get(name) {
            Some(TypeName::Bool) => HandlerExpr::BoolPropRead { path: name.clone() },
            Some(TypeName::Str) => HandlerExpr::StrPropRead { path: name.clone() },
            _ => HandlerExpr::PropRead { path: name.clone() },
        },
        Expr::Measurement { value, .. } => HandlerExpr::IntLit(*value as i32),
        Expr::FloatLit { .. } => panic!("lower_rhs_expr: float not supported"),
        // Ratio / Color literals in handler RHS position are a positional
        // error rejected by `wasamoc check` per DD-M3-P2-004 (constant-only
        // contract); see lower_state above.
        Expr::RatioLit { .. } => {
            panic!("lower_rhs_expr: ratio literal in handler RHS (check should have rejected)")
        }
        Expr::ColorLit { .. } => {
            panic!("lower_rhs_expr: color literal in handler RHS (check should have rejected)")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::check;
    use crate::lexer::tokenize;
    use crate::parser::parse;

    fn lower_src(src: &str) -> IrComponent {
        let tokens = tokenize(src, "<test>").unwrap();
        let ast = parse(&tokens, "<test>").unwrap();
        let result = check(&ast, "<test>");
        assert!(
            !result.has_errors(),
            "check errors: {:?}",
            result.diagnostics
        );
        lower(&ast, &result.namespace)
    }

    #[test]
    fn state_lowered_to_ir_state() {
        let comp = lower_src("component C inherits W { state count: i32 = 0 VStack {} }");
        assert_eq!(comp.states.len(), 1);
        assert_eq!(
            comp.states[0],
            IrState {
                name: "count".into(),
                ty: IrType::I32,
                default: IrLiteral::Int(0)
            }
        );
    }

    #[test]
    fn static_prop_bind_lowered_to_ir_prop() {
        let comp = lower_src(r#"component C inherits W { VStack { spacing: 12px } }"#);
        let root_child = &comp.root;
        assert_eq!(root_child.props.len(), 1);
        assert_eq!(root_child.props[0].name, "spacing");
        assert_eq!(root_child.props[0].value, IrLiteral::Int(12));
    }

    #[test]
    fn ident_prop_bind_lowered_to_ir_prop_ident() {
        let comp = lower_src("component C inherits W { VStack { theme: system } }");
        let prop = &comp.root.props[0];
        assert_eq!(prop.name, "theme");
        assert_eq!(prop.value, IrLiteral::Ident("system".into()));
    }

    #[test]
    fn dynamic_string_interp_lowered_to_ir_binding() {
        let comp = lower_src(
            r#"component C inherits W { state count: i32 = 0 VStack { text: "Count: \{root.count}" } }"#,
        );
        let vstack = &comp.root;
        assert_eq!(vstack.bindings.len(), 1);
        let b = &vstack.bindings[0];
        assert_eq!(b.prop_name, "text");
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
    fn dynamic_string_interp_uses_str_prop_read_for_string_state() {
        let comp = lower_src(
            r#"component C inherits W { state label: string = "Ready" VStack { text: "State: \{root.label}" } }"#,
        );
        let vstack = &comp.root;
        assert_eq!(vstack.bindings.len(), 1);
        let b = &vstack.bindings[0];
        assert_eq!(b.prop_name, "text");
        assert_eq!(
            b.expr,
            HandlerExpr::Interpolation(vec![
                InterpolationPart::Literal("State: ".into()),
                InterpolationPart::Expr(HandlerExpr::StrPropRead {
                    path: "label".into()
                }),
            ])
        );
    }

    #[test]
    fn signal_handler_compound_assign_lowered() {
        let comp = lower_src(
            "component C inherits W { state count: i32 = 0 VStack { clicked => { root.count += 1; } } }",
        );
        let vstack = &comp.root;
        assert_eq!(vstack.handlers.len(), 1);
        let h = &vstack.handlers[0];
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
    fn bool_state_lowered_to_ir_state() {
        let comp = lower_src("component C inherits W { state ready: bool = false VStack {} }");
        assert_eq!(comp.states.len(), 1);
        assert_eq!(
            comp.states[0],
            IrState {
                name: "ready".into(),
                ty: IrType::Bool,
                default: IrLiteral::Bool(false),
            }
        );
    }

    #[test]
    fn bool_literal_prop_bind_lowered_to_ir_prop() {
        let comp = lower_src("component C inherits W { Button { enabled: true } }");
        let prop = &comp.root.props[0];
        assert_eq!(prop.name, "enabled");
        assert_eq!(prop.value, IrLiteral::Bool(true));
    }

    #[test]
    fn bool_literal_in_handler_lowered_to_handler_expr() {
        let comp = lower_src(
            "component C inherits W { state ready: bool = true Button { clicked => { root.ready = false; } } }",
        );
        let button = &comp.root;
        let h = &button.handlers[0];
        assert_eq!(h.signal, "clicked");
        assert_eq!(
            h.expr,
            HandlerExpr::Assign {
                lhs: "ready".into(),
                rhs: Box::new(HandlerExpr::BoolLit(false)),
            }
        );
    }

    #[test]
    fn bool_state_ident_in_prop_bind_lowered_to_bool_prop_read_binding() {
        // Per DD-M3-P1-010 identifier-resolution row: a bind whose RHS is
        // an ident matching a `bool` state lowers to a reactive
        // `BoolPropRead` binding (not a static `IrLiteral::Ident`).
        let comp = lower_src(
            "component C inherits W { state ready: bool = true Button { enabled: ready } }",
        );
        let button = &comp.root;
        assert!(
            button.props.is_empty(),
            "expected no static prop, found {:?}",
            button.props
        );
        assert_eq!(button.bindings.len(), 1);
        let b = &button.bindings[0];
        assert_eq!(b.prop_name, "enabled");
        assert_eq!(
            b.expr,
            HandlerExpr::BoolPropRead {
                path: "ready".into(),
            }
        );
    }

    #[test]
    fn i32_state_ident_in_prop_bind_lowered_to_prop_read_binding() {
        // i32 state ident in prop-bind RHS lowers to the i32-implicit
        // `PropRead` (DD-M3-P1-003 Option A leaves `PropRead` as the
        // implicit-i32 variant). The catalog-soft `bind` target here
        // (`VStack.spacing`) is not type-checked so the unchanged
        // `*PropRead` shape is the lowering outcome.
        let comp =
            lower_src("component C inherits W { state count: i32 = 0 VStack { spacing: count } }");
        let vstack = &comp.root;
        assert!(vstack.props.is_empty());
        assert_eq!(vstack.bindings.len(), 1);
        assert_eq!(
            vstack.bindings[0].expr,
            HandlerExpr::PropRead {
                path: "count".into(),
            }
        );
    }

    #[test]
    fn string_state_ident_in_prop_bind_lowered_to_str_prop_read_binding() {
        let comp = lower_src(
            r#"component C inherits W { state label: string = "hi" Text { text: label } }"#,
        );
        let text = &comp.root;
        assert!(text.props.is_empty());
        assert_eq!(text.bindings.len(), 1);
        assert_eq!(
            text.bindings[0].expr,
            HandlerExpr::StrPropRead {
                path: "label".into(),
            }
        );
    }

    #[test]
    fn keyword_ident_not_in_namespace_stays_static() {
        // `system`, `accent`, `mica` etc. are keyword-valued idents — they
        // are not state names so `lower_expr` leaves them as static
        // `IrLiteral::Ident`. Regression guard: ensure T4's typed
        // identifier lowering does NOT capture non-state idents into
        // reactive bindings.
        let comp = lower_src("component C inherits W { VStack { theme: system } }");
        let vstack = &comp.root;
        assert!(vstack.bindings.is_empty());
        assert_eq!(vstack.props.len(), 1);
        assert_eq!(vstack.props[0].value, IrLiteral::Ident("system".into()));
    }

    #[test]
    fn bool_state_ident_in_handler_rhs_lowered_to_bool_prop_read() {
        // Handler-side ident RHS picks the typed `*PropRead` from the
        // state-type table, mirroring the prop-bind path.
        let comp = lower_src(
            "component C inherits W { state ready: bool = true state other: bool = false Button { clicked => { root.ready = other; } } }",
        );
        let h = &comp.root.handlers[0];
        assert_eq!(
            h.expr,
            HandlerExpr::Assign {
                lhs: "ready".into(),
                rhs: Box::new(HandlerExpr::BoolPropRead {
                    path: "other".into(),
                }),
            }
        );
    }

    #[test]
    fn string_state_ident_in_handler_rhs_lowered_to_str_prop_read() {
        let comp = lower_src(
            r#"component C inherits W { state a: string = "x" state b: string = "y" Button { clicked => { root.a = b; } } }"#,
        );
        let h = &comp.root.handlers[0];
        assert_eq!(
            h.expr,
            HandlerExpr::Assign {
                lhs: "a".into(),
                rhs: Box::new(HandlerExpr::StrPropRead { path: "b".into() }),
            }
        );
    }

    #[test]
    fn nested_widget_lowered() {
        let comp = lower_src("component C inherits W { VStack { Text {} Button {} } }");
        let vstack = &comp.root;
        assert_eq!(vstack.children.len(), 2);
        assert_eq!(vstack.children[0].widget_type, "Text");
        assert_eq!(vstack.children[1].widget_type, "Button");
    }
}
