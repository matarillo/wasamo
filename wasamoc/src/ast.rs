#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone)]
pub enum TypeName {
    Int,
    Str,
    Float,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unit {
    Px,
}

#[derive(Debug, Clone)]
pub struct QualifiedName {
    pub segments: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Text(String),
    Interp(QualifiedName),
}

#[derive(Debug, Clone)]
pub enum Expr {
    StringLit { parts: Vec<StringPart>, span: Span },
    IntLit { value: i64, span: Span },
    FloatLit { value: f64, span: Span },
    Measurement { value: f64, unit: Unit, span: Span },
    Ident { name: String, span: Span },
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::StringLit { span, .. }
            | Expr::IntLit { span, .. }
            | Expr::FloatLit { span, .. }
            | Expr::Measurement { span, .. }
            | Expr::Ident { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Eq,
    PlusEq,
    MinusEq,
    MulEq,
    DivEq,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub target: QualifiedName,
    pub op: AssignOp,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Member {
    PropertyDecl {
        name: String,
        ty: TypeName,
        default: Expr,
        span: Span,
    },
    PropertyBind {
        name: String,
        value: Expr,
        span: Span,
    },
    WidgetDecl {
        type_name: String,
        members: Vec<Member>,
        span: Span,
    },
    SignalHandler {
        signal: String,
        body: Block,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct ComponentDef {
    pub name: String,
    pub base: String,
    pub members: Vec<Member>,
    pub span: Span,
}
