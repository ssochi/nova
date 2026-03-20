# Variadic Functions and Explicit Ellipsis Research

## Goal

Capture the official Go behavior baseline for the next staged call-surface slice: variadic function declarations, final-argument `...` expansion, and the narrow builtin `append(slice, values...)` / `append([]byte, string...)` behavior that depends on explicit ellipsis syntax.

## Sources Reviewed

- Official Go language specification sections `Function types`, `Passing arguments to ... parameters`, and `Appending to and copying slices`
- Local Go 1.21.5 probes for zero-argument variadic calls, prefixed arguments plus `slice...`, `append([]byte, string...)`, non-variadic `...` rejection, and slice-sharing behavior across `values...`
- Existing local research note `docs/research/2026-03-20-slice-expressions-and-assignment.md`

## Confirmed Findings

- A variadic parameter is declared by placing `...` before the final parameter type in a function signature, such as `func collect(prefix int, values ...int)`.
- Only the final parameter may be variadic.
- Inside the called function, the variadic parameter has type `[]T`.
- Calling a variadic function with no arguments for the variadic portion yields a `nil` slice for that parameter.
- Ordinary calls to a variadic function may supply zero or more individual arguments in the variadic position.
- A call may also pass a final slice argument with `...`, such as `collect(1, values...)`.
- When a call uses the explicit `...` form, only the fixed non-variadic prefix arguments may appear before that final spread value; extra individual variadic arguments before `slice...` are rejected.
- Using `...` in a call to a non-variadic function is rejected.
- The `...` form applies only to the final source argument in a call.
- Passing `values...` forwards the existing slice view rather than flattening into unrelated scalars first; element mutations through the variadic parameter remain visible to the caller through shared slice storage.
- Untyped `nil...` is accepted when the variadic or builtin context provides the required slice type, such as `collect(nil...)` or `append(values, nil...)`.
- `append(dst, src...)` appends all elements of `src` when `src` is a slice whose element type matches `dst`.
- `append([]byte, string...)` is a special case in real Go that appends the raw bytes of the string operand.
- Existing ordinary `append(dst, elem1, elem2, ...)` behavior remains distinct from the explicit `src...` form.
- The current staged multi-result forwarding rule is separate: `...` is not a general-purpose second expansion mechanism for multi-result calls.

## Implementation Implications

- Variadic function declarations should stay explicit in the AST, semantic registry, checked model, and bytecode metadata instead of pretending the last parameter is an ordinary fixed-arity slot.
- The checked call model should add an explicit final spread argument form rather than erasing `...` into an ordinary expression list too early.
- Explicit `...` should require the source-side fixed prefix to match only the non-variadic parameters; once a spread value is present, later discrete variadic arguments should not be accepted in the same call.
- Runtime function metadata needs enough information to distinguish fixed parameters from an optional variadic tail so user-defined calls can accept `arity >= required_prefix`.
- The VM should materialize the variadic tail as a runtime slice value when entering a user-defined variadic function, reusing the existing slice runtime so zero-argument calls become nil slices naturally.
- Builtin validation should keep ordinary variadic-element appends and explicit spread appends distinct so `append(values, more...)` and `append(bytes, text...)` can validate without weakening ordinary element typing.
- The current `fmt` and builtin output helpers can remain unchanged for now because the project has no `[]any` / interface surface to make `args...` practically useful there.

## Deferred Questions

- Whether later grouped parameter-name shorthand such as `func f(a, b int)` should be implemented in the same function-signature machinery or remain a separate syntax slice
- Whether explicit `...` should later combine with broader package-backed variadic APIs once an interface or `any` surface exists
- Whether the current staged multi-result call forwarding and explicit ellipsis call spreading ever need a shared abstraction, or should remain separate checked forms
