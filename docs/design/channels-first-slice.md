# First Channel Runtime Slice

## Goal

Define the staged `chan` design for milestone `M3` so buffered channels become a real runtime category without coupling the VM to goroutines, scheduling, or `select`.

## Constraints

- Only the Rust standard library may be used.
- The project remains VM-first; new behavior must be inspectable through `dump-ast` and `dump-bytecode`.
- The runtime is currently single-threaded, so true blocking semantics cannot be modeled faithfully yet.
- New composite runtime categories must keep nil-vs-allocated state explicit.
- Existing document boundaries must stay layered: roadmap for execution state, design for intent, tech for reusable architecture notes, reports for validation.

## Current Scope

- `chan T` type references in declarations, parameters, returns, and `make`.
- Explicit send statements `ch <- value` in ordinary statement position.
- Explicit receive expressions `<-ch` in expression position.
- Explicit `make(chan T[, size])` lowering into channel-specific bytecode.
- Builtin expansion so `len`, `cap`, and `close` understand channels.
- Explicit nil-channel zero values for typed declarations and `nil` coercion.
- Channel equality for matching channel types plus `nil`.
- Runtime channel storage with:
  - shared queue state
  - explicit buffer capacity
  - explicit closed state
  - explicit nil state
- Runtime errors for staged blocking cases such as:
  - send to nil channel
  - receive from nil channel
  - send to full open buffered channel
  - receive from empty open channel
  - any send or receive that would require a concurrent peer on an unbuffered channel

## Deferred Scope

- Directional channel types `chan<- T` and `<-chan T`
- Channel receive comma-ok bindings
- Channel `range`
- `go`, `select`, scheduler work, or true concurrent execution
- Panic/recover fidelity for channel failures
- Header-position send statements, label control flow, and unrelated package/runtime expansion

## Interfaces and Extension Hooks

- Frontend:
  - Add `TypeRef::Chan`, `Statement::Send`, and `Expression::Receive`.
  - Keep `<-` explicit in rendered AST output.
- Semantic layer:
  - Add `Type::Chan`.
  - Keep send and receive as dedicated checked nodes rather than builtin calls.
  - Extend nil coercion and equality rules centrally.
- Bytecode:
  - Add dedicated instructions for nil channel, channel allocation, send, and receive.
  - Keep channel buffer metadata visible in rendered bytecode.
- Runtime:
  - Add a `ChannelValue` type with shared mutable state.
  - Keep queueing and close behavior centralized instead of scattering channel logic across builtin cases.
- Validation:
  - Cover send, receive, close, nil equality, and staged blocking failures through both focused tests and CLI integration tests.
