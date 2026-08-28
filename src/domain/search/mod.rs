mod recency;
mod score;
mod time_bias;
mod tokenize;

use std::cmp::Ordering;

use super::{EntryMetadata, SearchQuery};
use recency::{entry_to_unix_timestamp, final_score, recency_score};
use score::score_document;
use time_bias::detect_time_bias;
use tokenize::tokenize;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchMatch {
    pub entry: EntryMetadata,
    pub final_score: f64,
    pub text_score: f64,
    pub recency_score: f64,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCandidate {
    pub entry: EntryMetadata,
    pub summary: String,
}

pub fn rank(
    candidates: &[SearchCandidate],
    query: &SearchQuery,
    now_unix: i64,
) -> Vec<SearchMatch> {
    let (bias, cleaned_query) = detect_time_bias(query.as_str());
    let query_terms = tokenize(&cleaned_query);
    if query_terms.is_empty() {
        return Vec::new();
    }

    let normalized_query_phrase = query_terms.join(" ");
    let mut results = candidates
        .iter()
        .filter_map(|candidate| {
            let document_terms = tokenize(&candidate.summary);
            if document_terms.is_empty() {
                return None;
            }

            let text_score =
                score_document(&query_terms, &normalized_query_phrase, &document_terms);
            if text_score <= 0.0 {
                return None;
            }

            let timestamp = entry_to_unix_timestamp(&candidate.entry.name).unwrap_or(0);
            let recency = recency_score(timestamp, now_unix, 60.0);
            let combined = final_score(text_score, recency, bias);
            (combined > 0.0).then(|| SearchMatch {
                entry: candidate.entry.clone(),
                final_score: combined,
                text_score,
                recency_score: recency,
                summary: candidate.summary.clone(),
            })
        })
        .collect::<Vec<_>>();

    results.sort_by(|left, right| {
        right
            .final_score
            .partial_cmp(&left.final_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.entry.name.cmp(&left.entry.name))
    });
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EntryName;

    fn candidate(name: &str, summary: &str) -> SearchCandidate {
        SearchCandidate {
            entry: EntryMetadata::new(EntryName::parse(name).unwrap()),
            summary: summary.to_string(),
        }
    }

    #[test]
    fn ranks_multi_term_match_higher_than_single_term() {
        let candidates = vec![
            candidate(
                "2026-05-04-1200-notes.md",
                "Added notes that the owner can send to students or instructors.",
            ),
            candidate(
                "2026-05-03-1000-login.md",
                "Students can now log in to the platform.",
            ),
        ];

        let results = rank(
            &candidates,
            &SearchQuery::new("notes for students").unwrap(),
            1_780_000_000,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].entry.name.as_str(), "2026-05-04-1200-notes.md");
        assert!(results[0].final_score > results[1].final_score);
    }

    #[test]
    fn time_bias_recent_boosts_newer_entry() {
        let candidates = vec![
            candidate(
                "2026-01-01-0900-old-auth.md",
                "Setup authentication flow for users.",
            ),
            candidate(
                "2026-05-30-1000-new-auth.md",
                "Updated authentication flow for users.",
            ),
        ];
        let results = rank(
            &candidates,
            &SearchQuery::new("recent authentication flow").unwrap(),
            1_780_000_000,
        );
        assert_eq!(
            results[0].entry.name.as_str(),
            "2026-05-30-1000-new-auth.md"
        );
    }

    #[test]
    fn stop_word_only_query_returns_empty() {
        let candidates = vec![candidate("2026-05-01-1000-test.md", "Some content here.")];
        assert!(
            rank(
                &candidates,
                &SearchQuery::new("the and or is for").unwrap(),
                1_780_000_000,
            )
            .is_empty()
        );
    }

    #[test]
    fn equal_scores_fall_back_to_newest_filename() {
        let candidates = vec![
            candidate("2026-05-01-1000-first.md", "Deploy service."),
            candidate("2026-05-02-1000-second.md", "Deploy service."),
        ];
        let results = rank(
            &candidates,
            &SearchQuery::new("old deploy").unwrap(),
            1_780_000_000,
        );
        assert_eq!(results[0].entry.name.as_str(), "2026-05-02-1000-second.md");
    }
}
