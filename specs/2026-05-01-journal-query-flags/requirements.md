# Requirements Document — Journal Query Flags

## Introduction

This spec covers four new CLI flags for the Journal tool: `--type`, `--since`, `--between`, and `--filter`. Together they transform Journal from a simple list/summary viewer into a flexible query engine for structured Markdown journal entries. The existing interface (`--summary`, `--full`, `--files`, `help`) remains unchanged. The tool must stay zero-dependency (Rust standard library only).

See also: [docs/requirements.md](../../docs/requirements.md) for the full requirements including existing features.

---

## Requirements

### Requirement 7 — Section Extraction by Heading Type (`--type`)

**User Story:** As an LLM agent, I want to extract any named heading section from journal entries (not just Summary), so that I can pull targeted context like Next Actions, Issues, or Verification results.

#### Acceptance Criteria

1. WHEN `--type <heading>` is provided THEN the tool SHALL extract the `## <heading>` section content from each matching entry.
2. The heading match SHALL be case-insensitive.
3. WHEN the type is `"Issues"` or `"Unknowns"` THEN the tool SHALL match `## Issues / Unknowns`.
4. IF a file does not contain the requested heading THEN the tool SHALL print `[No ## <heading> section found]`.
5. `--type` SHALL compose with all filter flags (`--files`, `--since`, `--between`, `--filter`).
6. `--summary` is equivalent to `--type "Summary"`. They are mutually exclusive; if both are given, print an error.

### Requirement 8 — Date Range: Since (`--since`)

**User Story:** As a developer, I want to see only entries from a given date forward, so that I can focus on recent work.

#### Acceptance Criteria

1. WHEN `--since <date>` is provided THEN only entries with date/timestamp prefix >= the given value are included.
2. Accepts `YYYY-MM-DD` or `YYYY-MM-DD-HHmm`.
3. Composes with all other flags.

### Requirement 9 — Date Range: Between (`--between`)

**User Story:** As a developer, I want to query entries within a specific date or time window.

#### Acceptance Criteria

1. WHEN `--between <start> <end>` is provided THEN only entries with prefix >= start AND <= end are included.
2. Start and end each accept `YYYY-MM-DD` or `YYYY-MM-DD-HHmm`.
3. IF start > end THEN print an error and exit.
4. `--between` and `--since` are mutually exclusive; if both are given, print an error.
5. Composes with all other flags.

### Requirement 10 — Content Keyword Filter (`--filter`)

**User Story:** As a developer, I want to filter entries by keywords in their content.

#### Acceptance Criteria

1. WHEN `--filter <pattern>` is provided THEN only entries whose content contains at least one of the pipe-delimited terms are included.
2. Match is case-insensitive.
3. Filter applies to the full file content.
4. Composes with all other flags. When combined with `--type`, filter selects files first, then `--type` extracts the section.

### Requirement 11 — Combined Flag Behavior

**User Story:** As a power user, I want to combine multiple flags in a single invocation.

#### Acceptance Criteria

1. All filter flags (`--files`, `--since`, `--between`, `--filter`) narrow the set together (AND logic).
2. Display flags (`--summary`, `--full`, `--type`) control what is shown from the filtered set.
3. If no display flag is given, list matching files grouped by date.
