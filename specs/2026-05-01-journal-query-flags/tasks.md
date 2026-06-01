# Implementation Plan — Journal Query Flags

- [ ] 1. Introduce `QueryParams` and `DisplayMode` structs and refactor arg parsing
  - Add `QueryParams` struct with fields: `display_mode`, `files_query`, `since`, `between`, `filter`
  - Add `DisplayMode` enum: `List`, `Summary`, `TypeSection(String)`, `Full`
  - Parse `--type <heading>`, `--since <date>`, `--between <start> <end>`, `--filter <pattern>` from args
  - Validate mutually exclusive flags: `--since` vs `--between`, `--summary` vs `--type`
  - Validate `--between` start <= end
  - Split filter string on `|` into a `Vec<String>`
  - Wire `main()` to use `QueryParams` instead of individual booleans
  - Update help text to document all new flags
  - Ref: Requirements 7, 8, 9, 10, 11

- [ ] 2. Generalize section extraction (`extract_section`)
  - Replace `extract_summary_section` with `extract_section(content, heading) -> Option<String>`
  - Case-insensitive heading comparison
  - Special alias: `"Issues"` or `"Unknowns"` matches `## Issues / Unknowns`
  - Update `format_file_summaries` to call `extract_section(content, "Summary")`
  - Add unit tests: exact match, case-insensitive, Issues alias, Unknowns alias, missing heading, heading at EOF
  - Ref: Requirement 7

- [ ] 3. Implement `--type` display mode
  - When `DisplayMode::TypeSection(heading)` is active, extract and print the named section from each filtered entry
  - Print `[No ## <heading> section found]` when the heading is absent
  - Ensure `--type` composes with `--files`
  - Add unit test for `--type` output formatting
  - Ref: Requirement 7

- [ ] 4. Implement `--since` filter
  - Add `file_timestamp_prefix(filename) -> &str` to extract the comparable `YYYY-MM-DD` or `YYYY-MM-DD-HHmm` prefix
  - Add `matches_since(filename, since) -> bool`: prefix >= since (lexicographic)
  - Apply as a filter step after `--files` filtering
  - Add unit tests: date-only since, date-time since, boundary values
  - Ref: Requirement 8

- [ ] 5. Implement `--between` filter
  - Add `matches_between(filename, start, end) -> bool`: prefix >= start AND <= end
  - Apply as a filter step after `--since` / `--files`
  - Handle mixed format (date-only start, date-time end and vice versa)
  - Add unit tests: date range, time range, mixed formats, boundary inclusive
  - Ref: Requirement 9

- [ ] 6. Implement `--filter` content filter
  - Add `matches_content_filter(content, terms) -> bool`: case-insensitive, any term matches
  - Apply after date-based filters (requires reading file content)
  - Add unit tests: single term, multiple terms, case-insensitivity, no match
  - Ref: Requirement 10

- [ ] 7. Wire the full filter pipeline and display modes together in `main`
  - Apply filters in order: `--files` → `--since` / `--between` → `--filter`
  - Route to display mode: `--type` / `--summary` / `--full` / list
  - Ensure all combinations work (e.g., `--type "Next Actions" --since "2026-04-26" --filter "deploy"`)
  - Ref: Requirement 11

- [ ] 8. Update help text and add integration tests
  - Update `HELP_TEXT` to document `--type`, `--since`, `--between`, `--filter` with examples
  - Add integration tests with temp directory and sample `.md` files covering combined flag scenarios
  - Verify error messages for mutually exclusive flags and invalid ranges
  - Ref: Requirements 6, 7, 8, 9, 10, 11
