# Strings and Bytes LastIndex / Byte Search Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-10-08-16-strings-bytes-last-index-byte-search`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/strings_bytes_last_index.go` to exercise the full happy path for last-index substring search, first/last byte search, and nil-slice byte-search behavior in one CLI flow.
2. Ran `cargo run -- dump-ast examples/strings_bytes_last_index.go` to confirm the new package-backed calls remain visible as source-level operations.
3. Ran `cargo run -- dump-bytecode examples/strings_bytes_last_index.go` to confirm the package-call path stays explicit and debuggable.
4. Ran `cargo run -- check` on two failure paths to inspect wrong-type byte arguments for `strings.IndexByte` and `bytes.LastIndexByte`.

## Positive Experience

- The CLI now covers another realistic standard-library slice without any new syntax or hidden lowering rules; it feels like a straight extension of the existing package seam.
- The example is compact but still shows the important edge semantics: empty-needle `LastIndex` and nil-slice `bytes.LastIndex` are both visible in source.
- `dump-bytecode` remains a good debugging surface because the new helpers show up as dedicated `call-package` operations instead of disappearing into opaque runtime fallbacks.
- The diagnostic path is sharp: wrong byte-argument types fail during `check` with direct package-argument messages before execution starts.

## Issues and Severity

- Medium: the package seam is broader, but search helpers that depend on Unicode or rune classes still remain unavailable because the runtime is intentionally byte-oriented.
- Low: the core umbrella CLI integration files are still large, so future package slices need the same focused-test-file discipline used in this round.

## Conclusion and Next Recommended Steps

The real CLI path improved in a concrete way: search-oriented `strings` / `bytes` usage now looks more like real Go while keeping the VM model honest about its byte-only string semantics. The strongest next continuation is another byte-oriented package slice with low semantic risk, or a deliberate quality-oriented pass that keeps large contract and test files from drifting back toward the repository limit.
