# Plan: For Clauses, Break, and Continue

## Basic Information

- Plan ID: `2026-03-20-06-12-55-for-clauses-break-continue`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add staged classic `for` clause support so loops can express init / condition / post control flow through the normal CLI path.
- Add unlabeled `break` and `continue` so `for`, `range`, and `switch` become materially more usable for realistic Go control flow.
- Keep the staged control-flow surface explicit in the AST, checked model, and bytecode dumps so later loop and channel work can build on it safely.

## Scope

- Research and document the Go behavior baseline for `for` statements, omitted conditions, `break`, `continue`, and the nearest-enclosing-target rule.
- Extend the frontend with:
  - unlabeled `break` and `continue` statements
  - explicit classic `for` clauses with optional init, optional condition, and optional post
  - preserved support for the existing condition-only `for`, infinite `for`, and staged `range`
- Extend semantic analysis, checked modeling, and termination analysis for:
  - loop / switch breakability tracking
  - loop-only `continue`
  - staged `for` init scope shared by condition, post, and body
  - conservative infinite-loop return-path analysis that respects modeled `break`
- Extend bytecode lowering for loop and switch control transfer while keeping `dump-bytecode` readable.
- Add examples, automated tests, CLI validation, and synchronized docs for the new control-flow slice.

## Non-Goals

- Labels, labeled `break` / `continue`, `goto`, `fallthrough`, `defer`, `go`, `select`, or type switches.
- Full Go simple-statement coverage such as `++`, `--`, send statements, or general short variable declarations.
- String / channel / integer / iterator-function `range`.
- Channel runtime work or broader package-backed standard-library seams in this round.

## Phase Breakdown

1. Open the active `M3` plan and lock the compatibility baseline for staged loop control work.
2. Extend the AST and parser for classic `for` clauses plus `break` / `continue`.
3. Add semantic analysis, checked modeling, termination analysis, and bytecode lowering for the new control-flow surface.
4. Add examples, tests, validation reports, experience notes, and synchronize roadmap / design / tech / boot context.

## Acceptance Criteria

- `cargo test` passes with parser, semantic, and CLI coverage for classic `for` clauses plus unlabeled `break` / `continue`.
- `cargo run -- run` succeeds on a new example that exercises `for` init / condition / post, `break` in both `switch` and loop contexts, and `continue` in both classic and `range` loops.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` expose the new loop-control surface clearly enough to debug without reading Rust source.
- `cargo run -- check` rejects at least one invalid source for `break` outside a breakable construct and one invalid source for `continue` outside a loop.
- No modified source file exceeds the repository's 1000-line limit.

## Risks

- `break` target analysis is easy to get wrong once loops and `switch` nest, especially for return-path reasoning.
- Classic `for` clause parsing can become ambiguous with the existing staged `range` headers unless the parser keeps the surfaces explicit.
- Lowering `continue` across classic `for` post statements and `range` loop increments can become unreadable if loop patch points are not modeled deliberately.
