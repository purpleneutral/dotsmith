pub mod add;
pub mod deploy;
pub mod deploy_remote;
pub mod diff;
pub mod doctor;
pub mod edit;
pub mod history;
pub mod init;
pub mod list;
pub mod plugins;
pub mod profile;
pub mod reload;
pub mod remove;
pub mod repo;
pub mod rollback;
pub mod search;
pub mod snapshot;
pub mod status;
pub mod watch;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

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

    /// Run health checks on tracked tools and configuration
    Doctor {
        /// Specific tool to check (checks all if omitted)
        tool: Option<String>,
    },

    /// Search config options across all Tier 1 tool databases
    Search {
        /// Search query (matches option names, descriptions, and tags)
        query: String,
    },

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

    /// Open a tool's config file in your editor (auto-snapshots before editing)
    Edit {
        /// Tool name to edit
        tool: String,
    },

    /// Watch tracked configs for changes and auto-snapshot on save
    Watch {
        /// Specific tool to watch (watches all if omitted)
        tool: Option<String>,
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

    /// Generate shell completions for bash, zsh, or fish
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Generate man page (hidden, for packaging)
    #[command(hide = true)]
    Mangen,

    /// Manage configuration profiles (save/load/list/delete)
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Deploy tracked configs to a remote host via SSH
    DeployRemote {
        /// Remote host (hostname, IP, or SSH alias from ~/.ssh/config)
        host: String,

        /// SSH user (defaults to current user / ssh config)
        #[arg(short, long)]
        user: Option<String>,

        /// Deploy only specific tool(s) (can be specified multiple times)
        #[arg(short, long)]
        tool: Option<Vec<String>>,

        /// Preview changes without copying anything
        #[arg(long)]
        dry_run: bool,
    },

    /// Manage dotfile git repo for backups
    Repo {
        #[command(subcommand)]
        action: RepoAction,
    },
}

#[derive(Subcommand)]
pub enum RepoAction {
    /// Initialize a git repo for dotfile backups
    Init {
        /// Path to create the repo at (e.g., ~/dots)
        path: String,
    },

    /// Sync tracked configs into the repo and commit
    Sync,

    /// Show repo status
    Status,
}

#[derive(Subcommand)]
pub enum ProfileAction {
    /// Save current configs as a named profile
    Save {
        /// Profile name (e.g., workstation, laptop, minimal)
        name: String,
    },

    /// Restore config files from a saved profile
    Load {
        /// Profile name to load
        name: String,

        /// Add tools from the profile that aren't currently tracked
        #[arg(long)]
        add_untracked: bool,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,
    },

    /// List saved profiles
    List,

    /// Delete a saved profile
    Delete {
        /// Profile name to delete
        name: String,
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
