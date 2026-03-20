# Strings and Bytes Clone Seams Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-10-32-34-strings-bytes-clone-seams`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged `strings.Clone` and `bytes.Clone` across semantic analysis, bytecode lowering, VM execution, CLI inspection, and nil-vs-empty byte-slice behavior.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/strings_bytes_clone.go`
- `cargo run -- dump-ast examples/strings_bytes_clone.go`
- `cargo run -- dump-bytecode examples/strings_bytes_clone.go`
- `cargo run -- check examples/strings_bytes_clone.go`
- `cargo run -- check /tmp/nova-go-bad-strings-clone.go`
- `cargo run -- check /tmp/nova-go-bad-bytes-clone.go`
- `wc -l src/package.rs src/semantic/packages.rs src/semantic/packages/tests.rs src/runtime/vm/packages.rs src/runtime/vm/support.rs src/runtime/vm/tests.rs src/runtime/value.rs tests/cli_execution.rs tests/cli_diagnostics.rs tests/cli_strings_bytes_clone.rs tests/cli_strings_bytes_clone_diagnostics.rs examples/strings_bytes_clone.go`

## Results

- `cargo test` passed with 138 unit tests, 62 CLI diagnostic tests, 98 CLI execution tests, and 6 focused clone CLI tests, including new semantic-contract, runtime-helper, VM execution, example, and diagnostic coverage for `strings.Clone` and `bytes.Clone`.
- `cargo fmt` and `cargo fmt --check` both succeeded; no environment repair was needed in this round.
- `cargo run -- run examples/strings_bytes_clone.go` printed:
  - `nova`
  - `true`
  - `false`
  - `[103 111]`
  This confirms `strings.Clone("nova")` preserves byte content, `bytes.Clone(nil)` stays nil, `bytes.Clone([]byte{})` remains non-nil, and non-empty byte slices are copied through the package path.
- `cargo run -- dump-ast examples/strings_bytes_clone.go` renders both package calls directly, including `bytes.Clone(nil)` and `bytes.Clone(empty)`, so the nil-vs-empty boundary remains visible at the source layer.
- `cargo run -- dump-bytecode examples/strings_bytes_clone.go` shows explicit `call-package strings.Clone 1` and `call-package bytes.Clone 1`, confirming the new helpers stay inside the metadata-backed package-call path without hidden lowering.
- `cargo run -- check examples/strings_bytes_clone.go` succeeded, proving the package-level validation path accepts the new helpers without runtime execution.
- The invalid paths report targeted package diagnostics:
  - `/tmp/nova-go-bad-strings-clone.go`: `argument 1 in call to \`strings.Clone\` requires \`string\`, found \`[]byte\``
  - `/tmp/nova-go-bad-bytes-clone.go`: `argument 1 in call to \`bytes.Clone\` requires \`[]byte\`, found \`string\``
- File-size checks stayed within the repository ceiling: `src/semantic/packages.rs` 923 lines, `src/runtime/vm/tests.rs` 960, `src/runtime/value.rs` 936, `tests/cli_execution.rs` 901, `tests/cli_diagnostics.rs` 832, `src/runtime/vm/support.rs` 585, `src/runtime/vm/packages.rs` 268, `src/package.rs` 107, `tests/cli_strings_bytes_clone.rs` 37, `tests/cli_strings_bytes_clone_diagnostics.rs` 33, and `examples/strings_bytes_clone.go` 15.

## Remaining Risks

- `strings.Clone` does not expose Go allocation identity in the current VM, so the staged behavior is limited to byte-content preservation.
- `src/runtime/vm/tests.rs` and `src/runtime/value.rs` are now close to the repository size ceiling, so future runtime-heavy slices should keep splitting focused tests and helpers instead of growing those files casually.
- The package layer is still metadata-backed only; there is no filesystem import graph or real standard-library loading yet.
