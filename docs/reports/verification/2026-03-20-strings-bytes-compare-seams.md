# Strings and Bytes Compare Seams Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-10-21-19-strings-bytes-compare-seams`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged `strings.Compare` and `bytes.Compare` across semantic analysis, bytecode lowering, VM execution, CLI inspection, and file-size governance.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/strings_bytes_compare.go`
- `cargo run -- dump-ast examples/strings_bytes_compare.go`
- `cargo run -- dump-bytecode examples/strings_bytes_compare.go`
- `cargo run -- check examples/strings_bytes_compare.go`
- `cargo run -- check /tmp/nova-go-bad-strings-compare.go`
- `cargo run -- check /tmp/nova-go-bad-bytes-compare.go`
- `wc -l src/package.rs src/semantic/packages.rs src/semantic/packages/tests.rs src/runtime/vm/packages.rs src/runtime/vm/support.rs src/runtime/vm/tests.rs tests/cli_execution.rs tests/cli_diagnostics.rs tests/cli_strings_bytes_compare.rs tests/cli_strings_bytes_compare_diagnostics.rs examples/strings_bytes_compare.go`

## Results

- `cargo test` passed with 134 unit tests, 62 CLI diagnostic tests, 98 CLI execution tests, and 6 focused compare CLI tests, including new semantic-contract, runtime-helper, VM execution, example, and diagnostic coverage for `strings.Compare` and `bytes.Compare`.
- `cargo fmt` and `cargo fmt --check` both succeeded; no environment repair was needed in this round.
- `cargo run -- run examples/strings_bytes_compare.go` printed:
  - `0`
  - `-1`
  - `1`
  - `0`
  - `-1`
  - `1`
  This confirms equal, less-than, and greater-than ordering for strings, plus the staged Go rule that `bytes.Compare(nil, empty)` yields equality while non-empty `[]byte` remains greater than nil/empty.
- `cargo run -- dump-ast examples/strings_bytes_compare.go` renders both package calls directly, including `bytes.Compare(nil, empty)` and `bytes.Compare([]byte("vm"), nil)`, keeping the source-facing path inspectable.
- `cargo run -- dump-bytecode examples/strings_bytes_compare.go` shows explicit `call-package strings.Compare 2` and `call-package bytes.Compare 2`, confirming the new surface reuses the existing package-call path without hidden lowering.
- `cargo run -- check examples/strings_bytes_compare.go` succeeded, proving the package-level validation path accepts the new helpers without runtime execution.
- The invalid paths report targeted package diagnostics:
  - `/tmp/nova-go-bad-strings-compare.go`: `argument 1 in call to \`strings.Compare\` requires \`string\`, found \`[]byte\``
  - `/tmp/nova-go-bad-bytes-compare.go`: `argument 2 in call to \`bytes.Compare\` requires \`[]byte\`, found \`string\``
- File-size checks stayed within the repository ceiling: `src/semantic/packages.rs` 888 lines, `src/runtime/vm/tests.rs` 920, `tests/cli_execution.rs` 901, `tests/cli_diagnostics.rs` 832, `src/runtime/vm/support.rs` 553, `src/runtime/vm/packages.rs` 262, `src/package.rs` 103, `tests/cli_strings_bytes_compare.rs` 38, `tests/cli_strings_bytes_compare_diagnostics.rs` 33, and `examples/strings_bytes_compare.go` 17.

## Remaining Risks

- Unicode- or rune-aware comparison helpers remain intentionally deferred; the current runtime only models byte-oriented semantics.
- The package layer is still metadata-backed only; there is no filesystem import graph or real standard library loading yet.
- `src/runtime/vm/tests.rs`, `tests/cli_execution.rs`, and `tests/cli_diagnostics.rs` remain near the repository size ceiling, so future slices should continue using focused test files.
