use anyhow::Result;
use colored::Colorize;

use crate::core::manifest::Manifest;
use crate::util;

pub fn run(verbose: bool) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;

    if manifest.tools.is_empty() {
        println!("No tools tracked.");
        println!("  Run {} to start.", "dotsmith add <tool>".bold());
        return Ok(());
    }

    let mut warnings: Vec<String> = Vec::new();

    for (name, entry) in &manifest.tools {
        let mut existing_count = 0;
        let mut total_count = 0;

        for path_str in &entry.config_paths {
            total_count += 1;
            let path = util::paths::expand_tilde(path_str);

            if path.exists() {
                existing_count += 1;
            } else if util::fs::is_symlink(&path) {
                // Broken symlink
                warnings.push(format!("{}: broken symlink {}", name, path_str));
            } else {
                warnings.push(format!("{}: missing {}", name, path_str));
            }
        }

        let tier_label = format!("Tier {}", entry.tier);
        let status_icon = if existing_count == total_count {
            "OK".green().bold().to_string()
        } else {
            "!!".yellow().bold().to_string()
        };

        let pm_info = match &entry.plugin_manager {
            Some(pm) => format!("  plugins: {}", pm),
            None => String::new(),
        };

        println!(
            "  {} {:<12} {}  {}/{} paths{}",
            status_icon,
            name,
            tier_label.dimmed(),
            existing_count,
            total_count,
            pm_info.dimmed()
        );

        if verbose {
            for path_str in &entry.config_paths {
                let path = util::paths::expand_tilde(path_str);
                let indicator = if path.exists() {
                    "OK".green().to_string()
                } else {
                    "!!".red().to_string()
                };
                println!("      {} {}", indicator, path_str);
            }
        }
    }

    if !warnings.is_empty() {
        println!();
        println!("{}", "Warnings:".yellow().bold());
        for w in &warnings {
            println!("  {} {}", "!!".yellow(), w);
        }
    }

    Ok(())
}
