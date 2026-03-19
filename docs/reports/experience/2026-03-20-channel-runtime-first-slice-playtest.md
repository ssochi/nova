# Channel Runtime First Slice Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-07-17-39-channel-runtime-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

1. Ran `cargo run -- run examples/channels.go` to exercise nil-channel equality, buffered allocation, send statements, receive expressions, `close`, and channel-aware `len` / `cap` through the real CLI entrypoint.
2. Ran `cargo run -- dump-ast examples/channels.go` to confirm `chan`, send, and receive stay readable in the source-facing debug path.
3. Ran `cargo run -- dump-bytecode examples/channels.go` to confirm channel allocation and operations lower into dedicated instructions instead of disappearing behind generic builtin dispatch.
4. Ran `cargo run -- check` on two failure paths: invalid send-value type mismatch and invalid `close` target.

## Positive Experience

- The CLI now handles a real new runtime category instead of only collection-like composites; `chan int` feels first-class from parsing through execution.
- `dump-ast` remains readable because send and receive stay visible as `ready <- 4` and `<-ready` instead of becoming synthetic helper calls.
- `dump-bytecode` is especially useful here because `push-nil-chan`, `make-chan`, `send`, and `receive` make the staged runtime model obvious without opening Rust code.
- Closed-channel receive behavior is easy to observe from the CLI: draining buffered values and then yielding the element zero value is visible in one short example.

## Issues and Severity

- Medium: blocking channel behavior is intentionally incomplete because the single-threaded VM cannot model goroutine wakeups yet; would-block cases currently surface as runtime errors.
- Medium: only bidirectional `chan T` exists in the staged type system; directional channels, channel `range`, and comma-ok receive are still absent.
- Low: the current runtime error messages are pragmatic rather than panic-accurate, so they are useful for debugging but not yet faithful to full Go failure semantics.

## Conclusion and Next Recommended Steps

This round materially broadens the runtime model: channels are now a real typed value category with explicit nil state, buffered storage, and inspectable send / receive / close lowering across the full CLI path. The strongest next continuation is to decide whether `M3` should return to broader package-backed standard-library seams or deliberately plan the next concurrency-adjacent slice such as channel follow-up work tied to scheduler or multi-result design.
