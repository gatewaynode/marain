//! String literal scanner.
//!
//! Recognizes `"..."` with escape sequences `\"`, `\\`, `\n`, `\t`, `\r`,
//! `\0`, and f-strings `f"…{^x}…"` (R17). Triple-quoted (`"""..."""`) is
//! deferred past v0.1.

use crate::span::{FileId, Span};
use crate::token::{FStringSeg, Token, TokenKind};

use super::cursor::Cursor;
use super::error::LexError;
use super::idents::scan_sigiled_ident;

/// Scan a string literal. Caller has verified `cursor.peek() == Some(b'"')`.
pub(super) fn scan_string(cursor: &mut Cursor, file: FileId) -> Result<Token, LexError> {
    let start = cursor.pos();
    cursor.advance(); // consume opening quote

    let mut value = String::new();
    loop {
        // Scan a chunk of plain text up to a special byte. Special bytes
        // are all ASCII; no risk of breaking inside a UTF-8 multi-byte char.
        let chunk_start = cursor.pos();
        while let Some(b) = cursor.peek() {
            if matches!(b, b'"' | b'\\' | b'\n') {
                break;
            }
            cursor.advance();
        }
        let chunk_end = cursor.pos();
        if chunk_end > chunk_start {
            value.push_str(cursor.slice(chunk_start, chunk_end));
        }

        match cursor.peek() {
            None | Some(b'\n') => {
                return Err(LexError::UnterminatedString {
                    span: Span::new(start, cursor.pos(), file),
                });
            }
            Some(b'"') => {
                cursor.advance(); // consume closing quote
                return Ok(Token::new(
                    TokenKind::StringLit(value),
                    Span::new(start, cursor.pos(), file),
                ));
            }
            Some(b'\\') => {
                let escape_start = cursor.pos();
                cursor.advance(); // consume backslash
                value.push(decode_escape(cursor, escape_start, start, file)?);
            }
            Some(_) => unreachable!("chunk loop only breaks on special bytes"),
        }
    }
}

/// Decode the escape body following a `\` (already consumed by the caller) and
/// return the resolved char. Shared by [`scan_string`] and [`scan_fstring`].
/// `escape_start` points at the backslash (for `InvalidEscape` spans);
/// `str_start` points at the opening quote (for `UnterminatedString` spans).
fn decode_escape(
    cursor: &mut Cursor,
    escape_start: u32,
    str_start: u32,
    file: FileId,
) -> Result<char, LexError> {
    match cursor.advance() {
        None => Err(LexError::UnterminatedString {
            span: Span::new(str_start, cursor.pos(), file),
        }),
        Some(b'"') => Ok('"'),
        Some(b'\\') => Ok('\\'),
        Some(b'n') => Ok('\n'),
        Some(b't') => Ok('\t'),
        Some(b'r') => Ok('\r'),
        Some(b'0') => Ok('\0'),
        Some(other) => Err(LexError::InvalidEscape {
            ch: other as char,
            span: Span::new(escape_start, cursor.pos(), file),
        }),
    }
}

/// Consume zero or more inline spaces (used to tolerate `{ ^x }` padding).
fn skip_inline_spaces(cursor: &mut Cursor) {
    while cursor.peek() == Some(b' ') {
        cursor.advance();
    }
}

/// Scan an f-string literal `f"…{^x}…"` (R17). The caller has consumed the `f`
/// prefix and verified `cursor.peek() == Some(b'"')`; `start` is the byte
/// offset of the `f`.
///
/// The whole literal — including each `{…}` hole — is resolved here in a single
/// pass: a hole is read by reusing [`scan_sigiled_ident`], so its `SigiledIdent`
/// carries a correct source span and `FileId` with no re-lexing. Holes are
/// variable-refs-only: `{}`, `{nomen}` (no sigil), `{^a plus ^b}` (expression),
/// and an unmatched `}` are all [`LexError::InvalidFStringHole`]. `{{`/`}}`
/// decode to literal braces. Because internal `{`/`}` never reach the main
/// dispatch, they do not perturb the indent/bracket state machine.
pub(super) fn scan_fstring(
    cursor: &mut Cursor,
    start: u32,
    file: FileId,
) -> Result<Token, LexError> {
    cursor.advance(); // consume opening quote
    let mut parts: Vec<FStringSeg> = Vec::new();
    let mut lit = String::new();

    loop {
        // Plain-text chunk up to a special byte. `{`/`}` join the escape set.
        let chunk_start = cursor.pos();
        while let Some(b) = cursor.peek() {
            if matches!(b, b'"' | b'\\' | b'\n' | b'{' | b'}') {
                break;
            }
            cursor.advance();
        }
        let chunk_end = cursor.pos();
        if chunk_end > chunk_start {
            lit.push_str(cursor.slice(chunk_start, chunk_end));
        }

        match cursor.peek() {
            None | Some(b'\n') => {
                return Err(LexError::UnterminatedString {
                    span: Span::new(start, cursor.pos(), file),
                });
            }
            Some(b'"') => {
                cursor.advance(); // consume closing quote
                if !lit.is_empty() {
                    parts.push(FStringSeg::Literal(lit));
                }
                return Ok(Token::new(
                    TokenKind::FStringLit(parts),
                    Span::new(start, cursor.pos(), file),
                ));
            }
            Some(b'\\') => {
                let escape_start = cursor.pos();
                cursor.advance(); // consume backslash
                lit.push(decode_escape(cursor, escape_start, start, file)?);
            }
            Some(b'{') => {
                if cursor.peek_at(1) == Some(b'{') {
                    cursor.advance();
                    cursor.advance();
                    lit.push('{');
                } else {
                    if !lit.is_empty() {
                        parts.push(FStringSeg::Literal(std::mem::take(&mut lit)));
                    }
                    parts.push(scan_fstring_hole(cursor, file)?);
                }
            }
            Some(b'}') => {
                if cursor.peek_at(1) == Some(b'}') {
                    cursor.advance();
                    cursor.advance();
                    lit.push('}');
                } else {
                    let here = cursor.pos();
                    cursor.advance();
                    return Err(LexError::InvalidFStringHole {
                        span: Span::new(here, cursor.pos(), file),
                    });
                }
            }
            Some(_) => unreachable!("chunk loop only breaks on special bytes"),
        }
    }
}

/// Scan one `{…}` hole. Caller is positioned at the opening `{`. The interior
/// must be exactly one sigiled variable (optionally space-padded); anything
/// else is [`LexError::InvalidFStringHole`].
fn scan_fstring_hole(cursor: &mut Cursor, file: FileId) -> Result<FStringSeg, LexError> {
    let hole_start = cursor.pos();
    cursor.advance(); // consume `{`
    skip_inline_spaces(cursor);

    let interp = match cursor.peek() {
        Some(b'@') | Some(b'^') => scan_sigiled_ident(cursor, file)?,
        _ => {
            return Err(LexError::InvalidFStringHole {
                span: Span::new(hole_start, cursor.pos(), file),
            });
        }
    };

    skip_inline_spaces(cursor);
    if cursor.peek() != Some(b'}') {
        return Err(LexError::InvalidFStringHole {
            span: Span::new(hole_start, cursor.pos(), file),
        });
    }
    cursor.advance(); // consume `}`

    match interp.kind {
        TokenKind::SigiledIdent { sigil, name } => Ok(FStringSeg::Interp {
            sigil,
            name,
            span: interp.span,
        }),
        _ => unreachable!("scan_sigiled_ident always yields SigiledIdent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fid() -> FileId {
        FileId::new(1).expect("nonzero")
    }

    #[test]
    fn simple_string() {
        let mut c = Cursor::new("\"hello\"");
        let t = scan_string(&mut c, fid()).expect("ok");
        assert_eq!(t.kind, TokenKind::StringLit("hello".to_string()));
        assert_eq!(c.peek(), None);
    }

    #[test]
    fn empty_string() {
        let mut c = Cursor::new("\"\"");
        let t = scan_string(&mut c, fid()).expect("ok");
        assert_eq!(t.kind, TokenKind::StringLit(String::new()));
    }

    #[test]
    fn escape_sequences() {
        let mut c = Cursor::new(r#""a\nb\tc\\d\"e\0""#);
        let t = scan_string(&mut c, fid()).expect("ok");
        assert_eq!(t.kind, TokenKind::StringLit("a\nb\tc\\d\"e\0".to_string()),);
    }

    #[test]
    fn unterminated_at_eof() {
        let mut c = Cursor::new("\"oops");
        let err = scan_string(&mut c, fid()).expect_err("err");
        assert!(matches!(err, LexError::UnterminatedString { .. }));
    }

    #[test]
    fn unterminated_at_newline() {
        let mut c = Cursor::new("\"oops\nmore\"");
        let err = scan_string(&mut c, fid()).expect_err("err");
        assert!(matches!(err, LexError::UnterminatedString { .. }));
    }

    #[test]
    fn invalid_escape() {
        let mut c = Cursor::new("\"\\q\"");
        let err = scan_string(&mut c, fid()).expect_err("err");
        assert!(matches!(err, LexError::InvalidEscape { ch: 'q', .. }));
    }

    #[test]
    fn utf8_body_preserved() {
        let mut c = Cursor::new("\"sálve\"");
        let t = scan_string(&mut c, fid()).expect("ok");
        assert_eq!(t.kind, TokenKind::StringLit("sálve".to_string()));
    }

    #[test]
    fn stops_at_closing_quote_leaves_cursor_after() {
        let mut c = Cursor::new("\"a\".rest");
        scan_string(&mut c, fid()).expect("ok");
        assert_eq!(c.peek(), Some(b'.'));
    }

    // --- R17: f-strings ---
    // `scan_fstring` is called after the `f` prefix is consumed, so each input
    // here begins at the opening quote (the lexer dispatch handles the `f`).

    use crate::token::Sigil;

    fn fstr(text: &str) -> Vec<FStringSeg> {
        let mut c = Cursor::new(text);
        match scan_fstring(&mut c, 0, fid()).expect("ok").kind {
            TokenKind::FStringLit(parts) => parts,
            other => panic!("expected FStringLit, got {other:?}"),
        }
    }

    fn lit(seg: &FStringSeg) -> &str {
        match seg {
            FStringSeg::Literal(s) => s.as_str(),
            other => panic!("expected Literal, got {other:?}"),
        }
    }

    fn interp(seg: &FStringSeg) -> (Sigil, &str) {
        match seg {
            FStringSeg::Interp { sigil, name, .. } => (*sigil, name.as_str()),
            other => panic!("expected Interp, got {other:?}"),
        }
    }

    #[test]
    fn fstring_plain_text_is_one_literal() {
        let p = fstr("\"salve\"");
        assert_eq!(p.len(), 1);
        assert_eq!(lit(&p[0]), "salve");
    }

    #[test]
    fn fstring_empty_has_no_parts() {
        assert!(fstr("\"\"").is_empty());
    }

    #[test]
    fn fstring_single_hole() {
        let p = fstr("\"{^x}\"");
        assert_eq!(p.len(), 1);
        assert_eq!(interp(&p[0]), (Sigil::Immutable, "x"));
    }

    #[test]
    fn fstring_text_around_hole() {
        let p = fstr("\"salve {^nomen}!\"");
        assert_eq!(p.len(), 3);
        assert_eq!(lit(&p[0]), "salve ");
        assert_eq!(interp(&p[1]), (Sigil::Immutable, "nomen"));
        assert_eq!(lit(&p[2]), "!");
    }

    #[test]
    fn fstring_adjacent_holes_are_concatenation() {
        let p = fstr("\"{^a}{^b}\"");
        assert_eq!(p.len(), 2);
        assert_eq!(interp(&p[0]), (Sigil::Immutable, "a"));
        assert_eq!(interp(&p[1]), (Sigil::Immutable, "b"));
    }

    #[test]
    fn fstring_mutable_sigil_hole() {
        let p = fstr("\"{@count}\"");
        assert_eq!(interp(&p[0]), (Sigil::Mutable, "count"));
    }

    #[test]
    fn fstring_doubled_braces_are_literal() {
        let p = fstr("\"{{x}}\"");
        assert_eq!(p.len(), 1);
        assert_eq!(lit(&p[0]), "{x}");
    }

    #[test]
    fn fstring_hole_tolerates_surrounding_spaces() {
        let p = fstr("\"{ ^x }\"");
        assert_eq!(interp(&p[0]), (Sigil::Immutable, "x"));
    }

    #[test]
    fn fstring_decodes_escapes_in_literal() {
        let p = fstr("\"a\\nb\"");
        assert_eq!(lit(&p[0]), "a\nb");
    }

    #[test]
    fn fstring_hole_span_points_at_the_variable() {
        // `"{^x}"` — the `^x` sits at byte offsets 2..4 (after the quote and `{`).
        let p = fstr("\"{^x}\"");
        match &p[0] {
            FStringSeg::Interp { span, .. } => {
                assert_eq!(span.start, 2);
                assert_eq!(span.end, 4);
            }
            other => panic!("expected Interp, got {other:?}"),
        }
    }

    fn fstr_err(text: &str) -> LexError {
        let mut c = Cursor::new(text);
        scan_fstring(&mut c, 0, fid()).expect_err("expected error")
    }

    #[test]
    fn fstring_empty_hole_is_error() {
        assert!(matches!(
            fstr_err("\"{}\""),
            LexError::InvalidFStringHole { .. }
        ));
    }

    #[test]
    fn fstring_hole_without_sigil_is_error() {
        assert!(matches!(
            fstr_err("\"{nomen}\""),
            LexError::InvalidFStringHole { .. }
        ));
    }

    #[test]
    fn fstring_expression_hole_is_error() {
        assert!(matches!(
            fstr_err("\"{^a plus ^b}\""),
            LexError::InvalidFStringHole { .. }
        ));
    }

    #[test]
    fn fstring_unmatched_close_brace_is_error() {
        assert!(matches!(
            fstr_err("\"a}b\""),
            LexError::InvalidFStringHole { .. }
        ));
    }

    #[test]
    fn fstring_unterminated_is_error() {
        assert!(matches!(
            fstr_err("\"oops"),
            LexError::UnterminatedString { .. }
        ));
    }

    #[test]
    fn fstring_unterminated_at_newline_is_error() {
        assert!(matches!(
            fstr_err("\"oops\n\""),
            LexError::UnterminatedString { .. }
        ));
    }
}
