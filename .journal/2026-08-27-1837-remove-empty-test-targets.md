# Summary

Removed misleading zero-test reports from `cargo test` by disabling the empty binary unit-test harness and adding a meaningful doctest for journal entry filename validation.

# Context

- The Rust test runner created a harness for the thin `src/main.rs` binary even though it intentionally contains no unit-testable behavior.
- The library exposed reusable APIs but had no documentation examples, so the doctest target also reported zero tests.
- Existing coverage already included library unit tests, application tests with fakes, and process-level CLI contract tests.

# References

- User report: some test targets displayed `passed 0, failed 0`.
- `.cursor/skills/rust-app/references/runtime_and_quality.md`
- `.cursor/skills/rust-cli/references/testing_and_release.md`

# Results

- Added an explicit `[[bin]]` target with `test = false` so Cargo does not build a pointless test harness for `src/main.rs`.
- Added a compiling documentation example to `EntryName::parse` that validates date and timestamp extraction.
- A full `cargo test` run now contains no zero-test targets.

# Verification

- `cargo fmt --all -- --check` — PASS.
- `cargo test` — PASS: 36 library unit tests, 5 application tests, 10 CLI contract tests, and 1 doctest.
- `cargo clippy --all-targets -- -D warnings` — PASS.
- `git diff --check` — PASS; only informational LF-to-CRLF working-copy warnings were emitted.

# Artifacts

- `Cargo.toml`
- `src/domain/entry.rs`
- `.journal/2026-08-27-1837-remove-empty-test-targets.md`

# Issues / Unknowns

- No functional blockers remain.
- The repository still contains the broader uncommitted architecture refactor that predates this focused adjustment.

# Next Actions

- Review and commit the broader refactor plus this test-target cleanup when ready.

# Delta

- Replaced two empty test reports with one intentionally omitted binary harness and one real doctest.
