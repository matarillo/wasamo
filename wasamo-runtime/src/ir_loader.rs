//! IR loader (DD-M2-P6-006) — parses the normative Wasamo IR text grammar
//! (DD-M2-P6-002) and constructs the runtime widget tree.
//!
//! The module is split in two: a pure-logic parser (`parse_ir`, testable
//! without any Win32/WinRT dependency) and a builder (`build_widget_tree`,
//! requires a live `Compositor` and `TextRenderer`). The C ABI front-end
//! (`wasamo_load_ui`) is wired in DD-M2-P6-005 — this module exposes the
//! Rust-level entry points only.

use std::rc::Rc;

use wasamo_ir::{
    CompoundOp, HandlerExpr, InterpolationPart, IrBinding, IrComponent, IrHandler, IrLiteral,
    IrNode, IrProp, IrState, IrType,
};

use crate::layout::Alignment;
use crate::reactive::{
    register_binding, register_bool_binding, set_active_registry, BindingTarget, PropertyKey,
    Signal, SignalRegistry, WidgetId,
};
use crate::text::{TextRenderer, TypographyStyle};
use crate::widget::{
    widget_write_property, widget_write_property_bool, ButtonStyle, WidgetNode,
    PROP_BUTTON_ENABLED, PROP_BUTTON_LABEL, PROP_BUTTON_STYLE, PROP_TEXT_CONTENT, PROP_TEXT_STYLE,
};

use windows::UI::Composition::Compositor;

/// Errors surfaced by the IR loader.
///
/// `InvalidHeader`, `Parse`, `UnknownWidget`, and `Validate` are all
/// "malformed IR" failures — DD-M2-P6-005 maps them to
/// `WASAMO_ERR_IR_MALFORMED` at the C ABI boundary, with the `Display`
/// rendering surfaced through `wasamo_last_error_message`. `Build`
/// failures originate from the Win32/WinRT side and are not part of the
/// defense-in-depth surface targeted by DD-M2-P6-009.
#[derive(Debug, Clone, PartialEq)]
pub enum IrLoadError {
    InvalidHeader(String),
    Parse(String),
    UnknownWidget(String),
    Validate(String),
    Build(String),
}

impl IrLoadError {
    /// Whether this variant represents a "malformed IR" failure
    /// (DD-M2-P6-009). Useful for the C ABI translation in DD-M2-P6-005.
    pub fn is_malformed(&self) -> bool {
        matches!(
            self,
            IrLoadError::InvalidHeader(_)
                | IrLoadError::Parse(_)
                | IrLoadError::UnknownWidget(_)
                | IrLoadError::Validate(_)
        )
    }
}

impl std::fmt::Display for IrLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrLoadError::InvalidHeader(msg) => write!(f, "invalid IR header: {msg}"),
            IrLoadError::Parse(msg) => write!(f, "IR parse error: {msg}"),
            IrLoadError::UnknownWidget(name) => write!(f, "unknown widget type: {name}"),
            IrLoadError::Validate(msg) => write!(f, "IR validation error: {msg}"),
            IrLoadError::Build(msg) => write!(f, "IR build error: {msg}"),
        }
    }
}

const HEADER_MAGIC: &str = ";wasamo-ir v0";

/// A widget tree built from an IR component, paired with the SignalRegistry
/// for the component's `state` declarations. The registry is also installed
/// as the runtime's active registry (see `reactive::set_active_registry`) so
/// click-handler dispatch can reach it; the field here keeps an extra Rc
/// around so the registry's lifetime is tied to the BuiltUi (and therefore
/// to whatever owns the widget tree), independent of the thread-local.
pub struct BuiltUi {
    pub root: Box<WidgetNode>,
    #[allow(dead_code)]
    pub(crate) registry: Rc<SignalRegistry>,
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Parse the normative IR text (DD-M2-P6-002) into an `IrComponent`. Pure
/// logic — testable without any Win32/WinRT dependency.
///
/// On success the returned `IrComponent` has passed defense-in-depth
/// validation (DD-M2-P6-009): header magic + version, top-level document
/// structure (enforced by the parser), unique `state` names, and
/// resolution of every name referenced by a binding/handler expression
/// to a declared `state`. Per-node value-type integrity is trusted from
/// the emitter (`wasamoc`) and is **not** re-validated here.
pub fn parse_ir(text: &str) -> Result<IrComponent, IrLoadError> {
    let body = check_and_strip_header(text)?;
    let tokens = tokenize(body)?;
    let mut p = Parser {
        tokens: &tokens,
        pos: 0,
    };
    let comp = p.parse_component()?;
    if p.pos < p.tokens.len() {
        return Err(IrLoadError::Parse(format!(
            "unexpected trailing tokens after component (token #{})",
            p.pos
        )));
    }
    validate(&comp)?;
    Ok(comp)
}

/// Defense-in-depth validation pass (DD-M2-P6-009).
///
/// Currently checks:
/// 1. `state` declarations have unique names (flat namespace per
///    DD-M2-P6-004's name-resolution rules).
/// 2. Every name referenced by a binding/handler expression resolves to
///    a declared `state`. Widget-instance references are not yet part of
///    the IR — see `docs/notes/dsl-grammar.md` Q1 for the open question.
fn validate(comp: &IrComponent) -> Result<(), IrLoadError> {
    let mut declared = std::collections::HashSet::new();
    for state in &comp.states {
        if !declared.insert(state.name.as_str()) {
            return Err(IrLoadError::Validate(format!(
                "duplicate `state` name: `{}`",
                state.name
            )));
        }
    }
    validate_node_references(&comp.root, &declared)
}

fn validate_node_references(
    node: &IrNode,
    declared: &std::collections::HashSet<&str>,
) -> Result<(), IrLoadError> {
    for binding in &node.bindings {
        validate_expr_references(&binding.expr, declared, &|name| {
            format!(
                "binding `{}` references undeclared name `{}`",
                binding.prop_name, name
            )
        })?;
    }
    for handler in &node.handlers {
        validate_expr_references(&handler.expr, declared, &|name| {
            format!(
                "handler `on {}` references undeclared name `{}`",
                handler.signal, name
            )
        })?;
    }
    for child in &node.children {
        validate_node_references(child, declared)?;
    }
    Ok(())
}

fn validate_expr_references(
    expr: &HandlerExpr,
    declared: &std::collections::HashSet<&str>,
    err_msg: &dyn Fn(&str) -> String,
) -> Result<(), IrLoadError> {
    match expr {
        HandlerExpr::IntLit(_) | HandlerExpr::StrLit(_) | HandlerExpr::BoolLit(_) => Ok(()),
        HandlerExpr::PropRead { path }
        | HandlerExpr::StrPropRead { path }
        | HandlerExpr::BoolPropRead { path } => {
            if !declared.contains(path.as_str()) {
                Err(IrLoadError::Validate(err_msg(path)))
            } else {
                Ok(())
            }
        }
        HandlerExpr::Assign { lhs, rhs } => {
            if !declared.contains(lhs.as_str()) {
                return Err(IrLoadError::Validate(err_msg(lhs)));
            }
            validate_expr_references(rhs, declared, err_msg)
        }
        HandlerExpr::CompoundAssign { lhs, rhs, .. } => {
            if !declared.contains(lhs.as_str()) {
                return Err(IrLoadError::Validate(err_msg(lhs)));
            }
            validate_expr_references(rhs, declared, err_msg)
        }
        HandlerExpr::Interpolation(parts) => {
            for part in parts {
                if let InterpolationPart::Expr(inner) = part {
                    validate_expr_references(inner, declared, err_msg)?;
                }
            }
            Ok(())
        }
        HandlerExpr::Block(exprs) => {
            for inner in exprs {
                validate_expr_references(inner, declared, err_msg)?;
            }
            Ok(())
        }
    }
}

fn check_and_strip_header(text: &str) -> Result<&str, IrLoadError> {
    let trimmed = text.trim_start();
    let line_end = trimmed.find('\n').unwrap_or(trimmed.len());
    let line = trimmed[..line_end].trim_end();
    if line != HEADER_MAGIC {
        return Err(IrLoadError::InvalidHeader(format!(
            "expected `{HEADER_MAGIC}`, got `{line}`"
        )));
    }
    if line_end < trimmed.len() {
        Ok(&trimmed[line_end + 1..])
    } else {
        Ok("")
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LBrace,
    RBrace,
    LParen,
    RParen,
    Eq,
    Colon,
    Ident(String),
    Int(i32),
    Str(String),
    AssignOp(CompoundOp),
}

fn tokenize(text: &str) -> Result<Vec<Token>, IrLoadError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == ';' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        match c {
            '{' => {
                tokens.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '=' => {
                tokens.push(Token::Eq);
                i += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
                i += 1;
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        let esc = chars[i + 1];
                        let ch = match esc {
                            '"' => '"',
                            '\\' => '\\',
                            'n' => '\n',
                            't' => '\t',
                            other => {
                                return Err(IrLoadError::Parse(format!(
                                    "unknown escape: \\{other}"
                                )));
                            }
                        };
                        s.push(ch);
                        i += 2;
                    } else {
                        s.push(chars[i]);
                        i += 1;
                    }
                }
                if i >= chars.len() {
                    return Err(IrLoadError::Parse("unterminated string literal".into()));
                }
                i += 1;
                tokens.push(Token::Str(s));
            }
            '+' | '*' | '/' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    let op = match c {
                        '+' => CompoundOp::Add,
                        '*' => CompoundOp::Mul,
                        '/' => CompoundOp::Div,
                        _ => unreachable!(),
                    };
                    tokens.push(Token::AssignOp(op));
                    i += 2;
                } else {
                    return Err(IrLoadError::Parse(format!("unexpected character: '{c}'")));
                }
            }
            '-' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::AssignOp(CompoundOp::Sub));
                    i += 2;
                } else if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                    i += 1;
                    let start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let s: String = chars[start..i].iter().collect();
                    let n: i32 = s
                        .parse()
                        .map_err(|_| IrLoadError::Parse(format!("invalid integer: -{s}")))?;
                    tokens.push(Token::Int(-n));
                } else {
                    return Err(IrLoadError::Parse("unexpected character: '-'".into()));
                }
            }
            d if d.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                let n: i32 = s
                    .parse()
                    .map_err(|_| IrLoadError::Parse(format!("invalid integer: {s}")))?;
                tokens.push(Token::Int(n));
            }
            l if l.is_ascii_alphabetic() || l == '_' => {
                let start = i;
                while i < chars.len() {
                    let cc = chars[i];
                    if cc.is_ascii_alphanumeric() || cc == '_' || cc == '-' {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(s));
            }
            other => {
                return Err(IrLoadError::Parse(format!(
                    "unexpected character: '{other}'"
                )));
            }
        }
    }
    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), IrLoadError> {
        let actual = self
            .advance()
            .ok_or_else(|| IrLoadError::Parse(format!("expected {expected:?}, got EOF")))?;
        if actual != expected {
            return Err(IrLoadError::Parse(format!(
                "expected {expected:?}, got {actual:?}"
            )));
        }
        Ok(())
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), IrLoadError> {
        match self.advance() {
            Some(Token::Ident(s)) if s == kw => Ok(()),
            other => Err(IrLoadError::Parse(format!(
                "expected keyword `{kw}`, got {other:?}"
            ))),
        }
    }

    fn expect_ident(&mut self) -> Result<String, IrLoadError> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            other => Err(IrLoadError::Parse(format!(
                "expected identifier, got {other:?}"
            ))),
        }
    }

    fn parse_component(&mut self) -> Result<IrComponent, IrLoadError> {
        self.expect_keyword("component")?;
        let name = self.expect_ident()?;
        self.expect_keyword("inherits")?;
        let base = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut states = Vec::new();
        let mut root: Option<IrNode> = None;

        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::Ident(s)) if s == "state" => {
                    states.push(self.parse_state()?);
                }
                Some(Token::Ident(s)) if s == "node" => {
                    if root.is_some() {
                        return Err(IrLoadError::Parse(
                            "multiple root nodes in component".into(),
                        ));
                    }
                    root = Some(self.parse_node()?);
                }
                Some(other) => {
                    return Err(IrLoadError::Parse(format!(
                        "unexpected token in component body: {other:?}"
                    )));
                }
                None => {
                    return Err(IrLoadError::Parse(
                        "unexpected EOF in component body".into(),
                    ));
                }
            }
        }

        let root = root.ok_or_else(|| IrLoadError::Parse("component has no root node".into()))?;
        Ok(IrComponent {
            name,
            base,
            states,
            root,
        })
    }

    fn parse_state(&mut self) -> Result<IrState, IrLoadError> {
        self.expect_keyword("state")?;
        let name = self.expect_ident()?;
        self.expect(&Token::Colon)?;
        let ty_str = self.expect_ident()?;
        let ty = match ty_str.as_str() {
            "i32" => IrType::I32,
            "string" => IrType::Str,
            "bool" => IrType::Bool,
            other => {
                return Err(IrLoadError::Parse(format!("unknown state type: {other}")));
            }
        };
        self.expect(&Token::Eq)?;
        let default = self.parse_literal()?;
        Ok(IrState { name, ty, default })
    }

    fn parse_node(&mut self) -> Result<IrNode, IrLoadError> {
        self.expect_keyword("node")?;
        let widget_type = self.expect_ident()?;
        self.expect(&Token::LBrace)?;

        let mut props = Vec::new();
        let mut bindings = Vec::new();
        let mut handlers = Vec::new();
        let mut children = Vec::new();

        loop {
            match self.peek() {
                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::Ident(s)) if s == "prop" => props.push(self.parse_prop()?),
                Some(Token::Ident(s)) if s == "bind" => bindings.push(self.parse_binding()?),
                Some(Token::Ident(s)) if s == "on" => handlers.push(self.parse_handler()?),
                Some(Token::Ident(s)) if s == "node" => children.push(self.parse_node()?),
                Some(other) => {
                    return Err(IrLoadError::Parse(format!(
                        "unexpected token in node body: {other:?}"
                    )));
                }
                None => {
                    return Err(IrLoadError::Parse("unexpected EOF in node body".into()));
                }
            }
        }

        Ok(IrNode {
            widget_type,
            props,
            bindings,
            handlers,
            children,
        })
    }

    fn parse_prop(&mut self) -> Result<IrProp, IrLoadError> {
        self.expect_keyword("prop")?;
        let name = self.expect_ident()?;
        self.expect(&Token::Eq)?;
        let value = self.parse_literal()?;
        Ok(IrProp { name, value })
    }

    fn parse_binding(&mut self) -> Result<IrBinding, IrLoadError> {
        self.expect_keyword("bind")?;
        let prop_name = self.expect_ident()?;
        self.expect(&Token::Eq)?;
        let expr = self.parse_expr()?;
        Ok(IrBinding { prop_name, expr })
    }

    fn parse_handler(&mut self) -> Result<IrHandler, IrLoadError> {
        self.expect_keyword("on")?;
        let signal = self.expect_ident()?;
        self.expect(&Token::LBrace)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::RBrace)?;
        Ok(IrHandler { signal, expr })
    }

    fn parse_literal(&mut self) -> Result<IrLiteral, IrLoadError> {
        match self.advance() {
            Some(Token::Int(n)) => Ok(IrLiteral::Int(*n)),
            Some(Token::Str(s)) => Ok(IrLiteral::Str(s.clone())),
            Some(Token::Ident(s)) if s == "true" => Ok(IrLiteral::Bool(true)),
            Some(Token::Ident(s)) if s == "false" => Ok(IrLiteral::Bool(false)),
            Some(Token::Ident(s)) => Ok(IrLiteral::Ident(s.clone())),
            other => Err(IrLoadError::Parse(format!(
                "expected literal, got {other:?}"
            ))),
        }
    }

    fn parse_expr(&mut self) -> Result<HandlerExpr, IrLoadError> {
        match self.peek() {
            Some(Token::Int(n)) => {
                let v = *n;
                self.advance();
                Ok(HandlerExpr::IntLit(v))
            }
            Some(Token::Str(s)) => {
                let v = s.clone();
                self.advance();
                Ok(HandlerExpr::StrLit(v))
            }
            Some(Token::Ident(s)) if s == "true" => {
                self.advance();
                Ok(HandlerExpr::BoolLit(true))
            }
            Some(Token::Ident(s)) if s == "false" => {
                self.advance();
                Ok(HandlerExpr::BoolLit(false))
            }
            Some(Token::LParen) => self.parse_sexpr(),
            other => Err(IrLoadError::Parse(format!(
                "expected expression, got {other:?}"
            ))),
        }
    }

    fn parse_sexpr(&mut self) -> Result<HandlerExpr, IrLoadError> {
        self.expect(&Token::LParen)?;
        let tag = self.expect_ident()?;
        let result = match tag.as_str() {
            "prop-read" => {
                let path = self.expect_ident()?;
                HandlerExpr::PropRead { path }
            }
            "str-prop-read" => {
                let path = self.expect_ident()?;
                HandlerExpr::StrPropRead { path }
            }
            "bool-prop-read" => {
                let path = self.expect_ident()?;
                HandlerExpr::BoolPropRead { path }
            }
            "assign" => {
                let lhs = self.expect_ident()?;
                let rhs = self.parse_expr()?;
                HandlerExpr::Assign {
                    lhs,
                    rhs: Box::new(rhs),
                }
            }
            "compound-assign" => {
                let op = match self.advance() {
                    Some(Token::AssignOp(op)) => *op,
                    other => {
                        return Err(IrLoadError::Parse(format!(
                            "expected compound op, got {other:?}"
                        )));
                    }
                };
                let lhs = self.expect_ident()?;
                let rhs = self.parse_expr()?;
                HandlerExpr::CompoundAssign {
                    op,
                    lhs,
                    rhs: Box::new(rhs),
                }
            }
            "interp" => {
                let mut parts = Vec::new();
                while !matches!(self.peek(), Some(Token::RParen)) {
                    parts.push(self.parse_interp_part()?);
                }
                HandlerExpr::Interpolation(parts)
            }
            "block" => {
                let mut exprs = Vec::new();
                while !matches!(self.peek(), Some(Token::RParen)) {
                    exprs.push(self.parse_expr()?);
                }
                HandlerExpr::Block(exprs)
            }
            other => {
                return Err(IrLoadError::Parse(format!("unknown S-expr tag: {other}")));
            }
        };
        self.expect(&Token::RParen)?;
        Ok(result)
    }

    fn parse_interp_part(&mut self) -> Result<InterpolationPart, IrLoadError> {
        match self.peek() {
            Some(Token::Str(s)) => {
                let s = s.clone();
                self.advance();
                Ok(InterpolationPart::Literal(s))
            }
            Some(Token::LParen) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(InterpolationPart::Expr(inner))
            }
            other => Err(IrLoadError::Parse(format!(
                "expected interp part, got {other:?}"
            ))),
        }
    }
}

// ── Builder (Win32/WinRT) ─────────────────────────────────────────────────────

/// Build a runtime widget tree from a parsed `IrComponent`, plus the
/// `SignalRegistry` populated from `state` declarations.
pub fn build_widget_tree(
    comp: &IrComponent,
    compositor: &Compositor,
    renderer: &TextRenderer,
) -> Result<BuiltUi, IrLoadError> {
    let registry = build_signal_registry(&comp.states);
    let registry = Rc::new(registry);
    // Install before building children so binding closures captured during
    // `build_node` see the registry — and so click-handler dispatch (which
    // happens after `wasamo::run()` enters the message loop) can find it.
    set_active_registry(Rc::clone(&registry));
    let root = build_node(&comp.root, compositor, renderer, &registry)?;
    Ok(BuiltUi { root, registry })
}

fn build_signal_registry(states: &[IrState]) -> SignalRegistry {
    let mut registry = SignalRegistry::new();
    for state in states {
        match state.ty {
            IrType::I32 => {
                let initial = match &state.default {
                    IrLiteral::Int(n) => *n,
                    _ => 0,
                };
                registry
                    .i32s
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrType::Str => {
                let initial = match &state.default {
                    IrLiteral::Str(s) => s.clone(),
                    _ => String::new(),
                };
                registry
                    .strings
                    .insert(state.name.clone(), Signal::new(initial));
            }
            IrType::Bool => {
                let initial = match &state.default {
                    IrLiteral::Bool(b) => *b,
                    _ => false,
                };
                registry
                    .bools
                    .insert(state.name.clone(), Signal::new(initial));
            }
        }
    }
    registry
}

fn build_node(
    node: &IrNode,
    compositor: &Compositor,
    renderer: &TextRenderer,
    registry: &Rc<SignalRegistry>,
) -> Result<Box<WidgetNode>, IrLoadError> {
    let mut widget = construct_widget(node, compositor, renderer)?;

    // Bindings: register each `bind` as a reactive Effect targeting the widget property.
    for binding in &node.bindings {
        let Some((prop_key, prop_ty)) = resolve_prop_key(&node.widget_type, &binding.prop_name)
        else {
            // Unknown property name on this widget type — silently skip in M2.
            // M3 will surface this through the diagnostic system.
            continue;
        };
        let widget_id = WidgetId(widget.as_mut() as *mut WidgetNode as *mut ());
        let target = BindingTarget::WidgetProperty {
            node: widget_id,
            prop: prop_key,
        };
        // Per-type writer dispatch (DD-M3-P1-007 Option A + DD-M3-P1-009):
        // the loader selects the evaluator/writer pair matching the target
        // property's declared `IrType`. The reactive engine itself stays
        // type-agnostic; the seam lives here at the call site.
        let handle = match prop_ty {
            IrType::Bool => register_bool_binding(
                target,
                binding.expr.clone(),
                Rc::clone(registry),
                widget_write_property_bool,
            ),
            // I32 and Str properties continue through the M2 string-baked
            // writer (stringified by `evaluate_binding`, parsed at the
            // per-widget setter — typed-i32 writer lands when its use case
            // arrives).
            IrType::I32 | IrType::Str => register_binding(
                target,
                binding.expr.clone(),
                Rc::clone(registry),
                widget_write_property,
            ),
        };
        widget.bindings.push(handle);
    }

    // Handlers: attach each `on` body via Phase 3's set_inline_handler path.
    for handler in &node.handlers {
        widget.set_inline_handler(handler.signal.clone(), handler.expr.clone());
    }

    // Children: recurse and attach via the Phase 4 internal mutation API.
    for child in &node.children {
        let child_widget = build_node(child, compositor, renderer, registry)?;
        widget
            .append_child(child_widget)
            .map_err(|e| IrLoadError::Build(format!("append_child failed: {e:?}")))?;
    }

    Ok(widget)
}

fn construct_widget(
    node: &IrNode,
    compositor: &Compositor,
    renderer: &TextRenderer,
) -> Result<Box<WidgetNode>, IrLoadError> {
    match node.widget_type.as_str() {
        "VStack" => {
            let spacing = extract_int_prop(&node.props, "spacing").unwrap_or(0) as f32;
            let padding = extract_int_prop(&node.props, "padding").unwrap_or(0) as f32;
            WidgetNode::vstack(compositor, spacing, padding, Alignment::Center)
                .map_err(|e| IrLoadError::Build(format!("vstack: {e}")))
        }
        "HStack" => {
            let spacing = extract_int_prop(&node.props, "spacing").unwrap_or(0) as f32;
            let padding = extract_int_prop(&node.props, "padding").unwrap_or(0) as f32;
            WidgetNode::hstack(compositor, spacing, padding, Alignment::Center)
                .map_err(|e| IrLoadError::Build(format!("hstack: {e}")))
        }
        "Text" => {
            let text = extract_str_prop(&node.props, "text").unwrap_or_default();
            let style = extract_typography(&node.props, "font");
            // If `text` will be filled by a binding, construct with empty content;
            // the binding's initial run writes the actual value during build.
            let initial = if has_binding(&node.bindings, "text") {
                String::new()
            } else {
                text
            };
            WidgetNode::text(compositor, renderer, &initial, style)
                .map_err(|e| IrLoadError::Build(format!("text: {e}")))
        }
        "Button" => {
            let label = extract_str_prop(&node.props, "text").unwrap_or_default();
            let style = extract_button_style(&node.props, "style");
            let initial = if has_binding(&node.bindings, "text") {
                String::new()
            } else {
                label
            };
            WidgetNode::button(compositor, renderer, &initial, style)
                .map_err(|e| IrLoadError::Build(format!("button: {e}")))
        }
        other => Err(IrLoadError::UnknownWidget(other.to_string())),
    }
}

// Widget catalog: `(widget_type, prop_name) → (PROP_* id, declared IrType)`.
// DD-M3-P1-009 widens the return shape so the binding loader can pick the
// per-type writer that matches the target property. The catalog mirrors the
// soft `wasamoc::check` widget-property table (kept independently so the
// compiler stays self-contained — see m3-phase-1-progress.md T3 Notes).
fn resolve_prop_key(widget_type: &str, prop_name: &str) -> Option<(PropertyKey, IrType)> {
    match (widget_type, prop_name) {
        ("Text", "text") => Some((PROP_TEXT_CONTENT, IrType::Str)),
        ("Text", "font") => Some((PROP_TEXT_STYLE, IrType::I32)),
        ("Button", "text") => Some((PROP_BUTTON_LABEL, IrType::Str)),
        ("Button", "style") => Some((PROP_BUTTON_STYLE, IrType::I32)),
        ("Button", "enabled") => Some((PROP_BUTTON_ENABLED, IrType::Bool)),
        _ => None,
    }
}

fn extract_int_prop(props: &[IrProp], name: &str) -> Option<i32> {
    props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Int(n) => Some(*n),
            _ => None,
        })
}

fn extract_str_prop(props: &[IrProp], name: &str) -> Option<String> {
    props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Str(s) => Some(s.clone()),
            _ => None,
        })
}

fn extract_typography(props: &[IrProp], name: &str) -> TypographyStyle {
    let ident = props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Ident(id) => Some(id.as_str()),
            _ => None,
        });
    match ident {
        Some("caption") => TypographyStyle::Caption,
        Some("subtitle") => TypographyStyle::Subtitle,
        Some("title") => TypographyStyle::Title,
        _ => TypographyStyle::Body,
    }
}

fn extract_button_style(props: &[IrProp], name: &str) -> ButtonStyle {
    let ident = props
        .iter()
        .find(|p| p.name == name)
        .and_then(|p| match &p.value {
            IrLiteral::Ident(id) => Some(id.as_str()),
            _ => None,
        });
    match ident {
        Some("accent") => ButtonStyle::Accent,
        _ => ButtonStyle::Default,
    }
}

fn has_binding(bindings: &[IrBinding], name: &str) -> bool {
    bindings.iter().any(|b| b.prop_name == name)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── resolve_prop_key / binding dispatch (M3-Phase 1 T8 / DD-M3-P1-009) ──
    //
    // resolve_prop_key drives the per-type writer dispatch in `build_node`:
    // its returned `IrType` selects between `register_bool_binding`
    // (+ widget_write_property_bool) and the string-baked `register_binding`
    // path. End-to-end exercise lives in the Windows-bound integration test
    // for `Button.enabled` (T6); these tests cover the pure-logic seam.

    #[test]
    fn resolve_prop_key_button_enabled_is_bool() {
        let (key, ty) = resolve_prop_key("Button", "enabled").expect("Button.enabled exists");
        assert_eq!(key, PROP_BUTTON_ENABLED);
        assert_eq!(ty, IrType::Bool);
    }

    #[test]
    fn resolve_prop_key_text_text_is_string() {
        let (key, ty) = resolve_prop_key("Text", "text").expect("Text.text exists");
        assert_eq!(key, PROP_TEXT_CONTENT);
        assert_eq!(ty, IrType::Str);
    }

    #[test]
    fn resolve_prop_key_button_text_is_string() {
        let (key, ty) = resolve_prop_key("Button", "text").expect("Button.text exists");
        assert_eq!(key, PROP_BUTTON_LABEL);
        assert_eq!(ty, IrType::Str);
    }

    #[test]
    fn resolve_prop_key_button_style_is_i32() {
        let (key, ty) = resolve_prop_key("Button", "style").expect("Button.style exists");
        assert_eq!(key, PROP_BUTTON_STYLE);
        assert_eq!(ty, IrType::I32);
    }

    #[test]
    fn resolve_prop_key_text_font_is_i32() {
        let (key, ty) = resolve_prop_key("Text", "font").expect("Text.font exists");
        assert_eq!(key, PROP_TEXT_STYLE);
        assert_eq!(ty, IrType::I32);
    }

    #[test]
    fn resolve_prop_key_unknown_pair_is_none() {
        assert!(resolve_prop_key("Button", "nonsuch").is_none());
        assert!(resolve_prop_key("Nonsuch", "enabled").is_none());
    }

    fn parse_ok(src: &str) -> IrComponent {
        match parse_ir(src) {
            Ok(c) => c,
            Err(e) => panic!("parse failed: {e}\nsrc:\n{src}"),
        }
    }

    #[test]
    fn header_required() {
        let err = parse_ir("component C inherits W { node V {} }").unwrap_err();
        assert!(matches!(err, IrLoadError::InvalidHeader(_)));
    }

    #[test]
    fn header_wrong_version() {
        let err = parse_ir(";wasamo-ir v9\ncomponent C inherits W { node V {} }").unwrap_err();
        assert!(matches!(err, IrLoadError::InvalidHeader(_)));
    }

    #[test]
    fn empty_component_with_root() {
        let c = parse_ok(";wasamo-ir v0\ncomponent C inherits W { node V {} }");
        assert_eq!(c.name, "C");
        assert_eq!(c.base, "W");
        assert!(c.states.is_empty());
        assert_eq!(c.root.widget_type, "V");
    }

    #[test]
    fn state_i32_with_int_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V {}\n}",
        );
        assert_eq!(c.states.len(), 1);
        assert_eq!(c.states[0].name, "count");
        assert_eq!(c.states[0].ty, IrType::I32);
        assert_eq!(c.states[0].default, IrLiteral::Int(0));
    }

    #[test]
    fn state_string_with_str_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state msg: string = \"hi\"\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].ty, IrType::Str);
        assert_eq!(c.states[0].default, IrLiteral::Str("hi".into()));
    }

    #[test]
    fn prop_int_and_ident_and_str() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V {\n\
               prop spacing = 12\n\
               prop theme = system\n\
               prop title = \"Hi\"\n\
             }\n}",
        );
        let props = &c.root.props;
        assert_eq!(props.len(), 3);
        assert_eq!(
            props[0],
            IrProp {
                name: "spacing".into(),
                value: IrLiteral::Int(12)
            }
        );
        assert_eq!(
            props[1],
            IrProp {
                name: "theme".into(),
                value: IrLiteral::Ident("system".into())
            }
        );
        assert_eq!(
            props[2],
            IrProp {
                name: "title".into(),
                value: IrLiteral::Str("Hi".into())
            }
        );
    }

    #[test]
    fn nested_nodes() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V {\n\
               node Text {}\n\
               node Button {}\n\
             }\n}",
        );
        assert_eq!(c.root.widget_type, "V");
        assert_eq!(c.root.children.len(), 2);
        assert_eq!(c.root.children[0].widget_type, "Text");
        assert_eq!(c.root.children[1].widget_type, "Button");
    }

    #[test]
    fn binding_with_prop_read() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V { bind text = (prop-read count) }\n}",
        );
        assert_eq!(c.root.bindings.len(), 1);
        let b = &c.root.bindings[0];
        assert_eq!(b.prop_name, "text");
        assert_eq!(
            b.expr,
            HandlerExpr::PropRead {
                path: "count".into()
            }
        );
    }

    #[test]
    fn binding_with_str_prop_read() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state label: string = \"hi\"\n\
             node V { bind text = (str-prop-read label) }\n}",
        );
        assert_eq!(c.root.bindings.len(), 1);
        let b = &c.root.bindings[0];
        assert_eq!(b.prop_name, "text");
        assert_eq!(
            b.expr,
            HandlerExpr::StrPropRead {
                path: "label".into()
            }
        );
    }

    #[test]
    fn binding_with_interpolation() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V { bind text = (interp \"Count: \" ((prop-read count))) }\n}",
        );
        let b = &c.root.bindings[0];
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
    fn handler_with_compound_assign() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node V { on clicked { (compound-assign += count 1) } }\n}",
        );
        assert_eq!(c.root.handlers.len(), 1);
        let h = &c.root.handlers[0];
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
    fn handler_with_assign_and_block() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state x: i32 = 0\n\
             state y: i32 = 0\n\
             node V { on clicked { (block (assign x 1) (assign y 2)) } }\n}",
        );
        let h = &c.root.handlers[0];
        let HandlerExpr::Block(exprs) = &h.expr else {
            panic!("expected block, got {:?}", h.expr);
        };
        assert_eq!(exprs.len(), 2);
        assert_eq!(
            exprs[0],
            HandlerExpr::Assign {
                lhs: "x".into(),
                rhs: Box::new(HandlerExpr::IntLit(1))
            }
        );
    }

    #[test]
    fn state_bool_with_false_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = false\n\
             node V {}\n}",
        );
        assert_eq!(c.states.len(), 1);
        assert_eq!(c.states[0].name, "ready");
        assert_eq!(c.states[0].ty, IrType::Bool);
        assert_eq!(c.states[0].default, IrLiteral::Bool(false));
    }

    #[test]
    fn state_bool_with_true_default() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].ty, IrType::Bool);
        assert_eq!(c.states[0].default, IrLiteral::Bool(true));
    }

    #[test]
    fn prop_bool_literal() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Button {\n\
               prop enabled = true\n\
             }\n}",
        );
        let props = &c.root.props;
        assert_eq!(
            props[0],
            IrProp {
                name: "enabled".into(),
                value: IrLiteral::Bool(true)
            }
        );
    }

    #[test]
    fn binding_with_bool_prop_read() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = false\n\
             node Button { bind enabled = (bool-prop-read ready) }\n}",
        );
        assert_eq!(c.root.bindings.len(), 1);
        let b = &c.root.bindings[0];
        assert_eq!(b.prop_name, "enabled");
        assert_eq!(
            b.expr,
            HandlerExpr::BoolPropRead {
                path: "ready".into()
            }
        );
    }

    #[test]
    fn binding_with_bool_literal() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node Button { bind enabled = true }\n}",
        );
        assert_eq!(c.root.bindings[0].expr, HandlerExpr::BoolLit(true));
    }

    #[test]
    fn handler_assign_bool_literal() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state ready: bool = true\n\
             node Button { on clicked { (assign ready false) } }\n}",
        );
        let h = &c.root.handlers[0];
        assert_eq!(
            h.expr,
            HandlerExpr::Assign {
                lhs: "ready".into(),
                rhs: Box::new(HandlerExpr::BoolLit(false))
            }
        );
    }

    #[test]
    fn handler_assign_bool_prop_read() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state a: bool = false\n\
             state b: bool = true\n\
             node V { on clicked { (assign a (bool-prop-read b)) } }\n}",
        );
        let h = &c.root.handlers[0];
        assert_eq!(
            h.expr,
            HandlerExpr::Assign {
                lhs: "a".into(),
                rhs: Box::new(HandlerExpr::BoolPropRead { path: "b".into() })
            }
        );
    }

    #[test]
    fn negative_int_literal() {
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state x: i32 = -5\n\
             node V {}\n}",
        );
        assert_eq!(c.states[0].default, IrLiteral::Int(-5));
    }

    #[test]
    fn parse_full_counter_round_trip() {
        // Build an IrComponent identical in shape to what wasamoc emits for
        // counter.ui, render it through a minimal hand-rolled emit (mirroring
        // wasamoc::emit's grammar §8), and assert parse_ir yields back the
        // same structure. This covers the full grammar surface (header,
        // state, prop, bind+interp, on+compound-assign, nested nodes).
        let original = IrComponent {
            name: "Counter".into(),
            base: "Window".into(),
            states: vec![IrState {
                name: "count".into(),
                ty: IrType::I32,
                default: IrLiteral::Int(0),
            }],
            root: IrNode {
                widget_type: "VStack".into(),
                props: vec![
                    IrProp {
                        name: "spacing".into(),
                        value: IrLiteral::Int(12),
                    },
                    IrProp {
                        name: "padding".into(),
                        value: IrLiteral::Int(24),
                    },
                ],
                bindings: vec![],
                handlers: vec![],
                children: vec![
                    IrNode {
                        widget_type: "Text".into(),
                        props: vec![IrProp {
                            name: "font".into(),
                            value: IrLiteral::Ident("title".into()),
                        }],
                        bindings: vec![IrBinding {
                            prop_name: "text".into(),
                            expr: HandlerExpr::Interpolation(vec![
                                InterpolationPart::Literal("Count: ".into()),
                                InterpolationPart::Expr(HandlerExpr::PropRead {
                                    path: "count".into(),
                                }),
                            ]),
                        }],
                        handlers: vec![],
                        children: vec![],
                    },
                    IrNode {
                        widget_type: "Button".into(),
                        props: vec![
                            IrProp {
                                name: "text".into(),
                                value: IrLiteral::Str("Increment".into()),
                            },
                            IrProp {
                                name: "style".into(),
                                value: IrLiteral::Ident("accent".into()),
                            },
                        ],
                        bindings: vec![],
                        handlers: vec![IrHandler {
                            signal: "clicked".into(),
                            expr: HandlerExpr::CompoundAssign {
                                op: CompoundOp::Add,
                                lhs: "count".into(),
                                rhs: Box::new(HandlerExpr::IntLit(1)),
                            },
                        }],
                        children: vec![],
                    },
                ],
            },
        };

        let text = render(&original);
        let parsed = parse_ok(&text);
        assert_eq!(parsed, original, "round-trip mismatch\nIR text:\n{text}");
    }

    /// Minimal IR text renderer mirroring `wasamoc::emit` (§8 normative grammar).
    /// Kept in the test module so the parser test does not require `wasamoc` as
    /// a dev-dependency; correctness vs. the compiler's emitter is enforced by
    /// the cross-crate round-trip integration test in
    /// `wasamo-runtime/tests/ir_loader_roundtrip.rs`.
    fn render(c: &IrComponent) -> String {
        let mut out = String::new();
        out.push_str(";wasamo-ir v0\n\n");
        out.push_str(&format!("component {} inherits {} {{\n", c.name, c.base));
        for s in &c.states {
            let ty = match s.ty {
                IrType::I32 => "i32",
                IrType::Str => "string",
                IrType::Bool => "bool",
            };
            out.push_str(&format!(
                "    state {}: {} = {}\n",
                s.name,
                ty,
                render_lit(&s.default)
            ));
        }
        if !c.states.is_empty() {
            out.push('\n');
        }
        render_node(&mut out, &c.root, 1);
        out.push_str("}\n");
        out
    }

    fn render_node(out: &mut String, n: &IrNode, depth: usize) {
        let i = "    ".repeat(depth);
        out.push_str(&format!("{i}node {} {{\n", n.widget_type));
        for p in &n.props {
            out.push_str(&format!(
                "{i}    prop {} = {}\n",
                p.name,
                render_lit(&p.value)
            ));
        }
        for b in &n.bindings {
            out.push_str(&format!(
                "{i}    bind {} = {}\n",
                b.prop_name,
                render_expr(&b.expr)
            ));
        }
        for h in &n.handlers {
            out.push_str(&format!("{i}    on {} {{\n", h.signal));
            out.push_str(&format!("{i}        {}\n", render_expr(&h.expr)));
            out.push_str(&format!("{i}    }}\n"));
        }
        for child in &n.children {
            render_node(out, child, depth + 1);
        }
        out.push_str(&format!("{i}}}\n"));
    }

    fn render_lit(l: &IrLiteral) -> String {
        match l {
            IrLiteral::Int(n) => n.to_string(),
            IrLiteral::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            IrLiteral::Ident(id) => id.clone(),
            IrLiteral::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
        }
    }

    fn render_expr(e: &HandlerExpr) -> String {
        match e {
            HandlerExpr::IntLit(n) => n.to_string(),
            HandlerExpr::StrLit(s) => format!("\"{}\"", s),
            HandlerExpr::BoolLit(b) => (if *b { "true" } else { "false" }).to_string(),
            HandlerExpr::PropRead { path } => format!("(prop-read {})", path),
            HandlerExpr::StrPropRead { path } => format!("(str-prop-read {})", path),
            HandlerExpr::BoolPropRead { path } => format!("(bool-prop-read {})", path),
            HandlerExpr::Assign { lhs, rhs } => format!("(assign {} {})", lhs, render_expr(rhs)),
            HandlerExpr::CompoundAssign { lhs, op, rhs } => {
                let op_str = match op {
                    CompoundOp::Add => "+=",
                    CompoundOp::Sub => "-=",
                    CompoundOp::Mul => "*=",
                    CompoundOp::Div => "/=",
                };
                format!("(compound-assign {} {} {})", op_str, lhs, render_expr(rhs))
            }
            HandlerExpr::Interpolation(parts) => {
                let inner: Vec<String> = parts
                    .iter()
                    .map(|p| match p {
                        InterpolationPart::Literal(s) => format!("\"{}\"", s),
                        InterpolationPart::Expr(e) => format!("({})", render_expr(e)),
                    })
                    .collect();
                format!("(interp {})", inner.join(" "))
            }
            HandlerExpr::Block(exprs) => {
                if exprs.is_empty() {
                    "(block)".to_string()
                } else {
                    let inner: Vec<String> = exprs.iter().map(render_expr).collect();
                    format!("(block {})", inner.join(" "))
                }
            }
        }
    }

    // ── DD-M2-P6-009 defense-in-depth validation tests ──────────────────

    fn parse_err(src: &str) -> IrLoadError {
        match parse_ir(src) {
            Ok(_) => panic!("expected parse error, but parse succeeded:\n{src}"),
            Err(e) => e,
        }
    }

    fn assert_malformed_display_nonempty(err: &IrLoadError) {
        assert!(
            err.is_malformed(),
            "expected malformed-class error, got {err:?}"
        );
        let msg = err.to_string();
        assert!(
            !msg.is_empty(),
            "Display impl must produce non-empty message for {err:?}"
        );
    }

    // Top-level structure ----------------------------------------------------

    #[test]
    fn malformed_no_root_node() {
        let err = parse_err(";wasamo-ir v0\ncomponent C inherits W { }");
        assert!(matches!(err, IrLoadError::Parse(ref m) if m.contains("no root node")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_multiple_root_nodes() {
        let err = parse_err(";wasamo-ir v0\ncomponent C inherits W { node V {} node W {} }");
        assert!(matches!(err, IrLoadError::Parse(ref m) if m.contains("multiple root nodes")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_missing_component_keyword() {
        let err = parse_err(";wasamo-ir v0\nfoo C inherits W { node V {} }");
        assert!(matches!(err, IrLoadError::Parse(_)));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_missing_inherits_keyword() {
        let err = parse_err(";wasamo-ir v0\ncomponent C foo W { node V {} }");
        assert!(matches!(err, IrLoadError::Parse(_)));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_trailing_tokens() {
        let err = parse_err(";wasamo-ir v0\ncomponent C inherits W { node V {} } stray");
        assert!(matches!(err, IrLoadError::Parse(ref m) if m.contains("trailing tokens")));
        assert_malformed_display_nonempty(&err);
    }

    // Reference resolution ---------------------------------------------------

    #[test]
    fn malformed_propread_undeclared() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { bind text = (prop-read missing) }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("missing")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_bool_prop_read_undeclared() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { bind enabled = (bool-prop-read missing) }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("missing")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_assign_undeclared() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { on clicked { (assign nope 1) } }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("nope")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_compound_assign_undeclared() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { on clicked { (compound-assign += counter 1) } }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("counter")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_undeclared_inside_interpolation() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             node V { bind text = (interp \"x: \" ((prop-read ghost))) }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("ghost")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_undeclared_inside_block() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state x: i32 = 0\n\
             node V { on clicked { (block (assign x 1) (assign y 2)) } }\n}",
        );
        // `x` is fine; `y` is not.
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("y")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_undeclared_in_child_node() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             node Root {\n\
               node Inner { bind text = (prop-read missing) }\n\
             }\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("missing")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn malformed_duplicate_state_name() {
        let err = parse_err(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             state count: string = \"hi\"\n\
             node V {}\n}",
        );
        assert!(matches!(err, IrLoadError::Validate(ref m) if m.contains("duplicate")));
        assert_malformed_display_nonempty(&err);
    }

    #[test]
    fn validate_passes_with_all_references_declared() {
        // Sanity-check: every reference resolves, so parse_ir succeeds.
        let c = parse_ok(
            ";wasamo-ir v0\ncomponent C inherits W {\n\
             state count: i32 = 0\n\
             state label: string = \"hi\"\n\
             node Root {\n\
               node Text { bind text = (interp \"n=\" ((prop-read count))) }\n\
               node Button { on clicked { (compound-assign += count 1) } }\n\
             }\n}",
        );
        assert_eq!(c.states.len(), 2);
    }

    #[test]
    fn header_error_is_malformed_class() {
        let err = parse_err("not-a-header\ncomponent C inherits W { node V {} }");
        assert!(matches!(err, IrLoadError::InvalidHeader(_)));
        assert_malformed_display_nonempty(&err);
    }
}
