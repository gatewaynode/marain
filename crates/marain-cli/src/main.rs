//! The `marain` binary — thin shim over [`crate::driver`].
//!
//! Parses arguments, dispatches, reports any error to stderr, and exits
//! with `0` on success or `1` on any failure (PRD §6, ARCHITECTURE.md §9).

#![forbid(unsafe_code)]

use clap::Parser;

mod args;
mod driver;
mod error;
mod paths;

fn main() {
    let cli = args::Cli::parse();
    let code = match driver::dispatch(cli) {
        Ok(()) => 0,
        Err(e) => {
            e.report();
            1
        }
    };
    std::process::exit(code);
}
