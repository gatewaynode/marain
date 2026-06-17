//! Expression emission: `emit_expr` plus the call and f-string helpers.
//!
//! Split from `emit.rs` (which sat at the 500-LOC target) per CLAUDE.md's
//! decomposition rule — the file had a clean statement/expression seam.
//! Statement emission and the shared escapers / `EmitError` stay in the parent
//! `emit` module; this child module reaches them via `super::`.

use std::fmt::Write;

use crate::ast::{CallExpr, Expr, FStringLit, FStringPart};

use super::{EmitError, escape_ident_for_rust, escape_string_for_rust};

pub(super) fn emit_expr(out: &mut String, expr: &Expr) -> Result<(), EmitError> {
    match expr {
        Expr::StringLit(s) => {
            out.push('"');
            out.push_str(&escape_string_for_rust(&s.value));
            out.push('"');
        }
        Expr::FString(f) => emit_fstring(out, f)?,
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
        Expr::Call(c) => emit_call(out, c)?,
        Expr::Range(r) => {
            // No paren-wrap (ranges aren't BinOps and don't share the
            // paren-everywhere rule); operands carry their own paren-wrap via
            // emit_expr if they're BinOp/UnaryOp shapes.
            if let Some(start) = &r.start {
                emit_expr(out, start)?;
            }
            out.push_str(if r.inclusive { "..=" } else { ".." });
            if let Some(end) = &r.end {
                emit_expr(out, end)?;
            }
        }
    }
    Ok(())
}

pub(super) fn emit_call(out: &mut String, c: &CallExpr) -> Result<(), EmitError> {
    let name = escape_ident_for_rust(&c.callee.name, c.callee.span)?;
    out.push_str(&name);
    out.push('(');
    for (i, arg) in c.args.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        emit_expr(out, arg)?;
    }
    out.push(')');
    Ok(())
}

/// Lower an f-string to `format!` (R17). Literal parts build the format string
/// (with `{`/`}` doubled to `{{`/`}}`); each interpolation contributes one `{}`
/// placeholder and one trailing argument, in order. Interpolated variables emit
/// as bare (escaped) names via the ordinary `VarRef` rule — a use site, never a
/// binding, so never `mut`. The all-interpolation form `f"{^a}{^b}"` is the
/// concatenation idiom: `format!("{}{}", a, b)`.
fn emit_fstring(out: &mut String, f: &FStringLit) -> Result<(), EmitError> {
    out.push_str("format!(\"");
    for part in &f.parts {
        match part {
            FStringPart::Literal(text) => push_fstring_literal(out, text),
            FStringPart::Interp(_) => out.push_str("{}"),
        }
    }
    out.push('"');
    for part in &f.parts {
        if let FStringPart::Interp(var) = part {
            out.push_str(", ");
            out.push_str(&escape_ident_for_rust(&var.name, var.span)?);
        }
    }
    out.push(')');
    Ok(())
}

/// Escape a literal segment for a Rust format string. Mirrors
/// `super::escape_string_for_rust` but ALSO doubles `{`/`}` so they survive
/// `format!`'s placeholder parser. Single-pass on the source char, so the
/// braces a control char expands to (`\u{..}`) are emitted whole, not doubled.
fn push_fstring_literal(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            '{' => out.push_str("{{"),
            '}' => out.push_str("}}"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", c as u32);
            }
            c => out.push(c),
        }
    }
}
