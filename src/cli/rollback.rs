use anyhow::Result;
use colored::Colorize;

use crate::core::snapshot::SnapshotEngine;
use crate::util;

/// Rollback a config file to a specific snapshot.
pub fn run(verbose: bool, snapshot_id: i64, dry_run: bool) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let engine = SnapshotEngine::open(&config_dir)?;

    // First, show what would be rolled back
    let (file_path, content) = engine
        .get_snapshot(snapshot_id)?
        .ok_or_else(|| anyhow::anyhow!("snapshot #{} not found", snapshot_id))?;

    println!(
        "Snapshot #{}: {} ({} bytes)",
        snapshot_id,
        file_path.bold(),
        content.len()
    );

    if dry_run {
        println!("{} Would rollback {} to snapshot #{}", "[dry-run]".yellow().bold(), file_path, snapshot_id);

        // Show diff between current and snapshot
        let current_path = util::paths::expand_tilde(&file_path);
        if current_path.exists() {
            let current = std::fs::read_to_string(&current_path)?;
            if current != content {
                let diff = util::diff::unified_diff(&current, &content, &file_path);
                if !diff.is_empty() {
                    println!();
                    println!("{}", diff);
                }
            } else {
                println!("  File is already at this snapshot state.");
            }
        } else {
            println!("  File does not exist — would be created.");
        }

        return Ok(());
    }

    let backup_dir = config_dir.join("backups");
    let restored_path = engine.rollback(snapshot_id, &backup_dir)?;

    println!(
        "{} Rolled back {} to snapshot #{}",
        "OK".green().bold(),
        restored_path.bold(),
        snapshot_id
    );

    if verbose {
        println!("  Backup saved to {}", backup_dir.display());
    }

    Ok(())
}
