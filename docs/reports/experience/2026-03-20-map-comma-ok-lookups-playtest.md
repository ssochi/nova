# Map Comma-Ok Lookups Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-05-11-15-map-comma-ok-and-literal-diagnostics`
- Focus: real CLI experience for staged comma-ok `map` lookups and duplicate constant-key diagnostics

## Experience Path

1. Ran `cargo run -- run examples/map_lookup.go` to exercise nil-map reads, populated-map reads, blank left-hand-side handling, and same-block short redeclaration through the real CLI entrypoint.
2. Ran `cargo run -- dump-ast examples/map_lookup.go` to confirm the new statement surface remains readable without implementation context.
3. Ran `cargo run -- dump-bytecode examples/map_lookup.go` to inspect whether the new lowering path stays visible at the VM-facing debug layer.
4. Ran `cargo run -- check <temp-source with duplicate "go" map literal keys>` as the first error path.
5. Ran `cargo run -- check <temp-source with value, ok := values[0]>` as the second error path.
6. Ran `cargo run -- check <temp-source with no new variables on comma-ok :=>` as the third error path.

## Positive Experience

- The happy-path sample feels much closer to ordinary Go map usage because presence checks no longer require awkward sentinel comparisons against the zero value. Nil maps, missing keys, and present keys all read clearly through the CLI.
- The debug surfaces stay coherent. `dump-ast` preserves the source spelling of the comma-ok statement, while `dump-bytecode` exposes the dedicated `lookup-map` instruction instead of burying the behavior in implicit runtime conventions.
- The failure paths are understandable without code reading. Users get direct explanations for duplicate constant keys, non-map right-hand sides, and invalid short redeclarations.

## Issues and Severity

- Medium: The new lookup surface is still statement-only, so common Go patterns such as `if value, ok := counts["go"]; ok { ... }` remain unavailable.
- Low: `dump-bytecode` shows the reversed store order that falls out of the stack-machine lowering (`ok` stored before `value`), which is correct but mildly non-obvious at first glance.
- Medium: Duplicate-key diagnostics currently stop at literal scalar keys; broader constant-expression duplicates will still need future work.

## Conclusion and Next Recommended Steps

This round materially improves real CLI usability because `map` reads now cover the most common Go presence-check idiom, and invalid literals fail early with clearer diagnostics. The strongest next `M3` continuation is to build on this explicit multi-result groundwork with `if` initializers or broader control-flow ergonomics, or to pivot into the first `chan` runtime slice if concurrency leverage becomes more important than more map polish.
