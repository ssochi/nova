# Plan: Strings and Bytes Index/Trim Seams

## Basic Information

- Plan ID: `2026-03-20-09-49-02-strings-bytes-index-trim`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Expand the metadata-backed `strings` and `bytes` seams with a realistic search/trim helper slice that works within the current byte-oriented runtime model.
- Improve standard-library compatibility without claiming unsupported UTF-8/rune-sensitive behavior.
- Keep nil-vs-empty byte-slice behavior explicit so package helpers remain trustworthy for later standard-library work.

## Scope

- Add package contracts, lowering visibility, and VM execution for:
  - `strings.Index`
  - `strings.HasSuffix`
  - `strings.TrimPrefix`
  - `strings.TrimSuffix`
  - `bytes.Index`
  - `bytes.HasSuffix`
  - `bytes.TrimPrefix`
  - `bytes.TrimSuffix`
- Add examples, unit tests, CLI integration coverage, serial CLI validation, and roadmap/report/doc synchronization.

## Non-Goals

- `Split`, `SplitN`, or any empty-separator behavior that depends on UTF-8 sequence semantics.
- New syntax, AST nodes, or checked-model call shapes.
- Broader `fmt` formatting, filesystem-backed import graphs, or interface-backed package APIs.

## Phase Breakdown

1. Record the compatibility baseline for the chosen search/trim helpers and open the active `M3` plan.
2. Extend shared package metadata and semantic package contracts for the new `strings` / `bytes` functions.
3. Extend VM package dispatch plus runtime helper code, preserving byte-slice nil/view behavior where Go does.
4. Add examples, automated coverage, CLI validation, and documentation/report updates.
5. Archive the plan if all acceptance criteria pass.

## Acceptance Criteria

- `strings.Index`, `strings.HasSuffix`, `strings.TrimPrefix`, and `strings.TrimSuffix` check, lower, and execute through the CLI and VM path.
- `bytes.Index`, `bytes.HasSuffix`, `bytes.TrimPrefix`, and `bytes.TrimSuffix` check, lower, and execute through the CLI and VM path.
- `bytes.TrimPrefix` / `bytes.TrimSuffix` preserve staged nil-vs-empty behavior and shared slice views instead of forcing copies.
- Invalid package calls fail during semantic analysis with targeted argument diagnostics.

## Risks

- The package surface can sprawl into rune-sensitive helpers unless the scope stays aligned with the current byte-oriented runtime model.
- Byte-slice trim helpers can accidentally erase nilness or shared-backing behavior if they materialize fresh slices eagerly.
- Package metadata, semantic contracts, and runtime dispatch can drift unless every new helper is wired through the same centralized tables.
