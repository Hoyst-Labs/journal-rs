# Implement --latest flag for journal CLI

## Summary

Added `journal --latest {x}` flag that shows the `## Summary` section from the x most recent journal entries. The flag composes with other filters and display modes — when no explicit display mode is set, it defaults to summary output.

## Context

* The journal CLI already supported `--summary`, `--full`, `--type`, `--files`, `--since`, `--between`, and `--filter` flags.
* The user wanted a quick way to see summaries of the N most recent entries without specifying date ranges.
* Files are sorted newest-first by `list_journal_files`, so "latest N" means taking the first N after filtering.

## References

- AGENTS.md — project structure and CLI interface spec
- specs/2026-05-01-journal-query-flags/requirements.md — original query flags requirements

## Results

* `--latest <count>` parses a positive integer argument
* After all other filters (files, since, between, filter) are applied, the entry list is truncated to the most recent `count`
* If no display mode was explicitly set (i.e. defaults to List), it auto-promotes to Summary display
* Can be combined with `--full` or `--type` to override the default summary behavior

## Verification

* All 28 unit tests pass (24 existing + 4 new)
* New tests cover: parsing valid count, rejecting zero, rejecting missing value, integration test with 4 entries confirming only the 2 latest are shown with their summaries
* Smoke test with `cargo run -- --latest 2` and `--help` confirmed correct output

## Artifacts

* `src/model.rs` — added `latest: Option<usize>` to `QueryParams`
* `src/cli.rs` — `--latest` parsing + 3 unit tests
* `src/lib.rs` — truncate logic + display mode promotion + 1 integration test
* `src/help.rs` — updated help text and examples

## Issues / Unknowns

* None — straightforward addition with no blocking issues.

## Next Actions

* Update AGENTS.md planned commands section to reflect `--latest` as implemented
* Consider whether `--latest` should also work as a positional subcommand (e.g. `journal latest 3`)
