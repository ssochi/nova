# Plan: Typed Var Declarations and Zero-Value Slices

## Basic Information

- Plan ID: `2026-03-20-02-18-45-typed-var-zero-values`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Add explicit typed `var` declarations so Go source can introduce zero-valued locals without forcing an initializer expression.
- Extend the runtime slice model with first-class nil-slice zero values instead of requiring slice literals or reslices as the only construction path.
- Keep the change layered across parser, semantic analysis, bytecode, VM execution, tests, and roadmap/docs so later allocation work can build on the same seams.

## Scope

- Extend parsing and AST rendering for `var name T` and `var name T = expr`.
- Synthesize zero values for declared locals across `int`, `bool`, `string`, and `[]T`, including nil-slice runtime representation.
- Make nil slices work with the current slice surface: rendering, `len`, `cap`, `copy`, `append`, indexing diagnostics, and slice expressions where valid.
- Add a realistic CLI example and layered tests for typed declarations and nil-slice behavior.
- Sync research, design, tech, roadmap, verification, experience, and `BOOT.md`.

## Non-Goals

- The `make` builtin or any type-valued call arguments
- Untyped `nil` expressions, conversions, or slice equality with `nil`
- Array, map, channel, or pointer zero values
- Full Go compile-time constant diagnostics for zero-value-related bounds and allocation checks

## Phase Breakdown

1. Open the active `M3` plan and update roadmap indexes.
2. Extend the existing slice research baseline with variable declaration zero-value and nil-slice semantics.
3. Implement typed `var` declarations, synthesized zero values, and nil-slice runtime behavior across the frontend, semantic layer, bytecode, and VM.
4. Add example coverage, focused unit tests, CLI integration tests, and serial CLI validation for happy-path and diagnostic behavior.
5. Sync docs, archive the completed plan if done, and commit plus push the full working tree.

## Acceptance Criteria

- `cargo test` passes with new coverage for typed declarations and nil-slice zero-value behavior.
- `cargo run -- run` executes a realistic example that declares zero-valued locals and grows a nil slice through the CLI.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` expose the new declaration and zero-value path clearly enough for debugging.
- `cargo run -- check` rejects at least one invalid typed-declaration or nil-slice misuse with a targeted diagnostic.
- Research, design, tech, verification, experience, roadmap, and `BOOT.md` all describe the new zero-value surface and its explicit limits.

## Risks

- Zero-value synthesis can silently diverge between semantic typing and VM execution if local declarations do not share one representation.
- Nil slices and empty slices have overlapping observable behavior in the current subset, so the runtime boundary must stay explicit for later compatibility work.
- Typed variable declarations can complicate inference and assignment checks unless inferred and explicit declaration paths stay centralized.
