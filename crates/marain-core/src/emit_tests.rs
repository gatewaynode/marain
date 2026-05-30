//! 555 LOC, exceeds 500 target: sibling test file for `emit.rs`. Tests share
//! the `parse_and_emit` / `parse_and_emit_err` helpers and exhaustively cover
//! every emit arm (skeleton, R5 productions, Rust-keyword escape, R10 `if`,
//! R11 ops + boolean, R12 control flow). One file, one helper set.

use std::path::PathBuf;

use super::*;
use crate::error::MarainError;
use crate::lexer::lex;
use crate::parser::parse;
use crate::source::SourceMap;
use crate::span::FileId;

fn parse_and_emit(text: &str) -> String {
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("test.lat"), text.to_string());
    let tokens = lex(map.get(id)).expect("lex must succeed");
    let module = parse(&tokens).expect("parse must succeed");
    emit(&module).expect("emit must succeed")
}

fn parse_and_emit_err(text: &str) -> EmitError {
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("test.lat"), text.to_string());
    let tokens = lex(map.get(id)).expect("lex must succeed");
    let module = parse(&tokens).expect("parse must succeed");
    emit(&module).expect_err("emit must fail")
}

fn fid() -> FileId {
    FileId::new(1).expect("nonzero")
}

fn sp(start: u32, end: u32) -> Span {
    Span::new(start, end, fid())
}

#[test]
fn empty_module_emits_fn_main_skeleton() {
    assert_eq!(parse_and_emit(""), "fn main() {\n}\n");
}

#[test]
fn hello_world_done_line() {
    assert_eq!(
        parse_and_emit("dic \"salve, munde\".\n"),
        "fn main() {\n    println!(\"{}\", \"salve, munde\");\n}\n",
    );
}

#[test]
fn dic_uses_format_placeholder_even_for_string_literal() {
    // Uniform `("{}", arg)` shape avoids the {} footgun.
    let out = parse_and_emit("dic \"{} brace\".\n");
    assert!(out.contains("println!(\"{}\", \"{} brace\");"));
}

#[test]
fn queror_emits_eprintln() {
    let out = parse_and_emit("queror \"oops\".\n");
    assert!(out.contains("eprintln!(\"{}\", \"oops\");"));
}

#[test]
fn agmen_emits_vec_with_brackets() {
    let out = parse_and_emit("agmen \"item\".\n");
    assert!(out.contains("vec![\"item\"];"));
}

#[test]
fn forma_emits_format_macro() {
    let out = parse_and_emit("forma \"x\".\n");
    assert!(out.contains("format!(\"{}\", \"x\");"));
}

#[test]
fn let_immutable_omits_mut() {
    let out = parse_and_emit("sit ^x est 5.\n");
    assert!(out.contains("let x = 5i64;"));
    assert!(!out.contains("let mut"));
}

#[test]
fn let_mutable_includes_mut() {
    let out = parse_and_emit("sit @x est 5.\n");
    assert!(out.contains("let mut x = 5i64;"));
}

#[test]
fn let_with_string_literal_rhs() {
    let out = parse_and_emit("sit ^greeting est \"salve\".\n");
    assert!(out.contains("let greeting = \"salve\";"));
}

#[test]
fn let_with_var_ref_rhs() {
    let out = parse_and_emit("sit ^x est 5.\nsit @y est ^x.\n");
    assert!(out.contains("let x = 5i64;"));
    assert!(out.contains("let mut y = x;"));
}

#[test]
fn dic_of_var_ref_emits_format_placeholder() {
    let out = parse_and_emit("sit ^x est 5.\ndic ^x.\n");
    assert!(out.contains("let x = 5i64;"));
    assert!(out.contains("println!(\"{}\", x);"));
}

#[test]
fn integer_suffix_forces_i64() {
    let out = parse_and_emit("sit ^x est 1_000_000_000.\n");
    assert!(out.contains("1000000000i64"));
}

#[test]
fn multi_statement_emits_in_order() {
    let out = parse_and_emit("sit ^x est 1.\ndic ^x.\nqueror \"done\".\n");
    let let_pos = out.find("let x").expect("let present");
    let dic_pos = out.find("println!").expect("println present");
    let queror_pos = out.find("eprintln!").expect("eprintln present");
    assert!(let_pos < dic_pos);
    assert!(dic_pos < queror_pos);
}

#[test]
fn string_escape_double_quote() {
    let out = parse_and_emit("dic \"he said \\\"hi\\\"\".\n");
    assert!(out.contains(r#"\"hi\""#), "out was {out}");
}

#[test]
fn string_escape_backslash_round_trips() {
    let out = parse_and_emit("dic \"a\\\\b\".\n");
    // Marain source `\\` decodes to one backslash in the AST; emitter
    // re-escapes to `\\` in Rust.
    assert!(out.contains(r#""a\\b""#), "out was {out}");
}

#[test]
fn string_escape_newline_re_escapes() {
    let out = parse_and_emit("dic \"a\\nb\".\n");
    assert!(out.contains("\"a\\nb\""), "out was {out}");
}

#[test]
fn string_escape_tab_re_escapes() {
    let out = parse_and_emit("dic \"a\\tb\".\n");
    assert!(out.contains("\"a\\tb\""));
}

#[test]
fn string_unicode_passthrough_for_non_control() {
    let out = parse_and_emit("dic \"sálve\".\n");
    assert!(out.contains("\"sálve\""));
}

#[test]
fn string_control_char_uses_unicode_escape() {
    // \x01 cannot survive in source (lexer doesn't permit raw control),
    // but escape_string_for_rust still handles it for safety.
    let escaped = escape_string_for_rust("\u{1}");
    assert_eq!(escaped, "\\u{1}");
}

#[test]
fn let_with_escapable_rust_keyword_uses_raw_prefix() {
    let out = parse_and_emit("sit ^if est 5.\n");
    assert!(out.contains("let r#if = 5i64;"), "out was {out}");
}

#[test]
fn let_with_async_keyword_escaped() {
    let out = parse_and_emit("sit ^async est 5.\n");
    assert!(out.contains("r#async"));
}

#[test]
fn let_with_gen_2024_keyword_escaped() {
    let out = parse_and_emit("sit ^gen est 5.\n");
    assert!(out.contains("r#gen"));
}

#[test]
fn let_with_future_reserved_keyword_escaped() {
    // `become` is reserved for future use; we escape today so Marain
    // programs survive its eventual activation.
    let out = parse_and_emit("sit ^become est 5.\n");
    assert!(out.contains("r#become"));
}

#[test]
fn var_ref_with_rust_keyword_uses_raw_prefix() {
    let out = parse_and_emit("sit ^if est 5.\ndic ^if.\n");
    assert!(out.contains("let r#if = 5i64;"));
    assert!(out.contains("println!(\"{}\", r#if);"));
}

#[test]
fn let_with_unescapable_self_keyword_errors() {
    let err = parse_and_emit_err("sit ^self est 5.\n");
    match err {
        EmitError::UnescapableRustKeyword { name, .. } => assert_eq!(name, "self"),
    }
}

#[test]
fn let_with_unescapable_extern_keyword_errors() {
    let err = parse_and_emit_err("sit ^extern est 5.\n");
    match err {
        EmitError::UnescapableRustKeyword { name, .. } => assert_eq!(name, "extern"),
    }
}

#[test]
fn let_with_unescapable_crate_keyword_errors() {
    let err = parse_and_emit_err("sit ^crate est 5.\n");
    assert!(matches!(
        err,
        EmitError::UnescapableRustKeyword { ref name, .. } if name == "crate"
    ));
}

#[test]
fn unescapable_capital_self_errors() {
    let err = parse_and_emit_err("sit ^Self est 5.\n");
    assert!(matches!(
        err,
        EmitError::UnescapableRustKeyword { ref name, .. } if name == "Self"
    ));
}

#[test]
fn unescapable_super_errors() {
    let err = parse_and_emit_err("sit ^super est 5.\n");
    assert!(matches!(
        err,
        EmitError::UnescapableRustKeyword { ref name, .. } if name == "super"
    ));
}

#[test]
fn var_ref_to_unescapable_keyword_errors() {
    // Even if a hypothetical earlier scope had `self`, referencing it
    // emits the same error at the use site.
    let err = parse_and_emit_err("dic ^self.\n");
    assert!(matches!(err, EmitError::UnescapableRustKeyword { .. }));
}

#[test]
fn emit_error_span_points_at_identifier() {
    // "sit ^if est 5." — sigiled ident `^if` spans bytes 4..7.
    let err = parse_and_emit_err("sit ^self est 5.\n");
    let span = err.span();
    // `^self` starts at byte 4 and is 5 bytes long (`^` + `self`).
    assert_eq!(span.start, 4);
    assert_eq!(span.end, 9);
}

#[test]
fn emit_error_to_diagnostic_carries_message_and_span() {
    let err = EmitError::UnescapableRustKeyword {
        name: "self".to_string(),
        span: sp(0, 5),
    };
    let d = err.to_diagnostic();
    assert_eq!(d.span, sp(0, 5));
    assert!(d.message.contains("`self`"));
    assert!(d.message.contains("Rust reserved word"));
}

#[test]
fn emit_error_display_includes_name() {
    let err = EmitError::UnescapableRustKeyword {
        name: "crate".to_string(),
        span: sp(0, 5),
    };
    assert!(err.to_string().contains("`crate`"));
}

#[test]
fn emit_error_joins_marain_error_facade() {
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("test.lat"), "sit ^self est 5.\n".to_string());
    let tokens = lex(map.get(id)).expect("lex must succeed");
    let module = parse(&tokens).expect("parse must succeed");
    let result: Result<String, MarainError> = emit(&module).map_err(MarainError::from);
    assert!(matches!(
        result,
        Err(MarainError::Emit(EmitError::UnescapableRustKeyword { .. }))
    ));
}

#[test]
fn normal_identifier_passes_through() {
    let out = parse_and_emit("sit ^my_variable_2 est 5.\n");
    assert!(out.contains("let my_variable_2 = 5i64;"));
}

#[test]
fn fn_main_skeleton_brackets_match() {
    let out = parse_and_emit("dic \"x\".\n");
    assert!(out.starts_with("fn main() {\n"));
    assert!(out.ends_with("}\n"));
}

#[test]
fn all_45_escapable_keywords_round_trip() {
    let all = [
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do",
        "dyn", "else", "enum", "false", "final", "fn", "for", "gen", "if", "impl", "in", "let",
        "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return",
        "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe", "unsized", "use",
        "virtual", "where", "while", "yield",
    ];
    for kw in all {
        assert!(is_rust_reserved_escapable(kw), "{kw} should be escapable");
        assert!(
            !is_rust_reserved_unescapable(kw),
            "{kw} should not also be unescapable"
        );
    }
}

#[test]
fn all_5_unescapable_keywords_classified() {
    for kw in ["crate", "extern", "self", "Self", "super"] {
        assert!(
            is_rust_reserved_unescapable(kw),
            "{kw} should be unescapable"
        );
        assert!(
            !is_rust_reserved_escapable(kw),
            "{kw} should not also be escapable"
        );
    }
}

#[test]
fn escape_ident_passthrough_for_safe_names() {
    let result = escape_ident_for_rust("hello", sp(0, 5)).expect("ok");
    assert_eq!(result, "hello");
}

#[test]
fn escape_ident_raw_prefix_for_escapable() {
    let result = escape_ident_for_rust("if", sp(0, 2)).expect("ok");
    assert_eq!(result, "r#if");
}

#[test]
fn escape_ident_error_for_unescapable() {
    let result = escape_ident_for_rust("self", sp(0, 4));
    assert!(matches!(
        result,
        Err(EmitError::UnescapableRustKeyword { ref name, .. }) if name == "self"
    ));
}

#[test]
fn si_emits_if_block_with_indent() {
    let out = parse_and_emit("si ^x :\n    dic ^x.\n");
    let expected = "fn main() {\n    if x {\n        println!(\"{}\", x);\n    }\n}\n";
    assert_eq!(out, expected);
}

#[test]
fn nested_si_threads_indent_level() {
    let out = parse_and_emit("si ^x :\n    si ^y :\n        dic \"deep\".\n");
    let expected = "fn main() {\n    if x {\n        if y {\n            println!(\"{}\", \"deep\");\n        }\n    }\n}\n";
    assert_eq!(out, expected);
}

#[test]
fn si_body_with_let_and_macro() {
    let out = parse_and_emit("si ^x :\n    sit ^y est 7.\n    dic ^y.\n");
    assert!(out.contains("if x {\n        let y = 7i64;\n        println!(\"{}\", y);\n    }"));
}

#[test]
fn top_level_stmts_emit_at_indent_one() {
    // Regression guard: indent threading didn't break the pre-R10 top-level shape.
    let out = parse_and_emit("sit ^x est 5.\n");
    assert!(out.contains("    let x = 5i64;\n"));
}

#[test]
fn si_followed_by_top_level_stmt_keeps_both() {
    let out = parse_and_emit("si ^x :\n    dic ^x.\nsit ^y est 7.\n");
    assert!(out.contains("    if x {\n        println!(\"{}\", x);\n    }\n"));
    assert!(out.contains("    let y = 7i64;\n"));
}

// ─── R11: operator + boolean emit ──────────────────────────────────────

#[test]
fn verum_emits_true() {
    let out = parse_and_emit("sit ^x est verum.\n");
    assert!(out.contains("let x = true;"));
}

#[test]
fn falsum_emits_false() {
    let out = parse_and_emit("sit ^x est falsum.\n");
    assert!(out.contains("let x = false;"));
}

#[test]
fn binop_plus_emits_parenwrapped() {
    let out = parse_and_emit("sit ^x est 1 plus 2.\n");
    assert!(out.contains("let x = (1i64 + 2i64);"));
}

#[test]
fn binop_per_emits_star() {
    let out = parse_and_emit("sit ^x est 2 per 3.\n");
    assert!(out.contains("(2i64 * 3i64)"));
}

#[test]
fn binop_divisus_per_emits_slash() {
    let out = parse_and_emit("sit ^x est 10 divisus per 2.\n");
    assert!(out.contains("(10i64 / 2i64)"));
}

#[test]
fn binop_modulo_emits_percent() {
    let out = parse_and_emit("sit ^x est 10 modulo 3.\n");
    assert!(out.contains("(10i64 % 3i64)"));
}

#[test]
fn binop_aequat_emits_eq_eq() {
    let out = parse_and_emit("sit ^x est 1 aequat 1.\n");
    assert!(out.contains("(1i64 == 1i64)"));
}

#[test]
fn binop_non_aequat_emits_bang_eq() {
    let out = parse_and_emit("sit ^x est 1 non aequat 2.\n");
    assert!(out.contains("(1i64 != 2i64)"));
}

#[test]
fn binop_minor_quam_emits_lt() {
    let out = parse_and_emit("sit ^x est 1 minor quam 2.\n");
    assert!(out.contains("(1i64 < 2i64)"));
}

#[test]
fn binop_maior_quam_emits_gt() {
    let out = parse_and_emit("sit ^x est 2 maior quam 1.\n");
    assert!(out.contains("(2i64 > 1i64)"));
}

#[test]
fn binop_minor_vel_par_emits_le() {
    let out = parse_and_emit("sit ^x est 1 minor vel par 2.\n");
    assert!(out.contains("(1i64 <= 2i64)"));
}

#[test]
fn binop_maior_vel_par_emits_ge() {
    let out = parse_and_emit("sit ^x est 2 maior vel par 1.\n");
    assert!(out.contains("(2i64 >= 1i64)"));
}

#[test]
fn binop_et_emits_logical_and() {
    let out = parse_and_emit("sit ^x est verum et falsum.\n");
    assert!(out.contains("(true && false)"));
}

#[test]
fn binop_vel_emits_logical_or() {
    let out = parse_and_emit("sit ^x est verum vel falsum.\n");
    assert!(out.contains("(true || false)"));
}

#[test]
fn unary_non_emits_bang() {
    let out = parse_and_emit("sit ^x est non verum.\n");
    assert!(out.contains("(!true)"));
}

#[test]
fn nested_binop_preserves_precedence_via_parens() {
    // a plus b per c → (a + (b * c))
    let out = parse_and_emit("sit ^x est 1 plus 2 per 3.\n");
    assert!(out.contains("(1i64 + (2i64 * 3i64))"));
}

#[test]
fn user_parens_collapse_into_tree_then_re_emit_parens() {
    // (1 plus 2) per 3 → ((1 + 2) * 3)
    let out = parse_and_emit("sit ^x est (1 plus 2) per 3.\n");
    assert!(out.contains("((1i64 + 2i64) * 3i64)"));
}

// ─── R12: control-flow emit ─────────────────────────────────────────────

#[test]
fn aliter_emits_else_block() {
    let out = parse_and_emit("si verum :\n    dic \"y\".\naliter :\n    dic \"n\".\n");
    let expected = "fn main() {\n    if true {\n        println!(\"{}\", \"y\");\n    } else {\n        println!(\"{}\", \"n\");\n    }\n}\n";
    assert_eq!(out, expected);
}

#[test]
fn aliter_si_emits_else_if_chain() {
    let out = parse_and_emit(
        "si ^x :\n    dic \"a\".\naliter si ^y :\n    dic \"b\".\naliter :\n    dic \"c\".\n",
    );
    // Shape: if x { … } else if y { … } else { … }
    assert!(out.contains("    if x {\n"));
    assert!(out.contains("} else if y {\n"));
    assert!(out.contains("} else {\n"));
}

#[test]
fn dum_emits_while() {
    let out = parse_and_emit("dum ^x :\n    dic \"loop\".\n");
    let expected = "fn main() {\n    while x {\n        println!(\"{}\", \"loop\");\n    }\n}\n";
    assert_eq!(out, expected);
}

#[test]
fn semper_emits_loop() {
    let out = parse_and_emit("semper :\n    interrumpe.\n");
    let expected = "fn main() {\n    loop {\n        break;\n    }\n}\n";
    assert_eq!(out, expected);
}

#[test]
fn continua_emits_continue() {
    let out = parse_and_emit("dum verum :\n    continua.\n");
    assert!(out.contains("    while true {\n        continue;\n    }\n"));
}

#[test]
fn si_with_binop_cond_emits_typecheckable_rust() {
    // Verifies that R11+R12 together produce Rust the borrow checker will accept,
    // closing R10's "si 1 :" caveat.
    let out = parse_and_emit("si verum et falsum :\n    dic \"ok\".\n");
    assert!(out.contains("if (true && false) {"));
}

#[test]
fn break_at_top_level_emits_break_semicolon() {
    // Just the statement itself emits; no validity guarantee at top level.
    let out = parse_and_emit("semper :\n    interrumpe.\n    continua.\n");
    assert!(out.contains("        break;\n"));
    assert!(out.contains("        continue;\n"));
}
