# Requirements Document — Journal CLI

## Introduction

Journal CLI is a Rust-based command-line tool for browsing, querying, and extracting structured journal entries stored as Markdown files. It is designed for use by both humans and LLM agents working in multi-project repositories, where each project maintains its own `.journal/` or `journal/` directory of timestamped Markdown entries.

The tool must remain zero-dependency (standard library only), fast, and predictable in its output so that scripts, agents, and humans can all rely on it.

---

## Existing Requirements (Derived from Current Code)

### Requirement 1 — Journal Directory Discovery

**User Story:** As a developer, I want the tool to automatically find journal entries in the current directory, so that I do not need to pass explicit paths.

#### Acceptance Criteria

1. WHEN the tool is invoked THEN it SHALL look for `.journal/` in the current working directory first.
2. IF `.journal/` does not exist THEN it SHALL look for `journal/`.
3. IF neither directory exists THEN it SHALL print `No journal entries found.` and exit cleanly.
4. The tool SHALL NOT search parent directories, home directories, or any other fallback location.

### Requirement 2 — Entry Listing

**User Story:** As a developer, I want to list all journal entries grouped by date, so that I can quickly see what work happened and when.

#### Acceptance Criteria

1. WHEN the tool is invoked with no arguments THEN it SHALL list all `.md` files grouped by their `YYYY-MM-DD` date prefix, newest first.
2. Each date group SHALL show its files indented under the date heading.
3. Files within a group SHALL be sorted in reverse chronological order (newest first).

### Requirement 3 — File Filtering by Date or Timestamp

**User Story:** As a developer, I want to filter entries by a date or timestamp prefix, so that I can narrow down to a specific day or moment.

#### Acceptance Criteria

1. WHEN `--files <query>` is provided THEN the tool SHALL show only files whose name starts with the query string.
2. The query SHALL support both `YYYY-MM-DD` (date) and `YYYY-MM-DD-HHmm` (timestamp) formats.
3. WHEN a bare positional argument is provided (not a known command) THEN it SHALL be treated as a file filter query.
4. Positional form `files <query>` SHALL behave identically to `--files <query>`.

### Requirement 4 — Summary Extraction

**User Story:** As an LLM agent, I want to extract only the `## Summary` section from each entry, so that I can quickly load context without reading full files.

#### Acceptance Criteria

1. WHEN `--summary` is provided THEN the tool SHALL print only the content between `## Summary` and the next `## ` heading for each matching entry.
2. IF a file has no `## Summary` heading THEN the tool SHALL print `[No ## Summary section found]` for that file.
3. `--summary` SHALL combine with `--files <query>` to filter which entries are summarized.

### Requirement 5 — Full File Display

**User Story:** As a developer, I want to print the full content of journal entries, so that I can review all details.

#### Acceptance Criteria

1. WHEN `--full` is provided THEN the tool SHALL print the complete contents of each matching entry.
2. `--full` SHALL combine with `--files <query>` to filter which entries are displayed.

### Requirement 6 — Help

**User Story:** As a developer, I want a help command that shows usage, so that I can discover available options.

#### Acceptance Criteria

1. WHEN `help`, `--help`, or `-h` is provided THEN the tool SHALL print usage text and exit.
2. The help text SHALL list all supported options and examples.

---

## New Requirements

### Requirement 7 — Section Extraction by Heading Type (`--type`)

**User Story:** As an LLM agent, I want to extract any named heading section from journal entries (not just Summary), so that I can pull targeted context like Next Actions, Issues, or Verification results.

#### Acceptance Criteria

1. WHEN `--type <heading>` is provided THEN the tool SHALL extract the content under the `## <heading>` section from each matching entry.
2. The heading match SHALL be case-insensitive.
3. WHEN the type is `"Issues"` or `"Unknowns"` THEN the tool SHALL match the heading `## Issues / Unknowns` (either keyword maps to the combined heading).
4. IF a file does not contain the requested heading THEN the tool SHALL print `[No ## <heading> section found]` for that file.
5. `--type` SHALL combine with `--files`, `--since`, `--between`, and `--filter`.
6. WHEN `--type` is not provided AND `--summary` is not provided THEN the tool SHALL behave as it does today (list or full display).

### Requirement 8 — Date Range: Since (`--since`)

**User Story:** As a developer, I want to see only entries from a given date forward, so that I can focus on recent work.

#### Acceptance Criteria

1. WHEN `--since <date>` is provided THEN the tool SHALL include only entries whose date prefix is >= the given date.
2. The date SHALL accept `YYYY-MM-DD` format.
3. The date SHALL also accept `YYYY-MM-DD-HHmm` format for time-level precision (comparing against the file's timestamp prefix).
4. `--since` SHALL combine with `--type`, `--summary`, `--full`, `--files`, and `--filter`.

### Requirement 9 — Date Range: Between (`--between`)

**User Story:** As a developer, I want to query entries within a specific date or time range, so that I can review a bounded window of work.

#### Acceptance Criteria

1. WHEN `--between <start> <end>` is provided THEN the tool SHALL include only entries whose date/timestamp prefix is >= start AND <= end.
2. Start and end SHALL each accept `YYYY-MM-DD` or `YYYY-MM-DD-HHmm` format.
3. Start and end formats do not need to match (one can be date-only, the other can include time).
4. IF start > end THEN the tool SHALL print an error and exit.
5. `--between` SHALL combine with `--type`, `--summary`, `--full`, `--files`, and `--filter`.
6. `--between` and `--since` SHALL be mutually exclusive. IF both are provided THEN the tool SHALL print an error.

### Requirement 10 — Content Keyword Filter (`--filter`)

**User Story:** As a developer, I want to filter entries by keywords in their content, so that I can find entries related to a specific topic across all dates.

#### Acceptance Criteria

1. WHEN `--filter <pattern>` is provided THEN the tool SHALL include only entries whose file content contains at least one of the filter terms.
2. Filter terms SHALL be pipe-delimited (e.g., `"auth|login|session"`).
3. The match SHALL be case-insensitive.
4. The filter applies to the full file content, not just headings.
5. `--filter` SHALL combine with `--type`, `--summary`, `--full`, `--files`, `--since`, and `--between`.
6. WHEN combined with `--type`, the filter runs first (selecting files), then `--type` extracts the heading section from matching files.

### Requirement 11 — Combined Flag Behavior

**User Story:** As a power user, I want to combine multiple flags in a single invocation, so that I can build precise queries.

#### Acceptance Criteria

1. All filtering flags (`--files`, `--since`, `--between`, `--filter`) SHALL be composable — they narrow the result set together (AND logic).
2. Display flags (`--summary`, `--full`, `--type`) control what is shown from the filtered set.
3. IF no display flag is given THEN the tool SHALL list matching files grouped by date (default behavior).
4. `--summary` and `--type` SHALL be mutually exclusive. `--summary` is shorthand for `--type "Summary"`.
