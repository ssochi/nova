# String and Byte Conversion Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-03-23-06-string-byte-conversions`

## Validation Goal

Verify that `nova-go` now supports explicit `[]byte(string)` and `string([]byte)` conversion syntax across parsing, semantic analysis, bytecode lowering, VM execution, and CLI inspection without regressing the existing byte-oriented string runtime model.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/string_byte_conversions.go`
- `cargo run -- dump-ast examples/string_byte_conversions.go`
- `cargo run -- dump-bytecode examples/string_byte_conversions.go`
- `cargo run -- check <temp-source with string([]int)>`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeed with the current local toolchain.
- `cargo test` now passes with 38 unit tests, 24 CLI diagnostic tests, and 41 CLI execution tests, including new coverage for parsing conversions, checked conversion diagnostics, runtime conversion execution, and CLI inspection of explicit conversion bytecode.
- `cargo run -- run examples/string_byte_conversions.go` prints:
  - `4 110 97`
  - `nova Xova go`
  - `0 0`
- The run output proves four behaviors together: `[]byte(string)` exposes raw bytes, mutating the converted byte slice does not mutate the original string, `string([]byte)` rebuilds a string from visible byte-slice elements, and `[]byte("")` yields a usable empty slice that round-trips back to the empty string.
- `cargo run -- dump-ast examples/string_byte_conversions.go` renders `var bytes = []byte(text)`, `println(text, string(bytes), string([]byte("go")))`, and `var empty = []byte("")` directly, confirming the new syntax remains readable at the source-oriented CLI layer.
- `cargo run -- dump-bytecode examples/string_byte_conversions.go` shows `convert string->[]byte`, `convert []byte->string`, and `set-index`, confirming the conversion path is explicit in VM-facing debug output rather than being hidden inside builtin dispatch.
- The failure path `cargo run -- check <temp-source with string([]int)>` reports `conversion to \`string\` requires \`[]byte\`, found \`[]int\``, confirming invalid conversion sources are rejected during semantic analysis before execution.

## Remaining Risks

- Conversion support is still intentionally narrow: numeric conversions, rune-aware conversions, and broader Go conversion coverage remain deferred.
- `[]byte(string)` currently allocates exact-length capacity; real Go leaves the resulting capacity implementation-specific.
- CLI output is still lossy for invalid UTF-8 byte sequences because the buffered output model remains Rust `String` based.
