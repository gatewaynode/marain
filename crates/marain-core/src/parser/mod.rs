//! Parser driver.
//!
//! Owns the token cursor and exposes `parse(&[Token]) -> Result<Module, ParseError>`.
//! Per-production grammar functions live in [`grammar`].

mod error;
mod grammar;

pub use error::ParseError;

use crate::ast::Module;
use crate::span::Span;
use crate::token::{Token, TokenKind};

/// Parse a token stream into a Stage 1 [`Module`].
///
/// `tokens` must end with [`TokenKind::Eof`] (the lexer's contract).
pub fn parse(tokens: &[Token]) -> Result<Module, ParseError> {
    let mut parser = Parser::new(tokens);
    grammar::parse_module(&mut parser)
}

/// Token cursor. Internal to the parser module.
pub(crate) struct Parser<'tokens> {
    tokens: &'tokens [Token],
    pos: usize,
}

impl<'tokens> Parser<'tokens> {
    fn new(tokens: &'tokens [Token]) -> Self {
        debug_assert!(
            matches!(tokens.last().map(|t| &t.kind), Some(TokenKind::Eof)),
            "parser requires a token stream terminated with Eof",
        );
        Self { tokens, pos: 0 }
    }

    pub(crate) fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    /// Peek the kind `offset` tokens ahead without advancing. Past-end peeks
    /// clamp to the trailing [`TokenKind::Eof`] (the lexer's contract guarantees
    /// it is the last token).
    pub(crate) fn peek_kind_at(&self, offset: usize) -> &TokenKind {
        let idx = self.pos + offset;
        let last = self.tokens.len() - 1;
        let safe = if idx <= last { idx } else { last };
        &self.tokens[safe].kind
    }

    pub(crate) fn peek_span(&self) -> Span {
        self.tokens[self.pos].span
    }

    pub(crate) fn current_clone(&self) -> Token {
        self.tokens[self.pos].clone()
    }

    pub(crate) fn advance(&mut self) {
        if !matches!(self.peek_kind(), TokenKind::Eof) {
            self.pos += 1;
        }
    }

    pub(crate) fn at_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
