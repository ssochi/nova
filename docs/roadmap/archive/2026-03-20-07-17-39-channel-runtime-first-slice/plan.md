# Plan: First Channel Runtime Slice

## Basic Information

- Plan ID: `2026-03-20-07-17-39-channel-runtime-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Introduce the first staged `chan` runtime category so the VM can model buffered communication primitives explicitly instead of deferring channels entirely.
- Keep channel construction, send, receive, and close semantics explicit across AST, checked model, bytecode, and runtime.
- Extend builtin and nil-handling surfaces so `chan` behaves like a first-class composite runtime type in the current staged subset.

## Scope

- Research and document the official Go baseline for:
  - `chan T` type syntax
  - `make(chan T[, size])`
  - `len`, `cap`, and `close` behavior for channels
  - send statements and receive expressions
  - nil, closed, and equality behavior relevant to the staged subset
- Extend the frontend with:
  - `chan T` type references
  - send statements `ch <- value`
  - receive expressions `<-ch`
  - source rendering and CLI dumps that keep channel operations explicit
- Extend semantic analysis, checked modeling, and lowering for:
  - typed `chan` locals, parameters, returns, and `nil` coercion
  - `make(chan T[, size])`, builtin `close`, and channel-aware `len` / `cap`
  - explicit send statements and receive expressions
  - staged channel equality over matching channel types
- Extend the runtime with:
  - explicit nil-vs-allocated channel state
  - buffered queue storage and close state
  - dedicated bytecode for make/send/receive
  - runtime errors for currently unmodeled blocking behavior
- Add examples, tests, verification, experience evidence, and synchronized roadmap / design / tech / boot docs.

## Non-Goals

- Goroutines, scheduler work, `select`, `defer`, `go`, or any true concurrent execution model.
- Directional channel types such as `chan<- T` or `<-chan T`.
- Channel `range`, channel receive comma-ok bindings, or broader multi-result receive syntax.
- Panic/recover fidelity for `close` / send / receive failures; the current VM may surface runtime errors instead.
- Labels, `goto`, `fallthrough`, or unrelated import/package expansion.

## Phase Breakdown

1. Open the plan, lock the compatibility baseline in research, and record the staged channel boundary.
2. Extend tokens, AST, parser, and semantic type resolution for `chan`, send, and receive.
3. Add checked-model, builtin-contract, bytecode, and runtime support for the first channel slice.
4. Add examples, tests, validation reports, experience notes, and synchronize roadmap / design / tech / boot context.

## Acceptance Criteria

- `cargo test` passes with parser, semantic, runtime, and CLI coverage for the staged channel slice.
- `cargo run -- run` succeeds on a new example that uses buffered channels, send, receive, `close`, and `len` / `cap`.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` keep `chan`, send, and receive visible without reading Rust source.
- `cargo run -- check` rejects at least one invalid send type mismatch and one invalid `close` target.
- No modified source file exceeds the repository's 1000-line limit.

## Risks

- Go channel behavior is tightly coupled to blocking semantics; the staged single-threaded VM must avoid implying concurrency it does not actually model.
- Introducing `<-` affects both expression parsing and statement parsing, so precedence and ambiguity around send vs receive must stay explicit.
- Channel equality and nil handling need to remain consistent across semantic analysis, bytecode lowering, and runtime identity semantics.
