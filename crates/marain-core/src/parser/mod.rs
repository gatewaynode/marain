//! Parser driver.
//!
//! Owns the token cursor and exposes `parse(&[Token]) -> Result<Module, ParseError>`.
//! Per-production grammar functions live in [`grammar`].

mod error;
mod grammar;

pub use error::ParseError;

use crate::ast::Module;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// Parse a token stream into a Stage 1 [`Module`].
///
/// `tokens` must end with [`TokenKind::Eof`] (the lexer's contract).
pub fn parse(tokens: &[Token]) -> Result<Module, ParseError> {
    let mut parser = Parser::new(tokens);
    grammar::parse_module(&mut parser)
}

/// Token cursor. Internal to the parser module.
pub(crate) struct Parser<'tokens> {
    tokens: &'tokens [Token],
    pos: usize,
}

impl<'tokens> Parser<'tokens> {
    fn new(tokens: &'tokens [Token]) -> Self {
        debug_assert!(
            matches!(tokens.last().map(|t| &t.kind), Some(TokenKind::Eof)),
            "parser requires a token stream terminated with Eof",
        );
        Self { tokens, pos: 0 }
    }

    pub(crate) fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    pub(crate) fn peek_span(&self) -> Span {
        self.tokens[self.pos].span
    }

    pub(crate) fn current_clone(&self) -> Token {
        self.tokens[self.pos].clone()
    }

    pub(crate) fn advance(&mut self) {
        if !matches!(self.peek_kind(), TokenKind::Eof) {
            self.pos += 1;
        }
    }

    pub(crate) fn at_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::ast::{Expr, Stmt};
    use crate::error::MarainError;
    use crate::lexer::lex;
    use crate::source::SourceMap;
    use crate::token::Sigil;

    fn parse_str(text: &str) -> Result<Module, MarainError> {
        let mut map = SourceMap::new();
        let id = map.add(PathBuf::from("test.lat"), text.to_string());
        let tokens = lex(map.get(id))?;
        Ok(parse(&tokens)?)
    }

    fn parse_ok(text: &str) -> Module {
        parse_str(text).expect("expected parse to succeed")
    }

    fn parse_err(text: &str) -> ParseError {
        let mut map = SourceMap::new();
        let id = map.add(PathBuf::from("test.lat"), text.to_string());
        let tokens = lex(map.get(id)).expect("lex must succeed for these inputs");
        parse(&tokens).expect_err("expected parse to fail")
    }

    #[test]
    fn hello_world_parses() {
        let m = parse_ok("dic \"salve, munde\".\n");
        assert_eq!(m.items.len(), 1);
        match &m.items[0] {
            Stmt::MacroCall(c) => {
                assert_eq!(c.callee.name, "dic");
                match &c.arg {
                    Expr::StringLit(s) => assert_eq!(s.value, "salve, munde"),
                    other => panic!("expected StringLit, got {other:?}"),
                }
            }
            other => panic!("expected MacroCall, got {other:?}"),
        }
    }

    #[test]
    fn let_with_integer_literal() {
        let m = parse_ok("sit ^x est 5.\n");
        assert_eq!(m.items.len(), 1);
        match &m.items[0] {
            Stmt::Let(l) => {
                assert_eq!(l.name.sigil, Sigil::Immutable);
                assert_eq!(l.name.name, "x");
                match &l.value {
                    Expr::IntegerLit(i) => assert_eq!(i.value, 5),
                    other => panic!("expected IntegerLit, got {other:?}"),
                }
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn let_with_string_literal() {
        let m = parse_ok("sit @greeting est \"salve\".\n");
        match &m.items[0] {
            Stmt::Let(l) => {
                assert_eq!(l.name.sigil, Sigil::Mutable);
                assert_eq!(l.name.name, "greeting");
                match &l.value {
                    Expr::StringLit(s) => assert_eq!(s.value, "salve"),
                    other => panic!("expected StringLit, got {other:?}"),
                }
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn dic_of_var_ref() {
        let m = parse_ok("dic ^x.\n");
        match &m.items[0] {
            Stmt::MacroCall(c) => match &c.arg {
                Expr::VarRef(v) => {
                    assert_eq!(v.sigil, Sigil::Immutable);
                    assert_eq!(v.name, "x");
                }
                other => panic!("expected VarRef, got {other:?}"),
            },
            other => panic!("expected MacroCall, got {other:?}"),
        }
    }

    #[test]
    fn multi_statement_program() {
        let m = parse_ok("sit ^x est 5.\ndic ^x.\n");
        assert_eq!(m.items.len(), 2);
        assert!(matches!(m.items[0], Stmt::Let(_)));
        assert!(matches!(m.items[1], Stmt::MacroCall(_)));
    }

    #[test]
    fn multi_statements_on_one_line() {
        let m = parse_ok("dic \"a\". dic \"b\".\n");
        assert_eq!(m.items.len(), 2);
        assert!(matches!(m.items[0], Stmt::MacroCall(_)));
        assert!(matches!(m.items[1], Stmt::MacroCall(_)));
    }

    #[test]
    fn empty_source_parses_to_empty_module() {
        let m = parse_ok("");
        assert!(m.items.is_empty());
    }

    #[test]
    fn whitespace_only_source_parses_to_empty_module() {
        let m = parse_ok("   \n  \n");
        assert!(m.items.is_empty());
    }

    #[test]
    fn dic_with_no_arg_is_error() {
        let e = parse_err("dic.\n");
        assert!(
            matches!(e, ParseError::ExpectedExpression { .. }),
            "got {e:?}",
        );
    }

    #[test]
    fn dic_with_trailing_garbage_is_error() {
        let e = parse_err("dic \"a\" \"b\".\n");
        // After "a", expect period; found another string.
        assert!(matches!(e, ParseError::UnexpectedToken { .. }), "got {e:?}");
    }

    #[test]
    fn sit_without_est_is_error() {
        let e = parse_err("sit ^x 5.\n");
        // After ^x, expecting `est`, found integer.
        match e {
            ParseError::UnexpectedToken { expected, .. } => {
                assert!(expected.contains("est"), "expected label was {expected:?}");
            }
            other => panic!("expected UnexpectedToken, got {other:?}"),
        }
    }

    #[test]
    fn sit_without_sigil_is_error() {
        let e = parse_err("sit x est 5.\n");
        match e {
            ParseError::UnexpectedToken { expected, .. } => {
                assert!(expected.contains("sigiled identifier"));
            }
            other => panic!("expected UnexpectedToken, got {other:?}"),
        }
    }

    #[test]
    fn unknown_statement_start_is_error() {
        let e = parse_err("functio foo.\n");
        assert!(
            matches!(e, ParseError::UnknownStatementStart { .. }),
            "got {e:?}",
        );
    }

    #[test]
    fn missing_period_at_eof_is_error() {
        let e = parse_err("dic \"a\"");
        match e {
            ParseError::UnexpectedToken {
                expected, found, ..
            } => {
                assert_eq!(expected, "`.`");
                assert!(matches!(found, TokenKind::Eof));
            }
            other => panic!("expected UnexpectedToken, got {other:?}"),
        }
    }

    #[test]
    fn let_value_var_ref() {
        let m = parse_ok("sit @y est ^x.\n");
        match &m.items[0] {
            Stmt::Let(l) => match &l.value {
                Expr::VarRef(v) => {
                    assert_eq!(v.sigil, Sigil::Immutable);
                    assert_eq!(v.name, "x");
                    assert!(v.inflection.is_none());
                }
                other => panic!("expected VarRef, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn integer_with_underscores_round_trips() {
        let m = parse_ok("sit ^big est 1_000_000.\n");
        match &m.items[0] {
            Stmt::Let(l) => match &l.value {
                Expr::IntegerLit(i) => assert_eq!(i.value, 1_000_000),
                other => panic!("expected IntegerLit, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn let_stmt_span_covers_sit_through_period() {
        let m = parse_ok("sit ^x est 5.\n");
        match &m.items[0] {
            Stmt::Let(l) => {
                assert_eq!(l.span.start, 0);
                // "sit ^x est 5." is 13 bytes; the period is at byte 12, so the
                // period span ends at byte 13.
                assert_eq!(l.span.end, 13);
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn macro_call_span_covers_keyword_through_period() {
        let m = parse_ok("dic \"a\".\n");
        match &m.items[0] {
            Stmt::MacroCall(c) => {
                assert_eq!(c.span.start, 0);
                assert_eq!(c.span.end, 8);
            }
            other => panic!("expected MacroCall, got {other:?}"),
        }
    }

    #[test]
    fn inflection_slot_defaults_none_after_parse() {
        let m = parse_ok("sit ^x est 5.\ndic ^x.\n");
        match &m.items[0] {
            Stmt::Let(l) => assert!(l.name.inflection.is_none()),
            _ => unreachable!(),
        }
        match &m.items[1] {
            Stmt::MacroCall(c) => match &c.arg {
                Expr::VarRef(v) => assert!(v.inflection.is_none()),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn other_no_punct_macros_recognized() {
        // queror, agmen, forma all dispatch through the same macro-call form.
        // agmen normally takes a list expression, but with a string lit it still parses.
        let m = parse_ok("queror \"oops\".\n");
        assert!(matches!(m.items[0], Stmt::MacroCall(_)));
        let m = parse_ok("forma \"x\".\n");
        assert!(matches!(m.items[0], Stmt::MacroCall(_)));
        let m = parse_ok("agmen \"x\".\n");
        assert!(matches!(m.items[0], Stmt::MacroCall(_)));
    }

    #[test]
    fn parse_error_joins_marain_error_facade() {
        let result = parse_str("dic.\n");
        match result {
            Err(MarainError::Parse(ParseError::ExpectedExpression { .. })) => {}
            other => panic!("expected MarainError::Parse, got {other:?}"),
        }
    }

    #[test]
    #[should_panic(expected = "parser requires a token stream terminated with Eof")]
    fn parser_requires_eof_terminator() {
        let _ = Parser::new(&[]);
    }

    #[test]
    fn si_with_single_statement_body() {
        let m = parse_ok("si ^x :\n    dic ^x.\n");
        assert_eq!(m.items.len(), 1);
        match &m.items[0] {
            Stmt::If(i) => {
                match &i.cond {
                    Expr::VarRef(v) => {
                        assert_eq!(v.sigil, Sigil::Immutable);
                        assert_eq!(v.name, "x");
                    }
                    other => panic!("expected VarRef cond, got {other:?}"),
                }
                assert_eq!(i.then_block.stmts.len(), 1);
                assert!(matches!(i.then_block.stmts[0], Stmt::MacroCall(_)));
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn si_with_multi_statement_body() {
        let m = parse_ok("si ^x :\n    dic ^x.\n    sit ^y est 7.\n");
        match &m.items[0] {
            Stmt::If(i) => {
                assert_eq!(i.then_block.stmts.len(), 2);
                assert!(matches!(i.then_block.stmts[0], Stmt::MacroCall(_)));
                assert!(matches!(i.then_block.stmts[1], Stmt::Let(_)));
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn si_with_nested_si_body() {
        let m = parse_ok("si ^x :\n    si ^y :\n        dic \"deep\".\n");
        match &m.items[0] {
            Stmt::If(outer) => {
                assert_eq!(outer.then_block.stmts.len(), 1);
                match &outer.then_block.stmts[0] {
                    Stmt::If(inner) => {
                        assert_eq!(inner.then_block.stmts.len(), 1);
                        assert!(matches!(inner.then_block.stmts[0], Stmt::MacroCall(_)));
                    }
                    other => panic!("expected nested If, got {other:?}"),
                }
            }
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn si_with_integer_literal_cond_parses() {
        // R10 doesn't validate type — the condition expression set is whatever
        // parse_expr accepts (string/int/var-ref). Producing valid Rust is R11's job.
        let m = parse_ok("si 1 :\n    dic \"hi\".\n");
        match &m.items[0] {
            Stmt::If(i) => match &i.cond {
                Expr::IntegerLit(n) => assert_eq!(n.value, 1),
                other => panic!("expected IntegerLit cond, got {other:?}"),
            },
            other => panic!("expected If, got {other:?}"),
        }
    }

    #[test]
    fn si_then_following_statement_at_column_zero() {
        // The body lands at column 0 — same indent as the parent — so the lexer
        // emits no Indent. parse_block surfaces the missing indent.
        let m = parse_ok("si ^x :\n    dic ^x.\nsit ^y est 7.\n");
        assert_eq!(m.items.len(), 2);
        assert!(matches!(m.items[0], Stmt::If(_)));
        assert!(matches!(m.items[1], Stmt::Let(_)));
    }

    #[test]
    fn si_without_colon_is_error() {
        let e = parse_err("si ^x\n    dic ^x.\n");
        match e {
            ParseError::UnexpectedToken { expected, .. } => {
                assert!(expected.contains("`:`"), "expected label was {expected:?}");
            }
            other => panic!("expected UnexpectedToken, got {other:?}"),
        }
    }

    #[test]
    fn si_without_condition_is_error() {
        let e = parse_err("si :\n    dic \"hi\".\n");
        assert!(
            matches!(e, ParseError::ExpectedExpression { .. }),
            "got {e:?}",
        );
    }

    #[test]
    fn si_without_indented_body_is_error() {
        let e = parse_err("si ^x :\nsit ^y est 1.\n");
        match e {
            ParseError::UnexpectedToken { expected, .. } => {
                assert!(
                    expected.contains("indented block"),
                    "expected label was {expected:?}",
                );
            }
            other => panic!("expected UnexpectedToken, got {other:?}"),
        }
    }

    #[test]
    fn si_at_eof_with_no_body_is_error() {
        let e = parse_err("si ^x :\n");
        match e {
            ParseError::UnexpectedToken {
                expected, found, ..
            } => {
                assert!(
                    expected.contains("indented block"),
                    "expected label was {expected:?}",
                );
                assert!(matches!(found, TokenKind::Eof), "found was {found:?}");
            }
            other => panic!("expected UnexpectedToken, got {other:?}"),
        }
    }

    #[test]
    fn if_stmt_span_covers_si_through_dedent() {
        let m = parse_ok("si ^x :\n    dic ^x.\n");
        match &m.items[0] {
            Stmt::If(i) => {
                assert_eq!(i.span.start, 0);
                // Span end is the Dedent's end byte; verify the structural extent
                // rather than an exact byte (Dedent is synthetic).
                assert!(i.span.end > i.then_block.stmts[0].span().end);
            }
            other => panic!("expected If, got {other:?}"),
        }
    }
}
