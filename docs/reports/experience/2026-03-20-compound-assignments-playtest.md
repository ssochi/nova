# Compound Assignments Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-06-56-53-compound-assignments`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

1. Ran `cargo run -- run examples/compound_assignments.go` to exercise compound assignments in ordinary statements, `if` / `switch` headers, classic `for` post clauses, map-index string updates, and byte-slice index updates through the real CLI entrypoint.
2. Ran `cargo run -- dump-ast examples/compound_assignments.go` to confirm the new `op=` forms stay readable in the source-facing debug path.
3. Ran `cargo run -- dump-bytecode examples/compound_assignments.go` to confirm hidden locals make indexed compound-assignment lowering inspectable without reading Rust code.
4. Ran `cargo run -- check` on two failure paths: invalid `bool` target for `+=` and non-assignable left side for `+=`.

## Positive Experience

- The CLI now feels closer to ordinary Go loop and accumulator code because `for i := 0; i < len(values); i += 1` and `total += values[i]` work directly instead of forcing verbose rewrites.
- `dump-ast` stays readable because compound assignments remain visible as source-level statements instead of disappearing into synthetic assignments.
- `dump-bytecode` remains debuggable for indexed updates because `compound$target*` and `compound$index*` locals make the single-evaluation rule obvious.
- The error paths are understandable from the CLI alone; invalid targets fail with specific messages instead of generic runtime errors.

## Issues and Severity

- Medium: the current `op=` surface is still partial because `%=` plus bitwise and shift assignment operators remain deferred.
- Medium: byte-target compound assignments are useful but still narrower than full Go because the project does not yet model broader untyped numeric constants.
- Low: the temporary-file workflow for manual `check` failure paths is still slightly awkward because the CLI currently expects file input rather than inline source snippets.

## Conclusion and Next Recommended Steps

This round removes one of the most visible remaining syntax frictions in everyday Go code: accumulators, in-place updates, and header-side counter changes can now use explicit compound assignments across the real CLI path. The strongest next continuation is to return to runtime breadth with the first `chan` slice, because the current simple-statement subset is materially less awkward and the milestone still needs broader runtime categories.
