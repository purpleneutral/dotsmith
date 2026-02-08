use anyhow::Result;
use colored::Colorize;

use crate::core::config::DotsmithConfig;
use crate::core::manifest::Manifest;
use crate::core::repo;
use crate::util;

pub fn run_init(verbose: bool, path: &str) -> Result<()> {
    let expanded = util::paths::expand_tilde(path);
    let repo_path = std::path::Path::new(&expanded);

    repo::init_repo(repo_path)?;

    // Save repo_path to config
    let config_dir = util::paths::config_dir()?;
    let mut config = DotsmithConfig::load(&config_dir);
    config.general.repo_path = Some(path.to_string());
    config.save(&config_dir)?;

    if verbose {
        println!("Initialized dotfile repo at {}", repo_path.display());
    }
    println!(
        "{} Repo initialized at {}",
        "done:".green().bold(),
        path
    );

    Ok(())
}

pub fn run_sync(verbose: bool) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let config = DotsmithConfig::load(&config_dir);

    let repo_path_str = config
        .general
        .repo_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No repo path configured. Run `dotsmith repo init <path>` first."))?;

    let expanded = util::paths::expand_tilde(repo_path_str);
    let repo_path = std::path::Path::new(&expanded);
    let manifest = Manifest::load(&config_dir)?;

    let result = repo::sync_repo(repo_path, &manifest)?;

    if verbose {
        println!(
            "Synced {} file(s), committed: {}",
            result.files_copied, result.committed
        );
    }

    if result.committed {
        println!(
            "{} Synced {} file(s) and committed",
            "done:".green().bold(),
            result.files_copied
        );
    } else {
        println!("{} No changes to commit", "done:".green().bold());
    }

    Ok(())
}

pub fn run_status(verbose: bool) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let config = DotsmithConfig::load(&config_dir);

    let repo_path_str = config
        .general
        .repo_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("No repo path configured. Run `dotsmith repo init <path>` first."))?;

    let expanded = util::paths::expand_tilde(repo_path_str);
    let repo_path = std::path::Path::new(&expanded);
    let status = repo::repo_status(repo_path)?;

    if !status.initialized {
        println!("Repo not initialized at {}", repo_path_str);
        return Ok(());
    }

    if verbose {
        println!("Repo at: {}", repo_path_str);
    }

    if status.changed_files > 0 {
        println!(
            "{} {} uncommitted change(s) in repo",
            "repo:".yellow().bold(),
            status.changed_files
        );
    } else {
        println!("{} Repo is clean", "repo:".green().bold());
    }

    Ok(())
}
