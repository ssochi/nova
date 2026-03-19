# Map Runtime Groundwork CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-03-42-30-map-runtime-groundwork`
- Entry surface: `run`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/maps.go` to validate typed nil maps, `make(map[K]V, hint)`, map reads, and map writes through the normal CLI entrypoint.
2. Ran `cargo run -- dump-ast examples/maps.go` to verify that `map[K]V` declarations and `make(map[K]V)` syntax stay readable at the source-facing inspection layer.
3. Ran `cargo run -- dump-bytecode examples/maps.go` to confirm the new runtime path is explicit enough to debug without opening the Rust implementation.
4. Ran `cargo run -- check <temp-source with map[[]int]int>` to inspect the semantic failure path for unsupported key types.
5. Ran `cargo run -- run <temp-source with nil-map assignment>` to inspect the runtime failure path for writes through a nil map.

## Positive Experience

- The new map slice feels coherent because it reaches all the way from syntax to runtime behavior instead of stopping at parsing or type-checking.
- Nil-map reads returning zero values make the first map example feel Go-like immediately, which gives typed zero-value declarations more credibility.
- The bytecode dump is clear enough for debugging because map allocation, lookup, and assignment now appear as dedicated instructions rather than opaque builtin calls.
- The negative CLI paths are understandable: unsupported key types fail during `check`, while nil-map writes fail during `run`, which keeps semantic and runtime responsibilities distinct.

## Issues and Severity

- Medium: map support is still narrow, so users still cannot write map literals, `delete`, comma-ok lookups, or `range` loops.
- Medium: debug rendering is deterministic rather than Go-like because the runtime currently uses sorted map storage for stable CLI output.
- Low: the optional `make(map[K]V, hint)` hint is accepted and validated, but the current runtime does not use it for real capacity behavior.

## Conclusion and Next Recommended Steps

This round closes a major `M3` runtime gap because `nova-go` now has the first usable map path with explicit syntax, nil-state behavior, and debug visibility. The next strongest follow-up is either deepening map usability with literals and `delete`, or moving laterally into channel/runtime groundwork once the milestone priority is re-evaluated.
