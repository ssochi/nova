# M3: Standard Library and Runtime Model

- Status: `in_progress`
- Current Main Plan: none currently

## Goals

- Expand the runtime value model beyond integers and booleans.
- Increase builtin coverage and start building stable seams for standard library support.
- Keep the VM-first execution path while preparing for broader Go program compatibility.

## Completion Criteria

- The semantic and runtime layers support at least one new non-scalar runtime category or interoperability seam needed for future standard library work.
- Builtin coverage expands beyond `println` in a way that remains layered and testable.
- The CLI validation path covers richer runtime behavior than the current arithmetic / branch / loop subset.
- Technical and roadmap documents describe the runtime model clearly enough for backend-bridge work to build on later.

## Task Breakdown

- Define the next runtime value and builtin slice with explicit non-goals.
- Extend semantic analysis, lowering, and VM execution for the chosen runtime model changes.
- Add CLI-first validation, diagnostics, and documentation for the richer runtime surface.
- Keep standard-library-oriented work decomposed into resumable plans instead of one large jump.

## Related Plans

- `2026-03-20-09-49-02-strings-bytes-index-trim`: completed plan for staged `strings` / `bytes` index, suffix, and trim package seams
- `2026-03-20-09-21-38-variadic-functions-ellipsis`: completed plan for staged variadic function declarations, explicit final-argument `...`, and builtin `append` spread semantics
- `2026-03-20-08-55-11-call-argument-multi-result-forwarding`: completed plan for staged call-argument multi-result forwarding plus `strings` / `bytes` `CutPrefix` / `CutSuffix` seams
- `2026-03-20-08-14-13-multi-result-functions-cut-seams`: completed plan for the first staged multi-result model plus `strings.Cut` / `bytes.Cut`
- `2026-03-20-07-46-15-import-aliases-and-bytes-package`: completed plan for grouped imports, explicit import aliases, and the staged `bytes` package seam
- `2026-03-20-07-17-39-channel-runtime-first-slice`: completed staged buffered `chan` support with explicit send / receive / close behavior, nil-channel zero values, and channel-aware `len` / `cap`
- `2026-03-20-06-56-53-compound-assignments`: completed staged compound assignments across ordinary statements, headers, classic `for` clauses, and single-evaluation index lowering
- `2026-03-20-06-12-55-for-clauses-break-continue`: completed staged classic `for` clauses, unlabeled `break` / `continue`, and conservative loop termination analysis
- `2026-03-20-06-34-01-simple-statements-incdec`: completed staged short declarations plus explicit `++` / `--` in ordinary statements, headers, and classic `for` clauses
- `2026-03-20-05-46-33-switch-statements`: completed staged expression `switch`, tagless `switch`, shared control-flow header modeling, and duplicate-clause diagnostics
- `2026-03-20-05-29-16-if-initializers-else-if`: completed staged `if` statement initializers, shared header scope, and `else if` chains
- `2026-03-20-05-11-15-map-comma-ok-and-literal-diagnostics`: completed staged comma-ok `map` lookups, short redeclaration rules, and duplicate literal-key diagnostics
- `2026-03-20-04-49-50-slice-map-range-loops`: completed staged `range` loops over `slice` and `map`
- `2026-03-20-04-28-25-explicit-nil-comparisons`: completed explicit `nil` expressions and `slice/map` nil comparisons
- `2026-03-20-04-07-11-map-literals-delete`: completed staged map usability work for `map[K]V{...}` literals and builtin `delete`
- `2026-03-20-03-42-30-map-runtime-groundwork`: completed staged `map[K]V` support with `make`, `len`, indexing, and assignment
- `2026-03-20-03-02-10-byte-strings-and-slicing`: completed byte-oriented runtime strings, string indexing/slicing, `byte`, and `copy([]byte, string)`
- `2026-03-20-03-23-06-string-byte-conversions`: completed first-class `[]byte(string)` and `string([]byte)` conversion syntax
- `2026-03-20-02-39-55-make-slice-allocation`: completed builtin `make([]T, len[, cap])` and the first type-argument builtin path
- `2026-03-20-00-09-59-string-runtime-builtins`: completed first richer runtime value slice with builtin contract centralization
- `2026-03-20-00-35-11-import-fmt-seam`: completed the first narrow import and package-backed standard-library seam
- `2026-03-20-00-55-55-slice-runtime-testing`: completed the first composite runtime value and layered test coverage
- `2026-03-20-01-17-51-strings-package-contracts`: completed typed package contracts, a second standard-library seam, and the research baseline
- `2026-03-20-01-33-44-slice-expressions-assignment`: completed slice windows, shared slice storage, and indexed slice assignment
- `2026-03-20-01-54-16-slice-builtins-capacity`: completed `cap`, `copy`, and capacity-aware `append`
- `2026-03-20-02-18-45-typed-var-zero-values`: completed explicit typed `var` declarations and nil-slice zero-value runtime behavior

## Current Risks

- Runtime-surface growth can sprawl quickly if types, builtins, imports, and control-flow work are mixed into the same plan.
- Builtin additions can become hardcoded special cases unless their contracts stay centralized.
- Supporting more realistic Go programs will require careful staging so the VM remains understandable.
- The current runtime now has byte-oriented strings, slice allocation, narrow explicit string/byte conversions, staged map groundwork, staged map literals plus `delete`, explicit `nil` for slice/map/chan contexts, staged `range` loops over slices/maps, duplicate constant literal-key diagnostics, staged comma-ok lookups, staged `if` headers, staged expression `switch`, classic `for` clauses, unlabeled `break` / `continue`, staged multi-binding short declarations and assignments, explicit `++` / `--`, staged compound assignments, the first buffered `chan` slice, grouped imports, explicit import aliases, explicit multi-result function signatures / returns, staged variadic function declarations, explicit final-argument `...`, builtin `append` spread handling including `[]byte` plus `string...`, and package-backed `fmt` / `strings` / `bytes` seams including `Cut`; scheduler-aware blocking, channel directions, channel `range`, labels, broader `assign_op` coverage, real import graphs, wider package-backed runtime services, grouped parameter shorthand, and wider multi-result or interface-backed variadic consumption remain open.

## Next-Round Recommendations

- Open the next `M3` plan for either grouped parameter-name shorthand or the next package/API slice that avoids rune-sensitive behavior until the runtime models it deliberately.
- If control-flow stays the next priority, reuse the explicit `ForStatement` / `CheckedForStatement` model and the compiler control-flow stack instead of lowering new targets through ad hoc jumps.
- Grouped parameter-name shorthand, `LastIndex`-style search helpers, or another package-backed API slice are the strongest adjacent continuations; keep explicit `...` separate from the staged multi-result forwarding path and keep UTF-8-sequence-sensitive helpers deferred.
- Reuse the `docs/research/` flow before locking the next compatibility-sensitive slice.
