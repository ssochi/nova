# Panic-Aware Unwind First Slice

## Goal

Establish the official and locally verified behavior baseline for a first staged panic slice built on top of the existing explicit `defer` implementation.

## Sources Reviewed

- Go specification, `Defer statements` and `Handling panics` sections on `go.dev/ref/spec`
- `go doc builtin panic`
- `go doc builtin recover`
- Local Go 1.21.5 probes executed with `go run` for:
  - deferred calls during panic propagation
  - deferred panic overriding a normal return
  - `panic(nil)` behavior

## Confirmed Findings

- Deferred calls run when a function returns because of a panic, not only on ordinary `return`.
- Panic propagation is frame-by-frame: the current function's deferred calls run first, then the caller's deferred calls, and so on.
- A deferred call that itself panics replaces an in-progress normal return path; the function no longer returns its pending result to the caller.
- `panic(nil)` is accepted syntactically, but starting in Go 1.21 it produces a separate run-time panic rather than reporting a nil recovered value.
- The spec describes both explicit builtin `panic` and ordinary run-time panics as entering the same panicking sequence.
- `recover` is meaningful only when called directly inside a deferred function during an active panicking sequence.

## Implementation Implications

- The VM should represent panic-state unwinding explicitly instead of treating builtin `panic` as just another immediate runtime error.
- Existing runtime traps that already correspond to Go run-time panics should be able to enter the same unwind path so deferred calls still run.
- The existing per-frame deferred-call stack and pending-return storage are the right extension points; the runtime should not bolt on a second unwind mechanism.
- The first slice can ship accurate `panic` behavior without `recover`, because `recover` needs a result carrier equivalent to `interface{}` / `any`, which the current type system does not model.
- `panic(nil)` should stay explicit in the staged design; either it needs dedicated lowering/runtime handling or the divergence must be documented clearly.

## Deferred Questions

- How should recovered panic payloads be typed once the project introduces `interface{}` or `any`?
- Which remaining runtime trap sites should be promoted from plain runtime errors into staged run-time panics in later rounds?
- How closely should the CLI eventually approximate real Go panic formatting and stack traces?
