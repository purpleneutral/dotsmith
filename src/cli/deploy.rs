use anyhow::Result;
use colored::Colorize;

use crate::core::deploy::{self, DeployActionType};
use crate::util;

/// Deploy config files by creating symlinks from a source to a target.
pub fn run(
    verbose: bool,
    source: &str,
    target: &str,
    dry_run: bool,
) -> Result<()> {
    let source_path = util::paths::expand_tilde(source);
    let target_path = util::paths::expand_tilde(target);

    // Safety check
    util::fs::check_path_safety(&source_path)?;
    util::fs::check_path_safety(&target_path)?;

    let actions = deploy::plan_deploy(&source_path, &target_path)?;

    if actions.is_empty() {
        println!("Nothing to deploy.");
        return Ok(());
    }

    // Display plan
    for action in &actions {
        let status = match action.action {
            DeployActionType::CreateSymlink => "create".green(),
            DeployActionType::AlreadyCorrect => "ok".dimmed(),
            DeployActionType::BackupAndLink => "backup+link".yellow(),
            DeployActionType::Relink => "relink".yellow(),
            DeployActionType::SourceMissing => "missing".red(),
        };

        println!(
            "  [{}] {} -> {}",
            status,
            action.target.display(),
            action.source.display()
        );
    }

    if dry_run {
        println!();
        println!(
            "{} No changes made (dry run)",
            "[dry-run]".yellow().bold()
        );
        return Ok(());
    }

    // Check if any actions need user attention
    let needs_action = actions.iter().any(|a| {
        matches!(
            a.action,
            DeployActionType::CreateSymlink
                | DeployActionType::BackupAndLink
                | DeployActionType::Relink
        )
    });

    if !needs_action {
        println!("All symlinks are already correct.");
        return Ok(());
    }

    let config_dir = util::paths::config_dir()?;
    let backup_dir = config_dir.join("backups");

    let backed_up = deploy::execute_deploy(&actions, &backup_dir)?;

    println!();
    println!("{} Deploy complete", "OK".green().bold());

    if !backed_up.is_empty() {
        println!(
            "  {} file(s) backed up to {}",
            backed_up.len(),
            backup_dir.display()
        );
        if verbose {
            for path in &backed_up {
                println!("    {}", path.display());
            }
        }
    }

    Ok(())
}
