# Variadic Functions and Explicit Ellipsis Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-09-21-38-variadic-functions-ellipsis`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged variadic function declarations, explicit final-argument `...` spreading, and builtin `append` spread behavior including the narrow `append([]byte, string...)` path across parsing, semantic analysis, bytecode lowering, VM execution, and CLI diagnostics.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/variadic.go`
- `cargo run -- dump-ast examples/variadic.go`
- `cargo run -- dump-bytecode examples/variadic.go`
- `cargo run -- check <temp source with println(total(1, values...))>`
- `find src tests docs examples -type f \( -name '*.rs' -o -name '*.md' -o -name '*.go' \) -print0 | xargs -0 wc -l | sort -n | tail -n 20`

## Results

- `cargo test` passed after adding parser, builtin, semantic, runtime, CLI execution, and CLI diagnostic coverage for variadic declarations plus explicit spread calls.
- `cargo run -- run examples/variadic.go` produced:
  - `true 0`
  - `1`
  - `false 2`
  - `6`
  - `false 2`
  - `6`
  - `true 0`
  - `4`
  - `go-nova!`
- `dump-ast` renders the staged source forms directly, including `func total(prefix int, values ...int) int`, `total(1, values...)`, and `append(bytes, "!"...)`.
- `dump-bytecode` keeps the new behavior explicit through `function 0: total (params=1 + ...int, returns=int, ...)`, `call-function-spread 0 1`, and `call-builtin-spread append 1`.
- The invalid CLI path `println(total(1, values...))` for a zero-fixed-prefix variadic function fails with `function \`total\` with \`...\` requires 0 fixed arguments before the spread value, found 1`, confirming the real Go fixed-prefix rule stays enforced.
- Modified file sizes remain within the repository limit; the largest source file after this slice is `src/frontend/ast.rs` at 978 lines.

## Remaining Risks

- Explicit `...` is still intentionally narrow: package-backed variadic slice forwarding remains deferred because the project does not yet model `[]any` / interfaces.
- Grouped parameter-name shorthand such as `func f(a, b int)` still remains outside the current parser surface even though variadic final parameters now exist.
- The current spread model remains separate from staged multi-result call forwarding by design; later work should preserve that distinction instead of flattening both into one generic expansion path.
