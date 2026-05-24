//! Stage 1 abstract syntax tree.
//!
//! Every node carries a [`Span`]. Identifier-bearing nodes carry an
//! `Option<Inflection>` slot that Stage 1 never populates; the slot exists so
//! Stage 2 can grow Latin grammar metadata without changing every AST
//! construction site (carry-over concern α, ARCHITECTURE.md §11).

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
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Self::Let(s) => s.span,
            Self::MacroCall(s) => s.span,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    StringLit(StringLit),
    IntegerLit(IntegerLit),
    VarRef(SigiledIdent),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::StringLit(e) => e.span,
            Self::IntegerLit(e) => e.span,
            Self::VarRef(s) => s.span,
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
}
