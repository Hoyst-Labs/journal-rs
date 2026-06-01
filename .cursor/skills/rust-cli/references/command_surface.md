## Command Surface Design

Design the CLI as a stable public interface.

Prioritize:

- Clear verb/noun command naming.
- Stable, explicit flags.
- Minimal ambiguity in positional arguments.
- Backward-compatible evolution when possible.

A command should be understandable from `--help` alone.


## Subcommand Modeling

Use subcommands for distinct actions rather than mode flags.

Good:

```txt
journal list
journal show <entry-id>
journal query --since 2026-05-01 --type Summary
```

Avoid:

```txt
journal --list
journal --show <entry-id>
journal --query --since 2026-05-01 --type Summary
```

Prefer enum-based dispatch so every subcommand maps to a typed execution path:

```rust
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "journal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    List(ListArgs),
    Show(ShowArgs),
    Query(QueryArgs),
}
```


## Flag and Argument Design

Treat flags as a compatibility contract.

Rules:

- Prefer long-form flags for readability (`--output`, `--since`).
- Add short aliases only for frequent actions (`-o`, `-q`, `-v`).
- Keep flag meaning consistent across subcommands.
- Avoid overloaded flags with unrelated meanings.

Boolean flags:

- Use positive names (`--verbose`, `--json`).
- Use explicit disable flags only when needed (`--no-color`).

Positional arguments:

- Use for required primary targets.
- Keep count small and ordering obvious.
- Prefer named flags once ambiguity appears.

Conflict and dependency rules should be explicit:

```rust
#[derive(Debug, clap::Args)]
pub struct QueryArgs {
    #[arg(long)]
    pub since: Option<String>,

    #[arg(long)]
    pub until: Option<String>,

    #[arg(long, conflicts_with = "summary")]
    pub full: bool,

    #[arg(long, requires = "since")]
    pub until_inclusive: bool,

    #[arg(long)]
    pub summary: bool,
}
```


## Validation and Parsing Boundaries

The parser should parse shape. Application logic should validate meaning.

Examples:

- Parser checks that `--limit` is a `usize`.
- App checks that `limit` is within policy (`1..=5000`).
- Parser checks that `--format` is one of known values.
- App checks that the chosen format is valid for the command.

Map parsed CLI types into execution requests:

```rust
#[derive(Debug)]
pub struct QueryRequest {
    pub since: Option<String>,
    pub until: Option<String>,
    pub output: OutputMode,
}

impl From<QueryArgs> for QueryRequest {
    fn from(args: QueryArgs) -> Self {
        Self {
            since: args.since,
            until: args.until,
            output: args.output,
        }
    }
}
```


## Help Text and Usage Examples

Good help output reduces support burden and user mistakes.

Use:

- One-sentence command summary.
- Focused per-argument descriptions.
- Examples for common workflows.
- Notes for destructive actions.

Example:

```rust
#[derive(Debug, clap::Args)]
pub struct DeleteArgs {
    /// Entry identifier (YYYY-MM-DD-HHmm-name.md)
    pub entry_id: String,

    /// Skip confirmation prompt
    #[arg(long)]
    pub yes: bool,
}
```

Include concrete usage examples:

```txt
Examples:
  journal list --since 2026-05-01
  journal show 2026-05-01-1430-refactor.md
  journal query --type Summary --format json
```


## Shell Completion and Discoverability

Support completion generation when CLI complexity is non-trivial.

Common pattern:

- `completion bash`
- `completion zsh`
- `completion fish`
- `completion powershell`

Treat completion as optional enhancement, not a replacement for good help text.

Discoverability checklist:

- Top-level help clearly lists primary subcommands.
- Every subcommand has examples.
- Validation errors include corrective hints.
- Error output references `--help` where relevant.
