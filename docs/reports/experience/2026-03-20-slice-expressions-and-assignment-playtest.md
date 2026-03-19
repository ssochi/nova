# Slice Expressions and Assignment CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-01-33-44-slice-expressions-assignment`
- Entry surface: `run`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/slice_windows.go` to verify the real execution path for slice windows and indexed updates.
2. Ran `cargo run -- dump-ast examples/slice_windows.go` to inspect whether the new syntax still reads like the original source.
3. Ran `cargo run -- dump-bytecode examples/slice_windows.go` to confirm the VM-facing form makes the new slice behavior visible.
4. Ran `cargo run -- check <temp-full-slice-source>` to inspect the failure path for a deliberately unsupported full slice expression.

## Positive Experience

- The happy-path example makes the new behavior easy to see because it shows both reslicing and shared mutation in two short output lines.
- The AST output stays readable; `values[:2]` and `reopen[2] = 7` remain recognizable instead of collapsing into generic expression noise.
- The bytecode output is more transparent than before because `slice` and `set-index` are explicit instructions rather than hidden inside broader runtime helpers.
- The error path is sharp: unsupported full slice syntax fails immediately with a direct parser diagnostic instead of a vague later-stage error.

## Issues and Severity

- Medium: users familiar with Go may expect string slicing to work too, but this round intentionally limits execution support to `[]T`.
- Medium: `append` still behaves like a copy-producing helper rather than a capacity-aware Go append, so slice semantics are closer to Go but not yet complete.
- Low: the bytecode `slice low=<bool> high=<bool>` form is accurate for debugging, but not yet especially friendly for users who do not already know the omitted-bound rules.

## Conclusion and Next Recommended Steps

This round materially improves the composite-value experience because slice-bearing programs can now reslice and mutate through the real CLI path, which unlocks a much more Go-like workflow than literal-plus-index-only slices. The next strongest follow-up is either string/runtime representation work needed for string slicing and richer `strings` compatibility, or broader slice/builtin coverage such as `cap`, `copy`, and more realistic `append` behavior.
