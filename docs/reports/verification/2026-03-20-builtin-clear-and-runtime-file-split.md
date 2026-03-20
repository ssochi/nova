# Builtin Clear and Runtime File Split Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-10-46-29-builtin-clear-and-runtime-file-split`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that builtin `clear` now works for staged `slice` and `map` values across semantic analysis, bytecode lowering, VM execution, CLI inspection, and repository file-size governance.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/builtin_clear.go`
- `cargo run -- dump-ast examples/builtin_clear.go`
- `cargo run -- dump-bytecode examples/builtin_clear.go`
- `cargo run -- check examples/builtin_clear.go`
- `cargo run -- check /tmp/nova-go-bad-clear-string.go`
- `cargo run -- check /tmp/nova-go-bad-clear-chan.go`
- `wc -l src/builtin.rs src/semantic/builtins.rs src/runtime/value.rs src/runtime/value/tests.rs src/runtime/vm/builtins.rs src/runtime/vm/tests.rs src/runtime/vm/tests/clear.rs tests/cli_builtin_clear.rs tests/cli_builtin_clear_diagnostics.rs tests/cli_execution.rs tests/cli_diagnostics.rs examples/builtin_clear.go`

## Results

- `cargo test` passed with 145 unit tests, 62 CLI diagnostic tests, 98 baseline CLI execution tests, 4 focused builtin-clear CLI tests, and 2 focused builtin-clear diagnostic tests.
- `cargo fmt` and `cargo fmt --check` both succeeded; no environment repair was needed in this round.
- `cargo run -- run examples/builtin_clear.go` printed:
  - `1 0 0 4 2 3`
  - `true 0 0`
  - `true true`
  - `0 0 false`
  This confirms slice-window clearing only zeroes the visible range, nil slices remain nil, string slices clear to empty-string zero values, and clearing a map through an alias empties the shared map without turning it into nil.
- `cargo run -- dump-ast examples/builtin_clear.go` keeps `clear(window)`, `clear(missing)`, `clear(labels)`, and `clear(alias)` explicit at the source layer.
- `cargo run -- dump-bytecode examples/builtin_clear.go` shows repeated `call-builtin clear 1`, confirming the mutating surface stays explicit instead of being lowered into hidden loops.
- `cargo run -- check examples/builtin_clear.go` succeeded, proving the package-level validation path accepts builtin `clear` without depending on runtime execution.
- Invalid checks report direct builtin diagnostics:
  - `/tmp/nova-go-bad-clear-string.go`: `argument 1 in call to builtin \`clear\` requires \`slice\` or \`map\`, found \`string\``
  - `/tmp/nova-go-bad-clear-chan.go`: `argument 1 in call to builtin \`clear\` requires \`slice\` or \`map\`, found \`chan int\``
- File-size checks stayed within the repository ceiling, and the runtime-value split materially reduced pressure: `src/runtime/value.rs` is now 683 lines and `src/runtime/value/tests.rs` 349, while `src/runtime/vm/tests.rs` remains 962 and the new focused `src/runtime/vm/tests/clear.rs` is 104.

## Remaining Risks

- Slice clearing derives zero values from the staged runtime value variants, so future richer runtime categories must keep their nil or scalar zero constructors aligned with real Go semantics.
- `src/runtime/vm/tests.rs` remains close to the repository ceiling, so future VM slices should keep adding focused submodules instead of growing the baseline file.
- Generic type-parameter `clear` behavior remains intentionally out of scope until generics are modeled.
