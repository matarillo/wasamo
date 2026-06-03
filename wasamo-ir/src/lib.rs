//! Wasamo IR types — the in-memory representation shared between the compiler
//! (`wasamoc`) and the runtime loader (`wasamo-runtime`).
//!
//! Grammar spec: DD-M2-P6-002 / DD-M2-P6-003.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    I32,
    Str,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrLiteral {
    Int(i32),
    Str(String),
    Ident(String),
    Bool(bool),
    /// Ratio literal — surface form `<num>:<den>`, both sides positive
    /// integer literals (DD-M3-P2-002). Phase 2 admits this literal only
    /// as the RHS of `Box.aspect`; value-validity (`num > 0`, `den > 0`)
    /// is enforced at `wasamoc check`, not at the variant.
    Ratio {
        num: i32,
        den: i32,
    },
    /// Color literal — surface forms `#RRGGBB` / `#RRGGBBAA`, packed in
    /// `0xAARRGGBB` layout with alpha in the most-significant byte
    /// (DD-M3-P2-003; dsl_spec §8.2 `COLOR` token). Phase 2 admits this
    /// literal only as the RHS of `Box.fill`.
    Color(u32),
}

/// HandlerExpr — the tagged-value expression form (DD-M2-P6-003 = Option A).
/// Maps 1-to-1 to the IR text grammar §8.9.
///
/// This is the single source of truth shared between `wasamoc` (lowering /
/// emit) and `wasamo-runtime` (loader / evaluator). Field / variant naming
/// follows the runtime evaluator's prior conventions: `PropRead { path }`
/// (the field carries a dot-separated path like `root.count`); `CompoundOp`
/// variants are operator names without an `Eq` suffix because the assignment
/// is implicit in the enclosing `CompoundAssign`.
#[derive(Debug, Clone, PartialEq)]
pub enum HandlerExpr {
    IntLit(i32),
    StrLit(String),
    BoolLit(bool),
    PropRead {
        path: String,
    },
    StrPropRead {
        path: String,
    },
    BoolPropRead {
        path: String,
    },
    Assign {
        lhs: String,
        rhs: Box<HandlerExpr>,
    },
    CompoundAssign {
        lhs: String,
        op: CompoundOp,
        rhs: Box<HandlerExpr>,
    },
    Interpolation(Vec<InterpolationPart>),
    Block(Vec<HandlerExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationPart {
    Literal(String),
    Expr(HandlerExpr),
}

/// A `state` node in the IR component.
#[derive(Debug, Clone, PartialEq)]
pub struct IrState {
    pub name: String,
    pub ty: IrType,
    pub default: IrLiteral,
}

/// A static property set (`prop name = value`).
#[derive(Debug, Clone, PartialEq)]
pub struct IrProp {
    pub name: String,
    pub value: IrLiteral,
}

/// Grid track sizing form (DD-M3-P5-002). Phase 5 admits fixed integer
/// pixels and weighted-star tracks only; `auto` / `minmax` / named lines
/// are deferred — this enum is their additive extension point (adding a
/// future `Auto` variant lowers existing star/fixed track lists
/// unchanged). Value validity (`Fixed` `>= 1`; `Star` weight in
/// `[1, 1024]`) is enforced at `wasamoc check` and runtime `validate()`,
/// not at the variant. Unit star `*` lowers to `Star(1)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackSize {
    Fixed(i32),
    Star(u32),
}

/// Grid kind-specific payload carried on a still-generic `IrNode`
/// (DD-M3-P5-001 carrier decision **c1**). Grid's `columns:` / `rows:`
/// track lists live here rather than in `IrProp`, so `IrProp.value`
/// stays strictly `IrLiteral` for every kind (the M2 / Phase 1..4
/// invariant). `IrNode.kind_payload` is `None` for every non-Grid kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindPayload {
    Grid {
        columns: Vec<TrackSize>,
        rows: Vec<TrackSize>,
    },
}

/// A reactive binding (`bind name = expr`).
#[derive(Debug, Clone, PartialEq)]
pub struct IrBinding {
    pub prop_name: String,
    pub expr: HandlerExpr,
}

/// A signal handler (`on signal { expr }`).
#[derive(Debug, Clone, PartialEq)]
pub struct IrHandler {
    pub signal: String,
    pub expr: HandlerExpr,
}

/// A member in a widget node body.
#[derive(Debug, Clone, PartialEq)]
pub enum IrMember {
    Widget(IrNode),
    ControlFlow(ControlFlowNode),
}

/// Structural control-flow member. Phase 6 ships only the single-branch
/// `If` form; the branch list is the family extension point for `else`.
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlowNode {
    If { branches: Vec<ControlFlowBranch> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlowBranch {
    pub condition: HandlerExpr,
    pub body: Vec<IrMember>,
}

/// A widget node in the IR tree.
#[derive(Debug, Clone, PartialEq)]
pub struct IrNode {
    pub widget_type: String,
    pub props: Vec<IrProp>,
    pub bindings: Vec<IrBinding>,
    pub handlers: Vec<IrHandler>,
    pub children: Vec<IrMember>,
    /// Grid kind-specific payload (DD-M3-P5-001 carrier c1); `None` for
    /// every non-Grid widget kind. Set explicitly at each construction
    /// site (the IR types deliberately derive no `Default`, so adding
    /// this field surfaces every site at compile time — R-C
    /// construction-site discipline).
    pub kind_payload: Option<KindPayload>,
}

impl IrNode {
    pub fn widget_children(&self) -> impl Iterator<Item = &IrNode> {
        self.children.iter().filter_map(|member| match member {
            IrMember::Widget(node) => Some(node),
            IrMember::ControlFlow(_) => None,
        })
    }
}

/// Top-level IR component.
#[derive(Debug, Clone, PartialEq)]
pub struct IrComponent {
    pub name: String,
    pub base: String,
    pub states: Vec<IrState>,
    pub root: IrNode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ir_type_bool_distinct_from_i32_and_str() {
        assert_ne!(IrType::Bool, IrType::I32);
        assert_ne!(IrType::Bool, IrType::Str);
        assert_eq!(IrType::Bool, IrType::Bool);
    }

    #[test]
    fn ir_literal_bool_round_trip_values() {
        let t = IrLiteral::Bool(true);
        let f = IrLiteral::Bool(false);
        assert_ne!(t, f);
        assert_eq!(t.clone(), IrLiteral::Bool(true));
        assert_eq!(f.clone(), IrLiteral::Bool(false));
    }

    #[test]
    fn ir_literal_bool_distinct_from_int_zero_one() {
        assert_ne!(IrLiteral::Bool(false), IrLiteral::Int(0));
        assert_ne!(IrLiteral::Bool(true), IrLiteral::Int(1));
    }

    #[test]
    fn handler_expr_bool_lit_round_trip() {
        let t = HandlerExpr::BoolLit(true);
        let f = HandlerExpr::BoolLit(false);
        assert_ne!(t, f);
        assert_eq!(t.clone(), HandlerExpr::BoolLit(true));
    }

    #[test]
    fn handler_expr_bool_prop_read_carries_path() {
        let e = HandlerExpr::BoolPropRead {
            path: "root.ready".into(),
        };
        match &e {
            HandlerExpr::BoolPropRead { path } => assert_eq!(path, "root.ready"),
            _ => panic!("expected BoolPropRead"),
        }
    }

    #[test]
    fn handler_expr_bool_variants_distinct_from_other_scalars() {
        assert_ne!(HandlerExpr::BoolLit(true), HandlerExpr::IntLit(1));
        assert_ne!(
            HandlerExpr::BoolLit(false),
            HandlerExpr::StrLit("false".into())
        );
        assert_ne!(
            HandlerExpr::BoolPropRead {
                path: "ready".into()
            },
            HandlerExpr::PropRead {
                path: "ready".into()
            }
        );
        assert_ne!(
            HandlerExpr::BoolPropRead {
                path: "ready".into()
            },
            HandlerExpr::StrPropRead {
                path: "ready".into()
            }
        );
    }

    #[test]
    fn ir_literal_ratio_round_trip_values() {
        let r = IrLiteral::Ratio { num: 16, den: 9 };
        match &r {
            IrLiteral::Ratio { num, den } => {
                assert_eq!(*num, 16);
                assert_eq!(*den, 9);
            }
            _ => panic!("expected Ratio"),
        }
        assert_eq!(r.clone(), IrLiteral::Ratio { num: 16, den: 9 });
    }

    #[test]
    fn ir_literal_ratio_distinct_by_components() {
        assert_ne!(
            IrLiteral::Ratio { num: 16, den: 9 },
            IrLiteral::Ratio { num: 9, den: 16 }
        );
        assert_ne!(
            IrLiteral::Ratio { num: 16, den: 9 },
            IrLiteral::Ratio { num: 32, den: 18 }
        );
    }

    #[test]
    fn ir_literal_color_round_trip_value() {
        let c = IrLiteral::Color(0x80_00_00_00);
        match &c {
            IrLiteral::Color(v) => assert_eq!(*v, 0x80_00_00_00),
            _ => panic!("expected Color"),
        }
        assert_eq!(c.clone(), IrLiteral::Color(0x80_00_00_00));
    }

    #[test]
    fn ir_literal_color_distinct_by_packed_value() {
        // #cccccc with implicit alpha 0xFF → 0xFFCCCCCC
        let opaque = IrLiteral::Color(0xFF_CC_CC_CC);
        // #cccccc00 (fully transparent same RGB) → 0x00CCCCCC
        let transparent = IrLiteral::Color(0x00_CC_CC_CC);
        assert_ne!(opaque, transparent);
    }

    #[test]
    fn ir_literal_ratio_and_color_distinct_from_other_variants() {
        assert_ne!(IrLiteral::Ratio { num: 1, den: 1 }, IrLiteral::Int(1));
        assert_ne!(IrLiteral::Color(0), IrLiteral::Int(0));
        assert_ne!(
            IrLiteral::Color(0xFFFFFFFF),
            IrLiteral::Ident("white".into())
        );
    }

    #[test]
    fn track_size_variants_distinct() {
        assert_ne!(TrackSize::Fixed(1), TrackSize::Star(1));
        assert_ne!(TrackSize::Fixed(180), TrackSize::Fixed(181));
        assert_ne!(TrackSize::Star(1), TrackSize::Star(2));
        assert_eq!(TrackSize::Star(3), TrackSize::Star(3));
    }

    #[test]
    fn kind_payload_grid_round_trip() {
        let p = KindPayload::Grid {
            columns: vec![
                TrackSize::Fixed(180),
                TrackSize::Star(1),
                TrackSize::Star(2),
            ],
            rows: vec![TrackSize::Star(1), TrackSize::Star(1)],
        };
        match &p {
            KindPayload::Grid { columns, rows } => {
                assert_eq!(
                    columns,
                    &vec![
                        TrackSize::Fixed(180),
                        TrackSize::Star(1),
                        TrackSize::Star(2)
                    ]
                );
                assert_eq!(rows, &vec![TrackSize::Star(1), TrackSize::Star(1)]);
            }
        }
        assert_eq!(p.clone(), p);
    }

    #[test]
    fn ir_node_kind_payload_defaults_none_for_generic_kinds() {
        let n = IrNode {
            widget_type: "VStack".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: None,
        };
        assert_eq!(n.kind_payload, None);
    }

    #[test]
    fn ir_member_encodes_widget_and_control_flow() {
        let text = IrNode {
            widget_type: "Text".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: None,
        };
        let control = ControlFlowNode::If {
            branches: vec![ControlFlowBranch {
                condition: HandlerExpr::BoolLit(true),
                body: vec![IrMember::Widget(text.clone())],
            }],
        };

        assert!(matches!(IrMember::Widget(text), IrMember::Widget(_)));
        assert!(matches!(
            IrMember::ControlFlow(control),
            IrMember::ControlFlow(ControlFlowNode::If { .. })
        ));
    }

    #[test]
    fn ir_node_carries_grid_kind_payload() {
        let n = IrNode {
            widget_type: "Grid".into(),
            props: vec![],
            bindings: vec![],
            handlers: vec![],
            children: vec![],
            kind_payload: Some(KindPayload::Grid {
                columns: vec![TrackSize::Fixed(180), TrackSize::Star(1)],
                rows: vec![TrackSize::Star(1)],
            }),
        };
        assert!(matches!(n.kind_payload, Some(KindPayload::Grid { .. })));
    }

    #[test]
    fn ir_state_bool_declaration() {
        let s = IrState {
            name: "ready".into(),
            ty: IrType::Bool,
            default: IrLiteral::Bool(false),
        };
        assert_eq!(s.ty, IrType::Bool);
        assert_eq!(s.default, IrLiteral::Bool(false));
    }
}
