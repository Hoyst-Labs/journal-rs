# Requirements Document

## Introduction

The journal CLI currently supports `--filter`, which performs simple case-insensitive substring matching (OR across pipe-delimited terms). This gives boolean "match / no match" results with no ranking.

We want a **scored keyword search** command that ranks journal entries by lexical relevance and recency. The scorer operates on extracted `## Summary` sections using phrase-aware text matching — exact phrase hits, individual term frequency, proximity, and term order — blended with a time-based recency boost derived from the filename timestamp.

This is not a full-text search index. It is a linear scan with a smart scorer, designed for the small-to-medium journal corpus (hundreds to low thousands of entries) where explainability and zero dependencies matter more than throughput.

## Requirements

### 1. Search Command Interface

**User Story:** As a CLI user, I want a `--search <query>` flag so that I can find the most relevant journal entries by keyword rather than exact substring.

#### Acceptance Criteria

1. WHEN the user passes `--search "notes for students"` THEN the system SHALL tokenize the query, score all entries, and return results sorted by final score descending.
2. WHEN `--search` is combined with date-range flags (`--since`, `--between`) THEN the system SHALL apply date filters first, then score the remaining entries.
3. WHEN `--search` is passed with no query string or an empty string THEN the system SHALL return an error message.
4. IF `--search` is combined with `--filter` THEN the system SHALL return an error (mutual exclusion — `--filter` is boolean, `--search` is scored).
5. WHEN results are displayed THEN the system SHALL show the entry filename, score, and summary text for each match.
6. IF no entries score above zero THEN the system SHALL print a "no matches" message and exit 0.

### 2. Text Tokenization and Normalization

**User Story:** As a search user, I want the system to handle plural forms, stop words, and case differences so that my queries match naturally without exact wording.

#### Acceptance Criteria

1. WHEN tokenizing text THEN the system SHALL lowercase all characters, split on non-alphanumeric boundaries, and remove empty tokens.
2. WHEN tokenizing THEN the system SHALL remove standard English stop words (`a`, `an`, `the`, `to`, `for`, `of`, `in`, `on`, `and`, `or`, `is`, `are`, `was`, `were`, `can`, `now`, `with`, `that`, `this`).
3. WHEN normalizing tokens THEN the system SHALL apply light plural reduction: words ending in `ies` (len > 4) → replace suffix with `y`; words ending in `s` (len > 3) → remove trailing `s`.
4. WHEN the query consists entirely of stop words THEN the system SHALL return an empty result set (no error).

### 3. Summary Section Extraction

**User Story:** As a search user, I want scoring to focus on the Summary section so that results reflect the most relevant intent of each entry.

#### Acceptance Criteria

1. WHEN scoring an entry THEN the system SHALL extract the `## Summary` section content using the existing `section.rs` extraction logic.
2. IF an entry has no `## Summary` section THEN the system SHALL skip that entry (score = 0, not included in results).
3. WHEN extracting THEN the system SHALL handle both `## Summary` and `# Summary` headings (case-insensitive match).

### 4. Text Scoring Model

**User Story:** As a search user, I want results ranked by how well the summary matches my query — with exact phrases and close term proximity weighted highest.

#### Acceptance Criteria

1. WHEN scoring THEN the system SHALL compute:
   - **Matched terms count** × 10 (distinct query terms found in document)
   - **Total term frequency** × 2 (sum of occurrences of all matched query terms)
   - **All-terms bonus** = 25 (when every query term appears)
   - **Exact phrase bonus** = 75 (when the full normalized query phrase appears as a substring of the normalized doc)
   - **Proximity bonus** = 50 / (1 + span) where span is the smallest window containing all query terms
   - **Order bonus** = 10 (when query terms appear in document in query order)
2. WHEN only one query term exists THEN the system SHALL skip proximity and order bonuses (they require ≥ 2 terms).
3. WHEN no query terms match THEN the system SHALL assign a text score of 0.0.

### 5. Recency Scoring

**User Story:** As a search user, I want newer entries to get a slight relevance boost so that recent work surfaces more easily when text scores are close.

#### Acceptance Criteria

1. WHEN computing the final score THEN the system SHALL calculate a recency score using exponential decay: `recency = 100 × e^(-age_days / half_life_days)` with a default half-life of 60 days.
2. WHEN blending scores THEN the system SHALL use adaptive weights based on text score strength:
   - Text score ≥ 80 → recency weight 0.08, text weight 0.92
   - Text score ≥ 40 → recency weight 0.15, text weight 0.85
   - Text score < 40 → recency weight 0.25, text weight 0.75
3. IF the text score is 0.0 THEN the final score SHALL be 0.0 (recency alone cannot produce a result).
4. WHEN computing age THEN the system SHALL derive the entry timestamp from the filename prefix (`YYYY-MM-DD-HHmm`) and compare to the current system time.

### 6. Time Bias Detection

**User Story:** As a search user, I want to type "recent auth changes" and have the system amplify recency weighting without needing separate flags.

#### Acceptance Criteria

1. WHEN the raw query contains the words `recent`, `latest`, or `newest` THEN the system SHALL classify the query as `TimeBias::Recent` and increase recency weights (0.18 / 0.30 / 0.40 by tier).
2. WHEN the raw query contains `old`, `older`, or `earliest` THEN the system SHALL classify as `TimeBias::Old` and set recency weight to 0.0 (oldest entries rank by text score only).
3. WHEN a time-bias keyword is detected THEN the system SHALL strip it from the query terms before text scoring (it is a meta-modifier, not a search term).
4. WHEN no time-bias keywords are detected THEN the system SHALL use `TimeBias::Normal` (default adaptive weights).

### 7. Result Limiting

**User Story:** As a CLI user, I want search results capped at a sensible default so I don't get flooded with low-relevance matches.

#### Acceptance Criteria

1. WHEN displaying search results THEN the system SHALL default to showing the top 10 results.
2. WHEN `--latest <N>` is combined with `--search` THEN the system SHALL cap results at N instead of the default 10.
3. WHEN fewer than the limit entries score > 0 THEN the system SHALL show only those entries (no padding).

### 8. Output Format

**User Story:** As a CLI user, I want search results displayed with enough context to decide which entry to open.

#### Acceptance Criteria

1. WHEN displaying search results THEN each result SHALL show: score (2 decimal places), filename, and summary text (first 120 chars, truncated with `...` if longer).
2. WHEN `--full` is combined with `--search` THEN the system SHALL show full file content for matched entries (sorted by score).
3. WHEN `--type <section>` is combined with `--search` THEN the system SHALL extract the named section from matched entries (sorted by score).

### 9. Performance

**User Story:** As a developer, I want search to feel instant for typical journal sizes (< 5000 entries).

#### Acceptance Criteria

1. WHEN scanning entries THEN the system SHALL tokenize each summary only once per invocation (pre-tokenize after extraction, then score against the tokenized form).
2. The implementation SHALL NOT introduce external crate dependencies — all scoring logic uses standard library types.
3. The implementation SHALL avoid reading file content more than once per entry per invocation.

### 10. Integration with Existing Filters

**User Story:** As a power user, I want to combine search with date range and latest constraints for precise results.

#### Acceptance Criteria

1. WHEN `--search` is combined with `--since` or `--between` THEN the system SHALL filter by date first, then score remaining entries.
2. WHEN `--search` is combined with `--files` THEN the system SHALL filter by filename prefix first, then score remaining entries.
3. `--search` SHALL be mutually exclusive with `--filter` (error on combination).
4. `--search` SHALL be mutually exclusive with `--summary` and `--type` when used as display-mode selectors without explicit override — search results default to their own output format (Req 8).

### 11. Error Handling

**User Story:** As a CLI user, I want clear error messages when I misuse the search flag.

#### Acceptance Criteria

1. WHEN `--search` is passed without a value THEN the system SHALL print `Error: --search requires a query string` and exit 1.
2. WHEN `--search` is combined with `--filter` THEN the system SHALL print `Error: --search and --filter cannot be used together` and exit 1.
3. WHEN the query reduces to zero terms after stop-word removal THEN the system SHALL print no results and exit 0 (not an error).
