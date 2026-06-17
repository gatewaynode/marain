//! Lexer driver: orchestrates per-token scanners and the indent state
//! machine into a complete token stream ending in EOF.
//!
//! Driver integration tests live in the sibling `mod_tests.rs` (declared at
//! the foot of this file) — split out per CLAUDE.md when the in-file tests
//! pushed the module past the 500-LOC target while the dispatcher itself
//! stayed small.

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
            // `f"` (with no intervening space) is the f-string prefix (R17).
            // A bare `f` or `functio`/`fit`/`f(...)` falls through to the ident
            // path; variables always carry a sigil, so `f"` is unambiguous.
            b'f' if cursor.peek_at(1) == Some(b'"') => {
                cursor.advance(); // consume the `f` prefix
                strings::scan_fstring(&mut cursor, start, file_id)?
            }
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
#[path = "mod_tests.rs"]
mod tests;
