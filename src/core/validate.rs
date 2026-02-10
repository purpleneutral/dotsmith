use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

/// Result of validating a config file's syntax.
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    fn ok() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    fn with_errors(errors: Vec<String>) -> Self {
        Self {
            valid: errors.is_empty(),
            errors,
        }
    }
}

/// Validate a config file's syntax based on its format.
///
/// Supported formats: "toml", "key-value", "git", "tmux".
/// Formats "shell" and "lua" are skipped (too complex to parse).
pub fn validate_config(path: &Path, format: &str) -> Result<ValidationResult> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    match format {
        "toml" => validate_toml(&content),
        "key-value" => Ok(validate_key_value(&content)),
        "git" => Ok(validate_git_config(&content)),
        "tmux" => Ok(validate_tmux(&content)),
        // Shell and Lua are too complex to parse correctly
        "shell" | "lua" => Ok(ValidationResult::ok()),
        _ => Ok(ValidationResult::ok()),
    }
}

/// Validate TOML syntax using the toml crate.
fn validate_toml(content: &str) -> Result<ValidationResult> {
    match content.parse::<toml::Value>() {
        Ok(_) => Ok(ValidationResult::ok()),
        Err(e) => Ok(ValidationResult::with_errors(vec![e.to_string()])),
    }
}

/// Validate key-value format (kitty-style).
/// Non-blank, non-comment lines must have a whitespace or `=` separator.
fn validate_key_value(content: &str) -> ValidationResult {
    let mut errors = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Must contain at least one space, tab, or equals sign
        if !trimmed.contains(' ') && !trimmed.contains('\t') && !trimmed.contains('=') {
            errors.push(format!(
                "line {}: expected key-value pair, got '{}'",
                i + 1,
                truncate(trimmed, 40)
            ));
        }
    }

    ValidationResult::with_errors(errors)
}

/// Validate git config INI format.
/// Lines must be blank, comments, `[section]` headers, or `key = value`.
fn validate_git_config(content: &str) -> ValidationResult {
    let mut errors = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        // Section header
        if trimmed.starts_with('[') {
            if !trimmed.ends_with(']') {
                errors.push(format!(
                    "line {}: unclosed section header '{}'",
                    i + 1,
                    truncate(trimmed, 40)
                ));
            }
            continue;
        }

        // Key = value (or key with whitespace value)
        if !trimmed.contains('=') && !trimmed.contains(' ') && !trimmed.contains('\t') {
            errors.push(format!(
                "line {}: expected key = value, got '{}'",
                i + 1,
                truncate(trimmed, 40)
            ));
        }
    }

    ValidationResult::with_errors(errors)
}

/// Light validation for tmux config.
/// Non-blank, non-comment lines should start with a known tmux command word.
fn validate_tmux(content: &str) -> ValidationResult {
    let known_commands: &[&str] = &[
        "set", "bind", "unbind", "source", "run", "if", "set-option", "set-window-option",
        "bind-key", "unbind-key", "source-file", "display", "display-message", "new",
        "new-session", "new-window", "send", "send-keys", "select", "select-pane",
        "select-window", "split", "split-window", "swap", "move", "resize", "copy",
        "paste", "choose", "command", "confirm", "break", "join", "kill", "last", "link",
        "list", "load", "lock", "next", "pipe", "previous", "refresh", "rename", "respawn",
        "rotate", "save", "show", "switch", "wait", "has", "attach", "detach", "setw",
        "set-environment", "setenv", "is-prefix", "%if", "%endif", "%else", "%hidden",
    ];

    let mut errors = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Line continuation with backslash is valid
        if trimmed == "\\" {
            continue;
        }

        // Get first word
        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        // Strip leading - (for set -g, etc.)
        if !known_commands.contains(&first_word) {
            errors.push(format!(
                "line {}: unrecognized command '{}'",
                i + 1,
                truncate(first_word, 40)
            ));
        }
    }

    ValidationResult::with_errors(errors)
}

fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_valid_toml() {
        let result = validate_toml("[section]\nkey = \"value\"\n").unwrap();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_toml() {
        let result = validate_toml("[section\nkey = ").unwrap();
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_valid_key_value() {
        let content = "# comment\nfont_family JetBrains Mono\nfont_size 12\n";
        let result = validate_key_value(content);
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_key_value() {
        let content = "font_family JetBrains Mono\nbadline\n";
        let result = validate_key_value(content);
        assert!(!result.valid);
        assert!(result.errors[0].contains("line 2"));
    }

    #[test]
    fn test_valid_git_config() {
        let content = "[user]\n\tname = John\n\temail = john@example.com\n";
        let result = validate_git_config(content);
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_git_config() {
        let content = "[user\n\tname = John\n";
        let result = validate_git_config(content);
        assert!(!result.valid);
        assert!(result.errors[0].contains("unclosed section"));
    }

    #[test]
    fn test_valid_tmux() {
        let content = "# tmux config\nset -g mouse on\nbind r source-file ~/.tmux.conf\n";
        let result = validate_tmux(content);
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_tmux() {
        let content = "set -g mouse on\nfoobar something\n";
        let result = validate_tmux(content);
        assert!(!result.valid);
        assert!(result.errors[0].contains("foobar"));
    }

    #[test]
    fn test_shell_format_skipped() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.zsh");
        fs::write(&path, "this is not valid anything {{{").unwrap();
        let result = validate_config(&path, "shell").unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_lua_format_skipped() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.lua");
        fs::write(&path, "this is not valid lua {{{").unwrap();
        let result = validate_config(&path, "lua").unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_unknown_format_skipped() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.conf");
        fs::write(&path, "whatever").unwrap();
        let result = validate_config(&path, "unknown").unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_validate_toml_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.toml");
        fs::write(&path, "[window]\nopacity = 0.9\n").unwrap();
        let result = validate_config(&path, "toml").unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_validate_toml_file_invalid() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.toml");
        fs::write(&path, "[window\nopacity = ").unwrap();
        let result = validate_config(&path, "toml").unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn test_key_value_with_equals() {
        let content = "key=value\n";
        let result = validate_key_value(content);
        assert!(result.valid);
    }

    #[test]
    fn test_git_config_with_comments() {
        let content = "# comment\n; another comment\n[core]\n\tautocrlf = true\n";
        let result = validate_git_config(content);
        assert!(result.valid);
    }

    #[test]
    fn test_tmux_with_setw() {
        let content = "setw -g mode-keys vi\n";
        let result = validate_tmux(content);
        assert!(result.valid);
    }
}
