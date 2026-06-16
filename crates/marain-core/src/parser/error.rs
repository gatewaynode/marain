//! Parser error type.

use std::fmt;

use crate::error::Diagnostic;
use crate::span::Span;
use crate::token::TokenKind;

#[derive(Clone, Debug)]
pub enum ParseError {
    /// A token appeared where some specific kind was expected.
    UnexpectedToken {
        found: TokenKind,
        expected: &'static str,
        span: Span,
    },
    /// An expression was expected but the next token cannot begin one.
    ExpectedExpression { found: TokenKind, span: Span },
    /// A statement was expected but its first token matches no statement form.
    UnknownStatementStart { found: TokenKind, span: Span },
    /// A type-position identifier did not start with an uppercase letter
    /// (PRD §4.9: types use PascalCase; mismatch is a hard error, not a lint).
    TypePositionRequiresPascalCase { name: String, span: Span },
    /// A `fit` reassignment targeted an immutable (`^`) binding. PRD §4.5 makes
    /// the sigil the mutability marker at every use site, so reassigning a `^`
    /// target is a contradiction caught at parse time (locked decision
    /// 2026-06-16). `name` is the bare identifier (no sigil).
    ImmutableReassignmentTarget { name: String, span: Span },
}

impl ParseError {
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedToken { span, .. }
            | Self::ExpectedExpression { span, .. }
            | Self::UnknownStatementStart { span, .. }
            | Self::TypePositionRequiresPascalCase { span, .. }
            | Self::ImmutableReassignmentTarget { span, .. } => *span,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::UnexpectedToken {
                found, expected, ..
            } => format!("expected {expected}, found {found}"),
            Self::ExpectedExpression { found, .. } => {
                format!("expected expression, found {found}")
            }
            Self::UnknownStatementStart { found, .. } => {
                format!("statement cannot begin with {found}")
            }
            Self::TypePositionRequiresPascalCase { name, .. } => {
                format!("type names must use PascalCase; got `{name}` (PRD §4.9)")
            }
            Self::ImmutableReassignmentTarget { name, .. } => format!(
                "cannot reassign immutable binding `^{name}`; declare it `@{name}` to mutate (PRD §4.5)"
            ),
        }
    }

    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::error(self.span(), self.message())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::FileId;

    fn fid() -> FileId {
        FileId::new(1).expect("nonzero")
    }

    fn sp() -> Span {
        Span::new(0, 1, fid())
    }

    #[test]
    fn unexpected_token_message_mentions_found_and_expected() {
        let e = ParseError::UnexpectedToken {
            found: TokenKind::Period,
            expected: "keyword `est`",
            span: sp(),
        };
        let msg = e.message();
        assert!(msg.contains("expected keyword `est`"));
        assert!(msg.contains("`.`"), "found token should render: {msg}");
    }

    #[test]
    fn expected_expression_message() {
        let e = ParseError::ExpectedExpression {
            found: TokenKind::Period,
            span: sp(),
        };
        assert!(e.message().contains("expected expression"));
    }

    #[test]
    fn unknown_statement_start_message() {
        let e = ParseError::UnknownStatementStart {
            found: TokenKind::Comma,
            span: sp(),
        };
        assert!(e.message().contains("statement cannot begin"));
    }

    #[test]
    fn type_position_requires_pascal_case_message() {
        let e = ParseError::TypePositionRequiresPascalCase {
            name: "sermo".to_string(),
            span: sp(),
        };
        let msg = e.message();
        assert!(msg.contains("PascalCase"), "msg: {msg}");
        assert!(msg.contains("sermo"), "msg should quote name: {msg}");
        assert!(msg.contains("PRD §4.9"), "msg should cite PRD: {msg}");
    }

    #[test]
    fn immutable_reassignment_target_message() {
        let e = ParseError::ImmutableReassignmentTarget {
            name: "x".to_string(),
            span: sp(),
        };
        let msg = e.message();
        assert!(msg.contains("cannot reassign immutable"), "msg: {msg}");
        assert!(
            msg.contains("`^x`"),
            "msg should show the `^` target: {msg}"
        );
        assert!(msg.contains("`@x`"), "msg should suggest `@x`: {msg}");
        assert!(msg.contains("PRD §4.5"), "msg should cite PRD: {msg}");
    }

    #[test]
    fn span_extraction_round_trip() {
        let e = ParseError::ExpectedExpression {
            found: TokenKind::Period,
            span: sp(),
        };
        assert_eq!(e.span(), sp());
    }

    #[test]
    fn to_diagnostic_carries_message_and_span() {
        let e = ParseError::ExpectedExpression {
            found: TokenKind::Period,
            span: sp(),
        };
        let d = e.to_diagnostic();
        assert_eq!(d.message, e.message());
        assert_eq!(d.span, e.span());
    }

    #[test]
    fn display_delegates_to_message() {
        let e = ParseError::ExpectedExpression {
            found: TokenKind::Period,
            span: sp(),
        };
        assert_eq!(e.to_string(), e.message());
    }
}
