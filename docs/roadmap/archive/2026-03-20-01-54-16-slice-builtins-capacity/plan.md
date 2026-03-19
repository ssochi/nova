# Plan: Slice Builtins and Capacity-Aware Append

## Basic Information

- Plan ID: `2026-03-20-01-54-16-slice-builtins-capacity`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Expand the current slice runtime model with builtin helpers that real Go code expects early: `cap` and `copy`.
- Upgrade `append` so slice growth can reuse shared backing storage when spare capacity exists.
- Keep the implementation narrow, explicit, and layered so later runtime work can build on the same slice storage model.

## Scope

- Capture official Go behavior for `cap`, `copy`, and slice-backed `append` reuse under `docs/research/`.
- Extend builtin contracts and VM execution for `cap(slice)` and `copy(dstSlice, srcSlice)`.
- Change slice append behavior so appends within capacity reuse backing storage and remain observable through overlapping slice views.
- Add a realistic CLI example plus layered automated and serial CLI validation for the new slice behavior.
- Sync design, tech, roadmap, verification, experience, and `BOOT.md` after implementation.

## Non-Goals

- String slicing or byte-addressed string runtime changes
- `make`, `nil`, variadic forwarding with `...`, or array / pointer-to-array operands for `cap` and `copy`
- Full parity with Go runtime growth heuristics when append must allocate a new backing store
- Broader package-backed standard library work outside the chosen slice builtin surface

## Phase Breakdown

1. Open the new active `M3` plan and sync roadmap indexes.
2. Capture official builtin semantics and refresh the slice runtime design boundary.
3. Implement `cap`, `copy`, and capacity-aware append reuse across semantic validation and VM execution.
4. Add examples, focused unit tests, CLI integration tests, and serial CLI validation for happy-path and error-path behavior.
5. Sync docs, archive the completed plan if done, and commit plus push the full working tree.

## Acceptance Criteria

- `cargo test` passes with new coverage for `cap`, `copy`, and append reuse behavior.
- `cargo run -- run` executes a realistic example that demonstrates both `cap` and copy/append effects through the real CLI.
- `check` rejects at least one invalid `cap` or `copy` call with a targeted diagnostic.
- Research, design, tech, verification, experience, roadmap, and boot artifacts all describe the new runtime slice surface and its explicit limits.
- The implementation preserves centralized builtin contracts and extends the existing shared-slice runtime model instead of introducing ad hoc special cases.

## Risks

- `copy` overlap semantics can be wrong if the VM mutates through the same storage without a temporary snapshot.
- Capacity-aware append can silently regress prior slice-window behavior if the shared-storage model is updated inconsistently.
- Supporting only slice operands for `cap` and `copy` must stay documented clearly so partial Go compatibility is explicit.
