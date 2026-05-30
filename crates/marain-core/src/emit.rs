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

use crate::ast::{
    Block, ElseBranch, Expr, IfStmt, LetStmt, LoopStmt, MacroCallStmt, Module, Stmt, WhileStmt,
};
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
        Stmt::While(w) => emit_while(out, w, indent_level)?,
        Stmt::Loop(l) => emit_loop(out, l, indent_level)?,
        Stmt::Break(_) => out.push_str("break;"),
        Stmt::Continue(_) => out.push_str("continue;"),
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
    if let Some(else_branch) = &i.else_branch {
        emit_else_branch(out, else_branch, indent_level)?;
    }
    Ok(())
}

fn emit_else_branch(
    out: &mut String,
    branch: &ElseBranch,
    indent_level: usize,
) -> Result<(), EmitError> {
    match branch {
        ElseBranch::Block(block) => {
            out.push_str(" else {\n");
            emit_block_body(out, block, indent_level + 1)?;
            push_indent(out, indent_level);
            out.push('}');
        }
        ElseBranch::If(inner) => {
            // Chained `else if`. emit_if writes `if <cond> { ... }` (no leading
            // indent, no trailing newline), so prefixing ` else ` here yields
            // the standard Rust `} else if <cond> { ... }` shape.
            out.push_str(" else ");
            emit_if(out, inner, indent_level)?;
        }
    }
    Ok(())
}

fn emit_while(out: &mut String, w: &WhileStmt, indent_level: usize) -> Result<(), EmitError> {
    out.push_str("while ");
    emit_expr(out, &w.cond)?;
    out.push_str(" {\n");
    emit_block_body(out, &w.body, indent_level + 1)?;
    push_indent(out, indent_level);
    out.push('}');
    Ok(())
}

fn emit_loop(out: &mut String, l: &LoopStmt, indent_level: usize) -> Result<(), EmitError> {
    out.push_str("loop {\n");
    emit_block_body(out, &l.body, indent_level + 1)?;
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
        Expr::BoolLit(b) => {
            out.push_str(if b.value { "true" } else { "false" });
        }
        Expr::VarRef(v) => {
            let escaped = escape_ident_for_rust(&v.name, v.span)?;
            out.push_str(&escaped);
        }
        Expr::BinOp(b) => {
            // Wrap every binary op in parens; the parser already encodes correct
            // precedence in the tree shape, paren-everywhere makes emission
            // bulletproof against precedence drift in the Rust target.
            out.push('(');
            emit_expr(out, &b.lhs)?;
            out.push(' ');
            out.push_str(b.op.as_rust());
            out.push(' ');
            emit_expr(out, &b.rhs)?;
            out.push(')');
        }
        Expr::UnaryOp(u) => {
            out.push('(');
            out.push_str(u.op.as_rust());
            emit_expr(out, &u.operand)?;
            out.push(')');
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
#[path = "emit_tests.rs"]
mod tests;
