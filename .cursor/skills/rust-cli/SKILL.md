---
name: rust-cli
description: Create, review, or refactor Rust command-line applications with strong CLI UX and contract discipline: clear command surfaces, robust argument parsing, stable stdout/stderr behavior, reliable exit codes, composable input/output, and testable command behavior. Use when the user asks about Rust CLIs, subcommands, flags, help text, command output, shell usage, scripting compatibility, or CLI release readiness.
---

# rust-cli Skill

Use this skill when building or improving Rust command-line applications.

This skill is intentionally standalone and self-sufficient.


## Core Goal

A Rust CLI should be predictable for both humans and scripts.

Strong CLI design means:

- Command names are clear and stable.
- Flags and arguments are explicit and validated.
- Help text is useful and example-driven.
- Structured data goes to `stdout`.
- Diagnostics and progress go to `stderr`.
- Exit codes are stable and intentional.
- Behavior is testable as an external process contract.


## CLI-Only Principles

1. The CLI surface is a product contract.
2. The contract must be composable in shells and CI.
3. Human UX and machine UX are both first-class.
4. Errors are actionable and script-safe.
5. Defaults are safe, deterministic, and unsurprising.


## Non-Negotiable Rules

1. Keep command parsing and command execution separate.
2. Never mix data output with logs, warnings, or progress text.
3. Use `stdout` for command results and `stderr` for diagnostics.
4. Define and document stable non-zero exit codes.
5. Avoid interactive prompts unless explicitly enabled.
6. Treat command names, flags, and output schema as compatibility surface.
7. Validate inputs at the boundary and return clear failure messages.
8. Keep output ordering deterministic when scripts depend on it.
9. Do not leak secrets in errors, logs, or debug output.
10. No production `unwrap()`/`expect()` in command execution paths.


## Agent Workflow

When implementing or refactoring a Rust CLI:

1. Define the command contract first (subcommands, args, flags, output formats, exit codes).
2. Model parsed CLI input into typed command requests.
3. Implement execution paths that return typed outcomes.
4. Render outcomes into `stdout` formats and route diagnostics to `stderr`.
5. Map errors into user-safe messages plus stable exit codes.
6. Add integration tests that execute the binary as users/scripts do.
7. Verify help text and examples stay in sync with behavior.


## Reference Catalog

Load reference files selectively based on the current CLI task.

### `references/command_surface.md`
- `## Command Surface Design`
- `## Subcommand Modeling`
- `## Flag and Argument Design`
- `## Validation and Parsing Boundaries`
- `## Help Text and Usage Examples`
- `## Shell Completion and Discoverability`

### `references/output_errors_and_exit_codes.md`
- `## stdout and stderr Contract`
- `## Output Modes`
- `## Error Message Design`
- `## Exit Code Policy`
- `## TTY-Aware UX`
- `## Machine-Readable Errors`

### `references/input_config_and_files.md`
- `## Input Sources and Precedence`
- `## Config Loading Strategy`
- `## Stdin and Stream Handling`
- `## File and Path Safety`
- `## Interactive Mode Rules`
- `## Environment Variable Conventions`

### `references/testing_and_release.md`
- `## CLI Testing Strategy`
- `## Integration Test Patterns`
- `## Snapshot and Golden Output Tests`
- `## Backward Compatibility Rules`
- `## Packaging and Distribution`
- `## Recommended CLI Crates`

### `references/minimal_template.md`
- `## Minimal Rust CLI Template`
