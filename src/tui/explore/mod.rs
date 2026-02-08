pub mod handler;
pub mod view;

use crate::core::module::{ModuleRegistry, OptionEntry};

/// Which panel has focus in the explore view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Categories,
    Options,
    Details,
}

/// A category item with its option count.
#[derive(Debug, Clone)]
pub struct CategoryItem {
    pub name: String,
    pub count: usize,
}

/// State for the explore view.
pub struct ExploreState {
    pub tool_name: String,
    pub all_options: Vec<OptionEntry>,
    pub categories: Vec<CategoryItem>,
    pub filtered_indices: Vec<usize>,
    pub focus: Panel,
    pub category_selected: usize,
    pub option_selected: usize,
    pub search_mode: bool,
    pub search_query: String,
}

impl ExploreState {
    /// Create a new explore state for a Tier 1 tool.
    /// Returns `None` if the tool has no option database.
    pub fn new(tool_name: &str) -> Option<Self> {
        let db = ModuleRegistry::get_options(tool_name)?;
        let all_options = db.options;

        // Build category list with counts, starting with "All"
        let mut cat_counts = std::collections::BTreeMap::new();
        for opt in &all_options {
            *cat_counts.entry(opt.category.clone()).or_insert(0usize) += 1;
        }

        let total = all_options.len();
        let mut categories = vec![CategoryItem {
            name: "All".to_string(),
            count: total,
        }];
        for (name, count) in &cat_counts {
            categories.push(CategoryItem {
                name: name.clone(),
                count: *count,
            });
        }

        // Start with all options visible
        let filtered_indices: Vec<usize> = (0..all_options.len()).collect();

        Some(Self {
            tool_name: tool_name.to_string(),
            all_options,
            categories,
            filtered_indices,
            focus: Panel::Categories,
            category_selected: 0,
            option_selected: 0,
            search_mode: false,
            search_query: String::new(),
        })
    }

    /// Apply the current category filter (and search if active).
    pub fn apply_filters(&mut self) {
        let category = &self.categories[self.category_selected].name;
        let is_all = category == "All";
        let query = self.search_query.to_lowercase();

        self.filtered_indices = self
            .all_options
            .iter()
            .enumerate()
            .filter(|(_, opt)| {
                // Category filter
                if !is_all && opt.category != *category {
                    return false;
                }
                // Search filter
                if !query.is_empty() {
                    let name_match = opt.name.to_lowercase().contains(&query);
                    let desc_match = opt.description.to_lowercase().contains(&query);
                    let tag_match = opt
                        .tags
                        .as_ref()
                        .is_some_and(|tags| tags.iter().any(|t| t.to_lowercase().contains(&query)));
                    if !name_match && !desc_match && !tag_match {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

        // Clamp option selection
        if self.filtered_indices.is_empty() {
            self.option_selected = 0;
        } else {
            self.option_selected = self.option_selected.min(self.filtered_indices.len() - 1);
        }
    }

    /// Get the currently selected option, if any.
    pub fn selected_option(&self) -> Option<&OptionEntry> {
        self.filtered_indices
            .get(self.option_selected)
            .map(|&i| &self.all_options[i])
    }

    pub fn select_next_category(&mut self) {
        if !self.categories.is_empty() {
            self.category_selected = (self.category_selected + 1).min(self.categories.len() - 1);
            self.apply_filters();
        }
    }

    pub fn select_prev_category(&mut self) {
        self.category_selected = self.category_selected.saturating_sub(1);
        self.apply_filters();
    }

    pub fn select_next_option(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.option_selected = (self.option_selected + 1).min(self.filtered_indices.len() - 1);
        }
    }

    pub fn select_prev_option(&mut self) {
        self.option_selected = self.option_selected.saturating_sub(1);
    }

    pub fn cycle_focus_forward(&mut self) {
        self.focus = match self.focus {
            Panel::Categories => Panel::Options,
            Panel::Options => Panel::Details,
            Panel::Details => Panel::Categories,
        };
    }

    pub fn cycle_focus_backward(&mut self) {
        self.focus = match self.focus {
            Panel::Categories => Panel::Details,
            Panel::Options => Panel::Categories,
            Panel::Details => Panel::Options,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tmux() {
        let state = ExploreState::new("tmux").unwrap();
        assert_eq!(state.tool_name, "tmux");
        assert_eq!(state.all_options.len(), 31);
        assert!(state.categories.len() > 1);
        assert_eq!(state.categories[0].name, "All");
        assert_eq!(state.categories[0].count, 31);
        assert_eq!(state.filtered_indices.len(), 31);
    }

    #[test]
    fn test_new_zsh() {
        let state = ExploreState::new("zsh").unwrap();
        assert_eq!(state.tool_name, "zsh");
        assert_eq!(state.all_options.len(), 31);
    }

    #[test]
    fn test_new_git() {
        let state = ExploreState::new("git").unwrap();
        assert_eq!(state.tool_name, "git");
        assert_eq!(state.all_options.len(), 31);
    }

    #[test]
    fn test_new_nonexistent() {
        assert!(ExploreState::new("nonexistent").is_none());
    }

    #[test]
    fn test_category_filter() {
        let mut state = ExploreState::new("tmux").unwrap();

        // "All" shows everything
        assert_eq!(state.filtered_indices.len(), 31);

        // Select a specific category
        state.select_next_category();
        let cat_name = state.categories[state.category_selected].name.clone();
        let expected_count = state.categories[state.category_selected].count;
        assert_eq!(state.filtered_indices.len(), expected_count);

        // All filtered options should belong to that category
        for &i in &state.filtered_indices {
            assert_eq!(state.all_options[i].category, cat_name);
        }
    }

    #[test]
    fn test_search_filter() {
        let mut state = ExploreState::new("tmux").unwrap();
        state.search_query = "mouse".to_string();
        state.apply_filters();
        assert!(!state.filtered_indices.is_empty());
        // "mouse" option should be in results
        assert!(state
            .filtered_indices
            .iter()
            .any(|&i| state.all_options[i].name == "mouse"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut state = ExploreState::new("tmux").unwrap();
        state.search_query = "MOUSE".to_string();
        state.apply_filters();
        assert!(state
            .filtered_indices
            .iter()
            .any(|&i| state.all_options[i].name == "mouse"));
    }

    #[test]
    fn test_search_no_results() {
        let mut state = ExploreState::new("tmux").unwrap();
        state.search_query = "zzzznonexistent".to_string();
        state.apply_filters();
        assert!(state.filtered_indices.is_empty());
        assert!(state.selected_option().is_none());
    }

    #[test]
    fn test_category_plus_search() {
        let mut state = ExploreState::new("git").unwrap();
        // Select "core" category
        let core_idx = state
            .categories
            .iter()
            .position(|c| c.name == "core")
            .unwrap();
        state.category_selected = core_idx;
        state.search_query = "editor".to_string();
        state.apply_filters();
        assert!(!state.filtered_indices.is_empty());
        for &i in &state.filtered_indices {
            assert_eq!(state.all_options[i].category, "core");
        }
    }

    #[test]
    fn test_focus_cycling() {
        let mut state = ExploreState::new("tmux").unwrap();
        assert_eq!(state.focus, Panel::Categories);
        state.cycle_focus_forward();
        assert_eq!(state.focus, Panel::Options);
        state.cycle_focus_forward();
        assert_eq!(state.focus, Panel::Details);
        state.cycle_focus_forward();
        assert_eq!(state.focus, Panel::Categories);

        state.cycle_focus_backward();
        assert_eq!(state.focus, Panel::Details);
    }

    #[test]
    fn test_option_navigation() {
        let mut state = ExploreState::new("tmux").unwrap();
        assert_eq!(state.option_selected, 0);
        state.select_next_option();
        assert_eq!(state.option_selected, 1);
        state.select_prev_option();
        assert_eq!(state.option_selected, 0);
        state.select_prev_option();
        assert_eq!(state.option_selected, 0); // clamped
    }

    #[test]
    fn test_selected_option() {
        let state = ExploreState::new("tmux").unwrap();
        let opt = state.selected_option().unwrap();
        assert_eq!(opt, &state.all_options[0]);
    }
}
