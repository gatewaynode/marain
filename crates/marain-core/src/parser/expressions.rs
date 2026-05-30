//! Expression-level parsing: precedence-climbing cascade plus the leaf-side
//! `parse_call` and `make_binop` helpers.
//!
//! Split out of `grammar.rs` per the C-1 pressure-release rule (grammar.rs
//! crossed 500 LOC when R13's function/return/call statement productions
//! landed). Statement productions stay in `grammar.rs`; this file owns the
//! precedence cascade so each file remains one cohesive syntactic family.
//!
//! Lowest precedence at the top, highest at the bottom. All binary levels are
//! left-associative (Rust's behavior); unary `non` is right-associative by
//! recursion. The cascade mirrors PRD §4.4's Rust-inherited table.

use crate::ast::{
    BinOp, BinOpExpr, BoolLit, CallExpr, Expr, Ident, IntegerLit, SigiledIdent, StringLit, UnaryOp,
    UnaryOpExpr,
};
use crate::lexer::keywords::Keyword;
use crate::token::TokenKind;

use super::Parser;
use super::error::ParseError;
use super::grammar::{expect_keyword, expect_kind};

pub(super) fn parse_expr(p: &mut Parser) -> Result<Expr, ParseError> {
    parse_or(p)
}

fn parse_or(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_and(p)?;
    while matches!(p.peek_kind(), TokenKind::Keyword(Keyword::Vel)) {
        p.advance();
        let rhs = parse_and(p)?;
        lhs = make_binop(BinOp::Vel, lhs, rhs);
    }
    Ok(lhs)
}

fn parse_and(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_equality(p)?;
    while matches!(p.peek_kind(), TokenKind::Keyword(Keyword::Et)) {
        p.advance();
        let rhs = parse_equality(p)?;
        lhs = make_binop(BinOp::Et, lhs, rhs);
    }
    Ok(lhs)
}

fn parse_equality(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_comparison(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::Keyword(Keyword::Aequat) => {
                p.advance();
                BinOp::Aequat
            }
            TokenKind::Keyword(Keyword::Non)
                if matches!(p.peek_kind_at(1), TokenKind::Keyword(Keyword::Aequat)) =>
            {
                p.advance(); // non
                p.advance(); // aequat
                BinOp::NonAequat
            }
            _ => break,
        };
        let rhs = parse_comparison(p)?;
        lhs = make_binop(op, lhs, rhs);
    }
    Ok(lhs)
}

fn parse_comparison(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_additive(p)?;
    loop {
        let head = match p.peek_kind() {
            TokenKind::Keyword(Keyword::Minor) => Keyword::Minor,
            TokenKind::Keyword(Keyword::Maior) => Keyword::Maior,
            _ => break,
        };
        p.advance(); // minor | maior
        let op = consume_comparison_completer(p, head)?;
        let rhs = parse_additive(p)?;
        lhs = make_binop(op, lhs, rhs);
    }
    Ok(lhs)
}

fn consume_comparison_completer(p: &mut Parser, head: Keyword) -> Result<BinOp, ParseError> {
    match p.peek_kind() {
        TokenKind::Keyword(Keyword::Quam) => {
            p.advance();
            Ok(match head {
                Keyword::Minor => BinOp::MinorQuam,
                Keyword::Maior => BinOp::MaiorQuam,
                _ => unreachable!(),
            })
        }
        TokenKind::Keyword(Keyword::Vel) => {
            p.advance();
            expect_keyword(
                p,
                Keyword::Par,
                "keyword `par` to complete `vel par` phrase",
            )?;
            Ok(match head {
                Keyword::Minor => BinOp::MinorVelPar,
                Keyword::Maior => BinOp::MaiorVelPar,
                _ => unreachable!(),
            })
        }
        _ => {
            let tok = p.current_clone();
            let label = match head {
                Keyword::Minor => "keyword `quam` or `vel par` to complete `minor` comparison",
                Keyword::Maior => "keyword `quam` or `vel par` to complete `maior` comparison",
                _ => unreachable!(),
            };
            Err(ParseError::UnexpectedToken {
                found: tok.kind,
                expected: label,
                span: tok.span,
            })
        }
    }
}

fn parse_additive(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_multiplicative(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::Keyword(Keyword::Plus) => BinOp::Plus,
            TokenKind::Keyword(Keyword::Minus) => BinOp::Minus,
            _ => break,
        };
        p.advance();
        let rhs = parse_multiplicative(p)?;
        lhs = make_binop(op, lhs, rhs);
    }
    Ok(lhs)
}

fn parse_multiplicative(p: &mut Parser) -> Result<Expr, ParseError> {
    let mut lhs = parse_unary(p)?;
    loop {
        let op = match p.peek_kind() {
            TokenKind::Keyword(Keyword::Per) => {
                p.advance();
                BinOp::Per
            }
            TokenKind::Keyword(Keyword::Modulo) => {
                p.advance();
                BinOp::Modulo
            }
            TokenKind::Keyword(Keyword::Divisus) => {
                p.advance();
                expect_keyword(p, Keyword::Per, "keyword `per` to complete `divisus per`")?;
                BinOp::DivisusPer
            }
            _ => break,
        };
        let rhs = parse_unary(p)?;
        lhs = make_binop(op, lhs, rhs);
    }
    Ok(lhs)
}

fn parse_unary(p: &mut Parser) -> Result<Expr, ParseError> {
    // `non aequat` is the binary `!=` operator (handled at parse_equality);
    // standalone `non` here is the unary logical-not prefix. The equality level
    // pre-empts `non aequat`, so by the time we reach parse_unary the only `non`
    // we can see is the prefix form.
    if matches!(p.peek_kind(), TokenKind::Keyword(Keyword::Non)) {
        let non_span = p.peek_span();
        p.advance();
        let operand = parse_unary(p)?;
        let span = non_span.join(operand.span());
        return Ok(Expr::UnaryOp(UnaryOpExpr {
            op: UnaryOp::Non,
            operand: Box::new(operand),
            span,
        }));
    }
    parse_primary(p)
}

fn parse_primary(p: &mut Parser) -> Result<Expr, ParseError> {
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
        TokenKind::Keyword(Keyword::Verum) => {
            p.advance();
            Ok(Expr::BoolLit(BoolLit { value: true, span }))
        }
        TokenKind::Keyword(Keyword::Falsum) => {
            p.advance();
            Ok(Expr::BoolLit(BoolLit { value: false, span }))
        }
        TokenKind::SigiledIdent { sigil, name } => {
            p.advance();
            Ok(Expr::VarRef(SigiledIdent::new(sigil, name, span)))
        }
        TokenKind::PlainIdent(name) => {
            // The only PlainIdent that's valid in expression position is a
            // function call: `name(args)`. A bare PlainIdent without `(` is a
            // parse error (variables always carry a sigil per PRD §4.5).
            if matches!(p.peek_kind_at(1), TokenKind::LParen) {
                p.advance();
                let callee = Ident::new(name, span);
                parse_call(p, callee).map(Expr::Call)
            } else {
                Err(ParseError::ExpectedExpression {
                    found: TokenKind::PlainIdent(name),
                    span,
                })
            }
        }
        TokenKind::LParen => {
            p.advance();
            let inner = parse_expr(p)?;
            expect_kind(p, &TokenKind::RParen, "`)`")?;
            Ok(inner)
        }
        other => Err(ParseError::ExpectedExpression { found: other, span }),
    }
}

pub(super) fn parse_call(p: &mut Parser, callee: Ident) -> Result<CallExpr, ParseError> {
    let callee_start = callee.span;
    expect_kind(p, &TokenKind::LParen, "`(`")?;
    let mut args = Vec::new();
    if !matches!(p.peek_kind(), TokenKind::RParen) {
        loop {
            args.push(parse_expr(p)?);
            match p.peek_kind() {
                TokenKind::Comma => {
                    p.advance();
                    // Trailing comma: `foo(x, y,)` — exit if `)` follows.
                    if matches!(p.peek_kind(), TokenKind::RParen) {
                        break;
                    }
                }
                _ => break,
            }
        }
    }
    let rparen_span = expect_kind(p, &TokenKind::RParen, "`)`")?;
    Ok(CallExpr {
        callee,
        args,
        span: callee_start.join(rparen_span),
    })
}

fn make_binop(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    let span = lhs.span().join(rhs.span());
    Expr::BinOp(BinOpExpr {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        span,
    })
}
