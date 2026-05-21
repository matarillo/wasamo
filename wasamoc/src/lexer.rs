use crate::ast::{QualifiedName, Span, StringPart, Unit};
use crate::diagnostic::Diagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Component,
    Inherits,
    InOut,
    Property,
    State,
    True,
    False,
}

impl Keyword {
    pub fn description(&self) -> &'static str {
        match self {
            Keyword::Component => "`component`",
            Keyword::Inherits => "`inherits`",
            Keyword::InOut => "`in-out`",
            Keyword::Property => "`property`",
            Keyword::State => "`state`",
            Keyword::True => "`true`",
            Keyword::False => "`false`",
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
    /// Ratio literal `<num>:<den>` (DD-M3-P2-002). Both sides are
    /// non-negative integers that fit in `i32`; value validity
    /// (`num > 0`, `den > 0`) is enforced at `wasamoc check` time
    /// per dsl_spec §4.9, not here.
    RatioLit(i32, i32),
    /// Color literal `#RRGGBB` / `#RRGGBBAA` (DD-M3-P2-003), packed
    /// to `0xAARRGGBB` (alpha in MSB). 6-digit form takes `AA = 0xFF`.
    ColorLit(u32),
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
        if let Token::Ident(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Token::Kw(k) => k.description(),
            Token::Ident(_) => "identifier",
            Token::IntLit(_) => "integer literal",
            Token::FloatLit(_) => "float literal",
            Token::Measurement(_, _) => "measurement",
            Token::StringLit(_) => "string literal",
            Token::RatioLit(_, _) => "ratio literal",
            Token::ColorLit(_) => "color literal",
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
        Cursor {
            src,
            pos: 0,
            line: 1,
            col: 1,
        }
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
        Span {
            start: self.pos,
            end: self.pos,
            line: self.line,
            col: self.col,
        }
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
            tokens.push(SpannedToken {
                token: Token::Eof,
                span: sp,
            });
            break;
        }

        let start = c.here();
        let ch = c.peek().unwrap();

        let token = match ch {
            '{' => {
                c.advance();
                Token::LBrace
            }
            '}' => {
                c.advance();
                Token::RBrace
            }
            '<' => {
                c.advance();
                Token::LAngle
            }
            '>' => {
                c.advance();
                Token::RAngle
            }
            ':' => {
                c.advance();
                Token::Colon
            }
            '.' => {
                c.advance();
                Token::Dot
            }
            ';' => {
                c.advance();
                Token::Semicolon
            }
            '=' => {
                c.advance();
                if c.peek() == Some('>') {
                    c.advance();
                    Token::Arrow
                } else {
                    Token::Eq
                }
            }
            '+' => {
                c.advance();
                if c.peek() == Some('=') {
                    c.advance();
                    Token::PlusEq
                } else {
                    return Err(Diagnostic::error(
                        filename,
                        start.line,
                        start.col,
                        "expected `+=`",
                    ));
                }
            }
            '-' => {
                c.advance();
                if c.peek() == Some('=') {
                    c.advance();
                    Token::MinusEq
                } else if c.peek().map(|ch| ch.is_ascii_digit()).unwrap_or(false) {
                    // Negative integer literal. The DSL surface has no
                    // binary subtraction (M2 statement RHSes accept literal
                    // / ident expressions only), so a `-` directly preceding
                    // digits is unambiguous as the sign of an integer
                    // literal. Required by M3-Phase 3 to give `wasamoc
                    // check` reach over `item-cross-size: -1` etc. so the
                    // rejection diagnostic can name the offending attribute
                    // (DD-M3-P3-006 compile-time gate). The negative entry
                    // path in `scan_number` is integer-only: a fractional
                    // continuation (`-1.5`) or `px` unit (`-12px`) is
                    // rejected at lex time. Ratio literals (`-num:den`)
                    // are similarly excluded because RatioLit is unsigned
                    // per dsl_spec §4.9.
                    scan_number(&mut c, filename, start.line, start.col, true)?
                } else {
                    return Err(Diagnostic::error(
                        filename,
                        start.line,
                        start.col,
                        "unexpected `-`",
                    ));
                }
            }
            '*' => {
                c.advance();
                if c.peek() == Some('=') {
                    c.advance();
                    Token::StarEq
                } else {
                    return Err(Diagnostic::error(
                        filename,
                        start.line,
                        start.col,
                        "unexpected `*`",
                    ));
                }
            }
            '/' => {
                c.advance();
                if c.peek() == Some('=') {
                    c.advance();
                    Token::SlashEq
                } else {
                    return Err(Diagnostic::error(
                        filename,
                        start.line,
                        start.col,
                        "unexpected `/`",
                    ));
                }
            }
            '"' => scan_string(&mut c, filename)?,
            '#' => scan_color(&mut c, filename, start.line, start.col)?,
            '0'..='9' => scan_number(&mut c, filename, start.line, start.col, false)?,
            ch if ch.is_alphabetic() || ch == '_' => scan_ident(&mut c),
            other => {
                c.advance();
                return Err(Diagnostic::error(
                    filename,
                    start.line,
                    start.col,
                    format!("unexpected character `{}`", other),
                ));
            }
        };

        tokens.push(SpannedToken {
            token,
            span: Span {
                start: start.start,
                end: c.pos,
                line: start.line,
                col: start.col,
            },
        });
    }

    Ok(tokens)
}

fn scan_ident(c: &mut Cursor) -> Token {
    // Kebab-case identifier: one or more alphanumeric / underscore segments,
    // joined by `-`. Continuation requires the character following `-` to be
    // alphabetic, so `count -= 1` still tokenizes as `count`, `-=`, `1`
    // (the `-` after `count` is followed by `=`, breaking out of scan_ident).
    // Generalized in M3-Phase 3 from the previous `in-out`-only special-case
    // to admit kebab attribute names (`item-cross-size`, `item-spacing`,
    // `line-spacing`) without a parser-grammar change. `in-out` remains a
    // keyword via the post-scan match below.
    let mut s = String::new();
    loop {
        while let Some(ch) = c.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(ch);
                c.advance();
            } else {
                break;
            }
        }
        if c.peek() == Some('-') && c.peek2().map(|ch| ch.is_alphabetic()).unwrap_or(false) {
            s.push('-');
            c.advance();
            continue;
        }
        break;
    }

    match s.as_str() {
        "component" => Token::Kw(Keyword::Component),
        "inherits" => Token::Kw(Keyword::Inherits),
        "in-out" => Token::Kw(Keyword::InOut),
        "property" => Token::Kw(Keyword::Property),
        "state" => Token::Kw(Keyword::State),
        "true" => Token::Kw(Keyword::True),
        "false" => Token::Kw(Keyword::False),
        _ => Token::Ident(s),
    }
}

fn scan_number(
    c: &mut Cursor,
    filename: &str,
    line: u32,
    col: u32,
    negative: bool,
) -> Result<Token, Diagnostic> {
    let mut s = String::new();
    if negative {
        s.push('-');
    }
    let mut is_float = false;

    while let Some(ch) = c.peek() {
        if ch.is_ascii_digit() {
            s.push(ch);
            c.advance();
        } else {
            break;
        }
    }

    if c.peek() == Some('.') && c.peek2().map(|ch| ch.is_ascii_digit()).unwrap_or(false) {
        if negative {
            // Negative entry is integer-only. dsl_spec FloatLit surface is
            // unsigned (§5 AST table); the M3-Phase 3 lexer extension
            // exists only to give `wasamoc check` reach over negative
            // integer literals on WrapPanel attributes (DD-M3-P3-006).
            // Reject a fractional continuation explicitly rather than
            // silently widening the FloatLit surface.
            return Err(Diagnostic::error(
                filename,
                line,
                col,
                "negative float literals are not part of the DSL surface; \
                 `-` followed by digits only forms a negative integer",
            ));
        }
        s.push('.');
        c.advance();
        is_float = true;
        while let Some(ch) = c.peek() {
            if ch.is_ascii_digit() {
                s.push(ch);
                c.advance();
            } else {
                break;
            }
        }
    }

    // Unit "px". Negative measurements are not part of the DSL surface —
    // the spec's measurement form (`12px`) is unsigned.
    if c.remaining().starts_with("px") {
        let after = c.remaining()[2..].chars().next();
        if !after
            .map(|ch| ch.is_alphanumeric() || ch == '_')
            .unwrap_or(false)
        {
            if negative {
                return Err(Diagnostic::error(
                    filename,
                    line,
                    col,
                    "negative measurement literals are not part of the DSL surface; \
                     `-` followed by digits only forms a negative integer",
                ));
            }
            c.advance();
            c.advance();
            let value: f64 = s.parse().unwrap();
            return Ok(Token::Measurement(value, Unit::Px));
        }
    }

    if is_float {
        return Ok(Token::FloatLit(s.parse().unwrap()));
    }

    // Ratio literal `<num>:<den>` (DD-M3-P2-002). Only the unsigned entry
    // path produces a RatioLit; a leading `-` cannot start a ratio (negative
    // ratios are not part of the spec surface).
    if !negative
        && c.peek() == Some(':')
        && c.peek2().map(|ch| ch.is_ascii_digit()).unwrap_or(false)
    {
        let num = s.parse::<i32>().map_err(|_| {
            Diagnostic::error(filename, line, col, "ratio numerator out of range for i32")
        })?;
        c.advance(); // consume ':'
        let den_line = c.line;
        let den_col = c.col;
        let mut den_str = String::new();
        while let Some(ch) = c.peek() {
            if ch.is_ascii_digit() {
                den_str.push(ch);
                c.advance();
            } else {
                break;
            }
        }
        let den = den_str.parse::<i32>().map_err(|_| {
            Diagnostic::error(
                filename,
                den_line,
                den_col,
                "ratio denominator out of range for i32",
            )
        })?;
        return Ok(Token::RatioLit(num, den));
    }

    s.parse::<i64>()
        .map(Token::IntLit)
        .map_err(|_| Diagnostic::error(filename, line, col, "integer literal out of range"))
}

fn scan_color(c: &mut Cursor, filename: &str, line: u32, col: u32) -> Result<Token, Diagnostic> {
    // Consume the leading `#`.
    c.advance();

    let mut hex = String::new();
    while let Some(ch) = c.peek() {
        if ch.is_ascii_hexdigit() {
            hex.push(ch);
            c.advance();
        } else {
            break;
        }
    }

    // Spec §2.2 / §4.9: `#` followed by exactly 6 or 8 hex digits.
    // Pack to `0xAARRGGBB` per dsl_spec (alpha in MSB); 6-digit form
    // takes `AA = 0xFF`.
    let packed = match hex.len() {
        6 => {
            let rgb = u32::from_str_radix(&hex, 16).unwrap();
            0xFF00_0000 | rgb
        }
        8 => {
            let rr = u32::from_str_radix(&hex[0..2], 16).unwrap();
            let gg = u32::from_str_radix(&hex[2..4], 16).unwrap();
            let bb = u32::from_str_radix(&hex[4..6], 16).unwrap();
            let aa = u32::from_str_radix(&hex[6..8], 16).unwrap();
            (aa << 24) | (rr << 16) | (gg << 8) | bb
        }
        n => {
            return Err(Diagnostic::error(
                filename,
                line,
                col,
                format!(
                    "color literal `#{}` must have 6 or 8 hex digits, got {}",
                    hex, n
                ),
            ));
        }
    };

    Ok(Token::ColorLit(packed))
}

fn scan_string(c: &mut Cursor, filename: &str) -> Result<Token, Diagnostic> {
    c.advance(); // opening "
    let mut parts: Vec<StringPart> = Vec::new();
    let mut text = String::new();

    loop {
        match c.peek() {
            None => {
                return Err(Diagnostic::error(
                    filename,
                    c.line,
                    c.col,
                    "unterminated string literal",
                ));
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
                    Some('\\') => {
                        c.advance();
                        text.push('\\');
                    }
                    Some('"') => {
                        c.advance();
                        text.push('"');
                    }
                    Some('{') => {
                        c.advance();
                        if !text.is_empty() {
                            parts.push(StringPart::Text(std::mem::take(&mut text)));
                        }
                        parts.push(StringPart::Interp(scan_interp(c, filename)?));
                    }
                    Some(ch) => {
                        return Err(Diagnostic::error(
                            filename,
                            c.line,
                            c.col,
                            format!("unknown escape sequence `\\{}`", ch),
                        ));
                    }
                    None => {
                        return Err(Diagnostic::error(
                            filename,
                            c.line,
                            c.col,
                            "unterminated escape sequence",
                        ));
                    }
                }
            }
            Some('\n') | Some('\r') => {
                return Err(Diagnostic::error(
                    filename,
                    c.line,
                    c.col,
                    "unterminated string literal",
                ));
            }
            Some(ch) => {
                text.push(ch);
                c.advance();
            }
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
        return Err(Diagnostic::error(
            filename,
            c.line,
            c.col,
            "expected `}` to close interpolation",
        ));
    }
    c.advance();

    Ok(QualifiedName {
        segments,
        span: Span {
            start: start_pos,
            end: c.pos,
            line: start_line,
            col: start_col,
        },
    })
}

fn scan_interp_ident(c: &mut Cursor, filename: &str) -> Result<String, Diagnostic> {
    match c.peek() {
        Some(ch) if ch.is_alphabetic() || ch == '_' => {}
        _ => {
            return Err(Diagnostic::error(
                filename,
                c.line,
                c.col,
                "expected identifier in interpolation",
            ));
        }
    }
    let mut s = String::new();
    while let Some(ch) = c.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            s.push(ch);
            c.advance();
        } else {
            break;
        }
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
    fn in_outx_lexes_as_kebab_ident() {
        // Post-M3-Phase 3 generalization: `-` continues an ident when the
        // next char is alphabetic, so `in-outx` lexes as a single Ident
        // (no keyword match). This replaces the pre-generalization
        // "unexpected `-`" error path; `in-out` itself is matched by the
        // keyword table after the kebab-aware scan.
        let toks = lex_ok("in-outx");
        assert!(matches!(&toks[0], Token::Ident(s) if s == "in-outx"));
    }

    #[test]
    fn kebab_case_ident() {
        // M3-Phase 3 attribute names (`item-cross-size`, `item-spacing`,
        // `line-spacing`) are kebab-case Idents per dsl_spec §4.10's
        // attribute table. The lexer admits arbitrary kebab segments; the
        // parser's IDENT-keyed PropertyBind shape is unchanged.
        let toks = lex_ok("item-cross-size: 88");
        assert!(matches!(&toks[0], Token::Ident(s) if s == "item-cross-size"));
        assert!(matches!(&toks[1], Token::Colon));
        assert!(matches!(&toks[2], Token::IntLit(88)));
    }

    #[test]
    fn kebab_ident_breaks_on_non_alpha_after_hyphen() {
        // `count-=1`: after `count`, `-` is followed by `=` (not
        // alphabetic), so the kebab continuation rule does not fire and
        // the lexer falls through to `MinusEq`.
        let toks = lex_ok("count-=1");
        assert!(matches!(&toks[0], Token::Ident(s) if s == "count"));
        assert!(matches!(&toks[1], Token::MinusEq));
        assert!(matches!(&toks[2], Token::IntLit(1)));
    }

    #[test]
    fn negative_int_literal() {
        let toks = lex_ok("-1 -42");
        assert!(matches!(&toks[0], Token::IntLit(-1)));
        assert!(matches!(&toks[1], Token::IntLit(-42)));
    }

    #[test]
    fn negative_float_literal_rejected() {
        // Spec §5 AST table: FloatLit surface is unsigned. The M3-Phase 3
        // negative-entry path is integer-only; a fractional continuation
        // is rejected at lex time so the lexer's surface widening stays
        // scoped to the WrapPanel attribute use case.
        assert!(lex_err("-1.5").contains("negative float"));
    }

    #[test]
    fn negative_measurement_literal_rejected() {
        // Measurements (`12px`) are unsigned per spec. The negative-entry
        // path rejects `-12px` for the same scope-control reason as
        // negative floats.
        assert!(lex_err("-12px").contains("negative measurement"));
    }

    #[test]
    fn negative_int_in_property_bind_position() {
        // The check-layer reject for `item-spacing: -1` needs the lexer to
        // produce `IntLit(-1)` so the diagnostic reaches the AST. Verify
        // the four-token shape end-to-end.
        let toks = lex_ok("item-spacing: -1");
        assert!(matches!(&toks[0], Token::Ident(s) if s == "item-spacing"));
        assert!(matches!(&toks[1], Token::Colon));
        assert!(matches!(&toks[2], Token::IntLit(-1)));
    }

    #[test]
    fn identifiers() {
        let toks = lex_ok("Counter _foo abc123");
        assert!(matches!(&toks[0], Token::Ident(s) if s == "Counter"));
        assert!(matches!(&toks[1], Token::Ident(s) if s == "_foo"));
        assert!(matches!(&toks[2], Token::Ident(s) if s == "abc123"));
    }

    #[test]
    fn bool_keywords() {
        let toks = lex_ok("true false");
        assert!(matches!(&toks[0], Token::Kw(Keyword::True)));
        assert!(matches!(&toks[1], Token::Kw(Keyword::False)));
    }

    #[test]
    fn bool_keywords_not_treated_as_idents() {
        // `true` / `false` must not lex as Token::Ident.
        let toks = lex_ok("true");
        assert!(
            !matches!(&toks[0], Token::Ident(_)),
            "`true` lexed as identifier: {:?}",
            &toks[0]
        );
        let toks = lex_ok("false");
        assert!(
            !matches!(&toks[0], Token::Ident(_)),
            "`false` lexed as identifier: {:?}",
            &toks[0]
        );
    }

    #[test]
    fn bool_keyword_word_boundary() {
        // Identifiers that merely start with `true`/`false` must not be split.
        let toks = lex_ok("trueish falsey true_");
        assert!(matches!(&toks[0], Token::Ident(s) if s == "trueish"));
        assert!(matches!(&toks[1], Token::Ident(s) if s == "falsey"));
        assert!(matches!(&toks[2], Token::Ident(s) if s == "true_"));
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
    fn ratio_literal_basic() {
        let toks = lex_ok("16:9");
        assert!(matches!(&toks[0], Token::RatioLit(16, 9)));
        assert!(matches!(&toks[1], Token::Eof));
    }

    #[test]
    fn ratio_literal_one_to_one() {
        let toks = lex_ok("1:1");
        assert!(matches!(&toks[0], Token::RatioLit(1, 1)));
    }

    #[test]
    fn ratio_literal_in_property_bind_position() {
        // Two `:` tokens: one for the property-bind, one folded into the
        // ratio. The lexer must split them correctly.
        let toks = lex_ok("aspect: 16:9");
        assert!(matches!(&toks[0], Token::Ident(s) if s == "aspect"));
        assert!(matches!(&toks[1], Token::Colon));
        assert!(matches!(&toks[2], Token::RatioLit(16, 9)));
    }

    #[test]
    fn integer_followed_by_colon_then_non_digit_is_not_ratio() {
        // `state count: i32 = 16` end-of-expr followed by `: i32` of a next
        // hypothetical token sequence with whitespace must not absorb the
        // colon — verify on `16: x` shape.
        let toks = lex_ok("16: x");
        assert!(matches!(&toks[0], Token::IntLit(16)));
        assert!(matches!(&toks[1], Token::Colon));
        assert!(matches!(&toks[2], Token::Ident(s) if s == "x"));
    }

    #[test]
    fn integer_with_whitespace_before_colon_is_not_ratio() {
        // Whitespace between the integer and `:` defeats the ratio token.
        let toks = lex_ok("16 :9");
        assert!(matches!(&toks[0], Token::IntLit(16)));
        assert!(matches!(&toks[1], Token::Colon));
        assert!(matches!(&toks[2], Token::IntLit(9)));
    }

    #[test]
    fn float_followed_by_colon_is_not_ratio() {
        // `1.5:9` — the lexer must not fold a float into a RatioLit.
        let toks = lex_ok("1.5:9");
        assert!(matches!(&toks[0], Token::FloatLit(v) if *v == 1.5_f64));
        assert!(matches!(&toks[1], Token::Colon));
        assert!(matches!(&toks[2], Token::IntLit(9)));
    }

    #[test]
    fn measurement_not_disturbed_by_ratio_lookahead() {
        let toks = lex_ok("12px");
        assert!(matches!(&toks[0], Token::Measurement(v, Unit::Px) if *v == 12.0));
    }

    #[test]
    fn ratio_zero_sides_lex_ok_check_rejects_later() {
        // Lexer accepts `0:0` syntactically; value validity (positive
        // integers) is enforced at `wasamoc check` time per dsl_spec §4.9.
        let toks = lex_ok("0:0");
        assert!(matches!(&toks[0], Token::RatioLit(0, 0)));
    }

    #[test]
    fn color_literal_six_digit_packs_with_full_alpha() {
        // `#cccccc` → 0xFFcccccc (alpha in MSB defaults to 0xFF).
        let toks = lex_ok("#cccccc");
        assert!(matches!(&toks[0], Token::ColorLit(0xFFCC_CCCC)));
    }

    #[test]
    fn color_literal_eight_digit_explicit_alpha() {
        // `#00000080` → AA=80, RR=00, GG=00, BB=00 → 0x80000000 (scrim).
        let toks = lex_ok("#00000080");
        assert!(matches!(&toks[0], Token::ColorLit(0x8000_0000)));
    }

    #[test]
    fn color_literal_eight_digit_mixed_channels() {
        // `#11223344` → RR=11, GG=22, BB=33, AA=44 → 0x44112233.
        let toks = lex_ok("#11223344");
        assert!(matches!(&toks[0], Token::ColorLit(0x4411_2233)));
    }

    #[test]
    fn color_literal_uppercase_hex_accepted() {
        let toks = lex_ok("#ABCDEF");
        assert!(matches!(&toks[0], Token::ColorLit(0xFFAB_CDEF)));
    }

    #[test]
    fn color_literal_three_hex_rejected() {
        // Short-form `#ccc` is not in the grammar; reject.
        assert!(lex_err("#ccc").contains("6 or 8 hex digits"));
    }

    #[test]
    fn color_literal_seven_hex_rejected() {
        assert!(lex_err("#1234567").contains("6 or 8 hex digits"));
    }

    #[test]
    fn color_literal_no_hex_rejected() {
        assert!(lex_err("#xyz").contains("6 or 8 hex digits"));
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
