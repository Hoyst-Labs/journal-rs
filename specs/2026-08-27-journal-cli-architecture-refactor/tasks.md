# Implementation Plan — Journal CLI Architecture Refactor

- [x] 1. Lock down the current process-level CLI contract before moving production code
  - Create `tests/cli_contract.rs`, `tests/common/mod.rs`, and small deterministic fixtures under `tests/fixtures/journal/` using only the Rust standard library.
  - Invoke `env!("CARGO_BIN_EXE_journal")` with `std::process::Command` and isolated current directories.
  - Assert exact exit code, stdout, and stderr for help, missing journal storage, default listing, summary, full, named section, latest, search, positional aliases, invalid options, missing values, invalid ranges, and mutual exclusions.
  - Include `.journal` precedence over `journal`, deterministic newest-first ordering, exact `No journal entries found.`, exact `No matching entries found.`, score formatting, and empty stderr on success.
  - Run `cargo test --test cli_contract` and keep production behavior unchanged.
  - Refs: Requirements 1.1–1.8, 7.1–7.4, 8.4–8.6

- [x] 2. Introduce typed commands, query requests, and errors while preserving the existing execution pipeline
  - Create `src/error.rs`, `src/domain/mod.rs`, `src/domain/entry.rs`, and `src/domain/query.rs`.
  - Add `AppError`, `UsageError`, `StoreError`, `AppResult`, `Command`, `QueryRequest`, `EntrySelection`, `DateWindow`, `View`, `SearchQuery`, `SectionName`, and non-zero limit modeling.
  - Refactor the current parser into `src/cli/mod.rs` and `src/cli/parse.rs`; have it return `Command::Help` or `Command::Query` with invalid combinations rejected before execution.
  - Move `src/help.rs` to `src/cli/help.rs` without changing the help text.
  - Adapt `src/lib.rs` and `src/main.rs` immediately to consume the typed command/error surface so the crate continues to compile and all baseline contract tests remain green.
  - Add parser and constructor unit tests for every invalid state currently covered by `src/cli.rs` plus search-without-query and zero-limit cases.
  - Refs: Requirements 1.1–1.4, 2.2–2.4, 3.1–3.7, 7.2

- [x] 3. Separate pure entry-name, date, range, and section rules from filesystem behavior
  - Move filename/date/timestamp validation and comparison from `src/journal.rs` and `src/query.rs` into `src/domain/filename.rs` and the domain entry/query types.
  - Move `src/section.rs` to `src/domain/section.rs` with its current H1/H2, case-insensitive, and Issues/Unknowns alias behavior intact.
  - Refactor the active parser and execution code to use the new domain constructors and predicates before removing the migrated functions from the flat modules.
  - Preserve current accepted date shapes and inclusive mixed-precision range behavior.
  - Add unit tests for valid/invalid entry names, path-separator/traversal rejection, date/timestamp prefixes, mixed-precision ranges, content terms, CRLF section input, heading boundaries, aliases, and missing sections.
  - Run the focused domain tests and the process contract suite after wiring the new modules.
  - Refs: Requirements 1.4–1.5, 4.1–4.3, 5.6–5.7, 8.1

- [x] 4. Move ranked search into a pure domain module
  - Move the existing search files to `src/domain/search/{mod.rs,tokenize.rs,score.rs,recency.rs,time_bias.rs}`.
  - Replace path-based `search_entries(filenames, journal_dir, query, now)` with a pure `rank(candidates, query, now)` interface over supplied `EntryMetadata` and Summary text.
  - Preserve stop words, plural normalization, phrase/frequency/proximity/order scoring, adaptive recency weights, the 60-day half-life, time-bias keywords, and two-decimal rendered values.
  - Add a deterministic filename tie-breaker after final-score comparison.
  - Adapt the current execution path to load candidate summaries before calling `rank`; do not introduce the final application abstraction yet.
  - Move and strengthen search unit tests so they use in-memory candidates and a fixed timestamp with no temporary files.
  - Refs: Requirements 1.5, 4.4–4.6, 6.4, 8.1

- [x] 5. Introduce the journal-store and clock ports with production adapters
  - Create `src/ports.rs`, `src/adapters/mod.rs`, `src/adapters/fs_journal.rs`, and `src/adapters/system_clock.rs`.
  - Define only the coarse-grained `JournalStore::list_entries/read_entry` and `Clock::now_unix` capabilities described in the design.
  - Move current-working-directory journal discovery, Markdown enumeration, sorting, and file reads out of `src/journal.rs` into `FsJournalStore`.
  - Construct `FsJournalStore` with the caller's current directory; preserve `.journal` then `journal` discovery and the successful empty-list result when neither exists.
  - Validate `EntryName` before reads and enforce canonical root containment where the platform supports canonicalization; convert I/O and containment failures to `StoreError` without panics.
  - Replace direct `std::fs`, `std::env`, and `SystemTime::now` usage in the active execution path with these adapters.
  - Refs: Requirements 1.6–1.7, 2.1, 4.6, 5.1–5.8, 7.6

- [x] 6. Move orchestration into a dependency-injected application pipeline
  - Create `src/app.rs` with `Application<S, C>` and `execute(QueryRequest) -> AppResult<Outcome>`.
  - Define `Outcome`, `EntryBlock`, and `EntryBody` as structured, presentation-neutral results.
  - Move selection, lazy content loading, content filtering, section extraction, search candidate preparation, ranking, truncation, and read-error continuation out of `src/lib.rs` into `Application`.
  - Add an invocation-scoped cache keyed by `EntryName` so each required file is read at most once per command.
  - Preserve filter-before-read ordering, `--latest` default-to-summary behavior, search's default limit of ten, per-entry errors in non-search views, and skipped unreadable/summary-less search candidates.
  - Keep `src/lib.rs` compiling as a thin façade that composes the real adapters and delegates to `Application`.
  - Refs: Requirements 2.2–2.4, 3.6, 5.1–5.3, 6.1–6.9

- [x] 7. Add deterministic application tests using fakes
  - Create `tests/application.rs` with an in-memory `FakeJournalStore` and `FixedClock` implementing the ports.
  - Assert structured `Outcome` values for default list, full, Summary, named sections, combined filename/date/content filters, latest, search ranking, search limit, all-stop-word search, missing summaries, and empty storage.
  - Instrument the fake store with read counts and assert that content caching prevents duplicate reads.
  - Cover non-search read-error continuation and search read-error exclusion without real filesystem access.
  - Run `cargo test --test application` and the existing domain/unit suites.
  - Refs: Requirements 3.6–3.7, 6.1–6.9, 8.2, 8.5–8.6

- [x] 8. Move rendering and process composition to their final boundaries
  - Create `src/output/mod.rs` and `src/output/text.rs`; move grouped-list, block, missing-section, help, search-preview, truncation, and empty-state formatting out of `src/render.rs` and `src/lib.rs`.
  - Render only from structured `Outcome` and keep formatting functions free of stdout/stderr calls.
  - Create `src/cli/exit.rs` to map successful output and typed failures to stdout/stderr and the current exit codes `0` and `1`.
  - Reduce `src/main.rs` to argument collection, one CLI façade call, and process exit.
  - Make `src/lib.rs` module declarations and intentional re-exports plus the convenience `run` façade; remove direct orchestration and formatting from it.
  - Run the exact-output process suite after the move and adjust implementation, not expected outputs, for unintended differences.
  - Refs: Requirements 1.1–1.8, 2.2–2.4, 3.6–3.7, 7.1–7.6

- [x] 9. Add focused filesystem-adapter and security regression tests
  - Add tests beside `src/adapters/fs_journal.rs` using unique standard-library temporary directories with explicit cleanup.
  - Cover `.journal` precedence, `journal` fallback, missing storage, invalid file-name exclusion, reverse ordering, empty directories, unreadable/invalid UTF-8 entries, and actionable `StoreError` variants.
  - Cover `EntryName` path-separator/traversal rejection on every platform and canonical containment for symlink/junction escapes when the test host permits creating them.
  - Assert that the adapter never searches parents, the home directory, or fallback locations beyond the supplied current directory.
  - Run the adapter tests and full CLI contract suite.
  - Refs: Requirements 5.4–5.8, 7.6, 8.3, 8.6

- [x] 10. Remove the parallel flat architecture and finalize the crate surface
  - Migrate any remaining callers and delete replaced `src/cli.rs`, `src/help.rs`, `src/journal.rs`, `src/model.rs`, `src/query.rs`, `src/render.rs`, `src/section.rs`, and the old `src/search/` tree once each responsibility is active in its target module.
  - Restrict module and item visibility to the smallest necessary scope; stop publicly exposing search implementation submodules.
  - Confirm with source search that domain modules do not import filesystem, environment, current-time, stdout/stderr, process, CLI, adapter, or output concerns.
  - Update the README project-structure section and any module references as part of the same code migration so documentation describes the code that now builds.
  - Run `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`; resolve every warning or regression without changing the captured CLI contract.
  - Refs: Requirements 2.1–2.6, 4.6, 8.7, 9.1–9.6
