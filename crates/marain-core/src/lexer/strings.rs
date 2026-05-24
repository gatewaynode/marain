//! String literal scanner.
//!
//! Recognizes `"..."` with escape sequences `\"`, `\\`, `\n`, `\t`, `\r`,
//! `\0`. Triple-quoted (`"""..."""`) and f-strings are deferred past v0.1.

use crate::span::{FileId, Span};
use crate::token::{Token, TokenKind};

use super::cursor::Cursor;
use super::error::LexError;

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
                match cursor.advance() {
                    None => {
                        return Err(LexError::UnterminatedString {
                            span: Span::new(start, cursor.pos(), file),
                        });
                    }
                    Some(b'"') => value.push('"'),
                    Some(b'\\') => value.push('\\'),
                    Some(b'n') => value.push('\n'),
                    Some(b't') => value.push('\t'),
                    Some(b'r') => value.push('\r'),
                    Some(b'0') => value.push('\0'),
                    Some(other) => {
                        return Err(LexError::InvalidEscape {
                            ch: other as char,
                            span: Span::new(escape_start, cursor.pos(), file),
                        });
                    }
                }
            }
            Some(_) => unreachable!("chunk loop only breaks on special bytes"),
        }
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
}
