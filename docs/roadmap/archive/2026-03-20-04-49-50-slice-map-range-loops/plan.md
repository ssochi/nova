# Plan: Slice and Map Range Loops

## Basic Information

- Plan ID: `2026-03-20-04-49-50-slice-map-range-loops`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `in_progress`
- Owner: primary agent

## Goals

- Add the first staged `range` loop support to the language and VM.
- Keep `dump-ast` and `dump-bytecode` readable for the new loop form.
- Refactor affected implementation files so the new work does not push them past the single-file size limit.

## Scope

- Research and document the Go behavior baseline for `range` over `slice` and `map`.
- Support `for ... range ... {}` over `slice` and `map` in the parser, semantic layer, bytecode lowering, and VM.
- Support the common staged forms `for range expr`, `for key := range expr`, and `for key, value := range expr`.
- Keep nil `slice` and nil `map` iteration behavior explicit and testable.
- Split or extract helpers from `src/semantic/analyzer.rs` and `src/runtime/vm.rs` as needed to stay within repository code-size rules.

## Non-Goals

- String `range` support or rune semantics.
- `break`, `continue`, `goto`, or labeled control flow.
- Full short variable declaration support outside `range` headers.
- Real Go map iteration nondeterminism.

## Phase Breakdown

1. Research and design the staged `range` semantics and bytecode/runtime strategy.
2. Add frontend and checked-model support for staged `range` headers.
3. Lower `range` loops into explicit bytecode and execute them in the VM.
4. Add CLI examples, automated tests, verification records, and roadmap/documentation updates.

## Acceptance Criteria

- `cargo test` passes with parser, semantic, VM, and CLI coverage for `range` over `slice` and `map`.
- `cargo run -- run` succeeds on at least one new `range`-focused example program.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` show the staged `range` path without reading implementation code.
- Technical docs and roadmap docs describe the supported `range` surface and deferred gaps.
- No modified source file exceeds the repository's 1000-line limit.

## Risks

- `range` headers can sprawl into broader control-flow work if the staged syntax boundary is not kept tight.
- Map iteration needs an explicit deterministic strategy so the current runtime model stays debuggable.
- Hidden loop temporaries can make bytecode and semantic state harder to reason about unless naming and lowering stay explicit.
