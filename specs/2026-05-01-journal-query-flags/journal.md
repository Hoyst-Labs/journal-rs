# Journal

## Summary

Added four new query flags to the Journal CLI: `--type <heading>` for extracting any named section, `--since <date>` for date-forward filtering, `--between <start> <end>` for bounded date/time ranges, and `--filter <pattern>` for pipe-delimited keyword content search. All flags compose with each other and with existing flags. The section extraction logic was generalized from the hard-coded Summary extractor to support any heading, with a special alias mapping "Issues" and "Unknowns" to the "Issues / Unknowns" heading.

## What Changed

- Arg parsing refactored from individual booleans to a `QueryParams` struct with `DisplayMode` enum.
- `extract_summary_section` replaced by `extract_section(content, heading)` with case-insensitive matching and alias support.
- New filter functions: `matches_since`, `matches_between`, `matches_content_filter`.
- Filter pipeline applied in order: files → since/between → content filter → display.
- Help text updated with all new flags and examples.
- Mutually exclusive flag validation: `--since` vs `--between`, `--summary` vs `--type`.

## Expected Validation

- Unit tests for section extraction with various headings, aliases, and edge cases.
- Unit tests for date comparison functions with date-only and date-time formats.
- Unit tests for content filter with single/multiple terms and case-insensitivity.
- Integration tests with temp directories covering combined flag scenarios.
- Error cases tested: mutually exclusive flags, invalid ranges, missing values.

## Follow-Through

- Refactor `main.rs` into `lib.rs` + modules per the rust-app skill for better testability.
- Consider adding `--output json` for machine-readable output.
- Consider supporting glob patterns in `--filter` beyond simple keyword matching.
