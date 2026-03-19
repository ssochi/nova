# Byte-Oriented Strings and String Slicing CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-03-02-10-byte-strings-and-slicing`
- Entry surface: `run`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/byte_strings.go` to validate byte locals, string indexing, byte-oriented string slicing, and `copy([]byte, string)` through the normal execution path.
2. Ran `cargo run -- dump-ast examples/byte_strings.go` to inspect whether the new `byte` and string-window syntax still reads like ordinary Go in the CLI.
3. Ran `cargo run -- dump-bytecode examples/byte_strings.go` to confirm the byte-oriented path is explicit and debuggable at the VM-facing surface.
4. Ran `cargo run -- check <temp-source with copy([]int, string)>` to inspect the failure path for an invalid byte-copy attempt.

## Positive Experience

- The new example feels like a meaningful Go step forward because string manipulation is no longer limited to concatenation and package helpers; the core language now exposes byte-level indexing and slicing.
- The AST view stays readable. `byte` declarations and string windows appear almost exactly as written instead of collapsing into hidden special forms.
- The bytecode dump is materially better than a generic `index` / `slice` story because it calls out `index string`, `slice string`, and `make-slice byte`, which is enough to debug the feature without reading implementation code.
- The `copy([]byte, string)` seam makes `[]byte` immediately usable even without full conversion syntax, which gives the runtime model a practical bridge into byte-oriented workflows.

## Issues and Severity

- Medium: general conversion syntax is still missing, so users cannot yet write `[]byte("text")` or `string(bytes)` even though the runtime now understands bytes.
- Medium: invalid UTF-8 string slices render approximately in CLI output because the output buffer is still UTF-8 `String` based.
- Low: byte arithmetic and richer byte-constant assignment are still intentionally narrow, so `byte` is most useful right now through indexing, zero values, and slice storage.

## Conclusion and Next Recommended Steps

This round materially improves the core string and byte workflow because users can inspect raw string bytes, slice strings by byte offsets, allocate `[]byte`, and copy string data into byte slices through the normal CLI path. The next strongest `M3` follow-up is either general string/byte conversion syntax or another runtime expansion with similar leverage, such as map/channel groundwork.
