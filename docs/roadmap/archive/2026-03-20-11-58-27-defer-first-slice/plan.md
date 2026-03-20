# Plan: Defer First Slice

## Basic Information

- Plan ID: `2026-03-20-11-58-27-defer-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `in_progress`
- Owner: primary agent

## Goals

- Add a first staged `defer` statement that materially improves Go function-exit behavior on the existing VM path.
- Keep deferred calls explicit across AST, semantic analysis, bytecode, and VM execution instead of hiding them behind ad hoc return rewriting.
- Reuse the recently added named-result and multi-result infrastructure without claiming unsupported `defer` surfaces.

## Scope

- Research and document the official Go baseline for `defer` evaluation timing, LIFO execution, and staged call-shape restrictions.
- Parse source-level `defer <call>` statements and keep them readable in `dump-ast`.
- Type-check deferred calls using the existing builtin, package, and user-defined call contract path.
- Lower deferred calls into explicit bytecode/runtime behavior that preserves argument evaluation time and runs defers before a frame returns.
- Add focused examples, unit tests, CLI execution coverage, and CLI diagnostics for the staged surface.

## Non-Goals

- `panic`, `recover`, or panic-accurate deferred failure semantics
- Function literals, method values, or arbitrary deferred call targets beyond the currently supported direct call surface
- `defer` interactions that require closures or addressable named-result mutation
- Reworking the broader expression-statement compatibility rules outside the new `defer` slice

## Phase Breakdown

1. Capture the official baseline in research and lock the staged design.
2. Implement lexer/parser/semantic support for explicit `defer` statements.
3. Implement explicit bytecode and VM deferred-call execution with LIFO ordering and eager argument capture.
4. Add layered validation, CLI playtests, and roadmap/doc synchronization.

## Acceptance Criteria

- `cargo run -- run` executes a realistic example that proves eager defer-argument evaluation plus LIFO exit ordering.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` keep deferred calls visible enough to inspect without reading implementation code.
- `cargo run -- check` accepts staged valid `defer` usage and rejects at least one invalid non-call or parenthesized form with a targeted diagnostic.
- Research, design, tech, reports, roadmap artifacts, and `BOOT.md` all describe the shipped surface and deferred gaps.
- Touched code and document files remain within the repository's 1000-line soft limit.

## Risks

- VM return handling can become fragile if deferred-call execution is bolted on after values are already unwound instead of modeled explicitly per frame.
- The current language subset lacks closures and methods, so the plan must state clearly which real Go `defer` behaviors remain intentionally absent.
- Parser support can accidentally accept broader grouped expressions such as `defer (f())` unless the defer-specific restriction is enforced deliberately.
