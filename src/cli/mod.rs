pub mod add;
pub mod deploy;
pub mod diff;
pub mod history;
pub mod init;
pub mod list;
pub mod plugins;
pub mod reload;
pub mod remove;
pub mod rollback;
pub mod snapshot;
pub mod status;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "dotsmith",
    about = "The dotfile workbench — explore, manage, and master your configs",
    version
)]
pub struct DotsmithCli {
    #[command(subcommand)]
    pub command: Option<Commands>,

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

    /// Take a snapshot of config files
    Snapshot {
        /// Tool name (snapshots all tools if omitted)
        tool: Option<String>,

        /// Message to attach to this snapshot
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Show snapshot history for a tool
    History {
        /// Tool name
        tool: String,

        /// Maximum number of entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Show diff between current configs and last snapshot
    Diff {
        /// Tool name (diffs all tools if omitted)
        tool: Option<String>,
    },

    /// Rollback a config file to a specific snapshot
    Rollback {
        /// Snapshot ID to rollback to (from history output)
        snapshot_id: i64,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Deploy config symlinks from source to target
    Deploy {
        /// Source path (where config files live)
        source: String,

        /// Target path (where symlinks should be created)
        target: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// Reload configuration for a running tool
    Reload {
        /// Tool name to reload
        tool: String,
    },

    /// Explore config options for a tool (interactive TUI)
    Explore {
        /// Tool name (e.g., tmux, zsh, git)
        tool: String,
    },

    /// Manage plugins for a tool (zsh, tmux)
    Plugins {
        /// Tool name (e.g., zsh, tmux)
        tool: String,

        #[command(subcommand)]
        action: PluginAction,
    },
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// Add a plugin (GitHub shorthand: user/repo)
    Add {
        /// Plugin repository (e.g., zsh-users/zsh-autosuggestions)
        repo: String,
    },

    /// Remove an installed plugin
    Remove {
        /// Plugin name to remove
        name: String,
    },

    /// List installed plugins
    List,

    /// Update one or all plugins
    Update {
        /// Plugin name (updates all if omitted)
        name: Option<String>,
    },
}
