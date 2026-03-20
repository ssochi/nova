# Plan: Strings and Bytes Clone Seams

## Basic Information

- Plan ID: `2026-03-20-10-32-34-strings-bytes-clone-seams`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Expand the metadata-backed `strings` and `bytes` package seams with `Clone` helpers that fit the current byte-oriented runtime model.
- Preserve observable Go semantics around `bytes.Clone(nil)`, non-nil empty slices, and `strings.Clone("")`.
- Keep package contracts, lowering visibility, and VM dispatch centralized and debuggable.

## Scope

- Add package contracts, lowering visibility, and VM execution for:
  - `strings.Clone`
  - `bytes.Clone`
- Record the official behavior baseline in `docs/research/`.
- Add an example program plus focused semantic, runtime, and CLI coverage.
- Update roadmap, technical docs, verification/experience reports, and handoff context when the slice is complete.

## Non-Goals

- Pointer-identity or allocation-observability guarantees beyond the behavior visible through the current VM.
- Rune-aware or UTF-8-sequence-aware `strings` / `bytes` helpers.
- General refactors of the package system unless a narrow extraction is required to stay under file-size limits.

## Phase Breakdown

1. Open the active `M3` plan and record the Go behavior baseline for both `Clone` helpers.
2. Extend shared package metadata and semantic package contracts.
3. Extend VM package dispatch and runtime helpers for string and byte-slice cloning semantics.
4. Add examples, focused tests, and serial CLI validation coverage.
5. Update reports, technical docs, milestone state, and archive the plan when complete.

## Acceptance Criteria

- `strings.Clone` and `bytes.Clone` type-check, lower, and execute through the existing CLI and VM flow.
- `bytes.Clone(nil)` stays nil while cloning a non-nil empty slice remains non-nil.
- `dump-ast`, `dump-bytecode`, and `check` all expose the new package calls without hidden lowering.
- Repository documentation and plan artifacts reflect the new slice and its deferred boundaries.

## Risks

- `bytes.Clone` nil-vs-empty behavior is observable and easy to regress if the helper reuses the wrong slice-construction path.
- `strings.Clone` is semantically simple but still needs to stay explicit so the package surface does not drift from docs and validation.
- `src/semantic/packages.rs` is already near the repository size ceiling, so this slice must avoid unnecessary sprawl.
