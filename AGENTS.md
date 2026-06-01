# Journal CLI

A Rust-based command-line tool for browsing and querying structured journal entries stored as Markdown files. Journal entries follow a date-timestamped naming convention (`YYYY-MM-DD-HHmm-description.md`) and live in `.journal/` or `journal/` directories within each project.

The tool is designed to be used by both humans and LLM agents to quickly review past work, extract specific sections, and query across time ranges.

## What It Does

- Discovers journal entries from the current working directory (`.journal/` then `journal/`)
- Lists entries grouped by date
- Extracts specific heading sections from entries (Summary, Issues / Unknowns, Next Actions, etc.)
- Filters entries by date, timestamp, time ranges, and content keywords
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

### Summary
Summary of everything for a single / quick lookup.

### Context

What the agent believes the current situation is.

* Relevant system state
* Constraints / assumptions
* What changed since last pass

### References
Reference to original requirements, plan, tasks (if applicable)

### Results

What happened as a result of the actions.

* Outputs
* Errors
* Observed behavior
* Unexpected side effects

### Verification

Did it work?

* Tests run
* Assertions checked
* Manual validation notes
* Pass / Fail Status

### Artifacts

Anything produced that future passes need.

* File paths
* IDs
* Generated outputs
* Links or references

### Issues / Unknowns

What is unclear or blocking.

* Open questions
* Ambiguities
* Risks

### Next Actions

What should happen next.

* Immediate next step
* Fallback if that fails
* Optional improvements

### Delta (Optional)

What changed vs last pass.

* Updated logic
* Decisions made

**END REQUIRED STRUCTURE**

## CLI Interface

### Current Commands

```
journal                                  # List all entries grouped by date
journal help                             # Show help
journal --summary                        # Print ## Summary from each entry
journal --full --files 2026-04-04-1054   # Print full content of matching entries
journal files 2026-04-04-1054            # List files matching a date/timestamp
```

### Planned Commands

```
journal --type "Summary"                 # Extract a specific heading section
journal --type "Issues"                  # Matches "Issues / Unknowns" heading
journal --since "2026-04-26"             # Entries since a date
journal --between "2026-04-01" "2026-04-30"          # Entries in a date range
journal --between "2026-04-01-0900" "2026-04-01-1700" # Entries in a time range
journal --filter "auth|login|session"    # Keyword filter (pipe-delimited, any match)
journal --type "Next Actions" --since "2026-04-26" --filter "deploy"  # Combined
```

## Build and Architecture

- Language: Rust (edition 2024)
- Single binary: `src/main.rs` (to be refactored into `lib.rs` + modules per rust-app skill)
- No external crate dependencies
- Journal discovery: `.journal/` preferred over `journal/`
- File naming convention: `YYYY-MM-DD-HHmm-description.md`
- Sections are identified by `## Heading` lines in Markdown
