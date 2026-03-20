# Strings and Bytes Index/Trim Seams Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-09-49-02-strings-bytes-index-trim`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/strings_bytes_search_trim.go` to exercise the full happy path for string search/suffix/trim helpers plus byte-slice search/suffix/trim helpers and nil-slice preservation in one CLI flow.
2. Ran `cargo run -- dump-ast examples/strings_bytes_search_trim.go` to confirm the new package-backed API slice remains legible as source rather than disappearing into generic call noise.
3. Ran `cargo run -- dump-bytecode examples/strings_bytes_search_trim.go` to confirm the package-call path remains explicit and debuggable.
4. Ran `cargo run -- check /tmp/nova-go-bad-search-trim.go` to exercise a real error path for mistyped package arguments.

## Positive Experience

- The CLI now supports another realistic standard-library slice without needing any new syntax or hidden lowering rules; the package seam feels consistent with earlier `strings` / `bytes` work.
- The example is easy to read as a user path because it shows both ordinary success cases and the important `bytes.TrimPrefix(nil, []byte(""))` nil-preservation edge directly in source.
- `dump-bytecode` still carries real debugging value: the new helpers remain visible as first-class `call-package` instructions instead of collapsing into opaque runtime special cases.
- The diagnostic path stays understandable because the wrong-type `strings.Index` call fails before execution with a direct argument-type message.

## Issues and Severity

- Medium: the package seam is broader, but common split helpers are still absent because the project should not fake UTF-8/rune-sensitive behavior while strings remain byte-oriented.
- Low: byte-slice trim behavior is correct and inspectable, but users still need to know that `[]byte("...")` lowers through the narrow explicit conversion path rather than a richer general conversion system.

## Conclusion and Next Recommended Steps

The real CLI path improved again: metadata-backed package extension remains cheap, readable, and testable, while byte-slice nil/view behavior now covers a more convincing subset of real Go helpers. The strongest next continuation is either grouped parameter-name shorthand for more realistic source compatibility or another package/API slice that stays clear of rune-sensitive behavior until the runtime models it deliberately.
