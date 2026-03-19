# Explicit Nil Comparisons Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-04-28-25-explicit-nil-comparisons`
- Focus: real CLI experience for explicit `nil` declarations, comparisons, and failure diagnostics

## Experience Path

1. Ran `cargo run -- run examples/nil_values.go` to exercise the main user flow for typed nil declarations, nil reassignment, nil comparisons, user-defined call arguments, package-backed `strings.Join(nil, ":")`, and nil-returning functions.
2. Ran `cargo run -- dump-ast examples/nil_values.go` to inspect whether the new syntax remains legible at the AST surface.
3. Ran `cargo run -- dump-bytecode examples/nil_values.go` to confirm the typed nil execution path is still visible at the VM-facing debug layer.
4. Ran `cargo run -- check <temp-source with var values = nil>` as the first error path.
5. Ran `cargo run -- check <temp-source with println(nil == nil)>` as the second error path.

## Positive Experience

- The happy-path program feels Go-like immediately because `nil` is no longer trapped behind implicit zero values; users can write `var values []int = nil`, pass `nil` into typed functions, and compare against `nil` in the obvious way.
- The AST and bytecode inspection surfaces are still useful after the feature: `dump-ast` shows explicit `nil`, and `dump-bytecode` clearly shows `push-nil-slice` / `push-nil-map` instead of hiding the behavior behind generic runtime fallback logic.
- The two failure paths are understandable without reading the code. One tells the user that `nil` needs explicit type context, and the other explains why `nil == nil` is invalid.

## Issues and Severity

- Low: `dump-ast` renders equality expressions with extra parentheses such as `println((values == nil), (counts == nil))`. The output is still readable, but not especially Go-like.
- Medium: Bare `nil` in variadic output paths such as `println(nil)` or `fmt.Println(nil)` still has no usable type context in the current subset, so some real Go snippets will remain rejected until the nilable-type surface broadens.

## Conclusion and Next Recommended Steps

This round materially improves the CLI-first experience because slice/map programs can now express `nil` directly instead of relying only on implicit zero values. The next strongest `M3` continuation is to build on the same nil-aware composite foundation with either deeper map compatibility such as comma-ok and `range`, or the first channel/runtime slice if milestone leverage shifts.
