# Strings and Bytes LastIndex / Byte Search Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-10-08-16-strings-bytes-last-index-byte-search`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged `strings` / `bytes` last-index and byte-search helpers across semantic analysis, bytecode lowering, VM execution, CLI inspection, and file-size governance.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/strings_bytes_last_index.go`
- `cargo run -- dump-ast examples/strings_bytes_last_index.go`
- `cargo run -- dump-bytecode examples/strings_bytes_last_index.go`
- `cargo run -- check examples/strings_bytes_last_index.go`
- `cargo run -- check /tmp/nova-go-bad-last-index-byte.go`
- `cargo run -- check /tmp/nova-go-bad-bytes-last-index-byte.go`
- `wc -l src/package.rs src/semantic/packages.rs src/semantic/packages/tests.rs src/runtime/vm/packages.rs src/runtime/vm/support.rs tests/cli_execution.rs tests/cli_diagnostics.rs tests/cli_strings_bytes_last_index.rs tests/cli_strings_bytes_last_index_diagnostics.rs examples/strings_bytes_last_index.go`

## Results

- `cargo test` passed with 129 unit tests, 64 CLI diagnostic tests, and 102 CLI execution tests, including new semantic-contract, runtime-helper, CLI happy-path, and CLI diagnostic coverage for `strings.LastIndex`, `strings.IndexByte`, `strings.LastIndexByte`, `bytes.LastIndex`, `bytes.IndexByte`, and `bytes.LastIndexByte`.
- `cargo fmt` and `cargo fmt --check` both succeeded; no environment repair was needed in this round.
- `cargo run -- run examples/strings_bytes_last_index.go` printed:
  - `8`
  - `10`
  - `4`
  - `9`
  - `true 0`
  - `8`
  - `4`
  - `9`
  This confirms last-match substring search, empty-needle `LastIndex` behavior, first/last byte search on strings and byte slices, plus nil-slice behavior for `bytes.LastIndex(nil, []byte(""))`.
- `cargo run -- dump-ast examples/strings_bytes_last_index.go` renders all six new package calls directly, including `strings.LastIndexByte(text, text[1])` and `bytes.LastIndex(raw, []byte(""))`, keeping the source-facing path inspectable.
- `cargo run -- dump-bytecode examples/strings_bytes_last_index.go` shows explicit `call-package strings.LastIndex 2`, `call-package strings.IndexByte 2`, `call-package strings.LastIndexByte 2`, `call-package bytes.LastIndex 2`, `call-package bytes.IndexByte 2`, and `call-package bytes.LastIndexByte 2`, confirming the new surface reuses the existing package-call path cleanly.
- `cargo run -- check examples/strings_bytes_last_index.go` succeeded, proving the package-level validation path accepts the new helpers without runtime execution.
- The invalid paths report targeted package diagnostics:
  - `/tmp/nova-go-bad-last-index-byte.go`: `argument 2 in call to \`strings.IndexByte\` requires \`byte\`, found \`string\``
  - `/tmp/nova-go-bad-bytes-last-index-byte.go`: `argument 2 in call to \`bytes.LastIndexByte\` requires \`byte\`, found \`int\``
- File-size checks stayed within the repository ceiling and this round reduced pressure on one hotspot by splitting semantic package tests: `src/semantic/packages.rs` 841 lines, `src/semantic/packages/tests.rs` 151, `tests/cli_execution.rs` 901, `tests/cli_diagnostics.rs` 832, `src/runtime/vm/support.rs` 524, `src/runtime/vm/packages.rs` 252, `src/package.rs` 99, `tests/cli_strings_bytes_last_index.rs` 43, `tests/cli_strings_bytes_last_index_diagnostics.rs` 33, and `examples/strings_bytes_last_index.go` 23.

## Remaining Risks

- Unicode- or rune-aware search helpers remain intentionally deferred; adding them on top of the current byte-oriented runtime would overstate compatibility.
- The package layer is still metadata-backed only; there is no filesystem import graph or real standard library loading yet.
- `tests/cli_execution.rs` and `tests/cli_diagnostics.rs` remain large even though this round avoided extending them, so future CLI slices should keep using focused integration files.
