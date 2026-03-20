# Recover First Slice Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-13-28-50-recover-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

- Ran the real CLI against `examples/recover.go` through `run`, `dump-ast`, `dump-bytecode`, and `check`.
- Probed one temporary `defer recover(); panic("boom")` program through `run` to confirm deferred builtin `recover()` stays non-recovering.
- Probed one temporary helper-based `recover()` program through `run` to confirm helper calls invoked by a deferred function still return nil and do not stop the panic.
- Probed one temporary invalid `recover(1)` program through `check` to confirm the builtin arity diagnostic.
- Scope boundary: this is a focused recover slice playtest, not a milestone-closeout full CLI blackbox pass.

## Positive Experience

- The new example is compact but covers the real user questions quickly: nil recovery outside panic, direct recovery, helper non-recovery, zero-value unnamed returns, and named-result preservation all show up in one CLI path.
- `dump-bytecode` stays useful because `call-builtin recover 0` and typed `panic <type>` instructions make the recoverable path easy to inspect without reading the VM.
- The runtime no longer feels like panic and ordinary return are separate systems; the visible behavior around deferred prints and recovered returns is coherent from the CLI.
- The focused diagnostic path stays sharp: invalid `recover` arity, deferred builtin non-recovery, and helper non-recovery all fail in understandable ways.

## Issues and Severity

- Medium: recovered runtime panic payloads still surface as boxed strings instead of Go's concrete runtime panic object types.
- Medium: common closure-based `defer func() { recover() }()` patterns remain unavailable until function literals and closures exist.
- Low: named-result recovery currently relies on the existing result-slot initialization shape rather than explicit compiled-function metadata.

## Conclusion and Next Recommended Steps

The staged `recover()` slice is usable and materially improves panic/defer realism because direct deferred user-defined functions can now stop active panics and return meaningful `any` payloads through the ordinary CLI flow. The strongest adjacent continuation is either richer runtime panic payload typing on top of the new recover path, or another standard-library/runtime seam that benefits from the improved panic/defer behavior without reopening closures yet.
