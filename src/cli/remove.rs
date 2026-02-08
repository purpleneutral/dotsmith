use anyhow::Result;
use colored::Colorize;

use crate::core::manifest::Manifest;
use crate::util;

pub fn run(_verbose: bool, tool: &str) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let mut manifest = Manifest::load(&config_dir)?;

    let removed = manifest.remove_tool(tool)?;
    manifest.save(&config_dir)?;

    println!(
        "{} Removed {} from dotsmith management",
        "OK".green().bold(),
        tool.bold()
    );
    println!("  Your config files are untouched:");
    for path in &removed.config_paths {
        println!("    {}", path);
    }

    Ok(())
}
