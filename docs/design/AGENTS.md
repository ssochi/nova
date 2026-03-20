# design Directory Conventions

This directory stores feature and subsystem designs. It describes intent, scope boundaries, staged choices, and design trade-offs without replacing roadmap execution records or low-level technical references.

## Directory Responsibilities

- Capture feature-level designs before or alongside implementation
- Record scope boundaries, assumptions, and staged decisions
- Link design decisions to related milestone and plan work

## When It Must Be Updated

- When a new subsystem or feature direction needs a design baseline
- When a shipped implementation changes the design boundary or invalidates an assumption
- When a design becomes important enough to serve as a later handoff interface

## File Format Convention

- Design files use kebab-case, such as `bootstrap-vm-execution.md`
- Each design should state its goal, constraints, current scope, deferred scope, and follow-up hooks

## Document Structure

- Each design document should include:
  - `# <Design Topic>`
  - `## Goal`
  - `## Constraints`
  - `## Current Scope`
  - `## Deferred Scope`
  - `## Interfaces and Extension Hooks`

## File Index

- `AGENTS.md`: this directory convention
- `strings-bytes-compare-seams.md`: design baseline for staged `strings.Compare` / `bytes.Compare` package helpers
- `multi-result-functions-and-cut-package-seams.md`: design baseline for unnamed multi-result functions, staged binding forms, and `strings.Cut` / `bytes.Cut`
- `import-aliases-and-bytes-package.md`: design baseline for grouped imports, explicit import aliases, and the staged `bytes` package seam
- `switch-statements.md`: design baseline for staged expression `switch` statements, tagless `switch`, clause scopes, and single-evaluation lowering
- `if-statement-headers.md`: design baseline for staged `if` statement initializers, shared header scope, and `else if` chains
- `bootstrap-vm-execution.md`: design baseline for the bootstrap VM execution loop
- `semantic-functions-branches.md`: design baseline for semantic analysis, user-defined calls, and branch control flow
- `for-loop-control-flow.md`: design baseline for condition-only `for` loops and loop-aware path analysis
- `string-runtime-builtins.md`: design baseline for string values, builtin contract centralization, and the first `M3` runtime slice
- `import-fmt-package-seam.md`: design baseline for top-level imports and the first package-backed `fmt` seam
- `slice-runtime-values.md`: design baseline for slice runtime values, typed zero-value declarations, `make` allocation, shared storage, helper builtins, and layered test coverage
- `byte-oriented-strings.md`: design baseline for byte-oriented runtime strings, string indexing/slicing, and the first `byte`-specialized builtin seam
- `string-byte-conversions.md`: design baseline for first-class `[]byte(string)` and `string([]byte)` conversion expressions
- `strings-package-contracts.md`: design baseline for typed package contracts and the first `strings` package seam
- `map-runtime-groundwork.md`: design baseline for staged `map[K]V` support with explicit make/index/assignment execution paths
- `explicit-nil-comparisons.md`: design baseline for explicit `nil` expressions and `slice/map` nil comparisons
- `simple-statements.md`: design baseline for staged short declarations plus explicit `++` / `--` as simple statements
- `channels-first-slice.md`: design baseline for staged buffered `chan` support, explicit send/receive syntax, and first-pass close behavior
