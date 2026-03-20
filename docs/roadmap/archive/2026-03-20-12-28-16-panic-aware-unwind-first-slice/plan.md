# Plan: Panic-Aware Unwind First Slice

## Basic Information

- Plan ID: `2026-03-20-12-28-16-panic-aware-unwind-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `in_progress`
- Owner: primary agent

## Goals

- Add a first staged explicit `panic(...)` builtin that fits the current typed language surface.
- Reuse the existing frame-level defer stack so panics unwind through deferred calls instead of bypassing them.
- Route staged runtime traps that already correspond to Go panic behavior through the same unwind path so CLI execution behaves more like real Go.

## Scope

- Research the current Go behavior for builtin `panic`, `panic(nil)`, defer execution during panic, and deferred panics overriding normal returns.
- Extend builtin metadata, semantic validation, bytecode lowering, and runtime dispatch for explicit `panic(...)`.
- Add VM panic-state tracking so deferred calls run during panic unwinding and the CLI reports a panic-oriented runtime failure.
- Convert a focused subset of current runtime trap sites to use the new panic path when doing so does not require new type-system features.
- Add focused examples, unit tests, CLI execution tests, CLI diagnostics, and documentation for the staged panic surface.

## Non-Goals

- `recover` in this round; the current type system has no `interface{}` / `any` carrier for the recovered payload.
- Interface types, method values, closures, or broader expression-statement cleanup.
- Precise Go stack-trace rendering or full runtime panic formatting parity.
- Scheduler, goroutines, `select`, channel `range`, or broader concurrency semantics.

## Phase Breakdown

1. Research and design
   - Confirm the staged panic semantics and the exact deferred scope to preserve.
   - Record why `recover` remains deferred.
2. Semantic and lowering work
   - Add builtin `panic` contracts and keep the statement/defer surface explicit.
   - Preserve readable bytecode output for panic execution.
3. VM unwind work
   - Add panic-state tracking that drains deferred calls in LIFO order across frames.
   - Make ordinary return unwinding and panic unwinding coexist without hidden synthetic control flow.
4. Validation and documentation
   - Add focused tests, CLI traces, reports, and line-count checks.
   - Sync roadmap, tech docs, and `BOOT.md` with the new behavior and follow-up hooks.

## Acceptance Criteria

- `panic(...)` is available as a builtin call and works through `run`, `dump-ast`, `dump-bytecode`, and `check`.
- A panic raised in a function with defers executes that function's deferred calls before propagating to callers.
- A deferred panic can override a normal return path, and outer defers still execute.
- At least one existing runtime trap path now uses panic-aware unwinding instead of aborting before defers run.
- Documentation and reports clearly state that `recover` is still deferred and why.

## Risks

- VM control-flow complexity can sprawl if panic-state tracking is bolted onto the existing return path instead of sharing one explicit unwind model.
- The lack of `interface{}` means builtin `panic` must stay carefully staged around currently typed values plus a documented `panic(nil)` treatment.
- Near-limit files such as `src/semantic/analyzer.rs`, `src/bytecode/compiler.rs`, and `src/runtime/vm.rs` may need helper extraction in the same round.
