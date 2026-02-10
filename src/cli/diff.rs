use anyhow::Result;
use colored::Colorize;

use crate::core::manifest::Manifest;
use crate::core::snapshot::SnapshotEngine;
use crate::util;

/// Show differences between current config files and last snapshot.
pub fn run(_verbose: bool, tool: Option<&str>) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;
    let engine = SnapshotEngine::open(&config_dir)?;

    let tools_to_diff: Vec<(&String, &crate::core::manifest::ToolEntry)> = match tool {
        Some(name) => {
            manifest
                .tools
                .get_key_value(name)
                .map(|(k, v)| vec![(k, v)])
                .ok_or_else(|| anyhow::anyhow!("'{}' is not tracked by dotsmith", name))?
        }
        None => manifest.tools.iter().collect(),
    };

    let mut any_diffs = false;

    for (name, entry) in &tools_to_diff {
        let diffs = engine.diff_current(name, &entry.config_paths)?;

        if diffs.is_empty() {
            continue;
        }

        any_diffs = true;

        for file_diff in &diffs {
            let output = util::diff::unified_diff(
                &file_diff.old_content,
                &file_diff.new_content,
                &file_diff.file_path,
            );

            if !output.is_empty() {
                println!("{}", output);
            }
        }
    }

    if !any_diffs {
        let scope = tool.unwrap_or("any tracked tool");
        println!("No changes detected for {}", scope.bold());
        println!(
            "  Run {} first to establish a baseline.",
            "dotsmith snapshot".bold()
        );
    }

    Ok(())
}
