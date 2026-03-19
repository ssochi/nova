# Plan: For Loops and Path Analysis

## Basic Information

- Plan ID: `2026-03-19-23-57-06-for-loops-path-analysis`
- Milestone: `M2-frontend-expansion`
- Status: `completed`

## Goals

- Add `for` loop control flow through parsing, semantic analysis, bytecode lowering, and VM execution.
- Extend semantic return-path analysis so value-returning functions remain validated once loops are introduced.
- Improve CLI-facing inspection and validation coverage for loop-heavy programs.

## Scope

- Frontend support for condition-only `for` loops
- Semantic validation for loop conditions, scoped loop bodies, and loop-aware return-path analysis
- Bytecode lowering that reuses existing jump instructions for looping control flow
- VM execution validation plus CLI-visible AST and bytecode output for loops
- Example programs, automated tests, and synced design / tech / roadmap documents

## Non-Goals

- Full Go `for` forms with init / post clauses or `range`
- `break`, `continue`, labels, or `switch`
- Import resolution, package graphs, or richer standard library emulation
- Native backend work or broader type-system expansion

## Phase Breakdown

1. Register the active `M2` plan and record the loop-focused execution surface.
2. Extend the frontend model and parser for `for <condition> { ... }`.
3. Update semantic analysis and return-path rules for loops without collapsing layer boundaries.
4. Lower loop control flow into bytecode and validate runtime execution through the existing VM.
5. Add examples and tests, then sync reports, milestone state, and handoff context.

## Acceptance Criteria

- `cargo test` passes with coverage for loop execution and loop-related semantic failures.
- `cargo run -- run` can execute a real program that iterates via `for`.
- `dump-ast` and `dump-bytecode` expose understandable loop structures for inspection.
- `check` rejects at least one invalid loop construct before bytecode lowering.
- The active plan, milestone, and supporting docs all describe the shipped loop surface and the remaining `M2` gap, if any.

## Risks

- Introducing loops can make return-path analysis subtly unsound if infinite-loop cases are over- or under-approximated.
- Parser growth around `for` can make room for unsupported Go forms unless errors stay explicit.
- Loop lowering bugs can hide in off-by-one jump targets, so bytecode inspection coverage matters.
