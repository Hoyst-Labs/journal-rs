# Design Document — Journal Query Flags

## Overview

Add four new flags (`--type`, `--since`, `--between`, `--filter`) to the Journal CLI. The design introduces a query pipeline that filters entries before display, and generalizes section extraction beyond the current hard-coded Summary logic.

The current codebase is a single `main.rs` with ~370 lines and zero dependencies. The design keeps the zero-dependency constraint and stays within a single binary, but the new flags require enough logic that a refactor into focused modules is recommended (per the rust-app skill).

## Architecture

```
CLI input
  │
  ├─ Parse args ──► QueryParams { type, since, between, filter, files, display_mode }
  │
  ├─ Discover journal dir (.journal / journal)
  │
  ├─ Load all entries (file name + lazy content)
  │
  ├─ Filter pipeline (each narrows the set):
  │    1. --files  → prefix match on filename
  │    2. --since  → date/timestamp >= value
  │    3. --between → date/timestamp >= start AND <= end
  │    4. --filter  → case-insensitive content keyword match (any term)
  │
  └─ Display:
       --type <heading> → extract ## <heading> section from each entry
       --summary        → equivalent to --type "Summary"
       --full           → print full content
       (none)           → list files grouped by date
```

### Flow

1. **Arg parsing** produces a `QueryParams` struct. Validation happens here: mutually exclusive flags (`--since` vs `--between`, `--summary` vs `--type`) are caught with an error message.
2. **Journal discovery** is unchanged (`.journal/` then `journal/`).
3. **Entry collection** gathers all `.md` files with valid date prefixes, sorted newest-first.
4. **Filter pipeline** runs each active filter in order. Filters are composable (AND). Content-based filters (`--filter`) require reading the file; date-based filters (`--since`, `--between`, `--files`) operate on the filename alone.
5. **Display** renders the filtered set based on the chosen display mode.

## Components and Interfaces

### QueryParams

```rust
struct QueryParams {
    display_mode: DisplayMode,
    files_query: Option<String>,
    since: Option<String>,         // YYYY-MM-DD or YYYY-MM-DD-HHmm
    between: Option<(String, String)>,  // (start, end)
    filter: Option<Vec<String>>,   // pipe-split keywords
}

enum DisplayMode {
    List,               // default: grouped file listing
    Summary,            // --summary (sugar for TypeSection("Summary"))
    TypeSection(String), // --type <heading>
    Full,               // --full
}
```

### Section Extraction (generalized)

Replace the current `extract_summary_section` with a general `extract_section`:

```rust
fn extract_section(content: &str, heading: &str) -> Option<String>
```

- Case-insensitive heading match.
- Special alias: if heading is `"Issues"` or `"Unknowns"`, match `## Issues / Unknowns`.
- Extracts content from after the heading line until the next `## ` line or EOF.

### Date/Timestamp Comparison

File names follow `YYYY-MM-DD-HHmm-description.md`. The comparable prefix is the first 10 chars (date) or first 15 chars (date-time). Comparison is lexicographic, which works correctly for this format.

```rust
fn file_timestamp_prefix(filename: &str) -> &str
// Returns the longest matching prefix: YYYY-MM-DD-HHmm (15 chars) or YYYY-MM-DD (10 chars)

fn matches_since(filename: &str, since: &str) -> bool
fn matches_between(filename: &str, start: &str, end: &str) -> bool
```

### Content Filter

```rust
fn matches_content_filter(content: &str, terms: &[String]) -> bool
// Case-insensitive check: returns true if content contains any term
```

## Data Models

No persistent data. All state is derived from the filesystem at invocation time.

- **Input:** Markdown files in `.journal/` or `journal/` named `YYYY-MM-DD-HHmm-*.md`.
- **Intermediate:** `BTreeMap<String, Vec<String>>` (date → filenames), unchanged from today.
- **Output:** UTF-8 text to stdout.

## Error Handling

| Condition | Behavior |
|---|---|
| No journal directory | Print `No journal entries found.` and exit 0 |
| `--since` and `--between` both present | Print error message and exit 1 |
| `--summary` and `--type` both present | Print error message and exit 1 |
| `--between` start > end | Print error message and exit 1 |
| `--type` missing its value | Print error message and exit 1 |
| `--between` missing one or both values | Print error message and exit 1 |
| File read failure | Print per-file error, continue with remaining files |
| Heading not found for `--type` | Print `[No ## <heading> section found]` for that file |

## Testing Strategy

- Unit tests for `extract_section` with various headings, the Issues/Unknowns alias, case-insensitivity, and missing headings.
- Unit tests for `matches_since`, `matches_between` with date and date-time formats.
- Unit tests for `matches_content_filter` with single and multiple pipe-delimited terms, case-insensitivity.
- Unit tests for arg parsing validation (mutually exclusive flags, missing values).
- Integration tests with temp directories and real `.md` files for end-to-end query scenarios.
