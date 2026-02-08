use std::path::{Path, PathBuf};

/// Get the dotsmith config directory.
/// Priority: DOTSMITH_CONFIG_DIR env var > ~/.config/dotsmith/
pub fn config_dir() -> anyhow::Result<PathBuf> {
    if let Ok(dir) = std::env::var("DOTSMITH_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine config directory"))?;
    Ok(base.join("dotsmith"))
}

/// Expand `~` to the user's home directory.
/// Only handles `~/path` — not `~user/path`.
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    if path == "~"
        && let Some(home) = dirs::home_dir()
    {
        return home;
    }
    PathBuf::from(path)
}

/// Contract an absolute path to use `~` for the home directory.
/// `/home/user/.config/tmux` → `~/.config/tmux`
pub fn contract_tilde(path: &Path) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(suffix) = path.strip_prefix(&home)
    {
        return format!("~/{}", suffix.display());
    }
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde_with_path() {
        let home = dirs::home_dir().expect("home dir");
        let result = expand_tilde("~/foo/bar");
        assert_eq!(result, home.join("foo/bar"));
    }

    #[test]
    fn test_expand_tilde_alone() {
        let home = dirs::home_dir().expect("home dir");
        let result = expand_tilde("~");
        assert_eq!(result, home);
    }

    #[test]
    fn test_expand_no_tilde() {
        let result = expand_tilde("/absolute/path");
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_contract_tilde() {
        let home = dirs::home_dir().expect("home dir");
        let path = home.join(".config/tmux");
        let result = contract_tilde(&path);
        assert_eq!(result, "~/.config/tmux");
    }

    #[test]
    fn test_contract_tilde_no_home_prefix() {
        let path = PathBuf::from("/etc/config");
        let result = contract_tilde(&path);
        assert_eq!(result, "/etc/config");
    }

    #[test]
    fn test_config_dir_env_override() {
        let original = std::env::var("DOTSMITH_CONFIG_DIR").ok();
        // SAFETY: This test runs in isolation and does not share the env var
        // with other tests that depend on it concurrently.
        unsafe {
            std::env::set_var("DOTSMITH_CONFIG_DIR", "/tmp/test-dotsmith");
        }
        let result = config_dir().unwrap();
        assert_eq!(result, PathBuf::from("/tmp/test-dotsmith"));
        unsafe {
            match original {
                Some(val) => std::env::set_var("DOTSMITH_CONFIG_DIR", val),
                None => std::env::remove_var("DOTSMITH_CONFIG_DIR"),
            }
        }
    }
}
