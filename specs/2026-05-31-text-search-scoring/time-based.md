Add timestamp as a **separate recency score**, then blend it with the text score. Do not let recency overpower relevance. The best result should be “highly relevant and recent,” not merely “new.”

A good scoring shape is:

```text
final_score =
  text_score * 0.85
+ recency_score * 0.15
```

But only apply full recency weight when there is at least some meaningful text match. Otherwise, a brand-new unrelated journal entry can float upward.

I would use a decaying score:

```text
recency_score = 100 * e^(-age_days / half_life_days)
```

For project journal entries, a reasonable half-life might be 30, 60, or 90 days depending on how fast the project changes.

For example, with a 60-day half-life:

```text
Today:        100
30 days old:   61
60 days old:   37
120 days old:  14
240 days old:   2
```

That gives recent entries a boost without deleting older entries from relevance.

In Rust, I would extend the model like this:

```rust
#[derive(Debug)]
pub struct JournalEntry {
    pub id: String,
    pub timestamp_unix: i64,
    pub summary: String,
    pub summary_terms: Vec<String>,
}
```

Then score like this:

```rust
fn final_score(text_score: f64, timestamp_unix: i64, now_unix: i64) -> f64 {
    if text_score <= 0.0 {
        return 0.0;
    }

    let recency = recency_score(timestamp_unix, now_unix, 60.0);

    let text_weight = 0.85;
    let recency_weight = 0.15;

    text_score * text_weight + recency * recency_weight
}

fn recency_score(timestamp_unix: i64, now_unix: i64, half_life_days: f64) -> f64 {
    let age_seconds = (now_unix - timestamp_unix).max(0) as f64;
    let age_days = age_seconds / 86_400.0;

    100.0 * (-age_days / half_life_days).exp()
}
```

That uses exponential decay. It is simple, fast, and works well for this kind of “newer is somewhat better” ranking.

You may want one more rule: if the text match is excellent, preserve it even if old. For that, use a smaller recency weight when the text score is already high:

```rust
fn final_score(text_score: f64, timestamp_unix: i64, now_unix: i64) -> f64 {
    if text_score <= 0.0 {
        return 0.0;
    }

    let recency = recency_score(timestamp_unix, now_unix, 60.0);

    let recency_weight = if text_score >= 80.0 {
        0.08
    } else if text_score >= 40.0 {
        0.15
    } else {
        0.25
    };

    let text_weight = 1.0 - recency_weight;

    text_score * text_weight + recency * recency_weight
}
```

This gives weaker matches more help from recency, while strong lexical matches stay dominant.

For your journal use case, I would probably use this version.

The full scoring object could look like this:

```rust
#[derive(Debug)]
pub struct ScoreBreakdown {
    pub final_score: f64,
    pub text_score: f64,
    pub recency_score: f64,
    pub matched_terms: usize,
}
```

Then your search result becomes easier to debug:

```rust
#[derive(Debug)]
pub struct SearchResult<'a> {
    pub id: &'a str,
    pub final_score: f64,
    pub text_score: f64,
    pub recency_score: f64,
    pub summary: &'a str,
}
```

A useful implementation pattern:

```rust
fn score_entry(
    query_terms: &[String],
    normalized_query_phrase: &str,
    entry: &JournalEntry,
    now_unix: i64,
) -> ScoreBreakdown {
    let text_score = score_document(
        query_terms,
        normalized_query_phrase,
        &entry.summary_terms,
    );

    if text_score <= 0.0 {
        return ScoreBreakdown {
            final_score: 0.0,
            text_score,
            recency_score: 0.0,
            matched_terms: 0,
        };
    }

    let recency = recency_score(entry.timestamp_unix, now_unix, 60.0);

    let recency_weight = if text_score >= 80.0 {
        0.08
    } else if text_score >= 40.0 {
        0.15
    } else {
        0.25
    };

    let final_score = text_score * (1.0 - recency_weight) + recency * recency_weight;

    ScoreBreakdown {
        final_score,
        text_score,
        recency_score: recency,
        matched_terms: count_matched_terms(query_terms, &entry.summary_terms),
    }
}

fn count_matched_terms(query_terms: &[String], doc_terms: &[String]) -> usize {
    use std::collections::HashSet;

    let doc_set: HashSet<&str> = doc_terms.iter().map(|s| s.as_str()).collect();

    query_terms
        .iter()
        .filter(|term| doc_set.contains(term.as_str()))
        .count()
}
```

I would also add one timestamp-specific behavior: when the user searches using date words, let those words affect filtering or ranking.

For example:

```text
recent notes for students
latest notes for students
old notes for students
last week notes for students
```

You can handle this with simple keyword detection before scoring:

```rust
#[derive(Debug, Clone, Copy)]
pub enum TimeBias {
    Normal,
    Recent,
    Old,
}

fn detect_time_bias(query: &str) -> TimeBias {
    let q = query.to_lowercase();

    if q.contains("recent") || q.contains("latest") || q.contains("newest") {
        TimeBias::Recent
    } else if q.contains("old") || q.contains("older") || q.contains("earliest") {
        TimeBias::Old
    } else {
        TimeBias::Normal
    }
}
```

Then adjust recency:

```rust
fn recency_weight_for(text_score: f64, time_bias: TimeBias) -> f64 {
    match time_bias {
        TimeBias::Recent => {
            if text_score >= 80.0 { 0.18 } else if text_score >= 40.0 { 0.30 } else { 0.40 }
        }
        TimeBias::Old => {
            0.0
        }
        TimeBias::Normal => {
            if text_score >= 80.0 { 0.08 } else if text_score >= 40.0 { 0.15 } else { 0.25 }
        }
    }
}
```

For `Old`, you could invert the date score instead of ignoring it:

```rust
fn oldness_score(timestamp_unix: i64, now_unix: i64, half_life_days: f64) -> f64 {
    100.0 - recency_score(timestamp_unix, now_unix, half_life_days)
}
```

But I would avoid that unless “oldest” is explicitly meaningful.

Recommended final shape:

```text
final_score =
  lexical relevance
+ phrase match
+ proximity
+ order
+ bounded recency boost
```

More concretely:

```rust
let final_score = match text_score {
    s if s >= 80.0 => s * 0.92 + recency * 0.08,
    s if s >= 40.0 => s * 0.85 + recency * 0.15,
    s              => s * 0.75 + recency * 0.25,
};
```

That is simple, explainable, and fast.
