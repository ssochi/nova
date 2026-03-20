# Strings and Bytes Clone Seams Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-10-32-34-strings-bytes-clone-seams`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/strings_bytes_clone.go` to exercise `strings.Clone`, nil-byte-slice cloning, non-nil empty-byte-slice cloning, and non-empty byte-slice cloning through the real CLI path.
2. Ran `cargo run -- dump-ast examples/strings_bytes_clone.go` to confirm the new package-backed calls remain visible as direct source-level operations.
3. Ran `cargo run -- dump-bytecode examples/strings_bytes_clone.go` to confirm the package-call path stays explicit and debuggable.
4. Ran `cargo run -- check` on two failure paths to inspect wrong-type arguments for `strings.Clone` and `bytes.Clone`.

## Positive Experience

- The feature fits cleanly into the existing package seam: no new syntax, no hidden lowering, and the example stays small while still exercising the important nil boundary.
- The AST and bytecode views remain useful because both helpers show up as dedicated `call-package` operations instead of disappearing into a builtin shortcut.
- The nil-vs-empty distinction for `bytes.Clone` is easy to understand from the CLI because the example prints `true` for `nil` input and `false` for a non-nil empty slice.
- The diagnostic path stays sharp: wrong argument types fail during `check` with direct package-argument messages before execution starts.

## Issues and Severity

- Medium: `strings.Clone` cannot prove fresh allocation behavior because the current VM does not expose pointer identity or allocation observability.
- Low: runtime-heavy unit test files are approaching the repository size ceiling, so future package slices should keep the same focused-test discipline used here.

## Conclusion and Next Recommended Steps

The real CLI path improved in a concrete way: `strings.Clone` and `bytes.Clone` now work end to end, and the byte-slice nil-vs-empty compatibility edge stays visible instead of being flattened away. The strongest next continuation is another byte-oriented package slice with similarly clear semantics, or a quality-oriented pass that splits runtime-heavy files before the next feature pushes them over the size ceiling.
