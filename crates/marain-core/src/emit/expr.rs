//! Expression emission: `emit_expr` plus the call and f-string helpers.
//!
//! Split from `emit.rs` (which sat at the 500-LOC target) per CLAUDE.md's
//! decomposition rule — the file had a clean statement/expression seam.
//! Statement emission and the shared escapers / `EmitError` stay in the parent
//! `emit` module; this child module reaches them via `super::`.

use std::fmt::Write;

use crate::ast::{Associativity, BinOp, CallExpr, Expr, FStringLit, FStringPart};

use super::{EmitError, escape_ident_for_rust, escape_string_for_rust};

/// Precedence above every binary operator: `non` (Rust `!`) binds tighter than
/// any binary op in our surface.
const UNARY_PREC: u8 = 0xF0;
/// Precedence below every binary operator: ranges are the loosest construct, so
/// any binop operand of a range binds tighter and never needs parens.
const RANGE_PREC: u8 = 1;
/// Atoms (literals, var-refs, calls, f-strings, already-parenthesized exprs)
/// never need wrapping — they re-parse as a single unit regardless of context.
const ATOM_PREC: u8 = u8::MAX;

/// Precedence rank of an expression for the minimal-paren decision. Exhaustive
/// over `Expr` so a future variant must declare where it sits.
fn expr_precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::BinOp(b) => b.op.precedence(),
        Expr::UnaryOp(_) => UNARY_PREC,
        Expr::Range(_) => RANGE_PREC,
        Expr::StringLit(_)
        | Expr::FString(_)
        | Expr::IntegerLit(_)
        | Expr::BoolLit(_)
        | Expr::VarRef(_)
        | Expr::Call(_) => ATOM_PREC,
    }
}

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
            // Minimal parens (R18, replacing R11+R12's paren-everywhere): wrap an
            // operand only when Rust's precedence/associativity would otherwise
            // re-parse it differently. See `emit_operand`.
            emit_operand(out, &b.lhs, b.op, false)?;
            out.push(' ');
            out.push_str(b.op.as_rust());
            out.push(' ');
            emit_operand(out, &b.rhs, b.op, true)?;
        }
        Expr::UnaryOp(u) => {
            out.push_str(u.op.as_rust());
            // `non`'s operand needs parens only when it binds looser than the
            // prefix op: `non (a et b)` → `!(a && b)`, but `non non a` → `!!a`.
            if expr_precedence(&u.operand) < UNARY_PREC {
                out.push('(');
                emit_expr(out, &u.operand)?;
                out.push(')');
            } else {
                emit_expr(out, &u.operand)?;
            }
        }
        Expr::Call(c) => emit_call(out, c)?,
        Expr::Range(r) => {
            // Range is the loosest construct, so its operands always bind
            // tighter and never need parens (the parser can't even produce a
            // range as a range operand). Emit them directly.
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

/// Emit `child` as an operand of a binary expression with operator `parent`,
/// parenthesizing only when Rust's grammar needs it to preserve the parse.
/// `is_right` marks the right-hand operand (matters for left-associative ops).
fn emit_operand(
    out: &mut String,
    child: &Expr,
    parent: BinOp,
    is_right: bool,
) -> Result<(), EmitError> {
    if operand_needs_parens(child, parent, is_right) {
        out.push('(');
        emit_expr(out, child)?;
        out.push(')');
    } else {
        emit_expr(out, child)?;
    }
    Ok(())
}

/// Wrap iff the child binds looser than the parent, or binds *equally* and its
/// position would re-group under the parent's associativity.
fn operand_needs_parens(child: &Expr, parent: BinOp, is_right: bool) -> bool {
    let parent_prec = parent.precedence();
    match expr_precedence(child) {
        c if c < parent_prec => true,
        c if c == parent_prec => regroups_at_equal_precedence(parent, is_right),
        _ => false,
    }
}

/// For an operand that ties the parent's precedence: a non-associative parent
/// (Rust's relationals) re-groups on *either* side (`a < b < c` is illegal), a
/// left-associative parent only on the right (`a - (b - c)` ≠ `a - b - c`).
fn regroups_at_equal_precedence(parent: BinOp, is_right: bool) -> bool {
    match parent.associativity() {
        Associativity::None => true,
        Associativity::Left => is_right,
    }
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
