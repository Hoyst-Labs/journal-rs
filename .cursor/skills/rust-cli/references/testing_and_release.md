## CLI Testing Strategy

Treat the CLI as a process-level contract.

Test layers:

1. Unit tests for parsing helpers and validation logic.
2. Unit tests for command execution behavior.
3. Integration tests that invoke the binary and assert:
   - exit code
   - stdout
   - stderr
4. Optional end-to-end tests for real filesystem/process interactions.


## Integration Test Patterns

Prefer binary invocation tests for user-visible behavior.

Example with `assert_cmd` and `predicates`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn query_json_writes_json_to_stdout() {
    let mut command = Command::cargo_bin("journal").unwrap();

    command
        .arg("query")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stderr(predicate::str::is_empty());
}
```

Use fixtures for deterministic input files.


## Snapshot and Golden Output Tests

For stable text output, snapshot or golden tests are useful.

Rules:

- Keep fixture inputs small and readable.
- Keep output deterministic (sorting, timestamps, locale).
- Review snapshots intentionally when command UX changes.

If using snapshots:

- Prefer explicit approval workflow.
- Avoid broad snapshotting that hides real regressions.


## Backward Compatibility Rules

Command-line interfaces break users when changed casually.

Treat these as compatibility-sensitive:

- Command/subcommand names
- Flag names and meanings
- Exit-code mapping
- JSON output schema

For breaking changes:

- Version appropriately.
- Add migration notes.
- Keep temporary aliases where practical.


## Packaging and Distribution

Support straightforward installation and version inspection.

Baseline:

- `cargo install --path .`
- `my-cli --version`
- `my-cli --help`

Release hygiene:

- Include version in build metadata.
- Ensure reproducible release commands.
- Validate binary behavior on target platforms.

If cross-platform support matters:

- Test path behavior on Windows and Unix.
- Avoid shell-specific assumptions in docs/examples.


## Recommended CLI Crates

Use only what the project needs.

Common choices:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
thiserror = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

Optional, depending on requirements:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

Pick one parser and keep patterns consistent across commands.
