use crate::ast::{QualifiedName, Span, StringPart, Unit};
use crate::diagnostic::Diagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Component,
    Inherits,
    InOut,
    Property,
}

impl Keyword {
    pub fn description(&self) -> &'static str {
        match self {
            Keyword::Component => "`component`",
            Keyword::Inherits => "`inherits`",
            Keyword::InOut => "`in-out`",
            Keyword::Property => "`property`",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Kw(Keyword),
    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    Measurement(f64, Unit),
    StringLit(Vec<StringPart>),
    LBrace,
    RBrace,
    LAngle,
    RAngle,
    Colon,
    Arrow,
    Dot,
    Semicolon,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    Eq,
    Eof,
}

impl Token {
    pub fn is_kw(&self, kw: &Keyword) -> bool {
        matches!(self, Token::Kw(k) if k == kw)
    }

    pub fn as_ident(&self) -> Option<&str> {
        if let Token::Ident(s) = self { Some(s) } else { None }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Token::Kw(k) => k.description(),
            Token::Ident(_) => "identifier",
            Token::IntLit(_) => "integer literal",
            Token::FloatLit(_) => "float literal",
            Token::Measurement(_, _) => "measurement",
            Token::StringLit(_) => "string literal",
            Token::LBrace => "`{`",
            Token::RBrace => "`}`",
            Token::LAngle => "`<`",
            Token::RAngle => "`>`",
            Token::Colon => "`:`",
            Token::Arrow => "`=>`",
            Token::Dot => "`.`",
            Token::Semicolon => "`;`",
            Token::PlusEq => "`+=`",
            Token::MinusEq => "`-=`",
            Token::StarEq => "`*=`",
            Token::SlashEq => "`/=`",
            Token::Eq => "`=`",
            Token::Eof => "end of file",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

struct Cursor<'a> {
    src: &'a str,
    pos: usize,
    line: u32,
    col: u32,
}

impl<'a> Cursor<'a> {
    fn new(src: &'a str) -> Self {
        Cursor { src, pos: 0, line: 1, col: 1 }
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn peek2(&self) -> Option<char> {
        let mut it = self.src[self.pos..].chars();
        it.next();
        it.next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.src[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn remaining(&self) -> &str {
        &self.src[self.pos..]
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn here(&self) -> Span {
        Span { start: self.pos, end: self.pos, line: self.line, col: self.col }
    }
}

pub fn tokenize(src: &str, filename: &str) -> Result<Vec<SpannedToken>, Diagnostic> {
    let mut c = Cursor::new(src);
    let mut tokens = Vec::new();

    loop {
        while c.peek().map(|ch| ch.is_whitespace()).unwrap_or(false) {
            c.advance();
        }

        if c.is_at_end() {
            let sp = c.here();
            tokens.push(SpannedToken { token: Token::Eof, span: sp });
            break;
        }

        let start = c.here();
        let ch = c.peek().unwrap();

        let token = match ch {
            '{' => { c.advance(); Token::LBrace }
            '}' => { c.advance(); Token::RBrace }
            '<' => { c.advance(); Token::LAngle }
            '>' => { c.advance(); Token::RAngle }
            ':' => { c.advance(); Token::Colon }
            '.' => { c.advance(); Token::Dot }
            ';' => { c.advance(); Token::Semicolon }
            '=' => {
                c.advance();
                if c.peek() == Some('>') { c.advance(); Token::Arrow } else { Token::Eq }
            }
            '+' => {
                c.advance();
                if c.peek() == Some('=') {
                    c.advance();
                    Token::PlusEq
                } else {
                    return Err(Diagnostic::error(filename, start.line, start.col, "expected `+=`"));
                }
            }
            '-' => {
                c.advance();
                if c.peek() == Some('=') {
                    c.advance();
                    Token::MinusEq
                } else {
                    return Err(Diagnostic::error(filename, start.line, start.col, "unexpected `-`"));
                }
            }
            '*' => {
                c.advance();
                if c.peek() == Some('=') {
                    c.advance();
                    Token::StarEq
                } else {
                    return Err(Diagnostic::error(filename, start.line, start.col, "unexpected `*`"));
                }
            }
            '/' => {
                c.advance();
                if c.peek() == Some('=') {
                    c.advance();
                    Token::SlashEq
                } else {
                    return Err(Diagnostic::error(filename, start.line, start.col, "unexpected `/`"));
                }
            }
            '"' => scan_string(&mut c, filename)?,
            '0'..='9' => scan_number(&mut c, filename, start.line, start.col)?,
            ch if ch.is_alphabetic() || ch == '_' => scan_ident(&mut c),
            other => {
                c.advance();
                return Err(Diagnostic::error(
                    filename, start.line, start.col,
                    format!("unexpected character `{}`", other),
                ));
            }
        };

        tokens.push(SpannedToken {
            token,
            span: Span { start: start.start, end: c.pos, line: start.line, col: start.col },
        });
    }

    Ok(tokens)
}

fn scan_ident(c: &mut Cursor) -> Token {
    let mut s = String::new();
    while let Some(ch) = c.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            s.push(ch);
            c.advance();
        } else {
            break;
        }
    }

    // Compound keyword "in-out" (DD-001)
    if s == "in" && c.remaining().starts_with("-out") {
        let after = c.remaining()[4..].chars().next();
        if !after.map(|ch| ch.is_alphanumeric() || ch == '_').unwrap_or(false) {
            for _ in 0..4 { c.advance(); }
            return Token::Kw(Keyword::InOut);
        }
    }

    match s.as_str() {
        "component" => Token::Kw(Keyword::Component),
        "inherits" => Token::Kw(Keyword::Inherits),
        "property" => Token::Kw(Keyword::Property),
        _ => Token::Ident(s),
    }
}

fn scan_number(c: &mut Cursor, filename: &str, line: u32, col: u32) -> Result<Token, Diagnostic> {
    let mut s = String::new();
    let mut is_float = false;

    while let Some(ch) = c.peek() {
        if ch.is_ascii_digit() { s.push(ch); c.advance(); } else { break; }
    }

    if c.peek() == Some('.') && c.peek2().map(|ch| ch.is_ascii_digit()).unwrap_or(false) {
        s.push('.'); c.advance(); is_float = true;
        while let Some(ch) = c.peek() {
            if ch.is_ascii_digit() { s.push(ch); c.advance(); } else { break; }
        }
    }

    // Unit "px"
    if c.remaining().starts_with("px") {
        let after = c.remaining()[2..].chars().next();
        if !after.map(|ch| ch.is_alphanumeric() || ch == '_').unwrap_or(false) {
            c.advance(); c.advance();
            let value: f64 = s.parse().unwrap();
            return Ok(Token::Measurement(value, Unit::Px));
        }
    }

    if is_float {
        Ok(Token::FloatLit(s.parse().unwrap()))
    } else {
        s.parse::<i64>()
            .map(Token::IntLit)
            .map_err(|_| Diagnostic::error(filename, line, col, "integer literal out of range"))
    }
}

fn scan_string(c: &mut Cursor, filename: &str) -> Result<Token, Diagnostic> {
    c.advance(); // opening "
    let mut parts: Vec<StringPart> = Vec::new();
    let mut text = String::new();

    loop {
        match c.peek() {
            None => {
                return Err(Diagnostic::error(filename, c.line, c.col, "unterminated string literal"));
            }
            Some('"') => {
                c.advance();
                if !text.is_empty() {
                    parts.push(StringPart::Text(std::mem::take(&mut text)));
                }
                return Ok(Token::StringLit(parts));
            }
            Some('\\') => {
                c.advance();
                match c.peek() {
                    Some('\\') => { c.advance(); text.push('\\'); }
                    Some('"') => { c.advance(); text.push('"'); }
                    Some('{') => {
                        c.advance();
                        if !text.is_empty() {
                            parts.push(StringPart::Text(std::mem::take(&mut text)));
                        }
                        parts.push(StringPart::Interp(scan_interp(c, filename)?));
                    }
                    Some(ch) => {
                        return Err(Diagnostic::error(
                            filename, c.line, c.col,
                            format!("unknown escape sequence `\\{}`", ch),
                        ));
                    }
                    None => {
                        return Err(Diagnostic::error(filename, c.line, c.col, "unterminated escape sequence"));
                    }
                }
            }
            Some('\n') | Some('\r') => {
                return Err(Diagnostic::error(filename, c.line, c.col, "unterminated string literal"));
            }
            Some(ch) => { text.push(ch); c.advance(); }
        }
    }
}

fn scan_interp(c: &mut Cursor, filename: &str) -> Result<QualifiedName, Diagnostic> {
    let start_pos = c.pos;
    let start_line = c.line;
    let start_col = c.col;

    let mut segments = vec![scan_interp_ident(c, filename)?];

    while c.peek() == Some('.') {
        c.advance();
        segments.push(scan_interp_ident(c, filename)?);
    }

    if c.peek() != Some('}') {
        return Err(Diagnostic::error(filename, c.line, c.col, "expected `}` to close interpolation"));
    }
    c.advance();

    Ok(QualifiedName {
        segments,
        span: Span { start: start_pos, end: c.pos, line: start_line, col: start_col },
    })
}

fn scan_interp_ident(c: &mut Cursor, filename: &str) -> Result<String, Diagnostic> {
    match c.peek() {
        Some(ch) if ch.is_alphabetic() || ch == '_' => {}
        _ => {
            return Err(Diagnostic::error(
                filename, c.line, c.col,
                "expected identifier in interpolation",
            ));
        }
    }
    let mut s = String::new();
    while let Some(ch) = c.peek() {
        if ch.is_alphanumeric() || ch == '_' { s.push(ch); c.advance(); } else { break; }
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{StringPart, Unit};

    fn lex_ok(src: &str) -> Vec<Token> {
        tokenize(src, "<test>")
            .expect("tokenize failed")
            .into_iter()
            .map(|st| st.token)
            .collect()
    }

    fn lex_err(src: &str) -> String {
        tokenize(src, "<test>").expect_err("expected error").message
    }

    #[test]
    fn keywords() {
        let toks = lex_ok("component inherits property");
        assert!(matches!(&toks[0], Token::Kw(Keyword::Component)));
        assert!(matches!(&toks[1], Token::Kw(Keyword::Inherits)));
        assert!(matches!(&toks[2], Token::Kw(Keyword::Property)));
    }

    #[test]
    fn in_out_keyword() {
        let toks = lex_ok("in-out");
        assert!(matches!(&toks[0], Token::Kw(Keyword::InOut)));
        assert!(matches!(&toks[1], Token::Eof));
    }

    #[test]
    fn in_out_followed_by_space() {
        let toks = lex_ok("in-out property");
        assert!(matches!(&toks[0], Token::Kw(Keyword::InOut)));
        assert!(matches!(&toks[1], Token::Kw(Keyword::Property)));
    }

    #[test]
    fn in_out_followed_by_alphanumeric_is_error() {
        // "in-outx": scans "in", hits "-" which is not "-=" so errors
        assert!(lex_err("in-outx").contains("unexpected"));
    }

    #[test]
    fn identifiers() {
        let toks = lex_ok("Counter _foo abc123");
        assert!(matches!(&toks[0], Token::Ident(s) if s == "Counter"));
        assert!(matches!(&toks[1], Token::Ident(s) if s == "_foo"));
        assert!(matches!(&toks[2], Token::Ident(s) if s == "abc123"));
    }

    #[test]
    fn integer_literals() {
        let toks = lex_ok("0 42 100");
        assert!(matches!(&toks[0], Token::IntLit(0)));
        assert!(matches!(&toks[1], Token::IntLit(42)));
        assert!(matches!(&toks[2], Token::IntLit(100)));
    }

    #[test]
    fn float_literal() {
        let toks = lex_ok("3.14");
        assert!(matches!(&toks[0], Token::FloatLit(v) if *v == 3.14_f64));
    }

    #[test]
    fn measurement_px() {
        let toks = lex_ok("12px 0px");
        assert!(matches!(&toks[0], Token::Measurement(v, Unit::Px) if *v == 12.0));
        assert!(matches!(&toks[1], Token::Measurement(v, Unit::Px) if *v == 0.0));
    }

    #[test]
    fn punctuation() {
        let toks = lex_ok("{ } < > : . ;");
        assert!(matches!(&toks[0], Token::LBrace));
        assert!(matches!(&toks[1], Token::RBrace));
        assert!(matches!(&toks[2], Token::LAngle));
        assert!(matches!(&toks[3], Token::RAngle));
        assert!(matches!(&toks[4], Token::Colon));
        assert!(matches!(&toks[5], Token::Dot));
        assert!(matches!(&toks[6], Token::Semicolon));
    }

    #[test]
    fn arrow_and_eq() {
        let toks = lex_ok("=> =");
        assert!(matches!(&toks[0], Token::Arrow));
        assert!(matches!(&toks[1], Token::Eq));
    }

    #[test]
    fn compound_assign_ops() {
        let toks = lex_ok("+= -= *= /=");
        assert!(matches!(&toks[0], Token::PlusEq));
        assert!(matches!(&toks[1], Token::MinusEq));
        assert!(matches!(&toks[2], Token::StarEq));
        assert!(matches!(&toks[3], Token::SlashEq));
    }

    #[test]
    fn string_plain_text() {
        let toks = lex_ok(r#""hello""#);
        if let Token::StringLit(parts) = &toks[0] {
            assert_eq!(parts.len(), 1);
            assert!(matches!(&parts[0], StringPart::Text(s) if s == "hello"));
        } else {
            panic!("expected StringLit");
        }
    }

    #[test]
    fn string_escape_backslash() {
        // DSL source: "\\" → single backslash in the text value
        let toks = lex_ok("\"\\\\\"");
        if let Token::StringLit(parts) = &toks[0] {
            assert_eq!(parts.len(), 1);
            assert!(matches!(&parts[0], StringPart::Text(s) if s == "\\"));
        } else {
            panic!("expected StringLit");
        }
    }

    #[test]
    fn string_escape_quote() {
        // DSL source: "\"" → double-quote in the text value
        let toks = lex_ok("\"\\\"\"");
        if let Token::StringLit(parts) = &toks[0] {
            assert_eq!(parts.len(), 1);
            assert!(matches!(&parts[0], StringPart::Text(s) if s == "\""));
        } else {
            panic!("expected StringLit");
        }
    }

    #[test]
    fn string_interpolation_single_ident() {
        let toks = lex_ok(r#""Count: \{count}""#);
        if let Token::StringLit(parts) = &toks[0] {
            assert_eq!(parts.len(), 2);
            assert!(matches!(&parts[0], StringPart::Text(s) if s == "Count: "));
            if let StringPart::Interp(qn) = &parts[1] {
                assert_eq!(qn.segments, vec!["count"]);
            } else {
                panic!("expected Interp");
            }
        } else {
            panic!("expected StringLit");
        }
    }

    #[test]
    fn string_interpolation_dotted_path() {
        let toks = lex_ok(r#""\{root.count}""#);
        if let Token::StringLit(parts) = &toks[0] {
            assert_eq!(parts.len(), 1);
            if let StringPart::Interp(qn) = &parts[0] {
                assert_eq!(qn.segments, vec!["root", "count"]);
            } else {
                panic!("expected Interp");
            }
        } else {
            panic!("expected StringLit");
        }
    }

    #[test]
    fn error_unterminated_string() {
        assert!(lex_err("\"hello").contains("unterminated"));
    }

    #[test]
    fn error_unknown_escape() {
        assert!(lex_err("\"\\n\"").contains("unknown escape"));
    }

    #[test]
    fn error_bare_minus() {
        assert!(lex_err("- ").contains("unexpected"));
    }

    #[test]
    fn error_unexpected_character() {
        assert!(lex_err("@").contains("unexpected character"));
    }

    #[test]
    fn error_interp_missing_ident_after_dot() {
        // \{root.} — dot followed by closing brace: missing ident
        assert!(lex_err(r#""\{root.}""#).contains("expected identifier"));
    }

    #[test]
    fn span_line_and_col() {
        let tokens = tokenize("component\n  Foo", "<test>").unwrap();
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.col, 1);
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[1].span.col, 3);
    }
}
