use anyhow::Result;
use colored::Colorize;

use crate::core::manifest::Manifest;
use crate::core::reload;
use crate::util;

/// Reload configuration for a tool.
pub fn run(verbose: bool, tool: &str) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;

    if !manifest.has_tool(tool) {
        anyhow::bail!("'{}' is not tracked by dotsmith", tool);
    }

    let entry = manifest.tools.get(tool).unwrap();

    // Use the first config path as the reload target
    let config_path = entry.config_paths.first().map(|s| s.as_str());

    if verbose {
        println!("Reloading {} configuration...", tool.bold());
    }

    let description = reload::reload_tool(tool, config_path)?;

    println!(
        "{} {}: {}",
        "OK".green().bold(),
        tool.bold(),
        description
    );

    Ok(())
}
