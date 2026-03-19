# Slice Runtime Values and Layered Test Coverage Verification

## Basic Information

- Date: `2026-03-20`
- Plan: `2026-03-20-00-55-55-slice-runtime-testing`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Confirm that the compiler and VM support the first composite runtime value slice, and that the upgraded automated test layout covers both unit and CLI-visible behavior.

## Execution Method

1. Ran `cargo fmt`.
2. Ran `cargo test`.
3. Ran serial CLI checks:
   - `cargo run -- dump-tokens examples/slices.go`
   - `cargo run -- dump-ast examples/slices.go`
   - `cargo run -- dump-bytecode examples/slices.go`
   - `cargo run -- check examples/slices.go`
   - `cargo run -- run examples/slices.go`
4. Ran a serial negative CLI check with a temporary source using `values[true]`.

## Results

- `cargo fmt` completed successfully.
- `cargo test` passed.
  - Unit tests: parser, semantic analyzer, builtin contracts, and VM slice execution
  - CLI integration tests: `19` execution-path cases and `12` diagnostic cases
- `dump-tokens` showed bracket tokens, `[]int` return types, slice literals, `append`, and index syntax.
- `dump-ast` rendered the expected slice program structure:
  - `func build() []int`
  - `var values = []int{1, 2}`
  - `println(len(values), values[0], values[3])`
- `dump-bytecode` showed the new runtime instructions and builtin dispatch:
  - `build-slice 2`
  - `call-builtin append 3`
  - `index`
- `check examples/slices.go` returned `ok: examples/slices.go`.
- `run examples/slices.go` produced:
  - `4 1 5`
  - `[1 2 3 5]`
- Negative CLI validation rejected a non-integer slice index with:
  - `index expression requires \`int\`, found \`bool\``

## Remaining Risks

- Slice support is still intentionally narrow: no slicing syntax, `make`, `cap`, `copy`, nil, or element assignment.
- Runtime slices currently clone values on `append` and local loads, which is acceptable for the current VM stage but not yet optimized.
