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
    Assign(AssignStmt),
    MacroCall(MacroCallStmt),
    If(IfStmt),
    While(WhileStmt),
    Loop(LoopStmt),
    For(ForStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Function(FunctionStmt),
    Return(ReturnStmt),
    Call(CallStmt),
    Nihil(NihilStmt),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Self::Let(s) => s.span,
            Self::Assign(s) => s.span,
            Self::MacroCall(s) => s.span,
            Self::If(s) => s.span,
            Self::While(s) => s.span,
            Self::Loop(s) => s.span,
            Self::For(s) => s.span,
            Self::Break(s) => s.span,
            Self::Continue(s) => s.span,
            Self::Function(s) => s.span,
            Self::Return(s) => s.span,
            Self::Call(s) => s.span,
            Self::Nihil(s) => s.span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LetStmt {
    pub name: SigiledIdent,
    pub value: Expr,
    pub span: Span,
}

/// `@x fit <expr> .` per PRD §4.4 (reassign copula). Re-binds an
/// already-declared mutable binding; mirrors [`LetStmt`] minus the `est`
/// copula. `target` always carries the `@` (mutable) sigil — the parser
/// rejects a `^` target outright (PRD §4.5: the sigil marks mutability at
/// every use site). Field and index targets (`@x.y`, `@x[i]`) are out of
/// scope until method-call / index syntax lands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssignStmt {
    pub target: SigiledIdent,
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

/// `pro <binding> in <iter> :` per PRD §4.11.2. `binding` is a sigiled
/// identifier (sigil drops at emit time; `@` → `mut`). `iter` is any
/// expression — typically a range literal (`0..10`) or a variable reference
/// to a collection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForStmt {
    pub binding: SigiledIdent,
    pub iter: Expr,
    pub body: Block,
    pub span: Span,
}

/// `nihil.` per PRD §4.11.4. The "do nothing on purpose" sentinel; lowers to
/// a Rust unit-statement (`();`) so it satisfies the "block must contain at
/// least one statement" rule without committing to behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NihilStmt {
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

/// `functio <name>(<params>) [dat <Tipus>] : <body>` per PRD §4.11.1.
///
/// `return_type` is `None` when the `dat` clause is omitted — the emitter
/// produces no `-> ...` annotation and Rust infers `()`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionStmt {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: Block,
    pub span: Span,
}

/// A single parameter in a `functio` signature: `<sigiled-name>: <Tipus>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Param {
    pub name: SigiledIdent,
    pub type_ref: TypeRef,
    pub span: Span,
}

/// A type-position identifier. Stage 1 has no generics — `name` is always a
/// bare PascalCase [`Ident`]. The newtype reserves a shape that v0.3+ can grow
/// without touching every type-position consumer (carry-over hook for generics).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeRef {
    pub name: Ident,
    pub span: Span,
}

/// `redde [<expr>] .` — `value` is `None` for bare `redde.` (unit return,
/// matches Rust's `return;`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

/// A bare call at statement position: `<name>(<args>) .` Lowers to a Rust
/// expression-statement; side effects only (the return value, if any, is
/// discarded — rustc's `unused_must_use` lint can adjudicate).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallStmt {
    pub call: CallExpr,
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
    FString(FStringLit),
    IntegerLit(IntegerLit),
    BoolLit(BoolLit),
    VarRef(SigiledIdent),
    BinOp(BinOpExpr),
    UnaryOp(UnaryOpExpr),
    Call(CallExpr),
    Range(RangeExpr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::StringLit(e) => e.span,
            Self::FString(e) => e.span,
            Self::IntegerLit(e) => e.span,
            Self::BoolLit(e) => e.span,
            Self::VarRef(s) => s.span,
            Self::BinOp(b) => b.span,
            Self::UnaryOp(u) => u.span,
            Self::Call(c) => c.span,
            Self::Range(r) => r.span,
        }
    }
}

/// A range expression: `a..b` (exclusive) or `a..=b` (inclusive). v0.2 only
/// produces fully-bounded ranges from the parser; `start` and `end` are
/// `Option` so the shape covers all six Rust range variants when open-ended
/// forms land in a future round.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RangeExpr {
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub inclusive: bool,
    pub span: Span,
}

/// A function-call expression: `<callee>(<args>)`. Stage 1 callees are bare
/// [`Ident`]s — no module paths, no function-as-value semantics (a `^f(...)`
/// shape is rejected at parse time).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallExpr {
    pub callee: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringLit {
    pub value: String,
    pub span: Span,
}

/// An f-string literal `f"…{^x}…"` (R17) — sugar over `format!`. An ordered
/// list of literal-text and variable-interpolation parts; emit lowers it to
/// `format!("…{}…", x, …)`. Holes are variable-refs-only (a future round may
/// admit full expressions); concatenation is the all-holes form `f"{^a}{^b}"`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FStringLit {
    pub parts: Vec<FStringPart>,
    pub span: Span,
}

/// One part of an [`FStringLit`]: verbatim text, or a single interpolated
/// variable. Mirrors [`crate::token::FStringSeg`] but carries a resolved
/// [`SigiledIdent`] so emit reuses the ordinary variable-reference path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FStringPart {
    Literal(String),
    Interp(SigiledIdent),
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
#[path = "ast_tests.rs"]
mod tests;
