mod score;
pub mod time_bias;
pub mod tokenize;
pub mod recency;

use std::fs;
use std::path::Path;

use crate::section::extract_section;
use recency::{filename_to_unix_timestamp, final_score, recency_score};
use score::score_document;
use time_bias::detect_time_bias;
use tokenize::tokenize;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub filename: String,
    pub final_score: f64,
    pub text_score: f64,
    pub recency_score: f64,
    pub summary: String,
}

pub fn search_entries(
    filenames: &[String],
    journal_dir: &Path,
    query: &str,
    now_unix: i64,
) -> Vec<SearchResult> {
    let (bias, cleaned_query) = detect_time_bias(query);
    let query_terms = tokenize(&cleaned_query);

    if query_terms.is_empty() {
        return vec![];
    }

    let normalized_query_phrase = query_terms.join(" ");

    let mut results: Vec<SearchResult> = Vec::new();

    for filename in filenames {
        let path = journal_dir.join(filename);
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let summary = match extract_section(&content, "Summary") {
            Some(s) if !s.is_empty() => s,
            _ => continue,
        };

        let doc_terms = tokenize(&summary);
        if doc_terms.is_empty() {
            continue;
        }

        let text = score_document(&query_terms, &normalized_query_phrase, &doc_terms);
        if text <= 0.0 {
            continue;
        }

        let timestamp = filename_to_unix_timestamp(filename).unwrap_or(0);
        let recency = recency_score(timestamp, now_unix, 60.0);
        let combined = final_score(text, recency, bias);

        if combined > 0.0 {
            results.push(SearchResult {
                filename: filename.clone(),
                final_score: combined,
                text_score: text,
                recency_score: recency,
                summary,
            });
        }
    }

    results.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_dir(prefix: &str) -> std::path::PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("journal-search-tests-{prefix}-{unique}"));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn ranks_multi_term_match_higher_than_single() {
        let dir = create_test_dir("rank");

        fs::write(
            dir.join("2026-05-04-1200-notes.md"),
            "## Summary\n\nAdded notes that the owner can send to students or instructors.\n",
        )
        .unwrap();
        fs::write(
            dir.join("2026-05-03-1000-login.md"),
            "## Summary\n\nStudents can now log in to the platform.\n",
        )
        .unwrap();

        let files = vec![
            "2026-05-04-1200-notes.md".to_string(),
            "2026-05-03-1000-login.md".to_string(),
        ];

        let now = 1_780_000_000; // roughly mid-2026
        let results = search_entries(&files, &dir, "notes for students", now);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].filename, "2026-05-04-1200-notes.md");
        assert!(results[0].final_score > results[1].final_score);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn time_bias_recent_boosts_newer_entry() {
        let dir = create_test_dir("bias");

        fs::write(
            dir.join("2026-01-01-0900-old-auth.md"),
            "## Summary\n\nSetup authentication flow for users.\n",
        )
        .unwrap();
        fs::write(
            dir.join("2026-05-30-1000-new-auth.md"),
            "## Summary\n\nUpdated authentication flow for users.\n",
        )
        .unwrap();

        let files = vec![
            "2026-01-01-0900-old-auth.md".to_string(),
            "2026-05-30-1000-new-auth.md".to_string(),
        ];

        let now = 1_780_000_000;
        let results_normal = search_entries(&files, &dir, "authentication flow", now);
        let results_recent = search_entries(&files, &dir, "recent authentication flow", now);

        // With "recent" bias, the newer entry should rank higher
        assert_eq!(results_recent[0].filename, "2026-05-30-1000-new-auth.md");
        // Score difference should be bigger with recent bias
        let normal_gap =
            results_normal[0].final_score - results_normal[1].final_score;
        let recent_gap =
            results_recent[0].final_score - results_recent[1].final_score;
        assert!(recent_gap >= normal_gap);

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn entry_without_summary_skipped() {
        let dir = create_test_dir("no-summary");

        fs::write(
            dir.join("2026-05-01-1000-nosummary.md"),
            "## Context\n\nJust context, no summary here.\n",
        )
        .unwrap();

        let files = vec!["2026-05-01-1000-nosummary.md".to_string()];
        let results = search_entries(&files, &dir, "context", 1_780_000_000);
        assert!(results.is_empty());

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn empty_query_after_stopwords_returns_empty() {
        let dir = create_test_dir("stopwords");

        fs::write(
            dir.join("2026-05-01-1000-test.md"),
            "## Summary\n\nSome content here.\n",
        )
        .unwrap();

        let files = vec!["2026-05-01-1000-test.md".to_string()];
        let results = search_entries(&files, &dir, "the and or is for", 1_780_000_000);
        assert!(results.is_empty());

        fs::remove_dir_all(dir).unwrap();
    }
}
