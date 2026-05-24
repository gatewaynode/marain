//! Lexer error type.

use std::fmt;

use crate::error::Diagnostic;
use crate::span::Span;

#[derive(Clone, Debug)]
pub enum LexError {
    /// A character was encountered that does not start any token.
    UnexpectedChar { ch: char, span: Span },
    /// A string literal opened with `"` was not closed before EOF or newline.
    UnterminatedString { span: Span },
    /// A tab byte appeared anywhere (PRD §4.6: spaces only, all positions).
    TabCharacter { span: Span },
    /// An invalid escape sequence inside a string literal (e.g. `\q`).
    InvalidEscape { ch: char, span: Span },
    /// A line's indentation level matched no outer indent on the stack.
    InconsistentIndent { span: Span },
    /// A sigil (`@` or `^`) was followed by something that isn't an identifier.
    SigilWithoutIdent { sigil: char, span: Span },
    /// A numeric literal failed to parse (overflow or malformed).
    InvalidInteger { text: String, span: Span },
}

impl LexError {
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedChar { span, .. }
            | Self::UnterminatedString { span }
            | Self::TabCharacter { span }
            | Self::InvalidEscape { span, .. }
            | Self::InconsistentIndent { span }
            | Self::SigilWithoutIdent { span, .. }
            | Self::InvalidInteger { span, .. } => *span,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::UnexpectedChar { ch, .. } => format!("unexpected character {ch:?}"),
            Self::UnterminatedString { .. } => "unterminated string literal".to_string(),
            Self::TabCharacter { .. } => {
                "tab character not permitted; use spaces (PRD §4.6)".to_string()
            }
            Self::InvalidEscape { ch, .. } => format!("invalid string escape: \\{ch}"),
            Self::InconsistentIndent { .. } => {
                "indentation does not match any outer level".to_string()
            }
            Self::SigilWithoutIdent { sigil, .. } => {
                format!("sigil '{sigil}' must be followed by an identifier with no whitespace")
            }
            Self::InvalidInteger { text, .. } => format!("integer literal `{text}` is not valid"),
        }
    }

    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::error(self.span(), self.message())
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::FileId;

    fn s() -> Span {
        Span::new(0, 1, FileId::new(1).expect("nonzero"))
    }

    #[test]
    fn unexpected_char_message() {
        let e = LexError::UnexpectedChar { ch: '?', span: s() };
        assert!(e.message().contains("unexpected character"));
        assert!(e.message().contains("'?'"));
    }

    #[test]
    fn tab_message_mentions_spaces() {
        let e = LexError::TabCharacter { span: s() };
        assert!(e.message().contains("spaces"));
    }

    #[test]
    fn unterminated_string_message() {
        let e = LexError::UnterminatedString { span: s() };
        assert!(e.message().contains("unterminated"));
    }

    #[test]
    fn span_extraction_round_trip() {
        let e = LexError::UnexpectedChar { ch: '?', span: s() };
        assert_eq!(e.span(), s());
    }

    #[test]
    fn to_diagnostic_carries_message_and_span() {
        let e = LexError::UnexpectedChar { ch: '?', span: s() };
        let d = e.to_diagnostic();
        assert_eq!(d.message, e.message());
        assert_eq!(d.span, e.span());
    }

    #[test]
    fn display_renders_message() {
        let e = LexError::UnterminatedString { span: s() };
        assert_eq!(e.to_string(), "unterminated string literal");
    }
}
