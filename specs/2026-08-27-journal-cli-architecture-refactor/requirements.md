# Requirements Document — Journal CLI Architecture Refactor

## Introduction

The Journal CLI has grown from a small file browser into a zero-dependency query and ranked-search tool. Its current behavior is well tested, but application orchestration, filesystem access, time access, domain rules, CLI parsing, and text rendering cross module boundaries in ways that will make future changes harder to isolate.

This refactor will reorganize the existing single Rust crate around explicit CLI, application, domain, port, adapter, and output responsibilities. It is a behavior-preserving change: the existing command surface, current-working-directory discovery contract, output text, ordering, search scoring, and standard-library-only constraint remain authoritative. This spec does not add a new user-facing command or alter journal storage.

## Requirements

### Requirement 1 — Preserve the Existing CLI Contract

**User Story:** As a Journal CLI user or calling script, I want the refactored binary to behave exactly like the current binary, so that the restructure does not break established workflows.

#### Acceptance Criteria

1. WHEN the refactored binary receives any currently supported flag or positional command THEN the system SHALL preserve its current meaning and output behavior.
2. WHEN the user invokes `journal`, `journal help`, `journal --summary`, `journal --full`, `journal --type`, `journal --files`, `journal --since`, `journal --between`, `journal --filter`, `journal --search`, or `journal --latest` THEN the system SHALL preserve the current command contract.
3. WHEN the user invokes the positional aliases `summary`, `full`, `files`, or `search` THEN the system SHALL preserve the current alias behavior.
4. WHEN mutually exclusive options are combined THEN the system SHALL preserve the current validation rules for `--summary` with `--type`, `--since` with `--between`, and `--search` with `--filter`.
5. WHEN matching entries are returned THEN the system SHALL preserve current sorting, grouping, truncation, score formatting, section fallback text, and search preview formatting.
6. IF `.journal/` and `journal/` are both absent from the caller's current working directory THEN the system SHALL print exactly `No journal entries found.` and exit successfully.
7. IF both `.journal/` and `journal/` exist THEN the system SHALL select `.journal/` first.
8. WHEN the refactor is complete THEN the system SHALL NOT introduce a new command, flag, output mode, storage location, or CLI dependency.

### Requirement 2 — Establish a Right-Sized Single-Crate Architecture

**User Story:** As a maintainer, I want the crate organized around stable responsibilities, so that future changes can be made without touching unrelated code.

#### Acceptance Criteria

1. WHEN the source tree is reorganized THEN the system SHALL remain one Cargo package with one `journal` binary and one reusable library.
2. WHEN the binary starts THEN `main.rs` SHALL perform only process-boundary work: collect arguments, invoke the library, route output and diagnostics, and select an exit code.
3. WHEN library consumers inspect `lib.rs` THEN it SHALL expose an intentional application surface rather than contain the execution pipeline or expose internal search helpers.
4. WHEN responsibilities are assigned THEN CLI parsing, application orchestration, domain rules, external adapters, and output formatting SHALL reside in distinct modules.
5. WHEN the new module tree is introduced THEN it SHALL use concrete, purpose-based names and SHALL avoid a Cargo workspace, async runtime, framework, or trait-per-function architecture.
6. WHEN the crate is built THEN it SHALL continue to use only the Rust standard library.

### Requirement 3 — Model Commands and Outcomes with Explicit Types

**User Story:** As a maintainer, I want parsed input and application output represented by valid typed states, so that incompatible combinations are rejected at the boundary rather than handled deep in execution.

#### Acceptance Criteria

1. WHEN CLI arguments are parsed THEN the parser SHALL return a typed command that distinguishes help from journal queries.
2. WHEN a journal query is parsed THEN the result SHALL group entry selection, view mode, search input, and result limit into explicit types.
3. IF a command contains a missing value, invalid date shape, invalid range, zero limit, unknown option, or mutually exclusive combination THEN the parser SHALL reject it before application execution.
4. WHEN a search view reaches the application layer THEN its search query SHALL be present by construction.
5. WHEN help is requested THEN the parser SHALL represent help as a command variant rather than a boolean attached to query state.
6. WHEN the application completes THEN it SHALL return a structured outcome that is independent of stdout formatting.
7. WHEN an operation fails THEN it SHALL return a typed error rather than an unclassified `String` error from the reusable application surface.

### Requirement 4 — Keep Journal and Search Rules Pure

**User Story:** As a maintainer, I want journal parsing, filtering, section extraction, and ranking to be pure domain behavior, so that they can be tested deterministically without real files or system time.

#### Acceptance Criteria

1. WHEN journal filenames are validated or compared THEN the domain layer SHALL perform the work without reading the filesystem.
2. WHEN date, timestamp, range, filename-prefix, or content filters are evaluated THEN the domain layer SHALL operate only on supplied values.
3. WHEN a Markdown section is extracted THEN the domain layer SHALL preserve current H1/H2 boundary and `Issues / Unknowns` alias behavior without performing I/O.
4. WHEN ranked search is executed THEN tokenization, time-bias detection, text scoring, recency scoring, and result ordering SHALL operate on supplied entry data and a supplied current timestamp.
5. WHEN search behavior is moved THEN the system SHALL preserve the current stop words, normalization, scoring constants, 60-day recency half-life, adaptive blending, and time-bias keywords.
6. WHEN domain modules are compiled THEN they SHALL NOT directly access `std::fs`, `std::env`, `SystemTime::now`, stdout, stderr, or process exit.

### Requirement 5 — Isolate Filesystem and Clock Side Effects

**User Story:** As a maintainer, I want external behavior behind small ports and concrete adapters, so that the application can run against fakes in tests and real resources in production.

#### Acceptance Criteria

1. WHEN the application needs journal metadata or content THEN it SHALL use a `JournalStore` port rather than call filesystem functions directly.
2. WHEN ranked search needs the current time THEN the application SHALL use a `Clock` port rather than call the system clock directly.
3. WHEN the production binary is composed THEN it SHALL use a filesystem-backed journal store and a system-clock adapter.
4. WHEN the filesystem adapter discovers a journal directory THEN it SHALL inspect only `./.journal` followed by `./journal` under the supplied current working directory.
5. WHEN the filesystem adapter lists entries THEN it SHALL include only Markdown files with valid date-prefixed journal names and SHALL return them in deterministic newest-first order.
6. WHEN entry content is requested THEN the filesystem adapter SHALL read only entries returned by its own discovery/listing boundary and SHALL not treat user input as an arbitrary filesystem path.
7. IF an entry filename contains path separators or escapes the selected journal directory THEN the filesystem adapter SHALL reject the read without exposing unrelated filesystem content.
8. IF a filesystem read fails THEN the adapter SHALL return a typed I/O error with actionable path context and SHALL not panic.

### Requirement 6 — Centralize the Application Execution Pipeline

**User Story:** As a maintainer, I want one application pipeline that coordinates selection, loading, searching, and outcomes, so that behavior is consistent across every display mode.

#### Acceptance Criteria

1. WHEN a query is executed THEN the application layer SHALL coordinate journal discovery, entry listing, filename/date selection, optional content filtering, optional content loading, optional search ranking, limit application, and outcome creation.
2. WHEN filename and date filters are present THEN the application SHALL apply them before content reads.
3. WHEN `--filter` is present THEN the application SHALL load content after filename/date selection and apply the current case-insensitive any-term rule.
4. WHEN `--search` is present THEN the application SHALL apply filename/date selection before loading summaries and ranking entries.
5. WHEN `--latest N` is used without an explicit display selector THEN the application SHALL preserve the current default-to-summary behavior.
6. WHEN `--search` is used without `--latest` THEN the application SHALL preserve the current default limit of ten results.
7. IF a non-search display mode encounters an unreadable selected entry THEN the application SHALL preserve the current per-file error result while continuing other entries.
8. IF search encounters an unreadable entry or an entry without a non-empty Summary section THEN the application SHALL preserve current behavior by excluding that entry and continuing the search.
9. WHEN the same entry content is required by multiple stages of one invocation THEN the application SHALL reuse the loaded content rather than perform redundant reads.

### Requirement 7 — Preserve Output, Diagnostics, and Exit Semantics

**User Story:** As a shell and automation user, I want result data and diagnostics to remain predictable, so that the CLI stays safe to compose in scripts.

#### Acceptance Criteria

1. WHEN a command succeeds THEN the CLI adapter SHALL write the rendered result to stdout.
2. WHEN argument parsing or application execution fails THEN the CLI adapter SHALL write the diagnostic to stderr and return the current non-zero exit status of `1`.
3. WHEN no entries or no search matches are found THEN the CLI SHALL preserve the current successful exit behavior and exact message for that condition.
4. WHEN outcomes are rendered THEN the output layer SHALL preserve the current text contract for grouped lists, full-file blocks, extracted sections, search results, and help.
5. WHEN the application layer is tested THEN it SHALL be possible to inspect outcomes without parsing rendered stdout text.
6. IF internal errors contain implementation details not useful to the caller THEN the CLI adapter SHALL render a concise user-facing diagnostic without leaking file contents or unrelated paths.

### Requirement 8 — Add Layered Regression Coverage

**User Story:** As a maintainer, I want tests at domain, application, adapter, and process boundaries, so that file moves and future changes cannot silently break the CLI contract.

#### Acceptance Criteria

1. WHEN pure filename, query, section, tokenization, scoring, recency, or time-bias behavior is changed or moved THEN colocated unit tests SHALL verify its existing behavior.
2. WHEN application pipeline behavior is tested THEN tests SHALL use a fake journal store and fixed clock without touching the real filesystem or system time.
3. WHEN filesystem discovery and reading are tested THEN focused adapter tests SHALL use isolated temporary directories and cover `.journal` precedence, `journal` fallback, missing directories, invalid filenames, read errors, and path containment.
4. WHEN the CLI contract is tested THEN process-level integration tests SHALL assert exit code, stdout, and stderr for representative commands, aliases, invalid input, empty state, search, and combined filters.
5. WHEN stable text output is asserted THEN fixtures or explicit expected strings SHALL make ordering and formatting regressions visible.
6. WHEN tests are added THEN they SHALL remain standard-library-only and SHALL not add development dependencies.
7. WHEN the refactor is complete THEN `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` SHALL pass.

### Requirement 9 — Complete the Migration Without Parallel Architectures

**User Story:** As a maintainer, I want the refactor completed incrementally and cleanly, so that the repository does not retain duplicate execution paths or stale documentation.

#### Acceptance Criteria

1. WHEN a new boundary replaces an existing module responsibility THEN callers and tests SHALL be migrated before the obsolete code is removed.
2. WHEN the refactor is complete THEN there SHALL be one argument parser, one application execution pipeline, one filesystem journal adapter, one set of pure search rules, and one text renderer.
3. WHEN internal modules are finalized THEN visibility SHALL be restricted to the smallest required crate or public scope.
4. WHEN the source layout changes THEN the README project structure SHALL be updated in the same implementation change to match the actual module tree.
5. WHEN the migration is complete THEN obsolete `model.rs`, `journal.rs`, `query.rs`, `render.rs`, `help.rs`, or equivalent duplicate responsibilities SHALL not remain as compatibility shims unless a concrete library consumer requires them.
6. WHEN implementation work is sequenced THEN every intermediate task SHALL leave the crate compiling with relevant automated tests passing.

