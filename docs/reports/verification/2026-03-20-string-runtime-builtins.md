# String Runtime and Builtin Contracts Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-00-09-59-string-runtime-builtins`

## Validation Goal

Verify that `nova-go` now supports strings, centralized builtin call contracts, and the expanded builtin execution path across semantic analysis, bytecode lowering, VM execution, and CLI inspection.

## Execution Method

- `cargo test`
- `cargo run -- run examples/strings.go`
- `cargo run -- dump-tokens examples/strings.go`
- `cargo run -- dump-ast examples/strings.go`
- `cargo run -- dump-bytecode examples/strings.go`
- `cargo run -- check examples/strings.go`
- `cargo run -- check <temp-invalid-len-source>`
- `cargo fmt --check`

## Results

- `cargo test` passes with nineteen integration tests, including string execution, token / AST / bytecode inspection, builtin contract failures, and unterminated string lexing.
- `cargo run -- run examples/strings.go` prints `hello, nova! 11` followed by `true`, proving string return values, concatenation, `print`, `println`, `len`, and string equality on the VM path.
- `cargo run -- dump-tokens examples/strings.go` shows `string("hello, ")`, `string("nova")`, and `identifier(len)`, proving the lexer and token renderer expose the new surface.
- `cargo run -- dump-ast examples/strings.go` renders string literals, builtin calls, and string equality before lowering.
- `cargo run -- dump-bytecode examples/strings.go` shows `push-string`, `concat`, `call-builtin print 1`, and `call-builtin len 1`, confirming the string-aware lowering path.
- `cargo run -- check examples/strings.go` succeeds at package-level semantic validation without requiring runtime entrypoint execution.
- An invalid builtin call with `len(1)` is rejected during semantic analysis with `argument 1 in call to builtin len requires string, found int`.
- `cargo fmt --check` still fails because `cargo-fmt` / `rustfmt` is not installed for the local stable toolchain.

## Remaining Risks

- String literal coverage is still narrow: there is no raw string literal support and only a small escape subset exists.
- Builtin coverage remains intentionally small and does not yet model standard library packages or import resolution.
- `print` / `println` formatting is simplified and should not yet be treated as Go-exact behavior.
- Diagnostics still lack source spans or snippets for semantic and runtime failures.
