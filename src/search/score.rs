use std::collections::{HashMap, HashSet};

pub fn score_document(
    query_terms: &[String],
    normalized_query_phrase: &str,
    doc_terms: &[String],
) -> f64 {
    if query_terms.is_empty() || doc_terms.is_empty() {
        return 0.0;
    }

    let mut positions_by_term: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, term) in doc_terms.iter().enumerate() {
        positions_by_term.entry(term.as_str()).or_default().push(i);
    }

    let mut matched_terms = 0;
    let mut total_frequency = 0;

    for term in query_terms {
        if let Some(positions) = positions_by_term.get(term.as_str()) {
            matched_terms += 1;
            total_frequency += positions.len();
        }
    }

    if matched_terms == 0 {
        return 0.0;
    }

    let mut score = 0.0;

    score += matched_terms as f64 * 10.0;
    score += total_frequency as f64 * 2.0;

    if matched_terms == query_terms.len() {
        score += 25.0;
    }

    let normalized_doc_phrase = doc_terms.join(" ");
    if normalized_doc_phrase.contains(normalized_query_phrase) {
        score += 75.0;
    }

    if matched_terms >= 2
        && let Some(span) = smallest_matching_span(query_terms, doc_terms)
    {
        score += 50.0 / (1.0 + span as f64);
    }

    if matched_terms >= 2 && appears_in_order(query_terms, doc_terms) {
        score += 10.0;
    }

    score
}

fn smallest_matching_span(query_terms: &[String], doc_terms: &[String]) -> Option<usize> {
    let query_set: HashSet<&str> = query_terms.iter().map(|s| s.as_str()).collect();
    let required_count = query_set.len();

    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut matched_count = 0;
    let mut left = 0;
    let mut best_span: Option<usize> = None;

    for right in 0..doc_terms.len() {
        let term = doc_terms[right].as_str();

        if query_set.contains(term) {
            let count = counts.entry(term).or_insert(0);
            if *count == 0 {
                matched_count += 1;
            }
            *count += 1;
        }

        while matched_count == required_count && left <= right {
            let span = right - left;
            best_span = Some(best_span.map_or(span, |best| best.min(span)));

            let left_term = doc_terms[left].as_str();
            if query_set.contains(left_term)
                && let Some(count) = counts.get_mut(left_term)
            {
                *count -= 1;
                if *count == 0 {
                    matched_count -= 1;
                }
            }

            left += 1;
        }
    }

    best_span
}

fn appears_in_order(query_terms: &[String], doc_terms: &[String]) -> bool {
    let mut query_index = 0;

    for term in doc_terms {
        if query_index < query_terms.len() && term == &query_terms[query_index] {
            query_index += 1;
        }
    }

    query_index == query_terms.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terms(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    #[test]
    fn no_match_returns_zero() {
        let query = terms(&["missing"]);
        let doc = terms(&["hello", "world"]);
        assert_eq!(score_document(&query, "missing", &doc), 0.0);
    }

    #[test]
    fn single_term_match() {
        let query = terms(&["hello"]);
        let doc = terms(&["hello", "world"]);
        let score = score_document(&query, "hello", &doc);
        // matched_terms(1) * 10 + frequency(1) * 2 + all_terms(25) + exact_phrase(75)
        assert_eq!(score, 112.0);
    }

    #[test]
    fn multi_term_match_with_proximity() {
        let query = terms(&["note", "student"]);
        let doc = terms(&["added", "note", "owner", "send", "student"]);
        let score = score_document(&query, "note student", &doc);

        // matched_terms(2)*10 + freq(2)*2 + all_terms(25) + proximity(50/(1+3)) + order(10)
        // No exact phrase match since "note student" not adjacent
        let expected = 20.0 + 4.0 + 25.0 + (50.0 / 4.0) + 10.0;
        assert!((score - expected).abs() < 0.01);
    }

    #[test]
    fn exact_phrase_bonus_applied() {
        let query = terms(&["note", "student"]);
        let doc = terms(&["note", "student", "added"]);
        let score = score_document(&query, "note student", &doc);

        // Has exact phrase "note student" in doc
        assert!(score > 100.0);
    }

    #[test]
    fn order_bonus_when_terms_in_order() {
        let query = terms(&["alpha", "beta"]);
        let doc = terms(&["alpha", "gamma", "beta"]);
        let score_ordered = score_document(&query, "alpha beta", &doc);

        let query_rev = terms(&["beta", "alpha"]);
        let doc_same = terms(&["alpha", "gamma", "beta"]);
        let score_reversed = score_document(&query_rev, "beta alpha", &doc_same);

        // ordered should have order bonus, reversed should not
        assert!(score_ordered > score_reversed);
    }

    #[test]
    fn proximity_span_calculation() {
        let query = terms(&["a", "b"]);
        let doc = terms(&["a", "b"]);
        assert_eq!(smallest_matching_span(&query, &doc), Some(1));

        let doc_far = terms(&["a", "x", "x", "x", "b"]);
        assert_eq!(smallest_matching_span(&query, &doc_far), Some(4));
    }

    #[test]
    fn appears_in_order_basic() {
        let query = terms(&["a", "b"]);
        let doc = terms(&["a", "x", "b"]);
        assert!(appears_in_order(&query, &doc));

        let doc_rev = terms(&["b", "x", "a"]);
        assert!(!appears_in_order(&query, &doc_rev));
    }

    #[test]
    fn empty_query_returns_zero() {
        let doc = terms(&["hello", "world"]);
        assert_eq!(score_document(&[], "", &doc), 0.0);
    }

    #[test]
    fn empty_doc_returns_zero() {
        let query = terms(&["hello"]);
        assert_eq!(score_document(&query, "hello", &[]), 0.0);
    }

    #[test]
    fn single_query_term_skips_proximity_and_order() {
        let query = terms(&["hello"]);
        let doc = terms(&["hello", "world", "hello"]);
        let score = score_document(&query, "hello", &doc);
        // matched_terms(1)*10 + freq(2)*2 + all_terms(25) + exact_phrase(75)
        // No proximity or order bonuses for single term
        assert_eq!(score, 114.0);
    }
}
