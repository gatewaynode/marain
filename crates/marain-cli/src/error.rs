//! [`DriverError`] — the CLI's top-level error type. Composes the three
//! error families the binary actually surfaces:
//!
//! * source-mappable compiler errors via [`MarainError`] + [`SourceMap`],
//!   rendered as `path:line:col: error: msg` through [`Diagnostic::render`];
//! * filesystem errors from writing the cargo shim via [`ShimError`];
//! * other I/O errors (reading source, canonicalizing paths, spawning cargo)
//!   carried as `io::Error` with a free-text operation context;
//! * a cargo non-zero exit (the user program failed or rustc failed).
//!
//! Reporting goes to stderr via [`DriverError::report`]; the binary then
//! exits with code 1 on any [`DriverError`].
//!
//! Per ARCHITECTURE.md §5 [`MarainError`] only carries source-mappable
//! variants; the filesystem / process boundary is composed here, in the CLI.

use std::fmt;
use std::io;

use marain_core::error::MarainError;
use marain_core::shim::ShimError;
use marain_core::source::SourceMap;

pub enum DriverError {
    /// A source-mappable error (lex / parse / emit). Carries the [`SourceMap`]
    /// so the diagnostic can be rendered with the right `path:line:col`.
    Source { error: MarainError, map: SourceMap },
    /// Filesystem error from writing the shim project.
    Shim(ShimError),
    /// Other I/O error (read source, canonicalize, cargo spawn).
    /// `context` is a short operation description such as
    /// `"failed to read hello.lat"`.
    Io { context: String, source: io::Error },
    /// Cargo exited non-zero. `exit_code` is `None` when cargo was killed by
    /// a signal; otherwise carries the cargo exit code, which the binary
    /// surfaces verbatim to the user (`marain run` proxies the user
    /// program's status).
    Cargo { exit_code: Option<i32> },
}

impl DriverError {
    /// Bind a [`MarainError`] to the [`SourceMap`] it points into so its
    /// diagnostic can be rendered.
    pub fn from_source(error: MarainError, map: SourceMap) -> Self {
        Self::Source { error, map }
    }

    /// Wrap an [`io::Error`] with a human-readable operation context, e.g.
    /// `"failed to read hello.lat"`.
    pub fn from_io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    /// Print this error to stderr in the canonical shape for its kind.
    ///
    /// * `Source` errors render via [`Diagnostic::render`] using the
    ///   [`SourceMap`] held alongside the error.
    /// * All other errors render with a `marain: error: ...` prefix
    ///   (mirroring `cargo:` / `rustc:` convention) so the user can tell
    ///   driver-layer errors apart from source-level diagnostics.
    pub fn report(&self) {
        match self {
            Self::Source { error, map } => {
                eprintln!("{}", error.to_diagnostic().render(map));
            }
            Self::Shim(e) => {
                eprintln!("marain: error: {e}");
            }
            Self::Io { context, source } => {
                eprintln!("marain: error: {context}: {source}");
            }
            Self::Cargo {
                exit_code: Some(code),
            } => {
                eprintln!("marain: error: cargo exited with status {code}");
            }
            Self::Cargo { exit_code: None } => {
                eprintln!("marain: error: cargo terminated by signal");
            }
        }
    }
}

impl From<ShimError> for DriverError {
    fn from(e: ShimError) -> Self {
        Self::Shim(e)
    }
}

// Hand-rolled `Debug` because [`SourceMap`] doesn't derive `Debug` and we
// don't want to take that as a Stage 1 surface change.
impl fmt::Debug for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source { error, .. } => f
                .debug_struct("Source")
                .field("error", error)
                .field("map", &"<SourceMap>")
                .finish(),
            Self::Shim(e) => f.debug_tuple("Shim").field(e).finish(),
            Self::Io { context, source } => f
                .debug_struct("Io")
                .field("context", context)
                .field("source", source)
                .finish(),
            Self::Cargo { exit_code } => f
                .debug_struct("Cargo")
                .field("exit_code", exit_code)
                .finish(),
        }
    }
}

impl fmt::Display for DriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source { error, .. } => fmt::Display::fmt(error, f),
            Self::Shim(e) => fmt::Display::fmt(e, f),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::Cargo {
                exit_code: Some(code),
            } => write!(f, "cargo exited with status {code}"),
            Self::Cargo { exit_code: None } => write!(f, "cargo terminated by signal"),
        }
    }
}

impl std::error::Error for DriverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source { error, .. } => Some(error),
            Self::Shim(e) => Some(e),
            Self::Io { source, .. } => Some(source),
            Self::Cargo { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use marain_core::lexer::LexError;
    use marain_core::shim::ShimError;
    use marain_core::span::{FileId, Span};

    use super::*;

    fn one_file_map(path: &str, text: &str) -> (SourceMap, FileId) {
        let mut map = SourceMap::new();
        let id = map.add(PathBuf::from(path), text.to_string());
        (map, id)
    }

    #[test]
    fn from_io_attaches_context() {
        let e = DriverError::from_io("failed to read hello.lat", io::Error::other("disk down"));
        match e {
            DriverError::Io { context, source } => {
                assert_eq!(context, "failed to read hello.lat");
                assert!(source.to_string().contains("disk down"));
            }
            _ => panic!("expected Io"),
        }
    }

    #[test]
    fn from_source_binds_map() {
        let (map, id) = one_file_map("hello.lat", "dic \"x\".");
        let lex = LexError::UnexpectedChar {
            ch: '?',
            span: Span::new(0, 1, id),
        };
        let e = DriverError::from_source(lex.into(), map);
        assert!(matches!(e, DriverError::Source { .. }));
    }

    #[test]
    fn from_impl_for_shim_error() {
        let shim = ShimError::WriteFile {
            path: PathBuf::from("/x"),
            source: io::Error::other("permission denied"),
        };
        let e: DriverError = shim.into();
        assert!(matches!(e, DriverError::Shim(_)));
    }

    #[test]
    fn display_for_io_includes_context_and_source() {
        let e = DriverError::from_io("ctx", io::Error::other("oops"));
        let s = e.to_string();
        assert!(s.contains("ctx"));
        assert!(s.contains("oops"));
    }

    #[test]
    fn display_for_cargo_with_exit_code() {
        let e = DriverError::Cargo {
            exit_code: Some(101),
        };
        assert_eq!(e.to_string(), "cargo exited with status 101");
    }

    #[test]
    fn display_for_cargo_signal() {
        let e = DriverError::Cargo { exit_code: None };
        assert_eq!(e.to_string(), "cargo terminated by signal");
    }

    #[test]
    fn display_for_source_delegates_to_marain_error() {
        let (map, id) = one_file_map("hello.lat", "x");
        let lex = LexError::UnterminatedString {
            span: Span::new(0, 1, id),
        };
        let e = DriverError::from_source(lex.into(), map);
        assert_eq!(e.to_string(), "unterminated string literal");
    }

    #[test]
    fn source_returns_inner_for_each_variant() {
        use std::error::Error;

        let (map, id) = one_file_map("h.lat", "x");
        let lex = LexError::UnterminatedString {
            span: Span::new(0, 1, id),
        };
        let de = DriverError::from_source(lex.into(), map);
        assert!(de.source().is_some());

        let shim = ShimError::WriteFile {
            path: PathBuf::from("/x"),
            source: io::Error::other("bad"),
        };
        let de: DriverError = shim.into();
        assert!(de.source().is_some());

        let de = DriverError::from_io("ctx", io::Error::other("oops"));
        assert!(de.source().is_some());

        let de = DriverError::Cargo { exit_code: Some(1) };
        assert!(de.source().is_none());
    }

    #[test]
    fn debug_does_not_panic() {
        // Just exercise the manual `Debug` impl so a future refactor doesn't
        // silently lose a variant arm.
        let (map, id) = one_file_map("h.lat", "x");
        let lex = LexError::UnterminatedString {
            span: Span::new(0, 1, id),
        };
        let _ = format!("{:?}", DriverError::from_source(lex.into(), map));
        let _ = format!(
            "{:?}",
            DriverError::Shim(ShimError::WriteFile {
                path: PathBuf::from("/x"),
                source: io::Error::other("bad"),
            }),
        );
        let _ = format!(
            "{:?}",
            DriverError::from_io("ctx", io::Error::other("oops")),
        );
        let _ = format!("{:?}", DriverError::Cargo { exit_code: Some(1) });
        let _ = format!("{:?}", DriverError::Cargo { exit_code: None });
    }
}
