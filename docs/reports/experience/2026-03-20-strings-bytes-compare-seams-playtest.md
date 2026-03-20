# Strings and Bytes Compare Seams Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-10-21-19-strings-bytes-compare-seams`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/strings_bytes_compare.go` to exercise equal, less-than, greater-than, and nil-vs-empty comparison behavior through the full CLI path.
2. Ran `cargo run -- dump-ast examples/strings_bytes_compare.go` to confirm the new package-backed calls remain visible as direct source-level operations.
3. Ran `cargo run -- dump-bytecode examples/strings_bytes_compare.go` to confirm the package-call path stays explicit and debuggable.
4. Ran `cargo run -- check` on two failure paths to inspect wrong-type arguments for `strings.Compare` and `bytes.Compare`.

## Positive Experience

- The CLI picks up another realistic standard-library slice without new syntax or hidden lowering rules; the feature feels like a direct extension of the existing package seam.
- The example is compact but still shows the important edge semantics, especially the `bytes.Compare(nil, empty) == 0` case that could easily be lost in implementation details.
- `dump-bytecode` remains a useful debugging surface because both helpers show up as dedicated `call-package` operations.
- The diagnostic path stays sharp: type mistakes fail during `check` with direct package-argument messages before execution starts.

## Issues and Severity

- Medium: broader text-comparison APIs that depend on Unicode or case-folding semantics still remain unavailable because the runtime is intentionally byte-oriented.
- Low: the core umbrella runtime and CLI test files are still large, so future package slices need the same focused-test-file discipline used in this round.

## Conclusion and Next Recommended Steps

The real CLI path improved in a concrete way: lexicographic `strings` / `bytes` comparisons now work end to end while keeping the VM honest about its current byte-only text model. The strongest next continuation is another byte-oriented package slice with similarly clean semantics, or a quality-oriented pass that reduces file-size pressure in the largest runtime and CLI test files.
