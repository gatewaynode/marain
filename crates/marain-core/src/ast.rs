//! Stage 1 abstract syntax tree.
//!
//! Every node carries a [`Span`]. Identifier-bearing nodes carry an
//! `Option<Inflection>` slot that Stage 1 never populates; the slot exists so
//! Stage 2 can grow Latin grammar metadata without changing every AST
//! construction site (carry-over concern α, ARCHITECTURE.md §11).
//!
//! Naming convention: enum variants whose name mirrors a *user-facing Marain
//! keyword* — `Stmt::Let` / `Stmt::If` / `Stmt::While` / etc. — track the Rust
//! lowering target rather than the Latin spelling. Variants that name an
//! *operator surface* — `BinOp::Plus` / `BinOp::Aequat` / etc. — use the Latin
//! spelling because the parser sees Latin tokens, not Rust symbols. FUTURE: if
//! this file crowds the 500-LOC target, split as `ast/{stmt,expr,item}.rs`.

use crate::span::Span;
use crate::token::Sigil;

/// Latin morphological inflection placeholder.
///
/// Empty in Stage 1; its purpose is to reserve the field's *type* so Stage 2
/// can extend it without churning Stage 1 parser sites.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Inflection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Module {
    pub items: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
    MacroCall(MacroCallStmt),
    If(IfStmt),
    While(WhileStmt),
    Loop(LoopStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Self::Let(s) => s.span,
            Self::MacroCall(s) => s.span,
            Self::If(s) => s.span,
            Self::While(s) => s.span,
            Self::Loop(s) => s.span,
            Self::Break(s) => s.span,
            Self::Continue(s) => s.span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LetStmt {
    pub name: SigiledIdent,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MacroCallStmt {
    pub callee: Ident,
    pub arg: Expr,
    pub span: Span,
}

/// `si <cond> :` with an indented `then` body, optionally followed by an
/// `aliter :` block or an `aliter si <cond> :` chain (both encoded via
/// [`ElseBranch`]; the `aliter si` arm boxes a nested [`IfStmt`] so a
/// multi-arm chain nests inside `else_branch`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_branch: Option<ElseBranch>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ElseBranch {
    /// `aliter :` — terminal else block.
    Block(Block),
    /// `aliter si <cond> :` — `else if`, boxed to allow chains.
    If(Box<IfStmt>),
}

impl ElseBranch {
    pub fn span(&self) -> Span {
        match self {
            Self::Block(b) => b.span,
            Self::If(i) => i.span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopStmt {
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakStmt {
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinueStmt {
    pub span: Span,
}

/// An indented sequence of statements. Span covers the `Indent`..`Dedent`
/// region (inclusive of both layout tokens).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    StringLit(StringLit),
    IntegerLit(IntegerLit),
    BoolLit(BoolLit),
    VarRef(SigiledIdent),
    BinOp(BinOpExpr),
    UnaryOp(UnaryOpExpr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::StringLit(e) => e.span,
            Self::IntegerLit(e) => e.span,
            Self::BoolLit(e) => e.span,
            Self::VarRef(s) => s.span,
            Self::BinOp(b) => b.span,
            Self::UnaryOp(u) => u.span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringLit {
    pub value: String,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerLit {
    pub value: i64,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoolLit {
    pub value: bool,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BinOpExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: Span,
}

/// Latin operator surface. Compound names (`DivisusPer`, `NonAequat`,
/// `MinorQuam`, …) reflect the multi-word phrases their parser-level rule
/// consumes (PRD §4.4).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinOp {
    Plus,        // +
    Minus,       // -
    Per,         // *
    DivisusPer,  // /
    Modulo,      // %
    Aequat,      // ==
    NonAequat,   // !=
    MinorQuam,   // <
    MaiorQuam,   // >
    MinorVelPar, // <=
    MaiorVelPar, // >=
    Et,          // &&
    Vel,         // ||
}

impl BinOp {
    /// Rust source representation of this operator.
    pub fn as_rust(self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Per => "*",
            Self::DivisusPer => "/",
            Self::Modulo => "%",
            Self::Aequat => "==",
            Self::NonAequat => "!=",
            Self::MinorQuam => "<",
            Self::MaiorQuam => ">",
            Self::MinorVelPar => "<=",
            Self::MaiorVelPar => ">=",
            Self::Et => "&&",
            Self::Vel => "||",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
    pub span: Span,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Non, // !
}

impl UnaryOp {
    pub fn as_rust(self) -> &'static str {
        match self {
            Self::Non => "!",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
    pub inflection: Option<Inflection>,
}

impl Ident {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            name,
            span,
            inflection: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SigiledIdent {
    pub sigil: Sigil,
    pub name: String,
    pub span: Span,
    pub inflection: Option<Inflection>,
}

impl SigiledIdent {
    pub fn new(sigil: Sigil, name: String, span: Span) -> Self {
        Self {
            sigil,
            name,
            span,
            inflection: None,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
