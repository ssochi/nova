# Plan: Semantic Analysis, Functions, and Branches

## Basic Information

- Plan ID: `2026-03-19-23-37-05-semantic-functions-branches`
- Milestone: `M2-frontend-expansion`
- Status: `completed`

## Goals

- Insert a dedicated semantic analysis stage between parsing and bytecode lowering.
- Support multi-function programs with user-defined calls and real VM call frames.
- Add boolean comparisons plus `if` / `else` branching through the full CLI path.
- Leave clear docs, validation evidence, and handoff context for the remaining loop work in `M2`.

## Scope

- Frontend growth for function signatures, boolean literals, comparisons, and `if` / `else`
- New semantic module for symbol collection, scope checks, and basic type validation
- Bytecode and VM updates for function calls, returns, comparisons, and conditional jumps
- New examples, integration tests, and CLI verification traces
- Documentation sync across roadmap, design, tech, and validation layers

## Non-Goals

- Loop forms such as `for`, `range`, or `switch`
- Package imports or standard library emulation beyond existing builtins
- Native backend work or backend-facing IR stabilization
- Rich Go typing beyond the minimum integer / boolean / void surface needed for this slice

## Phase Breakdown

1. Open and register the active `M2` plan.
2. Extend the syntax tree and parser for functions, calls, comparisons, and branching.
3. Add semantic analysis and route the driver through it before lowering.
4. Expand bytecode lowering and VM execution for calls and branches.
5. Validate through tests and real CLI commands, then synchronize roadmap and technical documents.

## Acceptance Criteria

- `cargo test` passes with new coverage for multi-function execution and semantic failures.
- `cargo run -- run` can execute a program that calls user-defined functions and takes an `if` / `else` path.
- `check` surfaces at least one semantic error before bytecode lowering.
- The current active plan, milestone state, and supporting docs all reflect the shipped capability and remaining gap.

## Risks

- Function signatures can drag the parser toward a larger type system if this slice is not kept narrow.
- Adding semantic validation and lowering at the same time can blur boundaries unless the driver contracts stay explicit.
- Branch instructions and call frames raise VM complexity; diagnostics must remain understandable when failures occur.
