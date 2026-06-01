# Implement scored keyword search for journal CLI

## Summary

Added `journal --search <query>` — a ranked keyword search that scores journal entries by lexical relevance blended with time-based recency. The scorer operates on `## Summary` sections using phrase matching, term frequency, proximity, order, and exponential recency decay. Results are displayed ranked by composite score with filename and summary preview. Supports time-bias keywords like "recent" and "old" that amplify or suppress the recency component.

## Context

* The journal CLI had `--filter` for simple case-insensitive substring matching (OR across pipe-delimited terms) but no ranked search.
* The `specs/2026-05-31-text-search-scoring/` folder contained concept.md (phrase-aware lexical scorer) and time-based.md (recency decay model) as design inputs.
* The codebase is zero-dependency Rust with a clean modular layout — the search feature needed to fit as a new submodule without disrupting existing patterns.

## References

- specs/2026-05-31-text-search-scoring/requirements.md
- specs/2026-05-31-text-search-scoring/design.md
- specs/2026-05-31-text-search-scoring/tasks.md
- specs/2026-05-31-text-search-scoring/concept.md (scoring model input)
- specs/2026-05-31-text-search-scoring/time-based.md (recency model input)

## Results

* `--search <query>` tokenizes the query, scores all entries, and returns results sorted by final score descending
* Scoring signals: matched terms (×10), term frequency (×2), all-terms bonus (25), exact phrase (75), proximity (50/(1+span)), order (10)
* Recency: exponential decay with 60-day half-life, adaptive blending (strong text matches get less recency weight)
* Time bias: "recent"/"latest"/"newest" amplify recency; "old"/"older"/"earliest" suppress it; keywords stripped from query
* Mutual exclusion with `--filter` (clear error message)
* Composes with `--since`, `--between`, `--files`, `--latest`
* Default result limit: 10 (overridable with `--latest N`)
* Positional alias: `journal search "query"`

## Verification

* 82 tests pass (72 existing + 10 new integration tests)
* Unit tests cover: tokenization, scoring signals, recency decay, time bias detection, filename timestamp parsing
* Integration tests cover: ranked results, --since combo, --between combo, mutual exclusion error, --latest cap, all-stop-words query, time bias stripping, no-summary entries, missing value error, positional command
* `cargo clippy` clean (only 1 pre-existing warning)
* Release build succeeds

## Artifacts

* `src/search/mod.rs` — orchestrator with `search_entries()` public API
* `src/search/tokenize.rs` — stop words, light_normalize(), tokenize()
* `src/search/score.rs` — phrase/proximity/order/frequency text scoring
* `src/search/recency.rs` — exponential decay, adaptive blending, filename_to_unix_timestamp()
* `src/search/time_bias.rs` — TimeBias enum, detect_time_bias()
* `src/model.rs` — DisplayMode::Search, search_query field
* `src/cli.rs` — --search parsing, search positional command, mutual exclusion
* `src/help.rs` — documented --search with examples
* `src/render.rs` — format_search_results()
* `src/lib.rs` — pipeline wiring, 10 integration tests

## Issues / Unknowns

* None — implementation matches the spec cleanly.

## Next Actions

* Update AGENTS.md to reflect --search as implemented
* Consider adding --verbose/--debug to expose score breakdown for tuning
* Future: extend search scope beyond Summary via --search-scope flag
* Future: minimum score threshold flag (--min-score)
