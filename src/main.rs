use anyhow::Result;
use clap::Parser;

mod cli;
mod core;
mod tui;
mod util;

use cli::{Commands, DotsmithCli, RepoAction};

fn main() -> Result<()> {
    let cli = DotsmithCli::parse();

    let result = match cli.command {
        None => tui::run(None),
        Some(Commands::Explore { ref tool }) => tui::run(Some(tool)),
        Some(Commands::Init) => cli::init::run(cli.verbose),
        Some(Commands::Add { ref tool }) => cli::add::run(cli.verbose, tool),
        Some(Commands::Remove { ref tool }) => cli::remove::run(cli.verbose, tool),
        Some(Commands::List) => cli::list::run(cli.verbose),
        Some(Commands::Status) => cli::status::run(cli.verbose),
        Some(Commands::Snapshot {
            ref tool,
            ref message,
        }) => cli::snapshot::run(cli.verbose, tool.as_deref(), message.as_deref()),
        Some(Commands::History { ref tool, limit }) => cli::history::run(cli.verbose, tool, limit),
        Some(Commands::Diff { ref tool }) => cli::diff::run(cli.verbose, tool.as_deref()),
        Some(Commands::Rollback {
            snapshot_id,
            dry_run,
        }) => cli::rollback::run(cli.verbose, snapshot_id, dry_run),
        Some(Commands::Deploy {
            ref source,
            ref target,
            dry_run,
        }) => cli::deploy::run(cli.verbose, source, target, dry_run),
        Some(Commands::Reload { ref tool }) => cli::reload::run(cli.verbose, tool),
        Some(Commands::Plugins {
            ref tool,
            ref action,
        }) => cli::plugins::run(cli.verbose, tool, action),
        Some(Commands::Repo { action }) => match action {
            RepoAction::Init { path } => cli::repo::run_init(cli.verbose, &path),
            RepoAction::Sync => cli::repo::run_sync(cli.verbose),
            RepoAction::Status => cli::repo::run_status(cli.verbose),
        },
    };

    if let Err(e) = result {
        eprintln!("{}: {}", colored::Colorize::red("error"), e);
        std::process::exit(1);
    }

    Ok(())
}
