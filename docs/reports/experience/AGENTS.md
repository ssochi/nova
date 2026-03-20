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
- `2026-03-20-strings-bytes-last-index-byte-search-playtest.md`: records the CLI walkthrough for staged `strings` / `bytes` last-index and byte-search package seams
- `2026-03-20-strings-bytes-index-trim-seams-playtest.md`: records the CLI walkthrough for staged `strings` / `bytes` index, suffix, and trim package seams
- `2026-03-20-variadic-functions-and-ellipsis-playtest.md`: records the CLI walkthrough for staged variadic declarations, explicit final-argument `...`, and builtin `append` spread semantics
- `2026-03-20-call-argument-multi-result-forwarding-playtest.md`: records the CLI walkthrough for staged call-argument forwarding plus the `strings` / `bytes` `CutPrefix` / `CutSuffix` seams
- `2026-03-20-multi-result-functions-and-cut-package-seams-playtest.md`: records the CLI walkthrough for unnamed multi-result functions plus the `strings.Cut` / `bytes.Cut` seams
- `2026-03-20-import-aliases-and-bytes-package-playtest.md`: records the CLI walkthrough for grouped imports, explicit import aliases, and the staged `bytes` package seam
- `2026-03-20-loop-control-flow-playtest.md`: records the CLI walkthrough for classic `for` clauses plus unlabeled `break` / `continue`
- `2026-03-20-switch-statements-playtest.md`: records the CLI walkthrough for staged expression `switch`, tagless `switch`, shared header scope, and duplicate-clause diagnostics
- `2026-03-20-if-statement-headers-playtest.md`: records the CLI walkthrough for staged `if` statement initializers, shared header scoping, and `else if` chains
- `2026-03-20-map-comma-ok-lookups-playtest.md`: records the CLI walkthrough for staged comma-ok `map` lookups and duplicate constant-key diagnostics
- `2026-03-20-range-loops-slices-maps-playtest.md`: records the CLI walkthrough for staged `range` loops over `slice` and `map`
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
- `2026-03-20-map-runtime-groundwork-playtest.md`: records the CLI walkthrough for staged `map[K]V` support, nil-map behavior, and explicit map bytecode
- `2026-03-20-map-literals-delete-playtest.md`: records the CLI walkthrough for staged map literals, empty-map construction, and builtin `delete`
- `2026-03-20-explicit-nil-comparisons-playtest.md`: records the CLI walkthrough for explicit `nil`, typed nil coercion, and `slice/map` nil comparisons
- `2026-03-20-simple-statements-incdec-playtest.md`: records the CLI walkthrough for staged short declarations and explicit `++` / `--`
- `2026-03-20-compound-assignments-playtest.md`: records the CLI walkthrough for staged compound assignments and single-evaluation index updates
- `2026-03-20-channel-runtime-first-slice-playtest.md`: records the CLI walkthrough for staged buffered `chan` support and explicit send / receive / close behavior
