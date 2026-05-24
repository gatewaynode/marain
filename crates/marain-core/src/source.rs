//! Source-file storage and registry.
//!
//! [`SourceFile`] owns one file's bytes and its eagerly-computed line index;
//! [`SourceMap`] is the per-compilation registry that mints [`FileId`]s.

use std::path::PathBuf;

use crate::span::FileId;

pub struct SourceFile {
    id: FileId,
    path: PathBuf,
    text: String,
    line_starts: Vec<u32>,
}

impl SourceFile {
    fn new(id: FileId, path: PathBuf, text: String) -> Self {
        let line_starts = compute_line_starts(&text);
        Self {
            id,
            path,
            text,
            line_starts,
        }
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Resolve a byte offset to a 1-indexed (line, column) pair.
    ///
    /// Column is measured in bytes from the start of the line. Marain
    /// identifiers are ASCII (PRD §4.9), but UTF-8 inside string literals
    /// yields multi-byte columns there.
    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line = (line_idx as u32) + 1;
        let col = (offset - self.line_starts[line_idx]) + 1;
        (line, col)
    }
}

fn compute_line_starts(text: &str) -> Vec<u32> {
    std::iter::once(0u32)
        .chain(
            text.bytes()
                .enumerate()
                .filter_map(|(i, b)| (b == b'\n').then_some((i as u32) + 1)),
        )
        .collect()
}

#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, path: PathBuf, text: String) -> FileId {
        let raw = u32::try_from(self.files.len() + 1)
            .expect("SourceMap holds more files than u32 can index");
        let id = FileId::new(raw).expect("len + 1 is always nonzero");
        self.files.push(SourceFile::new(id, path, text));
        id
    }

    pub fn get(&self, id: FileId) -> &SourceFile {
        &self.files[(id.raw() - 1) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(text: &str) -> SourceFile {
        SourceFile::new(
            FileId::new(1).expect("nonzero"),
            PathBuf::from("test.lat"),
            text.to_string(),
        )
    }

    #[test]
    fn line_starts_empty_file() {
        assert_eq!(compute_line_starts(""), vec![0]);
    }

    #[test]
    fn line_starts_single_line_no_newline() {
        assert_eq!(compute_line_starts("abc"), vec![0]);
    }

    #[test]
    fn line_starts_multiple_lines() {
        assert_eq!(compute_line_starts("a\nbb\nccc"), vec![0, 2, 5]);
    }

    #[test]
    fn line_starts_trailing_newline() {
        // Trailing newline implies an empty final line, which gets its own start.
        assert_eq!(compute_line_starts("a\n"), vec![0, 2]);
    }

    #[test]
    fn line_col_first_char() {
        let f = mk("hello\nworld");
        assert_eq!(f.line_col(0), (1, 1));
    }

    #[test]
    fn line_col_within_first_line() {
        let f = mk("hello\nworld");
        assert_eq!(f.line_col(3), (1, 4));
    }

    #[test]
    fn line_col_at_newline_byte() {
        let f = mk("hello\nworld");
        assert_eq!(f.line_col(5), (1, 6));
    }

    #[test]
    fn line_col_start_of_second_line() {
        let f = mk("hello\nworld");
        assert_eq!(f.line_col(6), (2, 1));
    }

    #[test]
    fn line_col_within_second_line() {
        let f = mk("hello\nworld");
        assert_eq!(f.line_col(9), (2, 4));
    }

    #[test]
    fn source_map_round_trip() {
        let mut map = SourceMap::new();
        let a = map.add(PathBuf::from("a.lat"), "alpha".to_string());
        let b = map.add(PathBuf::from("b.lat"), "beta".to_string());
        assert_ne!(a, b);
        assert_eq!(map.get(a).text(), "alpha");
        assert_eq!(map.get(b).text(), "beta");
        assert_eq!(map.get(a).path(), &PathBuf::from("a.lat"));
    }

    #[test]
    fn source_map_first_id_is_one() {
        let mut map = SourceMap::new();
        let id = map.add(PathBuf::from("x.lat"), String::new());
        assert_eq!(id.raw(), 1);
    }
}
