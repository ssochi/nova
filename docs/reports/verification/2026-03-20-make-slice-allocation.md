# Make-Based Slice Allocation Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-02-39-55-make-slice-allocation`

## Validation Goal

Verify that `nova-go` now supports builtin `make([]T, len[, cap])` for slices while keeping parser output, semantic diagnostics, bytecode lowering, VM execution, and CLI inspection surfaces aligned.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/make_slices.go`
- `cargo run -- dump-ast examples/make_slices.go`
- `cargo run -- dump-bytecode examples/make_slices.go`
- `cargo run -- check /tmp/nova-go-bad-make.go`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeed with the current local toolchain.
- `cargo test` now passes with new parser, semantic, runtime, and CLI coverage for `make([]T, len[, cap])`, zero-filled spare capacity, and invalid `make` diagnostics.
- `cargo run -- run examples/make_slices.go` prints:
  - `2 4 7 8`
  - `0 3 4`
  - `3 4 9`
  - `2 2 0 0`
- The happy-path output proves four behaviors together: `make([]int, 2, 4)` allocates a non-nil slice with explicit spare capacity, reslicing into spare capacity exposes zero-filled elements, `append` reuses that allocation when capacity allows, and `make([]string, 2)` zero-initializes string elements.
- `cargo run -- dump-ast examples/make_slices.go` renders `var values = make([]int, 2, 4)` and `var labels = make([]string, 2)` directly, confirming the source-oriented CLI surface stays readable for the type-argument builtin.
- `cargo run -- dump-bytecode examples/make_slices.go` shows `make-slice int cap=explicit` and `make-slice string cap=len`, confirming the allocation path is explicit at the VM-facing debug surface.
- The failure path `cargo run -- check /tmp/nova-go-bad-make.go` reports `builtin \`make\` length 3 exceeds capacity 2`, confirming invalid constant-size slice allocation is rejected before execution with a targeted diagnostic.

## Remaining Risks

- `make` currently supports slices only; map and channel allocation remain outside the implemented subset.
- Compile-time bound rejection is still narrow and currently catches only literal integer `len > cap` cases instead of broader Go constant evaluation.
- The project still lacks string slicing and byte-oriented runtime values, so `make` improves slice allocation without solving broader `[]byte` compatibility.
