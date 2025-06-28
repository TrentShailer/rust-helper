//! A basic CLI for generic applications.

use clap::{Parser, Subcommand};

/// A basic CLI.
#[derive(Debug, Parser)]
pub struct Cli {
    /// The subcommand
    #[command(subcommand)]
    pub subcommand: Option<Command>,

    /// Enable verbose logging.
    #[arg(long, action)]
    pub verbose: bool,
}

/// Subcommands for the CLI.
#[derive(Debug, Subcommand)]
pub enum Command {
    #[cfg(feature = "config-command")]
    /// Config subcommand.
    #[command(subcommand)]
    Config(crate::config_command::ConfigSubcommand),
}
