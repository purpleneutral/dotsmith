use std::path::Path;

/// Information extracted from a plugin's README and git metadata.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub config_excerpt: Option<String>,
}

/// Scan a plugin directory and extract info from README and git remote.
///
/// `plugin_dir` is the cloned plugin directory (e.g., `~/.config/dotsmith/plugins/zsh/zsh-autosuggestions/`).
/// `name` is the plugin name, `repo` is the repo specifier from the manifest.
pub fn scan_plugin(plugin_dir: &Path, name: &str, repo: &str) -> PluginInfo {
    let url = resolve_url(repo, plugin_dir);
    let (description, config_excerpt) = read_readme(plugin_dir);

    PluginInfo {
        name: name.to_string(),
        url,
        description,
        config_excerpt,
    }
}

/// Resolve a browsable URL from the repo specifier or git remote.
fn resolve_url(repo: &str, plugin_dir: &Path) -> String {
    // If it's a shorthand like "user/repo", make a GitHub URL
    if !repo.contains("://") && repo.contains('/') {
        return format!("https://github.com/{}", repo);
    }

    // If it's already a full URL, clean it up for browsing
    if repo.starts_with("https://") || repo.starts_with("http://") {
        return repo
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .to_string();
    }

    // Try reading the git remote as a fallback
    if let Some(remote) = git_remote_url(plugin_dir) {
        return clean_git_url(&remote);
    }

    repo.to_string()
}

/// Read the git remote origin URL from a cloned repo.
fn git_remote_url(repo_dir: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !url.is_empty() {
            return Some(url);
        }
    }
    None
}

/// Convert a git URL to a browsable HTTPS URL.
fn clean_git_url(url: &str) -> String {
    let cleaned = url
        .trim_end_matches('/')
        .trim_end_matches(".git");

    // Convert SSH URLs: git@github.com:user/repo -> https://github.com/user/repo
    if let Some(rest) = cleaned.strip_prefix("git@") {
        if let Some((host, path)) = rest.split_once(':') {
            return format!("https://{}/{}", host, path);
        }
    }

    cleaned.to_string()
}

/// Read README.md (or README, README.rst, README.txt) and extract info.
/// Returns (description, config_excerpt).
fn read_readme(plugin_dir: &Path) -> (Option<String>, Option<String>) {
    let readme_names = ["README.md", "readme.md", "README", "README.rst", "README.txt"];

    let readme_path = readme_names
        .iter()
        .map(|name| plugin_dir.join(name))
        .find(|p| p.is_file());

    let Some(path) = readme_path else {
        return (None, None);
    };

    let Ok(content) = std::fs::read_to_string(&path) else {
        return (None, None);
    };

    let description = extract_description(&content);
    let config_excerpt = extract_config_section(&content);

    (description, config_excerpt)
}

/// Check if a line is a setext heading underline (=== or ---).
fn is_setext_underline(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && (trimmed.chars().all(|c| c == '=') || trimmed.chars().all(|c| c == '-'))
}

/// Extract the first meaningful paragraph from a README as a description.
///
/// Skips the title line (# ... or setext-style), badges, and blank lines.
/// Returns up to 5 lines of description text.
fn extract_description(content: &str) -> Option<String> {
    let mut lines = content.lines().peekable();
    let mut description_lines = Vec::new();

    // Skip leading blank lines
    while lines.peek().is_some_and(|l| l.trim().is_empty()) {
        lines.next();
    }

    // Skip the title — ATX-style (# heading) or setext-style (text + === underline)
    if lines.peek().is_some_and(|l| l.starts_with('#')) {
        lines.next();
    } else {
        // Might be setext: consume the text line, then check for underline
        let first = lines.next();
        if first.is_some() && lines.peek().is_some_and(|l| is_setext_underline(l)) {
            lines.next(); // consume the === or --- underline
        } else if let Some(text) = first {
            // Not a heading at all — put it back by using it as description
            if !text.trim().is_empty() {
                description_lines.push(text.trim().to_string());
            }
        }
    }

    // Skip blank lines after title
    while lines.peek().is_some_and(|l| l.trim().is_empty()) {
        lines.next();
    }

    // Skip badge lines (contain ![, [![, or start with <)
    while lines.peek().is_some_and(|l| {
        let trimmed = l.trim();
        trimmed.contains("![") || trimmed.starts_with('<') || trimmed.starts_with("[!")
    }) {
        lines.next();
    }

    // Skip blank lines after badges
    while lines.peek().is_some_and(|l| l.trim().is_empty()) {
        lines.next();
    }

    // Collect the first paragraph (up to 5 lines, stop at blank line or heading)
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            break;
        }
        // Stop if next line is a setext underline (this line was actually a heading)
        if lines.peek().is_some_and(|l| is_setext_underline(l)) {
            break;
        }
        description_lines.push(trimmed.to_string());
        if description_lines.len() >= 5 {
            break;
        }
    }

    if description_lines.is_empty() {
        None
    } else {
        Some(description_lines.join(" "))
    }
}

/// Extract a "Configuration" or "Options" section from the README.
///
/// Looks for headings containing: config, option, setting, usage, install.
/// Handles both ATX-style (## Heading) and setext-style (Heading\n---) headings.
/// Returns up to 30 lines from the first matching section.
fn extract_config_section(content: &str) -> Option<String> {
    let config_keywords = ["config", "option", "setting", "usage", "variable", "customiz"];
    let lines = content.lines();
    let mut section_lines = Vec::new();
    let mut in_section = false;
    let mut section_level = 0; // 1 = h1, 2 = h2, etc.
    let mut prev_line: Option<&str> = None;

    for line in lines {
        if in_section {
            // Stop at next ATX heading of same or higher level
            if line.starts_with('#') {
                let level = line.chars().take_while(|c| *c == '#').count();
                if level <= section_level {
                    break;
                }
            }
            // Stop at next setext heading of same or higher level
            if is_setext_underline(line) && prev_line.is_some_and(|p| !p.trim().is_empty()) {
                let setext_level = if line.trim().starts_with('=') { 1 } else { 2 };
                if setext_level <= section_level {
                    // Remove the text line we already added
                    section_lines.pop();
                    break;
                }
            }
            section_lines.push(line);
            if section_lines.len() >= 30 {
                section_lines.push("...");
                break;
            }
        } else if line.starts_with('#') {
            let heading_text = line.trim_start_matches('#').trim().to_lowercase();
            if config_keywords.iter().any(|kw| heading_text.contains(kw)) {
                in_section = true;
                section_level = line.chars().take_while(|c| *c == '#').count();
            }
        } else if is_setext_underline(line) && prev_line.is_some_and(|p| !p.trim().is_empty()) {
            // Setext heading: prev_line is the heading text
            let heading_text = prev_line.unwrap().trim().to_lowercase();
            if config_keywords.iter().any(|kw| heading_text.contains(kw)) {
                in_section = true;
                section_level = if line.trim().starts_with('=') { 1 } else { 2 };
            }
        }
        prev_line = Some(line);
    }

    if section_lines.is_empty() {
        return None;
    }

    // Trim leading/trailing blank lines
    while section_lines.first().is_some_and(|l| l.trim().is_empty()) {
        section_lines.remove(0);
    }
    while section_lines.last().is_some_and(|l| l.trim().is_empty()) {
        section_lines.pop();
    }

    if section_lines.is_empty() {
        None
    } else {
        Some(section_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_description_basic() {
        let content = "# My Plugin\n\nThis is a great plugin for doing things.\n\n## Installation\n";
        let desc = extract_description(content).unwrap();
        assert_eq!(desc, "This is a great plugin for doing things.");
    }

    #[test]
    fn test_extract_description_with_badges() {
        let content = "# Plugin\n\n[![Build](https://badge.svg)](url)\n![License](badge)\n\nActual description here.\n";
        let desc = extract_description(content).unwrap();
        assert_eq!(desc, "Actual description here.");
    }

    #[test]
    fn test_extract_description_multiline() {
        let content = "# Plugin\n\nFirst line of description.\nSecond line continues.\n\n## Next section\n";
        let desc = extract_description(content).unwrap();
        assert_eq!(desc, "First line of description. Second line continues.");
    }

    #[test]
    fn test_extract_description_empty() {
        let content = "# Plugin\n\n";
        let desc = extract_description(content);
        assert!(desc.is_none());
    }

    #[test]
    fn test_extract_config_section() {
        let content = "# Plugin\n\nDescription.\n\n## Configuration\n\nSet `FOO=bar` to enable.\n\n## License\n\nMIT\n";
        let config = extract_config_section(content).unwrap();
        assert!(config.contains("FOO=bar"));
    }

    #[test]
    fn test_extract_config_section_options_heading() {
        let content = "# Plugin\n\n## Options\n\n- `opt1`: does X\n- `opt2`: does Y\n\n## Other\n";
        let config = extract_config_section(content).unwrap();
        assert!(config.contains("opt1"));
        assert!(config.contains("opt2"));
    }

    #[test]
    fn test_extract_config_section_none() {
        let content = "# Plugin\n\nJust a description.\n\n## License\n\nMIT\n";
        let config = extract_config_section(content);
        assert!(config.is_none());
    }

    #[test]
    fn test_clean_git_url_ssh() {
        assert_eq!(
            clean_git_url("git@github.com:user/repo.git"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_clean_git_url_https() {
        assert_eq!(
            clean_git_url("https://github.com/user/repo.git"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn test_resolve_url_shorthand() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(
            resolve_url("zsh-users/zsh-autosuggestions", tmp.path()),
            "https://github.com/zsh-users/zsh-autosuggestions"
        );
    }

    #[test]
    fn test_scan_plugin_no_readme() {
        let tmp = TempDir::new().unwrap();
        let info = scan_plugin(tmp.path(), "test-plugin", "user/test-plugin");
        assert_eq!(info.name, "test-plugin");
        assert_eq!(info.url, "https://github.com/user/test-plugin");
        assert!(info.description.is_none());
        assert!(info.config_excerpt.is_none());
    }

    #[test]
    fn test_scan_plugin_with_readme() {
        let tmp = TempDir::new().unwrap();
        let readme = "# test-plugin\n\nA useful plugin for testing.\n\n## Configuration\n\nSet `TEST_OPT=1`.\n";
        std::fs::write(tmp.path().join("README.md"), readme).unwrap();

        let info = scan_plugin(tmp.path(), "test-plugin", "user/test-plugin");
        assert_eq!(info.description.as_deref(), Some("A useful plugin for testing."));
        assert!(info.config_excerpt.as_deref().unwrap().contains("TEST_OPT"));
    }

    #[test]
    fn test_extract_description_setext_heading() {
        let content = "My Plugin [![Build](badge)][ci]\n=======================\n\n**Great plugin for doing things.**\n\nMore text.\n";
        let desc = extract_description(content).unwrap();
        assert_eq!(desc, "**Great plugin for doing things.**");
    }

    #[test]
    fn test_extract_config_section_setext_heading() {
        let content = "Plugin\n======\n\nDescription.\n\nConfiguration\n-------------\n\nSet `BAR=1`.\n\nLicense\n-------\n\nMIT\n";
        let config = extract_config_section(content).unwrap();
        assert!(config.contains("BAR=1"));
        assert!(!config.contains("MIT"));
    }
}
