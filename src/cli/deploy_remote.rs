use anyhow::Result;
use colored::Colorize;

use crate::core::manifest::Manifest;
use crate::core::remote::{self, RemoteDeployOpts};
use crate::util;

pub fn run(
    verbose: bool,
    host: &str,
    user: Option<&str>,
    tools: Option<&[String]>,
    dry_run: bool,
) -> Result<()> {
    let config_dir = util::paths::config_dir()?;
    let manifest = Manifest::load(&config_dir)?;

    let tool_refs: Option<Vec<&str>> = tools.map(|t| t.iter().map(|s| s.as_str()).collect());

    let opts = RemoteDeployOpts {
        host,
        user,
        tools: tool_refs,
        dry_run,
    };

    let actions = remote::plan_remote_deploy(&manifest, &opts)?;

    if actions.is_empty() {
        println!("No files to deploy.");
        return Ok(());
    }

    let dest = match user {
        Some(u) => format!("{}@{}", u, host),
        None => host.to_string(),
    };

    println!("Deploy to {}:\n", dest.bold());

    for action in &actions {
        let status = if action.remote_exists {
            "overwrite".yellow()
        } else {
            "create".green()
        };

        let tool_prefix = format!("[{}]", action.tool).dimmed();
        println!(
            "  {} [{}] {} -> {}:{}",
            tool_prefix,
            status,
            util::paths::contract_tilde(&action.local_path),
            dest,
            action.remote_path,
        );
    }

    if dry_run {
        println!();
        println!(
            "{} No files copied (dry run)",
            "[dry-run]".yellow().bold(),
        );
        return Ok(());
    }

    let result = remote::execute_remote_deploy(&actions, host, user)?;

    println!();
    println!(
        "{} Deployed {} file(s) to {}",
        "OK".green().bold(),
        result.files_copied,
        dest,
    );

    if result.files_backed_up > 0 {
        println!(
            "  {} remote file(s) backed up before overwrite",
            result.files_backed_up,
        );
    }

    if verbose && result.files_skipped > 0 {
        println!(
            "  {} file(s) skipped (not found locally)",
            result.files_skipped,
        );
    }

    Ok(())
}
