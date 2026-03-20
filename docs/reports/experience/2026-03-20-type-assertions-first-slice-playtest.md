# Type Assertions First Slice Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-13-55-46-type-assertions-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

- Ran the real CLI against `examples/type_assertions.go` through `run`, `dump-ast`, `dump-bytecode`, and `check`.
- Probed one temporary nil-interface assertion program through `run` to confirm the runtime panic wording for `value.(string)` on a nil interface value.
- Probed one temporary mismatched-dynamic-type assertion program through `run` to confirm runtime mismatch panics for `value.([]byte)` when the boxed payload is a `string`.
- Probed one temporary non-interface assertion program through `check` to confirm compile-time rejection.
- Probed one temporary `value.(type)` program through `check` to keep unsupported type-switch syntax explicit at the CLI boundary.
- Scope boundary: this is a focused type-assertion slice playtest, not a milestone-closeout full CLI blackbox pass.

## Positive Experience

- The feature reads cleanly from the CLI: `dump-ast` shows source syntax directly, and `dump-bytecode` keeps the runtime step explicit through `type-assert <type>`.
- The example demonstrates the highest-value edges quickly: concrete assertion success, typed-nil slice preservation, and `value.(any)` all show up in one short path.
- Runtime failures are understandable from the terminal without reading internals because nil-interface and mismatched-type failures produce distinct interface-conversion messages.
- The parser-level rejection for `.(type)` makes the current scope boundary obvious instead of silently turning unsupported syntax into a later semantic mystery.

## Issues and Severity

- Medium: comma-ok assertions and type switches are still unavailable, so some idiomatic Go interface-consumption patterns remain blocked.
- Medium: interface-conversion panic text is intentionally staged and may diverge from real Go spelling for future richer runtime types.
- Low: only empty-interface assertions exist today; method-bearing interfaces and assertion-to-interface-implementation checks are still deferred.

## Conclusion and Next Recommended Steps

The staged `x.(T)` slice is now usable end to end and materially improves the `any` runtime surface because boxed interface values can finally flow back into concrete runtime values through the CLI without hidden lowering. The strongest adjacent continuation is comma-ok assertions and type switches on top of the new explicit assertion seam, or broader interface type-system work that makes assertion targets more interesting than empty-interface boxing alone.
