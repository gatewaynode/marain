//! AST unit tests. Sibling-split from `ast.rs` per CLAUDE.md pressure-release
//! rule: ast.rs was at 646 LOC with tests dominating; tests live here in one
//! cohesive group that all share `sp` / `varref` / `fid` helpers, splitting
//! by R-round would force helper chasing.

use super::*;
use crate::span::FileId;

fn fid() -> FileId {
    FileId::new(1).expect("nonzero")
}

fn sp(start: u32, end: u32) -> Span {
    Span::new(start, end, fid())
}

fn varref(name: &str, start: u32, end: u32) -> Expr {
    Expr::VarRef(SigiledIdent::new(
        Sigil::Immutable,
        name.to_string(),
        sp(start, end),
    ))
}

#[test]
fn ident_new_defaults_inflection_to_none() {
    let i = Ident::new("dic".to_string(), sp(0, 3));
    assert_eq!(i.name, "dic");
    assert_eq!(i.span, sp(0, 3));
    assert!(i.inflection.is_none());
}

#[test]
fn sigiled_ident_new_defaults_inflection_to_none() {
    let s = SigiledIdent::new(Sigil::Immutable, "x".to_string(), sp(4, 6));
    assert_eq!(s.sigil, Sigil::Immutable);
    assert_eq!(s.name, "x");
    assert_eq!(s.span, sp(4, 6));
    assert!(s.inflection.is_none());
}

#[test]
fn stmt_span_dispatches_to_inner_node() {
    let let_stmt = Stmt::Let(LetStmt {
        name: SigiledIdent::new(Sigil::Immutable, "x".to_string(), sp(4, 6)),
        value: Expr::IntegerLit(IntegerLit {
            value: 5,
            span: sp(11, 12),
        }),
        span: sp(0, 13),
    });
    assert_eq!(let_stmt.span(), sp(0, 13));

    let mac = Stmt::MacroCall(MacroCallStmt {
        callee: Ident::new("dic".to_string(), sp(0, 3)),
        arg: Expr::StringLit(StringLit {
            value: "a".to_string(),
            span: sp(4, 7),
        }),
        span: sp(0, 8),
    });
    assert_eq!(mac.span(), sp(0, 8));
}

#[test]
fn expr_span_dispatches_to_inner_node() {
    let s = Expr::StringLit(StringLit {
        value: "hi".to_string(),
        span: sp(0, 4),
    });
    assert_eq!(s.span(), sp(0, 4));
    let i = Expr::IntegerLit(IntegerLit {
        value: 42,
        span: sp(5, 7),
    });
    assert_eq!(i.span(), sp(5, 7));
    let v = Expr::VarRef(SigiledIdent::new(
        Sigil::Mutable,
        "y".to_string(),
        sp(8, 10),
    ));
    assert_eq!(v.span(), sp(8, 10));
}

#[test]
fn inflection_default_is_empty_marker() {
    let _i: Inflection = Default::default();
}

#[test]
fn block_holds_stmts_and_span() {
    let b = Block {
        stmts: vec![Stmt::MacroCall(MacroCallStmt {
            callee: Ident::new("dic".to_string(), sp(0, 3)),
            arg: Expr::StringLit(StringLit {
                value: "hi".to_string(),
                span: sp(4, 8),
            }),
            span: sp(0, 9),
        })],
        span: sp(0, 9),
    };
    assert_eq!(b.stmts.len(), 1);
    assert_eq!(b.span, sp(0, 9));
}

#[test]
fn if_stmt_span_dispatches() {
    let i = Stmt::If(IfStmt {
        cond: varref("x", 3, 5),
        then_block: Block {
            stmts: vec![],
            span: sp(7, 15),
        },
        else_branch: None,
        span: sp(0, 15),
    });
    assert_eq!(i.span(), sp(0, 15));
}

#[test]
fn bool_lit_span() {
    let b = Expr::BoolLit(BoolLit {
        value: true,
        span: sp(0, 5),
    });
    assert_eq!(b.span(), sp(0, 5));
}

#[test]
fn binop_expr_span_and_as_rust() {
    let lhs = varref("a", 0, 2);
    let rhs = varref("b", 8, 10);
    let e = Expr::BinOp(BinOpExpr {
        op: BinOp::Plus,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        span: sp(0, 10),
    });
    assert_eq!(e.span(), sp(0, 10));
    assert_eq!(BinOp::Plus.as_rust(), "+");
    assert_eq!(BinOp::DivisusPer.as_rust(), "/");
    assert_eq!(BinOp::NonAequat.as_rust(), "!=");
    assert_eq!(BinOp::MinorVelPar.as_rust(), "<=");
    assert_eq!(BinOp::MaiorVelPar.as_rust(), ">=");
    assert_eq!(BinOp::Et.as_rust(), "&&");
    assert_eq!(BinOp::Vel.as_rust(), "||");
}

#[test]
fn unary_op_expr_span_and_as_rust() {
    let operand = varref("x", 4, 6);
    let e = Expr::UnaryOp(UnaryOpExpr {
        op: UnaryOp::Non,
        operand: Box::new(operand),
        span: sp(0, 6),
    });
    assert_eq!(e.span(), sp(0, 6));
    assert_eq!(UnaryOp::Non.as_rust(), "!");
}

#[test]
fn while_stmt_span_dispatches() {
    let w = Stmt::While(WhileStmt {
        cond: varref("x", 4, 6),
        body: Block {
            stmts: vec![],
            span: sp(8, 20),
        },
        span: sp(0, 20),
    });
    assert_eq!(w.span(), sp(0, 20));
}

#[test]
fn loop_stmt_span_dispatches() {
    let l = Stmt::Loop(LoopStmt {
        body: Block {
            stmts: vec![],
            span: sp(7, 15),
        },
        span: sp(0, 15),
    });
    assert_eq!(l.span(), sp(0, 15));
}

#[test]
fn break_continue_spans() {
    let b = Stmt::Break(BreakStmt { span: sp(0, 11) });
    let c = Stmt::Continue(ContinueStmt { span: sp(0, 9) });
    assert_eq!(b.span(), sp(0, 11));
    assert_eq!(c.span(), sp(0, 9));
}

#[test]
fn function_stmt_span_dispatches() {
    let f = Stmt::Function(FunctionStmt {
        name: Ident::new("saluta".to_string(), sp(8, 14)),
        params: vec![],
        return_type: None,
        body: Block {
            stmts: vec![],
            span: sp(18, 30),
        },
        span: sp(0, 30),
    });
    assert_eq!(f.span(), sp(0, 30));
}

#[test]
fn return_stmt_span_dispatches() {
    let r_with = Stmt::Return(ReturnStmt {
        value: Some(Expr::IntegerLit(IntegerLit {
            value: 42,
            span: sp(6, 8),
        })),
        span: sp(0, 9),
    });
    assert_eq!(r_with.span(), sp(0, 9));

    let r_bare = Stmt::Return(ReturnStmt {
        value: None,
        span: sp(0, 6),
    });
    assert_eq!(r_bare.span(), sp(0, 6));
}

#[test]
fn param_holds_name_type_and_span() {
    let p = Param {
        name: SigiledIdent::new(Sigil::Immutable, "x".to_string(), sp(0, 2)),
        type_ref: TypeRef {
            name: Ident::new("Sermo".to_string(), sp(4, 9)),
            span: sp(4, 9),
        },
        span: sp(0, 9),
    };
    assert_eq!(p.name.name, "x");
    assert_eq!(p.type_ref.name.name, "Sermo");
    assert_eq!(p.span, sp(0, 9));
}

#[test]
fn type_ref_wraps_ident() {
    let t = TypeRef {
        name: Ident::new("Numerus".to_string(), sp(0, 7)),
        span: sp(0, 7),
    };
    assert_eq!(t.name.name, "Numerus");
    assert!(t.name.inflection.is_none());
}

#[test]
fn call_stmt_span_dispatches() {
    let cs = Stmt::Call(CallStmt {
        call: CallExpr {
            callee: Ident::new("saluta".to_string(), sp(0, 6)),
            args: vec![],
            span: sp(0, 8),
        },
        span: sp(0, 9),
    });
    assert_eq!(cs.span(), sp(0, 9));
}

#[test]
fn call_expr_span_dispatches() {
    let c = Expr::Call(CallExpr {
        callee: Ident::new("foo".to_string(), sp(0, 3)),
        args: vec![
            Expr::IntegerLit(IntegerLit {
                value: 1,
                span: sp(4, 5),
            }),
            Expr::IntegerLit(IntegerLit {
                value: 2,
                span: sp(7, 8),
            }),
        ],
        span: sp(0, 9),
    });
    assert_eq!(c.span(), sp(0, 9));
    if let Expr::Call(ce) = &c {
        assert_eq!(ce.args.len(), 2);
        assert_eq!(ce.callee.name, "foo");
    }
}

#[test]
fn else_branch_block_and_if_shapes() {
    let block = ElseBranch::Block(Block {
        stmts: vec![],
        span: sp(20, 30),
    });
    assert_eq!(block.span(), sp(20, 30));

    let inner = IfStmt {
        cond: varref("y", 22, 24),
        then_block: Block {
            stmts: vec![],
            span: sp(26, 36),
        },
        else_branch: None,
        span: sp(20, 36),
    };
    let chained = ElseBranch::If(Box::new(inner));
    assert_eq!(chained.span(), sp(20, 36));
}

#[test]
fn range_expr_span_dispatches_through_expr() {
    let r = Expr::Range(RangeExpr {
        start: Some(Box::new(Expr::IntegerLit(IntegerLit {
            value: 0,
            span: sp(0, 1),
        }))),
        end: Some(Box::new(Expr::IntegerLit(IntegerLit {
            value: 10,
            span: sp(3, 5),
        }))),
        inclusive: false,
        span: sp(0, 5),
    });
    assert_eq!(r.span(), sp(0, 5));
}

#[test]
fn for_stmt_span_dispatches_through_stmt() {
    let f = Stmt::For(ForStmt {
        binding: SigiledIdent::new(Sigil::Immutable, "i".to_string(), sp(4, 6)),
        iter: Expr::IntegerLit(IntegerLit {
            value: 10,
            span: sp(10, 12),
        }),
        body: Block {
            stmts: vec![],
            span: sp(14, 24),
        },
        span: sp(0, 24),
    });
    assert_eq!(f.span(), sp(0, 24));
}

#[test]
fn nihil_stmt_span_dispatches_through_stmt() {
    let n = Stmt::Nihil(NihilStmt { span: sp(0, 7) });
    assert_eq!(n.span(), sp(0, 7));
}

#[test]
fn assign_stmt_span_dispatches_through_stmt() {
    let a = Stmt::Assign(AssignStmt {
        target: SigiledIdent::new(Sigil::Mutable, "x".to_string(), sp(0, 2)),
        value: Expr::IntegerLit(IntegerLit {
            value: 5,
            span: sp(7, 8),
        }),
        span: sp(0, 9),
    });
    assert_eq!(a.span(), sp(0, 9));
}
