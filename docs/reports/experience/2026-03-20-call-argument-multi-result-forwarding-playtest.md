# Call-Argument Multi-Result Forwarding Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-08-55-11-call-argument-multi-result-forwarding`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/call_forwarding.go` to exercise user-defined call forwarding, direct `fmt.Println(strings.Cut(...))`, and the new `CutPrefix` / `CutSuffix` seams in one CLI path.
2. Ran `cargo run -- dump-ast examples/call_forwarding.go` to confirm the source-facing story stayed readable without reading implementation details.
3. Ran `cargo run -- dump-bytecode examples/call_forwarding.go` to confirm expanded-call lowering, wrapper calls, and package dispatch stayed inspectable.
4. Ran `cargo run -- check <temp source with take(1, pair())>` to confirm prefixed multi-result arguments still fail with a targeted diagnostic instead of being half-expanded.

## Positive Experience

- The CLI now handles a real Go idiom that was previously blocked: `consume(pair())`, `fmt.Println(strings.Cut(...))`, and package-backed two-result helpers all work without inventing tuple runtime values.
- `dump-bytecode` remains useful even with the new call path. The expanded call is visible through ordinary nested call instructions rather than opaque lowering.
- The new `strings` / `bytes` `CutPrefix` / `CutSuffix` seams make the package surface feel less one-off and more like a growing compatibility slice.
- Runtime internals are easier to continue from because builtin and package dispatch no longer live in one oversized VM file.

## Issues and Severity

- Medium: call forwarding is still narrower than a casual Go reader may expect. `f(pair())` works, but `f(1, pair())` still fails by design and needs to stay documented clearly.
- Medium: the parser still requires explicit parameter types per name, so some ordinary Go signature shorthand remains unavailable in examples and future package-style helpers.

## Conclusion and Next Recommended Steps

The real CLI path is materially better: staged multi-result results can now flow directly into another call, the new package seams execute end to end, and the debug surfaces still explain what happened. The strongest next continuation is to decide whether `M3` should spend this forwarding path on more package APIs that naturally return `(T, bool)` / `(T, T, bool)` or widen adjacent Go syntax that still blocks realistic library-shaped code.
