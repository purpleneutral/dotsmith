use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// A test environment with isolated directories.
/// All dotsmith operations use this temp dir instead of real `~/.config/`.
pub struct TestEnv {
    /// Root temporary directory (kept alive for duration of test)
    pub root: TempDir,
    /// Dotsmith config dir (like `~/.config/dotsmith/`)
    pub config_dir: PathBuf,
    /// Fake home dir (simulates user's `~` with fake configs)
    pub home_dir: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self {
        let root = TempDir::new().unwrap();
        let config_dir = root.path().join("dotsmith");
        let home_dir = root.path().join("home");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&home_dir).unwrap();

        Self {
            root,
            config_dir,
            home_dir,
        }
    }

    /// Get the config dir path as a string, for use in env vars.
    pub fn config_dir_str(&self) -> String {
        self.config_dir.display().to_string()
    }

    /// Create a fake tmux config setup with symlinks,
    /// mimicking the user's actual layout:
    /// `~/.config/tmux` -> dots/tmux (symlink)
    pub fn setup_tmux_with_symlink(&self) -> PathBuf {
        // Create the "dotfiles repo" directory (source of truth)
        let dots_dir = self.home_dir.join("dots/tmux");
        fs::create_dir_all(dots_dir.join("conf")).unwrap();
        fs::create_dir_all(dots_dir.join("plugs/tpm")).unwrap();

        // Write config files
        fs::write(
            dots_dir.join("tmux.conf"),
            "set -g mouse on\nset -g prefix C-a\n",
        )
        .unwrap();
        fs::write(
            dots_dir.join("conf/settings.conf"),
            "set -g base-index 1\nset -g escape-time 0\n",
        )
        .unwrap();
        fs::write(
            dots_dir.join("conf/keys.conf"),
            "bind r source-file ~/.config/tmux/tmux.conf\n",
        )
        .unwrap();

        // Create the symlink: ~/.config/tmux -> dots/tmux
        let config_tmux = self.home_dir.join(".config/tmux");
        fs::create_dir_all(self.home_dir.join(".config")).unwrap();
        std::os::unix::fs::symlink(&dots_dir, &config_tmux).unwrap();

        config_tmux
    }

    /// Create a plain (non-symlinked) config directory for a tool.
    pub fn setup_plain_config(&self, tool: &str) -> PathBuf {
        let config_dir = self.home_dir.join(format!(".config/{}", tool));
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.toml"), "[general]\nkey = 'value'\n").unwrap();
        config_dir
    }
}
