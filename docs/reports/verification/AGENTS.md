# reports/verification Directory Conventions

This directory stores validation evidence that "proves the change holds."

## Directory Responsibilities

- Record automated test results
- Record command checks, structure checks, and failure reasons
- Record blockers and remaining risks when validation cannot be completed

## When It Must Be Updated

- When adding a formal validation report
- When the validation report template, naming rules, or minimum fields change
- When temporary validation in a plan needs to be formalized into an official report

## File Format Convention

- Report files use `YYYY-MM-DD-<topic>.md`
- Reports must include at least: date, goal, execution method, result, and remaining risks
- If only lightweight validation was performed in this round, a trace should also be left in the plan `context.md`

## Document Structure

- Each validation report must always include:
  - `# <Title>`
  - `## Basic Information`
  - `## Validation Goal`
  - `## Execution Method`
  - `## Results`
  - `## Remaining Risks`
- If validation fails, `## Blockers` may be added after `## Results`

## File Index

- `AGENTS.md`: this directory convention
- `2026-03-19-bootstrap-vm-foundation.md`: validates the bootstrap CLI, VM, and test loop
- `2026-03-19-semantic-functions-branches.md`: validates semantic analysis, user-defined calls, and branch execution
- `2026-03-19-for-loops-path-analysis.md`: validates loop parsing, semantic analysis, lowering, and VM execution
- `2026-03-20-string-runtime-builtins.md`: validates string runtime values, builtin contracts, and CLI inspection
- `2026-03-20-import-fmt-seam.md`: validates import declarations, selector calls, and the first package-backed `fmt` seam
- `2026-03-20-slice-runtime-testing.md`: validates slice runtime values, builtin expansion, and layered automated coverage
- `2026-03-20-strings-package-contracts.md`: validates typed package contracts, the `strings` seam, and CLI inspection
- `2026-03-20-slice-expressions-and-assignment.md`: validates simple slice expressions, shared slice windows, and indexed slice assignment
- `2026-03-20-slice-builtins-capacity.md`: validates `cap`, `copy`, and capacity-aware append reuse across tests and CLI inspection
- `2026-03-20-typed-var-zero-values.md`: validates explicit typed `var` declarations, synthesized zero values, and nil-slice runtime behavior
- `2026-03-20-make-slice-allocation.md`: validates `make([]T, len[, cap])`, zero-filled spare capacity, and the explicit allocation bytecode path
