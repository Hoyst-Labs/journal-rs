## Refactoring Checklist

When asked to refactor a Rust app using this skill, verify:

- [ ] `main.rs` is thin.
- [ ] Core behavior lives in `lib.rs` modules.
- [ ] Domain logic is pure where possible.
- [ ] Side effects are isolated in adapters.
- [ ] Application layer coordinates use cases.
- [ ] Ports describe external dependencies.
- [ ] Formatting is separate from business logic.
- [ ] Error handling uses `Result` and meaningful errors.
- [ ] No production `unwrap()`/`expect()` except documented unrecoverable cases.
- [ ] Config is loaded at the edge and passed inward.
- [ ] Tests cover domain logic and app logic separately.
- [ ] Fakes/mocks can test app behavior without real IO.
- [ ] Types model the domain clearly.
- [ ] Functions are small and named by intent.
- [ ] Modules have clear ownership and responsibility.
- [ ] Public API surface is intentional.
- [ ] No framework/interface-specific assumptions are baked into core code.
- [ ] Duplicate knowledge is removed without over-abstracting.
- [ ] The code remains readable and idiomatic Rust.



## Agent Refactoring Instructions

When an agent applies this skill:

1. Inspect the current Rust files.
2. Identify mixed responsibilities.
3. Propose or directly create a modular structure.
4. Move pure logic first.
5. Add typed request/response structs.
6. Add ports for external dependencies where useful.
7. Move concrete IO into adapters.
8. Move formatting into output modules.
9. Keep entry points thin.
10. Preserve behavior with tests.
11. Avoid introducing unrelated frameworks.
12. Avoid creating CLI/web/API-specific abstractions unless the user explicitly asks.
13. Prefer the smallest clean architecture that solves the problem.
14. Keep names concrete and boring.
15. Do not hide simple logic behind unnecessary abstractions.

