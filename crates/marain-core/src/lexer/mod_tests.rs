//! 553 LOC, exceeds 500 target: sibling test file for `lexer/mod.rs`. All
//! tests share the `lex_str` / `lex_err` helpers and exercise one cohesive
//! surface — the full lex pipeline (indent / brackets / sigils / keywords /
//! strings / f-strings / numbers / line comments / `<>` lookalike). Splitting
//! by R-round would force callers to chase shared helpers across files for no
//! gain. `super` resolves to the `lexer` module, so `use super::*` and
//! `use keywords::Keyword` behave exactly as they did inline.

use std::path::PathBuf;

use super::*;
use crate::source::SourceMap;
use crate::token::Sigil;
use keywords::Keyword;

fn lex_str(text: &str) -> Vec<TokenKind> {
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("test.lat"), text.to_string());
    let tokens = lex(map.get(id)).expect("lex failed");
    tokens.into_iter().map(|t| t.kind).collect()
}

fn lex_err(text: &str) -> LexError {
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("test.lat"), text.to_string());
    lex(map.get(id)).expect_err("expected lex error")
}

#[test]
fn hello_world() {
    let toks = lex_str("dic \"salve, munde\".\n");
    assert_eq!(
        toks,
        vec![
            TokenKind::Keyword(Keyword::Dic),
            TokenKind::StringLit("salve, munde".to_string()),
            TokenKind::Period,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn binding_with_sigils() {
    let toks = lex_str("sit ^x est 5.\n");
    assert_eq!(
        toks,
        vec![
            TokenKind::Keyword(Keyword::Sit),
            TokenKind::SigiledIdent {
                sigil: Sigil::Immutable,
                name: "x".to_string(),
            },
            TokenKind::Keyword(Keyword::Est),
            TokenKind::IntegerLit(5),
            TokenKind::Period,
            TokenKind::Eof,
        ],
    );
}

#[test]
fn indented_block() {
    let toks = lex_str("functio foo:\n    dic \"a\".\n    dic \"b\".\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Functio),
        TokenKind::PlainIdent("foo".to_string()),
        TokenKind::Colon,
        TokenKind::Indent,
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::StringLit("a".to_string()),
        TokenKind::Period,
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::StringLit("b".to_string()),
        TokenKind::Period,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn nested_indents_emit_cascading_dedents() {
    let toks = lex_str("a:\n    b:\n        c.\n");
    let expected = vec![
        TokenKind::PlainIdent("a".to_string()),
        TokenKind::Colon,
        TokenKind::Indent,
        TokenKind::PlainIdent("b".to_string()),
        TokenKind::Colon,
        TokenKind::Indent,
        TokenKind::PlainIdent("c".to_string()),
        TokenKind::Period,
        TokenKind::Dedent,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn blank_lines_skipped() {
    let toks = lex_str("dic \"x\".\n\n\ndic \"y\".\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::StringLit("x".to_string()),
        TokenKind::Period,
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::StringLit("y".to_string()),
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn brackets_suppress_indent() {
    // A continuation line indented inside parens emits no INDENT.
    let toks = lex_str("dic(\n    \"a\",\n).\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::LParen,
        TokenKind::StringLit("a".to_string()),
        TokenKind::Comma,
        TokenKind::RParen,
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn tab_in_indentation_is_error() {
    let err = lex_err("\tdic \"x\".\n");
    assert!(matches!(err, LexError::TabCharacter { .. }));
}

#[test]
fn tab_mid_line_is_error() {
    let err = lex_err("dic\t\"x\".\n");
    assert!(matches!(err, LexError::TabCharacter { .. }));
}

#[test]
fn unexpected_char_is_error() {
    let err = lex_err("?");
    assert!(matches!(err, LexError::UnexpectedChar { ch: '?', .. }));
}

#[test]
fn unterminated_string_is_error() {
    let err = lex_err("dic \"unfinished\n");
    assert!(matches!(err, LexError::UnterminatedString { .. }));
}

#[test]
fn double_colon_one_token() {
    let toks = lex_str("a::b.\n");
    let expected = vec![
        TokenKind::PlainIdent("a".to_string()),
        TokenKind::DoubleColon,
        TokenKind::PlainIdent("b".to_string()),
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn bang_is_separate_token() {
    let toks = lex_str("dbg!(@x).\n");
    let expected = vec![
        TokenKind::PlainIdent("dbg".to_string()),
        TokenKind::Bang,
        TokenKind::LParen,
        TokenKind::SigiledIdent {
            sigil: Sigil::Mutable,
            name: "x".to_string(),
        },
        TokenKind::RParen,
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn integer_with_underscore() {
    let toks = lex_str("sit ^x est 1_000.\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Sit),
        TokenKind::SigiledIdent {
            sigil: Sigil::Immutable,
            name: "x".to_string(),
        },
        TokenKind::Keyword(Keyword::Est),
        TokenKind::IntegerLit(1000),
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn empty_source_emits_only_eof() {
    let toks = lex_str("");
    assert_eq!(toks, vec![TokenKind::Eof]);
}

#[test]
fn whitespace_only_source_emits_only_eof() {
    let toks = lex_str("   \n  \n");
    assert_eq!(toks, vec![TokenKind::Eof]);
}

#[test]
fn detonatio_keyword_recognized() {
    let toks = lex_str("DETONATIO!(\"oops\").\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Detonatio),
        TokenKind::Bang,
        TokenKind::LParen,
        TokenKind::StringLit("oops".to_string()),
        TokenKind::RParen,
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn escape_sequences_in_string() {
    let toks = lex_str("dic \"a\\nb\\tc\".\n");
    assert!(matches!(&toks[1], TokenKind::StringLit(s) if s == "a\nb\tc"));
}

#[test]
fn no_trailing_newline_still_drains_indent() {
    // Multi-statement with no trailing newline; final dedent must still emit.
    let toks = lex_str("a:\n    b.");
    let expected = vec![
        TokenKind::PlainIdent("a".to_string()),
        TokenKind::Colon,
        TokenKind::Indent,
        TokenKind::PlainIdent("b".to_string()),
        TokenKind::Period,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn inconsistent_indent_is_error() {
    // Indent to 4, then to 2 (which is on neither stack frame).
    let err = lex_err("a:\n    b.\n  c.\n");
    assert!(matches!(err, LexError::InconsistentIndent { .. }));
}

#[test]
fn multiple_statements_on_one_line() {
    let toks = lex_str("dic ^x. dic ^y.\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::SigiledIdent {
            sigil: Sigil::Immutable,
            name: "x".to_string(),
        },
        TokenKind::Period,
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::SigiledIdent {
            sigil: Sigil::Immutable,
            name: "y".to_string(),
        },
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

// --- R9: line comments (PRD §4.12) ---

#[test]
fn trailing_comment_after_statement() {
    let toks = lex_str("sit ^x est 5. // note\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Sit),
        TokenKind::SigiledIdent {
            sigil: Sigil::Immutable,
            name: "x".to_string(),
        },
        TokenKind::Keyword(Keyword::Est),
        TokenKind::IntegerLit(5),
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn standalone_comment_at_top_of_file() {
    let toks = lex_str("// preamble\nsit ^x est 5.\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Sit),
        TokenKind::SigiledIdent {
            sigil: Sigil::Immutable,
            name: "x".to_string(),
        },
        TokenKind::Keyword(Keyword::Est),
        TokenKind::IntegerLit(5),
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn comment_only_file_emits_only_eof() {
    let toks = lex_str("// just a comment\n");
    assert_eq!(toks, vec![TokenKind::Eof]);
}

#[test]
fn comment_only_file_no_trailing_newline() {
    let toks = lex_str("// no newline at end");
    assert_eq!(toks, vec![TokenKind::Eof]);
}

#[test]
fn consecutive_comment_only_lines_no_indent_change() {
    let toks = lex_str("// one\n// two\n// three\nsit ^x est 5.\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Sit),
        TokenKind::SigiledIdent {
            sigil: Sigil::Immutable,
            name: "x".to_string(),
        },
        TokenKind::Keyword(Keyword::Est),
        TokenKind::IntegerLit(5),
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn comment_inside_indented_block_does_not_dedent() {
    let toks = lex_str("a:\n    b.\n    // inside\n    c.\n");
    let expected = vec![
        TokenKind::PlainIdent("a".to_string()),
        TokenKind::Colon,
        TokenKind::Indent,
        TokenKind::PlainIdent("b".to_string()),
        TokenKind::Period,
        TokenKind::PlainIdent("c".to_string()),
        TokenKind::Period,
        TokenKind::Dedent,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn block_comment_is_deferred_error() {
    let err = lex_err("/* block */");
    match err {
        LexError::BlockCommentsDeferred { span } => {
            // span covers exactly the two-byte `/*`
            assert_eq!(span.start, 0);
            assert_eq!(span.end, 2);
        }
        other => panic!("expected BlockCommentsDeferred, got {other:?}"),
    }
}

#[test]
fn block_comment_message_mentions_alternative() {
    let err = lex_err("/* block */");
    let msg = err.message();
    assert!(
        msg.contains("//"),
        "message should suggest `//`; got: {msg}"
    );
    assert!(
        msg.contains("reserved"),
        "message should call out reserved status; got: {msg}",
    );
}

#[test]
fn open_angle_is_generics_lookalike() {
    let err = lex_err("dat Agmen<T>.\n");
    match err {
        LexError::GenericsLookalike { ch, span } => {
            assert_eq!(ch, '<');
            assert_eq!(span.end - span.start, 1);
        }
        other => panic!("expected GenericsLookalike, got {other:?}"),
    }
}

#[test]
fn close_angle_is_generics_lookalike() {
    let err = lex_err("functio foo() dat Sermo>.\n");
    match err {
        LexError::GenericsLookalike { ch, .. } => assert_eq!(ch, '>'),
        other => panic!("expected GenericsLookalike, got {other:?}"),
    }
}

#[test]
fn generics_lookalike_message_mentions_generics_and_alternative() {
    let err = lex_err("dat Agmen<T>.\n");
    let msg = err.message();
    assert!(
        msg.contains("generics"),
        "message should mention generics; got: {msg}"
    );
    assert!(
        msg.contains("v0.3"),
        "message should cite v0.3 deferral; got: {msg}"
    );
}

#[test]
fn bare_slash_is_unexpected_char() {
    let err = lex_err("/foo");
    match err {
        LexError::UnexpectedChar { ch, .. } => assert_eq!(ch, '/'),
        other => panic!("expected UnexpectedChar, got {other:?}"),
    }
}

// --- R14: range tokens (`..` / `..=`) ---

#[test]
fn dot_dot_is_one_token() {
    let toks = lex_str("0..10");
    let expected = vec![
        TokenKind::IntegerLit(0),
        TokenKind::DotDot,
        TokenKind::IntegerLit(10),
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn dot_dot_eq_is_one_token() {
    let toks = lex_str("0..=10");
    let expected = vec![
        TokenKind::IntegerLit(0),
        TokenKind::DotDotEq,
        TokenKind::IntegerLit(10),
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn period_unchanged_by_dotdot_dispatcher() {
    // Statement-terminator `.` (single dot) still lexes as Period.
    let toks = lex_str("dic ^x.\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::SigiledIdent {
            sigil: Sigil::Immutable,
            name: "x".to_string(),
        },
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn three_dots_lex_as_dotdot_then_period() {
    // `...` is not its own token; greedy `..` wins, third `.` is Period.
    let toks = lex_str("...");
    let expected = vec![TokenKind::DotDot, TokenKind::Period, TokenKind::Eof];
    assert_eq!(toks, expected);
}

#[test]
fn dot_dot_with_trailing_period() {
    // `0..10.` (range used as macro arg) — IntegerLit, DotDot, IntegerLit, Period.
    let toks = lex_str("dic 0..10.\n");
    let expected = vec![
        TokenKind::Keyword(Keyword::Dic),
        TokenKind::IntegerLit(0),
        TokenKind::DotDot,
        TokenKind::IntegerLit(10),
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

// --- R17: f-string prefix dispatch (`f"`) ---

#[test]
fn fstring_prefix_produces_fstring_token() {
    use crate::token::FStringSeg;
    let toks = lex_str("dic f\"salve {^nomen}\".\n");
    // The whole f-string is ONE token; its internal `{`/`}` never surface as
    // LBrace/RBrace and never perturb indent/bracket state.
    assert!(
        !toks
            .iter()
            .any(|t| matches!(t, TokenKind::LBrace | TokenKind::RBrace))
    );
    match &toks[1] {
        TokenKind::FStringLit(parts) => {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], FStringSeg::Literal("salve ".to_string()));
            assert!(matches!(
                &parts[1],
                FStringSeg::Interp { sigil: Sigil::Immutable, name, .. } if name == "nomen"
            ));
        }
        other => panic!("expected FStringLit, got {other:?}"),
    }
    assert_eq!(toks[0], TokenKind::Keyword(Keyword::Dic));
    assert_eq!(toks[2], TokenKind::Period);
    assert_eq!(toks[3], TokenKind::Eof);
}

#[test]
fn f_then_space_then_string_is_ident_not_fstring() {
    // `f "x"` (with a space) is a plain identifier `f` followed by a string,
    // NOT an f-string — the prefix requires `f"` with no gap.
    let toks = lex_str("f \"x\".\n");
    let expected = vec![
        TokenKind::PlainIdent("f".to_string()),
        TokenKind::StringLit("x".to_string()),
        TokenKind::Period,
        TokenKind::Eof,
    ];
    assert_eq!(toks, expected);
}

#[test]
fn f_word_starting_keyword_is_unaffected() {
    // `functio` starts with `f` but the byte after `f` is `u`, not `"`.
    let toks = lex_str("functio foo:\n    nihil.\n");
    assert_eq!(toks[0], TokenKind::Keyword(Keyword::Functio));
}

#[test]
fn fstring_empty_hole_is_lex_error() {
    let err = lex_err("dic f\"{}\".\n");
    assert!(matches!(err, LexError::InvalidFStringHole { .. }));
}
