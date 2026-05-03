// Spike IR loader — architectural feasibility probe for DD-M2-P2-001 Option B.
// Grammar and module structure are throwaway; only the pass/fail result matters.
//
// .uic format:
//   (component NAME
//     (state VAR i32 INIT)
//     (vstack SPACING PADDING CHILD...)
//     (text ID "CONTENT")
//     (button "LABEL" STYLE
//       (on clicked (assign-add VAR DELTA) (update-text TEXT-ID VAR PREFIX))))
//
// Supported styles: default, accent
// Handler ops: (assign-add VAR DELTA) (update-text TEXT-ID VAR PREFIX)

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::layout::Alignment;
use crate::text::TypographyStyle;
use crate::widget::{ButtonStyle, PropertyValue, WidgetNode, PROP_TEXT_CONTENT};

// ── Tokenizer ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,
    RParen,
    Str(String),
    Atom(String),
}

fn tokenize(src: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => { chars.next(); }
            ';' => { for c in chars.by_ref() { if c == '\n' { break; } } }
            '(' => { chars.next(); out.push(Token::LParen); }
            ')' => { chars.next(); out.push(Token::RParen); }
            '"' => {
                chars.next();
                let mut s = String::new();
                for c in chars.by_ref() {
                    if c == '"' { break; }
                    s.push(c);
                }
                out.push(Token::Str(s));
            }
            _ => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if matches!(c, '(' | ')' | '"' | ' ' | '\t' | '\n' | '\r') { break; }
                    s.push(c);
                    chars.next();
                }
                out.push(Token::Atom(s));
            }
        }
    }
    out
}

// ── S-expression parser ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    List(Vec<Expr>),
    Atom(String),
    Str(String),
    Int(i32),
}

fn parse_one(toks: &[Token], pos: &mut usize) -> Option<Expr> {
    if *pos >= toks.len() { return None; }
    match &toks[*pos] {
        Token::RParen => None,
        Token::LParen => {
            *pos += 1;
            let mut items = Vec::new();
            while *pos < toks.len() && toks[*pos] != Token::RParen {
                if let Some(e) = parse_one(toks, pos) { items.push(e); } else { break; }
            }
            if *pos < toks.len() { *pos += 1; }
            Some(Expr::List(items))
        }
        Token::Str(s) => { let e = Expr::Str(s.clone()); *pos += 1; Some(e) }
        Token::Atom(a) => {
            let e = if let Ok(n) = a.parse::<i32>() { Expr::Int(n) } else { Expr::Atom(a.clone()) };
            *pos += 1;
            Some(e)
        }
    }
}

fn parse(src: &str) -> Vec<Expr> {
    let toks = tokenize(src);
    let mut pos = 0;
    let mut out = Vec::new();
    while let Some(e) = parse_one(&toks, &mut pos) { out.push(e); }
    out
}

// ── Tagged value (Pass criterion 2) ──────────────────────────────────────────

#[derive(Debug, Clone)]
enum Value {
    #[allow(dead_code)]
    I32(i32),
    Str(String),
}

impl Value {
    fn as_property(&self) -> PropertyValue {
        match self {
            Value::I32(n) => PropertyValue::I32(*n),
            Value::Str(s) => PropertyValue::String(s.clone()),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn atom(e: &Expr) -> Option<&str> {
    match e { Expr::Atom(s) | Expr::Str(s) => Some(s), _ => None }
}

fn int_val(e: &Expr) -> Option<i32> {
    match e { Expr::Int(n) => Some(*n), Expr::Atom(s) => s.parse().ok(), _ => None }
}

fn float_val(e: &Expr) -> f32 {
    match e {
        Expr::Int(n) => *n as f32,
        Expr::Atom(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

// ── Tree builder ──────────────────────────────────────────────────────────────

struct BuildCtx {
    // Named integer variables from (state ...) declarations.
    vars: HashMap<String, Rc<Cell<i32>>>,
    // Widget pointers by id, registered after construction.
    ids: HashMap<String, *mut WidgetNode>,
}

fn build_widget(
    expr: &Expr,
    ctx: &mut BuildCtx,
) -> windows::core::Result<Box<WidgetNode>> {
    let rt = crate::runtime::get();
    let compositor = &rt.compositor;
    let renderer = &rt.text_renderer;

    let items = match expr {
        Expr::List(v) => v,
        _ => return Err(windows::core::Error::from(windows::Win32::Foundation::E_INVALIDARG)),
    };
    let tag = items.first().and_then(|e| atom(e)).unwrap_or("");

    match tag {
        "vstack" => {
            // (vstack SPACING PADDING CHILD...)
            let spacing = items.get(1).map(float_val).unwrap_or(0.0);
            let padding  = items.get(2).map(float_val).unwrap_or(0.0);
            let mut node = WidgetNode::vstack(compositor, spacing, padding, Alignment::Center)?;
            for child_expr in items.iter().skip(3) {
                let child = build_widget(child_expr, ctx)?;
                node.append_child(child)?;
            }
            Ok(node)
        }
        "text" => {
            // (text ID "CONTENT")
            let id      = items.get(1).and_then(atom).unwrap_or("").to_owned();
            let content = items.get(2).and_then(atom).unwrap_or("").to_owned();
            let mut node = WidgetNode::text(compositor, renderer, &content, TypographyStyle::Body)?;
            if !id.is_empty() {
                ctx.ids.insert(id, &mut *node as *mut WidgetNode);
            }
            Ok(node)
        }
        "button" => {
            // (button "LABEL" STYLE (on clicked ACTION...))
            let label     = items.get(1).and_then(atom).unwrap_or("").to_owned();
            let style_str = items.get(2).and_then(atom).unwrap_or("default");
            let btn_style = if style_str == "accent" { ButtonStyle::Accent } else { ButtonStyle::Default };

            let mut node = WidgetNode::button(compositor, renderer, &label, btn_style)?;

            // Find (on clicked ...) handler, if present.
            if let Some(Expr::List(handler)) = items.get(3) {
                if handler.first().and_then(atom) == Some("on")
                    && handler.get(1).and_then(atom) == Some("clicked")
                {
                    wire_handler(&mut node, &handler[2..], ctx);
                }
            }
            Ok(node)
        }
        other => {
            eprintln!("experimental_ir_loader: unknown widget tag `{other}`");
            Err(windows::core::Error::from(windows::Win32::Foundation::E_INVALIDARG))
        }
    }
}

// Wire the click handler from action expressions.
// Supported: (assign-add VAR DELTA) (update-text TEXT-ID VAR PREFIX)
fn wire_handler(button: &mut WidgetNode, actions: &[Expr], ctx: &mut BuildCtx) {
    // Resolve what we need from ctx now, before the 'static closure captures it.
    let mut assigns: Vec<(Rc<Cell<i32>>, i32)> = Vec::new();
    let mut updates: Vec<(*mut WidgetNode, Rc<Cell<i32>>, String)> = Vec::new();

    for action in actions {
        let items = match action { Expr::List(v) => v, _ => continue };
        let op = items.first().and_then(atom).unwrap_or("");
        match op {
            "assign-add" => {
                // (assign-add VAR DELTA)
                let var_name = items.get(1).and_then(atom).unwrap_or("");
                let delta    = items.get(2).and_then(int_val).unwrap_or(1);
                if let Some(cell) = ctx.vars.get(var_name) {
                    assigns.push((cell.clone(), delta));
                }
            }
            "update-text" => {
                // (update-text TEXT-ID VAR PREFIX)
                let text_id = items.get(1).and_then(atom).unwrap_or("");
                let var_name = items.get(2).and_then(atom).unwrap_or("");
                let prefix   = items.get(3).and_then(atom).unwrap_or("").to_owned();
                if let (Some(ptr), Some(cell)) =
                    (ctx.ids.get(text_id).copied(), ctx.vars.get(var_name))
                {
                    updates.push((ptr, cell.clone(), prefix));
                }
            }
            _ => {}
        }
    }

    button.set_clicked(move || {
        for (cell, delta) in &assigns {
            cell.set(cell.get() + delta);
        }
        for (ptr, cell, prefix) in &updates {
            let new_text = format!("{}{}", prefix, cell.get());
            let val = Value::Str(new_text).as_property();
            // Safety: the widget tree lives in WindowState which outlives the
            // message loop, so the pointer is valid for the program's lifetime.
            unsafe { (*(*ptr)).set_property(PROP_TEXT_CONTENT, &val).ok(); }
        }
    });
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Load a `.uic` file and return the root widget tree.
pub fn load(path: &str) -> windows::core::Result<Box<WidgetNode>> {
    let src = std::fs::read_to_string(path).map_err(|e| {
        windows::core::Error::new(windows::Win32::Foundation::E_FAIL, e.to_string())
    })?;

    let exprs = parse(&src);

    // Expect: (component NAME (state VAR i32 INIT) WIDGET-EXPR)
    let top = match exprs.first() {
        Some(Expr::List(v)) => v,
        _ => return Err(windows::core::Error::from(windows::Win32::Foundation::E_INVALIDARG)),
    };

    if top.first().and_then(atom) != Some("component") {
        return Err(windows::core::Error::from(windows::Win32::Foundation::E_INVALIDARG));
    }

    let mut ctx = BuildCtx { vars: HashMap::new(), ids: HashMap::new() };

    // Collect state declarations and find root widget expr.
    let mut root_expr: Option<&Expr> = None;
    for item in top.iter().skip(2) {
        if let Expr::List(parts) = item {
            if parts.first().and_then(atom) == Some("state") {
                // (state VAR i32 INIT)
                if let (Some(name), Some(init)) = (parts.get(1).and_then(atom), parts.get(3).and_then(int_val)) {
                    ctx.vars.insert(name.to_owned(), Rc::new(Cell::new(init)));
                }
                continue;
            }
        }
        root_expr = Some(item);
    }

    let root = root_expr
        .ok_or_else(|| windows::core::Error::from(windows::Win32::Foundation::E_INVALIDARG))?;

    build_widget(root, &mut ctx)
}

// ── Tests (pure logic: tokenizer + parser only) ───────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_basic() {
        let toks = tokenize(r#"(vstack 12 "hello")"#);
        assert_eq!(toks, vec![
            Token::LParen,
            Token::Atom("vstack".into()),
            Token::Atom("12".into()),
            Token::Str("hello".into()),
            Token::RParen,
        ]);
    }

    #[test]
    fn tokenize_skips_comments() {
        let toks = tokenize("; comment\n(a)");
        assert_eq!(toks, vec![Token::LParen, Token::Atom("a".into()), Token::RParen]);
    }

    #[test]
    fn parse_nested() {
        let exprs = parse("(component Counter (state count i32 0) (text lbl \"Count: 0\"))");
        assert!(matches!(exprs.first(), Some(Expr::List(_))));
        if let Some(Expr::List(top)) = exprs.first() {
            assert!(matches!(top.first(), Some(Expr::Atom(s)) if s == "component"));
            assert_eq!(top.len(), 4); // component + Counter + state + text
        }
    }

    #[test]
    fn parse_int_atom() {
        let exprs = parse("(spacing 12)");
        if let Some(Expr::List(items)) = exprs.first() {
            assert!(matches!(items.get(1), Some(Expr::Int(12))));
        }
    }

    #[test]
    fn value_as_property_roundtrip() {
        let v = Value::Str("Count: 5".into());
        let pv = v.as_property();
        assert!(matches!(pv, PropertyValue::String(s) if s == "Count: 5"));

        let v2 = Value::I32(42);
        let pv2 = v2.as_property();
        assert!(matches!(pv2, PropertyValue::I32(42)));
    }
}
