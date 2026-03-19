# Plan: Map Literals and Delete

## Basic Information

- Plan ID: `2026-03-20-04-07-11-map-literals-delete`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Make staged maps materially more usable by supporting direct `map[K]V{...}` construction.
- Add the builtin `delete(map, key)` with explicit semantic and runtime behavior instead of hiding it behind ad hoc map mutation paths.
- Preserve layered debug visibility through AST, checked expressions, bytecode, CLI validation, and resumable documentation.

## Scope

- Parse and render `map[K]V{key: value}` literals, including the empty literal form.
- Validate map literal key/value types centrally and lower literals through explicit checked and bytecode forms.
- Add builtin `delete(map, key)` with Go-like nil-map behavior and typed diagnostics.
- Extend runtime map storage with explicit entry removal while keeping nil-vs-allocated state visible.
- Add examples, automated tests, serial CLI validation, and updated reports for the new map usability slice.

## Non-Goals

- Comma-ok map lookups, `range`, map equality with `nil`, or multi-value assignments.
- Channel syntax or runtime support.
- General composite literal support beyond the staged map literal form.

## Phase Breakdown

1. Extend the existing map research/design baseline with official literal and `delete` semantics, then register the active plan.
2. Add frontend support for map literal syntax while keeping typed conversion and `make` parsing explicit.
3. Add semantic, bytecode, and VM support for map literals and `delete`.
4. Validate with focused tests plus serial CLI `run`, `dump-ast`, `dump-bytecode`, and `check` coverage for happy and failure paths.
5. Sync roadmap, tech/design docs, reports, and startup guidance; archive the plan if the slice is complete.

## Acceptance Criteria

- `map[string]int{"nova": 1, "go": 2}` and `map[string]int{}` parse, type-check, and execute through the VM.
- Duplicate key assignments inside a literal behave coherently through the lowered runtime path.
- `delete(counts, "nova")` removes an entry, and `delete(nilMap, key)` is a no-op rather than a runtime failure.
- Invalid key/value types in map literals or invalid `delete` calls fail with targeted diagnostics during semantic analysis.
- CLI debug surfaces expose map literal construction and deletion clearly enough to debug without reading Rust implementation code.

## Risks

- Map literals introduce a typed composite literal path that can sprawl into broader composite-literal ambitions unless kept narrowly staged.
- `delete` is a void builtin acting on mutable shared state, so analyzer, lowering, and VM stack behavior must stay aligned.
- Parser growth is close to the repository file-size ceiling, so tests or helpers may need to be split while landing the feature.
