use anyhow::Result;
use colored::Colorize;

use crate::core::manifest::Manifest;
use crate::core::snapshot::SnapshotEngine;
use crate::util;

/// Show snapshot history for a tool.
pub fn run(_verbose: bool, tool: &str, limit: usize) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;

    if !manifest.has_tool(tool) {
        anyhow::bail!("'{}' is not tracked by dotsmith", tool);
    }

    let engine = SnapshotEngine::open(&config_dir)?;
    let history = engine.history(tool, limit)?;

    if history.is_empty() {
        println!("No snapshots found for {}", tool.bold());
        println!(
            "  Run {} to take one.",
            format!("dotsmith snapshot {}", tool).bold()
        );
        return Ok(());
    }

    println!("{} snapshot history for {}:", "Showing".bold(), tool.bold());
    println!();

    for entry in &history {
        let id = format!("#{}", entry.id).cyan();
        let hash_short = &entry.hash[..8];
        let msg = entry
            .message
            .as_deref()
            .unwrap_or("(no message)");

        println!(
            "  {}  {}  {}  {}",
            id,
            entry.created_at.dimmed(),
            hash_short.yellow(),
            entry.file_path
        );
        if msg != "(no message)" {
            println!("       {}", msg.dimmed());
        }
    }

    println!();
    println!(
        "  {} snapshots shown (use {} for more)",
        history.len(),
        "--limit N".bold()
    );

    Ok(())
}
