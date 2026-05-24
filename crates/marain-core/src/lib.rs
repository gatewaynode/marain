//! Marain compiler front-end.
//!
//! Houses the lexer, parser, AST, Rust emitter, and cargo-shim generator.
//! Module decomposition lands as each architecture design round closes.

#![forbid(unsafe_code)]

pub mod ast;
pub mod emit;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod shim;
pub mod source;
pub mod span;
pub mod token;
