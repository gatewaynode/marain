//! Stage 1 Rust-source emitter.
//!
//! Pure functional: `emit(&Module) -> Result<String, EmitError>`. Stage 1's
//! emit is mechanical for every R5 production; the only failure mode is a
//! Marain identifier that collides with a Rust reserved word that cannot be
//! raw-escaped (`crate`, `extern`, `self`, `Self`, `super`).
//!
//! Rust identifier collisions are handled with the raw-identifier syntax
//! (`r#name`); the 5 unescapable keywords surface as `EmitError`. This is the
//! complete Rust 2024 reserved-word table — including future-reserved words
//! (`abstract`, `become`, `final`, etc.) — so a Marain program that lexes and
//! parses today still emits valid Rust when those words later become active.

use std::fmt;
use std::fmt::Write;

use crate::ast::{Block, Expr, IfStmt, LetStmt, MacroCallStmt, Module, Stmt};
use crate::error::Diagnostic;
use crate::span::Span;
use crate::token::Sigil;

/// Emit a Stage 1 module as a complete Rust source string.
///
/// Output shape:
/// ```text
/// fn main() {
///     <emitted statements, one per line, 4-space indent>
/// }
/// ```
pub fn emit(module: &Module) -> Result<String, EmitError> {
    let mut out = String::new();
    out.push_str("fn main() {\n");
    for stmt in &module.items {
        emit_stmt(&mut out, stmt, 1)?;
    }
    out.push_str("}\n");
    Ok(out)
}

fn emit_stmt(out: &mut String, stmt: &Stmt, indent_level: usize) -> Result<(), EmitError> {
    push_indent(out, indent_level);
    match stmt {
        Stmt::Let(l) => emit_let(out, l)?,
        Stmt::MacroCall(c) => emit_macro_call(out, c)?,
        Stmt::If(i) => emit_if(out, i, indent_level)?,
    }
    out.push('\n');
    Ok(())
}

fn push_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("    ");
    }
}

fn emit_if(out: &mut String, i: &IfStmt, indent_level: usize) -> Result<(), EmitError> {
    out.push_str("if ");
    emit_expr(out, &i.cond)?;
    out.push_str(" {\n");
    emit_block_body(out, &i.then_block, indent_level + 1)?;
    push_indent(out, indent_level);
    out.push('}');
    Ok(())
}

fn emit_block_body(out: &mut String, block: &Block, indent_level: usize) -> Result<(), EmitError> {
    for stmt in &block.stmts {
        emit_stmt(out, stmt, indent_level)?;
    }
    Ok(())
}

fn emit_let(out: &mut String, l: &LetStmt) -> Result<(), EmitError> {
    out.push_str("let ");
    if matches!(l.name.sigil, Sigil::Mutable) {
        out.push_str("mut ");
    }
    let escaped = escape_ident_for_rust(&l.name.name, l.name.span)?;
    out.push_str(&escaped);
    out.push_str(" = ");
    emit_expr(out, &l.value)?;
    out.push(';');
    Ok(())
}

fn emit_macro_call(out: &mut String, c: &MacroCallStmt) -> Result<(), EmitError> {
    // The parser only ever produces `dic` / `queror` / `agmen` / `forma` as
    // no-punct macro callees (see parser/grammar.rs). The dispatch below is
    // exhaustive over that set; anything else is a parser invariant violation.
    let (rust_macro, shape) = match c.callee.name.as_str() {
        "dic" => ("println", MacroShape::PrintLike),
        "queror" => ("eprintln", MacroShape::PrintLike),
        "agmen" => ("vec", MacroShape::Brackets),
        "forma" => ("format", MacroShape::PrintLike),
        other => unreachable!("parser rejected non-no-punct macro: {other}"),
    };
    out.push_str(rust_macro);
    out.push('!');
    match shape {
        MacroShape::PrintLike => {
            // Uniform `("{}", arg)` shape avoids the format-string footgun
            // where `dic "{} works".` would otherwise emit
            // `println!("{} works");` and Rust would treat `{}` as a placeholder.
            out.push_str("(\"{}\", ");
            emit_expr(out, &c.arg)?;
            out.push_str(");");
        }
        MacroShape::Brackets => {
            out.push('[');
            emit_expr(out, &c.arg)?;
            out.push_str("];");
        }
    }
    Ok(())
}

enum MacroShape {
    PrintLike,
    Brackets,
}

fn emit_expr(out: &mut String, expr: &Expr) -> Result<(), EmitError> {
    match expr {
        Expr::StringLit(s) => {
            out.push('"');
            out.push_str(&escape_string_for_rust(&s.value));
            out.push('"');
        }
        Expr::IntegerLit(i) => {
            // i64 suffix forces type to match the lexer's parsed representation
            // and prevents `let x = 5_000_000_000;` defaulting to i32 (overflow).
            let _ = write!(out, "{}i64", i.value);
        }
        Expr::VarRef(v) => {
            let escaped = escape_ident_for_rust(&v.name, v.span)?;
            out.push_str(&escaped);
        }
    }
    Ok(())
}

fn escape_string_for_rust(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn escape_ident_for_rust(name: &str, span: Span) -> Result<String, EmitError> {
    if is_rust_reserved_unescapable(name) {
        return Err(EmitError::UnescapableRustKeyword {
            name: name.to_string(),
            span,
        });
    }
    if is_rust_reserved_escapable(name) {
        Ok(format!("r#{name}"))
    } else {
        Ok(name.to_string())
    }
}

/// The five Rust reserved words that raw-identifier syntax cannot escape.
/// See https://doc.rust-lang.org/reference/identifiers.html — "Except for
/// `crate`, `extern`, `self`, `super` and `Self`, raw identifiers may be
/// used for keywords."
fn is_rust_reserved_unescapable(name: &str) -> bool {
    matches!(name, "crate" | "extern" | "self" | "Self" | "super")
}

/// Rust 2024 strict + reserved keywords that DO accept `r#` escape.
/// Strict keywords for every edition through 2024 plus all reserved-for-future
/// words. Mirrors the Rust reference's keyword tables; future-reserved entries
/// are escaped today so we keep working when they become active.
fn is_rust_reserved_escapable(name: &str) -> bool {
    matches!(
        name,
        "abstract"
            | "as"
            | "async"
            | "await"
            | "become"
            | "box"
            | "break"
            | "const"
            | "continue"
            | "do"
            | "dyn"
            | "else"
            | "enum"
            | "false"
            | "final"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "macro"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "override"
            | "priv"
            | "pub"
            | "ref"
            | "return"
            | "static"
            | "struct"
            | "trait"
            | "true"
            | "try"
            | "type"
            | "typeof"
            | "unsafe"
            | "unsized"
            | "use"
            | "virtual"
            | "where"
            | "while"
            | "yield"
    )
}

#[derive(Clone, Debug)]
pub enum EmitError {
    /// Marain identifier collides with a Rust reserved word that cannot be
    /// raw-escaped. The user must rename the Marain binding.
    UnescapableRustKeyword { name: String, span: Span },
}

impl EmitError {
    pub fn span(&self) -> Span {
        match self {
            Self::UnescapableRustKeyword { span, .. } => *span,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::UnescapableRustKeyword { name, .. } => format!(
                "identifier `{name}` is a Rust reserved word with no raw-identifier escape; rename the Marain binding"
            ),
        }
    }

    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::error(self.span(), self.message())
    }
}

impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for EmitError {}

#[cfg(test)]
mod tests {
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
            "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
            "do", "dyn", "else", "enum", "false", "final", "fn", "for", "gen", "if", "impl", "in",
            "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub",
            "ref", "return", "static", "struct", "trait", "true", "try", "type", "typeof",
            "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
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
}
