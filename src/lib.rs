use std::env;
use std::path::Path;

mod cli;
mod help;
mod journal;
mod model;
mod query;
mod render;
mod section;

use cli::parse_args;
use help::HELP_TEXT;
use journal::{
    NO_ENTRIES_MESSAGE, discover_journal_dir_from, group_files_by_date, list_journal_files,
    read_file,
};
use model::DisplayMode;
use query::{EntryContent, MatchedEntry, apply_filters};
use render::{format_file_blocks, format_grouped_files};
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
    let entries = apply_filters(files, &params, &journal_dir);

    let output = match &params.display_mode {
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
