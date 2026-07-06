use crate::ast::{
    AssignOp, Block, BlockStatement, CollectionElemType, ComponentDef, Expr, Member, Statement,
    StringPart, TrackAxis, TrackSize, TypeName,
};
use crate::check::Namespace;
use crate::ir::{
    CompoundOp, ControlFlowBranch, ControlFlowNode, HandlerExpr, InterpolationPart, IrAlignment,
    IrBinding, IrChildSlot, IrComponent, IrHandler, IrLiteral, IrMember, IrNode, IrProp,
    IrSlotData, IrState, IrStateType, IrType, KindPayload, TrackSize as IrTrackSize,
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
            Member::PropertyBind { name, value, .. } => match lower_expr(value, ns, None) {
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
            Member::SignalHandler { .. } | Member::Conditional { .. } => {}
            _ => {}
        }
    }

    let root = root_opt.expect("component must contain a root widget node");

    IrComponent {
        name: ast.name.clone(),
        base: ast.base.clone(),
        host_props: comp_props,
        host_bindings: comp_bindings,
        states,
        root,
    }
}

fn lower_state(name: &str, ty: &TypeName, default: &Expr) -> IrState {
    let (ir_state_type, elem_type) = match ty {
        TypeName::Int => (IrStateType::Scalar(IrType::I32), IrType::I32),
        TypeName::Str => (IrStateType::Scalar(IrType::Str), IrType::Str),
        TypeName::Bool => (IrStateType::Scalar(IrType::Bool), IrType::Bool),
        TypeName::Collection(elem) => {
            let elem_type = lower_collection_elem_type(*elem);
            (IrStateType::Collection(elem_type.clone()), elem_type)
        }
        TypeName::Float => {
            panic!("lower_state: unsupported type (check should have rejected this)")
        }
    };
    let ir_default = match (ty, default) {
        (TypeName::Collection(_), Expr::ListLit { items, .. }) => IrLiteral::List(
            items
                .iter()
                .map(|item| lower_scalar_literal(item, &elem_type))
                .collect(),
        ),
        (_, Expr::IntLit { value, .. }) => IrLiteral::Int(*value as i32),
        (_, Expr::StringLit { parts, .. }) => IrLiteral::Str(string_parts_to_static(parts)),
        (_, Expr::BoolLit { value, .. }) => IrLiteral::Bool(*value),
        // Ratio / Color literals in a `state` default are a positional
        // error rejected by `wasamoc check` (DD-M3-P2-002 / 003 confine
        // them to `Box.aspect` / `Box.fill` RHS); they fall through to
        // the catch-all below as a defense-in-depth panic.
        _ => panic!("lower_state: unsupported default (check should have rejected this)"),
    };
    IrState {
        name: name.to_string(),
        ty: ir_state_type,
        default: ir_default,
    }
}

fn lower_collection_elem_type(elem: CollectionElemType) -> IrType {
    match elem {
        CollectionElemType::Int => IrType::I32,
        CollectionElemType::Str => IrType::Str,
        CollectionElemType::Bool => IrType::Bool,
    }
}

fn lower_scalar_literal(expr: &Expr, elem: &IrType) -> IrLiteral {
    match (elem, expr) {
        (IrType::I32, Expr::IntLit { value, .. }) => IrLiteral::Int(*value as i32),
        (IrType::Str, Expr::StringLit { parts, .. }) => {
            IrLiteral::Str(string_parts_to_static(parts))
        }
        (IrType::Bool, Expr::BoolLit { value, .. }) => IrLiteral::Bool(*value),
        _ => panic!("lower_scalar_literal: wrong element type (check should have rejected)"),
    }
}

fn lower_any_scalar_literal(expr: &Expr) -> IrLiteral {
    match expr {
        Expr::IntLit { value, .. } => IrLiteral::Int(*value as i32),
        Expr::StringLit { parts, .. } => IrLiteral::Str(string_parts_to_static(parts)),
        Expr::BoolLit { value, .. } => IrLiteral::Bool(*value),
        _ => panic!("lower_any_scalar_literal: non-literal list item (check should have rejected)"),
    }
}

fn lower_node(widget_type: &str, members: &[Member], ns: &Namespace) -> IrNode {
    lower_node_with_loop(widget_type, members, ns, None)
}

struct LowerLoopContext<'a> {
    binder: &'a str,
    index_binder: Option<&'a str>,
}

fn lower_node_with_loop(
    widget_type: &str,
    members: &[Member],
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> IrNode {
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
            Member::PropertyBind { name, value, .. } => match lower_expr(value, ns, loop_ctx) {
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
                    expr: lower_block(body, ns, loop_ctx),
                });
            }
            Member::WidgetDecl {
                type_name,
                members: child_members,
                ..
            } => {
                children.push(IrMember::Widget(lower_child_slot_with_loop(
                    widget_type,
                    type_name,
                    child_members,
                    ns,
                    loop_ctx,
                )));
            }
            Member::Conditional {
                condition, body, ..
            } => {
                children.push(IrMember::ControlFlow(ControlFlowNode::If {
                    branches: vec![ControlFlowBranch {
                        condition: lower_condition_expr(condition, ns),
                        body: body
                            .iter()
                            .map(|m| lower_widget_body_member(widget_type, m, ns, loop_ctx))
                            .collect(),
                    }],
                }));
            }
            Member::For {
                binder,
                index_binder,
                collection,
                body,
                ..
            } => {
                let Expr::Ident { name, .. } = collection else {
                    panic!("lower_node: invalid for collection (check should have rejected)");
                };
                let elem = match ns.get(name) {
                    Some(TypeName::Collection(elem)) => lower_collection_elem_type(*elem),
                    _ => {
                        panic!("lower_node: non-collection for target (check should have rejected)")
                    }
                };
                let child_loop = LowerLoopContext {
                    binder,
                    index_binder: index_binder.as_deref(),
                };
                children.push(IrMember::ControlFlow(ControlFlowNode::For {
                    binder: binder.clone(),
                    index_binder: index_binder.clone(),
                    collection: HandlerExpr::ListPropRead {
                        path: name.clone(),
                        elem,
                    },
                    body: body
                        .iter()
                        .map(|m| lower_widget_body_member(widget_type, m, ns, Some(&child_loop)))
                        .collect(),
                }));
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

fn lower_widget_body_member(
    parent_widget_type: &str,
    member: &Member,
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> IrMember {
    match member {
        Member::WidgetDecl {
            type_name, members, ..
        } => IrMember::Widget(lower_child_slot_with_loop(
            parent_widget_type,
            type_name,
            members,
            ns,
            loop_ctx,
        )),
        _ => panic!("lower_widget_body_member: non-widget body (check should have rejected)"),
    }
}

fn lower_child_slot_with_loop(
    parent_widget_type: &str,
    widget_type: &str,
    members: &[Member],
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> IrChildSlot {
    match parent_widget_type {
        "Grid" if widget_type == "Cell" => lower_grid_cell_slot(members, ns, loop_ctx),
        "Grid" => lower_grid_direct_child_slot(widget_type, members, ns, loop_ctx),
        "ZStack" => lower_zstack_child_slot(widget_type, members, ns, loop_ctx),
        _ => IrChildSlot {
            node: lower_node_with_loop(widget_type, members, ns, loop_ctx),
            slot_data: None,
        },
    }
}

fn slot_key(name: &str) -> Option<&str> {
    name.strip_prefix("slot.")
}

fn lower_grid_cell_slot(
    members: &[Member],
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> IrChildSlot {
    let mut row = 0;
    let mut column = 0;
    let mut row_span = 1;
    let mut column_span = 1;
    let mut h_align = IrAlignment::Stretch;
    let mut v_align = IrAlignment::Stretch;
    let mut content: Option<(&str, &[Member])> = None;

    for member in members {
        match member {
            Member::PropertyBind { name, value, .. } => match name.as_str() {
                "row" => row = lower_i32_placement(value).max(0) as u32,
                "column" => column = lower_i32_placement(value).max(0) as u32,
                "row-span" => row_span = lower_i32_placement(value).max(1) as u32,
                "column-span" => column_span = lower_i32_placement(value).max(1) as u32,
                "h-align" => h_align = lower_alignment_placement(value),
                "v-align" => v_align = lower_alignment_placement(value),
                _ => {}
            },
            Member::WidgetDecl {
                type_name, members, ..
            } => content = Some((type_name.as_str(), members.as_slice())),
            _ => {}
        }
    }

    let (type_name, child_members) =
        content.expect("lower_grid_cell_slot: Cell content missing (check should have rejected)");
    IrChildSlot {
        node: lower_node_with_loop(type_name, child_members, ns, loop_ctx),
        slot_data: Some(IrSlotData::Grid {
            row,
            column,
            row_span,
            column_span,
            h_align,
            v_align,
        }),
    }
}

fn lower_grid_direct_child_slot(
    widget_type: &str,
    members: &[Member],
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> IrChildSlot {
    let mut stripped_members = Vec::with_capacity(members.len());
    let mut has_slot_data = false;
    let mut row = 0;
    let mut column = 0;
    let mut row_span = 1;
    let mut column_span = 1;
    let mut h_align = IrAlignment::Stretch;
    let mut v_align = IrAlignment::Stretch;

    for member in members {
        match member {
            Member::PropertyBind { name, value, .. } => match slot_key(name) {
                Some("row") => {
                    has_slot_data = true;
                    row = lower_i32_placement(value).max(0) as u32;
                }
                Some("column") => {
                    has_slot_data = true;
                    column = lower_i32_placement(value).max(0) as u32;
                }
                Some("row-span") => {
                    has_slot_data = true;
                    row_span = lower_i32_placement(value).max(1) as u32;
                }
                Some("column-span") => {
                    has_slot_data = true;
                    column_span = lower_i32_placement(value).max(1) as u32;
                }
                Some("h-align") => {
                    has_slot_data = true;
                    h_align = lower_alignment_placement(value);
                }
                Some("v-align") => {
                    has_slot_data = true;
                    v_align = lower_alignment_placement(value);
                }
                Some(_) => {}
                None => stripped_members.push(member.clone()),
            },
            _ => stripped_members.push(member.clone()),
        }
    }

    IrChildSlot {
        node: lower_node_with_loop(widget_type, &stripped_members, ns, loop_ctx),
        slot_data: has_slot_data.then_some(IrSlotData::Grid {
            row,
            column,
            row_span,
            column_span,
            h_align,
            v_align,
        }),
    }
}

fn lower_zstack_child_slot(
    widget_type: &str,
    members: &[Member],
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> IrChildSlot {
    let mut stripped_members = Vec::with_capacity(members.len());
    let mut h_align: Option<IrAlignment> = None;
    let mut v_align: Option<IrAlignment> = None;

    for member in members {
        match member {
            Member::PropertyBind { name, value, .. } if name == "slot.h-align" => {
                h_align = Some(lower_alignment_placement(value));
            }
            Member::PropertyBind { name, value, .. } if name == "slot.v-align" => {
                v_align = Some(lower_alignment_placement(value));
            }
            _ => stripped_members.push(member.clone()),
        }
    }

    let slot_data = if h_align.is_some() || v_align.is_some() {
        Some(IrSlotData::ZStack {
            h_align: h_align.unwrap_or(IrAlignment::Center),
            v_align: v_align.unwrap_or(IrAlignment::Center),
        })
    } else {
        None
    };

    IrChildSlot {
        node: lower_node_with_loop(widget_type, &stripped_members, ns, loop_ctx),
        slot_data,
    }
}

fn lower_i32_placement(value: &Expr) -> i32 {
    match value {
        Expr::IntLit { value, .. } => *value as i32,
        _ => panic!("lower_i32_placement: non-int placement (check should have rejected)"),
    }
}

fn lower_alignment_placement(value: &Expr) -> IrAlignment {
    match value {
        Expr::Ident { name, .. } => match name.as_str() {
            "start" => IrAlignment::Start,
            "center" => IrAlignment::Center,
            "end" => IrAlignment::End,
            "stretch" => IrAlignment::Stretch,
            _ => panic!("lower_alignment_placement: bad alignment (check should have rejected)"),
        },
        _ => panic!("lower_alignment_placement: non-ident alignment (check should have rejected)"),
    }
}

fn lower_condition_expr(expr: &Expr, ns: &Namespace) -> HandlerExpr {
    match expr {
        Expr::BoolLit { value, .. } => HandlerExpr::BoolLit(*value),
        Expr::Ident { name, .. } => match ns.get(name) {
            Some(TypeName::Bool) => HandlerExpr::BoolPropRead { path: name.clone() },
            _ => panic!("lower_condition_expr: non-bool condition (check should have rejected)"),
        },
        _ => panic!("lower_condition_expr: unsupported condition (check should have rejected)"),
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

fn lower_expr(expr: &Expr, ns: &Namespace, loop_ctx: Option<&LowerLoopContext<'_>>) -> LoweredExpr {
    match expr {
        Expr::IntLit { value, .. } => LoweredExpr::Static(IrLiteral::Int(*value as i32)),
        Expr::BoolLit { value, .. } => LoweredExpr::Static(IrLiteral::Bool(*value)),
        Expr::Ident { name, .. } => {
            if let Some(ctx) = loop_ctx {
                if name == ctx.binder {
                    return LoweredExpr::Dynamic(HandlerExpr::ItemRead {
                        binder: name.clone(),
                    });
                }
                if ctx.index_binder == Some(name.as_str()) {
                    return LoweredExpr::Dynamic(HandlerExpr::IndexRead {
                        binder: name.clone(),
                    });
                }
            }
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
                Some(TypeName::Float) | Some(TypeName::Collection(_)) | None => {
                    LoweredExpr::Static(IrLiteral::Ident(name.clone()))
                }
            }
        }
        Expr::Measurement { value, .. } => LoweredExpr::Static(IrLiteral::Int(*value as i32)),
        Expr::StringLit { parts, .. } => {
            if is_static_string(parts) {
                LoweredExpr::Static(IrLiteral::Str(string_parts_to_static(parts)))
            } else {
                LoweredExpr::Dynamic(HandlerExpr::Interpolation(lower_string_parts(
                    parts, ns, loop_ctx,
                )))
            }
        }
        Expr::QualifiedRef { name } => {
            let path = name.segments.last().cloned().unwrap_or_default();
            match ns.get(&path) {
                Some(TypeName::Bool) => LoweredExpr::Dynamic(HandlerExpr::BoolPropRead { path }),
                Some(TypeName::Str) => LoweredExpr::Dynamic(HandlerExpr::StrPropRead { path }),
                Some(TypeName::Int) => LoweredExpr::Dynamic(HandlerExpr::PropRead { path }),
                Some(TypeName::Collection(_)) | Some(TypeName::Float) | None => {
                    LoweredExpr::Static(IrLiteral::Ident(path))
                }
            }
        }
        Expr::ListLit { .. } | Expr::CollectionCall { .. } => {
            panic!(
                "lower_expr: collection expression in prop position (check should have rejected)"
            )
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
        Expr::UnsupportedOperator { .. } => {
            panic!("lower_expr: unsupported operator (check should have rejected this)")
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

fn lower_string_parts(
    parts: &[StringPart],
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> Vec<InterpolationPart> {
    parts
        .iter()
        .map(|p| match p {
            StringPart::Text(s) => InterpolationPart::Literal(s.clone()),
            StringPart::Interp(qn) => {
                let path = qn.segments.last().cloned().unwrap_or_default();
                let expr = if let Some(ctx) = loop_ctx {
                    if path == ctx.binder {
                        HandlerExpr::ItemRead { binder: path }
                    } else if ctx.index_binder == Some(path.as_str()) {
                        HandlerExpr::IndexRead { binder: path }
                    } else {
                        lower_interp_state_read(path, ns)
                    }
                } else {
                    lower_interp_state_read(path, ns)
                };
                InterpolationPart::Expr(expr)
            }
        })
        .collect()
}

fn lower_interp_state_read(path: String, ns: &Namespace) -> HandlerExpr {
    match ns.get(&path) {
        Some(TypeName::Str) => HandlerExpr::StrPropRead { path },
        Some(TypeName::Bool) => HandlerExpr::BoolPropRead { path },
        _ => HandlerExpr::PropRead { path },
    }
}

fn lower_block(
    block: &Block,
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> HandlerExpr {
    let mut exprs: Vec<HandlerExpr> = block
        .statements
        .iter()
        .map(|s| lower_block_statement(s, ns, loop_ctx))
        .collect();
    if exprs.len() == 1 {
        exprs.remove(0)
    } else {
        HandlerExpr::Block(exprs)
    }
}

fn lower_block_statement(
    stmt: &BlockStatement,
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> HandlerExpr {
    match stmt {
        BlockStatement::Assignment(stmt) => lower_statement(stmt, ns, loop_ctx),
        BlockStatement::Expr(_) => {
            panic!("lower_block_statement: expression statement (check should have rejected)")
        }
    }
}

fn lower_statement(
    stmt: &Statement,
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> HandlerExpr {
    let lhs = stmt.target.segments.last().cloned().unwrap_or_default();
    let rhs = Box::new(lower_rhs_expr(&stmt.value, ns, loop_ctx));
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

fn lower_rhs_expr(
    expr: &Expr,
    ns: &Namespace,
    loop_ctx: Option<&LowerLoopContext<'_>>,
) -> HandlerExpr {
    match expr {
        Expr::IntLit { value, .. } => HandlerExpr::IntLit(*value as i32),
        Expr::BoolLit { value, .. } => HandlerExpr::BoolLit(*value),
        Expr::StringLit { parts, .. } => {
            if is_static_string(parts) {
                HandlerExpr::StrLit(string_parts_to_static(parts))
            } else {
                HandlerExpr::Interpolation(lower_string_parts(parts, ns, loop_ctx))
            }
        }
        // Per DD-M3-P1-010: identifier resolution in handler RHS consults
        // the state-type table — `bool` → `BoolPropRead`, `String` →
        // `StrPropRead`, `i32` → `PropRead`. An ident not in `ns` keeps the
        // M2 i32-implicit `PropRead` shape (runtime will reject if the
        // surrounding assignment is bool-typed).
        Expr::Ident { name, .. } => {
            if let Some(ctx) = loop_ctx {
                if name == ctx.binder {
                    return HandlerExpr::ItemRead {
                        binder: name.clone(),
                    };
                }
                if ctx.index_binder == Some(name.as_str()) {
                    return HandlerExpr::IndexRead {
                        binder: name.clone(),
                    };
                }
            }
            match ns.get(name) {
                Some(TypeName::Bool) => HandlerExpr::BoolPropRead { path: name.clone() },
                Some(TypeName::Str) => HandlerExpr::StrPropRead { path: name.clone() },
                _ => HandlerExpr::PropRead { path: name.clone() },
            }
        }
        Expr::Measurement { value, .. } => HandlerExpr::IntLit(*value as i32),
        Expr::QualifiedRef { name } => {
            let path = name.segments.last().cloned().unwrap_or_default();
            match ns.get(&path) {
                Some(TypeName::Bool) => HandlerExpr::BoolPropRead { path },
                Some(TypeName::Str) => HandlerExpr::StrPropRead { path },
                _ => HandlerExpr::PropRead { path },
            }
        }
        Expr::ListLit { items, .. } => {
            HandlerExpr::ListLit(items.iter().map(lower_any_scalar_literal).collect())
        }
        Expr::CollectionCall {
            receiver,
            method,
            args,
            ..
        } => {
            let path = receiver.segments.last().cloned().unwrap_or_default();
            let elem = match ns.get(&path) {
                Some(TypeName::Collection(elem)) => lower_collection_elem_type(*elem),
                _ => panic!("lower_rhs_expr: collection call receiver not collection"),
            };
            match method.as_str() {
                "append" => HandlerExpr::ListAppend {
                    path,
                    elem,
                    value: Box::new(lower_rhs_expr(&args[0], ns, loop_ctx)),
                },
                "drop-last" => HandlerExpr::ListDropLast { path, elem },
                _ => panic!("lower_rhs_expr: unknown collection method"),
            }
        }
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
        Expr::UnsupportedOperator { .. } => {
            panic!("lower_rhs_expr: unsupported operator (check should have rejected)")
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

    fn child_widget<'a>(node: &'a IrNode, index: usize) -> &'a IrNode {
        match &node.children[index] {
            IrMember::Widget(slot) => &slot.node,
            other => panic!("expected widget child at {index}, got {other:?}"),
        }
    }

    #[test]
    fn state_lowered_to_ir_state() {
        let comp = lower_src("component C inherits W { state count: i32 = 0 VStack {} }");
        assert_eq!(comp.states.len(), 1);
        assert_eq!(
            comp.states[0],
            IrState {
                name: "count".into(),
                ty: IrStateType::Scalar(IrType::I32),
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
    fn component_host_prop_lowers_to_host_surface() {
        let comp = lower_src(r#"component C inherits W { title: "Counter" VStack {} }"#);
        assert!(comp.root.props.is_empty());
        assert_eq!(comp.host_props.len(), 1);
        assert_eq!(comp.host_props[0].name, "title");
        assert_eq!(comp.host_props[0].value, IrLiteral::Str("Counter".into()));
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
                ty: IrStateType::Scalar(IrType::Bool),
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
        assert_eq!(child_widget(vstack, 0).widget_type, "Text");
        assert_eq!(child_widget(vstack, 1).widget_type, "Button");
    }

    #[test]
    fn togglebutton_literal_checked_lowers_to_ir_prop() {
        let comp = lower_src(
            r#"component C inherits W {
                state selected: bool = true
                ToggleButton {
                    text: "Photos"
                    style: accent
                    enabled: true
                    checked: false
                    clicked => { selected = false; }
                }
            }"#,
        );
        let toggle = &comp.root;
        assert_eq!(toggle.widget_type, "ToggleButton");
        assert_eq!(toggle.props.len(), 4);
        assert!(toggle
            .props
            .iter()
            .any(|p| { p.name == "text" && p.value == IrLiteral::Str("Photos".into()) }));
        assert!(toggle
            .props
            .iter()
            .any(|p| { p.name == "style" && p.value == IrLiteral::Ident("accent".into()) }));
        assert!(toggle
            .props
            .iter()
            .any(|p| { p.name == "enabled" && p.value == IrLiteral::Bool(true) }));
        assert!(toggle
            .props
            .iter()
            .any(|p| { p.name == "checked" && p.value == IrLiteral::Bool(false) }));
        assert_eq!(toggle.handlers.len(), 1);
        assert_eq!(toggle.handlers[0].signal, "clicked");
    }

    #[test]
    fn togglebutton_absent_checked_lowers_no_ir_prop_or_binding() {
        let comp = lower_src(r#"component C inherits W { ToggleButton { text: "Albums" } }"#);
        let toggle = &comp.root;
        assert_eq!(toggle.widget_type, "ToggleButton");
        assert!(!toggle.props.iter().any(|p| p.name == "checked"));
        assert!(!toggle.bindings.iter().any(|b| b.prop_name == "checked"));
    }

    #[test]
    fn togglebutton_checked_binding_lowers_to_bool_prop_read() {
        let comp = lower_src(
            "component C inherits W { state selected: bool = true ToggleButton { checked: selected } }",
        );
        let toggle = &comp.root;
        assert_eq!(toggle.widget_type, "ToggleButton");
        assert!(toggle.props.is_empty(), "{:?}", toggle.props);
        assert_eq!(toggle.bindings.len(), 1);
        let binding = &toggle.bindings[0];
        assert_eq!(binding.prop_name, "checked");
        assert_eq!(
            binding.expr,
            HandlerExpr::BoolPropRead {
                path: "selected".into(),
            }
        );
    }

    #[test]
    fn togglebutton_alpha_tab_band_lowers_block_assignment_handlers() {
        let comp = lower_src(
            r#"component C inherits W {
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
            }"#,
        );
        let hstack = &comp.root;
        assert_eq!(hstack.widget_type, "HStack");
        assert_eq!(hstack.children.len(), 3);
        let albums = child_widget(hstack, 1);
        assert_eq!(albums.widget_type, "ToggleButton");
        assert_eq!(albums.bindings.len(), 1);
        assert_eq!(albums.bindings[0].prop_name, "checked");
        let HandlerExpr::Block(exprs) = &albums.handlers[0].expr else {
            panic!("expected block assignment handler");
        };
        assert_eq!(exprs.len(), 3);
        assert!(matches!(
            &exprs[0],
            HandlerExpr::Assign { lhs, rhs }
                if lhs == "all" && matches!(rhs.as_ref(), HandlerExpr::BoolLit(false))
        ));
        assert!(matches!(
            &exprs[1],
            HandlerExpr::Assign { lhs, rhs }
                if lhs == "albums" && matches!(rhs.as_ref(), HandlerExpr::BoolLit(true))
        ));
        assert!(matches!(
            &exprs[2],
            HandlerExpr::Assign { lhs, rhs }
                if lhs == "favorites" && matches!(rhs.as_ref(), HandlerExpr::BoolLit(false))
        ));
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
        let child = child_widget(b, 0);
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
        assert_eq!(child_widget(w, 0).widget_type, "Box");
        assert_eq!(
            find_prop(child_widget(w, 0), "aspect"),
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
        for child in w.widget_children() {
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
        for child in w.widget_children() {
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
    fn grid_cell_lowers_to_child_slot_with_grid_slot_data() {
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
        let IrMember::Widget(slot) = &grid.children[0] else {
            panic!("expected Grid child slot");
        };
        assert_eq!(slot.node.widget_type, "Text");
        assert!(slot.node.kind_payload.is_none());
        assert_eq!(
            slot.slot_data,
            Some(IrSlotData::Grid {
                row: 0,
                column: 1,
                row_span: 1,
                column_span: 1,
                h_align: IrAlignment::Center,
                v_align: IrAlignment::Stretch,
            })
        );
    }

    #[test]
    fn grid_direct_slot_lowers_to_same_grid_slot_data_as_cell() {
        let cell_comp = lower_src(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1*
                    rows: 1*
                    Cell { row: 0 column: 1 h-align: center Text { text: "x" } }
                }
            }"#,
        );
        let direct_comp = lower_src(
            r#"component C inherits W {
                Grid {
                    columns: 1* 1*
                    rows: 1*
                    Text { slot.row: 0 slot.column: 1 slot.h-align: center text: "x" }
                }
            }"#,
        );
        let IrMember::Widget(cell_slot) = &cell_comp.root.children[0] else {
            panic!("expected Cell-lowered slot");
        };
        let IrMember::Widget(direct_slot) = &direct_comp.root.children[0] else {
            panic!("expected direct slot");
        };
        assert_eq!(direct_slot.node.widget_type, "Text");
        assert_eq!(find_prop(&direct_slot.node, "slot.row"), None);
        assert_eq!(find_prop(&direct_slot.node, "slot.column"), None);
        assert_eq!(find_prop(&direct_slot.node, "slot.h-align"), None);
        assert_eq!(direct_slot.slot_data, cell_slot.slot_data);
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
    fn zstack_lowers_child_placement_to_slot_data() {
        let comp = lower_src(
            r#"component C inherits W {
                ZStack {
                    Box { fill: #00000080 }
                    Text { slot.h-align: center slot.v-align: end text: "caption" }
                }
            }"#,
        );

        let zstack = &comp.root;
        assert_eq!(zstack.widget_type, "ZStack");
        assert!(zstack.kind_payload.is_none());
        assert!(zstack.props.is_empty());
        assert_eq!(zstack.children.len(), 2);
        assert_eq!(child_widget(zstack, 0).widget_type, "Box");
        assert_eq!(child_widget(zstack, 1).widget_type, "Text");
        assert_eq!(find_prop(child_widget(zstack, 1), "h-align"), None);
        assert_eq!(find_prop(child_widget(zstack, 1), "v-align"), None);
        assert_eq!(find_prop(child_widget(zstack, 1), "slot.h-align"), None);
        assert_eq!(find_prop(child_widget(zstack, 1), "slot.v-align"), None);
        let IrMember::Widget(slot) = &zstack.children[1] else {
            panic!("expected ZStack child slot");
        };
        assert_eq!(
            slot.slot_data,
            Some(IrSlotData::ZStack {
                h_align: IrAlignment::Center,
                v_align: IrAlignment::End,
            })
        );
    }

    #[test]
    fn zstack_grid_child_slot_lowers_to_zstack_slot_data() {
        // M3-Phase 7b T6b non-GUI positive control: a `Grid` placed as a
        // direct `ZStack` child lowers its own `slot.h-align` / `slot.v-align`
        // into the ZStack child slot's `IrSlotData::ZStack`, and the Grid node
        // keeps its Grid identity (kind payload / its Cell child) while the
        // placement props do NOT remain on the Grid node. This pins that the
        // newly-accepted author form (T6b checker fix) actually reaches the
        // one slot record, not just that it passes `check`.
        let comp = lower_src(
            r#"component C inherits W {
                ZStack {
                    Grid { slot.h-align: end slot.v-align: start columns: 1* rows: 1* Cell { Text { text: "x" } } }
                }
            }"#,
        );
        let zstack = &comp.root;
        assert_eq!(zstack.widget_type, "ZStack");
        assert_eq!(zstack.children.len(), 1);

        // The Grid node keeps its identity and carries no leftover placement.
        let grid = child_widget(zstack, 0);
        assert_eq!(grid.widget_type, "Grid");
        assert!(grid.kind_payload.is_some(), "Grid tracks must survive");
        assert_eq!(find_prop(grid, "slot.h-align"), None);
        assert_eq!(find_prop(grid, "slot.v-align"), None);
        assert_eq!(grid.children.len(), 1, "Grid's Cell child must survive");

        // The placement rides the ZStack child slot, as ZStack slot data.
        let IrMember::Widget(slot) = &zstack.children[0] else {
            panic!("expected ZStack child slot");
        };
        assert_eq!(slot.node.widget_type, "Grid");
        assert_eq!(
            slot.slot_data,
            Some(IrSlotData::ZStack {
                h_align: IrAlignment::End,
                v_align: IrAlignment::Start,
            })
        );
    }

    #[test]
    fn zstack_slot_defaults_omitted_axis_to_center() {
        let comp = lower_src(
            r#"component C inherits W {
                ZStack {
                    Text { slot.h-align: end text: "caption" }
                }
            }"#,
        );
        let IrMember::Widget(slot) = &comp.root.children[0] else {
            panic!("expected ZStack child slot");
        };
        assert_eq!(
            slot.slot_data,
            Some(IrSlotData::ZStack {
                h_align: IrAlignment::End,
                v_align: IrAlignment::Center,
            })
        );
    }

    #[test]
    fn conditional_body_root_slot_lowers_under_zstack() {
        let comp = lower_src(
            r#"component C inherits W {
                state ready: bool = true
                ZStack { if ready { Text { slot.h-align: end } } }
            }"#,
        );
        let IrMember::ControlFlow(ControlFlowNode::If { branches }) = &comp.root.children[0] else {
            panic!("expected if child");
        };
        let IrMember::Widget(slot) = &branches[0].body[0] else {
            panic!("expected if body child slot");
        };
        assert_eq!(
            slot.slot_data,
            Some(IrSlotData::ZStack {
                h_align: IrAlignment::End,
                v_align: IrAlignment::Center,
            })
        );
    }

    #[test]
    fn conditional_lowers_to_control_flow_member() {
        let comp = lower_src(
            "component C inherits W { state ready: bool = true VStack { if ready { Text {} } } }",
        );
        let root = &comp.root;
        assert_eq!(root.children.len(), 1);
        match &root.children[0] {
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
            other => panic!("expected control-flow child, got {other:?}"),
        }
    }

    #[test]
    fn conditional_bool_literal_lowers_to_bool_lit_condition() {
        let comp = lower_src("component C inherits W { VStack { if true { Text {} } } }");
        match &comp.root.children[0] {
            IrMember::ControlFlow(ControlFlowNode::If { branches }) => {
                assert_eq!(branches[0].condition, HandlerExpr::BoolLit(true));
            }
            other => panic!("expected control-flow child, got {other:?}"),
        }
    }

    #[test]
    fn collection_state_and_for_loop_lower_to_ir() {
        let comp = lower_src(
            r#"component C inherits W {
                state labels: string[] = ["a", "b"]
                WrapPanel { for label, i in labels { Text { text: label } } }
            }"#,
        );
        assert_eq!(
            comp.states[0],
            IrState {
                name: "labels".into(),
                ty: IrStateType::Collection(IrType::Str),
                default: IrLiteral::List(vec![
                    IrLiteral::Str("a".into()),
                    IrLiteral::Str("b".into())
                ]),
            }
        );
        match &comp.root.children[0] {
            IrMember::ControlFlow(ControlFlowNode::For {
                binder,
                index_binder,
                collection,
                body,
            }) => {
                assert_eq!(binder, "label");
                assert_eq!(index_binder.as_deref(), Some("i"));
                assert_eq!(
                    collection,
                    &HandlerExpr::ListPropRead {
                        path: "labels".into(),
                        elem: IrType::Str,
                    }
                );
                let IrMember::Widget(text) = &body[0] else {
                    panic!("expected Text body");
                };
                assert_eq!(
                    text.node.bindings[0].expr,
                    HandlerExpr::ItemRead {
                        binder: "label".into()
                    }
                );
            }
            other => panic!("expected For control-flow, got {other:?}"),
        }
    }

    #[test]
    fn gallery_like_for_shape_lowers_single_box_body_and_external_mutations() {
        let comp = lower_src(
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

        let vstack = &comp.root;
        assert_eq!(vstack.widget_type, "VStack");
        let scroll = child_widget(vstack, 0);
        assert_eq!(scroll.widget_type, "ScrollView");
        let wrap = child_widget(scroll, 0);
        assert_eq!(wrap.widget_type, "WrapPanel");
        match &wrap.children[0] {
            IrMember::ControlFlow(ControlFlowNode::For {
                binder,
                index_binder,
                collection,
                body,
            }) => {
                assert_eq!(binder, "label");
                assert_eq!(index_binder.as_deref(), Some("i"));
                assert_eq!(
                    collection,
                    &HandlerExpr::ListPropRead {
                        path: "labels".into(),
                        elem: IrType::Str,
                    }
                );
                let IrMember::Widget(box_node) = &body[0] else {
                    panic!("expected Box body");
                };
                assert_eq!(box_node.node.widget_type, "Box");
                assert_eq!(box_node.node.children.len(), 1);
                let text = child_widget(&box_node.node, 0);
                let HandlerExpr::Interpolation(parts) = &text.bindings[0].expr else {
                    panic!("expected interpolation binding");
                };
                assert!(matches!(
                    &parts[1],
                    InterpolationPart::Expr(HandlerExpr::ItemRead { binder }) if binder == "label"
                ));
                assert!(matches!(
                    &parts[3],
                    InterpolationPart::Expr(HandlerExpr::IndexRead { binder }) if binder == "i"
                ));
            }
            other => panic!("expected For control-flow, got {other:?}"),
        }

        let add = child_widget(vstack, 1);
        let HandlerExpr::Assign { lhs, rhs } = &add.handlers[0].expr else {
            panic!("expected Add assignment handler");
        };
        assert_eq!(lhs, "labels");
        assert!(matches!(
            rhs.as_ref(),
            HandlerExpr::ListAppend { path, elem: IrType::Str, value }
                if path == "labels" && matches!(value.as_ref(), HandlerExpr::StrLit(v) if v == "S03")
        ));
        let remove = child_widget(vstack, 2);
        let HandlerExpr::Assign { lhs, rhs } = &remove.handlers[0].expr else {
            panic!("expected Remove assignment handler");
        };
        assert_eq!(lhs, "labels");
        assert!(matches!(
            rhs.as_ref(),
            HandlerExpr::ListDropLast { path, elem: IrType::Str } if path == "labels"
        ));
    }

    #[test]
    fn for_index_binder_lowers_to_index_read() {
        let comp = lower_src(
            r#"component C inherits W {
                state xs: i32[] = [1]
                WrapPanel { for x, i in xs { VStack { spacing: i } } }
            }"#,
        );
        match &comp.root.children[0] {
            IrMember::ControlFlow(ControlFlowNode::For { body, .. }) => {
                let IrMember::Widget(vstack) = &body[0] else {
                    panic!("expected VStack body");
                };
                assert_eq!(
                    vstack.node.bindings[0].expr,
                    HandlerExpr::IndexRead { binder: "i".into() }
                );
            }
            other => panic!("expected For control-flow, got {other:?}"),
        }
    }

    #[test]
    fn collection_assignment_lowers_to_handler_exprs() {
        let comp = lower_src(
            r#"component C inherits W {
                state xs: i32[] = [1]
                Button { clicked => { xs = xs.append(2); xs = xs.drop-last(); xs = [3]; } }
            }"#,
        );
        let h = &comp.root.handlers[0];
        let HandlerExpr::Block(exprs) = &h.expr else {
            panic!("expected block");
        };
        assert!(matches!(
            &exprs[0],
            HandlerExpr::Assign {
                lhs,
                rhs,
            } if lhs == "xs" && matches!(rhs.as_ref(), HandlerExpr::ListAppend { path, elem: IrType::I32, value } if path == "xs" && matches!(value.as_ref(), HandlerExpr::IntLit(2)))
        ));
        assert!(matches!(
            &exprs[1],
            HandlerExpr::Assign {
                lhs,
                rhs,
            } if lhs == "xs" && matches!(rhs.as_ref(), HandlerExpr::ListDropLast { path, elem: IrType::I32 } if path == "xs")
        ));
        assert!(matches!(
            &exprs[2],
            HandlerExpr::Assign {
                lhs,
                rhs,
            } if lhs == "xs" && matches!(rhs.as_ref(), HandlerExpr::ListLit(items) if items == &vec![IrLiteral::Int(3)])
        ));
    }
}
