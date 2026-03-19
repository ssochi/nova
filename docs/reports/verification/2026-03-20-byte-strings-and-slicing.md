# Byte-Oriented Strings and String Slicing Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-03-02-10-byte-strings-and-slicing`

## Validation Goal

Verify that `nova-go` now supports byte-oriented runtime strings, string indexing and slicing, the `byte` type, and builtin `copy([]byte, string)` while keeping parser, semantic analysis, bytecode lowering, VM execution, and CLI inspection surfaces aligned.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/byte_strings.go`
- `cargo run -- dump-ast examples/byte_strings.go`
- `cargo run -- dump-bytecode examples/byte_strings.go`
- `cargo run -- check <temp-source with copy([]int, string)>`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeed with the current local toolchain.
- `cargo test` now passes with 33 unit tests plus 61 CLI integration tests, including new coverage for `byte`, string indexing and slicing, `copy([]byte, string)`, and the refactored `runtime::vm` / `semantic::analyzer` test modules.
- `cargo run -- run examples/byte_strings.go` prints:
  - `0 103 oph 6 112 3`
  - `195 169 2`
- The run output proves five behaviors together: typed `byte` locals synthesize zero values, string indexing returns decimal byte values, simple string slicing is byte-oriented, `copy([]byte, string)` fills a byte slice and reports the copied count, and `len(string)` still reports byte length for multibyte literals.
- `cargo run -- dump-ast examples/byte_strings.go` renders `var marker byte`, `var first byte = word[0]`, `var middle = word[1:4]`, and `var buf = make([]byte, len(word))` directly, confirming the source-oriented CLI surface stays readable for the new `byte` and string-window path.
- `cargo run -- dump-bytecode examples/byte_strings.go` shows `push-byte 0`, `index string`, `slice string low=true high=true`, `make-slice byte cap=len`, and `call-builtin copy 2`, confirming the byte-oriented path is explicit at the VM-facing debug surface.
- The failure path `cargo run -- check <temp-source with copy([]int, "no")>` reports `argument 2 in call to builtin \`copy\` requires \`[]int\`, found \`string\``, confirming the `[]byte` <- `string` special case stays narrow and type-checked before execution.

## Remaining Risks

- General conversion syntax such as `[]byte("text")` and `string(bytes)` is still unsupported, so byte-oriented work currently relies on string indexing, slicing, and the narrow `copy([]byte, string)` seam.
- CLI rendering of invalid UTF-8 byte sequences is still approximate because VM output is stored in a Rust `String`.
- `append([]byte, string...)` remains deferred until the parser and call model grow explicit variadic forwarding support.
