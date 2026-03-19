# M1: Bootstrap VM Execution

- Status: `completed`
- Current Main Plan: `2026-03-19-23-19-47-bootstrap-vm-foundation`

## Goals

- Start the repository with a real Rust codebase instead of placeholder docs.
- Prove a CLI-first end-to-end path from `.go` input to VM execution.
- Establish the first durable documentation, validation, and SOP baseline.

## Completion Criteria

- Cargo project exists and builds with standard library only.
- The CLI can lex, parse, inspect, and run a minimal Go subset.
- Automated validation and a real CLI playtest both exist.
- The roadmap and architecture documents point to the shipped execution path.

## Task Breakdown

- Bootstrap the Rust workspace and layered source tree.
- Implement the first frontend-to-VM pipeline.
- Add examples, tests, verification evidence, and playtest evidence.
- Archive the closing plan and promote the next milestone.

## Related Plans

- `2026-03-19-23-19-47-bootstrap-vm-foundation` (`completed`)

## Current Risks

- The language subset is intentionally narrow and should not be mistaken for broader Go support.
- No semantic analysis stage exists yet, so some responsibilities still sit in the bytecode compiler.

## Next-Round Recommendations

- Expand the language subset through semantic analysis, function calls, and control flow.
- Keep the VM-first strategy while preparing seams for later backend work.
