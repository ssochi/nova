# Plan: Slice Expressions and Element Assignment

## Basic Information

- Plan ID: `2026-03-20-01-33-44-slice-expressions-assignment`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Expand the current slice runtime model from literals and indexing to real slice-window operations.
- Add slice element assignment so slice-bearing programs can update composite state instead of rebuilding values only through `append`.
- Keep the slice semantics narrow, explicit, and aligned with official Go behavior where the current VM can model it cleanly.

## Scope

- Add parsing support for two-index slice expressions such as `values[1:3]`, `values[:3]`, and `values[1:]`.
- Add semantic analysis, lowering, and VM execution for slice expressions on `[]T`.
- Add semantic analysis, lowering, and VM execution for slice element assignment such as `values[0] = 5`.
- Capture the official behavior baseline for the selected slice semantics under `docs/research/`.
- Add example programs, layered automated tests, and serial CLI validation for the new slice surface.
- Sync roadmap, design, tech, validation, experience, and boot documents for the new runtime slice.

## Non-Goals

- Three-index full slice expressions
- String slice execution, `cap`, `make`, `copy`, `nil`, or backing-array capacity semantics
- General lvalue support beyond local slice variables
- Full Go aliasing guarantees for independently stored nested composite values

## Phase Breakdown

1. Open the next active `M3` plan and record the slice-expression plus assignment scope.
2. Capture official-behavior research and refresh the slice design boundary.
3. Extend frontend, semantic analysis, bytecode, and VM execution for slice expressions and slice element assignment.
4. Add examples, unit tests, CLI integration tests, and serial CLI validation for happy-path and error-path behavior.
5. Sync docs, update milestone and plan state, archive the plan if complete, and commit the full working tree.

## Acceptance Criteria

- `cargo test` passes with new unit and CLI coverage for slice expressions and slice element assignment.
- `cargo run -- run` executes at least one realistic program using slice windows and in-place slice updates.
- `check` rejects at least one invalid slice expression or slice assignment with a targeted diagnostic.
- Research, design, tech, verification, experience, and roadmap artifacts all describe the new slice surface and its explicit limits.
- The implementation remains layered without introducing slice-specific semantic checks directly into lowering or ad hoc runtime-only validation.

## Risks

- Slice expressions can sprawl into partial Go compatibility if unsupported forms such as full slices are not rejected clearly.
- Element assignment may imply aliasing guarantees that the current cloned-value VM cannot fully honor; the implemented contract must stay explicit.
- Adding new slice bytecode instructions without shared helpers could make later composite-value work harder to extend.
