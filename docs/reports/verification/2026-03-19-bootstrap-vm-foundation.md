# Bootstrap VM Foundation Verification

## Basic Information

- Date: `2026-03-19`
- Related milestone: `M1-bootstrap-vm-execution`
- Related plan: `2026-03-19-23-19-47-bootstrap-vm-foundation`

## Validation Goal

Verify that the new Rust CLI can parse a minimal Go subset, compile it to bytecode, and execute it on the VM.

## Execution Method

- `cargo test`
- `cargo run -- run examples/hello.go`
- `cargo run -- dump-ast examples/hello.go`
- `cargo run -- dump-tokens examples/hello.go`
- `cargo run -- dump-bytecode examples/arithmetic.go`
- `cargo run -- check examples/hello.go`

## Results

- Integration tests cover the happy path, bytecode inspection, syntax-only check flow, and missing-entry failure.
- CLI execution prints `42` for `examples/hello.go`.
- AST and token dump commands both produce readable inspection output through the real CLI.
- Bytecode dump shows stack instructions, local slot writes, and builtin invocation.
- `check` validates the sample source without executing it.
- `cargo fmt` was not executed because the local Rust toolchain does not currently include `rustfmt`.

## Remaining Risks

- The frontend only supports a narrow bootstrap subset of Go.
- There is no semantic analysis stage yet; some future errors still surface during bytecode compilation.
- Only integer values and `println` are supported in the VM.
- Formatting remains a manual discipline until `rustfmt` is installed in the environment.
