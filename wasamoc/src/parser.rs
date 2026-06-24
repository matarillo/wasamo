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

    fn peek_n(&self, n: usize) -> &Token {
        self.tokens
            .get(self.pos + n)
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

    fn expect_lparen(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::LParen) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `(`, found {}", desc)))
        }
    }

    fn expect_rparen(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::RParen) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `)`, found {}", desc)))
        }
    }

    fn expect_lbracket(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::LBracket) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `[`, found {}", desc)))
        }
    }

    fn expect_rbracket(&mut self) -> Result<SpannedToken, Diagnostic> {
        let desc = self.peek().description();
        if matches!(self.peek(), Token::RBracket) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("expected `]`, found {}", desc)))
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

        if self.peek().is_kw(&Keyword::If) {
            return self.parse_conditional_member();
        }

        if self.peek().is_kw(&Keyword::For) {
            return self.parse_for_member();
        }

        if self.peek().is_kw(&Keyword::Else)
            || self.peek().is_kw(&Keyword::Switch)
            || self.peek().is_kw(&Keyword::In)
        {
            let desc = self.peek().description();
            return Err(self.error(format!(
                "{} is reserved for the structural control-flow family but is not yet supported as a member",
                desc
            )));
        }

        if matches!(self.peek(), Token::Ident(_)) {
            if matches!(self.peek(), Token::Ident(name) if name == "slot") {
                if matches!(self.peek_next(), Token::Dot) {
                    return self.parse_slot_property_bind();
                }
                if matches!(self.peek_next(), Token::Colon) {
                    return Err(self.error("malformed slot property key; expected `slot.<key>:`"));
                }
            }
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

    fn parse_conditional_member(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        self.expect_kw(Keyword::If)?;
        let condition = self.parse_condition_expr()?;
        self.expect_lbrace()?;

        let mut body = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            body.push(self.parse_member()?);
        }

        let end_tok = self.expect_rbrace()?;
        Ok(Member::Conditional {
            condition,
            body,
            span: Span {
                start: start.start,
                end: end_tok.span.end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_for_member(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        self.expect_kw(Keyword::For)?;
        let (binder, _) = self.expect_ident()?;
        let index_binder = if matches!(self.peek(), Token::Comma) {
            self.advance();
            let (index, _) = self.expect_ident()?;
            Some(index)
        } else {
            None
        };
        self.expect_kw(Keyword::In)?;
        let collection = self.parse_expr()?;
        self.expect_lbrace()?;

        let mut body = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            body.push(self.parse_member()?);
        }

        let end_tok = self.expect_rbrace()?;
        Ok(Member::For {
            binder,
            index_binder,
            collection,
            body,
            span: Span {
                start: start.start,
                end: end_tok.span.end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_condition_expr(&mut self) -> Result<Expr, Diagnostic> {
        if matches!(self.peek(), Token::Bang) {
            let tok = self.advance();
            if matches!(
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
            ) {
                let _ = self.parse_expr()?;
            }
            return Ok(Expr::UnsupportedOperator {
                op: tok.token.description().to_string(),
                span: tok.span,
            });
        }
        self.parse_expr()
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

    fn parse_slot_property_bind(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        let (prefix, _) = self.expect_ident()?;
        debug_assert_eq!(prefix, "slot");
        if !matches!(self.peek(), Token::Dot) {
            return Err(self.error("malformed slot property key; expected `slot.<key>:`"));
        }
        self.advance();
        let (key, _) = self.expect_ident()?;
        self.expect_colon()?;
        let value = self.parse_expr()?;
        let end = value.span().end;
        Ok(Member::PropertyBind {
            name: format!("slot.{key}"),
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

        // Widget-type context routing (R-A spike Decision 1): only inside a
        // `Grid` body do `columns:` / `rows:` route to the narrow
        // track-list parser path (DD-M3-P5-002). Every other member — and
        // every member of every non-Grid widget — stays on `parse_member`,
        // so `parse_expr` / `parse_property_bind` are untouched and no
        // general list grammar is opened.
        let is_grid = type_name == "Grid";

        let mut members = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            if is_grid && self.at_grid_track_attr() {
                members.push(self.parse_grid_track_list()?);
            } else {
                members.push(self.parse_member()?);
            }
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

    /// True when the upcoming tokens are a Grid track-list attribute
    /// (`columns:` or `rows:`). Used only inside a Grid body.
    fn at_grid_track_attr(&self) -> bool {
        matches!(self.peek(), Token::Ident(name) if name == "columns" || name == "rows")
            && matches!(self.peek_next(), Token::Colon)
    }

    /// Parse a Grid `columns:` / `rows:` track list (DD-M3-P5-002). A
    /// track list is a whitespace-separated sequence of track-size tokens
    /// terminated by the next member or the closing `}`. The parser is
    /// permissive about value validity (range / `auto` / float) so the
    /// check layer can name each problem precisely; here it only assembles
    /// the typed `TrackSize` sequence and requires it be non-empty.
    fn parse_grid_track_list(&mut self) -> Result<Member, Diagnostic> {
        let start = self.current_span().clone();
        let (name, _) = self.expect_ident()?;
        let axis = match name.as_str() {
            "columns" => TrackAxis::Columns,
            "rows" => TrackAxis::Rows,
            // `at_grid_track_attr` gates this call to exactly these names.
            _ => unreachable!("parse_grid_track_list on non-track attribute"),
        };
        self.expect_colon()?;

        let mut tracks = Vec::new();
        let mut end = self.current_span().start;
        loop {
            match self.peek() {
                Token::IntLit(v) => {
                    let value = *v;
                    let tok = self.advance();
                    if self.star_adjacent_to(tok.span.end) {
                        // `n*` — adjacent star marks a weighted-star track.
                        let star = self.advance();
                        end = star.span.end;
                        tracks.push(TrackSize::Star {
                            weight: value,
                            span: Span {
                                start: tok.span.start,
                                end: star.span.end,
                                line: tok.span.line,
                                col: tok.span.col,
                            },
                        });
                    } else {
                        end = tok.span.end;
                        tracks.push(TrackSize::Fixed {
                            value,
                            span: tok.span,
                        });
                    }
                }
                Token::FloatLit(_) => {
                    let tok = self.advance();
                    // `1.5*` — consume an adjacent star so the float forms
                    // a single (invalid) track element rather than leaving
                    // a spurious unit-star behind.
                    let elem_end = if self.star_adjacent_to(tok.span.end) {
                        self.advance().span.end
                    } else {
                        tok.span.end
                    };
                    end = elem_end;
                    tracks.push(TrackSize::InvalidFloat {
                        span: Span {
                            start: tok.span.start,
                            end: elem_end,
                            line: tok.span.line,
                            col: tok.span.col,
                        },
                    });
                }
                Token::Star => {
                    // Standalone `*` — unit star (`weight = 1`).
                    let tok = self.advance();
                    end = tok.span.end;
                    tracks.push(TrackSize::Star {
                        weight: 1,
                        span: tok.span,
                    });
                }
                // A bare word in track position (`auto`, or any other
                // ident) that does NOT begin a new member. An ident
                // followed by `:` / `{` / `=>` is the next member and
                // terminates the track list.
                Token::Ident(_)
                    if !matches!(
                        self.peek_next(),
                        Token::Colon | Token::LBrace | Token::Arrow
                    ) =>
                {
                    let tok = self.advance();
                    if let Token::Ident(name) = tok.token {
                        end = tok.span.end;
                        tracks.push(TrackSize::Word {
                            name,
                            span: tok.span,
                        });
                    } else {
                        unreachable!()
                    }
                }
                // Anything else (next member start, `}`, EOF, …) ends the
                // track list.
                _ => break,
            }
        }

        if tracks.is_empty() {
            return Err(self.error(format!(
                "expected at least one track size after `{}:`; a Grid track list is a whitespace-separated sequence of integer (fixed px) or `n*` (weighted star) tokens",
                axis.attr_name()
            )));
        }

        Ok(Member::GridTracks {
            axis,
            tracks,
            span: Span {
                start: start.start,
                end,
                line: start.line,
                col: start.col,
            },
        })
    }

    /// True when the current token is a `Star` immediately adjacent
    /// (no intervening whitespace) to the byte offset `prev_end`. This
    /// is the R-A spike Decision 3 mechanism distinguishing `1*` (one
    /// weighted-star track) from `1 *` (a fixed track and a unit-star
    /// track) — the same span-adjacency rule the lexer uses for ratios.
    fn star_adjacent_to(&self, prev_end: usize) -> bool {
        matches!(self.peek(), Token::Star) && self.current_span().start == prev_end
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

    fn parse_statement(&mut self) -> Result<BlockStatement, Diagnostic> {
        let start = self.current_span().clone();
        if self.starts_collection_call_expr() || matches!(self.peek(), Token::LBracket) {
            let value = self.parse_expr()?;
            if matches!(value, Expr::CollectionCall { .. }) && matches!(self.peek(), Token::Dot) {
                return Err(self.error(
                    "chained collection expressions are deferred in M3-Phase 7; assign a single `append` / `drop-last` call or a static list literal",
                ));
            }
            let semi_desc = self.peek().description();
            if !matches!(self.peek(), Token::Semicolon) {
                return Err(self.error(format!("expected `;`, found {}", semi_desc)));
            }
            let semi = self.advance();
            return Ok(BlockStatement::Expr(ExprStatement {
                value,
                span: Span {
                    start: start.start,
                    end: semi.span.end,
                    line: start.line,
                    col: start.col,
                },
            }));
        }

        let target = self.parse_qualified_name()?;
        let op = self.parse_assign_op()?;
        let value = self.parse_expr()?;
        if matches!(value, Expr::CollectionCall { .. }) && matches!(self.peek(), Token::Dot) {
            return Err(self.error(
                "chained collection expressions are deferred in M3-Phase 7; assign a single `append` / `drop-last` call or a static list literal",
            ));
        }

        let semi_desc = self.peek().description();
        if !matches!(self.peek(), Token::Semicolon) {
            return Err(self.error(format!("expected `;`, found {}", semi_desc)));
        }
        let semi = self.advance();
        Ok(BlockStatement::Assignment(Statement {
            target,
            op,
            value,
            span: Span {
                start: start.start,
                end: semi.span.end,
                line: start.line,
                col: start.col,
            },
        }))
    }

    fn starts_collection_call_expr(&self) -> bool {
        if !matches!(self.peek(), Token::Ident(_)) {
            return false;
        }
        let mut i = 1;
        while matches!(self.peek_n(i), Token::Dot) {
            if !matches!(self.peek_n(i + 1), Token::Ident(_)) {
                return false;
            }
            if matches!(self.peek_n(i + 2), Token::LParen) {
                return true;
            }
            i += 2;
        }
        false
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
                | Token::LBracket
                | Token::Kw(Keyword::True)
                | Token::Kw(Keyword::False)
                | Token::RatioLit(_, _)
                | Token::ColorLit(_)
        );
        if !is_valid {
            let desc = self.peek().description();
            return Err(self.error(format!("expected expression, found {}", desc)));
        }
        if matches!(self.peek(), Token::LBracket) {
            return self.parse_list_lit();
        }
        if matches!(self.peek(), Token::Ident(_)) {
            return self.parse_ident_expr();
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

    fn parse_ident_expr(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.current_span().clone();
        let (first, first_span) = self.expect_ident()?;
        let mut segments = vec![first];
        let mut last_end = first_span.end;

        while matches!(self.peek(), Token::Dot) {
            if matches!(self.peek_n(1), Token::Ident(_)) && matches!(self.peek_n(2), Token::LParen)
            {
                self.advance(); // dot
                let (method, _) = self.expect_ident()?;
                self.expect_lparen()?;
                let mut args = Vec::new();
                if !matches!(self.peek(), Token::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if matches!(self.peek(), Token::Comma) {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                let end_tok = self.expect_rparen()?;
                return Ok(Expr::CollectionCall {
                    receiver: QualifiedName {
                        segments,
                        span: Span {
                            start: start.start,
                            end: last_end,
                            line: start.line,
                            col: start.col,
                        },
                    },
                    method,
                    args,
                    span: Span {
                        start: start.start,
                        end: end_tok.span.end,
                        line: start.line,
                        col: start.col,
                    },
                });
            }

            self.advance(); // dot
            let (seg, seg_span) = self.expect_ident()?;
            last_end = seg_span.end;
            segments.push(seg);
        }

        // Indexed collection reads (`xs[i]`) have no grammar in M3-Phase 7;
        // an Ident immediately followed by `[` is the loop-external indexed
        // read deferral (DD-M3-P7-007). Reject with a named diagnostic rather
        // than the generic "expected member" fallthrough. A list literal `[…]`
        // is parsed before this path is reached, so a `[` here is always a
        // post-ident index.
        if matches!(self.peek(), Token::LBracket) {
            return Err(self.error(
                "collection reads outside iteration not yet supported; indexed reads (`xs[i]`) are deferred in M3-Phase 7 — read elements through a `for` binder",
            ));
        }

        if segments.len() == 1 {
            Ok(Expr::Ident {
                name: segments.remove(0),
                span: Span {
                    start: start.start,
                    end: last_end,
                    line: start.line,
                    col: start.col,
                },
            })
        } else {
            Ok(Expr::QualifiedRef {
                name: QualifiedName {
                    segments,
                    span: Span {
                        start: start.start,
                        end: last_end,
                        line: start.line,
                        col: start.col,
                    },
                },
            })
        }
    }

    fn parse_list_lit(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.current_span().clone();
        self.expect_lbracket()?;
        let mut items = Vec::new();
        if !matches!(self.peek(), Token::RBracket) {
            loop {
                items.push(self.parse_expr()?);
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                    continue;
                }
                break;
            }
        }
        let end_tok = self.expect_rbracket()?;
        Ok(Expr::ListLit {
            items,
            span: Span {
                start: start.start,
                end: end_tok.span.end,
                line: start.line,
                col: start.col,
            },
        })
    }

    fn parse_type_name(&mut self) -> Result<TypeName, Diagnostic> {
        let ident = self.peek().as_ident().map(|s| s.to_string());
        let scalar = match ident.as_deref() {
            Some("int") | Some("i32") => {
                self.advance();
                TypeName::Int
            }
            Some("string") => {
                self.advance();
                TypeName::Str
            }
            Some("float") => {
                self.advance();
                TypeName::Float
            }
            Some("bool") => {
                self.advance();
                TypeName::Bool
            }
            Some(other) => {
                let msg = format!("unknown type `{}`; expected i32, string, or bool", other);
                return Err(self.error(msg));
            }
            None => {
                let desc = self.peek().description();
                return Err(self.error(format!("expected type name, found {}", desc)));
            }
        };

        if matches!(self.peek(), Token::LBracket) {
            self.advance();
            self.expect_rbracket()?;
            if matches!(self.peek(), Token::LBracket) {
                return Err(self.error(
                    "nested collection types are not supported in M3-Phase 7; collection state types are i32[], string[], or bool[]",
                ));
            }
            let elem = match scalar {
                TypeName::Int => CollectionElemType::Int,
                TypeName::Str => CollectionElemType::Str,
                TypeName::Bool => CollectionElemType::Bool,
                TypeName::Float | TypeName::Collection(_) => {
                    return Err(self.error(
                        "collection state types are limited to i32[], string[], and bool[] in M3-Phase 7",
                    ));
                }
            };
            Ok(TypeName::Collection(elem))
        } else {
            Ok(scalar)
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
    fn slot_dotted_property_bind_canonicalizes_name() {
        let def = parse_ok("component C inherits W { ZStack { Text { slot.h-align: end } } }");
        let Member::WidgetDecl { members, .. } = &def.members[0] else {
            panic!("expected root widget");
        };
        let Member::WidgetDecl { members, .. } = &members[0] else {
            panic!("expected child widget");
        };
        let Member::PropertyBind { name, value, .. } = &members[0] else {
            panic!("expected slot property bind");
        };
        assert_eq!(name, "slot.h-align");
        assert!(matches!(value, Expr::Ident { name, .. } if name == "end"));
    }

    #[test]
    fn malformed_slot_property_keys_rejected_at_parse() {
        let msg = parse_err_msg("component C inherits W { ZStack { Text { slot: end } } }");
        assert!(
            msg.contains("malformed slot property key") && msg.contains("slot.<key>"),
            "message: {msg}"
        );

        let msg =
            parse_err_msg("component C inherits W { ZStack { Text { slot..h-align: end } } }");
        assert!(
            msg.contains("expected identifier") && msg.contains("`.`"),
            "message: {msg}"
        );

        let msg = parse_err_msg("component C inherits W { ZStack { Text { slot. } } }");
        assert!(
            msg.contains("expected identifier") && msg.contains("`}`"),
            "message: {msg}"
        );
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
            let BlockStatement::Assignment(stmt) = &body.statements[0] else {
                panic!("expected assignment statement");
            };
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
    fn conditional_member_parses_inside_widget_body() {
        let def = parse_ok("component C inherits W { VStack { if ready { Text {} } } }");
        if let Member::WidgetDecl { members, .. } = &def.members[0] {
            assert!(matches!(&members[0], Member::Conditional { body, .. } if body.len() == 1));
        } else {
            panic!("expected root widget");
        }
    }

    #[test]
    fn collection_state_and_for_member_parse() {
        let def = parse_ok(
            r#"component C inherits W {
                state labels: string[] = ["a", "b"]
                WrapPanel { for label, i in labels { Text { text: label } } }
            }"#,
        );
        assert!(matches!(
            &def.members[0],
            Member::StateMember {
                ty: TypeName::Collection(CollectionElemType::Str),
                default: Expr::ListLit { items, .. },
                ..
            } if items.len() == 2
        ));
        let Member::WidgetDecl { members, .. } = &def.members[1] else {
            panic!("expected WrapPanel");
        };
        assert!(matches!(
            &members[0],
            Member::For {
                binder,
                index_binder: Some(index),
                collection: Expr::Ident { name, .. },
                body,
                ..
            } if binder == "label" && index == "i" && name == "labels" && body.len() == 1
        ));
    }

    #[test]
    fn collection_assignment_contextual_methods_parse() {
        let def = parse_ok(
            "component C inherits W { state append: i32[] = [] Button { clicked => { append = append.append(1); append = append.drop-last(); } } }",
        );
        let Member::WidgetDecl { members, .. } = &def.members[1] else {
            panic!("expected Button");
        };
        let Member::SignalHandler { body, .. } = &members[0] else {
            panic!("expected handler");
        };
        assert_eq!(body.statements.len(), 2);
        let BlockStatement::Assignment(first) = &body.statements[0] else {
            panic!("expected assignment");
        };
        assert!(matches!(
            &first.value,
            Expr::CollectionCall { receiver, method, args, .. }
                if receiver.segments == ["append"] && method == "append" && args.len() == 1
        ));
    }

    #[test]
    fn for_keyword_binder_rejected_at_identifier_position() {
        let cases = [
            "component C inherits W { state xs: i32[] = [] WrapPanel { for in in xs { Text {} } } }",
            "component C inherits W { state xs: i32[] = [] WrapPanel { for x, in in xs { Text {} } } }",
        ];
        for src in cases {
            let msg = parse_err_msg(src);
            assert!(
                msg.contains("expected identifier") && msg.contains("`in`"),
                "message: {msg}"
            );
        }
    }

    #[test]
    fn nested_collection_type_rejected_at_parse() {
        let msg = parse_err_msg("component C inherits W { state xs: i32[][] = [] VStack {} }");
        assert!(msg.contains("nested collection types"), "message: {msg}");
    }

    #[test]
    fn chained_collection_call_rejected_at_parse() {
        let msg = parse_err_msg(
            "component C inherits W { state xs: i32[] = [] Button { clicked => { xs = xs.append(1).append(2); } } }",
        );
        assert!(
            msg.contains("chained collection expressions are deferred"),
            "message: {msg}"
        );
    }

    #[test]
    fn indexed_collection_read_rejected_at_parse() {
        // `xs[0]` has no grammar; reject with the named loop-external read
        // deferral rather than a generic "expected member" fallthrough.
        let msg =
            parse_err_msg("component C inherits W { state xs: i32[] = [] Text { text: xs[0] } }");
        assert!(
            msg.contains("collection reads outside iteration") && msg.contains("indexed reads"),
            "message: {msg}"
        );
    }

    #[test]
    fn reserved_control_flow_keywords_without_production_rejected() {
        let msg = parse_err_msg("component C inherits W { VStack { else { Text {} } } }");
        assert!(
            msg.contains("reserved") && msg.contains("not yet supported"),
            "message: {msg}"
        );
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

    // --- M3-Phase 5 T1: Grid track-list parser path (DD-M3-P5-002) ---

    /// Pull the `members` of the first widget decl in the component body.
    fn first_widget_members(def: &ComponentDef) -> &[Member] {
        match &def.members[0] {
            Member::WidgetDecl { members, .. } => members,
            other => panic!("expected WidgetDecl, got {other:?}"),
        }
    }

    #[test]
    fn grid_track_list_fixed_and_weighted_star() {
        let def = parse_ok("component C inherits W { Grid { columns: 180 1* 2* rows: 1* 1* } }");
        let members = first_widget_members(&def);
        let cols = members
            .iter()
            .find_map(|m| match m {
                Member::GridTracks {
                    axis: TrackAxis::Columns,
                    tracks,
                    ..
                } => Some(tracks),
                _ => None,
            })
            .expect("columns track list");
        assert!(matches!(cols[0], TrackSize::Fixed { value: 180, .. }));
        assert!(matches!(cols[1], TrackSize::Star { weight: 1, .. }));
        assert!(matches!(cols[2], TrackSize::Star { weight: 2, .. }));
        let rows = members
            .iter()
            .find_map(|m| match m {
                Member::GridTracks {
                    axis: TrackAxis::Rows,
                    tracks,
                    ..
                } => Some(tracks),
                _ => None,
            })
            .expect("rows track list");
        assert_eq!(rows.len(), 2);
        assert!(rows
            .iter()
            .all(|t| matches!(t, TrackSize::Star { weight: 1, .. })));
    }

    #[test]
    fn grid_adjacent_star_is_one_weighted_track() {
        // `1*` is a single weighted-star track.
        let def = parse_ok("component C inherits W { Grid { columns: 1* rows: 1* } }");
        let cols = grid_axis(&def, TrackAxis::Columns);
        assert_eq!(cols.len(), 1);
        assert!(matches!(cols[0], TrackSize::Star { weight: 1, .. }));
    }

    #[test]
    fn grid_non_adjacent_star_is_fixed_then_unit_star() {
        // `1 *` (whitespace) is a Fixed(1) track followed by a unit-star
        // track — the R-A spike Decision 3 adjacency distinction.
        let def = parse_ok("component C inherits W { Grid { columns: 1 * rows: 1* } }");
        let cols = grid_axis(&def, TrackAxis::Columns);
        assert_eq!(cols.len(), 2);
        assert!(matches!(cols[0], TrackSize::Fixed { value: 1, .. }));
        assert!(matches!(cols[1], TrackSize::Star { weight: 1, .. }));
    }

    #[test]
    fn grid_bare_unit_star_track() {
        let def = parse_ok("component C inherits W { Grid { columns: * rows: 1* } }");
        let cols = grid_axis(&def, TrackAxis::Columns);
        assert_eq!(cols.len(), 1);
        assert!(matches!(cols[0], TrackSize::Star { weight: 1, .. }));
    }

    #[test]
    fn grid_auto_word_captured_for_check_layer() {
        // The parser captures `auto` as a Word token; the reserved-future
        // diagnostic is the check layer's responsibility.
        let def = parse_ok("component C inherits W { Grid { columns: auto rows: 1* } }");
        let cols = grid_axis(&def, TrackAxis::Columns);
        assert!(matches!(&cols[0], TrackSize::Word { name, .. } if name == "auto"));
    }

    #[test]
    fn grid_float_track_captured_as_invalid_float() {
        let def = parse_ok("component C inherits W { Grid { columns: 1.5 rows: 1* } }");
        let cols = grid_axis(&def, TrackAxis::Columns);
        assert!(matches!(cols[0], TrackSize::InvalidFloat { .. }));
    }

    #[test]
    fn grid_empty_track_list_rejected_at_parse() {
        let msg = parse_err_msg("component C inherits W { Grid { columns: rows: 1* } }");
        assert!(
            msg.contains("at least one track size") && msg.contains("columns"),
            "message: {msg}"
        );
    }

    #[test]
    fn track_list_routing_only_inside_grid() {
        // `columns:` on a non-Grid widget stays a generic PropertyBind —
        // the track-list grammar never leaks outside Grid (R-A spike
        // Decision 1). Here `columns: 5` is a single-int property bind.
        let def = parse_ok("component C inherits W { VStack { columns: 5 } }");
        let members = first_widget_members(&def);
        assert!(matches!(
            &members[0],
            Member::PropertyBind { name, value: Expr::IntLit { value: 5, .. }, .. } if name == "columns"
        ));
    }

    #[test]
    fn grid_cell_children_parse_as_widget_decls() {
        let def = parse_ok(
            r#"component C inherits W {
                Grid {
                    columns: 1* rows: 1*
                    Cell { row: 0 column: 0 Text { text: "x" } }
                }
            }"#,
        );
        let members = first_widget_members(&def);
        let cell = members
            .iter()
            .find(|m| matches!(m, Member::WidgetDecl { type_name, .. } if type_name == "Cell"))
            .expect("Cell widget decl");
        if let Member::WidgetDecl { members: cm, .. } = cell {
            assert!(cm
                .iter()
                .any(|m| matches!(m, Member::PropertyBind { name, .. } if name == "row")));
            assert!(cm
                .iter()
                .any(|m| matches!(m, Member::WidgetDecl { type_name, .. } if type_name == "Text")));
        }
    }

    /// Helper: the track list for a given axis of the first widget decl.
    fn grid_axis(def: &ComponentDef, want: TrackAxis) -> &[TrackSize] {
        first_widget_members(def)
            .iter()
            .find_map(|m| match m {
                Member::GridTracks { axis, tracks, .. } if *axis == want => Some(tracks.as_slice()),
                _ => None,
            })
            .expect("track list for axis")
    }
}
