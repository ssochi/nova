# Plan: Map Comma-Ok Lookups and Literal Diagnostics

## Basic Information

- Plan ID: `2026-03-20-05-11-15-map-comma-ok-and-literal-diagnostics`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add the first staged comma-ok `map` lookup path to the language and VM.
- Improve `map` literal diagnostics so duplicate constant keys fail during semantic analysis instead of silently overwriting.
- Keep the new multi-result surface explicit in the AST, checked model, bytecode, and CLI inspection layers.

## Scope

- Research and document the Go behavior baseline for comma-ok `map` lookups, short redeclaration rules, and duplicate constant map-literal keys.
- Support staged comma-ok lookup statements in the common forms `value, ok := m[key]` and `value, ok = m[key]`.
- Support blank identifiers and controlled short-redeclaration behavior for the staged lookup statement without widening into general tuple expressions.
- Diagnose duplicate constant keys for the currently modeled scalar literal key forms in `map[K]V{...}`.
- Split or extract helpers from `src/runtime/vm.rs`, `src/frontend/parser.rs`, or nearby test modules if the new work would push them past repository limits.

## Non-Goals

- General tuple or multi-result expressions outside comma-ok map lookup statements.
- `if value, ok := m[key]; ok { ... }` initializers, `switch`, or other statement-header work.
- Full compile-time constant folding beyond the scalar literal keys already modeled in the AST.
- Broader short variable declaration support outside `range` headers and comma-ok lookup statements.

## Phase Breakdown

1. Lock the compatibility baseline and open the plan under `M3`.
2. Add frontend and checked-model support for explicit comma-ok lookup statements plus duplicate literal-key diagnostics.
3. Lower comma-ok lookup execution into readable bytecode and execute it in the VM.
4. Add examples, automated tests, CLI validation, and sync roadmap / design / tech / report documents.

## Acceptance Criteria

- `cargo test` passes with parser, semantic, VM, and CLI coverage for comma-ok map lookups and duplicate-key failures.
- `cargo run -- run` succeeds on at least one new comma-ok-focused example program.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` keep the staged comma-ok lookup path visible without reading implementation code.
- Duplicate constant map literal keys fail during semantic analysis before reaching runtime execution.
- No modified source file exceeds the repository's 1000-line limit.

## Risks

- General multi-result features can sprawl quickly unless comma-ok lookup remains a statement-only staged surface.
- Short redeclaration semantics can become inconsistent with `range` if name freshness and blank-identifier rules are not centralized.
- `src/runtime/vm.rs`, `tests/cli_execution.rs`, and `tests/cli_diagnostics.rs` are already near the repository size threshold and may require extraction.
