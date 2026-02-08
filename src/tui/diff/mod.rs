pub mod handler;
pub mod view;

use similar::ChangeTag;

use crate::core::manifest::Manifest;
use crate::core::snapshot::{FileDiff, SnapshotEngine};

/// A parsed diff line for TUI rendering.
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiffLineKind {
    Header,
    HunkHeader,
    Added,
    Removed,
    Context,
    Empty,
}

/// State for the diff view.
pub struct DiffState {
    pub tool_name: String,
    pub lines: Vec<DiffLine>,
    pub scroll_offset: usize,
    pub visible_height: usize,
    pub total_files: usize,
    pub has_changes: bool,
}

impl DiffState {
    /// Build diff state from a tool's current state vs last snapshot.
    pub fn from_tool(tool: &str, engine: &SnapshotEngine, manifest: &Manifest) -> Self {
        let diffs = manifest
            .tools
            .get(tool)
            .map(|e| engine.diff_current(tool, &e.config_paths).unwrap_or_default())
            .unwrap_or_default();

        let lines = build_diff_lines(&diffs);
        let total_files = diffs.len();
        let has_changes = !diffs.is_empty();

        Self {
            tool_name: tool.to_string(),
            lines,
            scroll_offset: 0,
            visible_height: 20,
            total_files,
            has_changes,
        }
    }

    /// Build diff state from two known strings (for viewing a snapshot diff).
    pub fn from_strings(tool: &str, file_path: &str, old: &str, new: &str) -> Self {
        let diffs = vec![FileDiff {
            file_path: file_path.to_string(),
            old_content: old.to_string(),
            new_content: new.to_string(),
        }];

        let lines = build_diff_lines(&diffs);
        let has_changes = old != new;

        Self {
            tool_name: tool.to_string(),
            lines,
            scroll_offset: 0,
            visible_height: 20,
            total_files: 1,
            has_changes,
        }
    }

    pub fn scroll_down(&mut self) {
        let max = self.lines.len().saturating_sub(self.visible_height);
        self.scroll_offset = (self.scroll_offset + 1).min(max);
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn page_down(&mut self) {
        let max = self.lines.len().saturating_sub(self.visible_height);
        self.scroll_offset = (self.scroll_offset + self.visible_height).min(max);
    }

    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_height);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_end(&mut self) {
        self.scroll_offset = self.lines.len().saturating_sub(self.visible_height);
    }
}

fn build_diff_lines(diffs: &[FileDiff]) -> Vec<DiffLine> {
    let mut lines = Vec::new();

    for file_diff in diffs {
        lines.push(DiffLine {
            kind: DiffLineKind::Header,
            content: format!("--- a/{}", file_diff.file_path),
        });
        lines.push(DiffLine {
            kind: DiffLineKind::Header,
            content: format!("+++ b/{}", file_diff.file_path),
        });

        let text_diff = similar::TextDiff::from_lines(
            &file_diff.old_content,
            &file_diff.new_content,
        );

        for hunk in text_diff.unified_diff().context_radius(3).iter_hunks() {
            lines.push(DiffLine {
                kind: DiffLineKind::HunkHeader,
                content: format!("{}", hunk.header()),
            });
            for change in hunk.iter_changes() {
                let (kind, prefix) = match change.tag() {
                    ChangeTag::Delete => (DiffLineKind::Removed, "-"),
                    ChangeTag::Insert => (DiffLineKind::Added, "+"),
                    ChangeTag::Equal => (DiffLineKind::Context, " "),
                };
                let text = change.to_string_lossy();
                lines.push(DiffLine {
                    kind,
                    content: format!("{}{}", prefix, text.trim_end_matches('\n')),
                });
            }
        }

        lines.push(DiffLine {
            kind: DiffLineKind::Empty,
            content: String::new(),
        });
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_strings_no_changes() {
        let state = DiffState::from_strings("tmux", "tmux.conf", "hello\n", "hello\n");
        assert!(!state.has_changes);
        assert_eq!(state.total_files, 1);
    }

    #[test]
    fn test_from_strings_with_changes() {
        let state = DiffState::from_strings("tmux", "tmux.conf", "old\n", "new\n");
        assert!(state.has_changes);
        assert!(!state.lines.is_empty());

        // Should have headers
        assert!(state.lines.iter().any(|l| l.kind == DiffLineKind::Header));
        // Should have added and removed lines
        assert!(state.lines.iter().any(|l| l.kind == DiffLineKind::Added));
        assert!(state.lines.iter().any(|l| l.kind == DiffLineKind::Removed));
    }

    #[test]
    fn test_scroll() {
        let mut state = DiffState::from_strings("tmux", "tmux.conf", "a\nb\nc\n", "x\ny\nz\n");
        state.visible_height = 2;

        state.scroll_down();
        assert_eq!(state.scroll_offset, 1);
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0); // clamped

        state.scroll_to_end();
        assert!(state.scroll_offset > 0);
        state.scroll_to_top();
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_page_navigation() {
        let mut state = DiffState::from_strings(
            "tmux",
            "tmux.conf",
            "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n",
            "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n",
        );
        state.visible_height = 3;

        state.page_down();
        assert_eq!(state.scroll_offset, 3);
        state.page_up();
        assert_eq!(state.scroll_offset, 0);
    }
}
