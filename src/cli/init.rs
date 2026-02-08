use std::fs;
use std::os::unix::fs::PermissionsExt;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::core::config::DotsmithConfig;
use crate::core::errors::DotsmithError;
use crate::core::manifest::Manifest;
use crate::util;

pub fn run(verbose: bool) -> Result<()> {
    let config_dir = util::paths::config_dir()?;

    // Check if already initialized
    if config_dir.join("manifest.toml").exists() {
        return Err(DotsmithError::AlreadyInitialized(config_dir.display().to_string()).into());
    }

    // Create config directory with restricted permissions
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("failed to create {}", config_dir.display()))?;
    fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("failed to set permissions on {}", config_dir.display()))?;

    // Write default config.toml atomically
    let config = DotsmithConfig::default();
    config.save(&config_dir)?;

    // Write empty manifest.toml
    let manifest = Manifest::default();
    manifest.save(&config_dir)?;

    if verbose {
        println!("Created {}", config_dir.display());
        println!("  config.toml  (dotsmith settings)");
        println!("  manifest.toml (tracked tools)");
    }

    println!(
        "{} Initialized dotsmith at {}",
        "OK".green().bold(),
        config_dir.display()
    );
    println!(
        "  Run {} to start tracking a tool.",
        "dotsmith add <tool>".bold()
    );

    Ok(())
}
