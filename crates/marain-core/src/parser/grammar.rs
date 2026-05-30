//! Per-production grammar functions.
//!
//! Hand-rolled recursive descent over a flat token slice. Stage 1 has fixed
//! word order (PRD §4.2). Expression-level operators (R11+R12) use precedence
//! climbing through a cascade of `parse_<level>` functions; the cascade order
//! mirrors Rust's precedence table (PRD §4.4) so `a plus b per c` parses as
//! `a plus (b per c)`. Multi-word phrases (`maior quam`, `minor vel par`,
//! `divisus per`, `non aequat`) are recognized greedily at the parser level —
//! the lexer emits one token per word per PRD §4.4.

use crate::ast::{
    BinOp, BinOpExpr, Block, BoolLit, BreakStmt, ContinueStmt, ElseBranch, Expr, Ident, IfStmt,
    IntegerLit, LetStmt, LoopStmt, MacroCallStmt, Module, SigiledIdent, Stmt, StringLit, UnaryOp,
    UnaryOpExpr, WhileStmt,
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
        TokenKind::Keyword(Keyword::Si) => parse_if(p).map(Stmt::If),
        TokenKind::Keyword(Keyword::Dum) => parse_while(p).map(Stmt::While),
        TokenKind::Keyword(Keyword::Semper) => parse_loop(p).map(Stmt::Loop),
        TokenKind::Keyword(Keyword::Interrumpe) => parse_break(p).map(Stmt::Break),
        TokenKind::Keyword(Keyword::Continua) => parse_continue(p).map(Stmt::Continue),
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

fn parse_if(p: &mut Parser) -> Result<IfStmt, ParseError> {
    let si_span = expect_keyword(p, Keyword::Si, "keyword `si`")?;
    let cond = parse_expr(p)?;
    expect_kind(p, &TokenKind::Colon, "`:`")?;
    let then_block = parse_block(p)?;
    let mut total_span = si_span.join(then_block.span);

    // After the then-body's Dedent, `aliter` (if any) appears as the next token.
    // `aliter si …` recurses through parse_if for the chain shape; bare `aliter :`
    // takes a terminal block.
    let else_branch = if matches!(p.peek_kind(), TokenKind::Keyword(Keyword::Aliter)) {
        p.advance();
        if matches!(p.peek_kind(), TokenKind::Keyword(Keyword::Si)) {
            let nested = parse_if(p)?;
            total_span = total_span.join(nested.span);
            Some(ElseBranch::If(Box::new(nested)))
        } else {
            expect_kind(p, &TokenKind::Colon, "`:`")?;
            let else_block = parse_block(p)?;
            total_span = total_span.join(else_block.span);
            Some(ElseBranch::Block(else_block))
        }
    } else {
        None
    };

    Ok(IfStmt {
        cond,
        then_block,
        else_branch,
        span: total_span,
    })
}

fn parse_while(p: &mut Parser) -> Result<WhileStmt, ParseError> {
    let dum_span = expect_keyword(p, Keyword::Dum, "keyword `dum`")?;
    let cond = parse_expr(p)?;
    expect_kind(p, &TokenKind::Colon, "`:`")?;
    let body = parse_block(p)?;
    Ok(WhileStmt {
        span: dum_span.join(body.span),
        cond,
        body,
    })
}

fn parse_loop(p: &mut Parser) -> Result<LoopStmt, ParseError> {
    let semper_span = expect_keyword(p, Keyword::Semper, "keyword `semper`")?;
    expect_kind(p, &TokenKind::Colon, "`:`")?;
    let body = parse_block(p)?;
    Ok(LoopStmt {
        span: semper_span.join(body.span),
        body,
    })
}

fn parse_break(p: &mut Parser) -> Result<BreakStmt, ParseError> {
    let kw_span = expect_keyword(p, Keyword::Interrumpe, "keyword `interrumpe`")?;
    let period_span = expect_kind(p, &TokenKind::Period, "`.`")?;
    Ok(BreakStmt {
        span: kw_span.join(period_span),
    })
}

fn parse_continue(p: &mut Parser) -> Result<ContinueStmt, ParseError> {
    let kw_span = expect_keyword(p, Keyword::Continua, "keyword `continua`")?;
    let period_span = expect_kind(p, &TokenKind::Period, "`.`")?;
    Ok(ContinueStmt {
        span: kw_span.join(period_span),
    })
}

fn parse_block(p: &mut Parser) -> Result<Block, ParseError> {
    let indent_span = expect_kind(p, &TokenKind::Indent, "indented block")?;
    let mut stmts = Vec::new();
    while !matches!(p.peek_kind(), TokenKind::Dedent | TokenKind::Eof) {
        stmts.push(parse_stmt(p)?);
    }
    let dedent_span = expect_kind(p, &TokenKind::Dedent, "end of indented block")?;
    Ok(Block {
        stmts,
        span: indent_span.join(dedent_span),
    })
}

// ─── Expression precedence cascade ──────────────────────────────────────────
//
// Lowest precedence at the top, highest at the bottom. All binary levels are
// left-associative (Rust's behavior); unary `non` is right-associative by
// recursion. The cascade mirrors PRD §4.4's Rust-inherited table.

fn parse_expr(p: &mut Parser) -> Result<Expr, ParseError> {
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
        TokenKind::LParen => {
            p.advance();
            let inner = parse_expr(p)?;
            expect_kind(p, &TokenKind::RParen, "`)`")?;
            Ok(inner)
        }
        other => Err(ParseError::ExpectedExpression { found: other, span }),
    }
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
