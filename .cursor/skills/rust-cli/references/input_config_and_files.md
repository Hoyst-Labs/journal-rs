## Input Sources and Precedence

For predictable CLI behavior, define precedence explicitly.

Recommended precedence:

1. CLI flags and arguments
2. Environment variables
3. Config file
4. Built-in defaults

Document this order in `--help` and docs.


## Config Loading Strategy

Load config once near command entry and pass typed config through execution.

Avoid:

- Scattered environment reads across modules.
- Multiple implicit config sources in deep call paths.

Example typed config:

```rust
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub data_dir: std::path::PathBuf,
    pub output: OutputMode,
    pub color: ColorMode,
}
```

Resolve effective config from precedence rules:

```rust
pub fn resolve_config(args: &GlobalArgs) -> Result<CliConfig, CliError> {
    // merge flags + env + file + defaults
    // validate final values
    todo!("resolve and validate effective config")
}
```


## Stdin and Stream Handling

Support pipeline-friendly input and output.

Patterns:

- Use `-` to mean stdin for file-like arguments.
- Read stdin only when explicitly requested or when no direct input is provided.
- Do not block waiting for stdin unexpectedly.

Example UX:

```txt
cat input.json | journal import --from -
journal import --from entries.json
```

When reading large input:

- Stream when practical.
- Avoid loading everything into memory unless necessary.


## File and Path Safety

Path handling rules:

- Use `Path` and `PathBuf`, not raw string concatenation.
- Normalize relative behavior in docs.
- Validate user-provided file names for unsafe traversal where needed.

Write behavior:

- Provide explicit overwrite controls (`--force`, `--no-clobber`).
- Avoid silent destructive writes.
- Use atomic write patterns when possible for config and generated files.

Failure messages should include the target path and next action.


## Interactive Mode Rules

Default to non-interactive behavior for script compatibility.

For destructive actions:

- Prompt only when running in interactive mode and confirmation is required.
- Support explicit `--yes` or `--confirm` to skip prompts in automation.

Example:

```txt
journal delete 2026-05-01-1430-refactor.md --yes
```

Do not require prompts in CI-friendly flows.


## Environment Variable Conventions

Use predictable uppercase names with a clear prefix.

Example:

- `JOURNAL_DATA_DIR`
- `JOURNAL_OUTPUT`
- `JOURNAL_COLOR`

Guidelines:

- Document env vars in help/docs.
- Keep env var names stable.
- Validate env input the same as CLI values.
- Report invalid env values with clear corrective hints.
