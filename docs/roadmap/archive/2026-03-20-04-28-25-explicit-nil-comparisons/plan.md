# Plan: Explicit Nil Comparisons

## Basic Information

- Plan ID: `2026-03-20-04-28-25-explicit-nil-comparisons`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `in_progress`
- Owner: primary agent

## Goals

- Add the predeclared `nil` literal to the language surface for currently modeled composite reference-like types.
- Support Go-like assignment, return, call-argument, and equality behavior for `nil` with slices and maps.
- Keep the new `nil` path explicit through AST, checked model, bytecode, and VM debug output.

## Scope

- Parse `nil` as an expression and render it through `dump-tokens` and `dump-ast`.
- Extend semantic analysis with an explicit untyped-`nil` representation plus assignability rules for `[]T` and `map[K]V`.
- Support `==` and `!=` between `nil` and slice/map values, while continuing to reject unsupported equality cases.
- Add CLI examples, diagnostics, unit coverage, and reports for explicit `nil` usage.
- Refresh roadmap, design, research, and technical documentation tied to the new surface.

## Non-Goals

- General multi-value expressions such as comma-ok map lookups.
- `chan`, pointer, interface, function, or `nil`-typed package seams beyond the currently modeled slice/map surface.
- Making `nil == nil` legal; keep Go's untyped-`nil` restriction explicit.
- Broad composite equality beyond `slice/map` compared with `nil`.

## Phase Breakdown

1. Lock compatibility scope with a focused `nil` research note and update the map/slice design baseline.
2. Implement parser, semantic, bytecode, and VM support for explicit `nil` expressions and composite-nil equality.
3. Add examples, tests, CLI validation, and synced reports.
4. Update roadmap/context artifacts, then archive or hand off the plan depending on completion state.

## Acceptance Criteria

- `cargo test` passes with new parser, semantic, VM, and CLI coverage for explicit `nil`.
- `cargo run -- run` executes a realistic example that assigns, compares, and passes `nil` slices/maps through the CLI path.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` make the new `nil` execution path visible without reading implementation code.
- `cargo run -- check` rejects at least one invalid untyped-`nil` use with a targeted diagnostic.
- Research, design, milestone, plan, verification, and experience docs all reflect the new scope and remaining gaps.

## Risks

- Untyped `nil` can sprawl into ad hoc exceptions unless assignability and equality rules stay centralized.
- Slice and map runtime values already distinguish nil from allocated state internally; source-level `nil` must preserve that boundary without silently normalizing to empty values.
- File-size pressure is highest in `src/semantic/analyzer.rs` and `src/runtime/vm.rs`; if the feature grows unexpectedly, supporting helpers or tests may need to move into submodules in the same round.
