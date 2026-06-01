This defines a simple in-memory **phrase-aware lexical scorer**:

./time-based.md defines a scoring model that includes a time-based component.

1. Extract only the `## Summary` or `# Summary` sections.
2. Normalize to lowercase.
3. Tokenize into words.
4. Stem very lightly, or at least singularize simple plurals.
5. Score:

   * exact phrase match: large boost
   * individual term hits: medium boost
   * number of matched query terms: medium boost
   * proximity/nearness: large boost when terms appear close together
   * order match: small boost when terms appear in query order

For your example search:

```text
notes for students
```

I would intentionally treat `for` as a stop word. The meaningful terms are:

```text
notes students
```

Then:

```text
Added notes that the owner can send to students or instructors.
```

matches both `notes` and `students`, fairly close together.

```text
Students can now log in
```

only matches `students`.

So example 1 naturally scores much higher.

A practical scoring model:

```text
score =
  exact_phrase_bonus
+ matched_terms_count * 10
+ total_term_frequency * 2
+ proximity_bonus
+ order_bonus
```

Where proximity bonus might be:

```text
proximity_bonus = 50 / (1 + span)
```

`span` is the number of tokens between the first and last matched query term in the best window.

For example:

```text
notes that the owner can send to students
```

The matched words are `notes` and `students`. If those positions are 1 and 8, span is 7. The proximity bonus is smaller, but still meaningful.

Here is a compact Rust implementation approach.

```rust
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct SearchResult<'a> {
    pub id: &'a str,
    pub score: f64,
    pub summary: String,
}

static STOP_WORDS: &[&str] = &[
    "a", "an", "the", "to", "for", "of", "in", "on", "and", "or", "is", "are",
    "was", "were", "can", "now", "with", "that", "this",
];

pub fn search_markdown<'a>(
    docs: &'a [(&'a str, &'a str)],
    query: &str,
) -> Vec<SearchResult<'a>> {
    let query_terms = tokenize(query);

    if query_terms.is_empty() {
        return vec![];
    }

    let normalized_query_phrase = query_terms.join(" ");

    let mut results = Vec::new();

    for (id, markdown) in docs {
        let Some(summary) = extract_summary(markdown) else {
            continue;
        };

        let summary_terms = tokenize(&summary);
        if summary_terms.is_empty() {
            continue;
        }

        let score = score_document(
            &query_terms,
            &normalized_query_phrase,
            &summary_terms,
        );

        if score > 0.0 {
            results.push(SearchResult {
                id,
                score,
                summary,
            });
        }
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

fn extract_summary(markdown: &str) -> Option<String> {
    let mut in_summary = false;
    let mut lines = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") {
            if in_summary {
                break;
            }

            let heading = trimmed.trim_start_matches("##").trim();

            if heading.eq_ignore_ascii_case("summary") {
                in_summary = true;
            }

            continue;
        }

        if in_summary {
            lines.push(line);
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n").trim().to_string())
    }
}

fn tokenize(text: &str) -> Vec<String> {
    let stop_words: HashSet<&str> = STOP_WORDS.iter().copied().collect();

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(light_normalize)
        .filter(|s| !s.is_empty())
        .filter(|s| !stop_words.contains(s.as_str()))
        .collect()
}

fn light_normalize(word: &str) -> String {
    // Cheap plural handling:
    // students -> student
    // notes -> note
    // entries -> entry
    //
    // This is not a real stemmer. It is intentionally simple.
    if word.len() > 4 && word.ends_with("ies") {
        format!("{}y", &word[..word.len() - 3])
    } else if word.len() > 3 && word.ends_with('s') {
        word[..word.len() - 1].to_string()
    } else {
        word.to_string()
    }
}

fn score_document(
    query_terms: &[String],
    normalized_query_phrase: &str,
    doc_terms: &[String],
) -> f64 {
    let mut positions_by_term: HashMap<&str, Vec<usize>> = HashMap::new();

    for (i, term) in doc_terms.iter().enumerate() {
        positions_by_term
            .entry(term.as_str())
            .or_default()
            .push(i);
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

    // Reward matching distinct query terms.
    score += matched_terms as f64 * 10.0;

    // Reward repeated useful hits, but lightly.
    score += total_frequency as f64 * 2.0;

    // Strongly reward matching all query terms.
    if matched_terms == query_terms.len() {
        score += 25.0;
    }

    // Exact normalized phrase match.
    let normalized_doc_phrase = doc_terms.join(" ");
    if normalized_doc_phrase.contains(normalized_query_phrase) {
        score += 75.0;
    }

    // Proximity bonus.
    if matched_terms >= 2 {
        if let Some(span) = smallest_matching_span(query_terms, doc_terms) {
            score += 50.0 / (1.0 + span as f64);
        }
    }

    // Order bonus.
    if appears_in_order(query_terms, doc_terms) {
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

            if query_set.contains(left_term) {
                if let Some(count) = counts.get_mut(left_term) {
                    *count -= 1;
                    if *count == 0 {
                        matched_count -= 1;
                    }
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
```

Usage:

```rust
fn main() {
    let docs = vec![
        (
            "example1",
            r#"
## Summary
Added notes that the owner can send to students or instructors.
"#,
        ),
        (
            "example2",
            r#"
## Summary
Students can now log in
"#,
        ),
    ];

    let results = search_markdown(&docs, "notes for students");

    for result in results {
        println!("{}: {:.2} - {}", result.id, result.score, result.summary);
    }
}
```

Expected behavior:

```text
example1: much higher score
example2: lower score
```

The important part is that this is not really “search engine” territory yet. It is just linear scanning with a smart-enough scorer. For journal-sized markdown files, this will be very fast in Rust.

A slightly better version would pre-tokenize each markdown file after loading it:

```rust
struct JournalEntry {
    id: String,
    summary: String,
    summary_terms: Vec<String>,
}
```

That is not an index in the full-text-search sense. It is just parsed working memory. Then every search only tokenizes the query and scores against already-tokenized summaries.

For raw speed, use this flow:

```text
load files
extract ## Summary
normalize/tokenize once
store Vec<JournalEntry>
on search:
  tokenize query
  score every JournalEntry
  sort by score
```

I would not start with Tantivy, SQLite FTS, Meilisearch, or a vector DB for this. Your use case is small, structured, and explainable. A direct scorer gives you control, is easy to debug, and will probably be fast enough until you are dealing with hundreds of thousands of entries.
