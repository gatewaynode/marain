//! Byte-level cursor over a `&str` source.
//!
//! Provides peek/advance/slice operations and tracks byte offset for span
//! construction. Callers slice on byte boundaries that align with UTF-8
//! char boundaries — true for every ASCII byte, which is what every Marain
//! token boundary is (string literals tolerate UTF-8 bodies between ASCII
//! quote bytes; the special bytes that bound a chunk are always ASCII).

pub(super) struct Cursor<'src> {
    source: &'src str,
    pos: usize,
}

impl<'src> Cursor<'src> {
    pub fn new(source: &'src str) -> Self {
        Self { source, pos: 0 }
    }

    pub fn pos(&self) -> u32 {
        self.pos as u32
    }

    pub fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }

    /// Peek `offset` bytes ahead of the current position. `peek_at(0)` is
    /// equivalent to `peek`. Used by the lexer driver to disambiguate
    /// two-character openers (e.g. `//` vs `/*` vs bare `/`) without a
    /// save-pos / advance / restore dance.
    pub fn peek_at(&self, offset: usize) -> Option<u8> {
        self.source.as_bytes().get(self.pos + offset).copied()
    }

    /// Consume one byte if present.
    pub fn advance(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    /// Consume bytes while `f` returns true; return (start, end) byte offsets.
    pub fn advance_while<F: Fn(u8) -> bool>(&mut self, f: F) -> (u32, u32) {
        let start = self.pos as u32;
        while let Some(b) = self.peek() {
            if !f(b) {
                break;
            }
            self.pos += 1;
        }
        (start, self.pos as u32)
    }

    /// Slice source by absolute byte offsets. Boundaries must align with
    /// UTF-8 char starts.
    pub fn slice(&self, start: u32, end: u32) -> &'src str {
        &self.source[start as usize..end as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_peeks_none() {
        let c = Cursor::new("");
        assert_eq!(c.peek(), None);
    }

    #[test]
    fn advance_returns_byte_and_progresses() {
        let mut c = Cursor::new("ab");
        assert_eq!(c.peek(), Some(b'a'));
        assert_eq!(c.advance(), Some(b'a'));
        assert_eq!(c.peek(), Some(b'b'));
        assert_eq!(c.pos(), 1);
    }

    #[test]
    fn advance_past_eof_is_none() {
        let mut c = Cursor::new("a");
        assert_eq!(c.advance(), Some(b'a'));
        assert_eq!(c.advance(), None);
        assert!(c.peek().is_none());
    }

    #[test]
    fn advance_while_collects_range_and_stops() {
        let mut c = Cursor::new("abc123def");
        let (s, e) = c.advance_while(|b| b.is_ascii_alphabetic());
        assert_eq!((s, e), (0, 3));
        assert_eq!(c.slice(s, e), "abc");
        assert_eq!(c.peek(), Some(b'1'));
    }

    #[test]
    fn advance_while_empty_match_does_not_progress() {
        let mut c = Cursor::new("123");
        let (s, e) = c.advance_while(|b| b.is_ascii_alphabetic());
        assert_eq!((s, e), (0, 0));
        assert_eq!(c.peek(), Some(b'1'));
    }

    #[test]
    fn slice_at_ascii_boundary_in_utf8_source() {
        // 'á' is two bytes (0xc3 0xa1); slicing the surrounding ASCII quotes
        // is valid.
        let c = Cursor::new("\"sálve\"");
        assert_eq!(c.slice(0, 1), "\"");
    }

    #[test]
    fn peek_at_zero_matches_peek() {
        let c = Cursor::new("abc");
        assert_eq!(c.peek_at(0), c.peek());
    }

    #[test]
    fn peek_at_offset_looks_ahead_without_advancing() {
        let c = Cursor::new("abc");
        assert_eq!(c.peek_at(1), Some(b'b'));
        assert_eq!(c.peek_at(2), Some(b'c'));
        // pos unchanged
        assert_eq!(c.pos(), 0);
        assert_eq!(c.peek(), Some(b'a'));
    }

    #[test]
    fn peek_at_past_end_is_none() {
        let c = Cursor::new("ab");
        assert_eq!(c.peek_at(2), None);
        assert_eq!(c.peek_at(99), None);
    }
}
