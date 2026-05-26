//! Comment scanner.
//!
//! Marain supports `//` line comments (PRD §4.12). The lexer's main loop
//! dispatches `/` bytes here. The scanner consumes characters until the
//! next newline (exclusive) and emits no token — comments are layout, not
//! syntax. The `\n` itself is left for the lexer's normal end-of-line
//! handling so the line counter stays accurate.
//!
//! Block comments (`/* */`) are reserved syntax in v0.2 but unimplemented;
//! the lexer surfaces `LexError::BlockCommentsDeferred` directly, without
//! invoking this scanner.

use super::cursor::Cursor;

/// Consume bytes up to (but not including) the next `\n` or EOF.
///
/// Assumes the opening `//` has already been consumed by the caller.
pub(super) fn scan_line_comment(cursor: &mut Cursor<'_>) {
    cursor.advance_while(|b| b != b'\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumes_empty_comment() {
        // Caller has already consumed `//`; the cursor sits at the `\n`.
        let mut c = Cursor::new("\n");
        scan_line_comment(&mut c);
        assert_eq!(c.peek(), Some(b'\n'));
    }

    #[test]
    fn consumes_up_to_but_not_including_newline() {
        let mut c = Cursor::new("foo bar\nnext line");
        scan_line_comment(&mut c);
        assert_eq!(c.peek(), Some(b'\n'));
    }

    #[test]
    fn consumes_to_eof_without_trailing_newline() {
        let mut c = Cursor::new("the very last word");
        scan_line_comment(&mut c);
        assert_eq!(c.peek(), None);
    }

    #[test]
    fn leaves_newline_for_caller() {
        // Critical for the lexer's line-counting and at-line-start machinery.
        let mut c = Cursor::new("text\n");
        scan_line_comment(&mut c);
        assert_eq!(c.peek(), Some(b'\n'));
    }

    #[test]
    fn does_not_lookback_into_marain_syntax() {
        // The body of a comment contains arbitrary text that would otherwise
        // tokenize as Marain. The scanner does not interpret it.
        let mut c = Cursor::new("@x ^y \"quote\" sit .\n");
        scan_line_comment(&mut c);
        assert_eq!(c.peek(), Some(b'\n'));
    }

    #[test]
    fn handles_utf8_in_comment_body() {
        // Latin prose with macrons inside a comment must not crash the
        // scanner; the comment is byte-oriented and only looks at `\n`.
        let mut c = Cursor::new("nota: macrōn-bearing prose\nrest");
        scan_line_comment(&mut c);
        assert_eq!(c.peek(), Some(b'\n'));
    }

    #[test]
    fn consecutive_double_slashes_stay_in_comment() {
        // `// // double` — the second `//` is just text inside the comment.
        let mut c = Cursor::new("// // double\nafter");
        scan_line_comment(&mut c);
        assert_eq!(c.peek(), Some(b'\n'));
    }
}
