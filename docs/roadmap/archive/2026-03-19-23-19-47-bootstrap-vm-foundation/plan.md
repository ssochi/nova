# Plan: Bootstrap VM Foundation

## Basic Information

- Plan ID: `2026-03-19-23-19-47-bootstrap-vm-foundation`
- Milestone: `M1-bootstrap-vm-execution`
- Status: `completed`

## Goals

- Initialize the Rust project and establish a real CLI-first execution surface.
- Ship the first minimal Go-to-bytecode-to-VM loop.
- Leave enough roadmap, architecture, SOP, and validation evidence for the next round.

## Scope

- Cargo project bootstrap
- CLI command parsing
- Minimal frontend, bytecode compiler, and VM runtime
- Example inputs, tests, and reports
- Milestone and SOP initialization

## Non-Goals

- Full Go compatibility
- Native backend code generation
- User-defined function calls, heap values, or standard library support beyond the bootstrap builtin

## Phase Breakdown

1. Create milestone and plan artifacts for the cold-start repository.
2. Bootstrap the Rust crate and layered modules.
3. Implement the minimal language slice and VM execution path.
4. Validate through tests and real CLI commands.
5. Synchronize milestone, roadmap, SOP, and report documents.

## Acceptance Criteria

- `cargo test` passes.
- The CLI can run `examples/hello.go` end to end.
- A formal milestone, archived plan, verification report, and experience report exist.
- Architecture and SOP documents reflect the shipped execution path.

## Risks

- The bootstrap slice may bias future parser and VM structure if extension seams are not documented.
- Language coverage is intentionally narrow, so user expectations must be managed through docs and diagnostics.
