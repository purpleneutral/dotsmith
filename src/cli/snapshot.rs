use anyhow::Result;
use colored::Colorize;

use crate::core::manifest::Manifest;
use crate::core::snapshot::SnapshotEngine;
use crate::util;

/// Take a snapshot of config files for a specific tool or all tools.
pub fn run(verbose: bool, tool: Option<&str>, message: Option<&str>) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;
    let engine = SnapshotEngine::open(&config_dir)?;

    match tool {
        Some(name) => {
            let entry = manifest
                .tools
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("'{}' is not tracked by dotsmith", name))?;

            let count = engine.snapshot_tool(name, &entry.config_paths, message)?;

            if count > 0 {
                println!(
                    "{} Snapshotted {} file(s) for {}",
                    "OK".green().bold(),
                    count,
                    name.bold()
                );
            } else {
                println!("No changes to snapshot for {}", name.bold());
            }
        }
        None => {
            let count = engine.snapshot_all(&manifest, message)?;

            if count > 0 {
                println!(
                    "{} Snapshotted {} file(s) across all tools",
                    "OK".green().bold(),
                    count
                );
            } else {
                println!("No changes to snapshot across any tracked tools");
            }
        }
    }

    if verbose {
        println!("  Snapshots stored in {}", config_dir.join("snapshots.db").display());
    }

    Ok(())
}
