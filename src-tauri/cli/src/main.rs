use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::env;

mod commands;
mod utils;

use commands::{config, health};
use utils::project_root;

/// Marain CLI - Command line interface for the Marain CMS
#[derive(Parser)]
#[command(name = "marc")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check system health and status
    Health {
        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Configuration management commands
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// List all loaded configurations
    List {
        /// Output format (json, yaml, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Read a specific configuration section
    Get {
        /// Configuration section path (e.g., "system.database.path")
        section: String,

        /// Output format (json, yaml, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    // Validate we're in the project root
    let project_root = match project_root::find_project_root() {
        Ok(root) => root,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            eprintln!(
                "{}",
                "marc must be run from the Marain project root directory".yellow()
            );
            std::process::exit(1);
        }
    };

    // Set the current directory to project root
    env::set_current_dir(&project_root)?;

    // Execute the command
    match cli.command {
        Commands::Health { format } => {
            health::execute(format).await?;
        }
        Commands::Config { action } => match action {
            ConfigAction::List { format } => {
                config::list(format).await?;
            }
            ConfigAction::Get { section, format } => {
                config::get(section, format).await?;
            }
        },
    }

    Ok(())
}
