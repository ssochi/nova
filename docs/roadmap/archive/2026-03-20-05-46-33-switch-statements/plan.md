# Plan: Switch Statements

## Basic Information

- Plan ID: `2026-03-20-05-46-33-switch-statements`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add staged `switch` statement support so common Go control-flow shapes work through the normal CLI path.
- Reuse the explicit statement-header abstraction introduced for `if` instead of creating a second incompatible header model.
- Keep `dump-ast` and `dump-bytecode` readable enough to debug `switch` lowering without reading Rust implementation code.

## Scope

- Research and document the Go behavior baseline for expression `switch`, tagless `switch`, header scoping, clause order, and duplicate `default` / constant-case diagnostics.
- Generalize the current `if`-only header statement model into a shared control-flow header abstraction used by both `if` and `switch`.
- Support staged `switch` syntax with:
  - optional header simple statement from the current supported subset
  - optional tag expression
  - expression lists in `case`
  - one optional `default`
  - implicit clause termination without `fallthrough`
- Extend semantic analysis, checked modeling, lowering, and return-path analysis for the new `switch` surface.
- Add examples, automated tests, CLI validation, and document updates for the new control-flow slice.

## Non-Goals

- Type switches, `select`, `fallthrough`, `break`, `continue`, `goto`, `defer`, or `go`.
- General short variable declarations beyond the already staged statement forms.
- Channel runtime work in this round.
- Multi-file package loading or broader package-backed runtime services.

## Phase Breakdown

1. Lock the compatibility baseline for staged `switch` support and open the active plan under `M3`.
2. Generalize header statement modeling and extend the AST / parser for `switch` syntax.
3. Add semantic analysis, checked-model support, duplicate-case diagnostics, and bytecode lowering for staged `switch`.
4. Add examples, tests, validation reports, and synchronize roadmap / design / tech / boot context.

## Acceptance Criteria

- `cargo test` passes with parser, semantic, and CLI coverage for expression and tagless `switch` statements.
- `cargo run -- run` succeeds on at least one new example that exercises header-scoped `switch`, multiple `case` expressions, and `default`.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` keep the `switch` control flow inspectable without source-code inspection.
- `check` rejects at least one invalid `switch` program involving duplicate `default`, incompatible case types, or leaked header bindings.
- No modified source file exceeds the repository's 1000-line limit.

## Risks

- Reusing the `if` header surface can create churn if the abstraction is only partially generalized.
- `switch` lowering can become unreadable if hidden tag storage or clause jumps are not made explicit in bytecode output.
- Parser growth can push `src/frontend/parser.rs` over the file-size ceiling unless helper extraction happens in the same round.
