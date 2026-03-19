# Range Loops for Slices and Maps

## Goal

Capture the official Go behavior baseline needed to stage `range` loop support in `nova-go`, with emphasis on `slice` and `map` iteration, nil behavior, iteration variables, and the smallest useful syntax surface for the current VM-first compiler.

## Sources Reviewed

- Official Go language specification section `For statements with range clause` (`https://go.dev/ref/spec`)
- Local Go toolchain spot-check with `go version go1.21.5 darwin/arm64`
- Local Go toolchain spot-check program verifying `for range values`, `for _, value := range []int{...}`, and `for key := range nilMap`

## Confirmed Findings

- The Go spec allows `range` over arrays, slices, strings, maps, channels, integers, and iterator functions, but `slice` and `map` are the compatible near-term targets for the current `nova-go` runtime model.
- The range expression is evaluated before the loop begins, except for a constant-expression optimization that only matters for arrays, pointers-to-arrays, or integer ranges.
- For `slice` iteration:
  - the first iteration value is the `int` index
  - the second iteration value is the element value
  - indices increase from `0` to `len(slice)-1`
  - a nil slice executes zero iterations
  - when at most one iteration variable is present, Go does not need to index into the slice to produce the omitted value
- For `map` iteration:
  - the first iteration value is the key
  - the second iteration value is `m[key]`
  - a nil map executes zero iterations
  - the iteration order is unspecified and may vary between iterations in real Go
- The left side of a range clause may be absent, assignment-style (`=`), or short-declaration-style (`:=`).
- At most two iteration variables are permitted for `slice` and `map` ranges.
- If the last iteration variable is `_`, the range clause is equivalent to omitting that variable entirely.
- The Go spec notes that `:=` loop variables have block scope and, in modern Go, each iteration gets fresh variables. Given the current `nova-go` subset, the observable parts that matter now are type, scope, and assignment behavior; closure-sensitive per-iteration identity can remain deferred.
- Local `go1.21.5` validation confirms that `for range values {}` is valid syntax and that nil `slice` / nil `map` loops run zero iterations in practice.

## Implementation Implications

- The first `nova-go` range slice should stay limited to `slice` and `map`; string `range` requires rune semantics that the current runtime intentionally does not model.
- Supporting all three staged forms is useful without widening into general short variable declarations:
  - `for range expr { ... }`
  - `for key := range expr { ... }`
  - `for key, value := range expr { ... }`
- Assignment-form range (`=`) is also compatible with the current language model and helps avoid painting future control-flow work into a corner, but the left side can stay restricted to identifiers or `_` in this stage instead of full addressable expressions.
- Nil iteration should be explicit in the runtime path and produce zero loop iterations rather than runtime errors.
- Because real Go leaves map order unspecified, `nova-go` can intentionally keep deterministic order for now as a debug-friendly staged behavior, but that deviation must be recorded in docs and reports.
- The bytecode path should make range execution inspectable instead of lowering it into opaque hidden calls; `dump-bytecode` should still reveal when a loop is slice-based versus map-based.
- Since `src/semantic/analyzer.rs` and `src/runtime/vm.rs` are already near the file-size limit, the range implementation should extract helpers or submodules in the same iteration.

## Deferred Questions

- Whether assignment-form range should later accept the full Go set of addressable operands instead of the staged identifier-only subset.
- Whether map range should eventually snapshot keys up front or support Go-like mutation-during-iteration behavior more precisely.
- How to model per-iteration fresh variables once closures, pointers, or address-taking make that distinction observable.
