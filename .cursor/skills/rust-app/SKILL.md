---
name: rust-app
description: Create, review, or refactor a Rust application so the codebase follows strong application-architecture practices: modularity, separation of concerns, SOLID, DRY, testability, explicit boundaries, clear error handling, and maintainable project structure.
meta: Created with Orcatect
date: 2026-05-01
---

# rust-app Skill

Use this skill when creating, reviewing, or refactoring a Rust application so the codebase follows strong application-architecture practices: modularity, separation of concerns, SOLID, DRY, testability, explicit boundaries, clear error handling, and maintainable project structure.

This skill is intentionally **not** specific to CLIs, web APIs, workers, daemons, embedded apps, games, or libraries. Platform-specific entry points belong in separate skills. This skill defines the reusable Rust application structure underneath those entry points.


## Core Goal

A Rust app should be organized around **application behavior**, not around the first interface that happens to call it.

Good Rust application structure separates:

- **Domain logic**: pure business rules and domain types.
- **Application/use-case logic**: orchestration of domain behavior.
- **Ports**: traits that describe required external capabilities.
- **Adapters**: concrete implementations for files, databases, HTTP, environment variables, clocks, queues, etc.
- **Entry points**: thin binaries, handlers, commands, jobs, or tests that call the application layer.
- **Presentation/formatting**: conversion of application results into human/API/UI output.

The ideal result is that the same core behavior can be called from a CLI, HTTP API, background worker, test harness, or other interface without rewriting business logic.


## Core Principles

A Rust application following this skill should feel like this:

- The binary is disposable.
- The core behavior is reusable.
- The domain is understandable.
- Side effects are obvious.
- Tests are easy to write.
- Errors are explicit.
- Modules have clear boundaries.
- The structure scales from small apps to larger apps without turning into ceremony.

Prefer practical, idiomatic Rust over architecture theater.


## Non-Negotiable Rules

1. **Do not put application logic in `main.rs`.**
   - `main.rs` should parse platform-specific input, build dependencies, call the app, and exit.
   - Most logic belongs in `lib.rs` and modules under `src/`.

2. **Do not bake interface concerns into core modules.**
   - Do not make domain/application functions know they are being called by a CLI, web route, Lambda, Worker, cron job, or GUI.
   - Return typed values. Let the caller decide how to print, serialize, log, or display them.

3. **Prefer small modules with clear responsibility.**
   - A module should have one main reason to change.
   - Avoid “god files” with argument parsing, filesystem access, formatting, filtering, and domain logic mixed together.

4. **Use explicit data types instead of loose strings, maps, or tuples.**
   - Model concepts with structs, enums, and newtypes.
   - Use `Result<T, E>` for fallible operations.

5. **Prefer dependency injection through traits or concrete dependency structs.**
   - Domain logic should not directly call `std::fs`, databases, HTTP clients, environment variables, clocks, random generators, or global state.
   - Wrap external behavior behind ports when it matters for testability or substitution.

6. **Keep side effects at the edges.**
   - Reading files, writing output, network access, environment variables, time, randomness, and process exits should be in adapters or entry-point layers.

7. **Separate computation from formatting.**
   - Functions should produce structured results.
   - Formatting output as text, JSON, HTML, Markdown, logs, etc. belongs in a presentation layer.

8. **Never use `unwrap()` or `expect()` in production paths unless failure is truly unrecoverable and documented.**
   - Tests may use `unwrap()` when it keeps test intent clear.
   - Application code should return errors with context.

9. **Keep error types meaningful.**
   - Library/domain/application code should expose useful typed errors.
   - Entry points can convert errors into exit codes, HTTP status codes, logs, or user-facing messages.

10. **Design for tests first.**
    - The core app should be testable without launching the binary, touching real files, using real networks, or relying on system time.


## Reference Catalog

Load reference files selectively based on the current task.

### `references/architecture_layers.md`
- `## Recommended Project Shape`
- ``## `main.rs` Should Stay Thin``
- ``## `lib.rs` Should Expose the App Surface``
- `## Domain Layer`
- `## Application Layer`
- `## Ports`
- `## Adapters`
- `## Dependency Injection Patterns`
- `### Generic Dependencies`
- `### Trait Objects`
- `### Shared Dependencies`

### `references/runtime_and_quality.md`
- `## Configuration`
- `## Error Handling`
- `## Result Types`
- `## Data Modeling`
- `## Formatting and Presentation`
- `## Logging and Tracing`
- `## Async Boundaries`
- `## Testing Strategy`
- `### Unit Test Pure Domain Logic`
- `### Test App Logic With Fakes`
- `### Temporary Directories Without Extra Crates`
- `### Prefer Deterministic Tests`

### `references/refactoring_large_single_file.md`
- `## Refactoring Large Single-File Apps`
- `## Example Refactor Target`

### `references/principles_and_guidelines.md`
- `## SOLID Applied to Rust`
- `### Single Responsibility`
- `### Open/Closed`
- `### Liskov Substitution`
- `### Interface Segregation`
- `### Dependency Inversion`
- `## DRY Without Over-Abstracting`
- `## Ownership and Borrowing Guidelines`
- `## Collections`
- `## Iterators`
- `## Module Visibility`
- `## Constants`
- `## Validation`
- `## Builder Pattern`
- `## Service Pattern`
- `## Repository Pattern`
- `## DTO Mapping`
- `## Serialization`
- `## Feature Flags`
- `## Cargo Workspace Pattern`
- `## Recommended Crates`
- `## Documentation`
- `## Comments`
- `## Performance Guidelines`
- `## Concurrency`
- `## State Management`
- `## File and Path Handling`
- `## Parsing`
- `## Sorting and Grouping`
- `## Naming Guidelines`
- `## What Not To Do`

### `references/minimal_template.md`
- `## Minimal Template`

### `references/refactoring.md`
- `## Refactoring Checklist`
- `## Agent Refactoring Instructions`
