# Journal CLI

A Rust-based command-line tool for browsing and querying structured journal entries stored as Markdown files. Journal entries follow a date-timestamped naming convention (`YYYY-MM-DD-HHmm-description.md`) and live in `.journal/` or `journal/` directories within each project.

The tool is designed to be used by both humans and LLM agents to quickly review past work, extract specific sections, and query across time ranges.

## What It Does

- Discovers journal entries from the current working directory (`.journal/` then `journal/`)
- Lists entries grouped by date
- Extracts specific heading sections from entries (Summary, Issues / Unknowns, Next Actions, etc.)
- Filters entries by date, timestamp, time ranges, and content keywords
- Ranked keyword search with phrase matching, proximity scoring, and recency decay
- Zero external dependencies — pure Rust, standard library only

## Skills  — Skills and Tools Are First-Class Citizens

We use the following skills to build the Journal CLI :
- rust-app
- rust-cli
- src 

Our Journal CLI and Src tools must be used respectively for each sub project.

### Using Journal

Use Journal to review past work when we start a new task.

There is a `.journal` in each project's sub folder in this repo, and in the root. Make sure you check all of them appropriately.

We can use the journal tool to quickly see only the Summary from each in a single run.

```
.journal
/proper-pilots-api/.journal
/properpilots-mobile-ui/.journal
```

## Journal Entry Structure

For coding related changes only —

**Required Structure**

# Summary
Summary of everything for a single / quick lookup.

# Context

What the agent believes the current situation is.

* Relevant system state
* Constraints / assumptions
* What changed since last pass

# References
Reference to original requirements, plan, tasks (if applicable)

# Results

What happened as a result of the actions.

* Outputs
* Errors
* Observed behavior
* Unexpected side effects

# Verification

Did it work?

* Tests run
* Assertions checked
* Manual validation notes
* Pass / Fail Status

# Artifacts

Anything produced that future passes need.

* File paths
* IDs
* Generated outputs
* Links or references

# Issues / Unknowns

What is unclear or blocking.

* Open questions
* Ambiguities
* Risks

# Next Actions

What should happen next.

* Immediate next step
* Fallback if that fails
* Optional improvements

# Delta (Optional)

What changed vs last pass.

* Updated logic
* Decisions made

**END REQUIRED STRUCTURE**

## CLI Interface

### Commands

```
journal                                  # List all entries grouped by date
journal help                             # Show help
journal --summary                        # Print ## Summary from each entry
journal --full --files 2026-04-04-1054   # Print full content of matching entries
journal files 2026-04-04-1054            # List files matching a date/timestamp
journal --latest 3                       # Show the 3 most recent entry summaries
journal --latest 5 --full                # Show full content of the 5 most recent entries
journal --type "Summary"                 # Extract a specific heading section
journal --type "Issues"                  # Matches "Issues / Unknowns" heading
journal --since "2026-04-26"             # Entries since a date
journal --between "2026-04-01" "2026-04-30"            # Entries in a date range
journal --between "2026-04-01-0900" "2026-04-01-1700"  # Entries in a time range
journal --filter "auth|login|session"    # Content filter (pipe-delimited, any match)
journal --search "notes for students"    # Ranked keyword search (scored by relevance + recency)
journal --search "recent auth changes"   # Time-bias keywords amplify recency scoring
journal search "deploy fixes"            # Positional command alias for --search
journal --search "deploy" --since 2026-04-01           # Combine search with date filters
journal --search "migration" --latest 5                # Cap search results
journal --type "Next Actions" --since "2026-04-26" --filter "deploy"  # Combined filters
```

### Mutual Exclusions

- `--summary` and `--type` cannot be combined
- `--since` and `--between` cannot be combined
- `--search` and `--filter` cannot be combined

## Build and Architecture

- Language: Rust (edition 2024)
- Thin binary `src/main.rs` + library `src/lib.rs` with modules per rust-app skill
- Modules: `cli.rs`, `model.rs`, `journal.rs`, `query.rs`, `section.rs`, `render.rs`, `help.rs`, `search/` (submodule)
- No external crate dependencies
- Journal discovery: `.journal/` preferred over `journal/`
- File naming convention: `YYYY-MM-DD-HHmm-description.md`
- Sections are identified by `## Heading` lines in Markdown

### Search Module (`src/search/`)

The search submodule implements scored keyword search:

- `tokenize.rs` — stop word removal, light plural normalization, tokenization
- `score.rs` — phrase matching, proximity (sliding window), term order, frequency scoring
- `recency.rs` — exponential decay with 60-day half-life, adaptive blending, filename timestamp parsing
- `time_bias.rs` — detects "recent"/"latest"/"newest"/"old"/"older"/"earliest" modifiers in queries
