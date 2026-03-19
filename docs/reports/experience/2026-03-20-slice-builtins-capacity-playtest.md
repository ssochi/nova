# Slice Builtins and Capacity-Aware Append CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-01-54-16-slice-builtins-capacity`
- Entry surface: `run`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/slice_builtins.go` to verify real execution of `cap`, `copy`, and append reuse through the CLI.
2. Ran `cargo run -- dump-ast examples/slice_builtins.go` to inspect whether the source-oriented surface still reads naturally with the new builtins.
3. Ran `cargo run -- dump-bytecode examples/slice_builtins.go` to confirm the VM-facing form exposes the new builtin operations clearly.
4. Ran `cargo run -- check /tmp/nova-go-bad-copy-check.go` to inspect the failure path for a mismatched `copy` call.

## Positive Experience

- The happy-path example is compact but expressive: one file shows slice capacity, append reuse, and overlapping copy without requiring users to understand internal runtime details first.
- The AST output stays readable because `cap(head)` and `copy(values, values[1:])` appear almost exactly as written in source.
- The bytecode output is transparent enough for debugging: each new builtin remains visible as its own `call-builtin` instruction instead of disappearing behind generic runtime helpers.
- The failure path is direct and early; a wrong `copy` element type is rejected at `check` time with a concrete diagnostic instead of failing later during execution.

## Issues and Severity

- Medium: users may still assume `cap` works on arrays or that `copy` supports `string` sources because those are valid in Go, but the current subset only supports slice-to-slice behavior.
- Medium: append behavior is more Go-like than before, yet reallocation growth remains intentionally minimal and undocumented to end users unless they read the reports.
- Low: the new example proves aliasing semantics, but there is still no CLI-facing tool that explains capacity metadata directly beyond program output and bytecode inspection.

## Conclusion and Next Recommended Steps

This round makes slice-bearing programs more believable through the real CLI because capacity now affects observable runtime behavior instead of being inert metadata. The next strongest `M3` follow-up is either byte-oriented string/runtime work for string slicing and `[]byte`-adjacent semantics, or a deeper allocation model such as `make` and nil slices so slice construction and growth stop depending only on literals and reslices.
