# Panic-Aware Unwind First Slice

## Goal

Add a first staged panic model that makes `defer` meaningful during abnormal exit while keeping the current VM-first architecture explicit and readable.

## Constraints

- Rust standard library only
- Reuse the current direct-call builtin surface instead of introducing function values or interfaces
- Keep panic-state handling explicit in runtime control flow and user-facing inspection surfaces
- Avoid pushing near-limit files over the repository size cap; split helpers when needed

## Current Scope

- A builtin `panic(...)` call integrated into the ordinary builtin registry and checked-call pipeline
- VM panic-state tracking that propagates across frames while draining deferred calls in LIFO order
- Shared unwind handling for explicit builtin panic plus a focused subset of current runtime traps
- Panic-oriented CLI runtime diagnostics that preserve the panic payload message without claiming full Go stack-trace fidelity

## Deferred Scope

- `recover` and recovered payload typing
- Interface types, `any`, closures, function values, and method values
- Full parity with real Go panic formatting or rich stack traces
- Panic/recover behavior for goroutines, scheduler-aware blocking, or concurrent channel paths

## Interfaces and Extension Hooks

- `src/semantic/builtins.rs` should keep builtin `panic` validation centralized, including any staged `panic(nil)` rule, instead of scattering exceptions through statement analysis
- `src/bytecode/instruction.rs` should keep panic execution visible in rendered bytecode instead of hiding it behind a generic runtime failure
- `src/runtime/vm.rs` should unify ordinary return unwinding and panic unwinding around frame metadata so future `recover` can hook into one place
- Runtime trap promotion should flow through a reusable helper rather than ad hoc `Err(...)` returns when the desired behavior is actually panicking
- Focused VM and CLI panic tests should live in their own files under `src/runtime/vm/tests/` and `tests/` so the broad suites stay small
