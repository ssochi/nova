# Plan: Staged Compound Assignments

## Basic Information

- Plan ID: `2026-03-20-06-56-53-compound-assignments`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add a staged compound-assignment surface so ordinary accumulation and update code can use `+=`, `-=`, `*=`, and `/=` directly.
- Keep compound assignments explicit in the AST, checked model, and bytecode instead of hiding them inside synthetic plain assignments.
- Preserve Go's single-evaluation rule for the left-hand side, especially for index targets such as `values[i] += 1` and `counts["go"] += 1`.

## Scope

- Research and document the Go behavior baseline for:
  - `op=` assignment semantics
  - single-evaluation behavior of the left-hand side
  - relation between compound assignment and `++` / `--`
  - which statement positions should accept compound assignment in the current staged frontend
- Extend the frontend with:
  - explicit compound-assignment operators for ordinary statements, `if` / `switch` headers, and classic `for` init / post positions
  - source rendering and CLI dumps that keep `op=` visible
- Extend semantic analysis, checked modeling, and lowering for:
  - assignable identifier and index targets
  - staged operator coverage over the currently modeled runtime surface
  - single-evaluation lowering for indexed compound assignments through hidden locals
- Add examples, tests, verification, experience evidence, and synchronized roadmap / design / tech docs.

## Non-Goals

- Tuple compound assignments or any multi-result assignment expansion.
- Bitwise, shift, or modulo compound assignments before the corresponding expression operators exist in the current staged frontend.
- Pointer indirections, selector-based field assignments, channels, labels, `goto`, `fallthrough`, `defer`, `go`, or `select`.
- Broad untyped-constant compatibility work beyond the currently modeled runtime types.

## Phase Breakdown

1. Lock the compatibility baseline in research, open the active `M3` plan, and confirm the staged operator subset.
2. Extend tokens, AST, parser, and rendering for explicit compound assignments in the supported statement positions.
3. Add semantic analysis, checked-model updates, and bytecode lowering with single-evaluation index handling.
4. Add examples, tests, validation reports, experience notes, and synchronize roadmap / design / tech / boot context.

## Acceptance Criteria

- `cargo test` passes with parser, semantic, runtime, and CLI coverage for staged compound assignments.
- `cargo run -- run` succeeds on a new example that uses compound assignments in ordinary statements plus at least one header or `for` clause.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` keep `op=` readable without reading Rust source.
- `cargo run -- check` rejects at least one invalid compound-assignment type mismatch and one invalid non-assignable left side.
- No modified source file exceeds the repository's 1000-line limit.

## Risks

- Compound assignment is deceptively close to plain assignment, but incorrect lowering can evaluate index targets twice and change behavior.
- The current runtime supports only a narrow arithmetic subset, so the staged `op=` surface must stay aligned with actually modeled operators instead of implying full Go coverage.
- Header and classic `for` parsing already juggle multiple simple-statement forms; adding `op=` opportunistically would make the grammar harder to maintain.
