# Channel Runtime First Slice

## Goal

Lock the official Go behavior baseline for the first staged `chan` slice so the project can add channel types, sends, receives, `make`, `len` / `cap`, and `close` without implying unsupported concurrency guarantees.

## Sources Reviewed

- Go Language Specification: `Channel types`, `Send statements`, `Receive operator`, `Length and capacity`, and `Close`
  - https://go.dev/ref/spec
- Go predeclared identifiers documentation for `make`, `len`, `cap`, and `close`
  - https://pkg.go.dev/builtin

## Confirmed Findings

- `chan T` is a first-class type constructor in Go; `make` is the normal way to allocate a non-nil channel value.
- `make(chan T, size)` creates a channel with buffer capacity `size`; if the size is omitted or zero, the channel is unbuffered.
- `len(ch)` returns the number of queued unread elements in a channel buffer; `len(nilChan)` is `0`.
- `cap(ch)` returns the channel buffer capacity; `cap(nilChan)` is `0`.
- A send statement has statement form `ch <- value`; it is not an expression.
- A receive operation has expression form `<-ch`.
- Sending to or receiving from a `nil` channel blocks forever in Go.
- Receiving from a closed channel continues to drain already-buffered values; once the buffer is empty, a receive succeeds immediately with the element zero value.
- Sending on a closed channel causes a run-time panic in Go.
- `close(ch)` is valid only for channels; closing a closed or nil channel causes a run-time panic in Go.
- Channel values are comparable. Two channel values compare equal when they refer to the same channel or when both are `nil`.

## Implementation Implications

- The staged type surface should add only bidirectional `chan T` for now; directional channel types are official Go behavior but can remain deferred because they would widen parser, type, and builtin rules immediately.
- The AST and checked model should keep send and receive explicit instead of lowering them into fake builtin calls.
- The first runtime slice should model nil-vs-allocated state explicitly, just as slice and map already do.
- `make(chan T[, size])` should lower into dedicated bytecode so buffer capacity stays explicit in `dump-bytecode`.
- `len` and `cap` should expand from `string|slice|map` to include `chan`.
- `close` should become a first-class builtin contract, but runtime handling may still be explicit.
- Because the current VM is single-threaded and has no goroutines or scheduler, full blocking semantics are not representable yet.
  Inference from the source constraints: operations that would block forever or until another goroutine runs should surface as runtime errors in this staged slice rather than pretending progress is possible.
- The staged receive surface can support only single-result receive expressions for now.
  Inference from the current checked model: comma-ok receive should stay deferred until the project has a broader multi-result expression or statement model beyond the existing map-only special case.
- Closed-channel receive should still follow the Go rule that an empty closed channel yields the element zero value without blocking.

## Deferred Questions

- When the project adds goroutines or a scheduler, should blocked channel operations become resumable VM states instead of immediate runtime errors?
- Should directional channel types land before channel `range`, or should they wait until call signatures and assignment variance are broader?
- Should channel comma-ok receive mirror the current explicit statement-scoped map lookup model, or wait for general tuple results?
