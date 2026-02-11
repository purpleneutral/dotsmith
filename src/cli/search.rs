use anyhow::Result;
use colored::Colorize;

use crate::core::module::{ModuleRegistry, OptionEntry};

pub fn run(_verbose: bool, query: &str) -> Result<()> {
    let query_lower = query.to_lowercase();
    let mut total_results = 0;
    let mut tools_with_results = 0;

    for tool_name in ModuleRegistry::builtin_names() {
        let Some(db) = ModuleRegistry::get_options(tool_name) else {
            continue;
        };

        let matches: Vec<&OptionEntry> = db
            .options
            .iter()
            .filter(|opt| matches_query(opt, &query_lower))
            .collect();

        if matches.is_empty() {
            continue;
        }

        tools_with_results += 1;
        total_results += matches.len();

        println!();
        println!("  {}", tool_name.cyan().bold());

        for opt in &matches {
            let type_str = format!("{:?}", opt.option_type).to_lowercase();
            println!(
                "    {} ({}) [{}]",
                opt.name.bold(),
                type_str,
                opt.category.dimmed()
            );
            println!("      {}", opt.description);
            if let Some(ref example) = opt.example {
                println!("      Example: {}", example.dimmed());
            }
            if let Some(ref url) = opt.url {
                println!("      {}", url.blue().underline());
            }
            println!();
        }
    }

    if total_results == 0 {
        println!("No results for \"{}\"", query);
    } else {
        println!(
            "  {} result(s) across {} tool(s)",
            total_results, tools_with_results
        );
    }

    Ok(())
}

fn matches_query(opt: &OptionEntry, query: &str) -> bool {
    let q = query.to_lowercase();
    if opt.name.to_lowercase().contains(&q) {
        return true;
    }
    if opt.description.to_lowercase().contains(&q) {
        return true;
    }
    if opt.category.to_lowercase().contains(&q) {
        return true;
    }
    if let Some(ref tags) = opt.tags {
        if tags.iter().any(|t| t.to_lowercase().contains(&q)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_finds_mouse() {
        let db = ModuleRegistry::get_options("tmux").unwrap();
        let matches: Vec<&OptionEntry> = db
            .options
            .iter()
            .filter(|opt| matches_query(opt, "mouse"))
            .collect();
        assert!(!matches.is_empty(), "should find 'mouse' in tmux options");
        assert!(matches.iter().any(|o| o.name == "mouse"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let db = ModuleRegistry::get_options("tmux").unwrap();
        let upper: Vec<&OptionEntry> = db
            .options
            .iter()
            .filter(|opt| matches_query(opt, "MOUSE"))
            .collect();
        let lower: Vec<&OptionEntry> = db
            .options
            .iter()
            .filter(|opt| matches_query(opt, "mouse"))
            .collect();
        assert_eq!(upper.len(), lower.len());
    }

    #[test]
    fn test_search_no_results() {
        let db = ModuleRegistry::get_options("tmux").unwrap();
        let matches: Vec<&OptionEntry> = db
            .options
            .iter()
            .filter(|opt| matches_query(opt, "zzzznonexistent"))
            .collect();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_search_matches_tags() {
        let db = ModuleRegistry::get_options("tmux").unwrap();
        // "mouse" option should have a relevant tag
        let matches: Vec<&OptionEntry> = db
            .options
            .iter()
            .filter(|opt| matches_query(opt, "interaction"))
            .collect();
        assert!(!matches.is_empty(), "should match category/tags");
    }

    #[test]
    fn test_search_matches_description() {
        let db = ModuleRegistry::get_options("tmux").unwrap();
        let matches: Vec<&OptionEntry> = db
            .options
            .iter()
            .filter(|opt| matches_query(opt, "clipboard"))
            .collect();
        assert!(!matches.is_empty(), "should match in description");
    }
}
