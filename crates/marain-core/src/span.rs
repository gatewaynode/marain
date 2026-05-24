//! Source position primitives.
//!
//! [`FileId`] names one source file registered in a [`crate::source::SourceMap`];
//! [`Span`] names a byte range within one file.

use std::num::NonZeroU32;

/// Identifier for a file registered in a [`crate::source::SourceMap`].
///
/// The `NonZeroU32` representation lets `Option<FileId>` stay 4 bytes via
/// the standard niche optimization.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FileId(NonZeroU32);

impl FileId {
    pub(crate) fn new(raw: u32) -> Option<Self> {
        NonZeroU32::new(raw).map(Self)
    }

    pub(crate) fn raw(self) -> u32 {
        self.0.get()
    }
}

/// Half-open byte range `[start, end)` within a single source file.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub file: FileId,
}

impl Span {
    pub fn new(start: u32, end: u32, file: FileId) -> Self {
        debug_assert!(start <= end, "span start must not exceed end");
        Self { start, end, file }
    }

    /// Smallest span containing both inputs. Cross-file joins are a compiler
    /// bug, caught by `debug_assert` in debug builds.
    pub fn join(self, other: Self) -> Self {
        debug_assert_eq!(self.file, other.file, "cannot join spans across files");
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            file: self.file,
        }
    }

    pub fn len(self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fid(n: u32) -> FileId {
        FileId::new(n).expect("test file id")
    }

    #[test]
    fn file_id_rejects_zero() {
        assert!(FileId::new(0).is_none());
        assert!(FileId::new(1).is_some());
    }

    #[test]
    fn option_file_id_is_niche_optimized() {
        assert_eq!(
            std::mem::size_of::<Option<FileId>>(),
            std::mem::size_of::<FileId>(),
        );
        assert_eq!(std::mem::size_of::<FileId>(), 4);
    }

    #[test]
    fn span_join_disjoint() {
        let a = Span::new(2, 5, fid(1));
        let b = Span::new(10, 12, fid(1));
        assert_eq!(a.join(b), Span::new(2, 12, fid(1)));
    }

    #[test]
    fn span_join_overlapping() {
        let a = Span::new(2, 8, fid(1));
        let b = Span::new(5, 12, fid(1));
        assert_eq!(a.join(b), Span::new(2, 12, fid(1)));
    }

    #[test]
    fn span_join_is_commutative() {
        let a = Span::new(2, 5, fid(1));
        let b = Span::new(10, 12, fid(1));
        assert_eq!(a.join(b), b.join(a));
    }

    #[test]
    fn span_len_and_empty() {
        assert_eq!(Span::new(3, 8, fid(1)).len(), 5);
        assert!(Span::new(4, 4, fid(1)).is_empty());
        assert!(!Span::new(4, 5, fid(1)).is_empty());
    }

    #[test]
    #[should_panic(expected = "cannot join spans across files")]
    fn span_join_across_files_panics_in_debug() {
        let _ = Span::new(0, 1, fid(1)).join(Span::new(0, 1, fid(2)));
    }
}
