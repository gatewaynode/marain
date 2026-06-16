//! 1213 LOC, exceeds 500 target: sibling test file for `parser/mod.rs`. All
//! tests share the `parse_ok` / `parse_err` helpers and exercise one cohesive
//! surface (the parser driver + grammar + expression productions). Splitting
//! by R-round would force callers to chase shared helpers across files for
//! no gain.

use std::path::PathBuf;

use super::*;
use crate::ast::{Expr, IntegerLit, Stmt};
use crate::error::MarainError;
use crate::lexer::lex;
use crate::source::SourceMap;
use crate::token::Sigil;

fn parse_str(text: &str) -> Result<Module, MarainError> {
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("test.lat"), text.to_string());
    let tokens = lex(map.get(id))?;
    Ok(parse(&tokens)?)
}

fn parse_ok(text: &str) -> Module {
    parse_str(text).expect("expected parse to succeed")
}

fn parse_err(text: &str) -> ParseError {
    let mut map = SourceMap::new();
    let id = map.add(PathBuf::from("test.lat"), text.to_string());
    let tokens = lex(map.get(id)).expect("lex must succeed for these inputs");
    parse(&tokens).expect_err("expected parse to fail")
}

#[test]
fn hello_world_parses() {
    let m = parse_ok("dic \"salve, munde\".\n");
    assert_eq!(m.items.len(), 1);
    match &m.items[0] {
        Stmt::MacroCall(c) => {
            assert_eq!(c.callee.name, "dic");
            match &c.arg {
                Expr::StringLit(s) => assert_eq!(s.value, "salve, munde"),
                other => panic!("expected StringLit, got {other:?}"),
            }
        }
        other => panic!("expected MacroCall, got {other:?}"),
    }
}

#[test]
fn let_with_integer_literal() {
    let m = parse_ok("sit ^x est 5.\n");
    assert_eq!(m.items.len(), 1);
    match &m.items[0] {
        Stmt::Let(l) => {
            assert_eq!(l.name.sigil, Sigil::Immutable);
            assert_eq!(l.name.name, "x");
            match &l.value {
                Expr::IntegerLit(i) => assert_eq!(i.value, 5),
                other => panic!("expected IntegerLit, got {other:?}"),
            }
        }
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn let_with_string_literal() {
    let m = parse_ok("sit @greeting est \"salve\".\n");
    match &m.items[0] {
        Stmt::Let(l) => {
            assert_eq!(l.name.sigil, Sigil::Mutable);
            assert_eq!(l.name.name, "greeting");
            match &l.value {
                Expr::StringLit(s) => assert_eq!(s.value, "salve"),
                other => panic!("expected StringLit, got {other:?}"),
            }
        }
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn dic_of_var_ref() {
    let m = parse_ok("dic ^x.\n");
    match &m.items[0] {
        Stmt::MacroCall(c) => match &c.arg {
            Expr::VarRef(v) => {
                assert_eq!(v.sigil, Sigil::Immutable);
                assert_eq!(v.name, "x");
            }
            other => panic!("expected VarRef, got {other:?}"),
        },
        other => panic!("expected MacroCall, got {other:?}"),
    }
}

#[test]
fn multi_statement_program() {
    let m = parse_ok("sit ^x est 5.\ndic ^x.\n");
    assert_eq!(m.items.len(), 2);
    assert!(matches!(m.items[0], Stmt::Let(_)));
    assert!(matches!(m.items[1], Stmt::MacroCall(_)));
}

#[test]
fn multi_statements_on_one_line() {
    let m = parse_ok("dic \"a\". dic \"b\".\n");
    assert_eq!(m.items.len(), 2);
    assert!(matches!(m.items[0], Stmt::MacroCall(_)));
    assert!(matches!(m.items[1], Stmt::MacroCall(_)));
}

#[test]
fn empty_source_parses_to_empty_module() {
    let m = parse_ok("");
    assert!(m.items.is_empty());
}

#[test]
fn whitespace_only_source_parses_to_empty_module() {
    let m = parse_ok("   \n  \n");
    assert!(m.items.is_empty());
}

#[test]
fn dic_with_no_arg_is_error() {
    let e = parse_err("dic.\n");
    assert!(
        matches!(e, ParseError::ExpectedExpression { .. }),
        "got {e:?}",
    );
}

#[test]
fn dic_with_trailing_garbage_is_error() {
    let e = parse_err("dic \"a\" \"b\".\n");
    // After "a", expect period; found another string.
    assert!(matches!(e, ParseError::UnexpectedToken { .. }), "got {e:?}");
}

#[test]
fn sit_without_est_is_error() {
    let e = parse_err("sit ^x 5.\n");
    // After ^x, expecting `est`, found integer.
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("est"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn sit_without_sigil_is_error() {
    let e = parse_err("sit x est 5.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("sigiled identifier"));
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn unknown_statement_start_is_error() {
    // `est` is a keyword but never legal at statement start (it's the
    // initializer in `sit ^x est <expr>`); a bare leading `est` trips the
    // dispatch's catch-all.
    let e = parse_err("est foo.\n");
    assert!(
        matches!(e, ParseError::UnknownStatementStart { .. }),
        "got {e:?}",
    );
}

#[test]
fn missing_period_at_eof_is_error() {
    let e = parse_err("dic \"a\"");
    match e {
        ParseError::UnexpectedToken {
            expected, found, ..
        } => {
            assert_eq!(expected, "`.`");
            assert!(matches!(found, TokenKind::Eof));
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn let_value_var_ref() {
    let m = parse_ok("sit @y est ^x.\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::VarRef(v) => {
                assert_eq!(v.sigil, Sigil::Immutable);
                assert_eq!(v.name, "x");
                assert!(v.inflection.is_none());
            }
            other => panic!("expected VarRef, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn integer_with_underscores_round_trips() {
    let m = parse_ok("sit ^big est 1_000_000.\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::IntegerLit(i) => assert_eq!(i.value, 1_000_000),
            other => panic!("expected IntegerLit, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn let_stmt_span_covers_sit_through_period() {
    let m = parse_ok("sit ^x est 5.\n");
    match &m.items[0] {
        Stmt::Let(l) => {
            assert_eq!(l.span.start, 0);
            // "sit ^x est 5." is 13 bytes; the period is at byte 12, so the
            // period span ends at byte 13.
            assert_eq!(l.span.end, 13);
        }
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn macro_call_span_covers_keyword_through_period() {
    let m = parse_ok("dic \"a\".\n");
    match &m.items[0] {
        Stmt::MacroCall(c) => {
            assert_eq!(c.span.start, 0);
            assert_eq!(c.span.end, 8);
        }
        other => panic!("expected MacroCall, got {other:?}"),
    }
}

#[test]
fn inflection_slot_defaults_none_after_parse() {
    let m = parse_ok("sit ^x est 5.\ndic ^x.\n");
    match &m.items[0] {
        Stmt::Let(l) => assert!(l.name.inflection.is_none()),
        _ => unreachable!(),
    }
    match &m.items[1] {
        Stmt::MacroCall(c) => match &c.arg {
            Expr::VarRef(v) => assert!(v.inflection.is_none()),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[test]
fn other_no_punct_macros_recognized() {
    // queror, agmen, forma all dispatch through the same macro-call form.
    // agmen normally takes a list expression, but with a string lit it still parses.
    let m = parse_ok("queror \"oops\".\n");
    assert!(matches!(m.items[0], Stmt::MacroCall(_)));
    let m = parse_ok("forma \"x\".\n");
    assert!(matches!(m.items[0], Stmt::MacroCall(_)));
    let m = parse_ok("agmen \"x\".\n");
    assert!(matches!(m.items[0], Stmt::MacroCall(_)));
}

#[test]
fn parse_error_joins_marain_error_facade() {
    let result = parse_str("dic.\n");
    match result {
        Err(MarainError::Parse(ParseError::ExpectedExpression { .. })) => {}
        other => panic!("expected MarainError::Parse, got {other:?}"),
    }
}

#[test]
#[should_panic(expected = "parser requires a token stream terminated with Eof")]
fn parser_requires_eof_terminator() {
    let _ = Parser::new(&[]);
}

#[test]
fn si_with_single_statement_body() {
    let m = parse_ok("si ^x :\n    dic ^x.\n");
    assert_eq!(m.items.len(), 1);
    match &m.items[0] {
        Stmt::If(i) => {
            match &i.cond {
                Expr::VarRef(v) => {
                    assert_eq!(v.sigil, Sigil::Immutable);
                    assert_eq!(v.name, "x");
                }
                other => panic!("expected VarRef cond, got {other:?}"),
            }
            assert_eq!(i.then_block.stmts.len(), 1);
            assert!(matches!(i.then_block.stmts[0], Stmt::MacroCall(_)));
        }
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn si_with_multi_statement_body() {
    let m = parse_ok("si ^x :\n    dic ^x.\n    sit ^y est 7.\n");
    match &m.items[0] {
        Stmt::If(i) => {
            assert_eq!(i.then_block.stmts.len(), 2);
            assert!(matches!(i.then_block.stmts[0], Stmt::MacroCall(_)));
            assert!(matches!(i.then_block.stmts[1], Stmt::Let(_)));
        }
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn si_with_nested_si_body() {
    let m = parse_ok("si ^x :\n    si ^y :\n        dic \"deep\".\n");
    match &m.items[0] {
        Stmt::If(outer) => {
            assert_eq!(outer.then_block.stmts.len(), 1);
            match &outer.then_block.stmts[0] {
                Stmt::If(inner) => {
                    assert_eq!(inner.then_block.stmts.len(), 1);
                    assert!(matches!(inner.then_block.stmts[0], Stmt::MacroCall(_)));
                }
                other => panic!("expected nested If, got {other:?}"),
            }
        }
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn si_with_integer_literal_cond_parses() {
    // R10 doesn't validate type — the condition expression set is whatever
    // parse_expr accepts (string/int/var-ref). Producing valid Rust is R11's job.
    let m = parse_ok("si 1 :\n    dic \"hi\".\n");
    match &m.items[0] {
        Stmt::If(i) => match &i.cond {
            Expr::IntegerLit(n) => assert_eq!(n.value, 1),
            other => panic!("expected IntegerLit cond, got {other:?}"),
        },
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn si_then_following_statement_at_column_zero() {
    // The body lands at column 0 — same indent as the parent — so the lexer
    // emits no Indent. parse_block surfaces the missing indent.
    let m = parse_ok("si ^x :\n    dic ^x.\nsit ^y est 7.\n");
    assert_eq!(m.items.len(), 2);
    assert!(matches!(m.items[0], Stmt::If(_)));
    assert!(matches!(m.items[1], Stmt::Let(_)));
}

#[test]
fn si_without_colon_is_error() {
    let e = parse_err("si ^x\n    dic ^x.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`:`"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn si_without_condition_is_error() {
    let e = parse_err("si :\n    dic \"hi\".\n");
    assert!(
        matches!(e, ParseError::ExpectedExpression { .. }),
        "got {e:?}",
    );
}

#[test]
fn si_without_indented_body_is_error() {
    let e = parse_err("si ^x :\nsit ^y est 1.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(
                expected.contains("indented block"),
                "expected label was {expected:?}",
            );
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn si_at_eof_with_no_body_is_error() {
    let e = parse_err("si ^x :\n");
    match e {
        ParseError::UnexpectedToken {
            expected, found, ..
        } => {
            assert!(
                expected.contains("indented block"),
                "expected label was {expected:?}",
            );
            assert!(matches!(found, TokenKind::Eof), "found was {found:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn if_stmt_span_covers_si_through_dedent() {
    let m = parse_ok("si ^x :\n    dic ^x.\n");
    match &m.items[0] {
        Stmt::If(i) => {
            assert_eq!(i.span.start, 0);
            // Span end is the Dedent's end byte; verify the structural extent
            // rather than an exact byte (Dedent is synthetic).
            assert!(i.span.end > i.then_block.stmts[0].span().end);
        }
        other => panic!("expected If, got {other:?}"),
    }
}

// ─── R11: expression precedence + new atoms ────────────────────────────

use crate::ast::{BinOp, ElseBranch, UnaryOp};

fn let_rhs(m: &Module) -> &Expr {
    match &m.items[0] {
        Stmt::Let(l) => &l.value,
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn verum_parses_as_bool_lit_true() {
    let m = parse_ok("sit ^x est verum.\n");
    match let_rhs(&m) {
        Expr::BoolLit(b) => assert!(b.value),
        other => panic!("expected BoolLit, got {other:?}"),
    }
}

#[test]
fn falsum_parses_as_bool_lit_false() {
    let m = parse_ok("sit ^x est falsum.\n");
    match let_rhs(&m) {
        Expr::BoolLit(b) => assert!(!b.value),
        other => panic!("expected BoolLit, got {other:?}"),
    }
}

#[test]
fn binop_plus_simple() {
    let m = parse_ok("sit ^x est 1 plus 2.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => {
            assert_eq!(b.op, BinOp::Plus);
            assert!(matches!(*b.lhs, Expr::IntegerLit(_)));
            assert!(matches!(*b.rhs, Expr::IntegerLit(_)));
        }
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn binop_per_binds_tighter_than_plus() {
    // a plus b per c → a plus (b per c)
    let m = parse_ok("sit ^x est 1 plus 2 per 3.\n");
    match let_rhs(&m) {
        Expr::BinOp(outer) => {
            assert_eq!(outer.op, BinOp::Plus);
            match &*outer.rhs {
                Expr::BinOp(inner) => assert_eq!(inner.op, BinOp::Per),
                other => panic!("expected inner Per BinOp, got {other:?}"),
            }
        }
        other => panic!("expected outer Plus BinOp, got {other:?}"),
    }
}

#[test]
fn binop_plus_left_associative() {
    // a plus b plus c → ((a plus b) plus c)
    let m = parse_ok("sit ^x est 1 plus 2 plus 3.\n");
    match let_rhs(&m) {
        Expr::BinOp(outer) => {
            assert_eq!(outer.op, BinOp::Plus);
            match &*outer.lhs {
                Expr::BinOp(inner) => assert_eq!(inner.op, BinOp::Plus),
                other => panic!("expected inner Plus, got {other:?}"),
            }
            assert!(matches!(*outer.rhs, Expr::IntegerLit(_)));
        }
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn unary_non_prefix() {
    let m = parse_ok("sit ^x est non verum.\n");
    match let_rhs(&m) {
        Expr::UnaryOp(u) => {
            assert_eq!(u.op, UnaryOp::Non);
            assert!(matches!(*u.operand, Expr::BoolLit(_)));
        }
        other => panic!("expected UnaryOp, got {other:?}"),
    }
}

#[test]
fn unary_non_right_associative() {
    // non non verum → !(!true)
    let m = parse_ok("sit ^x est non non verum.\n");
    match let_rhs(&m) {
        Expr::UnaryOp(outer) => {
            assert_eq!(outer.op, UnaryOp::Non);
            match &*outer.operand {
                Expr::UnaryOp(inner) => assert_eq!(inner.op, UnaryOp::Non),
                other => panic!("expected inner Non, got {other:?}"),
            }
        }
        other => panic!("expected UnaryOp, got {other:?}"),
    }
}

#[test]
fn minor_quam_recognized() {
    let m = parse_ok("sit ^x est ^a minor quam ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::MinorQuam),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn maior_quam_recognized() {
    let m = parse_ok("sit ^x est ^a maior quam ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::MaiorQuam),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn minor_vel_par_recognized() {
    let m = parse_ok("sit ^x est ^a minor vel par ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::MinorVelPar),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn maior_vel_par_recognized() {
    let m = parse_ok("sit ^x est ^a maior vel par ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::MaiorVelPar),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn aequat_recognized() {
    let m = parse_ok("sit ^x est ^a aequat ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::Aequat),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn non_aequat_recognized_as_binary() {
    let m = parse_ok("sit ^x est ^a non aequat ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::NonAequat),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn divisus_per_recognized() {
    let m = parse_ok("sit ^x est ^a divisus per ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::DivisusPer),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn logical_et_recognized() {
    let m = parse_ok("sit ^x est ^a et ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::Et),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn logical_vel_recognized() {
    let m = parse_ok("sit ^x est ^a vel ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::Vel),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn modulo_recognized() {
    let m = parse_ok("sit ^x est ^a modulo ^b.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::Modulo),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn parens_override_precedence() {
    // (1 plus 2) per 3 — parens must force Plus as the inner op
    let m = parse_ok("sit ^x est (1 plus 2) per 3.\n");
    match let_rhs(&m) {
        Expr::BinOp(outer) => {
            assert_eq!(outer.op, BinOp::Per);
            match &*outer.lhs {
                Expr::BinOp(inner) => assert_eq!(inner.op, BinOp::Plus),
                other => panic!("expected inner Plus, got {other:?}"),
            }
        }
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn full_precedence_cascade() {
    // et binds tighter than vel; aequat tighter than et; comparison tighter
    // than equality; additive tighter than comparison; multiplicative tighter
    // than additive; unary tighter than multiplicative.
    // non verum vel ^a et ^b aequat ^c minor quam ^d plus ^e per ^f
    //   parses as:
    // (non verum) vel (^a et (^b aequat (^c minor quam (^d plus (^e per ^f)))))
    let m = parse_ok("sit ^x est non verum vel ^a et ^b aequat ^c minor quam ^d plus ^e per ^f.\n");
    match let_rhs(&m) {
        Expr::BinOp(b) => assert_eq!(b.op, BinOp::Vel, "outermost should be vel"),
        other => panic!("expected BinOp, got {other:?}"),
    }
}

#[test]
fn bare_maior_without_phrase_completer_is_error() {
    let e = parse_err("sit ^x est ^a maior 5.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(
                expected.contains("`quam` or `vel par`"),
                "expected label was {expected:?}",
            );
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn bare_minor_without_phrase_completer_is_error() {
    let e = parse_err("sit ^x est ^a minor 5.\n");
    assert!(matches!(e, ParseError::UnexpectedToken { .. }), "got {e:?}");
}

#[test]
fn divisus_without_per_is_error() {
    let e = parse_err("sit ^x est ^a divisus 5.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(
                expected.contains("`per`"),
                "expected label was {expected:?}",
            );
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn minor_vel_without_par_is_error() {
    let e = parse_err("sit ^x est ^a minor vel 5.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(
                expected.contains("`par`"),
                "expected label was {expected:?}"
            );
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

// ─── R12: control flow ──────────────────────────────────────────────────

#[test]
fn si_with_aliter_block() {
    let m = parse_ok("si ^x :\n    dic \"yes\".\naliter :\n    dic \"no\".\n");
    match &m.items[0] {
        Stmt::If(i) => match &i.else_branch {
            Some(ElseBranch::Block(b)) => assert_eq!(b.stmts.len(), 1),
            other => panic!("expected ElseBranch::Block, got {other:?}"),
        },
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn si_with_aliter_si_chain() {
    let m = parse_ok("si ^x :\n    dic \"a\".\naliter si ^y :\n    dic \"b\".\n");
    match &m.items[0] {
        Stmt::If(i) => match &i.else_branch {
            Some(ElseBranch::If(inner)) => {
                assert!(matches!(inner.cond, Expr::VarRef(_)));
                assert!(inner.else_branch.is_none());
            }
            other => panic!("expected ElseBranch::If, got {other:?}"),
        },
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn si_with_multi_arm_aliter_si_then_terminal_aliter() {
    let m = parse_ok(
        "si ^x :\n    dic \"a\".\naliter si ^y :\n    dic \"b\".\naliter si ^z :\n    dic \"c\".\naliter :\n    dic \"d\".\n",
    );
    match &m.items[0] {
        Stmt::If(top) => {
            // top → If(y → If(z → Block))
            let l1 = match &top.else_branch {
                Some(ElseBranch::If(i)) => i,
                other => panic!("expected If at level 1, got {other:?}"),
            };
            let l2 = match &l1.else_branch {
                Some(ElseBranch::If(i)) => i,
                other => panic!("expected If at level 2, got {other:?}"),
            };
            assert!(matches!(l2.else_branch, Some(ElseBranch::Block(_))));
        }
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn dum_simple() {
    let m = parse_ok("dum ^x :\n    dic \"hi\".\n");
    match &m.items[0] {
        Stmt::While(w) => {
            assert!(matches!(w.cond, Expr::VarRef(_)));
            assert_eq!(w.body.stmts.len(), 1);
        }
        other => panic!("expected While, got {other:?}"),
    }
}

#[test]
fn semper_simple() {
    let m = parse_ok("semper :\n    dic \"forever\".\n");
    match &m.items[0] {
        Stmt::Loop(l) => assert_eq!(l.body.stmts.len(), 1),
        other => panic!("expected Loop, got {other:?}"),
    }
}

#[test]
fn semper_with_interrumpe() {
    let m = parse_ok("semper :\n    interrumpe.\n");
    match &m.items[0] {
        Stmt::Loop(l) => match &l.body.stmts[0] {
            Stmt::Break(_) => {}
            other => panic!("expected Break, got {other:?}"),
        },
        other => panic!("expected Loop, got {other:?}"),
    }
}

#[test]
fn dum_with_continua() {
    let m = parse_ok("dum ^x :\n    continua.\n");
    match &m.items[0] {
        Stmt::While(w) => match &w.body.stmts[0] {
            Stmt::Continue(_) => {}
            other => panic!("expected Continue, got {other:?}"),
        },
        other => panic!("expected While, got {other:?}"),
    }
}

#[test]
fn dum_missing_colon_is_error() {
    let e = parse_err("dum ^x\n    dic ^x.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`:`"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn interrumpe_missing_period_is_error() {
    let e = parse_err("semper :\n    interrumpe\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`.`"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn aliter_si_chain_span_extends_through_else() {
    let m = parse_ok("si ^x :\n    dic \"a\".\naliter :\n    dic \"b\".\n");
    match &m.items[0] {
        Stmt::If(top) => {
            let else_span = top.else_branch.as_ref().unwrap().span();
            assert!(top.span.end >= else_span.end);
        }
        other => panic!("expected If, got {other:?}"),
    }
}

#[test]
fn cond_with_binop_in_si() {
    let m = parse_ok("si ^a et ^b :\n    dic \"both\".\n");
    match &m.items[0] {
        Stmt::If(i) => match &i.cond {
            Expr::BinOp(b) => assert_eq!(b.op, BinOp::Et),
            other => panic!("expected BinOp cond, got {other:?}"),
        },
        other => panic!("expected If, got {other:?}"),
    }
}

// ─── R13: function declarations, returns, calls ────────────────────────

#[test]
fn functio_zero_arg_unit_return() {
    let m = parse_ok("functio saluta() :\n    dic \"hi\".\n");
    match &m.items[0] {
        Stmt::Function(f) => {
            assert_eq!(f.name.name, "saluta");
            assert!(f.params.is_empty());
            assert!(f.return_type.is_none());
            assert_eq!(f.body.stmts.len(), 1);
        }
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn functio_single_param_with_dat_return() {
    let m = parse_ok("functio echo(^x: Sermo) dat Sermo :\n    redde ^x.\n");
    match &m.items[0] {
        Stmt::Function(f) => {
            assert_eq!(f.name.name, "echo");
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.params[0].name.name, "x");
            assert_eq!(f.params[0].name.sigil, Sigil::Immutable);
            assert_eq!(f.params[0].type_ref.name.name, "Sermo");
            assert!(f.return_type.is_some());
            assert_eq!(f.return_type.as_ref().unwrap().name.name, "Sermo");
        }
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn functio_multi_param() {
    let m = parse_ok("functio add(^a: Numerus, ^b: Numerus) dat Numerus :\n    redde ^a.\n");
    match &m.items[0] {
        Stmt::Function(f) => {
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name.name, "a");
            assert_eq!(f.params[1].name.name, "b");
        }
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn functio_trailing_comma_in_param_list() {
    let m = parse_ok("functio foo(^x: Sermo,) :\n    dic \"x\".\n");
    match &m.items[0] {
        Stmt::Function(f) => assert_eq!(f.params.len(), 1),
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn functio_mutable_param_keeps_sigil_in_ast() {
    let m = parse_ok("functio bump(@x: Numerus) :\n    dic \"ok\".\n");
    match &m.items[0] {
        Stmt::Function(f) => {
            assert_eq!(f.params[0].name.sigil, Sigil::Mutable);
        }
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn functio_unknown_type_passes_through() {
    // B-3: open pass-through. `Custom` is not in the translation table.
    let m = parse_ok("functio f(^x: Custom) dat Custom :\n    dic \"x\".\n");
    match &m.items[0] {
        Stmt::Function(f) => {
            assert_eq!(f.params[0].type_ref.name.name, "Custom");
            assert_eq!(f.return_type.as_ref().unwrap().name.name, "Custom");
        }
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn redde_with_value() {
    let m = parse_ok("functio f() dat Numerus :\n    redde 42.\n");
    match &m.items[0] {
        Stmt::Function(f) => match &f.body.stmts[0] {
            Stmt::Return(r) => {
                assert!(r.value.is_some());
                match r.value.as_ref().unwrap() {
                    Expr::IntegerLit(i) => assert_eq!(i.value, 42),
                    other => panic!("expected IntegerLit, got {other:?}"),
                }
            }
            other => panic!("expected Return, got {other:?}"),
        },
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn redde_bare_unit() {
    let m = parse_ok("functio f() :\n    redde.\n");
    match &m.items[0] {
        Stmt::Function(f) => match &f.body.stmts[0] {
            Stmt::Return(r) => assert!(r.value.is_none()),
            other => panic!("expected Return, got {other:?}"),
        },
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn redde_outside_function_parses_cleanly() {
    // C-4: parser doesn't track function scope; rustc adjudicates.
    let m = parse_ok("redde 5.\n");
    match &m.items[0] {
        Stmt::Return(r) => assert!(r.value.is_some()),
        other => panic!("expected Return at top level, got {other:?}"),
    }
}

#[test]
fn call_zero_args() {
    let m = parse_ok("sit ^x est saluta().\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::Call(c) => {
                assert_eq!(c.callee.name, "saluta");
                assert!(c.args.is_empty());
            }
            other => panic!("expected Call, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn call_with_args() {
    let m = parse_ok("sit ^x est add(1, 2).\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::Call(c) => {
                assert_eq!(c.args.len(), 2);
            }
            other => panic!("expected Call, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn call_with_trailing_comma() {
    let m = parse_ok("sit ^x est add(1, 2,).\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::Call(c) => assert_eq!(c.args.len(), 2),
            other => panic!("expected Call, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn call_as_dic_arg() {
    // Use a non-keyword callee — keywords like `forma` can only appear in
    // their reserved statement position (no-punct macro head).
    let m = parse_ok("dic helper(\"x\").\n");
    match &m.items[0] {
        Stmt::MacroCall(mc) => match &mc.arg {
            Expr::Call(c) => assert_eq!(c.callee.name, "helper"),
            other => panic!("expected Call, got {other:?}"),
        },
        other => panic!("expected MacroCall, got {other:?}"),
    }
}

#[test]
fn call_with_binop_arg() {
    let m = parse_ok("sit ^x est f(1 plus 2).\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::Call(c) => match &c.args[0] {
                Expr::BinOp(b) => assert_eq!(b.op, BinOp::Plus),
                other => panic!("expected BinOp arg, got {other:?}"),
            },
            other => panic!("expected Call, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn nested_call() {
    let m = parse_ok("sit ^x est f(g(1), h()).\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::Call(c) => {
                assert_eq!(c.callee.name, "f");
                assert_eq!(c.args.len(), 2);
            }
            other => panic!("expected Call, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn function_then_main_stmt_at_module_level() {
    let m = parse_ok("functio greet() :\n    dic \"salve\".\nsit ^x est 5.\ndic \"done\".\n");
    assert_eq!(m.items.len(), 3);
    assert!(matches!(m.items[0], Stmt::Function(_)));
    assert!(matches!(m.items[1], Stmt::Let(_)));
    assert!(matches!(m.items[2], Stmt::MacroCall(_)));
}

// ─── R13: error paths ─────────────────────────────────────────────────

#[test]
fn functio_missing_parens_is_error() {
    let e = parse_err("functio foo :\n    dic \"x\".\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`(`"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn functio_missing_name_is_error() {
    let e = parse_err("functio () :\n    dic \"x\".\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(
                expected.contains("function name"),
                "expected label was {expected:?}"
            );
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn functio_missing_colon_after_signature_is_error() {
    let e = parse_err("functio foo()\n    dic \"x\".\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`:`"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn functio_missing_return_type_after_dat_is_error() {
    let e = parse_err("functio foo() dat :\n    dic \"x\".\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(
                expected.contains("type name"),
                "expected label was {expected:?}"
            );
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn lowercase_type_position_is_pascal_case_error() {
    let e = parse_err("functio f(^x: sermo) :\n    dic \"x\".\n");
    match e {
        ParseError::TypePositionRequiresPascalCase { name, .. } => {
            assert_eq!(name, "sermo");
        }
        other => panic!("expected TypePositionRequiresPascalCase, got {other:?}"),
    }
}

#[test]
fn return_type_lowercase_is_pascal_case_error() {
    let e = parse_err("functio f() dat numerus :\n    dic \"x\".\n");
    match e {
        ParseError::TypePositionRequiresPascalCase { name, .. } => {
            assert_eq!(name, "numerus");
        }
        other => panic!("expected TypePositionRequiresPascalCase, got {other:?}"),
    }
}

#[test]
fn param_missing_colon_is_error() {
    let e = parse_err("functio f(^x Sermo) :\n    dic \"x\".\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`:`"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn generics_attempt_is_lex_lookalike_error() {
    // Reaches the parser layer through MarainError::Lex; verifies that the
    // lexer's GenericsLookalike fires before any parser code can run.
    let mut map = SourceMap::new();
    let id = map.add(
        PathBuf::from("test.lat"),
        "functio f() dat Agmen<T> :\n    dic \"x\".\n".to_string(),
    );
    let err = lex(map.get(id)).expect_err("expected lex to fail");
    assert!(matches!(
        err,
        crate::lexer::LexError::GenericsLookalike { ch: '<', .. }
    ));
}

#[test]
fn redde_missing_period_is_error() {
    let e = parse_err("functio f() :\n    redde 5\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`.`"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn bare_plain_ident_in_expr_position_is_error() {
    // A PlainIdent without `(` is not a valid expression — variables must
    // carry a sigil per PRD §4.5.
    let e = parse_err("sit ^x est foo.\n");
    match e {
        ParseError::ExpectedExpression { .. } => {}
        other => panic!("expected ExpectedExpression, got {other:?}"),
    }
}

#[test]
fn call_as_statement_parses() {
    let m = parse_ok("saluta().\n");
    assert!(matches!(m.items[0], Stmt::Call(_)));
}

#[test]
fn call_stmt_with_args_parses() {
    let m = parse_ok("print(\"x\", 5).\n");
    match &m.items[0] {
        Stmt::Call(cs) => {
            assert_eq!(cs.call.callee.name, "print");
            assert_eq!(cs.call.args.len(), 2);
        }
        other => panic!("expected Call, got {other:?}"),
    }
}

#[test]
fn call_stmt_missing_period_is_error() {
    let e = parse_err("saluta()\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`.`"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn function_call_returns_value_to_let_binding() {
    // Round-trip integration: declaring + calling in one source.
    let m = parse_ok("functio answer() dat Numerus :\n    redde 42.\nsit ^x est answer().\n");
    assert_eq!(m.items.len(), 2);
    assert!(matches!(m.items[0], Stmt::Function(_)));
    assert!(matches!(m.items[1], Stmt::Let(_)));
}

// --- R14: range expressions ---

#[test]
fn range_exclusive_in_let_rhs() {
    let m = parse_ok("sit ^r est 0..10.\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::Range(r) => {
                assert!(!r.inclusive);
                assert!(matches!(
                    r.start.as_deref(),
                    Some(Expr::IntegerLit(IntegerLit { value: 0, .. }))
                ));
                assert!(matches!(
                    r.end.as_deref(),
                    Some(Expr::IntegerLit(IntegerLit { value: 10, .. }))
                ));
            }
            other => panic!("expected Range, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn range_inclusive_in_let_rhs() {
    let m = parse_ok("sit ^r est 0..=10.\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::Range(r) => assert!(r.inclusive),
            other => panic!("expected Range, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn range_with_binop_endpoints() {
    // `1 plus 2 .. 10 minus 3` — additive binds tighter than range.
    let m = parse_ok("sit ^r est 1 plus 2..10 minus 3.\n");
    match &m.items[0] {
        Stmt::Let(l) => match &l.value {
            Expr::Range(r) => {
                assert!(!r.inclusive);
                assert!(matches!(r.start.as_deref(), Some(Expr::BinOp(_))));
                assert!(matches!(r.end.as_deref(), Some(Expr::BinOp(_))));
            }
            other => panic!("expected Range, got {other:?}"),
        },
        other => panic!("expected Let, got {other:?}"),
    }
}

#[test]
fn range_missing_rhs_is_error() {
    // `0..` followed by statement terminator — RHS required (open ranges
    // deferred per B-1).
    let e = parse_err("sit ^r est 0...\n");
    assert!(matches!(e, ParseError::ExpectedExpression { .. }));
}

#[test]
fn range_at_statement_position_in_dic() {
    // `dic 0..10.` — range flows through macro arg.
    let m = parse_ok("dic 0..10.\n");
    match &m.items[0] {
        Stmt::MacroCall(c) => assert!(matches!(c.arg, Expr::Range(_))),
        other => panic!("expected MacroCall, got {other:?}"),
    }
}

// --- R14: `pro` for-loops ---

#[test]
fn pro_over_range_parses() {
    let m = parse_ok("pro ^i in 0..10 :\n    dic ^i.\n");
    match &m.items[0] {
        Stmt::For(f) => {
            assert_eq!(f.binding.name, "i");
            assert_eq!(f.binding.sigil, Sigil::Immutable);
            assert!(matches!(f.iter, Expr::Range(_)));
            assert_eq!(f.body.stmts.len(), 1);
        }
        other => panic!("expected For, got {other:?}"),
    }
}

#[test]
fn pro_over_inclusive_range_parses() {
    let m = parse_ok("pro ^i in 0..=10 :\n    dic ^i.\n");
    match &m.items[0] {
        Stmt::For(f) => match &f.iter {
            Expr::Range(r) => assert!(r.inclusive),
            other => panic!("expected Range, got {other:?}"),
        },
        other => panic!("expected For, got {other:?}"),
    }
}

#[test]
fn pro_with_mutable_binding_parses() {
    let m = parse_ok("pro @counter in 0..3 :\n    dic ^counter.\n");
    match &m.items[0] {
        Stmt::For(f) => {
            assert_eq!(f.binding.sigil, Sigil::Mutable);
            assert_eq!(f.binding.name, "counter");
        }
        other => panic!("expected For, got {other:?}"),
    }
}

#[test]
fn pro_over_var_ref_parses() {
    // Iterable need not be a range — any expression works.
    let m = parse_ok("sit ^xs est 0..3.\npro ^x in ^xs :\n    dic ^x.\n");
    assert_eq!(m.items.len(), 2);
    assert!(matches!(m.items[1], Stmt::For(_)));
}

#[test]
fn pro_missing_in_is_error() {
    let e = parse_err("pro ^i 0..10 :\n    dic ^i.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(
                expected.contains("`in`"),
                "expected label mentioned `in`; got: {expected:?}"
            );
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn pro_missing_sigil_on_binding_is_error() {
    let e = parse_err("pro i in 0..10 :\n    dic ^i.\n");
    assert!(matches!(e, ParseError::UnexpectedToken { .. }));
}

#[test]
fn pro_missing_colon_is_error() {
    let e = parse_err("pro ^i in 0..10\n    dic ^i.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`:`"), "got: {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

// --- R15: `nihil` ---

#[test]
fn bare_nihil_at_top_level_parses() {
    let m = parse_ok("nihil.\n");
    assert!(matches!(m.items[0], Stmt::Nihil(_)));
}

#[test]
fn nihil_inside_pro_body() {
    let m = parse_ok("pro ^i in 0..3 :\n    nihil.\n");
    match &m.items[0] {
        Stmt::For(f) => {
            assert_eq!(f.body.stmts.len(), 1);
            assert!(matches!(f.body.stmts[0], Stmt::Nihil(_)));
        }
        other => panic!("expected For, got {other:?}"),
    }
}

#[test]
fn nihil_inside_functio_body() {
    let m = parse_ok("functio stub() :\n    nihil.\n");
    match &m.items[0] {
        Stmt::Function(f) => {
            assert_eq!(f.body.stmts.len(), 1);
            assert!(matches!(f.body.stmts[0], Stmt::Nihil(_)));
        }
        other => panic!("expected Function, got {other:?}"),
    }
}

#[test]
fn nihil_missing_period_is_error() {
    let e = parse_err("nihil\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`.`"), "got: {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

// --- R16: `fit` reassignment ---

#[test]
fn fit_reassign_integer_parses() {
    let m = parse_ok("@x fit 5.\n");
    assert_eq!(m.items.len(), 1);
    match &m.items[0] {
        Stmt::Assign(a) => {
            assert_eq!(a.target.sigil, Sigil::Mutable);
            assert_eq!(a.target.name, "x");
            match &a.value {
                Expr::IntegerLit(i) => assert_eq!(i.value, 5),
                other => panic!("expected IntegerLit, got {other:?}"),
            }
        }
        other => panic!("expected Assign, got {other:?}"),
    }
}

#[test]
fn fit_reassign_increment_idiom_parses_binop_value() {
    // The accumulator idiom `@x fit @x plus 1.` — value is a BinOp over a VarRef.
    let m = parse_ok("@x fit @x plus 1.\n");
    match &m.items[0] {
        Stmt::Assign(a) => {
            assert_eq!(a.target.name, "x");
            match &a.value {
                Expr::BinOp(b) => {
                    assert!(matches!(b.lhs.as_ref(), Expr::VarRef(_)));
                    assert!(matches!(b.rhs.as_ref(), Expr::IntegerLit(_)));
                }
                other => panic!("expected BinOp, got {other:?}"),
            }
        }
        other => panic!("expected Assign, got {other:?}"),
    }
}

#[test]
fn fit_immutable_target_is_error() {
    // PRD §4.5: reassigning a `^` (immutable) target is rejected at parse time.
    let e = parse_err("^x fit 5.\n");
    match e {
        ParseError::ImmutableReassignmentTarget { name, .. } => assert_eq!(name, "x"),
        other => panic!("expected ImmutableReassignmentTarget, got {other:?}"),
    }
}

#[test]
fn fit_missing_period_is_error() {
    let e = parse_err("@x fit 5\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("`.`"), "got: {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

#[test]
fn sigiled_target_with_est_verb_is_error() {
    // `est` is the initializer copula (`sit ^x est …`), not a reassignment verb.
    // A sigiled-ident statement dispatches to parse_assign, which expects `fit`.
    let e = parse_err("@x est 5.\n");
    match e {
        ParseError::UnexpectedToken { expected, .. } => {
            assert!(expected.contains("fit"), "expected label was {expected:?}");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}
