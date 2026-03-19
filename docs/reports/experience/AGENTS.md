# reports/experience Directory Conventions

This directory stores experience validation reports close to the real user path.

## Directory Responsibilities

- Record real entry points and actual operation paths
- Record positive experience, issues, severity, and conclusions
- Record the degree to which the current project holds up in real use

## When It Must Be Updated

- When adding a formal experience report
- When the naming convention or minimum fields of the experience report change
- When the current project first fills in a real experience entry point

## File Format Convention

- Report files use `YYYY-MM-DD-<topic>.md`
- Reports must include at least: entry point and path, key observations, positive experience, issues and severity, conclusion, and next recommended steps
- If the project currently has no real experience entry point, do not generate an empty report; explain the gap in the validation report or plan

## Document Structure

- Each experience report must always include:
  - `# <Title>`
  - `## Basic Context`
  - `## Experience Path`
  - `## Positive Experience`
  - `## Issues and Severity`
  - `## Conclusion and Next Recommended Steps`
- If this round's experience focuses on a sub-scenario, the scope boundary may be stated under `## Experience Path`

## File Index

- `AGENTS.md`: this directory convention
- `2026-03-19-bootstrap-cli-playtest.md`: records the bootstrap CLI happy path and error path experience
- `2026-03-19-semantic-functions-branches-playtest.md`: records the multi-function and semantic-error CLI walkthrough
- `2026-03-19-m2-loop-closeout-playtest.md`: records the milestone-closeout CLI walkthrough for branches and loops
- `2026-03-20-string-runtime-builtins-playtest.md`: records the string and builtin CLI walkthrough for the first `M3` slice
- `2026-03-20-import-fmt-seam-playtest.md`: records the CLI walkthrough for import declarations and the first `fmt` package seam
- `2026-03-20-slice-runtime-testing-playtest.md`: records the CLI walkthrough for slice literals, indexing, and the layered test upgrade
- `2026-03-20-strings-package-contracts-playtest.md`: records the CLI walkthrough for typed `strings` package contracts
- `2026-03-20-slice-expressions-and-assignment-playtest.md`: records the CLI walkthrough for slice windows and indexed slice updates
- `2026-03-20-slice-builtins-capacity-playtest.md`: records the CLI walkthrough for `cap`, `copy`, and capacity-aware append reuse
- `2026-03-20-typed-zero-values-playtest.md`: records the CLI walkthrough for explicit typed `var` declarations and nil-slice zero values
- `2026-03-20-make-slice-allocation-playtest.md`: records the CLI walkthrough for slice allocation through builtin `make`
- `2026-03-20-byte-strings-and-slicing-playtest.md`: records the CLI walkthrough for `byte`, string indexing/slicing, and `copy([]byte, string)`
- `2026-03-20-string-byte-conversions-playtest.md`: records the CLI walkthrough for explicit `[]byte(string)` and `string([]byte)` conversions
