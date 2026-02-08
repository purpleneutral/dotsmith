use anyhow::Result;
use chrono::Utc;
use colored::Colorize;

use crate::core::detect;
use crate::core::errors::DotsmithError;
use crate::core::manifest::{Manifest, ToolEntry};
use crate::core::module::ModuleRegistry;
use crate::util;

pub fn run(verbose: bool, tool: &str) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let mut manifest = Manifest::load(&config_dir)?;

    // Pre-check: already tracked?
    if manifest.has_tool(tool) {
        return Err(DotsmithError::ToolAlreadyTracked(tool.to_string()).into());
    }

    // Determine tier and load module definition
    let (tier, module_def) = match ModuleRegistry::get_builtin(tool) {
        Some(def) => (1u8, Some(def)),
        None => (2u8, None),
    };

    // Check if the tool is installed
    let detect_cmd = module_def
        .as_ref()
        .map(|d| d.metadata.detect_command.as_str())
        .unwrap_or(&format!("which {}", tool))
        .to_string();
    detect::check_installed(tool, &detect_cmd)?;

    // Find config files
    let config_paths = if let Some(ref def) = module_def {
        detect::find_config_paths_from_module(def)?
    } else {
        detect::auto_detect_config_paths(tool)?
    };

    if config_paths.is_empty() {
        return Err(DotsmithError::NoConfigFound(tool.to_string()).into());
    }

    // Detect existing plugin manager
    let plugin_manager = detect::detect_plugin_manager(tool, &config_paths);

    // Build manifest entry
    let entry = ToolEntry {
        tier,
        config_paths: config_paths
            .iter()
            .map(|p| util::paths::contract_tilde(p))
            .collect(),
        plugins_managed: false, // Never auto-enable — respect existing setup
        plugin_manager: plugin_manager.clone(),
        added_at: Utc::now(),
        last_snapshot: None,
        plugins: std::collections::BTreeMap::new(),
    };

    manifest.add_tool(tool, entry)?;
    manifest.save(&config_dir)?;

    // Report results
    let tier_label = match tier {
        1 => "Tier 1 — full support",
        2 => "Tier 2 — basic tracking",
        3 => "Tier 3 — user-enriched",
        _ => "unknown tier",
    };

    println!(
        "{} Added {} ({})",
        "OK".green().bold(),
        tool.bold(),
        tier_label
    );

    println!("  Tracking {} config path(s):", config_paths.len());
    for path in &config_paths {
        let contracted = util::paths::contract_tilde(path);
        let suffix = if util::fs::is_symlink(path) {
            if let Some(target) = util::fs::symlink_target(path) {
                format!(" {} {}", "->".dimmed(), target.display())
            } else {
                String::new()
            }
        } else if path.is_dir() {
            format!(" {}", "(directory)".dimmed())
        } else {
            String::new()
        };

        if verbose {
            println!("    {}{}", contracted, suffix);
        } else {
            println!("    {}", contracted);
        }
    }

    // Option database info (Tier 1 only)
    if tier == 1
        && let Some(opts) = ModuleRegistry::get_options(tool)
    {
        println!(
            "  Option database: {} options available to explore",
            opts.options.len()
        );
    }

    // Plugin manager info
    if let Some(ref pm) = plugin_manager {
        println!("  Detected existing plugin manager: {}", pm.bold());
        println!(
            "    dotsmith will not manage plugins (use {} to opt in)",
            "dotsmith plugins".dimmed()
        );
    }

    Ok(())
}
