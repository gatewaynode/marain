//! Token kinds and the [`Token`] wrapper carried through the parser.

use std::fmt;

use crate::lexer::keywords::Keyword;
use crate::span::Span;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sigil {
    Mutable,   // @
    Immutable, // ^
}

impl Sigil {
    pub fn as_char(self) -> char {
        match self {
            Self::Mutable => '@',
            Self::Immutable => '^',
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum TokenKind {
    // Identifiers and literals
    PlainIdent(String),
    SigiledIdent { sigil: Sigil, name: String },
    Keyword(Keyword),
    StringLit(String),
    IntegerLit(i64),

    // Punctuation
    Period,
    DotDot,   // .. (range, exclusive)
    DotDotEq, // ..= (range, inclusive)
    Comma,
    Colon,
    DoubleColon,
    Bang,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Layout (synthetic)
    Indent,
    Dedent,
    Eof,
}

/// Human-readable rendering used by parser-error messages.
///
/// Renders the *kind* of token, not its value — `StringLit("hi")` formats as
/// `string literal`, not as `"hi"`, so diagnostics stay terse and do not leak
/// arbitrary user payloads into error text.
impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlainIdent(_) => f.write_str("identifier"),
            Self::SigiledIdent { sigil, .. } => {
                write!(f, "sigiled identifier (`{}`)", sigil.as_char())
            }
            Self::Keyword(k) => write!(f, "keyword `{}`", k.as_str()),
            Self::StringLit(_) => f.write_str("string literal"),
            Self::IntegerLit(_) => f.write_str("integer literal"),
            Self::Period => f.write_str("`.`"),
            Self::DotDot => f.write_str("`..`"),
            Self::DotDotEq => f.write_str("`..=`"),
            Self::Comma => f.write_str("`,`"),
            Self::Colon => f.write_str("`:`"),
            Self::DoubleColon => f.write_str("`::`"),
            Self::Bang => f.write_str("`!`"),
            Self::LParen => f.write_str("`(`"),
            Self::RParen => f.write_str("`)`"),
            Self::LBracket => f.write_str("`[`"),
            Self::RBracket => f.write_str("`]`"),
            Self::LBrace => f.write_str("`{`"),
            Self::RBrace => f.write_str("`}`"),
            Self::Indent => f.write_str("INDENT"),
            Self::Dedent => f.write_str("DEDENT"),
            Self::Eof => f.write_str("end of file"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigil_as_char() {
        assert_eq!(Sigil::Mutable.as_char(), '@');
        assert_eq!(Sigil::Immutable.as_char(), '^');
    }

    #[test]
    fn token_kind_display_hides_literal_values() {
        assert_eq!(
            TokenKind::StringLit("anything".to_string()).to_string(),
            "string literal",
        );
        assert_eq!(TokenKind::IntegerLit(42).to_string(), "integer literal");
        assert_eq!(
            TokenKind::PlainIdent("foo".to_string()).to_string(),
            "identifier"
        );
    }

    #[test]
    fn token_kind_display_renders_punctuation_with_backticks() {
        assert_eq!(TokenKind::Period.to_string(), "`.`");
        assert_eq!(TokenKind::DoubleColon.to_string(), "`::`");
        assert_eq!(TokenKind::Bang.to_string(), "`!`");
        assert_eq!(TokenKind::DotDot.to_string(), "`..`");
        assert_eq!(TokenKind::DotDotEq.to_string(), "`..=`");
    }

    #[test]
    fn token_kind_display_names_keyword() {
        assert_eq!(
            TokenKind::Keyword(Keyword::Sit).to_string(),
            "keyword `sit`",
        );
    }

    #[test]
    fn token_kind_display_sigil_in_sigiled_ident() {
        let t = TokenKind::SigiledIdent {
            sigil: Sigil::Immutable,
            name: "x".to_string(),
        };
        assert_eq!(t.to_string(), "sigiled identifier (`^`)");
    }

    #[test]
    fn token_kind_display_eof_human_readable() {
        assert_eq!(TokenKind::Eof.to_string(), "end of file");
    }
}
