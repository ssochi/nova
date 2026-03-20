# Plan: Strings and Bytes LastIndex / Byte Search Seams

## Basic Information

- Plan ID: `2026-03-20-10-08-16-strings-bytes-last-index-byte-search`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Expand the metadata-backed `strings` and `bytes` seams with more realistic byte-oriented search helpers that match the current VM/runtime model.
- Improve package compatibility without claiming rune-aware or UTF-8-sequence-aware behavior the project does not yet model.
- Keep the debug path readable so `dump-ast`, `dump-bytecode`, and `check` all expose the new slice clearly.

## Scope

- Add package contracts, lowering visibility, and VM execution for:
  - `strings.LastIndex`
  - `strings.IndexByte`
  - `strings.LastIndexByte`
  - `bytes.LastIndex`
  - `bytes.IndexByte`
  - `bytes.LastIndexByte`
- Add a new example program plus focused unit, semantic, CLI execution, and CLI diagnostic coverage.
- Keep test growth layered by adding new integration test files instead of extending the largest existing files.

## Non-Goals

- Unicode-aware or rune-aware helpers such as `IndexAny`, `LastIndexAny`, or split-family APIs.
- New syntax, new AST node kinds, or broader import/package-loading infrastructure.
- Interface-backed package APIs, formatting improvements, or scheduler/runtime concurrency work.

## Phase Breakdown

1. Record the official compatibility baseline and open the active `M3` plan.
2. Extend shared package metadata and semantic package contracts for the chosen search helpers.
3. Extend VM package dispatch and runtime helper code for the new string and byte-slice searches.
4. Add example coverage, focused automated tests, and serial CLI validation traces.
5. Update docs/reports/roadmap state, archive the plan if complete, and commit the working tree.

## Acceptance Criteria

- All six helpers type-check, lower, and execute through the existing CLI and VM path.
- `dump-ast` and `dump-bytecode` make the new package calls visible without reading implementation code.
- Invalid argument shapes fail during `check` with direct package-argument diagnostics.
- Documentation and roadmap artifacts are synchronized so the next agent can continue from repository state alone.

## Risks

- Package compatibility can overstate progress if byte-only helpers drift into Unicode-sensitive APIs without deliberate research.
- Package metadata, semantic contracts, and VM dispatch can drift unless every helper is wired through the centralized tables.
- Test coverage can become harder to maintain if new CLI cases keep accumulating in the largest existing integration files.
