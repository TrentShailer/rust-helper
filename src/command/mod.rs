//! A basic CLI for generic applications.

pub mod config_command;

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
    /// Config subcommand.
    #[command(subcommand)]
    Config(config_command::ConfigSubcommand),
}

impl Cli {
    /// Parse the CLI arguments.
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
