use anyhow::Result;
use colored::Colorize;

use crate::cli::PluginAction;
use crate::core::manifest::Manifest;
use crate::core::plugin;
use crate::util;

pub fn run(verbose: bool, tool: &str, action: &PluginAction) -> Result<()> {
    let config_dir = util::paths::config_dir()?;

    match action {
        PluginAction::Add { repo } => run_add(verbose, &config_dir, tool, repo),
        PluginAction::Remove { name } => run_remove(&config_dir, tool, name),
        PluginAction::List => run_list(&config_dir, tool),
        PluginAction::Update { name } => run_update(verbose, &config_dir, tool, name.as_deref()),
    }
}

fn run_add(
    verbose: bool,
    config_dir: &std::path::Path,
    tool: &str,
    repo: &str,
) -> Result<()> {
    let mut manifest = Manifest::load(config_dir)?;

    if verbose {
        println!("Cloning {} for {}...", repo.bold(), tool.bold());
    }

    let (name, init_file) = plugin::add_plugin(config_dir, &mut manifest, tool, repo)?;

    println!(
        "{} Added plugin {} for {}",
        "OK".green().bold(),
        name.bold(),
        tool.bold()
    );
    println!("  Init file: {}", init_file);

    let loader = plugin::loader_path(config_dir, tool);
    let loader_contracted = util::paths::contract_tilde(&loader);

    let source_line = match tool {
        "tmux" => format!("source-file {}", loader_contracted),
        _ => format!("source {}", loader_contracted),
    };

    let rc_file = match tool {
        "tmux" => "tmux.conf",
        "zsh" => ".zshrc",
        _ => "config",
    };

    println!(
        "\n  {} Add this line to your {} (if not already present):",
        "Hint:".yellow().bold(),
        rc_file
    );
    println!("    {}", source_line.dimmed());

    Ok(())
}

fn run_remove(config_dir: &std::path::Path, tool: &str, name: &str) -> Result<()> {
    let mut manifest = Manifest::load(config_dir)?;

    plugin::remove_plugin(config_dir, &mut manifest, tool, name)?;

    println!(
        "{} Removed plugin {} from {}",
        "OK".green().bold(),
        name.bold(),
        tool.bold()
    );

    Ok(())
}

fn run_list(config_dir: &std::path::Path, tool: &str) -> Result<()> {
    let manifest = Manifest::load(config_dir)?;

    let plugins = plugin::list_plugins(&manifest, tool)?;

    if plugins.is_empty() {
        println!("No plugins installed for {}.", tool.bold());
        println!(
            "  Run {} to add one.",
            format!("dotsmith plugins {} add <repo>", tool).bold()
        );
        return Ok(());
    }

    println!(
        "{} plugin(s) installed for {}:\n",
        plugins.len(),
        tool.bold()
    );

    println!(
        "  {:<30} {:<40} {}",
        "Name".bold(),
        "Repository".bold(),
        "Init File".bold()
    );
    println!("  {}", "-".repeat(80));

    for (name, repo, init) in &plugins {
        println!("  {:<30} {:<40} {}", name, repo.dimmed(), init.dimmed());
    }

    Ok(())
}

fn run_update(
    verbose: bool,
    config_dir: &std::path::Path,
    tool: &str,
    name: Option<&str>,
) -> Result<()> {
    let manifest = Manifest::load(config_dir)?;

    if verbose {
        match name {
            Some(n) => println!("Updating {} for {}...", n.bold(), tool.bold()),
            None => println!("Updating all plugins for {}...", tool.bold()),
        }
    }

    let results = plugin::update_plugins(config_dir, &manifest, tool, name)?;

    let updated_count = results.iter().filter(|r| r.updated).count();
    let up_to_date = results.len() - updated_count;

    for result in &results {
        if result.updated {
            let old_short = &result.old_commit[..7.min(result.old_commit.len())];
            let new_short = &result.new_commit[..7.min(result.new_commit.len())];
            println!(
                "  {} {} ({}..{})",
                "updated".green(),
                result.name.bold(),
                old_short,
                new_short,
            );
        } else if verbose {
            println!("  {} {}", "up to date".dimmed(), result.name);
        }
    }

    println!(
        "\n{} Updated {} plugin(s), {} already up to date",
        "OK".green().bold(),
        updated_count,
        up_to_date
    );

    Ok(())
}
