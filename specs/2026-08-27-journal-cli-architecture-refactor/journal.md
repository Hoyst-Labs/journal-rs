# Journal

## Summary

Restructured the Journal CLI into a right-sized single-crate architecture with explicit CLI, application, domain, port, adapter, and output boundaries. The existing command surface, current-working-directory discovery behavior, text output, ordering, ranked-search algorithm, successful empty states, and zero-dependency policy were preserved.

The binary remained thin and script-compatible while the reusable application behavior became independently testable with an in-memory journal store and fixed clock.

## What Changed

- Replaced the optional-field `QueryParams` model with typed `Command`, `QueryRequest`, selection, view, search, section, date-window, and non-zero limit types.
- Added typed application, usage, domain, and store errors with stable user-facing diagnostics and the existing `0`/`1` exit behavior.
- Moved filename/date validation, query predicates, Markdown section extraction, tokenization, scoring, recency, and time-bias logic into pure domain modules.
- Changed ranked search to operate on supplied in-memory candidates instead of reading paths directly.
- Added `JournalStore` and `Clock` ports with filesystem and system-clock production adapters.
- Preserved `.journal` precedence over `journal`, restricted discovery to the caller's current working directory, and added entry-name and root-containment protections.
- Moved query orchestration into `Application<S, C>` with invocation-scoped content caching and structured outcomes.
- Moved all text formatting into the output layer and stdout/stderr/exit handling into the CLI boundary.
- Reduced `main.rs` and `lib.rs` to process composition and an intentional reusable crate surface.
- Removed the replaced flat modules and narrowed internal search visibility.
- Updated the README project tree to match the implemented source layout.

## Expected Validation

- Domain unit tests passed for entry names, dates, timestamps, mixed-precision ranges, content filters, Markdown headings, aliases, tokenization, scoring, recency, time bias, and deterministic score ties.
- Application tests passed with a fake journal store and fixed clock for every display mode, combined filters, latest behavior, search limits, read failures, missing summaries, empty storage, and single-read caching.
- Filesystem-adapter tests passed for `.journal` precedence, `journal` fallback, missing directories, valid-entry selection, deterministic sorting, read errors, path rejection, and containment.
- Process-level contract tests passed for command aliases, exact stdout, exact stderr, exit codes, empty states, invalid input, search formatting, and combined queries.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --all-targets -- -D warnings` passed.
- `cargo test` passed.

## Follow-Through

- Keep the captured CLI contract tests as the gate for future command or formatting changes.
- Treat JSON output, subcommand redesign, richer exit codes, indexing, and additional storage backends as separate features rather than extensions of this refactor.
- Add another port only when a real external dependency requires substitution; do not abstract pure helpers or split the crate into a workspace without a concrete need.

