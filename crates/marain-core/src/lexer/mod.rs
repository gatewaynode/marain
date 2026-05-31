//! Lexer driver: orchestrates per-token scanners and the indent state
//! machine into a complete token stream ending in EOF.
//!
//! 665 LOC, exceeds 500 target: ~225 LOC executable plus ~440 LOC of driver
//! integration tests that exercise the full lex pipeline (indent / brackets /
//! sigils / keywords / strings / numbers / line comments / `<>` lookalike).
//! The tests share an in-scope `lex_str` / `lex_err` setup and want to live
//! near the dispatcher they exercise; per CLAUDE.md the sibling-file split
//! is the obvious decomposition if pressure rises further, but at 665 LOC
//! the single-file shape still reads more cleanly than a split would.

mod comments;
mod cursor;
mod error;
mod idents;
mod indent;
pub mod keywords;
mod numbers;
mod strings;

pub use error::LexError;

use crate::source::SourceFile;
use crate::span::Span;
use crate::token::{Token, TokenKind};

use cursor::Cursor;
use indent::{IndentState, LineStartOutcome};

/// Tokenize a source file into a complete token stream ending in EOF.
pub fn lex(file: &SourceFile) -> Result<Vec<Token>, LexError> {
    let mut cursor = Cursor::new(file.text());
    let mut indent = IndentState::new();
    let mut tokens = Vec::new();
    let file_id = file.id();
    let mut at_line_start = true;

    loop {
        if at_line_start {
            // Measure leading whitespace. Spaces only; any tab is a hard error.
            let indent_start = cursor.pos();
            let mut measured_indent: u32 = 0;
            loop {
                match cursor.peek() {
                    Some(b' ') => {
                        cursor.advance();
                        measured_indent += 1;
                    }
                    Some(b'\t') => {
                        return Err(LexError::TabCharacter {
                            span: Span::new(cursor.pos(), cursor.pos() + 1, file_id),
                        });
                    }
                    _ => break,
                }
            }

            // Handle EOF and blank lines specially.
            match cursor.peek() {
                None => {
                    let drain = indent.finalize();
                    let eof_span = Span::new(cursor.pos(), cursor.pos(), file_id);
                    for _ in 0..drain {
                        tokens.push(Token::new(TokenKind::Dedent, eof_span));
                    }
                    tokens.push(Token::new(TokenKind::Eof, eof_span));
                    return Ok(tokens);
                }
                Some(b'\n') => {
                    cursor.advance();
                    continue; // blank line: skip without changing state
                }
                _ => {}
            }

            // Comment-only lines do not affect indentation (PRD §4.12). After
            // leading whitespace, peek for `//`; if found, consume the
            // comment and skip indent consultation entirely — the line is
            // layout-only, identical to a blank line for the indent stack.
            if cursor.peek() == Some(b'/') && cursor.peek_at(1) == Some(b'/') {
                cursor.advance(); // first /
                cursor.advance(); // second /
                comments::scan_line_comment(&mut cursor);
                continue;
            }

            // Consult the indent stack.
            match indent.line_start(measured_indent) {
                LineStartOutcome::NoChange => {}
                LineStartOutcome::Indent => {
                    tokens.push(Token::new(
                        TokenKind::Indent,
                        Span::new(indent_start, cursor.pos(), file_id),
                    ));
                }
                LineStartOutcome::Dedents(n) => {
                    let zero_span = Span::new(cursor.pos(), cursor.pos(), file_id);
                    for _ in 0..n {
                        tokens.push(Token::new(TokenKind::Dedent, zero_span));
                    }
                }
                LineStartOutcome::Inconsistent => {
                    return Err(LexError::InconsistentIndent {
                        span: Span::new(indent_start, cursor.pos(), file_id),
                    });
                }
            }

            at_line_start = false;
        }

        // Skip inline whitespace.
        while let Some(b' ') = cursor.peek() {
            cursor.advance();
        }
        if cursor.peek() == Some(b'\t') {
            return Err(LexError::TabCharacter {
                span: Span::new(cursor.pos(), cursor.pos() + 1, file_id),
            });
        }

        let start = cursor.pos();
        let b = match cursor.peek() {
            None => {
                let drain = indent.finalize();
                let eof_span = Span::new(start, start, file_id);
                for _ in 0..drain {
                    tokens.push(Token::new(TokenKind::Dedent, eof_span));
                }
                tokens.push(Token::new(TokenKind::Eof, eof_span));
                return Ok(tokens);
            }
            Some(b'\n') => {
                cursor.advance();
                at_line_start = true;
                continue;
            }
            Some(b) => b,
        };

        // `/` opens either `//` (line comment, PRD §4.12) or `/*` (block
        // comment, reserved-deferred). Bare `/` is unexpected — division is
        // `divisus per` per PRD §4.4, so `/` has no standalone use today.
        if b == b'/' {
            cursor.advance(); // first /
            match cursor.peek() {
                Some(b'/') => {
                    cursor.advance(); // second /
                    comments::scan_line_comment(&mut cursor);
                    continue;
                }
                Some(b'*') => {
                    cursor.advance(); // consume *
                    return Err(LexError::BlockCommentsDeferred {
                        span: Span::new(start, cursor.pos(), file_id),
                    });
                }
                _ => {
                    return Err(LexError::UnexpectedChar {
                        ch: '/',
                        span: Span::new(start, cursor.pos(), file_id),
                    });
                }
            }
        }

        let token = match b {
            b'"' => strings::scan_string(&mut cursor, file_id)?,
            b'0'..=b'9' => numbers::scan_number(&mut cursor, file_id)?,
            b if idents::is_ident_start(b) => idents::scan_ident(&mut cursor, file_id),
            b'@' | b'^' => idents::scan_sigiled_ident(&mut cursor, file_id)?,
            b'.' => {
                cursor.advance(); // first .
                match cursor.peek() {
                    Some(b'.') => {
                        cursor.advance(); // second .
                        if cursor.peek() == Some(b'=') {
                            cursor.advance(); // =
                            Token::new(TokenKind::DotDotEq, Span::new(start, cursor.pos(), file_id))
                        } else {
                            Token::new(TokenKind::DotDot, Span::new(start, cursor.pos(), file_id))
                        }
                    }
                    _ => Token::new(TokenKind::Period, Span::new(start, cursor.pos(), file_id)),
                }
            }
            b',' => {
                cursor.advance();
                Token::new(TokenKind::Comma, Span::new(start, cursor.pos(), file_id))
            }
            b':' => {
                cursor.advance();
                if cursor.peek() == Some(b':') {
                    cursor.advance();
                    Token::new(
                        TokenKind::DoubleColon,
                        Span::new(start, cursor.pos(), file_id),
                    )
                } else {
                    Token::new(TokenKind::Colon, Span::new(start, cursor.pos(), file_id))
                }
            }
            b'!' => {
                cursor.advance();
                Token::new(TokenKind::Bang, Span::new(start, cursor.pos(), file_id))
            }
            b'(' => {
                cursor.advance();
                indent.enter_bracket();
                Token::new(TokenKind::LParen, Span::new(start, cursor.pos(), file_id))
            }
            b')' => {
                cursor.advance();
                indent.exit_bracket();
                Token::new(TokenKind::RParen, Span::new(start, cursor.pos(), file_id))
            }
            b'[' => {
                cursor.advance();
                indent.enter_bracket();
                Token::new(TokenKind::LBracket, Span::new(start, cursor.pos(), file_id))
            }
            b']' => {
                cursor.advance();
                indent.exit_bracket();
                Token::new(TokenKind::RBracket, Span::new(start, cursor.pos(), file_id))
            }
            b'{' => {
                cursor.advance();
                indent.enter_bracket();
                Token::new(TokenKind::LBrace, Span::new(start, cursor.pos(), file_id))
            }
            b'}' => {
                cursor.advance();
                indent.exit_bracket();
                Token::new(TokenKind::RBrace, Span::new(start, cursor.pos(), file_id))
            }
            b'<' | b'>' => {
                let ch = b as char;
                cursor.advance();
                return Err(LexError::GenericsLookalike {
                    ch,
                    span: Span::new(start, cursor.pos(), file_id),
                });
            }
            other => {
                let ch = other as char;
                cursor.advance();
                return Err(LexError::UnexpectedChar {
                    ch,
                    span: Span::new(start, cursor.pos(), file_id),
                });
            }
        };
        tokens.push(token);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::source::SourceMap;
    use crate::token::Sigil;
    use keywords::Keyword;

    fn lex_str(text: &str) -> Vec<TokenKind> {
        let mut map = SourceMap::new();
        let id = map.add(PathBuf::from("test.lat"), text.to_string());
        let tokens = lex(map.get(id)).expect("lex failed");
        tokens.into_iter().map(|t| t.kind).collect()
    }

    fn lex_err(text: &str) -> LexError {
        let mut map = SourceMap::new();
        let id = map.add(PathBuf::from("test.lat"), text.to_string());
        lex(map.get(id)).expect_err("expected lex error")
    }

    #[test]
    fn hello_world() {
        let toks = lex_str("dic \"salve, munde\".\n");
        assert_eq!(
            toks,
            vec![
                TokenKind::Keyword(Keyword::Dic),
                TokenKind::StringLit("salve, munde".to_string()),
                TokenKind::Period,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn binding_with_sigils() {
        let toks = lex_str("sit ^x est 5.\n");
        assert_eq!(
            toks,
            vec![
                TokenKind::Keyword(Keyword::Sit),
                TokenKind::SigiledIdent {
                    sigil: Sigil::Immutable,
                    name: "x".to_string(),
                },
                TokenKind::Keyword(Keyword::Est),
                TokenKind::IntegerLit(5),
                TokenKind::Period,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn indented_block() {
        let toks = lex_str("functio foo:\n    dic \"a\".\n    dic \"b\".\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Functio),
            TokenKind::PlainIdent("foo".to_string()),
            TokenKind::Colon,
            TokenKind::Indent,
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::StringLit("a".to_string()),
            TokenKind::Period,
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::StringLit("b".to_string()),
            TokenKind::Period,
            TokenKind::Dedent,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn nested_indents_emit_cascading_dedents() {
        let toks = lex_str("a:\n    b:\n        c.\n");
        let expected = vec![
            TokenKind::PlainIdent("a".to_string()),
            TokenKind::Colon,
            TokenKind::Indent,
            TokenKind::PlainIdent("b".to_string()),
            TokenKind::Colon,
            TokenKind::Indent,
            TokenKind::PlainIdent("c".to_string()),
            TokenKind::Period,
            TokenKind::Dedent,
            TokenKind::Dedent,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn blank_lines_skipped() {
        let toks = lex_str("dic \"x\".\n\n\ndic \"y\".\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::StringLit("x".to_string()),
            TokenKind::Period,
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::StringLit("y".to_string()),
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn brackets_suppress_indent() {
        // A continuation line indented inside parens emits no INDENT.
        let toks = lex_str("dic(\n    \"a\",\n).\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::LParen,
            TokenKind::StringLit("a".to_string()),
            TokenKind::Comma,
            TokenKind::RParen,
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn tab_in_indentation_is_error() {
        let err = lex_err("\tdic \"x\".\n");
        assert!(matches!(err, LexError::TabCharacter { .. }));
    }

    #[test]
    fn tab_mid_line_is_error() {
        let err = lex_err("dic\t\"x\".\n");
        assert!(matches!(err, LexError::TabCharacter { .. }));
    }

    #[test]
    fn unexpected_char_is_error() {
        let err = lex_err("?");
        assert!(matches!(err, LexError::UnexpectedChar { ch: '?', .. }));
    }

    #[test]
    fn unterminated_string_is_error() {
        let err = lex_err("dic \"unfinished\n");
        assert!(matches!(err, LexError::UnterminatedString { .. }));
    }

    #[test]
    fn double_colon_one_token() {
        let toks = lex_str("a::b.\n");
        let expected = vec![
            TokenKind::PlainIdent("a".to_string()),
            TokenKind::DoubleColon,
            TokenKind::PlainIdent("b".to_string()),
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn bang_is_separate_token() {
        let toks = lex_str("dbg!(@x).\n");
        let expected = vec![
            TokenKind::PlainIdent("dbg".to_string()),
            TokenKind::Bang,
            TokenKind::LParen,
            TokenKind::SigiledIdent {
                sigil: Sigil::Mutable,
                name: "x".to_string(),
            },
            TokenKind::RParen,
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn integer_with_underscore() {
        let toks = lex_str("sit ^x est 1_000.\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Sit),
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "x".to_string(),
            },
            TokenKind::Keyword(Keyword::Est),
            TokenKind::IntegerLit(1000),
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn empty_source_emits_only_eof() {
        let toks = lex_str("");
        assert_eq!(toks, vec![TokenKind::Eof]);
    }

    #[test]
    fn whitespace_only_source_emits_only_eof() {
        let toks = lex_str("   \n  \n");
        assert_eq!(toks, vec![TokenKind::Eof]);
    }

    #[test]
    fn detonatio_keyword_recognized() {
        let toks = lex_str("DETONATIO!(\"oops\").\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Detonatio),
            TokenKind::Bang,
            TokenKind::LParen,
            TokenKind::StringLit("oops".to_string()),
            TokenKind::RParen,
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn escape_sequences_in_string() {
        let toks = lex_str("dic \"a\\nb\\tc\".\n");
        assert!(matches!(&toks[1], TokenKind::StringLit(s) if s == "a\nb\tc"));
    }

    #[test]
    fn no_trailing_newline_still_drains_indent() {
        // Multi-statement with no trailing newline; final dedent must still emit.
        let toks = lex_str("a:\n    b.");
        let expected = vec![
            TokenKind::PlainIdent("a".to_string()),
            TokenKind::Colon,
            TokenKind::Indent,
            TokenKind::PlainIdent("b".to_string()),
            TokenKind::Period,
            TokenKind::Dedent,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn inconsistent_indent_is_error() {
        // Indent to 4, then to 2 (which is on neither stack frame).
        let err = lex_err("a:\n    b.\n  c.\n");
        assert!(matches!(err, LexError::InconsistentIndent { .. }));
    }

    #[test]
    fn multiple_statements_on_one_line() {
        let toks = lex_str("dic ^x. dic ^y.\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "x".to_string(),
            },
            TokenKind::Period,
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "y".to_string(),
            },
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    // --- R9: line comments (PRD §4.12) ---

    #[test]
    fn trailing_comment_after_statement() {
        let toks = lex_str("sit ^x est 5. // note\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Sit),
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "x".to_string(),
            },
            TokenKind::Keyword(Keyword::Est),
            TokenKind::IntegerLit(5),
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn standalone_comment_at_top_of_file() {
        let toks = lex_str("// preamble\nsit ^x est 5.\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Sit),
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "x".to_string(),
            },
            TokenKind::Keyword(Keyword::Est),
            TokenKind::IntegerLit(5),
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn comment_only_file_emits_only_eof() {
        let toks = lex_str("// just a comment\n");
        assert_eq!(toks, vec![TokenKind::Eof]);
    }

    #[test]
    fn comment_only_file_no_trailing_newline() {
        let toks = lex_str("// no newline at end");
        assert_eq!(toks, vec![TokenKind::Eof]);
    }

    #[test]
    fn consecutive_comment_only_lines_no_indent_change() {
        let toks = lex_str("// one\n// two\n// three\nsit ^x est 5.\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Sit),
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "x".to_string(),
            },
            TokenKind::Keyword(Keyword::Est),
            TokenKind::IntegerLit(5),
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn comment_inside_indented_block_does_not_dedent() {
        let toks = lex_str("a:\n    b.\n    // inside\n    c.\n");
        let expected = vec![
            TokenKind::PlainIdent("a".to_string()),
            TokenKind::Colon,
            TokenKind::Indent,
            TokenKind::PlainIdent("b".to_string()),
            TokenKind::Period,
            TokenKind::PlainIdent("c".to_string()),
            TokenKind::Period,
            TokenKind::Dedent,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn block_comment_is_deferred_error() {
        let err = lex_err("/* block */");
        match err {
            LexError::BlockCommentsDeferred { span } => {
                // span covers exactly the two-byte `/*`
                assert_eq!(span.start, 0);
                assert_eq!(span.end, 2);
            }
            other => panic!("expected BlockCommentsDeferred, got {other:?}"),
        }
    }

    #[test]
    fn block_comment_message_mentions_alternative() {
        let err = lex_err("/* block */");
        let msg = err.message();
        assert!(
            msg.contains("//"),
            "message should suggest `//`; got: {msg}"
        );
        assert!(
            msg.contains("reserved"),
            "message should call out reserved status; got: {msg}",
        );
    }

    #[test]
    fn open_angle_is_generics_lookalike() {
        let err = lex_err("dat Agmen<T>.\n");
        match err {
            LexError::GenericsLookalike { ch, span } => {
                assert_eq!(ch, '<');
                assert_eq!(span.end - span.start, 1);
            }
            other => panic!("expected GenericsLookalike, got {other:?}"),
        }
    }

    #[test]
    fn close_angle_is_generics_lookalike() {
        let err = lex_err("functio foo() dat Sermo>.\n");
        match err {
            LexError::GenericsLookalike { ch, .. } => assert_eq!(ch, '>'),
            other => panic!("expected GenericsLookalike, got {other:?}"),
        }
    }

    #[test]
    fn generics_lookalike_message_mentions_generics_and_alternative() {
        let err = lex_err("dat Agmen<T>.\n");
        let msg = err.message();
        assert!(
            msg.contains("generics"),
            "message should mention generics; got: {msg}"
        );
        assert!(
            msg.contains("v0.3"),
            "message should cite v0.3 deferral; got: {msg}"
        );
    }

    #[test]
    fn bare_slash_is_unexpected_char() {
        let err = lex_err("/foo");
        match err {
            LexError::UnexpectedChar { ch, .. } => assert_eq!(ch, '/'),
            other => panic!("expected UnexpectedChar, got {other:?}"),
        }
    }

    // --- R14: range tokens (`..` / `..=`) ---

    #[test]
    fn dot_dot_is_one_token() {
        let toks = lex_str("0..10");
        let expected = vec![
            TokenKind::IntegerLit(0),
            TokenKind::DotDot,
            TokenKind::IntegerLit(10),
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn dot_dot_eq_is_one_token() {
        let toks = lex_str("0..=10");
        let expected = vec![
            TokenKind::IntegerLit(0),
            TokenKind::DotDotEq,
            TokenKind::IntegerLit(10),
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn period_unchanged_by_dotdot_dispatcher() {
        // Statement-terminator `.` (single dot) still lexes as Period.
        let toks = lex_str("dic ^x.\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "x".to_string(),
            },
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }

    #[test]
    fn three_dots_lex_as_dotdot_then_period() {
        // `...` is not its own token; greedy `..` wins, third `.` is Period.
        let toks = lex_str("...");
        let expected = vec![TokenKind::DotDot, TokenKind::Period, TokenKind::Eof];
        assert_eq!(toks, expected);
    }

    #[test]
    fn dot_dot_with_trailing_period() {
        // `0..10.` (range used as macro arg) — IntegerLit, DotDot, IntegerLit, Period.
        let toks = lex_str("dic 0..10.\n");
        let expected = vec![
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::IntegerLit(0),
            TokenKind::DotDot,
            TokenKind::IntegerLit(10),
            TokenKind::Period,
            TokenKind::Eof,
        ];
        assert_eq!(toks, expected);
    }
}
