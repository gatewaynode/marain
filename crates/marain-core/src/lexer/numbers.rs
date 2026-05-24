//! Numeric literal scanner.
//!
//! v0.1 supports only decimal integers, with `_` separators (PRD §4.3).
//! Hex/oct/bin prefixes, exponents, and floats are deferred past v0.1.

use crate::span::{FileId, Span};
use crate::token::{Token, TokenKind};

use super::cursor::Cursor;
use super::error::LexError;

/// Scan a decimal integer. Caller has verified the next byte is a digit.
pub(super) fn scan_number(cursor: &mut Cursor, file: FileId) -> Result<Token, LexError> {
    let (start, end) = cursor.advance_while(|b| b.is_ascii_digit() || b == b'_');
    let raw = cursor.slice(start, end);
    let span = Span::new(start, end, file);
    let cleaned: String = raw.chars().filter(|&c| c != '_').collect();
    match cleaned.parse::<i64>() {
        Ok(n) => Ok(Token::new(TokenKind::IntegerLit(n), span)),
        Err(_) => Err(LexError::InvalidInteger {
            text: raw.to_string(),
            span,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fid() -> FileId {
        FileId::new(1).expect("nonzero")
    }

    #[test]
    fn simple_integer() {
        let mut c = Cursor::new("42");
        let t = scan_number(&mut c, fid()).expect("ok");
        assert_eq!(t.kind, TokenKind::IntegerLit(42));
    }

    #[test]
    fn integer_with_separators() {
        let mut c = Cursor::new("1_000_000");
        let t = scan_number(&mut c, fid()).expect("ok");
        assert_eq!(t.kind, TokenKind::IntegerLit(1_000_000));
    }

    #[test]
    fn stops_at_non_digit() {
        let mut c = Cursor::new("123abc");
        let t = scan_number(&mut c, fid()).expect("ok");
        assert_eq!(t.kind, TokenKind::IntegerLit(123));
        assert_eq!(c.peek(), Some(b'a'));
    }

    #[test]
    fn overflow_is_invalid() {
        let mut c = Cursor::new("99999999999999999999");
        let r = scan_number(&mut c, fid());
        assert!(matches!(r, Err(LexError::InvalidInteger { .. })));
    }

    #[test]
    fn zero_is_valid() {
        let mut c = Cursor::new("0");
        let t = scan_number(&mut c, fid()).expect("ok");
        assert_eq!(t.kind, TokenKind::IntegerLit(0));
    }
}
