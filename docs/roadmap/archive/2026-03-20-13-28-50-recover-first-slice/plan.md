# Plan: Recover First Slice

## Basic Information

- Plan ID: `2026-03-20-13-28-50-recover-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add the first staged `recover()` builtin slice on top of the existing panic-aware unwind model.
- Make recovered panic payloads flow back through explicit `any` values instead of ad hoc runtime messages.
- Keep the VM readable by extracting runtime call/unwind helpers while landing the feature.

## Scope

- Extend the existing panic/recover research baseline with the direct-deferred-call rules, nil-panic recovery behavior, and return-after-recover behavior needed for this slice.
- Add semantic builtin support for zero-argument `recover()` with explicit `any` return typing and the current statement-context rule.
- Update runtime panic payload recovery so deferred user-defined functions can stop a panic and receive the staged recovered value through `any`.
- Keep `dump-bytecode`, `run`, and `check` useful for the new path, and split VM helpers if needed to stay under the repository file-size ceiling.
- Add focused VM and CLI tests plus a real CLI playtest for successful recovery, nil recovery outside panic, and documented staged limits.

## Non-Goals

- Closures, function literals, or named-result mutation through deferred closures in this round.
- Goroutines, scheduler-aware panic behavior, labeled control flow, or stack-trace fidelity.
- Exact Go runtime payload types for recovered runtime panics such as `*runtime.PanicNilError` or runtime error interfaces.
- General method-bearing interfaces, type assertions, or type switches.

## Phase Breakdown

1. Research and plan refresh
   - Extend the existing panic/recover note with the direct-call and nil-panic findings needed now.
   - Record the staged divergence for recovered runtime panic payload typing.
2. Semantic and bytecode surface
   - Register builtin `recover`, validate zero arguments, and surface `any` as the result type.
   - Keep statement-context handling explicit for `recover()` and preserve readable bytecode for the path.
3. Runtime and unwind work
   - Track recover eligibility on deferred user-function frames.
   - Convert the current pending panic payload into a recovered `any` value when `recover()` succeeds.
   - Refactor VM call/unwind helpers as needed to keep touched files below 1000 lines.
4. Validation and synchronization
   - Add focused unit and CLI coverage, serial CLI evidence, documentation sync, and line-count checks.

## Acceptance Criteria

- `recover()` returns `any` and type-checks in expression positions, ordinary statement context, and deferred user-function bodies.
- A panic recovered by a directly deferred user-defined function stops the panic and lets the surrounding function return normally.
- `recover()` returns `<nil>` outside an active eligible panic-recovery context, including deferred builtin `recover()` and helper calls invoked by a deferred function.
- The staged runtime clearly documents that recovered runtime panic payloads are boxed into `any` using the currently modeled value types rather than full Go runtime error objects.
- Touched files, especially runtime VM files, remain at or below the repository line-count limit.

## Risks

- `src/runtime/vm.rs` is already near the repository ceiling and will require helper extraction in the same round.
- Recover eligibility is easy to over-broaden; the implementation must stay tied to directly deferred user-function frames rather than generic panic state.
- Without closures, some familiar Go `recover` patterns remain impossible; the docs must make that scope boundary obvious instead of implying full parity.
