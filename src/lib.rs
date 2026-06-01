use std::env;
use std::path::Path;

mod cli;
mod help;
mod journal;
mod model;
mod query;
mod render;
pub mod search;
mod section;

use std::time::{SystemTime, UNIX_EPOCH};

use cli::parse_args;
use help::HELP_TEXT;
use journal::{
    NO_ENTRIES_MESSAGE, discover_journal_dir_from, group_files_by_date, list_journal_files,
    read_file,
};
use model::DisplayMode;
use query::{EntryContent, MatchedEntry, apply_filters};
use render::{format_file_blocks, format_grouped_files, format_search_results};
use search::search_entries;
use section::extract_section;

pub fn run(args: &[String]) -> Result<String, String> {
    let current_dir = env::current_dir()
        .map_err(|error| format!("Failed to determine current directory: {error}"))?;
    execute_from(args, &current_dir)
}

fn execute_from(args: &[String], current_dir: &Path) -> Result<String, String> {
    let params = parse_args(args)?;
    if params.help_requested {
        return Ok(HELP_TEXT.to_string());
    }

    let Some(journal_dir) = discover_journal_dir_from(current_dir) else {
        return Ok(NO_ENTRIES_MESSAGE.to_string());
    };

    let files = list_journal_files(&journal_dir);
    let mut entries = apply_filters(files, &params, &journal_dir);

    if params.display_mode == DisplayMode::Search {
        let query = params.search_query.as_deref().unwrap_or("");
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let filenames: Vec<String> = entries.iter().map(|e| e.file_name.clone()).collect();
        let mut results = search_entries(&filenames, &journal_dir, query, now_unix);

        let limit = params.latest.unwrap_or(10);
        results.truncate(limit);

        return Ok(format_search_results(&results));
    }

    if let Some(count) = params.latest {
        entries.truncate(count);
    }

    let display_mode = if params.latest.is_some() && params.display_mode == DisplayMode::List {
        &DisplayMode::Summary
    } else {
        &params.display_mode
    };

    let output = match display_mode {
        DisplayMode::List => {
            let names = entries
                .iter()
                .map(|entry| entry.file_name.clone())
                .collect::<Vec<_>>();
            let grouped = group_files_by_date(&names);
            let grouped_output = format_grouped_files(&grouped);
            let read_errors = entries
                .iter()
                .filter_map(|entry| match entry.content.as_ref() {
                    Some(EntryContent::ReadError(error)) => {
                        Some(format!("{}\n{}", entry.file_name, error))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            if read_errors.is_empty() {
                grouped_output
            } else {
                format!("{grouped_output}\n\n{}", read_errors.join("\n\n"))
            }
        }
        DisplayMode::Full => {
            let blocks = entries
                .iter()
                .map(|entry| {
                    let body = match load_entry_content(entry, &journal_dir) {
                        EntryContent::Loaded(content) => content.trim_end().to_string(),
                        EntryContent::ReadError(error) => error,
                    };
                    (entry.file_name.clone(), body)
                })
                .collect::<Vec<_>>();
            format_file_blocks(&blocks)
        }
        DisplayMode::Summary => format_section_blocks(&entries, &journal_dir, "Summary"),
        DisplayMode::TypeSection(heading) => format_section_blocks(&entries, &journal_dir, heading),
        DisplayMode::Search => unreachable!(),
    };

    Ok(output)
}

fn format_section_blocks(entries: &[MatchedEntry], journal_dir: &Path, heading: &str) -> String {
    let missing_message = format!("[No ## {} section found]", heading.trim());
    let blocks = entries
        .iter()
        .map(|entry| {
            let body = match load_entry_content(entry, journal_dir) {
                EntryContent::Loaded(content) => {
                    extract_section(&content, heading).unwrap_or_else(|| missing_message.clone())
                }
                EntryContent::ReadError(error) => error,
            };
            (entry.file_name.clone(), body)
        })
        .collect::<Vec<_>>();

    format_file_blocks(&blocks)
}

fn load_entry_content(entry: &MatchedEntry, journal_dir: &Path) -> EntryContent {
    if let Some(content) = entry.content.as_ref() {
        return content.clone();
    }

    match read_file(journal_dir, &entry.file_name) {
        Ok(content) => EntryContent::Loaded(content),
        Err(error) => EntryContent::ReadError(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn runs_combined_type_since_and_filter_query() {
        let temp_dir = create_temp_dir("combined");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-04-25-0900-old.md"),
            "# Entry\n\n## Next Actions\n\n- archive docs\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-04-27-1100-release.md"),
            "# Entry\n\n## Next Actions\n\n- deploy release\n\n## Notes\n\ndeploy checklist\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-04-28-1200-other.md"),
            "# Entry\n\n## Next Actions\n\n- investigate\n",
        )
        .unwrap();

        let args = vec![
            "--type".to_string(),
            "Next Actions".to_string(),
            "--since".to_string(),
            "2026-04-26".to_string(),
            "--filter".to_string(),
            "deploy".to_string(),
        ];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("2026-04-27-1100-release.md"));
        assert!(output.contains("- deploy release"));
        assert!(!output.contains("2026-04-25-0900-old.md"));
        assert!(!output.contains("2026-04-28-1200-other.md"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn returns_error_for_invalid_between_range() {
        let temp_dir = create_temp_dir("invalid-range");
        fs::create_dir(temp_dir.join(".journal")).unwrap();

        let args = vec![
            "--between".to_string(),
            "2026-05-02".to_string(),
            "2026-05-01".to_string(),
        ];
        let error = execute_from(&args, &temp_dir).unwrap_err();

        assert!(error.contains("start must be <="));
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn uses_issues_alias_when_type_is_issues() {
        let temp_dir = create_temp_dir("issues-alias");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-05-01-1015-issues.md"),
            "# Entry\n\n## Issues / Unknowns\n\n- waiting on review\n",
        )
        .unwrap();

        let args = vec!["--type".to_string(), "Issues".to_string()];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("- waiting on review"));
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn reports_read_errors_without_stopping_other_results() {
        let temp_dir = create_temp_dir("read-error");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-05-01-0900-valid.md"),
            "# Entry\n\nDeploy checklist\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-05-01-0910-bad.md"),
            vec![0xff, 0xfe, 0xfd],
        )
        .unwrap();

        let args = vec![
            "--full".to_string(),
            "--filter".to_string(),
            "deploy".to_string(),
        ];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("2026-05-01-0900-valid.md"));
        assert!(output.contains("Deploy checklist"));
        assert!(output.contains("2026-05-01-0910-bad.md"));
        assert!(output.contains("Failed to read"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn latest_shows_only_n_most_recent_entries_with_summary() {
        let temp_dir = create_temp_dir("latest");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-05-01-0900-first.md"),
            "# Entry\n\n## Summary\n\nFirst entry summary\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-05-02-1000-second.md"),
            "# Entry\n\n## Summary\n\nSecond entry summary\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-05-03-1100-third.md"),
            "# Entry\n\n## Summary\n\nThird entry summary\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-05-04-1200-fourth.md"),
            "# Entry\n\n## Summary\n\nFourth entry summary\n",
        )
        .unwrap();

        let args = vec!["--latest".to_string(), "2".to_string()];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(!output.contains("First entry summary"));
        assert!(!output.contains("Second entry summary"));
        assert!(output.contains("2026-05-03-1100-third.md"));
        assert!(output.contains("Third entry summary"));
        assert!(output.contains("2026-05-04-1200-fourth.md"));
        assert!(output.contains("Fourth entry summary"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_returns_ranked_results() {
        let temp_dir = create_temp_dir("search-basic");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-05-04-1200-notes.md"),
            "# Entry\n\n## Summary\n\nAdded notes that the owner can send to students or instructors.\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-05-03-1000-login.md"),
            "# Entry\n\n## Summary\n\nStudents can now log in to the platform.\n",
        )
        .unwrap();

        let args = vec!["--search".to_string(), "notes for students".to_string()];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("2026-05-04-1200-notes.md"));
        assert!(output.contains("2026-05-03-1000-login.md"));
        // Notes entry should appear first (higher score)
        let notes_pos = output.find("2026-05-04-1200-notes.md").unwrap();
        let login_pos = output.find("2026-05-03-1000-login.md").unwrap();
        assert!(notes_pos < login_pos);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_combined_with_since_filters_first() {
        let temp_dir = create_temp_dir("search-since");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-04-01-0900-old-deploy.md"),
            "# Entry\n\n## Summary\n\nDeployed authentication service.\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-05-15-1000-new-deploy.md"),
            "# Entry\n\n## Summary\n\nDeployed authentication updates.\n",
        )
        .unwrap();

        let args = vec![
            "--search".to_string(),
            "deploy authentication".to_string(),
            "--since".to_string(),
            "2026-05-01".to_string(),
        ];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("2026-05-15-1000-new-deploy.md"));
        assert!(!output.contains("2026-04-01-0900-old-deploy.md"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_combined_with_between_filters_range() {
        let temp_dir = create_temp_dir("search-between");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-04-01-0900-before.md"),
            "# Entry\n\n## Summary\n\nSetup database migrations.\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-04-15-1000-in-range.md"),
            "# Entry\n\n## Summary\n\nRan database migrations for users table.\n",
        )
        .unwrap();
        fs::write(
            journal_dir.join("2026-05-01-1000-after.md"),
            "# Entry\n\n## Summary\n\nDatabase migration cleanup.\n",
        )
        .unwrap();

        let args = vec![
            "--search".to_string(),
            "database migration".to_string(),
            "--between".to_string(),
            "2026-04-10".to_string(),
            "2026-04-20".to_string(),
        ];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("2026-04-15-1000-in-range.md"));
        assert!(!output.contains("2026-04-01-0900-before.md"));
        assert!(!output.contains("2026-05-01-1000-after.md"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_and_filter_mutual_exclusion() {
        let temp_dir = create_temp_dir("search-filter");
        fs::create_dir(temp_dir.join(".journal")).unwrap();

        let args = vec![
            "--search".to_string(),
            "deploy".to_string(),
            "--filter".to_string(),
            "deploy".to_string(),
        ];
        let error = execute_from(&args, &temp_dir).unwrap_err();

        assert!(error.contains("--search and --filter cannot be used together"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_with_latest_caps_results() {
        let temp_dir = create_temp_dir("search-latest");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        for i in 1..=5 {
            fs::write(
                journal_dir.join(format!("2026-05-0{i}-1000-entry{i}.md")),
                format!("# Entry\n\n## Summary\n\nDeploy iteration {i} to production.\n"),
            )
            .unwrap();
        }

        let args = vec![
            "--search".to_string(),
            "deploy production".to_string(),
            "--latest".to_string(),
            "3".to_string(),
        ];
        let output = execute_from(&args, &temp_dir).unwrap();

        let result_count = output.matches("[").count();
        assert_eq!(result_count, 3);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_all_stop_words_returns_no_matches() {
        let temp_dir = create_temp_dir("search-stopwords");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-05-01-1000-test.md"),
            "# Entry\n\n## Summary\n\nSome content here.\n",
        )
        .unwrap();

        let args = vec!["--search".to_string(), "the and or".to_string()];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("No matching entries found."));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_time_bias_recent_strips_keyword() {
        let temp_dir = create_temp_dir("search-bias");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-05-30-1000-auth.md"),
            "# Entry\n\n## Summary\n\nUpdated authentication flow.\n",
        )
        .unwrap();

        let args = vec!["--search".to_string(), "recent authentication".to_string()];
        let output = execute_from(&args, &temp_dir).unwrap();

        // Should match on "authentication" (not "recent")
        assert!(output.contains("2026-05-30-1000-auth.md"));
        assert!(output.contains("authentication"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_no_summary_entries_returns_no_matches() {
        let temp_dir = create_temp_dir("search-nosummary");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-05-01-1000-nosummary.md"),
            "# Entry\n\n## Context\n\nJust context here.\n",
        )
        .unwrap();

        let args = vec!["--search".to_string(), "context".to_string()];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("No matching entries found."));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_without_value_returns_error() {
        let temp_dir = create_temp_dir("search-novalue");
        fs::create_dir(temp_dir.join(".journal")).unwrap();

        let args = vec!["--search".to_string()];
        let error = execute_from(&args, &temp_dir).unwrap_err();

        assert!(error.contains("--search requires a query string"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn search_positional_command_works() {
        let temp_dir = create_temp_dir("search-positional");
        let journal_dir = temp_dir.join(".journal");
        fs::create_dir(&journal_dir).unwrap();

        fs::write(
            journal_dir.join("2026-05-01-1000-deploy.md"),
            "# Entry\n\n## Summary\n\nDeployed the service to production.\n",
        )
        .unwrap();

        let args = vec!["search".to_string(), "deploy production".to_string()];
        let output = execute_from(&args, &temp_dir).unwrap();

        assert!(output.contains("2026-05-01-1000-deploy.md"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    fn create_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("journal-lib-tests-{prefix}-{unique}"));
        fs::create_dir(&path).unwrap();
        path
    }
}
