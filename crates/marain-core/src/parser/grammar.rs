//! Per-production grammar functions.
//!
//! Hand-rolled recursive descent over a flat token slice. Stage 1 has fixed
//! word order (PRD §4.2) so the parser dispatches on the leading token of
//! each statement and consumes a fixed shape. No precedence climbing in R5
//! because R5 carries no operators.

use crate::ast::{
    Expr, Ident, IntegerLit, LetStmt, MacroCallStmt, Module, SigiledIdent, Stmt, StringLit,
};
use crate::lexer::keywords::Keyword;
use crate::span::Span;
use crate::token::TokenKind;

use super::Parser;
use super::error::ParseError;

pub(super) fn parse_module(p: &mut Parser) -> Result<Module, ParseError> {
    let start = p.peek_span();
    let mut items = Vec::new();
    while !p.at_eof() {
        items.push(parse_stmt(p)?);
    }
    let end = p.peek_span();
    Ok(Module {
        items,
        span: start.join(end),
    })
}

fn parse_stmt(p: &mut Parser) -> Result<Stmt, ParseError> {
    match p.peek_kind() {
        TokenKind::Keyword(Keyword::Sit) => parse_let(p).map(Stmt::Let),
        TokenKind::Keyword(k) if is_no_punct_macro(*k) => parse_macro_call(p).map(Stmt::MacroCall),
        other => Err(ParseError::UnknownStatementStart {
            found: other.clone(),
            span: p.peek_span(),
        }),
    }
}

fn is_no_punct_macro(k: Keyword) -> bool {
    matches!(
        k,
        Keyword::Dic | Keyword::Queror | Keyword::Agmen | Keyword::Forma
    )
}

fn parse_let(p: &mut Parser) -> Result<LetStmt, ParseError> {
    let sit_span = expect_keyword(p, Keyword::Sit, "keyword `sit`")?;
    let name = parse_sigiled_ident(p)?;
    expect_keyword(p, Keyword::Est, "keyword `est`")?;
    let value = parse_expr(p)?;
    let period_span = expect_kind(p, &TokenKind::Period, "`.`")?;
    Ok(LetStmt {
        name,
        value,
        span: sit_span.join(period_span),
    })
}

fn parse_macro_call(p: &mut Parser) -> Result<MacroCallStmt, ParseError> {
    // parse_stmt has already verified the leading keyword is a no-punct macro.
    let tok = p.current_clone();
    let (callee_name, callee_span) = match tok.kind {
        TokenKind::Keyword(k) => (k.as_str().to_string(), tok.span),
        _ => unreachable!("dispatched here by parse_stmt"),
    };
    p.advance();
    let callee = Ident::new(callee_name, callee_span);
    let arg = parse_expr(p)?;
    let period_span = expect_kind(p, &TokenKind::Period, "`.`")?;
    Ok(MacroCallStmt {
        callee,
        arg,
        span: callee_span.join(period_span),
    })
}

fn parse_expr(p: &mut Parser) -> Result<Expr, ParseError> {
    let tok = p.current_clone();
    let span = tok.span;
    match tok.kind {
        TokenKind::StringLit(value) => {
            p.advance();
            Ok(Expr::StringLit(StringLit { value, span }))
        }
        TokenKind::IntegerLit(value) => {
            p.advance();
            Ok(Expr::IntegerLit(IntegerLit { value, span }))
        }
        TokenKind::SigiledIdent { sigil, name } => {
            p.advance();
            Ok(Expr::VarRef(SigiledIdent::new(sigil, name, span)))
        }
        other => Err(ParseError::ExpectedExpression { found: other, span }),
    }
}

fn parse_sigiled_ident(p: &mut Parser) -> Result<SigiledIdent, ParseError> {
    let tok = p.current_clone();
    let span = tok.span;
    match tok.kind {
        TokenKind::SigiledIdent { sigil, name } => {
            p.advance();
            Ok(SigiledIdent::new(sigil, name, span))
        }
        other => Err(ParseError::UnexpectedToken {
            found: other,
            expected: "sigiled identifier (`^name` or `@name`)",
            span,
        }),
    }
}

fn expect_keyword(p: &mut Parser, kw: Keyword, label: &'static str) -> Result<Span, ParseError> {
    let tok = p.current_clone();
    if matches!(&tok.kind, TokenKind::Keyword(k) if *k == kw) {
        p.advance();
        Ok(tok.span)
    } else {
        Err(ParseError::UnexpectedToken {
            found: tok.kind,
            expected: label,
            span: tok.span,
        })
    }
}

fn expect_kind(p: &mut Parser, want: &TokenKind, label: &'static str) -> Result<Span, ParseError> {
    let tok = p.current_clone();
    if std::mem::discriminant(&tok.kind) == std::mem::discriminant(want) {
        p.advance();
        Ok(tok.span)
    } else {
        Err(ParseError::UnexpectedToken {
            found: tok.kind,
            expected: label,
            span: tok.span,
        })
    }
}
