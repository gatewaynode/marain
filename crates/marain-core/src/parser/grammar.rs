//! Statement-level grammar productions plus the cross-cutting helpers
//! (`expect_kind`, `expect_keyword`, `parse_plain_ident`,
//! `parse_sigiled_ident`) used by both the statement and expression layers.
//!
//! Expression-level productions (the precedence cascade, `parse_call`,
//! `make_binop`) live in [`crate::parser::expressions`] — split out per the
//! C-1 pressure-release rule when R13 pushed this file across 500 LOC.
//!
//! Hand-rolled recursive descent over a flat token slice. Stage 1 has fixed
//! word order (PRD §4.2).

use crate::ast::{
    Block, BreakStmt, CallStmt, ContinueStmt, ElseBranch, FunctionStmt, Ident, IfStmt, LetStmt,
    LoopStmt, MacroCallStmt, Module, Param, ReturnStmt, SigiledIdent, Stmt, TypeRef, WhileStmt,
};
use crate::lexer::keywords::Keyword;
use crate::span::Span;
use crate::token::TokenKind;

use super::Parser;
use super::error::ParseError;
use super::expressions::{parse_call, parse_expr};

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
        TokenKind::Keyword(Keyword::Functio) => parse_function(p).map(Stmt::Function),
        TokenKind::Keyword(Keyword::Redde) => parse_return(p).map(Stmt::Return),
        TokenKind::Keyword(k) if is_no_punct_macro(*k) => parse_macro_call(p).map(Stmt::MacroCall),
        // `<name>(...)` at statement position — call as side-effect statement.
        // Sigiled idents are rejected here: a bare `^x.` has no observable
        // effect, so it would only be confusion.
        TokenKind::PlainIdent(_) if matches!(p.peek_kind_at(1), TokenKind::LParen) => {
            parse_call_stmt(p).map(Stmt::Call)
        }
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

fn parse_function(p: &mut Parser) -> Result<FunctionStmt, ParseError> {
    let kw_span = expect_keyword(p, Keyword::Functio, "keyword `functio`")?;
    let name = parse_plain_ident(p, "function name (PlainIdent)")?;
    expect_kind(p, &TokenKind::LParen, "`(`")?;
    let params = parse_param_list(p)?;
    expect_kind(p, &TokenKind::RParen, "`)`")?;
    let return_type = if matches!(p.peek_kind(), TokenKind::Keyword(Keyword::Dat)) {
        p.advance();
        Some(parse_type_ref(p)?)
    } else {
        None
    };
    expect_kind(p, &TokenKind::Colon, "`:`")?;
    let body = parse_block(p)?;
    Ok(FunctionStmt {
        span: kw_span.join(body.span),
        name,
        params,
        return_type,
        body,
    })
}

fn parse_param_list(p: &mut Parser) -> Result<Vec<Param>, ParseError> {
    let mut params = Vec::new();
    // Empty list: zero-arg signature `functio foo() :`.
    if matches!(p.peek_kind(), TokenKind::RParen) {
        return Ok(params);
    }
    loop {
        params.push(parse_param(p)?);
        match p.peek_kind() {
            TokenKind::Comma => {
                p.advance();
                // Trailing comma: `(^x: Sermo,)` — exit if `)` follows.
                if matches!(p.peek_kind(), TokenKind::RParen) {
                    break;
                }
            }
            _ => break,
        }
    }
    Ok(params)
}

fn parse_param(p: &mut Parser) -> Result<Param, ParseError> {
    let name = parse_sigiled_ident(p)?;
    expect_kind(p, &TokenKind::Colon, "`:`")?;
    let type_ref = parse_type_ref(p)?;
    Ok(Param {
        span: name.span.join(type_ref.span),
        name,
        type_ref,
    })
}

fn parse_type_ref(p: &mut Parser) -> Result<TypeRef, ParseError> {
    let tok = p.current_clone();
    match tok.kind {
        TokenKind::PlainIdent(name) => {
            // PRD §4.9: type names must use PascalCase. The lexer doesn't have
            // type-position context, so the check happens here.
            if !name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                return Err(ParseError::TypePositionRequiresPascalCase {
                    name,
                    span: tok.span,
                });
            }
            p.advance();
            let ident = Ident::new(name, tok.span);
            Ok(TypeRef {
                name: ident,
                span: tok.span,
            })
        }
        other => Err(ParseError::UnexpectedToken {
            found: other,
            expected: "type name (PascalCase identifier)",
            span: tok.span,
        }),
    }
}

fn parse_return(p: &mut Parser) -> Result<ReturnStmt, ParseError> {
    let kw_span = expect_keyword(p, Keyword::Redde, "keyword `redde`")?;
    // `redde.` is bare unit return; `redde <expr>.` carries a value.
    let value = if matches!(p.peek_kind(), TokenKind::Period) {
        None
    } else {
        Some(parse_expr(p)?)
    };
    let period_span = expect_kind(p, &TokenKind::Period, "`.`")?;
    Ok(ReturnStmt {
        value,
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

fn parse_call_stmt(p: &mut Parser) -> Result<CallStmt, ParseError> {
    let callee = parse_plain_ident(p, "function name (PlainIdent)")?;
    let call = parse_call(p, callee)?;
    let period_span = expect_kind(p, &TokenKind::Period, "`.`")?;
    Ok(CallStmt {
        span: call.span.join(period_span),
        call,
    })
}

fn parse_plain_ident(p: &mut Parser, label: &'static str) -> Result<Ident, ParseError> {
    let tok = p.current_clone();
    match tok.kind {
        TokenKind::PlainIdent(name) => {
            p.advance();
            Ok(Ident::new(name, tok.span))
        }
        other => Err(ParseError::UnexpectedToken {
            found: other,
            expected: label,
            span: tok.span,
        }),
    }
}

pub(super) fn parse_sigiled_ident(p: &mut Parser) -> Result<SigiledIdent, ParseError> {
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

pub(super) fn expect_keyword(
    p: &mut Parser,
    kw: Keyword,
    label: &'static str,
) -> Result<Span, ParseError> {
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

pub(super) fn expect_kind(
    p: &mut Parser,
    want: &TokenKind,
    label: &'static str,
) -> Result<Span, ParseError> {
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
