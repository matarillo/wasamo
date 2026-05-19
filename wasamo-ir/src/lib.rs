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

/// A widget node in the IR tree.
#[derive(Debug, Clone, PartialEq)]
pub struct IrNode {
    pub widget_type: String,
    pub props: Vec<IrProp>,
    pub bindings: Vec<IrBinding>,
    pub handlers: Vec<IrHandler>,
    pub children: Vec<IrNode>,
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
