# Context: Defer First Slice

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, latest archived plan context, milestone index, active plan index, `todo.md`, and the current architecture/runtime docs.
3. Confirmed milestone `M3-standard-library-and-runtime-model` remains `in_progress` with no active plan, so opening a new `M3` plan was the required next action.
4. Reviewed the current parser, semantic, bytecode, and VM seams plus recent named-result work to choose the next substantial slice.
5. Chose a first staged `defer` implementation because it materially improves function-exit semantics, composes with named-result infrastructure, and is stronger progress than another narrow package-helper round.
6. Verified local Go 1.21.5 behavior for eager argument evaluation, LIFO execution, parenthesized-form rejection, and builtin-result restrictions on `defer`.
7. Opened this plan and recorded the staged scope, non-goals, acceptance criteria, and risks.
8. Wrote the new research note and design note for staged `defer`, then synchronized the research/design indexes.
9. Added the `defer` keyword, AST statement node, parser restrictions, and focused parser tests for direct-call defer statements plus parenthesized-form rejection.
10. Added semantic support for explicit defer statements, including builtin statement-context filtering, checked-call reuse, and focused semantic tests.
11. Added explicit `defer-*` bytecode instructions plus frame-level deferred-call handling in the VM with eager argument capture, LIFO unwind order, pending return-value storage, and discard-on-return metadata for deferred user calls.
12. Added `examples/defer.go`, focused VM coverage, focused CLI execution/diagnostic suites, and test indexing updates.
13. Updated technical docs plus `BOOT.md`, then ran `cargo fmt`, `cargo test`, `cargo fmt --check`, serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and `check`, two invalid `check` probes, and touched-file line-count checks.
14. Wrote the formal verification and experience reports for the shipped defer slice.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`.

## Key Information for the Next Trigger

- `defer` now exists explicitly across AST, checked statements, bytecode, and the VM. The current bytecode surface is `defer-builtin`, `defer-package`, `defer-function`, and `defer-function-spread`.
- VM call frames now carry both a deferred-call stack and pending return values. That hook should be reused for future `panic` / `recover` work instead of rewriting defer again.
- The current defer slice intentionally supports only the already-modeled direct-call forms: selected builtins valid in statement context, imported package members, and user-defined function names. Closures, method values, and broader expression-statement cleanup remain open.
