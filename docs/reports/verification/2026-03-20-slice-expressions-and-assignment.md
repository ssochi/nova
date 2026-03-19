# Slice Expressions and Assignment Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-01-33-44-slice-expressions-assignment`

## Validation Goal

Verify that `nova-go` now supports simple slice expressions on `[]T` plus slice element assignment, while keeping the frontend, semantic layer, bytecode, VM, and CLI inspection surfaces aligned.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/slice_windows.go`
- `cargo run -- dump-ast examples/slice_windows.go`
- `cargo run -- dump-bytecode examples/slice_windows.go`
- `cargo run -- check <temp-full-slice-source>`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeed with the current local toolchain; no environment repair was needed.
- `cargo test` now passes with unit coverage for parser, semantic analysis, and VM execution of slice windows plus CLI coverage for the new `slice_windows` example and diagnostics.
- `cargo run -- run examples/slice_windows.go` prints:
  - `3 9 7`
  - `1 9 3 7`
- The `run` output confirms two important behaviors together: simple slice expressions on `[]T` work, and overlapping slice windows observe indexed writes through shared backing storage.
- `cargo run -- dump-ast examples/slice_windows.go` renders `values[:2]`, `head[1:4]`, and indexed assignment statements clearly, confirming the frontend surface stays readable at the CLI boundary.
- `cargo run -- dump-bytecode examples/slice_windows.go` shows `slice low=false high=true`, `slice low=true high=true`, and `set-index`, confirming lowering exposes the new slice operations explicitly instead of hiding them in existing instructions.
- An invalid source that uses `values[0:2:3]` is rejected before semantic analysis with `full slice expressions are not supported`, confirming the explicitly deferred full-slice form fails with a targeted diagnostic.

## Remaining Risks

- String slice execution is still deferred because the runtime string model is `String`-based rather than byte-addressed like Go strings.
- `append` still returns a fresh slice value instead of modeling Go's capacity-sensitive reuse behavior.
- The runtime now preserves shared slice windows, but broader composite-value aliasing semantics remain incomplete outside the current slice subset.
