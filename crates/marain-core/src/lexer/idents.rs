//! Identifier scanner: plain (bare word, possibly resolved to a keyword)
//! and sigiled (`@x` / `^x`, never a keyword per PRD §4.5).

use crate::span::{FileId, Span};
use crate::token::{Sigil, Token, TokenKind};

use super::cursor::Cursor;
use super::error::LexError;
use super::keywords::Keyword;

pub(super) fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

pub(super) fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Scan a plain identifier. Caller has verified `cursor.peek()` is an
/// identifier-start byte. If the resulting name matches the keyword table,
/// emits `Keyword`; otherwise `PlainIdent`.
pub(super) fn scan_ident(cursor: &mut Cursor, file: FileId) -> Token {
    let (start, end) = cursor.advance_while(is_ident_continue);
    let name = cursor.slice(start, end);
    let span = Span::new(start, end, file);
    if let Some(kw) = Keyword::lookup(name) {
        Token::new(TokenKind::Keyword(kw), span)
    } else {
        Token::new(TokenKind::PlainIdent(name.to_string()), span)
    }
}

/// Scan a sigiled identifier. Caller has verified `cursor.peek()` is `@` or
/// `^`. Sigiled identifiers never consult the keyword table.
pub(super) fn scan_sigiled_ident(cursor: &mut Cursor, file: FileId) -> Result<Token, LexError> {
    let start = cursor.pos();
    let sigil_byte = cursor
        .advance()
        .expect("caller verified sigil byte present");
    let sigil = match sigil_byte {
        b'@' => Sigil::Mutable,
        b'^' => Sigil::Immutable,
        _ => unreachable!("scan_sigiled_ident called with non-sigil byte"),
    };

    match cursor.peek() {
        Some(b) if is_ident_start(b) => {
            let (name_start, name_end) = cursor.advance_while(is_ident_continue);
            let name = cursor.slice(name_start, name_end);
            Ok(Token::new(
                TokenKind::SigiledIdent {
                    sigil,
                    name: name.to_string(),
                },
                Span::new(start, name_end, file),
            ))
        }
        _ => Err(LexError::SigilWithoutIdent {
            sigil: sigil.as_char(),
            span: Span::new(start, cursor.pos(), file),
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
    fn plain_ident() {
        let mut c = Cursor::new("hello");
        let t = scan_ident(&mut c, fid());
        assert_eq!(t.kind, TokenKind::PlainIdent("hello".to_string()));
    }

    #[test]
    fn plain_ident_resolves_keyword() {
        let mut c = Cursor::new("dic");
        let t = scan_ident(&mut c, fid());
        assert_eq!(t.kind, TokenKind::Keyword(Keyword::Dic));
    }

    #[test]
    fn plain_ident_stops_at_non_ident() {
        let mut c = Cursor::new("foo_bar123 ");
        let t = scan_ident(&mut c, fid());
        assert_eq!(t.kind, TokenKind::PlainIdent("foo_bar123".to_string()));
        assert_eq!(c.peek(), Some(b' '));
    }

    #[test]
    fn plain_ident_with_leading_underscore() {
        let mut c = Cursor::new("_unused");
        let t = scan_ident(&mut c, fid());
        assert_eq!(t.kind, TokenKind::PlainIdent("_unused".to_string()));
    }

    #[test]
    fn sigiled_mutable() {
        let mut c = Cursor::new("@x");
        let t = scan_sigiled_ident(&mut c, fid()).expect("ok");
        assert_eq!(
            t.kind,
            TokenKind::SigiledIdent {
                sigil: Sigil::Mutable,
                name: "x".to_string(),
            },
        );
    }

    #[test]
    fn sigiled_immutable() {
        let mut c = Cursor::new("^y");
        let t = scan_sigiled_ident(&mut c, fid()).expect("ok");
        assert_eq!(
            t.kind,
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "y".to_string(),
            },
        );
    }

    #[test]
    fn sigiled_with_keyword_name_is_still_variable() {
        // ^et is a variable named "et", NOT Keyword::Et.
        let mut c = Cursor::new("^et");
        let t = scan_sigiled_ident(&mut c, fid()).expect("ok");
        assert_eq!(
            t.kind,
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "et".to_string(),
            },
        );
    }

    #[test]
    fn sigil_without_ident_errors() {
        let mut c = Cursor::new("@ ");
        let r = scan_sigiled_ident(&mut c, fid());
        assert!(matches!(
            r,
            Err(LexError::SigilWithoutIdent { sigil: '@', .. })
        ));
    }

    #[test]
    fn sigil_at_eof_errors() {
        let mut c = Cursor::new("^");
        let r = scan_sigiled_ident(&mut c, fid());
        assert!(matches!(
            r,
            Err(LexError::SigilWithoutIdent { sigil: '^', .. })
        ));
    }

    #[test]
    fn sigil_before_digit_errors() {
        // @123 — digits are ident_continue but not ident_start.
        let mut c = Cursor::new("@123");
        let r = scan_sigiled_ident(&mut c, fid());
        assert!(matches!(r, Err(LexError::SigilWithoutIdent { .. })));
    }
}
