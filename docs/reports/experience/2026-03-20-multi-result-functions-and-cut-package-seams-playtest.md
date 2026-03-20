# Multi-Result Functions and Cut Package Seams Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-08-14-13-multi-result-functions-cut-seams`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/multi_results.go` to exercise user-defined multi-result functions, package-backed `Cut` calls, direct return forwarding, and staged reassignment in one CLI path.
2. Ran `cargo run -- dump-ast examples/multi_results.go` to confirm the source-facing surfaces stayed readable.
3. Ran `cargo run -- dump-bytecode examples/multi_results.go` to confirm function result metadata and package dispatch remained visible without reading implementation details.
4. Ran `cargo run -- check <temp source with println(pair())>` to confirm unsupported single-value usage still reports a targeted diagnostic.

## Positive Experience

- The happy-path CLI is now noticeably closer to real Go code: multi-result function signatures, `return strings.Cut(...)`, and `head, tail, found := ...` work directly.
- The parser surface remains readable in `dump-ast`; the new multi-binding forms and multi-result signatures show up explicitly instead of being flattened into hidden rewrites.
- `dump-bytecode` still exposes the runtime path clearly, including per-function result lists and explicit `call-package strings.Cut` / `call-package bytes.Cut` instructions.
- The failure path is precise and actionable rather than a generic arity or parser error.

## Issues and Severity

- Medium: `src/runtime/vm.rs` is now near the repository line-count ceiling, so the next runtime-heavy iteration should split helpers before growing the file further.
- Medium: multi-result calls are still staged and cannot yet flow through every Go context, so some natural-looking follow-up forms will still fail even though the core model exists now.

## Conclusion and Next Recommended Steps

The CLI path for staged multi-result functions is solid: real examples run, debug surfaces stay inspectable, and the unsupported contexts fail cleanly. The next sensible follow-up is to expand the multi-result consumer surface deliberately, either through more package APIs that depend on it or through the next explicitly designed runtime form such as comma-ok receive once the blocking model is planned.
