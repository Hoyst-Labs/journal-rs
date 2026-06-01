# Implementation Plan

- [ ] 1. Create the `search` submodule structure
  - Create `src/search/mod.rs`, `src/search/tokenize.rs`, `src/search/score.rs`, `src/search/recency.rs`, `src/search/time_bias.rs`
  - Add `pub mod search;` to `src/lib.rs`
  - Each file exports its public API with empty/placeholder implementations that compile
  - Verify `cargo check` passes
  - Refs: Req 9.2 (no external deps), Design §Module Layout

- [ ] 2. Implement tokenizer (`src/search/tokenize.rs`)
  - Define `STOP_WORDS` static array (19 words from Req 2.2)
  - Implement `pub fn tokenize(text: &str) -> Vec<String>`: lowercase → split on non-alphanumeric → remove empties → `light_normalize` → filter stop words
  - Implement `pub fn light_normalize(word: &str) -> String`: `ies` → `y` (len > 4), trailing `s` removal (len > 3)
  - Add unit tests: basic tokenization, stop word removal, plural normalization, all-stop-words input → empty vec, empty string → empty vec, mixed case
  - Refs: Req 2.1, 2.2, 2.3, 2.4

- [ ] 3. Implement time bias detection (`src/search/time_bias.rs`)
  - Define `pub enum TimeBias { Normal, Recent, Old }`
  - Implement `pub fn detect_time_bias(query: &str) -> (TimeBias, String)`: scan for bias keywords, strip them from returned query
  - Bias keywords: `recent`/`latest`/`newest` → Recent; `old`/`older`/`earliest` → Old
  - Add unit tests: each keyword triggers correct bias, keyword stripped from output, no-keyword → Normal, multiple keywords, keywords mid-sentence
  - Refs: Req 6.1, 6.2, 6.3, 6.4

- [ ] 4. Implement text scorer (`src/search/score.rs`)
  - Implement `pub fn score_document(query_terms: &[String], normalized_query_phrase: &str, doc_terms: &[String]) -> f64`
  - Implement scoring signals: matched_terms × 10, total_frequency × 2, all-terms bonus (25), exact phrase bonus (75), proximity bonus (50 / (1 + span)), order bonus (10)
  - Implement `fn smallest_matching_span(query_terms: &[String], doc_terms: &[String]) -> Option<usize>` using sliding window
  - Implement `fn appears_in_order(query_terms: &[String], doc_terms: &[String]) -> bool`
  - Add unit tests: single-term match, multi-term match, exact phrase, proximity scoring with known span values, order detection, no-match → 0.0, single query term skips proximity/order
  - Refs: Req 4.1, 4.2, 4.3

- [ ] 5. Implement recency scorer (`src/search/recency.rs`)
  - Implement `pub fn recency_score(timestamp_unix: i64, now_unix: i64, half_life_days: f64) -> f64`: exponential decay `100 × e^(-age_days / half_life)`
  - Implement `pub fn final_score(text_score: f64, recency: f64, time_bias: TimeBias) -> f64`: adaptive weight tiers per Design §Recency Scorer table
  - Add unit tests: known decay values (today = 100, 60 days = ~37, 120 days = ~14), text_score 0 → final 0, weight tiers (≥80, ≥40, <40), TimeBias::Recent amplifies, TimeBias::Old zeroes recency
  - Refs: Req 5.1, 5.2, 5.3

- [ ] 6. Add filename-to-timestamp helper
  - Add `pub fn filename_to_unix_timestamp(filename: &str) -> Option<i64>` in `src/search/recency.rs` (or `src/journal.rs` if better fit)
  - Parse `YYYY-MM-DD-HHmm` prefix from filename, convert to naive UTC unix timestamp using manual calculation (no chrono crate)
  - Handle date-only filenames (`YYYY-MM-DD`) by assuming `0000` time
  - Add unit tests: valid datetime filename, date-only filename, invalid prefix → None
  - Refs: Req 5.4, Design §Timestamp Extraction

- [ ] 7. Implement search orchestrator (`src/search/mod.rs`)
  - Define `pub struct SearchResult { filename, final_score, text_score, recency_score, summary }`
  - Implement `pub fn search_entries(entries: &[(String, PathBuf)], query: &str, now_unix: i64) -> Vec<SearchResult>`
  - Flow: detect time bias → tokenize cleaned query → early return if empty → for each entry: read file, extract summary, tokenize, score, compute recency, blend → filter > 0 → sort desc
  - Reuse `section::extract_section` for summary extraction
  - Add unit tests with in-memory test entries covering: multi-entry ranking, time bias effect, entry with no summary skipped, unreadable file skipped
  - Refs: Req 3.1, 3.2, 4, 5, 6, 9.1, 9.3

- [ ] 8. Extend CLI parsing for `--search`
  - Add `search_query: Option<String>` to `QueryParams` in `src/model.rs`
  - Add `Search` variant to `DisplayMode` enum in `src/model.rs`
  - Update `src/cli.rs` to parse `--search <value>`: consume next arg as query string, set `DisplayMode::Search`
  - Add validation: `--search` + `--filter` → error (Req 11.2), `--search` without value → error (Req 11.1)
  - Add `search <query>` as positional command alias (like `summary` and `full`)
  - Update `src/help.rs` with `--search` documentation and examples
  - Add unit tests: parse `--search "query"`, mutual exclusion errors, missing value error
  - Refs: Req 1.3, 1.4, 11.1, 11.2

- [ ] 9. Wire search into the execution pipeline (`src/lib.rs`)
  - In `execute_from`, after `apply_filters`, add branch: if `DisplayMode::Search` → call `search_entries` with filtered file list and current unix time
  - Apply `--latest` truncation to search results (default to 10 if `latest` not set, per Req 7.1)
  - Route to `format_search_results` for output
  - Handle the case where search returns empty → "No matching entries found." message
  - Refs: Req 1.1, 1.2, 1.5, 1.6, 7.1, 7.2, 7.3, 10.1, 10.2

- [ ] 10. Implement search result rendering (`src/render.rs`)
  - Add `pub fn format_search_results(results: &[SearchResult]) -> String`
  - Format: `[score] filename\n  summary_preview\n` with score to 2 decimal places, summary truncated at 120 chars with `...`
  - Handle `--full` + `--search`: show full content sorted by score instead of summary preview
  - Handle `--type` + `--search`: extract named section from matched entries sorted by score
  - Add unit tests: formatting, truncation at 120 chars, empty results
  - Refs: Req 8.1, 8.2, 8.3

- [ ] 11. Integration tests
  - Add integration tests in `src/lib.rs` (following existing pattern) covering:
    - `--search "notes for students"` → ranked results
    - `--search` + `--since` → date filter then score
    - `--search` + `--between` → range filter then score
    - `--search` + `--filter` → error
    - `--search` + `--latest 3` → capped at 3
    - `--search "the and or"` (all stop words) → empty results, exit 0
    - `--search "recent auth"` → time bias detected, "recent" stripped
    - `--search` with no summary entries → no results
  - Refs: Req 1–11 (full coverage)
