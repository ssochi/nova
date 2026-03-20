# Named Result Parameters Research

## Goal

Capture the official Go behavior baseline for named result parameters, grouped result declarations, and bare `return` so the next function-signature slice can extend compatibility without guessing at result-slot semantics.

## Sources Reviewed

- Official Go language specification sections `Function declarations`, `Signatures`, `Parameters`, and `Return statements`
- Local Go 1.21.5 compiler probes for grouped named results, mixed named/unnamed result lists, bare return success and failure cases, signature-name conflicts, and result-parameter shadowing
- Existing local notes `docs/research/2026-03-20-grouped-parameter-shorthand.md` and `docs/research/2026-03-20-multi-result-functions-and-cut-package-seams.md`

## Confirmed Findings

- Go allows grouped named result declarations such as `func split() (head, tail string, ok bool)`.
- Go also allows the existing unnamed result forms `func pair() (int, string)` and `func name() string`, but does not allow mixing named and unnamed results in the same result list.
- Named result parameters are part of the function signature scope together with ordinary parameters, so a result name cannot reuse an existing parameter name.
- Named result parameters are local variables initialized to the zero value of their declared type before the function body executes.
- Bare `return` is valid for functions with no results and for functions whose results are all named; it is rejected for functions that return unnamed values.
- When bare `return` is used, the current values of the result slots are returned in declaration order.
- If a named result parameter is shadowed in a nested scope, bare `return` from that shadowed point is rejected with a `result parameter <name> not in scope at return` diagnostic.
- Local Go 1.21.5 accepts blank identifiers in named result lists such as `func f() (_ int, ok bool)`; this implies the runtime result slot still exists even though no user-visible binding is introduced for `_`.

## Implementation Implications

- The frontend AST should represent result declarations explicitly, including grouped names, so `dump-ast` can render `func f() (left, right string)` instead of flattening back to repeated types.
- Result declarations should flatten into ordered result slots only after parsing, just as grouped input parameters already flatten into ordered parameter slots later in the pipeline.
- Semantic analysis should create zero-initialized result slots at function-entry time for named-result functions, while only non-blank names become user-visible local bindings.
- Bare `return` should stay explicit in semantic analysis by lowering to ordered reads from the tracked result slots rather than by inventing special VM behavior.
- Shadowing checks for bare `return` should happen during semantic analysis against the active scope stack, not during parsing or bytecode lowering.

## Deferred Questions

- Whether function-signature diagnostics should eventually distinguish parameter/result conflicts more precisely than the current signature-level wording
- Whether result-slot metadata should become more explicit in `dump-bytecode`, or whether visible result-local names are sufficient for now
