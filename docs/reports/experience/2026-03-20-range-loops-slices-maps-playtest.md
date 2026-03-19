# Range Loops for Slices and Maps Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-04-49-50-slice-map-range-loops`
- Focus: real CLI experience for staged `range` loops over `slice` and `map`

## Experience Path

1. Ran `cargo run -- run examples/range_loops.go` to exercise the new loop form through slice index/value iteration, no-binding `for range expr`, map key/value iteration, and nil composite zero-iteration behavior.
2. Ran `cargo run -- dump-ast examples/range_loops.go` to confirm the new syntax remains readable without implementation context.
3. Ran `cargo run -- dump-bytecode examples/range_loops.go` to inspect whether the loop lowering path stays visible at the VM-facing debug layer.
4. Ran `cargo run -- check <temp-source with for range 1>` as the first error path.
5. Ran `cargo run -- check <temp-source with for label = range []int{1}>` as the second error path.

## Positive Experience

- The happy-path sample feels substantially more Go-like because collection traversal is no longer forced through manual index loops. The staged forms `for range values`, `for index := range values`, and `for key, value := range counts` all read naturally.
- The real CLI path is coherent across surfaces. `run` demonstrates correct output, `dump-ast` preserves the exact loop spellings, and `dump-bytecode` still exposes the lowering path with explicit hidden range locals and `map-keys`.
- Both failure paths are understandable without code reading. One explains that only `slice` and `map` are iterable in the current subset, and the other explains why an existing variable cannot receive the wrong iteration type.

## Issues and Severity

- Low: `dump-bytecode` exposes many hidden locals for range lowering, which is useful for debugging but somewhat noisy for larger functions.
- Medium: Map iteration order is deterministic rather than Go-like unspecified, so users who compare `nova-go` output against real Go on order-sensitive examples may notice the staging choice.
- Medium: String `range`, `break`, and `continue` are still absent, so some common real Go loop idioms remain unavailable even after this improvement.

## Conclusion and Next Recommended Steps

This round materially improves the CLI-first experience because a core Go control-flow idiom now works end to end across parsing, semantics, lowering, and runtime execution. The strongest next `M3` continuation is to deepen map compatibility further with duplicate-constant-key diagnostics and comma-ok lookups, or to start the first channel/runtime slice if the milestone priority shifts toward concurrency groundwork.
