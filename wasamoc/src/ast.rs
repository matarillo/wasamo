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
    StringLit {
        parts: Vec<StringPart>,
        span: Span,
    },
    IntLit {
        value: i64,
        span: Span,
    },
    FloatLit {
        value: f64,
        span: Span,
    },
    BoolLit {
        value: bool,
        span: Span,
    },
    Measurement {
        value: f64,
        unit: Unit,
        span: Span,
    },
    Ident {
        name: String,
        span: Span,
    },
    /// Ratio literal `<num>:<den>` (DD-M3-P2-002). Per dsl_spec §4.9
    /// this expression is only accepted as the RHS of `Box.aspect`;
    /// rejection in other positions is `wasamoc check`'s responsibility.
    RatioLit {
        num: i32,
        den: i32,
        span: Span,
    },
    /// Color literal `#RRGGBB` / `#RRGGBBAA` (DD-M3-P2-003), packed to
    /// `0xAARRGGBB` (alpha in MSB). Position validity (Box.fill only)
    /// is enforced at `wasamoc check`.
    ColorLit {
        value: u32,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::StringLit { span, .. }
            | Expr::IntLit { span, .. }
            | Expr::FloatLit { span, .. }
            | Expr::BoolLit { span, .. }
            | Expr::Measurement { span, .. }
            | Expr::Ident { span, .. }
            | Expr::RatioLit { span, .. }
            | Expr::ColorLit { span, .. } => span,
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
    StateMember {
        name: String,
        ty: TypeName,
        default: Expr,
        span: Span,
    },
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
