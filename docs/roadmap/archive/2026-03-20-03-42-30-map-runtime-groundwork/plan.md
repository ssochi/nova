# Plan: Map Runtime Groundwork

## Basic Information

- Plan ID: `2026-03-20-03-42-30-map-runtime-groundwork`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add the first map runtime slice to the language surface with explicit type syntax and VM-backed execution.
- Extend builtin and indexing behavior so maps become usable from the CLI path without collapsing type-valued syntax into ad hoc runtime checks.
- Leave layered documentation, tests, and validation traces that make broader map work resumable.

## Scope

- Parse and render `map[K]V` type syntax in declarations and `make(map[K]V[, hint])`.
- Support typed zero-value map declarations, `len(map)`, map indexing, and map index assignment.
- Lower map construction and access into explicit checked and bytecode paths, then execute them in the VM.
- Add CLI examples, automated tests, verification, and experience reports for the new map surface.

## Non-Goals

- Map literals, `delete`, `range`, comma-ok map lookups, or channel support.
- Keys beyond the currently comparable scalar set already modeled by the compiler runtime.
- Real Go hash iteration order or backend-oriented map lowering.

## Phase Breakdown

1. Research and design the first staged Go-compatible map surface for `make`, indexing, and zero values.
2. Extend frontend and semantic layers with map type refs, checked expressions, and builtin validation.
3. Add bytecode/runtime support for map allocation, indexing, assignment, and display.
4. Validate via unit tests plus serial CLI `run`, `dump-ast`, `dump-bytecode`, and diagnostic coverage.
5. Sync roadmap, docs, and reports, then archive the plan if the slice is complete.

## Acceptance Criteria

- `var counts map[string]int` and `var counts = make(map[string]int)` are accepted and behave coherently.
- `len(counts)`, `counts["key"]`, and `counts["key"] = value` work through semantic analysis, lowering, and VM execution.
- Unsupported key types or invalid map operations fail with targeted diagnostics.
- CLI inspection surfaces expose map syntax and the new execution path without reading implementation code.
- The plan, docs, and validation evidence leave a clear entry point for the next `M3` runtime expansion.

## Risks

- Map behavior introduces a second composite runtime category, which can sprawl if slice-specific and map-specific logic are not kept separate.
- Key comparability rules must stay centralized; otherwise parser, semantic, and runtime layers may drift.
- Nil map reads and writes need explicit handling so the VM surfaces Go-like behavior instead of accidental Rust container semantics.
