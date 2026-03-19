# Plan: Simple Statements, Short Declarations, and Inc/Dec

## Basic Information

- Plan ID: `2026-03-20-06-34-01-simple-statements-incdec`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add a staged short-variable-declaration surface that unlocks common Go headers such as `if n := len(values); ...` and classic `for i := 0; ...`.
- Add explicit `++` / `--` statements so idiomatic counting loops no longer require verbose assignment rewrites.
- Keep the simple-statement surface explicit in the AST, checked model, and bytecode dumps instead of hiding these forms inside expression lowering.

## Scope

- Research and document the Go behavior baseline for:
  - single-value short variable declarations
  - same-block redeclaration rules
  - `++` / `--` as statements rather than expressions
  - which simple-statement positions accept these forms in `if`, `switch`, and `for`
- Extend the frontend with:
  - statement-level `name := expr`
  - header-level short declarations for `if`, `switch`, and classic `for` init
  - explicit increment / decrement statements over identifier and index-assignment targets
  - `for` post support for `++`, `--`, assignment, expression, and staged map lookup forms
- Extend semantic analysis, checked modeling, and lowering for:
  - same-scope short redeclaration with at least one new named binding
  - typed reuse of existing bindings during short redeclaration
  - assignability checks for `++` / `--`
  - readable bytecode lowering for inc/dec without turning them into user-visible expressions
- Add examples, tests, validation reports, experience notes, and synchronized docs.

## Non-Goals

- Multi-expression or tuple-style general short declarations.
- `++` / `--` as expressions, prefix operators, or values inside larger expressions.
- Labeled control flow, `fallthrough`, `goto`, `defer`, `go`, `select`, or send statements.
- Short declarations in classic `for` post clauses.
- Broader simple statements such as compound assignment operators.

## Phase Breakdown

1. Lock the compatibility baseline in research and open the active `M3` plan.
2. Extend the AST and parser for short declarations plus inc/dec statements in the allowed statement positions.
3. Add semantic analysis, checked-model updates, and bytecode lowering for the staged simple-statement expansion.
4. Add examples, tests, validation reports, experience notes, and synchronize roadmap / design / tech / boot context.

## Acceptance Criteria

- `cargo test` passes with parser, semantic, and CLI coverage for short declarations and inc/dec statements.
- `cargo run -- run` succeeds on a new example that uses `if` or `switch` headers with `:=` and a classic `for i := 0; ...; i++` loop.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` expose the new simple-statement forms clearly enough to debug without reading Rust source.
- `cargo run -- check` rejects at least one invalid short redeclaration and one invalid `++` or `--` target.
- No modified source file exceeds the repository's 1000-line limit.

## Risks

- Short-declaration redeclaration rules are easy to over-approximate unless scope lookup stays explicit and conservative.
- `++` / `--` share assignment-like typing rules but are statement-only; collapsing them into expression parsing would blur the boundary and create future cleanup cost.
- Header and `for`-clause parsing can become ambiguous if `:=` is added opportunistically instead of through explicit simple-statement helpers.
