# Panic-Aware Unwind and Recover Baseline

## Goal

Establish the official and locally verified behavior baseline for the staged panic/recover surface built on top of the existing explicit `defer` implementation.

## Sources Reviewed

- Go specification, `Defer statements` and `Handling panics` sections on `go.dev/ref/spec`
- `go doc builtin panic`
- `go doc builtin recover`
- Local Go 1.21.5 probes executed with `go run` for:
  - deferred calls during panic propagation
  - deferred panic overriding a normal return
  - `panic(nil)` behavior
  - direct deferred recovery through a named function
  - helper-call and deferred-builtin `recover()` non-recovery
  - `recover()` return values outside panic and after a successful recovery
  - post-recovery return values for unnamed and named results

## Confirmed Findings

- Deferred calls run when a function returns because of a panic, not only on ordinary `return`.
- Panic propagation is frame-by-frame: the current function's deferred calls run first, then the caller's deferred calls, and so on.
- A deferred call that itself panics replaces an in-progress normal return path; the function no longer returns its pending result to the caller.
- `panic(nil)` is accepted syntactically, but starting in Go 1.21 it produces a separate run-time panic rather than reporting a nil recovered value.
- The spec describes both explicit builtin `panic` and ordinary run-time panics as entering the same panicking sequence.
- `recover` is meaningful only when called directly inside a deferred function during an active panicking sequence.
- A directly deferred named function can recover a panic; `defer namedRecover()` works when `namedRecover` itself calls `recover()` directly.
- A helper invoked by the deferred function cannot recover the panic; `recover()` in that helper returns `nil`.
- `defer recover()` does not stop the panic in real Go; the deferred builtin call still yields `nil` and the panic continues.
- Once one direct `recover()` succeeds, later `recover()` calls in the same deferred function return `nil`.
- Recovering a panic resumes normal return from the panicking function. Unnamed results fall back to their zero values, while named results can still be changed by a deferred closure before the function returns.
- Recovering `panic(nil)` returns a non-nil payload in Go 1.21 (`*runtime.PanicNilError`) whose string form is `panic called with nil argument`.

## Implementation Implications

- The VM should represent panic-state unwinding explicitly instead of treating builtin `panic` as just another immediate runtime error.
- Existing runtime traps that already correspond to Go run-time panics should be able to enter the same unwind path so deferred calls still run.
- The existing per-frame deferred-call stack and pending-return storage are the right extension points; the runtime should not bolt on a second unwind mechanism.
- Now that `any` / `interface{}` exist, recovered payloads can reenter the staged language as boxed `any` values rather than raw runtime messages.
- Recover eligibility should be modeled on the deferred user-function frame itself; helper frames and deferred builtins must remain ineligible even while a panic is active.
- The current runtime still lacks Go's concrete runtime panic object types, so recovered runtime panic payloads and `panic(nil)` need a documented staged representation instead of pretending to return `runtime.Error` or `*runtime.PanicNilError`.

## Deferred Questions

- Which remaining runtime trap sites should be promoted from plain runtime errors into staged run-time panics in later rounds?
- How closely should the staged runtime eventually approximate Go's concrete recovered payload types for runtime panics and `panic(nil)`?
- How closely should the CLI eventually approximate real Go panic formatting and stack traces?
