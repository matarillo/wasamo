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
    UnsupportedOperator {
        op: String,
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
            | Expr::ColorLit { span, .. }
            | Expr::UnsupportedOperator { span, .. } => span,
        }
    }
}

/// Grid track-list axis (DD-M3-P5-001 / DD-M3-P5-002). Distinguishes the
/// `columns:` and `rows:` track lists parsed by the narrow Grid-specific
/// parser path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackAxis {
    Columns,
    Rows,
}

impl TrackAxis {
    pub fn attr_name(&self) -> &'static str {
        match self {
            TrackAxis::Columns => "columns",
            TrackAxis::Rows => "rows",
        }
    }
}

/// A single track-size token captured by the Grid-specific track-list
/// parser (DD-M3-P5-002, R-A spike Decision 4). The parser is permissive:
/// it records the parsed *shape* (and span) for every token in track
/// position, including the reserved-future `auto` word and out-of-range /
/// invalid forms, so the **check layer** can emit precise diagnostics
/// (range, reserved-future `auto`, float-not-valid, unknown token). Raw
/// integer values are carried as `i64` (the `IntLit` width); value-range
/// validation (`Fixed >= 1`, star weight in `[1, 1024]`) is the check
/// layer's job, not the parser's.
#[derive(Debug, Clone)]
pub enum TrackSize {
    /// Bare integer in track position (fixed pixels). `value` may be
    /// out of range / non-positive; check rejects.
    Fixed { value: i64, span: Span },
    /// `n*` (weighted star) or bare `*` (unit star → `weight = 1`).
    /// `weight` may be out of range; check rejects.
    Star { weight: i64, span: Span },
    /// A float in track position (`1.5`, `1.5*`) — never valid in
    /// Phase 5; check rejects with a float-not-valid diagnostic.
    InvalidFloat { span: Span },
    /// A bare word in track position. `auto` is the reserved-future
    /// token (DD-M3-P5-002); any other word is an unknown track token.
    Word { name: String, span: Span },
}

impl TrackSize {
    pub fn span(&self) -> &Span {
        match self {
            TrackSize::Fixed { span, .. }
            | TrackSize::Star { span, .. }
            | TrackSize::InvalidFloat { span }
            | TrackSize::Word { span, .. } => span,
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
    /// A Grid `columns:` / `rows:` track-list member (DD-M3-P5-002).
    /// Produced only by the narrow Grid-specific parser path (routed on
    /// `widget_type == "Grid"` in `parse_widget_decl`), so it never
    /// appears outside a `Grid` widget body. Lowers to
    /// `KindPayload::Grid` (carrier c1), never to an `IrProp`.
    GridTracks {
        axis: TrackAxis,
        tracks: Vec<TrackSize>,
        span: Span,
    },
    Conditional {
        condition: Expr,
        body: Vec<Member>,
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
