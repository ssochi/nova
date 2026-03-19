# Map Literals and Delete CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-04-07-11-map-literals-delete`
- Entry surface: `run`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/map_literals.go` to validate keyed map literals, empty map literals, `delete`, and nil-map deletion through the normal CLI entrypoint.
2. Ran `cargo run -- dump-ast examples/map_literals.go` to verify that literal syntax and `delete` calls remain recognizable at the source-facing inspection layer.
3. Ran `cargo run -- dump-bytecode examples/map_literals.go` to confirm map construction and deletion are explicit enough to debug without opening the Rust implementation.
4. Ran `cargo run -- check <temp-source with map literal value type mismatch>` to inspect the semantic failure path for literal entry typing.
5. Ran `cargo run -- check <temp-source with delete key type mismatch>` to inspect the semantic failure path for builtin `delete`.

## Positive Experience

- This slice makes maps feel much more natural because common setup no longer requires `make` plus a chain of assignments before any useful work can happen.
- The AST view stays trustworthy: the CLI prints the literal shape almost exactly as written, so the new surface does not feel like a compiler-only special form.
- The bytecode dump is still readable after the feature expansion because `build-map` and `call-builtin delete 2` make the mutating runtime path obvious.
- Nil-map deletion being a no-op removes a rough edge from the CLI experience; users can now clean up maps without first guarding against zero-value declarations.

## Issues and Severity

- Medium: duplicate constant map keys do not yet produce the real Go compile-time diagnostic; the current staged behavior is deterministic last-write-wins.
- Medium: map support still stops short of comma-ok lookups and `range`, so the overall Go map experience remains partial.
- Low: deterministic map rendering is useful for tests and CLI inspection, but it still differs from Go's unspecified iteration order.

## Conclusion and Next Recommended Steps

This round materially improves the day-to-day CLI experience because maps can now be created inline and mutated with the standard `delete` builtin while remaining visible across every debug surface. The next strongest `M3` follow-up is either another map-compatibility slice for comma-ok and `range`, or a lateral move into channel/runtime groundwork if the milestone needs a broader runtime category next.
