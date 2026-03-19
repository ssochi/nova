# Typed Var Declarations and Zero-Value Slices Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-02-18-45-typed-var-zero-values`

## Validation Goal

Verify that `nova-go` now supports explicit typed `var` declarations, synthesized scalar zero values, and nil-slice zero values while keeping semantic analysis, bytecode lowering, VM behavior, and CLI inspection surfaces aligned.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/typed_zero_values.go`
- `cargo run -- dump-ast examples/typed_zero_values.go`
- `cargo run -- dump-bytecode examples/typed_zero_values.go`
- `cargo run -- check /tmp/nova-go-bad-typed-var.go`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeed with the current local toolchain; no environment repair was needed.
- `cargo test` now passes with new parser, semantic, runtime, and CLI coverage for explicit typed declarations, synthesized scalar zero values, nil-slice behavior, and typed-initializer diagnostics.
- `cargo run -- run examples/typed_zero_values.go` prints:
  - `0 false 0 0 0`
  - `2 4 5`
  - `1 2 4`
- The happy-path output proves three behaviors together: typed locals can start from zero values without initializers, a zero-value slice exposes `len==0` and `cap==0`, and appending to that nil slice produces a regular usable slice.
- `cargo run -- dump-ast examples/typed_zero_values.go` renders `var total int`, `var values []int`, and `var head []int = values[:1]` directly, confirming the source-oriented CLI view stays readable for the new declaration form.
- `cargo run -- dump-bytecode examples/typed_zero_values.go` shows `push-int 0`, `push-bool false`, `push-string ""`, and `push-nil-slice`, confirming explicit zero-value lowering is visible in the VM-facing inspection surface.
- The failure path `cargo run -- check /tmp/nova-go-bad-typed-var.go` reports `variable \`ready\` requires \`bool\`, found \`int\``, confirming typed declarations reject mismatched initializers during semantic analysis with a targeted diagnostic.

## Remaining Risks

- The current compiler still does not support the `make` builtin, so typed zero values improve declaration ergonomics without solving general slice allocation yet.
- Nil slices and empty slices are not externally distinguishable in the current subset because slice equality and direct `nil` expressions remain out of scope.
- Zero-value synthesis currently covers only the project's existing scalar and slice types; broader Go zero-value categories will need additional staged work.
