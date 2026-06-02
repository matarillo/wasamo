use crate::ast::{
    AssignOp, Block, ComponentDef, Expr, Member, Statement, StringPart, TrackAxis, TrackSize,
    TypeName,
};
use crate::check::Namespace;
use crate::ir::{
    CompoundOp, HandlerExpr, InterpolationPart, IrBinding, IrComponent, IrHandler, IrLiteral,
    IrNode, IrProp, IrState, IrType, KindPayload, TrackSize as IrTrackSize,
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
        // them to `Box.aspect` / `Box.fill` RHS); they fall through to
        // the catch-all below as a defense-in-depth panic.
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
    // Grid track lists (DD-M3-P5-001 carrier c1). Collected outside
    // `props` so `IrProp.value` stays strictly `IrLiteral`.
    let mut grid_columns: Option<Vec<IrTrackSize>> = None;
    let mut grid_rows: Option<Vec<IrTrackSize>> = None;

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
            Member::GridTracks { axis, tracks, .. } => {
                let lowered = tracks.iter().map(lower_track_size).collect();
                match axis {
                    TrackAxis::Columns => grid_columns = Some(lowered),
                    TrackAxis::Rows => grid_rows = Some(lowered),
                }
            }
            Member::StateMember { .. } | Member::PropertyDecl { .. } => {}
        }
    }

    // A Grid has both track lists by the time lowering runs (`wasamoc
    // check` rejects a Grid missing `columns:` or `rows:`). The payload
    // is present iff at least one track list was seen — i.e. iff this is
    // a Grid node.
    let kind_payload = match (grid_columns, grid_rows) {
        (Some(columns), Some(rows)) => Some(KindPayload::Grid { columns, rows }),
        (None, None) => None,
        // check guarantees both-or-neither; a one-sided shape reaching
        // lowering is a check-layer bug.
        _ => panic!("lower_node: Grid with only one track list (check should have rejected)"),
    };

    IrNode {
        widget_type: widget_type.to_string(),
        props,
        bindings,
        handlers,
        children,
        kind_payload,
    }
}

/// Lower a checked AST `TrackSize` to its IR sibling. Reached only after
/// `wasamoc check` has rejected every invalid form, so the `InvalidFloat`
/// / `Word` arms are defense-in-depth panics (mirroring the FloatLit /
/// Ratio handler-RHS panics elsewhere in this module).
fn lower_track_size(t: &TrackSize) -> IrTrackSize {
    match t {
        TrackSize::Fixed { value, .. } => IrTrackSize::Fixed(*value as i32),
        TrackSize::Star { weight, .. } => IrTrackSize::Star(*weight as u32),
        TrackSize::InvalidFloat { .. } => {
            panic!("lower_track_size: float track size (check should have rejected)")
        }
        TrackSize::Word { name, .. } => {
            panic!("lower_track_size: word track size `{name}` (check should have rejected)")
        }
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
        // Ratio / Color literals are admitted only as the RHS of
        // `Box.aspect` / `Box.fill` (DD-M3-P2-002 / 003 Option A —
        // surface literal → IR literal, no `PropertyValue` round-trip).
        // `wasamoc check` rejects them in every other position, so the
        // arms here are reached only on the accepted positions and lower
        // straight to their `IrLiteral` siblings.
        Expr::RatioLit { num, den, .. } => LoweredExpr::Static(IrLiteral::Ratio {
            num: *num,
            den: *den,
        }),
        Expr::ColorLit { value, .. } => LoweredExpr::Static(IrLiteral::Color(*value)),
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
        // Ratio / Color literals in handler RHS position are positional
        // errors rejected by `wasamoc check` (DD-M3-P2-002 / 003 confine
        // them to `Box.aspect` / `Box.fill`; DD-M3-P2-004 keeps both
        // constant-only at the bind site). The arms remain as
        // defense-in-depth panics, mirroring `FloatLit` above.
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

    // --- T4: Box ratio / color literal lowering -------------------------
    //
    // The accepted positions for `Expr::RatioLit` / `Expr::ColorLit` are
    // `Box.aspect` / `Box.fill` RHS (DD-M3-P2-002 / 003 Option A — surface
    // literal → IR literal, no `PropertyValue`). These tests pin the
    // straight-through lowering for representative `Box { ... }` shapes
    // from dsl_spec §4.9.

    fn box_root<'a>(comp: &'a IrComponent) -> &'a IrNode {
        assert_eq!(comp.root.widget_type, "Box");
        &comp.root
    }

    #[test]
    fn box_aspect_only_lowered_to_ir_ratio() {
        let comp = lower_src("component C inherits W { Box { aspect: 16:9 } }");
        let b = box_root(&comp);
        assert!(b.bindings.is_empty());
        assert_eq!(b.props.len(), 1);
        assert_eq!(b.props[0].name, "aspect");
        assert_eq!(b.props[0].value, IrLiteral::Ratio { num: 16, den: 9 });
        assert!(b.children.is_empty());
    }

    #[test]
    fn box_fill_only_opaque_lowered_to_ir_color() {
        let comp = lower_src("component C inherits W { Box { fill: #cccccc } }");
        let b = box_root(&comp);
        assert_eq!(b.props.len(), 1);
        assert_eq!(b.props[0].name, "fill");
        // `#RRGGBB` → `0xFFRRGGBB` (alpha defaulted to 0xFF per dsl_spec §8.2).
        assert_eq!(b.props[0].value, IrLiteral::Color(0xFF_CC_CC_CC));
    }

    #[test]
    fn box_fill_with_alpha_lowered_to_ir_color() {
        let comp = lower_src("component C inherits W { Box { fill: #00000080 } }");
        let b = box_root(&comp);
        assert_eq!(b.props.len(), 1);
        assert_eq!(b.props[0].name, "fill");
        assert_eq!(b.props[0].value, IrLiteral::Color(0x80_00_00_00));
    }

    #[test]
    fn box_aspect_and_fill_lowered_together() {
        let comp = lower_src("component C inherits W { Box { aspect: 16:9 fill: #ffffffff } }");
        let b = box_root(&comp);
        assert_eq!(b.props.len(), 2);
        let aspect = b
            .props
            .iter()
            .find(|p| p.name == "aspect")
            .expect("aspect prop");
        assert_eq!(aspect.value, IrLiteral::Ratio { num: 16, den: 9 });
        let fill = b
            .props
            .iter()
            .find(|p| p.name == "fill")
            .expect("fill prop");
        assert_eq!(fill.value, IrLiteral::Color(0xFF_FF_FF_FF));
    }

    #[test]
    fn box_with_text_child_placeholder_shape_lowered() {
        // The dsl_spec §4.9 normative placeholder shape:
        //   Box { aspect: 16:9; fill: #cccccc; Text { text: "Photo 12" } }
        let comp = lower_src(
            r#"component C inherits W { Box { aspect: 16:9 fill: #cccccc Text { text: "Photo 12" } } }"#,
        );
        let b = box_root(&comp);
        assert_eq!(b.props.len(), 2);
        assert_eq!(
            b.props
                .iter()
                .find(|p| p.name == "aspect")
                .map(|p| &p.value),
            Some(&IrLiteral::Ratio { num: 16, den: 9 })
        );
        assert_eq!(
            b.props.iter().find(|p| p.name == "fill").map(|p| &p.value),
            Some(&IrLiteral::Color(0xFF_CC_CC_CC))
        );
        assert_eq!(b.children.len(), 1);
        let child = &b.children[0];
        assert_eq!(child.widget_type, "Text");
        assert_eq!(child.props.len(), 1);
        assert_eq!(child.props[0].name, "text");
        assert_eq!(child.props[0].value, IrLiteral::Str("Photo 12".into()));
    }

    // --- T3: WrapPanel widget + i32 attribute lowering ------------------
    //
    // Phase 3 introduces no new IR variants — `WrapPanel { ... }` lowers
    // to a generic `IrNode { widget_type: "WrapPanel", ... }` and the
    // three `item-cross-size` / `item-spacing` / `line-spacing` attributes
    // lower via the existing `IrLiteral::Int` path. Absent attributes are
    // omitted from the IR; defaults are applied at the runtime layer in T5
    // (DD-M3-P3-003 / DD-M3-P3-004), not at the IR layer.
    //
    // These tests double as regression coverage that the generic parser
    // handles WrapPanel-shaped declarations without modification (no new
    // parser grammar per the progress-doc preamble).

    fn wrap_root<'a>(comp: &'a IrComponent) -> &'a IrNode {
        assert_eq!(comp.root.widget_type, "WrapPanel");
        &comp.root
    }

    fn find_prop<'a>(node: &'a IrNode, name: &str) -> Option<&'a IrLiteral> {
        node.props.iter().find(|p| p.name == name).map(|p| &p.value)
    }

    #[test]
    fn wrap_panel_zero_children_lowered() {
        let comp = lower_src("component C inherits W { WrapPanel {} }");
        let w = wrap_root(&comp);
        assert!(w.props.is_empty());
        assert!(w.bindings.is_empty());
        assert!(w.handlers.is_empty());
        assert!(w.children.is_empty());
    }

    #[test]
    fn wrap_panel_single_box_child_with_item_cross_size_lowered() {
        let comp = lower_src(
            "component C inherits W { WrapPanel { item-cross-size: 88 Box { aspect: 1:1 } } }",
        );
        let w = wrap_root(&comp);
        assert_eq!(w.props.len(), 1);
        assert_eq!(w.props[0].name, "item-cross-size");
        assert_eq!(w.props[0].value, IrLiteral::Int(88));
        assert_eq!(w.children.len(), 1);
        assert_eq!(w.children[0].widget_type, "Box");
        assert_eq!(
            find_prop(&w.children[0], "aspect"),
            Some(&IrLiteral::Ratio { num: 1, den: 1 })
        );
    }

    #[test]
    fn wrap_panel_multi_box_child_all_three_attributes_lowered() {
        let comp = lower_src(
            "component C inherits W { WrapPanel { item-cross-size: 96 item-spacing: 8 line-spacing: 12 Box { aspect: 1:1 } Box { aspect: 1:1 } Box { aspect: 1:1 } } }",
        );
        let w = wrap_root(&comp);
        assert_eq!(w.props.len(), 3);
        assert_eq!(find_prop(w, "item-cross-size"), Some(&IrLiteral::Int(96)));
        assert_eq!(find_prop(w, "item-spacing"), Some(&IrLiteral::Int(8)));
        assert_eq!(find_prop(w, "line-spacing"), Some(&IrLiteral::Int(12)));
        assert_eq!(w.children.len(), 3);
        for child in &w.children {
            assert_eq!(child.widget_type, "Box");
        }
    }

    #[test]
    fn wrap_panel_multi_child_only_item_cross_size_set_omits_other_attrs() {
        // Only `item-cross-size` set; `item-spacing` / `line-spacing` are
        // absent from the IR. Defaults are applied at the runtime layer
        // in T5 (DD-M3-P3-003 / DD-M3-P3-004), not at the IR layer.
        let comp = lower_src(
            "component C inherits W { WrapPanel { item-cross-size: 64 Box { aspect: 1:1 } Box { aspect: 4:3 } } }",
        );
        let w = wrap_root(&comp);
        assert_eq!(w.props.len(), 1);
        assert_eq!(w.props[0].name, "item-cross-size");
        assert_eq!(w.props[0].value, IrLiteral::Int(64));
        assert!(
            find_prop(w, "item-spacing").is_none(),
            "absent attribute must be omitted from IR, found: {:?}",
            w.props,
        );
        assert!(
            find_prop(w, "line-spacing").is_none(),
            "absent attribute must be omitted from IR, found: {:?}",
            w.props,
        );
        assert_eq!(w.children.len(), 2);
    }

    #[test]
    fn wrap_panel_multi_child_no_attributes_set_lowered() {
        // All three attributes absent; the WrapPanel IR node carries zero
        // props and only the children list. Text children avoid the T2
        // aspect-only-Box warning so the test stays focused on lowering.
        let comp = lower_src("component C inherits W { WrapPanel { Text {} Text {} Text {} } }");
        let w = wrap_root(&comp);
        assert!(w.props.is_empty());
        assert!(w.bindings.is_empty());
        assert_eq!(w.children.len(), 3);
        for child in &w.children {
            assert_eq!(child.widget_type, "Text");
        }
    }

    // --- M3-Phase 5 T1: Grid carrier c1 lowering (DD-M3-P5-001) ---------
    //
    // Grid track lists lower into `KindPayload::Grid` on the Grid IrNode
    // (not into `IrProp`); Cell wrappers lower as ordinary child IrNodes
    // (`widget_type: "Cell"`) carrying their placement props as
    // `IrLiteral` entries. The runtime loader flattens Cell subtrees in
    // T3; T1 only proves the carrier shape.

    fn grid_payload(node: &IrNode) -> (&[IrTrackSize], &[IrTrackSize]) {
        match node.kind_payload.as_ref() {
            Some(KindPayload::Grid { columns, rows }) => (columns, rows),
            other => panic!("expected Grid kind payload, got {other:?}"),
        }
    }

    #[test]
    fn grid_track_lists_lower_to_kind_payload() {
        let comp = lower_src(
            r#"component C inherits W {
                Grid {
                    columns: 180 1* 2*
                    rows: 1* 1*
                    Cell { row: 0 column: 0 Text { text: "x" } }
                }
            }"#,
        );
        let grid = &comp.root;
        assert_eq!(grid.widget_type, "Grid");
        let (columns, rows) = grid_payload(grid);
        assert_eq!(
            columns,
            &[
                IrTrackSize::Fixed(180),
                IrTrackSize::Star(1),
                IrTrackSize::Star(2)
            ]
        );
        assert_eq!(rows, &[IrTrackSize::Star(1), IrTrackSize::Star(1)]);
        // Track lists do NOT leak into props (IrProp stays IrLiteral-only).
        assert!(
            grid.props
                .iter()
                .all(|p| p.name != "columns" && p.name != "rows"),
            "track lists must not appear as IrProp entries: {:?}",
            grid.props
        );
    }

    #[test]
    fn grid_cell_lowers_as_child_node_with_placement_props() {
        let comp = lower_src(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1*
                    rows: 1*
                    Cell { row: 0 column: 1 h-align: center Text { text: "x" } }
                }
            }"#,
        );
        let grid = &comp.root;
        assert_eq!(grid.children.len(), 1);
        let cell = &grid.children[0];
        assert_eq!(cell.widget_type, "Cell");
        assert!(cell.kind_payload.is_none());
        assert_eq!(
            cell.props
                .iter()
                .find(|p| p.name == "row")
                .map(|p| &p.value),
            Some(&IrLiteral::Int(0))
        );
        assert_eq!(
            cell.props
                .iter()
                .find(|p| p.name == "column")
                .map(|p| &p.value),
            Some(&IrLiteral::Int(1))
        );
        assert_eq!(
            cell.props
                .iter()
                .find(|p| p.name == "h-align")
                .map(|p| &p.value),
            Some(&IrLiteral::Ident("center".into()))
        );
        // The Cell's content widget is its single child node.
        assert_eq!(cell.children.len(), 1);
        assert_eq!(cell.children[0].widget_type, "Text");
    }

    #[test]
    fn non_grid_node_has_no_kind_payload() {
        let comp = lower_src("component C inherits W { VStack { Text {} } }");
        assert!(comp.root.kind_payload.is_none());
    }

    #[test]
    fn wrap_panel_zero_valued_attributes_lowered_to_ir_int_zero() {
        // DD-M3-P3-006 zero-handling: zero is a *valid* setting on all
        // three attributes (rejection threshold is `< 0`, not `<= 0`).
        // Pin that a zero attribute is recorded in the IR rather than
        // being treated as absence.
        let comp = lower_src(
            "component C inherits W { WrapPanel { item-cross-size: 0 item-spacing: 0 line-spacing: 0 Text {} } }",
        );
        let w = wrap_root(&comp);
        assert_eq!(w.props.len(), 3);
        assert_eq!(find_prop(w, "item-cross-size"), Some(&IrLiteral::Int(0)));
        assert_eq!(find_prop(w, "item-spacing"), Some(&IrLiteral::Int(0)));
        assert_eq!(find_prop(w, "line-spacing"), Some(&IrLiteral::Int(0)));
    }

    // --- M3-Phase 6 T1: ZStack direct-child lowering (DD-M3-P6-001) -----

    #[test]
    fn zstack_lowers_as_direct_children_without_kind_payload() {
        let comp = lower_src(
            r#"component C inherits W {
                ZStack {
                    Box { fill: #00000080 }
                    Text { h-align: center v-align: end text: "caption" }
                }
            }"#,
        );

        let zstack = &comp.root;
        assert_eq!(zstack.widget_type, "ZStack");
        assert!(zstack.kind_payload.is_none());
        assert!(zstack.props.is_empty());
        assert_eq!(zstack.children.len(), 2);
        assert_eq!(zstack.children[0].widget_type, "Box");
        assert_eq!(zstack.children[1].widget_type, "Text");
        assert_eq!(
            find_prop(&zstack.children[1], "h-align"),
            Some(&IrLiteral::Ident("center".into()))
        );
        assert_eq!(
            find_prop(&zstack.children[1], "v-align"),
            Some(&IrLiteral::Ident("end".into()))
        );
    }
}
