# Grouped Parameter Shorthand Research

## Goal

Capture the official Go behavior baseline for grouped parameter-name shorthand such as `func pair(a, b int)` so the next function-signature slice can improve compatibility without quietly weakening the current variadic rules.

## Sources Reviewed

- Official Go language specification sections `Function types`, `Signatures`, and `Parameters`
- Local Go 1.21.5 compiler probes for valid grouped parameters, missing-type grouped declarations, and grouped-name variadic declarations
- Existing local note `docs/research/2026-03-20-variadic-functions-and-ellipsis.md`

## Confirmed Findings

- Go permits grouped input parameter names that share one type, such as `func pair(a, b int)`.
- Grouped parameter declarations preserve left-to-right parameter order exactly; `func mix(a, b int, label string)` still has three incoming parameters.
- Grouped shorthand also works across multiple groups in one signature, such as `func convert(a, b int, prefix, suffix string)`.
- The current missing-type form `func broken(a int, b, c)` is rejected by the Go compiler with a mixed named/unnamed parameter diagnostic rather than being inferred from the previous type.
- Variadic parameters remain restricted to one final parameter slot. Local Go rejects `func broken(values, more ...int)` with `can only use ... with final parameter in list`.
- Grouped parameter-name shorthand therefore composes with variadics only when the variadic declaration remains a single final parameter, such as `func collect(prefix, suffix string, values ...int)`.
- Grouped shorthand is an input-signature convenience only in this scope; named result parameters and naked returns are separate features with their own semantic/runtime implications.

## Implementation Implications

- The frontend should keep grouped parameter declarations explicit enough to render `func pair(a, b int)` back through `dump-ast` instead of flattening to `a int, b int` too early.
- The semantic registry and per-function local-binding setup can still flatten grouped declarations into the existing ordered parameter slot model after parsing.
- Duplicate parameter-name checks must operate on the flattened ordered names so grouped declarations and ordinary declarations share one error path.
- Variadic validation should continue to enforce that only a single final parameter declaration may be variadic; grouped-name variadic declarations must fail during parsing.
- Result-type handling should stay unchanged in this slice because the current AST and semantic model intentionally keep unnamed results only.

## Deferred Questions

- Whether grouped result declarations should arrive with named results later, or remain separate until result-slot initialization and naked `return` are deliberately designed
- Whether a future function-type surface should reuse the same grouped-declaration AST node or introduce a distinct function-type representation
