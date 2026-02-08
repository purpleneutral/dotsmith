use std::process::Command;

use anyhow::{Context, Result};

use crate::core::module::ModuleRegistry;

/// Reload configuration for a tool.
/// Uses the module's reload_command if available, otherwise attempts common methods.
///
/// Returns a description of what was done, or an error if reload failed.
pub fn reload_tool(tool: &str, config_path: Option<&str>) -> Result<String> {
    // Try built-in module reload command first
    if let Some(module) = ModuleRegistry::get_builtin(tool)
        && let Some(ref reload_cmd) = module.metadata.reload_command
    {
        let description = module
            .metadata
            .reload_description
            .as_deref()
            .unwrap_or("reloading configuration");

        let cmd = if let Some(path) = config_path {
            reload_cmd.replace("{config_path}", path)
        } else {
            reload_cmd.clone()
        };

        execute_reload_command(&cmd)
            .with_context(|| format!("failed to reload {}", tool))?;

        return Ok(description.to_string());
    }

    // Fallback: try common reload methods
    match tool {
        "tmux" => reload_tmux(config_path),
        "kitty" => reload_kitty(),
        "sway" | "i3" => reload_wm(tool),
        _ => anyhow::bail!(
            "no reload method known for '{}'. You may need to restart it manually.",
            tool
        ),
    }
}

/// Execute a reload command string safely.
/// Splits on whitespace and uses Command::new with explicit args.
fn execute_reload_command(cmd: &str) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("empty reload command");
    }

    let status = Command::new(parts[0])
        .args(&parts[1..])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .with_context(|| format!("failed to execute: {}", cmd))?;

    if !status.success() {
        anyhow::bail!("reload command failed with exit code: {}", status);
    }

    Ok(())
}

/// Reload tmux by sourcing the config file.
fn reload_tmux(config_path: Option<&str>) -> Result<String> {
    let path = config_path.unwrap_or("~/.config/tmux/tmux.conf");
    let expanded = crate::util::paths::expand_tilde(path);

    let status = Command::new("tmux")
        .arg("source-file")
        .arg(&expanded)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .context("failed to run tmux source-file")?;

    if !status.success() {
        anyhow::bail!("tmux source-file failed — is tmux running?");
    }

    Ok("sourced tmux config".to_string())
}

/// Reload kitty by sending SIGUSR1.
fn reload_kitty() -> Result<String> {
    let status = Command::new("pkill")
        .args(["-USR1", "kitty"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("failed to send SIGUSR1 to kitty")?;

    if !status.success() {
        anyhow::bail!("failed to signal kitty — is it running?");
    }

    Ok("sent SIGUSR1 to kitty".to_string())
}

/// Reload a window manager (i3/sway).
fn reload_wm(tool: &str) -> Result<String> {
    let status = Command::new(tool)
        .arg("reload")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .with_context(|| format!("failed to reload {}", tool))?;

    if !status.success() {
        anyhow::bail!("{} reload failed", tool);
    }

    Ok(format!("reloaded {} config", tool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_tool_reload() {
        let result = reload_tool("unknown_tool_xyz", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no reload method"));
    }
}
