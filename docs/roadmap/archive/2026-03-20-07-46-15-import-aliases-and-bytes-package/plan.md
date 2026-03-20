# Plan: Import Aliases and Bytes Package Seam

## Basic Information

- Plan ID: `2026-03-20-07-46-15-import-aliases-and-bytes-package`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Expand import parsing and semantic binding so the CLI can accept grouped imports and explicit import aliases without introducing a real filesystem package graph.
- Add a new metadata-backed `bytes` package seam that reuses the current byte-slice runtime model and improves real-project compatibility.
- Preserve explicit AST, semantic, bytecode, and runtime seams so `dump-ast` and `dump-bytecode` stay useful.

## Scope

- Grouped imports such as `import ("fmt"; "bytes")`
- Explicit import aliases such as `import b "bytes"`
- Shared package identity updates for the new `bytes` package and alias-aware binding lookup
- Typed package contracts and VM support for a staged `bytes` subset that fits the current runtime:
  - `bytes.Equal([]byte, []byte) -> bool`
  - `bytes.Contains([]byte, []byte) -> bool`
  - `bytes.HasPrefix([]byte, []byte) -> bool`
  - `bytes.Join([][]byte, []byte) -> []byte`
  - `bytes.Repeat([]byte, int) -> []byte`
- Examples, automated coverage, CLI validation, and documentation updates for the expanded import/package surface

## Non-Goals

- Dot imports, blank imports, or filesystem-backed package loading
- `bytes` APIs that require interfaces, errors, runes, or broader multi-result support
- Changing the current byte-oriented string model
- General selector expressions beyond imported package call targets

## Phase Breakdown

1. Research Go import declaration forms and the selected `bytes` APIs from official sources.
2. Record the staged design for alias-aware imports and the new `bytes` seam.
3. Extend AST, parser, and semantic import binding logic for grouped imports plus aliases.
4. Add shared package identities, semantic package contracts, runtime dispatch, examples, and tests for the staged `bytes` functions.
5. Run formatting, tests, CLI inspection, and file-size checks; then update reports and archive the plan if complete.

## Acceptance Criteria

- `check`, `dump-ast`, `dump-bytecode`, and `run` all handle at least one example that combines grouped imports, alias imports, and `bytes` package calls.
- Import aliases resolve through semantic analysis, and unsupported alias/package/member errors remain targeted.
- `bytes` functions validate types centrally in `src/semantic/packages.rs` and execute through package-function dispatch instead of ad hoc analyzer/runtime special cases.
- New docs, reports, and roadmap records leave enough context for the next trigger to continue from the staged import/package model.

## Risks

- Import syntax growth can become misleading if dot/blank imports appear half-supported; diagnostics need to stay explicit.
- `bytes.Join` needs nested byte-slice handling without turning this round into a general slice-of-slice feature sprint.
- Nil and empty byte-slice behavior differs across APIs; the staged subset must document any deliberate approximation clearly.
