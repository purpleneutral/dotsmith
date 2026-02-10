use anyhow::Result;
use chrono::Utc;
use colored::Colorize;

use crate::core::detect;
use crate::core::manifest::Manifest;
use crate::core::module::ModuleRegistry;
use crate::core::snapshot::SnapshotEngine;
use crate::core::validate;
use crate::util;

struct CheckResult {
    ok: usize,
    warn: usize,
    error: usize,
    hints: Vec<String>,
}

impl CheckResult {
    fn new() -> Self {
        Self {
            ok: 0,
            warn: 0,
            error: 0,
            hints: Vec::new(),
        }
    }
}

pub fn run(verbose: bool, tool: Option<&str>) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let mut result = CheckResult::new();

    // --- Dotsmith setup checks ---
    println!("  Checking dotsmith setup...");

    // Config directory
    if config_dir.exists() {
        println!("    {}  config directory", "OK".green().bold());
        result.ok += 1;
    } else {
        println!(
            "    {}  config directory missing",
            "ERR".red().bold()
        );
        result.error += 1;
        result
            .hints
            .push("run `dotsmith init` to create config directory".to_string());
        // Can't continue without config dir
        print_summary(&result);
        return Ok(());
    }

    // Manifest
    let manifest = match Manifest::load(&config_dir) {
        Ok(m) => {
            println!(
                "    {}  manifest ({} tools tracked)",
                "OK".green().bold(),
                m.tools.len()
            );
            result.ok += 1;
            m
        }
        Err(_) => {
            println!(
                "    {}  manifest missing or invalid",
                "ERR".red().bold()
            );
            result.error += 1;
            result
                .hints
                .push("run `dotsmith init` to initialize".to_string());
            print_summary(&result);
            return Ok(());
        }
    };

    // Snapshot database
    match SnapshotEngine::open(&config_dir) {
        Ok(_) => {
            println!("    {}  snapshot database", "OK".green().bold());
            result.ok += 1;
        }
        Err(e) => {
            println!(
                "    {}  snapshot database: {}",
                "ERR".red().bold(),
                e
            );
            result.error += 1;
        }
    }

    if manifest.tools.is_empty() {
        println!();
        println!("  No tools tracked.");
        result
            .hints
            .push("run `dotsmith add <tool>` to start tracking".to_string());
        print_summary(&result);
        return Ok(());
    }

    // --- Per-tool checks ---
    println!();
    println!("  Checking tools...");

    let tools_to_check: Vec<(&String, &crate::core::manifest::ToolEntry)> = match tool {
        Some(name) => {
            manifest
                .tools
                .get_key_value(name)
                .map(|(k, v)| vec![(k, v)])
                .ok_or_else(|| anyhow::anyhow!("'{}' is not tracked by dotsmith", name))?
        }
        None => manifest.tools.iter().collect(),
    };

    for (name, entry) in &tools_to_check {
        check_tool(name, entry, verbose, &mut result);
    }

    println!();
    print_summary(&result);

    Ok(())
}

fn check_tool(
    name: &str,
    entry: &crate::core::manifest::ToolEntry,
    verbose: bool,
    result: &mut CheckResult,
) {
    let mut issues: Vec<String> = Vec::new();

    // Check if installed
    let installed = if let Some(module) = ModuleRegistry::get_builtin(name) {
        detect::check_installed(name, &module.metadata.detect_command).is_ok()
    } else {
        // Tier 2: try `which <tool_name>`
        detect::check_installed(name, &format!("which {}", name)).is_ok()
    };

    if !installed {
        issues.push("not installed".to_string());
    }

    // Check config paths
    let mut existing = 0;
    let total = entry.config_paths.len();
    let mut broken_links = Vec::new();
    let mut missing_paths = Vec::new();

    for path_str in &entry.config_paths {
        let path = util::paths::expand_tilde(path_str);
        if path.exists() {
            existing += 1;
        } else if util::fs::is_symlink(&path) {
            broken_links.push(path_str.clone());
        } else {
            missing_paths.push(path_str.clone());
        }
    }

    if !broken_links.is_empty() {
        issues.push(format!("{} broken symlink(s)", broken_links.len()));
    }
    if !missing_paths.is_empty() && existing == 0 {
        issues.push("no config found".to_string());
    } else if !missing_paths.is_empty() {
        issues.push(format!("{} missing path(s)", missing_paths.len()));
    }

    // Config syntax validation (Tier 1 only)
    if let Some(module) = ModuleRegistry::get_builtin(name) {
        for path_str in &entry.config_paths {
            let path = util::paths::expand_tilde(path_str);
            if path.is_file() {
                if let Ok(vr) = validate::validate_config(&path, &module.metadata.config_format) {
                    if !vr.valid {
                        issues.push(format!("syntax issues in {}", path_str));
                        if verbose {
                            for err in &vr.errors {
                                println!("          {} {}", "SYNTAX".yellow(), err);
                            }
                        }
                    }
                }
            }
        }
    }

    // Check snapshot freshness
    let snapshot_info = match entry.last_snapshot {
        Some(ts) => {
            let age = Utc::now().signed_duration_since(ts);
            if age.num_days() > 7 {
                issues.push(format!("snapshot {}d old", age.num_days()));
                format!("snapshot {}d ago", age.num_days())
            } else if age.num_hours() > 0 {
                format!("snapshot {}h ago", age.num_hours())
            } else {
                format!("snapshot {}m ago", age.num_minutes().max(1))
            }
        }
        None => {
            issues.push("never snapshotted".to_string());
            "never snapshotted".to_string()
        }
    };

    // Determine status
    let is_ok = issues.is_empty();
    let has_error = !installed || (existing == 0 && total > 0);

    if has_error {
        result.error += 1;
    } else if !is_ok {
        result.warn += 1;
    } else {
        result.ok += 1;
    }

    let icon = if has_error {
        "ERR".red().bold().to_string()
    } else if !is_ok {
        "!!".yellow().bold().to_string()
    } else {
        "OK".green().bold().to_string()
    };

    // Build status line
    let install_status = if installed { "installed" } else { "not installed" };
    let detail = if installed {
        format!(
            "{}, {}/{} paths, {}",
            install_status, existing, total, snapshot_info
        )
    } else {
        install_status.to_string()
    };

    println!("    {} {:<12} {}", icon, name, detail.dimmed());

    if verbose {
        for path_str in &entry.config_paths {
            let path = util::paths::expand_tilde(path_str);
            let indicator = if path.exists() {
                "OK".green().to_string()
            } else if util::fs::is_symlink(&path) {
                "BROKEN".red().to_string()
            } else {
                "MISSING".yellow().to_string()
            };
            println!("          {} {}", indicator, path_str);
        }
    }
}

fn print_summary(result: &CheckResult) {
    println!(
        "  Summary: {} healthy, {} warnings, {} errors",
        result.ok.to_string().green(),
        result.warn.to_string().yellow(),
        result.error.to_string().red()
    );

    for hint in &result.hints {
        println!("    {}: {}", "hint".cyan(), hint);
    }

    if (result.warn > 0 || result.error > 0) && result.hints.is_empty() {
        println!(
            "    {}: run `dotsmith snapshot` to create initial snapshots",
            "hint".cyan()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_new() {
        let r = CheckResult::new();
        assert_eq!(r.ok, 0);
        assert_eq!(r.warn, 0);
        assert_eq!(r.error, 0);
        assert!(r.hints.is_empty());
    }

    #[test]
    fn test_check_result_counts() {
        let mut r = CheckResult::new();
        r.ok = 5;
        r.warn = 2;
        r.error = 1;
        r.hints.push("test hint".to_string());
        assert_eq!(r.ok + r.warn + r.error, 8);
        assert_eq!(r.hints.len(), 1);
    }
}
