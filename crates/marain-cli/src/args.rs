//! Command-line argument schema for the `marain` binary.
//!
//! Uses `clap`'s derive API: top-level [`Cli`] with two subcommands —
//! [`Command::Build`] and [`Command::Run`] — each carrying one positional
//! `<file.lat>` path. `--help` and `--version` come from clap.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "marain",
    version,
    about = "The Marain transpiler — Rust through Latin."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Transpile a .lat source to a generated cargo project. On success,
    /// prints the shim project path to stdout so it can be `cd`-ed into.
    Build {
        /// Path to the Marain source file.
        path: PathBuf,
    },
    /// Transpile and execute. Forwards the user program's stdout/stderr
    /// verbatim and exits with cargo's exit code.
    Run {
        /// Path to the Marain source file.
        path: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_command_is_well_formed() {
        // clap's own internal-consistency check; panics on a malformed schema.
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_build_subcommand() {
        let cli = Cli::try_parse_from(["marain", "build", "hello.lat"]).expect("parses");
        match cli.command {
            Command::Build { path } => assert_eq!(path, PathBuf::from("hello.lat")),
            Command::Run { .. } => panic!("expected Build"),
        }
    }

    #[test]
    fn parse_run_subcommand() {
        let cli = Cli::try_parse_from(["marain", "run", "src/hello.lat"]).expect("parses");
        match cli.command {
            Command::Run { path } => assert_eq!(path, PathBuf::from("src/hello.lat")),
            Command::Build { .. } => panic!("expected Run"),
        }
    }

    #[test]
    fn missing_subcommand_is_error() {
        assert!(Cli::try_parse_from(["marain"]).is_err());
    }

    #[test]
    fn missing_path_is_error() {
        assert!(Cli::try_parse_from(["marain", "build"]).is_err());
        assert!(Cli::try_parse_from(["marain", "run"]).is_err());
    }

    #[test]
    fn unknown_subcommand_is_error() {
        assert!(Cli::try_parse_from(["marain", "frobnicate", "hello.lat"]).is_err());
    }

    #[test]
    fn help_flag_is_display_help() {
        let err = Cli::try_parse_from(["marain", "--help"]).expect_err("--help returns Err");
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn version_flag_is_display_version() {
        let err = Cli::try_parse_from(["marain", "--version"]).expect_err("--version returns Err");
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    }
}
