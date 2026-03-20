# Plan: Builtin Clear and Runtime File Split

## Basic Information

- Plan ID: `2026-03-20-10-46-29-builtin-clear-and-runtime-file-split`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Add builtin `clear` for `slice` and `map` values across semantic analysis, bytecode lowering, and VM execution.
- Preserve observable Go semantics around nil no-op behavior, slice-window mutation, and shared map aliasing.
- Reduce pressure on runtime-heavy test files so this feature does not push the repository past the file-size ceiling.

## Scope

- Add builtin metadata and semantic contracts for `clear(slice|map)`.
- Extend runtime values and VM builtin dispatch for slice/map clearing semantics.
- Add example coverage plus focused unit and CLI tests for success and diagnostic paths.
- Split runtime-heavy tests into focused submodules/files where needed to keep hot files under the repository ceiling.
- Update research, technical docs, reports, roadmap state, and handoff context when the slice is complete.

## Non-Goals

- Generics-aware `clear` behavior on type parameters.
- New package-backed standard-library APIs in the same round.
- Broad runtime redesign beyond the narrow file split needed for maintainability.

## Phase Breakdown

1. Record the Go behavior baseline for builtin `clear` and open the active `M3` plan.
2. Extend builtin identity, semantic validation, and explicit lowering visibility.
3. Implement runtime slice/map clearing helpers and wire them through VM builtin dispatch.
4. Add focused unit tests, CLI tests, and a CLI example while splitting runtime-heavy test files as needed.
5. Run formatting, automated tests, serial CLI validation, then update docs, reports, roadmap state, and archive the plan if complete.

## Acceptance Criteria

- `clear` type-checks only for `slice` and `map` arguments and fails with direct diagnostics on unsupported types.
- `clear` executes as a visible builtin call in `dump-bytecode` and preserves nil no-op plus aliasing behavior.
- Slice clearing zeroes only the visible range, preserving length, capacity, and nil state.
- Runtime/value and VM test organization stays within the repository file-size ceiling after the change.
- Documentation and plan artifacts explain both the new builtin surface and the runtime/file-organization implications.

## Risks

- Slice clearing needs correct zero-value synthesis for all currently supported element types, not just integers or bytes.
- Mutating shared slice/map storage is easy to get subtly wrong if helpers rebuild values instead of operating in place.
- File splitting can create awkward module boundaries if done hastily, so the refactor must stay narrow and test-oriented.
