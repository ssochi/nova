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
- `2026-03-20-strings-bytes-index-trim-seams.md`: validates staged `strings` / `bytes` index, suffix, and trim helpers across the full CLI and VM stack
- `2026-03-20-variadic-functions-and-ellipsis.md`: validates staged variadic declarations, explicit final-argument `...`, and builtin `append` spread semantics across the full CLI and VM stack
- `2026-03-20-call-argument-multi-result-forwarding.md`: validates staged call-argument multi-result forwarding plus the `strings` / `bytes` `CutPrefix` / `CutSuffix` seams across the full CLI and VM stack
- `2026-03-20-multi-result-functions-and-cut-package-seams.md`: validates unnamed multi-result functions, staged multi-binding statements, and the `strings.Cut` / `bytes.Cut` seams across the full CLI and VM stack
- `2026-03-20-import-aliases-and-bytes-package.md`: validates grouped imports, explicit import aliases, and the staged `bytes` package seam across the full CLI and VM stack
- `2026-03-20-loop-control-flow.md`: validates staged classic `for` clauses, unlabeled `break` / `continue`, and conservative loop termination analysis
- `2026-03-20-switch-statements.md`: validates staged expression `switch` statements, tagless `switch`, shared control-flow headers, and duplicate-clause diagnostics
- `2026-03-20-if-statement-headers.md`: validates staged `if` statement initializers, shared header scoping, and `else if` chains across the full CLI and VM stack
- `2026-03-20-map-comma-ok-lookups.md`: validates staged comma-ok `map` lookups and duplicate constant-key diagnostics across the full CLI and VM stack
- `2026-03-20-range-loops-slices-maps.md`: validates staged `range` loops over `slice` and `map` across the full CLI and VM stack
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
- `2026-03-20-byte-strings-and-slicing.md`: validates byte-oriented strings, `byte`, string indexing/slicing, and `copy([]byte, string)`
- `2026-03-20-string-byte-conversions.md`: validates explicit `[]byte(string)` / `string([]byte)` conversions across the full CLI and VM stack
- `2026-03-20-map-runtime-groundwork.md`: validates staged `map[K]V` support across parsing, semantic analysis, bytecode lowering, VM execution, and CLI inspection
- `2026-03-20-map-literals-delete.md`: validates staged `map[K]V{...}` literals and builtin `delete(map, key)` across the full CLI and VM stack
- `2026-03-20-explicit-nil-comparisons.md`: validates explicit `nil` expressions, typed nil coercion, and `slice/map` nil comparisons across the full CLI and VM stack
- `2026-03-20-simple-statements-incdec.md`: validates staged short declarations and explicit `++` / `--` across the full CLI and VM stack
- `2026-03-20-compound-assignments.md`: validates staged compound assignments across parsing, semantic analysis, bytecode lowering, VM execution, and CLI inspection
- `2026-03-20-channel-runtime-first-slice.md`: validates staged `chan` support, explicit send/receive lowering, builtin `close`, and buffered runtime behavior across the full CLI and VM stack
