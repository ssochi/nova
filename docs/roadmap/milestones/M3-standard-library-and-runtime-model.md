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

- `2026-03-20-13-55-46-type-assertions-first-slice`: completed plan for staged single-result `x.(T)` assertions over empty-interface values, interface-conversion panic behavior, and explicit AST/bytecode inspection
- `2026-03-20-13-28-50-recover-first-slice`: completed plan for staged builtin `recover()`, deferred-frame recovery eligibility, recovered `any` payloads, semantic panic termination, and VM helper extraction
- `2026-03-20-12-55-53-empty-interface-any-and-fmt-spread`: completed plan for staged empty-interface `any` / `interface{}` groundwork, explicit boxing, nil-interface runtime behavior, narrow interface equality, and `fmt` variadic spread support
- `2026-03-20-12-28-16-panic-aware-unwind-first-slice`: completed plan for staged builtin `panic`, panic-aware defer unwinding, selected runtime-trap integration, and preserved CLI output on runtime failure
- `2026-03-20-11-58-27-defer-first-slice`: completed plan for staged `defer` statements, eager argument capture, and explicit frame-level deferred-call execution
- `2026-03-20-11-23-19-named-result-parameters`: completed plan for grouped named result declarations, function-entry result-slot initialization, blank result identifiers, and bare `return`
- `2026-03-20-11-02-46-grouped-parameter-shorthand`: completed plan for staged grouped parameter-name shorthand such as `func f(a, b int)`
- `2026-03-20-10-46-29-builtin-clear-and-runtime-file-split`: completed plan for builtin `clear(slice|map)` plus runtime file-size governance
- `2026-03-20-10-32-34-strings-bytes-clone-seams`: completed plan for staged `strings.Clone` / `bytes.Clone` package seams
- `2026-03-20-10-21-19-strings-bytes-compare-seams`: completed plan for staged `strings.Compare` / `bytes.Compare` package seams
- `2026-03-20-10-08-16-strings-bytes-last-index-byte-search`: completed plan for staged `strings` / `bytes` last-index and byte-search package seams
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
- The current runtime now has byte-oriented strings, slice allocation, narrow explicit string/byte conversions, staged map groundwork, staged map literals plus `delete`, builtin `clear` for slices/maps, explicit `nil` for slice/map/chan contexts, staged `range` loops over slices/maps, duplicate constant literal-key diagnostics, staged comma-ok lookups, staged `if` headers, staged expression `switch`, classic `for` clauses, unlabeled `break` / `continue`, staged multi-binding short declarations and assignments, explicit `++` / `--`, staged compound assignments, the first buffered `chan` slice, grouped imports, explicit import aliases, explicit grouped input parameter-name shorthand, grouped named result declarations, explicit multi-result function signatures / returns, bare `return`, staged variadic function declarations, explicit final-argument `...`, staged direct-call `defer`, staged builtin `panic`, panic-aware deferred unwinding, staged builtin `recover()`, preserved CLI output on runtime failure, explicit empty-interface `any` / `interface{}` values, explicit boxing bytecode, staged single-result `x.(T)` assertions, narrow interface equality, `[]any` literals, builtin `append` spread handling including `[]byte` plus `string...`, and package-backed `fmt` / `strings` / `bytes` seams including `Compare`, `Clone`, `Cut`, and `fmt` `[]any...`; scheduler-aware blocking, channel directions, channel `range`, comma-ok assertions, type switches, labels, broader `assign_op` coverage, real import graphs, wider package-backed runtime services, richer recovered runtime payload typing, and broader interface behavior remain open.

## Next-Round Recommendations

- Open the next `M3` plan for another interface/runtime slice or a byte-oriented package/API slice that takes advantage of the new explicit assertion seam.
- The strongest adjacent interface continuation is comma-ok assertions and type switches on top of the new `x.(T)` path, or broader non-empty-interface groundwork that makes assertion targets more expressive than empty-interface boxing alone.
- If control-flow stays the next priority, reuse the explicit `ForStatement` / `CheckedForStatement` model and the compiler control-flow stack instead of lowering new targets through ad hoc jumps.
- Another byte-oriented package-backed API slice or a focused VM test-file split are the strongest adjacent continuations; keep explicit `...` separate from the staged multi-result forwarding path and keep UTF-8-sequence-sensitive helpers deferred.
- The next `M3` slice should now build on the richer function-signature surface rather than reopening it immediately; a standard-library seam or call-site compatibility slice is a stronger follow-up.
- Reuse the `docs/research/` flow before locking the next compatibility-sensitive slice.
- The strongest adjacent continuation after the recover slice is either a broader runtime-type metadata pass that safely widens interface equality / recovered payload fidelity, or another package-backed seam that benefits from the now more realistic panic/defer behavior.
