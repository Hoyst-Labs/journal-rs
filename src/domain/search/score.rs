use std::collections::{HashMap, HashSet};

pub fn score_document(
    query_terms: &[String],
    normalized_query_phrase: &str,
    document_terms: &[String],
) -> f64 {
    if query_terms.is_empty() || document_terms.is_empty() {
        return 0.0;
    }

    let mut positions_by_term: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, term) in document_terms.iter().enumerate() {
        positions_by_term
            .entry(term.as_str())
            .or_default()
            .push(index);
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

    let mut score = matched_terms as f64 * 10.0 + total_frequency as f64 * 2.0;
    if matched_terms == query_terms.len() {
        score += 25.0;
    }
    if document_terms.join(" ").contains(normalized_query_phrase) {
        score += 75.0;
    }
    if matched_terms >= 2
        && let Some(span) = smallest_matching_span(query_terms, document_terms)
    {
        score += 50.0 / (1.0 + span as f64);
    }
    if matched_terms >= 2 && appears_in_order(query_terms, document_terms) {
        score += 10.0;
    }
    score
}

fn smallest_matching_span(query_terms: &[String], document_terms: &[String]) -> Option<usize> {
    let query_set: HashSet<&str> = query_terms.iter().map(String::as_str).collect();
    let required_count = query_set.len();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut matched_count = 0;
    let mut left = 0;
    let mut best_span: Option<usize> = None;

    for right in 0..document_terms.len() {
        let term = document_terms[right].as_str();
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
            let left_term = document_terms[left].as_str();
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

fn appears_in_order(query_terms: &[String], document_terms: &[String]) -> bool {
    let mut query_index = 0;
    for term in document_terms {
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
        words.iter().map(|word| (*word).to_string()).collect()
    }

    #[test]
    fn empty_or_missing_terms_score_zero() {
        assert_eq!(score_document(&[], "", &terms(&["hello"])), 0.0);
        assert_eq!(
            score_document(&terms(&["missing"]), "missing", &terms(&["hello"])),
            0.0
        );
    }

    #[test]
    fn preserves_frequency_phrase_proximity_and_order_scores() {
        assert_eq!(
            score_document(&terms(&["hello"]), "hello", &terms(&["hello", "world"])),
            112.0
        );
        let score = score_document(
            &terms(&["note", "student"]),
            "note student",
            &terms(&["added", "note", "owner", "send", "student"]),
        );
        let expected = 20.0 + 4.0 + 25.0 + (50.0 / 4.0) + 10.0;
        assert!((score - expected).abs() < 0.01);
    }

    #[test]
    fn helper_rules_detect_span_and_order() {
        assert_eq!(
            smallest_matching_span(&terms(&["a", "b"]), &terms(&["a", "x", "b"])),
            Some(2)
        );
        assert!(appears_in_order(
            &terms(&["a", "b"]),
            &terms(&["a", "x", "b"])
        ));
        assert!(!appears_in_order(
            &terms(&["a", "b"]),
            &terms(&["b", "x", "a"])
        ));
    }
}
