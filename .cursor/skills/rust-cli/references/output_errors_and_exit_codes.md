## stdout and stderr Contract

For shell composability:

- `stdout`: command results meant for piping or redirection.
- `stderr`: diagnostics, progress, warnings, and errors.

Never print progress text to `stdout` when a command may be used in scripts.

Good:

```txt
# stdout
{"count": 12}

# stderr
scanned 12 entries
```

Bad:

```txt
# stdout (mixed)
scanned 12 entries
{"count": 12}
```


## Output Modes

Provide explicit output modes for commands that return data.

Typical modes:

- `text` for human-readable defaults.
- `json` for machine consumption.

Prefer explicit selection:

```txt
journal query --format text
journal query --format json
```

If JSON mode exists:

- Keep schema stable across patch releases.
- Avoid adding noisy metadata without flags.
- Keep ordering deterministic when possible.


## Error Message Design

Error output should be actionable and concise.

Include:

1. What failed.
2. Why it failed (if known).
3. What the user should do next.

Example:

```txt
error: invalid value for --since: "2026-13-99"
hint: expected YYYY-MM-DD or YYYY-MM-DD-HHmm
```

Avoid:

- Stack traces by default.
- Internal type names.
- Raw parser internals that users cannot act on.


## Exit Code Policy

Define a small, stable exit-code set.

Example policy:

- `0`: success
- `1`: generic application failure
- `2`: invalid usage or argument validation failure
- `3`: missing resource (e.g., file not found)
- `4`: partial success (optional, if command semantics require it)

Represent codes in a dedicated type:

```rust
#[derive(Debug, Clone, Copy)]
pub enum ExitCode {
    Ok = 0,
    Failure = 1,
    InvalidUsage = 2,
    NotFound = 3,
}

impl ExitCode {
    pub fn code(self) -> i32 {
        self as i32
    }
}
```

Map application errors to codes in one place.


## TTY-Aware UX

Progress bars, colors, and spinners should be TTY-aware.

Rules:

- Enable color by default only for TTY.
- Support `--color auto|always|never`.
- Disable spinners in non-interactive output.
- Keep `--quiet` behavior consistent.

Diagnostic verbosity should be explicit:

- `-v` for extra detail.
- `-q` for reduced output.


## Machine-Readable Errors

If commands support `--format json`, support JSON errors when practical.

Example:

```json
{
  "error": {
    "code": "INVALID_DATE",
    "message": "Invalid value for --since",
    "hint": "Expected YYYY-MM-DD or YYYY-MM-DD-HHmm"
  }
}
```

Guidelines:

- Keep `code` values stable.
- Keep human text in `message`.
- Include optional `hint` for quick recovery.
- Avoid exposing sensitive internal details.
