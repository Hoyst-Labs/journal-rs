# Journal

## Summary

Implemented `--search <query>` for the journal CLI — a scored keyword search that ranks journal entries by lexical relevance blended with time-based recency. The scorer operates on extracted `## Summary` sections using phrase matching, term frequency, proximity, order, and exponential recency decay. Results are displayed ranked by composite score with filename and summary preview. The feature integrates cleanly with existing date-range and filename filters, and maintains the project's zero-dependency constraint.

## What Changed

- **New `src/search/` submodule** with 5 files: `mod.rs` (orchestrator), `tokenize.rs` (stop words, normalization, tokenization), `score.rs` (phrase/proximity/order/frequency scoring), `recency.rs` (exponential decay, adaptive blending, filename-to-timestamp), `time_bias.rs` (query keyword detection for recent/old bias).

- **Extended CLI** (`cli.rs`, `model.rs`): Added `--search <query>` flag and `search` positional command, `DisplayMode::Search` variant, mutual exclusion with `--filter`, validation for missing query value.

- **Pipeline integration** (`lib.rs`): After existing date/file filters, the search branch tokenizes the query, scores all remaining entries, sorts by final score descending, and caps results (default 10 or `--latest N`).

- **Render extension** (`render.rs`): New `format_search_results` function producing `[score] filename\n  summary...` output, with `--full` and `--type` overrides supported.

- **Help text** (`help.rs`): Documented `--search` with usage examples.

- **Design decisions**: Summary-only scoring for relevance density; adaptive recency weights so strong text matches aren't displaced by newer-but-weaker entries; time-bias keywords stripped from query before scoring; naive UTC timestamp math (sufficient for relative age); no external crates.

## Expected Validation

- Unit tests pass for each search submodule: tokenization (stop words, plurals, edge cases), scoring (each signal isolated and combined), recency (known decay values, tier thresholds), time bias (keyword detection and stripping).
- Integration tests pass for `--search` basic usage, combined with `--since`/`--between`, mutual exclusion with `--filter`, all-stop-word queries, result capping, and time bias behavior.
- `cargo test` passes with no regressions to existing query-flag tests.
- Manual validation: `journal --search "notes for students"` returns ranked results with "Added notes that the owner can send to students" scoring highest.

## Follow-Through

- Update `AGENTS.md` to reflect `--search` as an implemented command (currently lists query flags as "planned" even though they're live).
- Consider adding `--verbose` or `--debug` flag to expose score breakdown (text_score, recency_score, matched terms) for tuning.
- Future: extend search scope beyond Summary to other sections via `--search-scope` flag.
- Future: consider adding a minimum score threshold flag (`--min-score`) to suppress noise.
- Future: TF-IDF weighting once entry count grows large enough for term rarity to matter.
