use colored::Colorize;
use similar::{ChangeTag, TextDiff};

/// Format a unified diff between two strings with colored output.
/// Returns a formatted string ready for terminal display.
pub fn unified_diff(old: &str, new: &str, file_path: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

    // Header
    output.push_str(&format!("--- a/{}\n", file_path).bold().to_string());
    output.push_str(&format!("+++ b/{}\n", file_path).bold().to_string());

    for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
        output.push_str(&format!("{}", hunk.header()).cyan().to_string());

        for change in hunk.iter_changes() {
            let line = match change.tag() {
                ChangeTag::Delete => format!("-{}", change).red().to_string(),
                ChangeTag::Insert => format!("+{}", change).green().to_string(),
                ChangeTag::Equal => format!(" {}", change).to_string(),
            };
            output.push_str(&line);
        }
    }

    output
}

/// Check if two strings have any differences.
#[allow(dead_code)]
pub fn has_changes(old: &str, new: &str) -> bool {
    old != new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_diff_basic() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nmodified\nline3\n";

        let result = unified_diff(old, new, "test.conf");
        assert!(result.contains("test.conf"));
    }

    #[test]
    fn test_unified_diff_addition() {
        let old = "line1\n";
        let new = "line1\nline2\n";

        let result = unified_diff(old, new, "test.conf");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_no_diff() {
        let content = "same\n";
        let result = unified_diff(content, content, "test.conf");
        // No hunks for identical content — only header lines present
        assert!(!result.contains("@@"));
    }

    #[test]
    fn test_has_changes() {
        assert!(has_changes("a", "b"));
        assert!(!has_changes("same", "same"));
    }

    #[test]
    fn test_empty_to_content() {
        let old = "";
        let new = "new content\n";

        let result = unified_diff(old, new, "new.conf");
        assert!(!result.is_empty());
    }
}
