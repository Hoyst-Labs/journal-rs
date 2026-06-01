use std::path::Path;

use crate::journal::{
    extract_date_prefix, extract_timestamp_prefix, is_valid_date, is_valid_date_time, read_file,
};
use crate::model::QueryParams;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryContent {
    Loaded(String),
    ReadError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchedEntry {
    pub file_name: String,
    pub content: Option<EntryContent>,
}

pub fn apply_filters(
    files: Vec<String>,
    params: &QueryParams,
    journal_dir: &Path,
) -> Vec<MatchedEntry> {
    let mut entries: Vec<MatchedEntry> = files
        .into_iter()
        .filter(|file| matches_files_query(file, params.files_query.as_deref()))
        .filter(|file| {
            params
                .since
                .as_ref()
                .is_none_or(|since| matches_since(file, since))
        })
        .filter(|file| {
            params
                .between
                .as_ref()
                .is_none_or(|(start, end)| matches_between(file, start, end))
        })
        .map(|file_name| MatchedEntry {
            file_name,
            content: None,
        })
        .collect();

    if let Some(terms) = params.filter_terms.as_ref() {
        entries = entries
            .into_iter()
            .filter_map(|entry| {
                let result = read_file(journal_dir, &entry.file_name);
                match result {
                    Ok(content) => {
                        if matches_content_filter(&content, terms) {
                            Some(MatchedEntry {
                                file_name: entry.file_name,
                                content: Some(EntryContent::Loaded(content)),
                            })
                        } else {
                            None
                        }
                    }
                    Err(error) => Some(MatchedEntry {
                        file_name: entry.file_name,
                        content: Some(EntryContent::ReadError(error)),
                    }),
                }
            })
            .collect();
    }

    entries
}

pub fn matches_files_query(file_name: &str, query: Option<&str>) -> bool {
    query.is_none_or(|value| file_name.starts_with(value))
}

pub fn matches_since(file_name: &str, since: &str) -> bool {
    if is_valid_date_time(since) {
        return comparable_datetime_prefix(file_name).is_some_and(|value| value.as_str() >= since);
    }

    if is_valid_date(since) {
        return extract_date_prefix(file_name).is_some_and(|value| value >= since);
    }

    false
}

pub fn matches_between(file_name: &str, start: &str, end: &str) -> bool {
    let lower_bound_ok = if is_valid_date_time(start) {
        comparable_datetime_prefix(file_name).is_some_and(|value| value.as_str() >= start)
    } else if is_valid_date(start) {
        extract_date_prefix(file_name).is_some_and(|value| value >= start)
    } else {
        false
    };

    let upper_bound_ok = if is_valid_date_time(end) {
        comparable_datetime_prefix(file_name).is_some_and(|value| value.as_str() <= end)
    } else if is_valid_date(end) {
        extract_date_prefix(file_name).is_some_and(|value| value <= end)
    } else {
        false
    };

    lower_bound_ok && upper_bound_ok
}

pub fn matches_content_filter(content: &str, terms: &[String]) -> bool {
    if terms.is_empty() {
        return true;
    }

    let lowered = content.to_ascii_lowercase();
    terms.iter().any(|term| lowered.contains(term))
}

fn comparable_datetime_prefix(file_name: &str) -> Option<String> {
    if let Some(timestamp) = extract_timestamp_prefix(file_name) {
        return Some(timestamp.to_string());
    }

    extract_date_prefix(file_name).map(|date| format!("{date}-0000"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_since_for_date_and_datetime() {
        let file = "2026-04-04-1525-entry.md";
        assert!(matches_since(file, "2026-04-04"));
        assert!(matches_since(file, "2026-04-04-1400"));
        assert!(!matches_since(file, "2026-04-05"));
        assert!(!matches_since(file, "2026-04-04-1600"));
    }

    #[test]
    fn matches_between_with_mixed_formats() {
        let file = "2026-04-04-1525-entry.md";
        assert!(matches_between(file, "2026-04-04", "2026-04-04-1600"));
        assert!(matches_between(file, "2026-04-04-1500", "2026-04-04"));
        assert!(!matches_between(file, "2026-04-04-1600", "2026-04-05"));
    }

    #[test]
    fn content_filter_is_case_insensitive_and_any_match() {
        let terms = vec!["auth".to_string(), "deploy".to_string()];
        assert!(matches_content_filter("We need DEPLOY notes", &terms));
        assert!(matches_content_filter("Auth token issue", &terms));
        assert!(!matches_content_filter("No related keywords", &terms));
    }
}
