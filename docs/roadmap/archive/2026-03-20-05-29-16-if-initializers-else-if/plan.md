# Plan: If Initializers and Else-If Chains

## Basic Information

- Plan ID: `2026-03-20-05-29-16-if-initializers-else-if`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add staged `if` statement initializers so existing bindings, assignments, and comma-ok `map` lookups can drive real Go-style branch headers.
- Add `else if` chaining while keeping `dump-ast` and `dump-bytecode` readable.
- Preserve explicit scoping in the checked layer so header bindings remain visible inside both `then` and `else` branches without leaking outside the statement.

## Scope

- Research and document the Go behavior baseline for `if` header simple statements, scope visibility, and `else if` chaining.
- Support `if <simple-stmt>; <condition> { ... }` for the currently modeled simple-statement subset: expression statements, assignments, `var` declarations, and staged comma-ok `map` lookups.
- Support `else if` syntax without lowering it into a lossy source representation.
- Extend semantic analysis, checked modeling, lowering, and CLI inspection so `if` header scopes stay explicit and testable.
- Add examples, automated tests, CLI validation, and documentation updates for the new control-flow ergonomics.

## Non-Goals

- Full Go simple-statement coverage such as send statements, `++`, `--`, `go`, `defer`, or generalized short variable declarations.
- `switch` or `for` initializers in this round.
- General tuple expressions or multi-result values beyond the existing staged comma-ok `map` lookup statement.
- Source-span-rich diagnostics or multi-file package semantics.

## Phase Breakdown

1. Lock the compatibility baseline and open the active plan under `M3`.
2. Extend the AST and parser for `if` initializers plus explicit `else if` chaining.
3. Add scoped semantic analysis, checked-model support, and bytecode lowering for the new `if` header surface.
4. Add examples, tests, CLI validation, and synchronize design / tech / roadmap / report documents.

## Acceptance Criteria

- `cargo test` passes with parser, semantic, bytecode, and CLI coverage for `if` initializers and `else if` chains.
- `cargo run -- run` succeeds on at least one new example that uses a comma-ok `map` lookup in an `if` header.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` keep the `if` initializer path readable without inspecting implementation code.
- Header-bound names are visible inside both `then` and `else` branches but fail when referenced after the `if` statement.
- No modified source file exceeds the repository's 1000-line limit.

## Risks

- Header scoping can easily become inconsistent if the analyzer treats the initializer as either an outer statement or a branch-local statement.
- `else if` support can degrade debug readability if it is hidden inside synthetic blocks too early.
- Parser and CLI test files are already moderately large, so test additions may require extra extraction discipline.
