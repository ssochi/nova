# Slice Builtins and Capacity-Aware Append Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-01-54-16-slice-builtins-capacity`

## Validation Goal

Verify that `nova-go` now supports `cap(slice)`, `copy(dstSlice, srcSlice)`, and capacity-aware `append` reuse while keeping semantic validation, runtime behavior, and CLI inspection surfaces aligned.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/slice_builtins.go`
- `cargo run -- dump-ast examples/slice_builtins.go`
- `cargo run -- dump-bytecode examples/slice_builtins.go`
- `cargo run -- check /tmp/nova-go-bad-copy-check.go`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeed with the current local toolchain; no environment repair was needed.
- `cargo test` now passes with new unit coverage for builtin contract validation, overlap-safe slice copying, and capacity-aware append reuse, plus CLI coverage for the new `slice_builtins` example and diagnostics.
- `cargo run -- run examples/slice_builtins.go` prints:
  - `2 4`
  - `9 3 4`
  - `3 2 9 4 4`
- The happy-path output proves three behaviors together: `cap(head)` exposes the original spare capacity, `append(head, 9)` reuses the same backing storage when capacity allows, and overlapping `copy(values, values[1:])` produces a stable result instead of alias-order corruption.
- `cargo run -- dump-ast examples/slice_builtins.go` renders `cap(head)`, `append(head, 9)`, and `copy(values, values[1:])` directly, confirming the CLI inspection surface stays readable for the new builtin subset.
- `cargo run -- dump-bytecode examples/slice_builtins.go` shows `call-builtin cap 1`, `call-builtin append 2`, and `call-builtin copy 2`, confirming lowering exposes the new builtin operations explicitly.
- The failure path `cargo run -- check /tmp/nova-go-bad-copy-check.go` reports `argument 2 in call to builtin \`copy\` requires \`[]int\`, found \`[]string\``, confirming type mismatches are rejected during semantic analysis with a targeted diagnostic.

## Remaining Risks

- `copy` still omits the Go special case that allows `[]byte` destinations with `string` sources because the runtime has no byte slice type yet.
- `append` now models reuse within capacity, but allocation growth beyond capacity still uses a minimal deterministic rule rather than Go's runtime growth heuristics.
- `cap` only supports slice operands in the current compiler subset; arrays, pointers to arrays, and channels remain out of scope.
