# Plan: Strings and Bytes Compare Seams

## Basic Information

- Plan ID: `2026-03-20-10-21-19-strings-bytes-compare-seams`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Expand the metadata-backed `strings` and `bytes` package seams with lexicographic comparison helpers that fit the current byte-oriented runtime model.
- Improve standard-library compatibility without claiming Unicode- or rune-aware behavior the VM does not yet model.
- Keep the existing package contract, lowering, and `call-package` runtime path centralized and inspectable.

## Scope

- Add package contracts, lowering visibility, and VM execution for:
  - `strings.Compare`
  - `bytes.Compare`
- Add a focused research note, design note, example program, unit tests, CLI integration tests, and serial CLI validation traces.
- Update roadmap, tech, report, and handoff documents so the next agent can continue from repository state alone.

## Non-Goals

- Unicode-aware or case-folding helpers such as `EqualFold`, `IndexAny`, `LastIndexAny`, or split-family APIs.
- New syntax, import-loading changes, or general comparison-operator rewrites.
- Panic-accurate runtime failure modeling beyond the current staged VM error surface.

## Phase Breakdown

1. Open the active `M3` plan and record the Go behavior baseline for `Compare`.
2. Extend shared package metadata and semantic package contracts for both helpers.
3. Extend VM package dispatch and byte-slice helper logic for lexicographic compare behavior.
4. Add examples, focused automated tests, and serial CLI validation coverage.
5. Update reports, technical docs, roadmap state, and archive the plan when complete.

## Acceptance Criteria

- `strings.Compare` and `bytes.Compare` type-check, lower, and execute through the existing CLI and VM flow.
- `bytes.Compare` preserves the Go rule that nil and empty slices compare as equal.
- `dump-ast`, `dump-bytecode`, and `check` all expose the new package calls without hidden lowering.
- Repository documentation and plan artifacts reflect the new slice and its deferred boundaries.

## Risks

- Package seams can drift if semantic contracts, shared package metadata, and VM dispatch are not updated together.
- Byte-oriented comparison behavior must stay explicit so the project does not overstate broader string compatibility.
- Adding new CLI coverage to the large umbrella integration files would increase file-size pressure unnecessarily.
