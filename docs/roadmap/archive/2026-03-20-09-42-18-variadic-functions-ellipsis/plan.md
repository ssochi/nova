# Plan: Variadic Functions and Explicit Ellipsis

## Basic Information

- Plan ID: `2026-03-20-09-21-38-variadic-functions-ellipsis`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Add staged user-defined variadic function declarations so more realistic Go helper APIs can be expressed without abandoning the VM-first model.
- Add explicit final-argument `...` call spreading as a first-class syntax and checked-model path.
- Spend that new call path on builtin `append(slice, other...)`, including the narrow Go-compatible `append([]byte, string...)` special case.

## Scope

- AST, parser, semantic registry, checked model, bytecode metadata, and VM entry handling for variadic final parameters.
- Explicit final-argument `...` support in ordinary call expressions.
- Semantic validation and lowering for variadic user-defined calls with fixed prefixes plus variadic tails.
- Builtin `append` support for spread slice arguments and `[]byte` plus `string...`.
- Examples, automated tests, CLI validation, research/doc sync, and roadmap/report updates for the new surface.

## Non-Goals

- Interface or `any` support for `fmt`-style `args...` slice forwarding.
- Grouped parameter-name shorthand such as `func f(a, b int)`.
- Variadic package declarations, named results, naked returns, or generalized tuple/spread expression forms.
- Non-final `...`, multiple spread arguments in one call, or mixing `...` with multi-result call expansion.

## Phase Breakdown

1. Record the compatibility baseline for variadic declarations and `...` calls, then open the active `M3` plan.
2. Extend the AST and parser so function signatures and call sites keep variadic metadata explicit.
3. Extend semantic analysis, checked-call modeling, and function metadata for variadic declarations, call checking, and `append` spread validation.
4. Extend bytecode lowering and VM call entry so variadic tails become runtime slices without hiding the feature behind ad hoc rewrites.
5. Add examples, tests, CLI validation, documentation/report updates, and archive the plan if all acceptance criteria pass.

## Acceptance Criteria

- A user-defined function such as `func collect(prefix int, values ...int) int` can be checked, lowered, and executed with both ordinary arguments and a final `slice...` argument.
- Zero-variadic-argument calls produce Go-like nil-slice behavior inside the callee.
- `append(values, more...)` and `append(bytes, text...)` succeed with readable `dump-ast` / `dump-bytecode` output.
- Invalid `...` uses fail with targeted diagnostics, including non-variadic calls and non-slice spread arguments.

## Risks

- Variadic declarations can blur the current fixed-arity function metadata unless the last-parameter distinction stays explicit end to end.
- Ellipsis support can collide conceptually with the staged multi-result expansion path if the checked call model does not keep them separate.
- `append([]byte, string...)` is a narrow but special rule; implementing it carelessly could weaken ordinary append type validation.
