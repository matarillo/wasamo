//! Wasamo IR types — the in-memory representation shared between the compiler
//! (`wasamoc`) and the runtime loader (`wasamo-runtime`).
//!
//! Grammar spec: DD-M2-P6-002 / DD-M2-P6-003.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    I32,
    Str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrLiteral {
    Int(i32),
    Str(String),
    Ident(String),
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
    PropRead { path: String },
    Assign { lhs: String, rhs: Box<HandlerExpr> },
    CompoundAssign { lhs: String, op: CompoundOp, rhs: Box<HandlerExpr> },
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
