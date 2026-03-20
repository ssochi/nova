# research Directory Conventions

This directory stores research notes that inform roadmap choices and implementation slices. Research documents are inputs to design and planning, not replacements for feature specs or validation reports.

## Directory Responsibilities

- Record external behavior baselines, comparisons, and capability surveys that affect project decisions
- Capture what was verified, what remains uncertain, and how the research should influence implementation scope
- Provide indexed research entry points that later agents can quickly reuse

## When It Must Be Updated

- When a plan depends on external behavior research or compatibility notes
- When a stable research note is added, retired, or materially revised
- When the directory needs a stronger indexing or naming convention

## File Format Convention

- Research files use `YYYY-MM-DD-<topic>.md`
- Each note should clearly separate confirmed facts, chosen scope, and deferred questions
- Research notes may cite official sources, but should summarize the conclusions instead of copying large passages

## Document Structure

- Each research note should include:
  - `# <Research Topic>`
  - `## Goal`
  - `## Sources Reviewed`
  - `## Confirmed Findings`
  - `## Implementation Implications`
  - `## Deferred Questions`

## File Index

- `AGENTS.md`: this directory convention
- `2026-03-20-empty-interface-any-and-fmt-spread.md`: local Go behavior baseline for staged empty-interface `any` / `interface{}` support, nil-interface behavior, interface equality edges, and `fmt` variadic spread over `[]any`
  - extended with staged `x.(T)` type assertions, comma-ok assertions, and the first empty-interface type-switch compatibility findings
- `2026-03-20-panic-aware-unwind-first-slice.md`: official and local Go behavior baseline for staged panic/recover semantics, panic-aware defer unwinding, direct recovery eligibility, and `panic(nil)` recovery behavior
- `2026-03-20-defer-first-slice.md`: official behavior baseline for the first staged `defer` statement slice
- `2026-03-20-named-result-parameters.md`: official behavior baseline for grouped named result declarations, result-slot initialization, and bare `return`
- `2026-03-20-grouped-parameter-shorthand.md`: official behavior baseline for grouped input parameter declarations such as `func f(a, b int)`
- `2026-03-20-builtin-clear-slice-map.md`: official behavior baseline for builtin `clear` on staged `slice` and `map` values
- `2026-03-20-strings-bytes-clone-seams.md`: official behavior baseline for staged `strings.Clone` / `bytes.Clone` package helpers
- `2026-03-20-strings-bytes-compare-seams.md`: official behavior baseline for staged `strings.Compare` / `bytes.Compare` package helpers
- `2026-03-20-strings-bytes-last-index-byte-search.md`: official behavior baseline for staged `strings` / `bytes` last-index and byte-search helpers
- `2026-03-20-strings-bytes-index-trim-seams.md`: official behavior baseline for the staged `strings` / `bytes` index, suffix, and trim package helpers
- `2026-03-20-variadic-functions-and-ellipsis.md`: official behavior baseline for staged variadic function declarations, final-argument `...` calls, and `append(slice, values...)`
- `2026-03-20-multi-result-functions-and-cut-package-seams.md`: official behavior baseline for unnamed multi-result functions, staged binding forms, and the `strings.Cut` / `bytes.Cut` seams
- `2026-03-20-import-aliases-and-bytes-package.md`: official behavior baseline for grouped imports, explicit import aliases, and the staged `bytes` package seam
- `2026-03-20-loop-control-flow.md`: official behavior baseline for staged classic `for` clauses, omitted conditions, and unlabeled `break` / `continue`
- `2026-03-20-switch-statements.md`: official behavior baseline for staged expression `switch` statements, tagless `switch`, shared header scope, and duplicate-clause diagnostics
- `2026-03-20-if-statement-headers.md`: official behavior baseline for `if` statement initializers, scope visibility, and `else if` chains
- `2026-03-20-map-comma-ok-lookups.md`: official behavior baseline for comma-ok `map` lookups, short redeclaration rules, and duplicate constant-key diagnostics
- `2026-03-20-range-loops-slices-maps.md`: official behavior baseline for staged `range` loops over `slice` and `map`
- `2026-03-20-strings-package-contracts.md`: official behavior baseline for the first `strings` package seam
- `2026-03-20-slice-expressions-and-assignment.md`: official behavior baseline for the current slice and string-window surface, including string indexing, typed zero values, `make`, `cap`, `copy`, and append-capacity semantics
- `2026-03-20-map-runtime-groundwork.md`: official behavior baseline for staged `map[K]V` support with `make`, `len`, indexing, and assignment
- `2026-03-20-composite-nil-semantics.md`: official behavior baseline for explicit `nil` usage with the current slice/map surface
- `2026-03-20-simple-statements-incdec.md`: official behavior baseline for staged short declarations and explicit `++` / `--` statements
- `2026-03-20-compound-assignments.md`: official behavior baseline for staged `op=` assignment semantics, single-evaluation lowering, and the supported operator subset
- `2026-03-20-channel-runtime-first-slice.md`: official behavior baseline for staged `chan` support, buffered send/receive semantics, and builtin `close`
