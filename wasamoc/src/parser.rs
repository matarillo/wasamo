use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::lexer::{Keyword, SpannedToken, Token};

pub fn parse(tokens: &[SpannedToken], filename: &str) -> Result<ComponentDef, Diagnostic> {
    let mut p = Parser {
        tokens,
        pos: 0,
        filename,
    };
    let def = p.parse_component_def()?;
    if !matches!(p.peek(), Token::Eof) {
        return Err(p.error(format!(
            "expected end of file, found {}",
            p.peek().description()
        )));
    }
    Ok(def)
}

struct Parser<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
    filename: &'a str,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    fn peek_next(&self) -> &Token {
        self.tokens
            .get(self.pos + 1)
            .map(|t| &t.token)
            .unwrap_or(&Token::Eof)
    }

    fn current_span(&self) -> &Span {
        &self.tokens[self.pos].span
    }

    fn advance(&mut self) -> SpannedToken {
        let tok = self.tokens[self.pos].clone();
        if !matches!(tok.token, Token::Eof) {
            self.pos += 1;
        }
        tok
    }

    fn error(&self, msg: impl Into<String>) -> Diagnostic {
        let sp = self.current_span();
        Diagnostic::error(self.filename, sp.line, sp.col, msg)
    }

    fn expect_kw(&mut self, kw: Keyword) -> Result<SpannedToken, Diagnostic> {
        let found_desc = self.peek().description();
        let expected_desc = kw.description();
        if self.peek().is_kw(&kw) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected {}, found {}", expected_desc, found_desc)))
        }
    }

    fn expect_lbrace(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::LBrace) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `{{`, found {}", desc)))
        }
    }

    fn expect_rbrace(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::RBrace) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `}}`, found {}", desc)))
        }
    }

    fn expect_colon(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::Colon) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `:`, found {}", desc)))
        }
    }

    fn expect_langle(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::LAngle) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `<`, found {}", desc)))
        }
    }

    fn expect_rangle(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::RAngle) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `>`, found {}", desc)))
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span), Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::Ident(_)) {
            let tok = self.advance();
            if let Token::Ident(s) = tok.token {
                return Ok((s, tok.span));
            }
            unreachable!()
        } else {
            Err(self.error(format!("expected identifier, found {}", desc)))
        }
    }

    fn parse_component_def(&mut self) -> Result<ComponentDef, Diagnostic> {
        let start = self.current_span().clone();
        self.expect_kw(Keyword::Component)?;
        let (name, _) = self.expect_ident()?;
        self.expect_kw(Keyword::Inherits)?;
        let (base, _) = self.expect_ident()?;
        self.expect_lbrace()?;

        let mut members = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            members.push(self.parse_member()?);
        }

        let end_tok = self.expect_rbrace()?;
        Ok(ComponentDef {
            name,
            base,
            members,
            span: Span {
                start: start.start,
                end: end_tok.span.end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_member(&mut self) -> Result<Member, Diagnostic> {
        if self.peek().is_kw(&Keyword::InOut) {
            return self.parse_property_decl();
        }

        if self.peek().is_kw(&Keyword::State) {
            return self.parse_state_member();
        }

        if matches!(self.peek(), Token::Ident(_)) {
            let next_colon = matches!(self.peek_next(), Token::Colon);
            let next_lbrace = matches!(self.peek_next(), Token::LBrace);
            let next_arrow = matches!(self.peek_next(), Token::Arrow);

            return if next_colon {
                self.parse_property_bind()
            } else if next_lbrace {
                self.parse_widget_decl()
            } else if next_arrow {
                self.parse_signal_handler()
            } else {
                let next_desc = self.peek_next().description();
                Err(self.error(format!("unexpected token {} after identifier", next_desc)))
            };
        }

        let desc = self.peek().description();
        Err(self.error(format!("expected member, found {}", desc)))
    }

    fn parse_state_member(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        self.expect_kw(Keyword::State)?;
        let (name, _) = self.expect_ident()?;
        self.expect_colon()?;
        let ty = self.parse_type_name()?;

        let eq_desc = self.peek().description();
        if !matches!(self.peek(), Token::Eq) {
            return Err(self.error(format!("expected `=`, found {}", eq_desc)));
        }
        self.advance();

        let default = self.parse_expr()?;
        let end = default.span().end;
        Ok(Member::StateMember {
            name,
            ty,
            default,
            span: Span {
                start: start.start,
                end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_property_decl(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        self.expect_kw(Keyword::InOut)?;
        self.expect_kw(Keyword::Property)?;
        self.expect_langle()?;
        let ty = self.parse_type_name()?;
        self.expect_rangle()?;
        let (name, _) = self.expect_ident()?;
        self.expect_colon()?;
        let default = self.parse_expr()?;
        let end = default.span().end;
        Ok(Member::PropertyDecl {
            name,
            ty,
            default,
            span: Span {
                start: start.start,
                end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_property_bind(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        let (name, _) = self.expect_ident()?;
        self.expect_colon()?;
        let value = self.parse_expr()?;
        let end = value.span().end;
        Ok(Member::PropertyBind {
            name,
            value,
            span: Span {
                start: start.start,
                end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_widget_decl(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        let (type_name, _) = self.expect_ident()?;
        self.expect_lbrace()?;

        let mut members = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            members.push(self.parse_member()?);
        }

        let end_tok = self.expect_rbrace()?;
        Ok(Member::WidgetDecl {
            type_name,
            members,
            span: Span {
                start: start.start,
                end: end_tok.span.end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_signal_handler(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        let (signal, _) = self.expect_ident()?;

        let arrow_desc = self.peek().description();
        if !matches!(self.peek(), Token::Arrow) {
            return Err(self.error(format!("expected `=>`, found {}", arrow_desc)));
        }
        self.advance();

        let body = self.parse_block()?;
        let end = body.span.end;
        Ok(Member::SignalHandler {
            signal,
            body,
            span: Span {
                start: start.start,
                end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_block(&mut self) -> Result<Block, Diagnostic> {
        let start = self.current_span().clone();
        self.expect_lbrace()?;

        let mut statements = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            statements.push(self.parse_statement()?);
        }

        let end_tok = self.expect_rbrace()?;
        Ok(Block {
            statements,
            span: Span {
                start: start.start,
                end: end_tok.span.end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, Diagnostic> {
        let start = self.current_span().clone();
        let target = self.parse_qualified_name()?;
        let op = self.parse_assign_op()?;
        let value = self.parse_expr()?;

        let semi_desc = self.peek().description();
        if !matches!(self.peek(), Token::Semicolon) {
            return Err(self.error(format!("expected `;`, found {}", semi_desc)));
        }
        let semi = self.advance();
        Ok(Statement {
            target,
            op,
            value,
            span: Span {
                start: start.start,
                end: semi.span.end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_assign_op(&mut self) -> Result<AssignOp, Diagnostic> {
        let desc = self.peek().description();
        let op = match self.peek() {
            Token::Eq => AssignOp::Eq,
            Token::PlusEq => AssignOp::PlusEq,
            Token::MinusEq => AssignOp::MinusEq,
            Token::StarEq => AssignOp::MulEq,
            Token::SlashEq => AssignOp::DivEq,
            _ => return Err(self.error(format!("expected assignment operator, found {}", desc))),
        };
        self.advance();
        Ok(op)
    }

    fn parse_qualified_name(&mut self) -> Result<QualifiedName, Diagnostic> {
        let start = self.current_span().clone();
        let (first, first_span) = self.expect_ident()?;
        let mut segments = vec![first];
        let mut last_end = first_span.end;

        while matches!(self.peek(), Token::Dot) {
            self.advance();
            let (seg, seg_span) = self.expect_ident()?;
            last_end = seg_span.end;
            segments.push(seg);
        }

        Ok(QualifiedName {
            segments,
            span: Span {
                start: start.start,
                end: last_end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        let is_valid = matches!(
            self.peek(),
            Token::StringLit(_)
                | Token::IntLit(_)
                | Token::FloatLit(_)
                | Token::Measurement(_, _)
                | Token::Ident(_)
                | Token::Kw(Keyword::True)
                | Token::Kw(Keyword::False)
                | Token::RatioLit(_, _)
                | Token::ColorLit(_)
        );
        if !is_valid {
            let desc = self.peek().description();
            return Err(self.error(format!("expected expression, found {}", desc)));
        }
        let tok = self.advance();
        match tok.token {
            Token::StringLit(parts) => Ok(Expr::StringLit {
                parts,
                span: tok.span,
            }),
            Token::IntLit(v) => Ok(Expr::IntLit {
                value: v,
                span: tok.span,
            }),
            Token::FloatLit(v) => Ok(Expr::FloatLit {
                value: v,
                span: tok.span,
            }),
            Token::Measurement(v, u) => Ok(Expr::Measurement {
                value: v,
                unit: u,
                span: tok.span,
            }),
            Token::Ident(name) => Ok(Expr::Ident {
                name,
                span: tok.span,
            }),
            Token::Kw(Keyword::True) => Ok(Expr::BoolLit {
                value: true,
                span: tok.span,
            }),
            Token::Kw(Keyword::False) => Ok(Expr::BoolLit {
                value: false,
                span: tok.span,
            }),
            Token::RatioLit(num, den) => Ok(Expr::RatioLit {
                num,
                den,
                span: tok.span,
            }),
            Token::ColorLit(value) => Ok(Expr::ColorLit {
                value,
                span: tok.span,
            }),
            _ => unreachable!(),
        }
    }

    fn parse_type_name(&mut self) -> Result<TypeName, Diagnostic> {
        let ident = self.peek().as_ident().map(|s| s.to_string());
        match ident.as_deref() {
            Some("int") | Some("i32") => {
                self.advance();
                Ok(TypeName::Int)
            }
            Some("string") => {
                self.advance();
                Ok(TypeName::Str)
            }
            Some("float") => {
                self.advance();
                Ok(TypeName::Float)
            }
            Some("bool") => {
                self.advance();
                Ok(TypeName::Bool)
            }
            Some(other) => {
                let msg = format!("unknown type `{}`; expected i32, string, or bool", other);
                Err(self.error(msg))
            }
            None => {
                let desc = self.peek().description();
                Err(self.error(format!("expected type name, found {}", desc)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::ast::*;
    use crate::diagnostic::Diagnostic;
    use crate::lexer::tokenize;

    fn parse_str(src: &str) -> Result<ComponentDef, Diagnostic> {
        let tokens = tokenize(src, "<test>").unwrap();
        parse(&tokens, "<test>")
    }

    fn parse_ok(src: &str) -> ComponentDef {
        parse_str(src).expect("parse failed")
    }

    fn parse_err_msg(src: &str) -> String {
        parse_str(src).expect_err("expected parse error").message
    }

    #[test]
    fn state_decl_i32() {
        let def = parse_ok("component C inherits W { state count: i32 = 0 }");
        assert_eq!(def.members.len(), 1);
        if let Member::StateMember {
            name, ty, default, ..
        } = &def.members[0]
        {
            assert_eq!(name, "count");
            assert!(matches!(ty, TypeName::Int));
            assert!(matches!(default, Expr::IntLit { value: 0, .. }));
        } else {
            panic!("expected StateMember");
        }
    }

    #[test]
    fn state_decl_bool_false() {
        let def = parse_ok("component C inherits W { state ready: bool = false }");
        assert_eq!(def.members.len(), 1);
        if let Member::StateMember {
            name, ty, default, ..
        } = &def.members[0]
        {
            assert_eq!(name, "ready");
            assert!(matches!(ty, TypeName::Bool));
            assert!(matches!(default, Expr::BoolLit { value: false, .. }));
        } else {
            panic!("expected StateMember");
        }
    }

    #[test]
    fn state_decl_bool_true() {
        let def = parse_ok("component C inherits W { state ready: bool = true }");
        if let Member::StateMember { default, .. } = &def.members[0] {
            assert!(matches!(default, Expr::BoolLit { value: true, .. }));
        } else {
            panic!("expected StateMember");
        }
    }

    #[test]
    fn property_bind_bool_literal() {
        let def = parse_ok("component C inherits W { Button { enabled: true } }");
        if let Member::WidgetDecl { members, .. } = &def.members[0] {
            if let Member::PropertyBind { name, value, .. } = &members[0] {
                assert_eq!(name, "enabled");
                assert!(matches!(value, Expr::BoolLit { value: true, .. }));
            } else {
                panic!("expected PropertyBind");
            }
        } else {
            panic!("expected WidgetDecl");
        }
    }

    #[test]
    fn true_rejected_as_state_name() {
        // `state true: bool = false` — `true` is reserved, cannot be used as an identifier.
        let msg = parse_err_msg("component C inherits W { state true: bool = false }");
        assert!(
            msg.contains("expected identifier") && msg.contains("`true`"),
            "message: {msg}"
        );
    }

    #[test]
    fn false_rejected_as_state_name() {
        let msg = parse_err_msg("component C inherits W { state false: bool = true }");
        assert!(
            msg.contains("expected identifier") && msg.contains("`false`"),
            "message: {msg}"
        );
    }

    #[test]
    fn true_rejected_as_widget_property_name() {
        // Property-bind LHS is also an identifier; `true: …` must not parse.
        let msg = parse_err_msg("component C inherits W { Button { true: false } }");
        assert!(
            msg.contains("expected identifier") || msg.contains("expected member"),
            "message: {msg}"
        );
    }

    #[test]
    fn state_decl_string() {
        let def = parse_ok(r#"component C inherits W { state label: string = "hello" }"#);
        assert_eq!(def.members.len(), 1);
        if let Member::StateMember { name, ty, .. } = &def.members[0] {
            assert_eq!(name, "label");
            assert!(matches!(ty, TypeName::Str));
        } else {
            panic!("expected StateMember");
        }
    }

    #[test]
    fn state_decl_before_widget() {
        let def = parse_ok("component C inherits W { state count: i32 = 0 VStack {} }");
        assert_eq!(def.members.len(), 2);
        assert!(matches!(&def.members[0], Member::StateMember { name, .. } if name == "count"));
        assert!(
            matches!(&def.members[1], Member::WidgetDecl { type_name, .. } if type_name == "VStack")
        );
    }

    #[test]
    fn empty_component() {
        let def = parse_ok("component Foo inherits Bar {}");
        assert_eq!(def.name, "Foo");
        assert_eq!(def.base, "Bar");
        assert!(def.members.is_empty());
    }

    #[test]
    fn property_decl_int() {
        let def = parse_ok("component C inherits W { in-out property <int> count: 0 }");
        assert_eq!(def.members.len(), 1);
        if let Member::PropertyDecl {
            name, ty, default, ..
        } = &def.members[0]
        {
            assert_eq!(name, "count");
            assert!(matches!(ty, TypeName::Int));
            assert!(matches!(default, Expr::IntLit { value: 0, .. }));
        } else {
            panic!("expected PropertyDecl");
        }
    }

    #[test]
    fn property_decl_string() {
        let def = parse_ok(r#"component C inherits W { in-out property <string> title: "hello" }"#);
        if let Member::PropertyDecl {
            name, ty, default, ..
        } = &def.members[0]
        {
            assert_eq!(name, "title");
            assert!(matches!(ty, TypeName::Str));
            assert!(matches!(default, Expr::StringLit { .. }));
        } else {
            panic!("expected PropertyDecl");
        }
    }

    #[test]
    fn property_bind_string() {
        let def = parse_ok(r#"component C inherits W { title: "Counter" }"#);
        if let Member::PropertyBind { name, value, .. } = &def.members[0] {
            assert_eq!(name, "title");
            assert!(matches!(value, Expr::StringLit { .. }));
        } else {
            panic!("expected PropertyBind");
        }
    }

    #[test]
    fn property_bind_ident() {
        let def = parse_ok("component C inherits W { theme: system }");
        if let Member::PropertyBind { name, value, .. } = &def.members[0] {
            assert_eq!(name, "theme");
            assert!(matches!(value, Expr::Ident { name: n, .. } if n == "system"));
        } else {
            panic!("expected PropertyBind");
        }
    }

    #[test]
    fn property_bind_measurement() {
        let def = parse_ok("component C inherits W { spacing: 12px }");
        if let Member::PropertyBind { name, value, .. } = &def.members[0] {
            assert_eq!(name, "spacing");
            assert!(
                matches!(value, Expr::Measurement { value: v, unit: Unit::Px, .. } if *v == 12.0)
            );
        } else {
            panic!("expected PropertyBind");
        }
    }

    #[test]
    fn widget_decl_empty() {
        let def = parse_ok("component C inherits W { VStack {} }");
        if let Member::WidgetDecl {
            type_name, members, ..
        } = &def.members[0]
        {
            assert_eq!(type_name, "VStack");
            assert!(members.is_empty());
        } else {
            panic!("expected WidgetDecl");
        }
    }

    #[test]
    fn widget_decl_with_property() {
        let def = parse_ok("component C inherits W { VStack { spacing: 12px } }");
        if let Member::WidgetDecl {
            type_name, members, ..
        } = &def.members[0]
        {
            assert_eq!(type_name, "VStack");
            assert_eq!(members.len(), 1);
            assert!(matches!(&members[0], Member::PropertyBind { name, .. } if name == "spacing"));
        } else {
            panic!("expected WidgetDecl");
        }
    }

    #[test]
    fn signal_handler_plus_eq() {
        let def = parse_ok("component C inherits W { clicked => { root.count += 1; } }");
        if let Member::SignalHandler { signal, body, .. } = &def.members[0] {
            assert_eq!(signal, "clicked");
            assert_eq!(body.statements.len(), 1);
            let stmt = &body.statements[0];
            assert_eq!(stmt.target.segments, vec!["root", "count"]);
            assert!(matches!(stmt.op, AssignOp::PlusEq));
            assert!(matches!(stmt.value, Expr::IntLit { value: 1, .. }));
        } else {
            panic!("expected SignalHandler");
        }
    }

    #[test]
    fn nested_widgets() {
        let def = parse_ok("component C inherits W { VStack { Text {} Button {} } }");
        if let Member::WidgetDecl {
            type_name, members, ..
        } = &def.members[0]
        {
            assert_eq!(type_name, "VStack");
            assert_eq!(members.len(), 2);
            assert!(
                matches!(&members[0], Member::WidgetDecl { type_name, .. } if type_name == "Text")
            );
            assert!(
                matches!(&members[1], Member::WidgetDecl { type_name, .. } if type_name == "Button")
            );
        } else {
            panic!("expected VStack WidgetDecl");
        }
    }

    #[test]
    fn full_counter_component() {
        let src = r#"component Counter inherits Window {
    title: "Counter"
    in-out property <int> count: 0
    VStack {
        spacing: 12px
        Text {
            text: "Count: \{root.count}"
        }
        Button {
            text: "Increment"
            clicked => { root.count += 1; }
        }
    }
}"#;
        let def = parse_ok(src);
        assert_eq!(def.name, "Counter");
        assert_eq!(def.base, "Window");
        assert_eq!(def.members.len(), 3);
        assert!(matches!(&def.members[0], Member::PropertyBind { name, .. } if name == "title"));
        assert!(matches!(&def.members[1], Member::PropertyDecl { name, .. } if name == "count"));
        assert!(
            matches!(&def.members[2], Member::WidgetDecl { type_name, .. } if type_name == "VStack")
        );
    }

    #[test]
    fn property_bind_ratio_literal() {
        // `Box { aspect: 16:9 }` — DD-M3-P2-002 surface form.
        let def = parse_ok("component C inherits W { Box { aspect: 16:9 } }");
        if let Member::WidgetDecl { members, .. } = &def.members[0] {
            if let Member::PropertyBind { name, value, .. } = &members[0] {
                assert_eq!(name, "aspect");
                assert!(matches!(
                    value,
                    Expr::RatioLit {
                        num: 16,
                        den: 9,
                        ..
                    }
                ));
            } else {
                panic!("expected PropertyBind");
            }
        } else {
            panic!("expected WidgetDecl");
        }
    }

    #[test]
    fn property_bind_color_literal_six_hex() {
        // `Box { fill: #cccccc }` — DD-M3-P2-003 surface form, alpha=0xFF.
        let def = parse_ok("component C inherits W { Box { fill: #cccccc } }");
        if let Member::WidgetDecl { members, .. } = &def.members[0] {
            if let Member::PropertyBind { name, value, .. } = &members[0] {
                assert_eq!(name, "fill");
                assert!(matches!(
                    value,
                    Expr::ColorLit {
                        value: 0xFFCC_CCCC,
                        ..
                    }
                ));
            } else {
                panic!("expected PropertyBind");
            }
        } else {
            panic!("expected WidgetDecl");
        }
    }

    #[test]
    fn property_bind_color_literal_eight_hex() {
        // Scrim: `#00000080` packs to 0x80000000.
        let def = parse_ok("component C inherits W { Box { fill: #00000080 } }");
        if let Member::WidgetDecl { members, .. } = &def.members[0] {
            if let Member::PropertyBind { value, .. } = &members[0] {
                assert!(matches!(
                    value,
                    Expr::ColorLit {
                        value: 0x8000_0000,
                        ..
                    }
                ));
            } else {
                panic!("expected PropertyBind");
            }
        } else {
            panic!("expected WidgetDecl");
        }
    }

    #[test]
    fn box_image_placeholder_shape() {
        // dsl_spec §4.9 normative shape:
        //   Box { aspect: 1:1; fill: #cccccc; Text { text: "Photo 12" } }
        // Whitespace (newlines) substitute for `;` separators between
        // members — the existing grammar has no statement terminator at
        // member level, mirroring the M1 / M2 surface.
        let src = r#"component C inherits W {
            Box {
                aspect: 1:1
                fill: #cccccc
                Text { text: "Photo 12" }
            }
        }"#;
        let def = parse_ok(src);
        if let Member::WidgetDecl {
            type_name, members, ..
        } = &def.members[0]
        {
            assert_eq!(type_name, "Box");
            assert_eq!(members.len(), 3);
            assert!(matches!(
                &members[0],
                Member::PropertyBind {
                    name,
                    value: Expr::RatioLit { num: 1, den: 1, .. },
                    ..
                } if name == "aspect"
            ));
            assert!(matches!(
                &members[1],
                Member::PropertyBind {
                    name,
                    value: Expr::ColorLit {
                        value: 0xFFCC_CCCC,
                        ..
                    },
                    ..
                } if name == "fill"
            ));
            assert!(matches!(
                &members[2],
                Member::WidgetDecl { type_name, .. } if type_name == "Text"
            ));
        } else {
            panic!("expected WidgetDecl for Box");
        }
    }

    #[test]
    fn error_missing_inherits() {
        let msg = parse_err_msg("component Foo Bar {}");
        assert!(msg.contains("`inherits`"), "message: {msg}");
    }

    #[test]
    fn error_missing_lbrace() {
        let msg = parse_err_msg("component Foo inherits Bar");
        assert!(
            msg.contains("`{`") || msg.contains("end of file"),
            "message: {msg}"
        );
    }

    #[test]
    fn error_trailing_tokens() {
        let msg = parse_err_msg("component Foo inherits Bar {} extra");
        assert!(msg.contains("end of file"), "message: {msg}");
    }
}
