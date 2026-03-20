# Defer First Slice Research

## Goal

Capture the official Go behavior baseline for a first staged `defer` slice so `nova-go` can add function-exit deferral without guessing at evaluation timing, call restrictions, or unwind order.

## Sources Reviewed

- Official Go language specification sections `Defer statements` and `Expression statements` at `https://go.dev/ref/spec`
- Local Go 1.21.5 compiler and runtime probes for eager defer-argument evaluation, LIFO ordering, deferred return sequencing, parenthesized-form rejection, builtin-call restrictions, and multi-result user-defined calls
- Existing local note `docs/research/2026-03-20-named-result-parameters.md`

## Confirmed Findings

- A `defer` statement evaluates the function value and arguments immediately, then saves the call for later execution when the surrounding function returns.
- Deferred calls in one function execute in last-in-first-out order.
- Deferred calls run after a `return` statement has evaluated its result expressions but before the function actually returns to its caller.
- Go rejects parenthesized deferred expressions such as `defer (println("x"))` with a targeted diagnostic; the defer operand must be a call expression, not an arbitrary grouped expression.
- Builtin calls inside `defer` follow the same statement-context restrictions as ordinary expression statements.
- Local Go 1.21.5 accepts `defer println("tail")` and `defer copy(dst, src)`, but rejects `defer len("x")`, `defer append(values, 2)`, and `defer make([]int, 1)` because those builtins produce values that are not permitted in statement context.
- Deferred user-defined calls may return one or more values and those results are simply discarded; local Go 1.21.5 accepts `defer pair()` where `pair` returns two values.
- Local Go 1.21.5 shows eager argument capture directly: `defer emit(value)` observes the value at defer time rather than a later reassigned value.

## Implementation Implications

- `defer` should be modeled as its own statement node while reusing the existing explicit checked-call structure for the deferred call payload.
- Semantic validation should reject non-call defer operands and should apply a builtin-specific statement-context filter instead of assuming all result-bearing builtins are invalid.
- Deferred calls should stay explicit in bytecode and the VM rather than being lowered into synthetic tail blocks, because return sites, multiple returns, and future panic-aware unwinding need one shared frame-level mechanism.
- VM frames should retain a LIFO deferred-call stack plus pending return values so deferred calls run after return-value evaluation but before frame removal.
- This first slice can stay compatible with the current language subset by supporting only the already-modeled direct call forms: builtins, imported package members, and user-defined function names.

## Deferred Questions

- How `defer` should interact with future closures, method values, and pointer- or interface-driven mutation of named result slots
- How panic-triggered unwinding should reuse the same deferred-call stack once `panic` / `recover` work is planned deliberately
