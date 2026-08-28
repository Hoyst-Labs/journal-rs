# Journal CLI

A Rust command-line tool for browsing and querying structured journal entries stored as Markdown files.

Journal entries follow a date-timestamped naming convention (`YYYY-MM-DD-HHmm-description.md`) and live in `.journal/` or `journal/` directories within each project.

## Features

- **Discovery** — automatically finds `.journal/` or `journal/` in the current directory
- **Listing** — entries grouped by date, newest first
- **Section extraction** — pull specific `## Heading` sections (Summary, Next Actions, etc.)
- **Date filtering** — `--since`, `--between` for date/time range queries
- **Content filtering** — `--filter` for case-insensitive substring matching (OR across terms)
- **Ranked search** — `--search` for scored keyword search with phrase matching, proximity, and recency decay
- **Latest** — `--latest N` for quick access to recent entries
- **Zero dependencies** — pure Rust, standard library only

## Installation

```bash
cargo build --release
# Binary at target/release/journal
```

## Usage

```bash
# List all entries grouped by date
journal

# Show summaries of all entries
journal --summary

# Show the 3 most recent entry summaries
journal --latest 3

# Extract a specific section from all entries
journal --type "Next Actions"

# Filter by date range
journal --since 2026-04-26
journal --between 2026-04-01 2026-04-30

# Content substring filter (OR across pipe-delimited terms)
journal --filter "deploy|release"

# Ranked keyword search (scored by relevance + recency)
journal --search "notes for students"

# Time-bias keywords: "recent"/"latest"/"newest" boost recency
journal --search "recent auth changes"

# Combine search with date filters
journal --search "deploy fixes" --since 2026-05-01 --latest 5

# Positional command aliases
journal search "deploy fixes"
journal files 2026-04-04-1054

# Show help
journal help
```

## Search Scoring

The `--search` command ranks entries using a composite score:

| Signal | Points |
|--------|--------|
| Matched distinct query terms | count x 10 |
| Total term frequency | hits x 2 |
| All query terms present | +25 |
| Exact phrase match | +75 |
| Proximity (terms close together) | 50 / (1 + span) |
| Terms appear in query order | +10 |
| Recency boost (adaptive) | blended with text score |

Recency uses exponential decay (60-day half-life). Strong text matches get minimal recency influence; weak matches get more help from being recent.

### Time Bias

Prefix your query with time-modifier words:
- `recent`, `latest`, `newest` — amplify recency weight
- `old`, `older`, `earliest` — suppress recency (rank by text score only)

These keywords are stripped from the query before text scoring.

## Entry Format

Entries are Markdown files named `YYYY-MM-DD-HHmm-description.md` with `## Heading` sections:

```markdown
## Summary
Brief description of the work.

## Context
Current situation, constraints, assumptions.

## Results
What happened — outputs, errors, behavior.

## Verification
Tests run, pass/fail status.

## Artifacts
File paths, IDs, generated outputs.

## Issues / Unknowns
Open questions, blockers.

## Next Actions
Immediate next steps.
```

## Project Structure

```
src/
├── main.rs                 # Process boundary: stdout, stderr, exit code
├── lib.rs                  # Intentional crate surface and adapter composition
├── app.rs                  # Dependency-injected query execution pipeline
├── error.rs                # Typed usage, application, and store errors
├── ports.rs                # JournalStore and Clock interfaces
├── adapters/
│   ├── fs_journal.rs       # Current-directory discovery and contained file reads
│   └── system_clock.rs     # Production Unix-time source
├── cli/
│   ├── parse.rs            # Arguments and aliases → typed Command
│   ├── help.rs             # Stable baked-in help text
│   └── exit.rs             # stdout/stderr and exit-status mapping
├── domain/
│   ├── entry.rs            # Safe entry names and metadata
│   ├── filename.rs         # Date/timestamp parsing and comparisons
│   ├── query.rs            # Typed selection and view models
│   ├── section.rs          # Markdown H1/H2 section extraction
│   └── search/
│       ├── mod.rs          # Pure in-memory ranking
│       ├── tokenize.rs     # Stop words and normalization
│       ├── score.rs        # Phrase/proximity/order/frequency scoring
│       ├── recency.rs      # Exponential decay and timestamp conversion
│       └── time_bias.rs    # Recent/old query modifiers
└── output/
    └── text.rs             # Stable human-readable rendering

tests/
├── application.rs          # Fake-store and fixed-clock application tests
├── cli_contract.rs         # Process-level stdout/stderr/exit tests
└── fixtures/journal/       # Deterministic CLI fixtures
```

## License

Private — HoystAI
