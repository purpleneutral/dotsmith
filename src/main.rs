use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

mod cli;
mod core;
mod tui;
mod util;

use cli::{Commands, DotsmithCli, RepoAction};

fn main() -> Result<()> {
    let cli = DotsmithCli::parse();

    // Auto-initialize for commands that need config infrastructure.
    // Skip for: Init (has its own UX), Completions, Mangen, Search (standalone).
    let skip_init = matches!(
        cli.command,
        Some(Commands::Init)
            | Some(Commands::Completions { .. })
            | Some(Commands::Mangen)
            | Some(Commands::Search { .. })
    );
    if !skip_init {
        cli::init::ensure_initialized()?;
    }

    let result = match cli.command {
        None => tui::run(None),
        Some(Commands::Explore { ref tool }) => tui::run(Some(tool)),
        Some(Commands::Init) => cli::init::run(cli.verbose),
        Some(Commands::Add { ref tool }) => cli::add::run(cli.verbose, tool),
        Some(Commands::Remove { ref tool }) => cli::remove::run(cli.verbose, tool),
        Some(Commands::List) => cli::list::run(cli.verbose),
        Some(Commands::Status) => cli::status::run(cli.verbose),
        Some(Commands::Doctor { ref tool }) => cli::doctor::run(cli.verbose, tool.as_deref()),
        Some(Commands::Search { ref query }) => cli::search::run(cli.verbose, query),
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
        Some(Commands::Edit { ref tool }) => cli::edit::run(cli.verbose, tool),
        Some(Commands::Watch { ref tool }) => cli::watch::run(cli.verbose, tool.as_deref()),
        Some(Commands::Reload { ref tool }) => cli::reload::run(cli.verbose, tool),
        Some(Commands::Plugins {
            ref tool,
            ref action,
        }) => cli::plugins::run(cli.verbose, tool, action),
        Some(Commands::Completions { shell }) => {
            let mut cmd = DotsmithCli::command();
            generate(shell, &mut cmd, "dotsmith", &mut std::io::stdout());
            Ok(())
        }
        Some(Commands::Mangen) => {
            let cmd = DotsmithCli::command();
            let man = clap_mangen::Man::new(cmd);
            man.render(&mut std::io::stdout())?;
            Ok(())
        }
        Some(Commands::Profile { ref action }) => cli::profile::run(cli.verbose, action),
        Some(Commands::DeployRemote {
            ref host,
            ref user,
            ref tool,
            dry_run,
        }) => cli::deploy_remote::run(cli.verbose, host, user.as_deref(), tool.as_deref(), dry_run),
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
