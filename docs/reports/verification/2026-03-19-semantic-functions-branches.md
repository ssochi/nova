# Semantic Functions and Branches Verification

## Basic Information

- Date: `2026-03-19`
- Related milestone: `M2-frontend-expansion`
- Related plan: `2026-03-19-23-37-05-semantic-functions-branches`

## Validation Goal

Verify that `nova-go` now performs semantic analysis before lowering, executes multi-function programs on the VM, and handles boolean branch control flow through the real CLI.

## Execution Method

- `cargo test`
- `cargo run -- run examples/functions_and_branches.go`
- `cargo run -- dump-bytecode examples/functions_and_branches.go`
- `cargo run -- check examples/functions_and_branches.go`
- `cargo run -- check /tmp/nova-go-bad-if.go`

## Results

- `cargo test` passes with eight integration tests, including user-defined calls, branch lowering, and semantic-failure coverage.
- The CLI executes `examples/functions_and_branches.go` and prints `false 11`.
- Bytecode inspection now shows per-function sections, `call-function`, comparisons, and `jump-if-false`.
- `check` succeeds for the new multi-function sample without requiring runtime execution.
- A non-boolean `if` condition is rejected during semantic analysis before bytecode lowering.
- `cargo fmt` could not be run because `cargo-fmt` / `rustfmt` is not installed for the local stable toolchain.

## Remaining Risks

- Loops are still unsupported, so milestone `M2` remains open.
- The type surface is limited to `int`, `bool`, and `void`.
- Diagnostics still lack source excerpts and exact semantic spans.
- Package loading remains single-file and does not yet model imports.
