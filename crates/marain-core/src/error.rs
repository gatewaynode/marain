//! Diagnostic output, the error-layering convention, and the [`MarainError`]
//! facade.
//!
//! [`Diagnostic`] is the renderable unit shown to the user. Per-stage error
//! types (e.g. [`crate::lexer::LexError`]) live in their own module and
//! expose `to_diagnostic()`. [`MarainError`] composes them via `From` impls
//! and dispatches `to_diagnostic` to the variant's own implementation.

use std::fmt;

use crate::emit::EmitError;
use crate::lexer::LexError;
use crate::parser::ParseError;
use crate::source::SourceMap;
use crate::span::Span;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => f.write_str("error"),
            Severity::Warning => f.write_str("warning"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn error(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            span,
            message: message.into(),
        }
    }

    pub fn warning(span: Span, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            span,
            message: message.into(),
        }
    }

    /// Render in the canonical `path:line:col: severity: message` form.
    pub fn render(&self, map: &SourceMap) -> String {
        let file = map.get(self.span.file);
        let (line, col) = file.line_col(self.span.start);
        format!(
            "{}:{}:{}: {}: {}",
            file.path().display(),
            line,
            col,
            self.severity,
            self.message,
        )
    }
}

/// Top-level error facade. Each stage's error enum composes via `From`.
/// New variants land as new stages materialize (shim disk-write and the CLI
/// driver compose their own errors at the binary layer; only source-mappable
/// errors join here).
#[derive(Debug)]
pub enum MarainError {
    Lex(LexError),
    Parse(ParseError),
    Emit(EmitError),
}

impl MarainError {
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            Self::Lex(e) => e.to_diagnostic(),
            Self::Parse(e) => e.to_diagnostic(),
            Self::Emit(e) => e.to_diagnostic(),
        }
    }
}

impl From<LexError> for MarainError {
    fn from(e: LexError) -> Self {
        Self::Lex(e)
    }
}

impl From<ParseError> for MarainError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<EmitError> for MarainError {
    fn from(e: EmitError) -> Self {
        Self::Emit(e)
    }
}

impl fmt::Display for MarainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(e) => fmt::Display::fmt(e, f),
            Self::Parse(e) => fmt::Display::fmt(e, f),
            Self::Emit(e) => fmt::Display::fmt(e, f),
        }
    }
}

impl std::error::Error for MarainError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Lex(e) => Some(e),
            Self::Parse(e) => Some(e),
            Self::Emit(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::span::FileId;

    fn one_file(path: &str, text: &str) -> (SourceMap, FileId) {
        let mut map = SourceMap::new();
        let id = map.add(PathBuf::from(path), text.to_string());
        (map, id)
    }

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
    }

    #[test]
    fn diagnostic_error_constructor() {
        let (_, id) = one_file("hello.lat", "dic \"salve\".");
        let d = Diagnostic::error(Span::new(0, 3, id), "unexpected character");
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.message, "unexpected character");
    }

    #[test]
    fn diagnostic_warning_constructor() {
        let (_, id) = one_file("hello.lat", "dic \"salve\".");
        let d = Diagnostic::warning(Span::new(0, 3, id), "be careful");
        assert_eq!(d.severity, Severity::Warning);
    }

    #[test]
    fn render_first_line_first_column() {
        let (map, id) = one_file("hello.lat", "dic \"salve\".");
        let d = Diagnostic::error(Span::new(0, 3, id), "boom");
        assert_eq!(d.render(&map), "hello.lat:1:1: error: boom");
    }

    #[test]
    fn render_uses_span_start_for_position() {
        let (map, id) = one_file("hello.lat", "line1\nline2\nline3");
        let d = Diagnostic::warning(Span::new(6, 11, id), "watch out");
        assert_eq!(d.render(&map), "hello.lat:2:1: warning: watch out");
    }

    #[test]
    fn render_with_subdir_path() {
        let (map, id) = one_file("src/hello.lat", "x");
        let d = Diagnostic::error(Span::new(0, 1, id), "msg");
        assert_eq!(d.render(&map), "src/hello.lat:1:1: error: msg");
    }

    #[test]
    fn render_offset_within_line() {
        let (map, id) = one_file("hello.lat", "ab\ncdef\nghi");
        let d = Diagnostic::error(Span::new(5, 6, id), "huh");
        assert_eq!(d.render(&map), "hello.lat:2:3: error: huh");
    }

    #[test]
    fn marain_error_wraps_lex_via_from() {
        let lex = LexError::UnexpectedChar {
            ch: '?',
            span: Span::new(0, 1, FileId::new(1).expect("nonzero")),
        };
        let m: MarainError = lex.into();
        assert!(matches!(m, MarainError::Lex(_)));
    }

    #[test]
    fn marain_error_to_diagnostic_dispatches_to_lex() {
        let span = Span::new(0, 1, FileId::new(1).expect("nonzero"));
        let m = MarainError::Lex(LexError::UnexpectedChar { ch: '?', span });
        let d = m.to_diagnostic();
        assert_eq!(d.span, span);
        assert!(d.message.contains("unexpected character"));
    }

    #[test]
    fn marain_error_display_delegates() {
        let lex = LexError::UnterminatedString {
            span: Span::new(0, 1, FileId::new(1).expect("nonzero")),
        };
        let m = MarainError::Lex(lex);
        assert_eq!(m.to_string(), "unterminated string literal");
    }

    #[test]
    fn marain_error_source_returns_lex() {
        use std::error::Error;
        let m = MarainError::Lex(LexError::UnterminatedString {
            span: Span::new(0, 1, FileId::new(1).expect("nonzero")),
        });
        assert!(m.source().is_some());
    }

    #[test]
    fn marain_error_wraps_parse_via_from() {
        use crate::token::TokenKind;
        let pe = ParseError::ExpectedExpression {
            found: TokenKind::Period,
            span: Span::new(0, 1, FileId::new(1).expect("nonzero")),
        };
        let m: MarainError = pe.into();
        assert!(matches!(m, MarainError::Parse(_)));
    }

    #[test]
    fn marain_error_to_diagnostic_dispatches_to_parse() {
        use crate::token::TokenKind;
        let span = Span::new(0, 1, FileId::new(1).expect("nonzero"));
        let m = MarainError::Parse(ParseError::ExpectedExpression {
            found: TokenKind::Period,
            span,
        });
        let d = m.to_diagnostic();
        assert_eq!(d.span, span);
        assert!(d.message.contains("expected expression"));
    }

    #[test]
    fn marain_error_display_delegates_for_parse() {
        use crate::token::TokenKind;
        let m = MarainError::Parse(ParseError::ExpectedExpression {
            found: TokenKind::Period,
            span: Span::new(0, 1, FileId::new(1).expect("nonzero")),
        });
        assert!(m.to_string().contains("expected expression"));
    }

    #[test]
    fn marain_error_source_returns_parse() {
        use crate::token::TokenKind;
        use std::error::Error;
        let m = MarainError::Parse(ParseError::ExpectedExpression {
            found: TokenKind::Period,
            span: Span::new(0, 1, FileId::new(1).expect("nonzero")),
        });
        assert!(m.source().is_some());
    }

    #[test]
    fn marain_error_wraps_emit_via_from() {
        let ee = EmitError::UnescapableRustKeyword {
            name: "self".to_string(),
            span: Span::new(0, 4, FileId::new(1).expect("nonzero")),
        };
        let m: MarainError = ee.into();
        assert!(matches!(m, MarainError::Emit(_)));
    }

    #[test]
    fn marain_error_to_diagnostic_dispatches_to_emit() {
        let span = Span::new(0, 4, FileId::new(1).expect("nonzero"));
        let m = MarainError::Emit(EmitError::UnescapableRustKeyword {
            name: "self".to_string(),
            span,
        });
        let d = m.to_diagnostic();
        assert_eq!(d.span, span);
        assert!(d.message.contains("`self`"));
    }

    #[test]
    fn marain_error_source_returns_emit() {
        use std::error::Error;
        let m = MarainError::Emit(EmitError::UnescapableRustKeyword {
            name: "self".to_string(),
            span: Span::new(0, 4, FileId::new(1).expect("nonzero")),
        });
        assert!(m.source().is_some());
    }
}
