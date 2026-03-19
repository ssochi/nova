# String and Byte Conversion CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-03-23-06-string-byte-conversions`
- Entry surface: `run`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/string_byte_conversions.go` to validate the happy path for `[]byte(string)`, byte-slice mutation, `string([]byte)`, and empty round-trips.
2. Ran `cargo run -- dump-ast examples/string_byte_conversions.go` to check whether conversion syntax still reads like ordinary Go instead of a hidden compiler form.
3. Ran `cargo run -- dump-bytecode examples/string_byte_conversions.go` to confirm the conversion path is explicit enough to debug without reading Rust code.
4. Ran `cargo run -- check <temp-source with string([]int)>` to inspect the invalid-conversion failure path.

## Positive Experience

- The new example feels like a real jump in usability because byte-oriented string work no longer depends on the narrow `copy([]byte, string)` seam; common Go-style conversion syntax now works directly.
- The AST view stays clean. `[]byte(text)` and `string(bytes)` appear exactly as written, which makes the language surface easier to trust.
- The bytecode dump is clear enough for debugging because conversions show up as dedicated `convert string->[]byte` and `convert []byte->string` instructions instead of vanishing into builtin lowering.
- Mutating the converted byte slice while the original string stays unchanged gives the runtime model a believable Go-like feel for everyday string/byte workflows.

## Issues and Severity

- Medium: conversion coverage is still narrow, so users still cannot write numeric conversions such as `byte(65)` or broader Go conversion forms.
- Medium: `[]byte(string)` currently uses exact-length capacity, while real Go leaves capacity implementation-specific.
- Low: invalid UTF-8 output remains approximate because CLI rendering still buffers through a Rust `String`.

## Conclusion and Next Recommended Steps

This round closes a meaningful compatibility gap because the compiler now supports the normal Go surface for moving between strings and byte slices, and the debug surfaces expose that path clearly. The next strongest `M3` follow-up is another core runtime expansion with similar leverage, such as map runtime groundwork and `make(map[K]V)` planning.
