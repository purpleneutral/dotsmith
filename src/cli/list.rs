use anyhow::Result;
use colored::Colorize;

use crate::core::manifest::Manifest;
use crate::util;

pub fn run(_verbose: bool) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;

    if manifest.tools.is_empty() {
        println!("No tools tracked yet.");
        println!("  Run {} to get started.", "dotsmith add <tool>".bold());
        return Ok(());
    }

    println!(
        "{:<14} {:<22} {:<8} {}",
        "Tool".bold(),
        "Tier".bold(),
        "Paths".bold(),
        "Plugin Manager".bold()
    );
    println!("{}", "-".repeat(58));

    for (name, entry) in &manifest.tools {
        let tier_label = match entry.tier {
            1 => "Tier 1 (full)".green().to_string(),
            2 => "Tier 2 (basic)".yellow().to_string(),
            3 => "Tier 3 (enriched)".blue().to_string(),
            _ => format!("Tier {}", entry.tier),
        };

        let pm = entry.plugin_manager.as_deref().unwrap_or("-");

        println!(
            "{:<14} {:<22} {:<8} {}",
            name,
            tier_label,
            entry.config_paths.len(),
            pm
        );
    }

    Ok(())
}
