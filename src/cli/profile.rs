use anyhow::Result;
use colored::Colorize;

use crate::cli::ProfileAction;
use crate::core::manifest::Manifest;
use crate::core::profile;
use crate::util;

pub fn run(verbose: bool, action: &ProfileAction) -> Result<()> {
    let config_dir = util::paths::config_dir()?;

    match action {
        ProfileAction::Save { name } => run_save(verbose, &config_dir, name),
        ProfileAction::Load {
            name,
            add_untracked,
            dry_run,
        } => run_load(verbose, &config_dir, name, *add_untracked, *dry_run),
        ProfileAction::List => run_list(&config_dir),
        ProfileAction::Delete { name } => run_delete(&config_dir, name),
    }
}

fn run_save(verbose: bool, config_dir: &std::path::Path, name: &str) -> Result<()> {
    let manifest = Manifest::load(config_dir)?;

    if manifest.tools.is_empty() {
        println!("No tools tracked. Add tools with {} first.", "dotsmith add <tool>".bold());
        return Ok(());
    }

    let (tool_count, file_count) = profile::save_profile(config_dir, &manifest, name)?;

    println!(
        "{} Saved profile '{}' ({} tool(s), {} file(s))",
        "OK".green().bold(),
        name.bold(),
        tool_count,
        file_count,
    );

    if verbose {
        for tool_name in manifest.tools.keys() {
            println!("  {}", tool_name);
        }
    }

    Ok(())
}

fn run_load(
    verbose: bool,
    config_dir: &std::path::Path,
    name: &str,
    add_untracked: bool,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        return run_load_dry_run(verbose, config_dir, name, add_untracked);
    }

    let mut manifest = Manifest::load(config_dir)?;
    let result = profile::load_profile(config_dir, &mut manifest, name, add_untracked)?;

    println!(
        "{} Loaded profile '{}' ({} file(s) restored)",
        "OK".green().bold(),
        name.bold(),
        result.restored_files,
    );

    if result.backed_up_files > 0 {
        println!(
            "  {} file(s) backed up before overwrite",
            result.backed_up_files,
        );
    }

    if !result.tools_added.is_empty() {
        println!(
            "  {} tool(s) added: {}",
            result.tools_added.len(),
            result.tools_added.join(", "),
        );
    }

    if !result.skipped_tools.is_empty() {
        println!(
            "  {} tool(s) skipped (not tracked, use {}): {}",
            result.skipped_tools.len(),
            "--add-untracked".bold(),
            result.skipped_tools.join(", "),
        );
    }

    if verbose && result.restored_files > 0 {
        println!(
            "  Run {} to review changes.",
            "dotsmith diff".cyan(),
        );
    }

    Ok(())
}

fn run_load_dry_run(
    _verbose: bool,
    config_dir: &std::path::Path,
    name: &str,
    add_untracked: bool,
) -> Result<()> {
    let manifest = Manifest::load(config_dir)?;

    // Load profile metadata without applying
    let profile_dir = profile::profiles_dir(config_dir).join(name);
    if !profile_dir.exists() {
        return Err(
            crate::core::errors::DotsmithError::ProfileNotFound(name.to_string()).into(),
        );
    }

    let meta_content = std::fs::read_to_string(profile_dir.join("profile.toml"))?;
    let meta: profile::ProfileMeta = toml::from_str(&meta_content)?;

    println!("Profile '{}' contains:\n", name.bold());

    for (tool_name, tool_entry) in &meta.tools {
        let tracked = manifest.has_tool(tool_name);
        let status = if tracked {
            "tracked".green()
        } else if add_untracked {
            "will add".yellow()
        } else {
            "skip".dimmed()
        };

        println!("  [{}] {}", status, tool_name.bold());
        for path in &tool_entry.config_paths {
            println!("    {}", path);
        }
    }

    println!();
    println!(
        "{} No changes made (dry run)",
        "[dry-run]".yellow().bold(),
    );

    Ok(())
}

fn run_list(config_dir: &std::path::Path) -> Result<()> {
    let profiles = profile::list_profiles(config_dir)?;

    if profiles.is_empty() {
        println!(
            "No profiles saved yet. Run {} to create one.",
            "dotsmith profile save <name>".bold(),
        );
        return Ok(());
    }

    println!("{:<20} {:<24} {:>6} {:>6}", "Name", "Created", "Tools", "Files");
    println!("{}", "-".repeat(60));

    for p in &profiles {
        println!(
            "{:<20} {:<24} {:>6} {:>6}",
            p.name.bold(),
            p.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            p.tool_count,
            p.file_count,
        );
    }

    Ok(())
}

fn run_delete(config_dir: &std::path::Path, name: &str) -> Result<()> {
    profile::delete_profile(config_dir, name)?;

    println!(
        "{} Deleted profile '{}'",
        "OK".green().bold(),
        name.bold(),
    );

    Ok(())
}
