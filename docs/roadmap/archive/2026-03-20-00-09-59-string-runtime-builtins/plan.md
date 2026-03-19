# Plan: String Runtime and Builtin Contracts

## Basic Information

- Plan ID: `2026-03-20-00-09-59-string-runtime-builtins`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Add a first richer runtime value category by supporting Go-style string literals end to end.
- Centralize builtin function contracts so semantic analysis and lowering stop scattering builtin rules.
- Expand the CLI-visible execution surface with string-aware builtins that improve real-program viability.

## Scope

- Frontend support for double-quoted string literals
- Semantic support for `string` values and builtin contract lookup
- Bytecode and VM support for string values plus builtin dispatch updates
- Builtin expansion beyond `println`, with validation and execution coverage
- Examples, tests, verification traces, and synced design / tech / roadmap documents

## Non-Goals

- Import declarations, package graphs, or standard library package implementations
- Composite types such as slices, maps, structs, or interfaces
- Full Go string literal coverage such as raw string literals or complex escape semantics
- Backend code generation or IR bridge work

## Phase Breakdown

1. Open the first active `M3` plan and record the runtime / builtin slice being introduced.
2. Extend the frontend and checked-program model for string literals and `string` typing.
3. Introduce a centralized builtin contract table and wire it into semantic validation and lowering.
4. Extend bytecode execution for string values and builtin behavior, then exercise the real CLI.
5. Sync docs, reports, and handoff context, then archive the plan if all acceptance criteria land.

## Acceptance Criteria

- `cargo test` passes with coverage for string literals and the new builtin surface.
- `cargo run -- run` can execute a real CLI example that manipulates and prints strings.
- `check` rejects at least one invalid builtin usage involving the new contracts.
- The builtin contract surface is defined in one reusable place rather than duplicated ad hoc.
- Milestone and technical documents describe the shipped runtime model slice and the remaining `M3` gap.

## Risks

- String support can leak runtime assumptions into the semantic layer if builtin and type rules are not isolated cleanly.
- Builtin expansion can turn into special-case branching unless metadata stays centralized.
- Lexer changes for quoted literals can create confusing diagnostics if unterminated strings are not reported clearly.
