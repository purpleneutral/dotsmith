pub mod add;
pub mod init;
pub mod list;
pub mod remove;
pub mod status;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "dotsmith",
    about = "The dotfile workbench — explore, manage, and master your configs",
    version,
    arg_required_else_help = true
)]
pub struct DotsmithCli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize dotsmith configuration directory
    Init,

    /// Add a tool to dotsmith management
    Add {
        /// Tool name (e.g., tmux, zsh, git)
        tool: String,
    },

    /// Remove a tool from dotsmith management
    Remove {
        /// Tool name to remove
        tool: String,
    },

    /// List all managed tools with status
    List,

    /// Show recent changes and warnings
    Status,
}
