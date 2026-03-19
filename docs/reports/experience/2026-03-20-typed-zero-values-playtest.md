# Typed Var Declarations and Zero-Value Slices CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-02-18-45-typed-var-zero-values`
- Entry surface: `run`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/typed_zero_values.go` to verify that typed locals and nil-slice zero values work through the normal execution path.
2. Ran `cargo run -- dump-ast examples/typed_zero_values.go` to inspect whether the new declaration form remains readable from the source-oriented CLI surface.
3. Ran `cargo run -- dump-bytecode examples/typed_zero_values.go` to confirm the zero-value path is still transparent at the VM-facing debug surface.
4. Ran `cargo run -- check /tmp/nova-go-bad-typed-var.go` to inspect the failure path for a mismatched typed initializer.

## Positive Experience

- The new declaration form reads naturally in the CLI: `var total int` and `var values []int` appear exactly as a Go user would expect.
- The runtime path feels coherent because nil-slice zero values immediately compose with the existing slice toolchain: `len`, `cap`, slicing, and `append` all work without extra ceremony.
- The bytecode dump remains useful instead of opaque; explicit zero-value instructions make it obvious how typed declarations are realized in the VM.
- The failure path stays early and specific; a wrong typed initializer is rejected during `check` with a concrete type mismatch instead of surfacing later during execution.

## Issues and Severity

- Medium: the language now has a stronger declaration story, but users still cannot write `make([]int, n)` for general slice allocation.
- Medium: nil slices are observable through behavior, but explicit `nil` expressions and nil comparisons are still absent, which may surprise Go users trying to port code directly.
- Low: the bytecode dump shows zero-value instructions clearly, but the function header still lists only local names rather than types.

## Conclusion and Next Recommended Steps

This round makes the CLI feel more like a real Go workflow because zero-valued locals no longer require placeholder initializers and nil slices work across the existing slice surface. The next strongest `M3` follow-up is to build on this declaration/runtime boundary with `make`-based slice allocation, or to switch to byte-oriented string work if the project wants to reopen string slicing and `[]byte` compatibility first.
