# Multi-Result Functions and Cut Package Seams

## Goal

Capture the official Go behavior baseline for the first staged `nova-go` multi-result slice, with emphasis on unnamed result lists, assignment and return usage, and the `strings.Cut` / `bytes.Cut` package contracts that depend on multi-result call plumbing.

## Sources Reviewed

- Official Go language specification sections `Function declarations`, `Return statements`, and `Assignments`
- Official `strings` package documentation for `Cut`
- Official `bytes` package documentation for `Cut`
- Local Go 1.21.5 probes for multi-result short declarations, reassignment, direct return forwarding, and `Cut` success/failure edge cases

## Confirmed Findings

- Go functions may return zero, one, or multiple values; unnamed result lists are written either as a single type or a parenthesized comma-separated type list.
- A call that produces multiple results is not a general tuple value. It is valid in assignment-like and return contexts that expect matching arity, but it is rejected in ordinary single-value expression contexts.
- Multi-result assignment and short declaration allow a single multi-result call on the right side when the left side count matches the produced result count.
- The multi-result return path allows direct forwarding such as `return pair()` when the callee result list matches the enclosing function result list.
- `strings.Cut(s, sep)` returns `before`, `after`, and `true` when `sep` is found; otherwise it returns `s`, `""`, and `false`.
- `strings.Cut("nova", "")` succeeds and returns `""`, `"nova"`, and `true`.
- `bytes.Cut(s, sep)` returns `before`, `after`, and `true` when `sep` is found; otherwise it returns the original slice, `nil`, and `false`.
- `bytes.Cut([]byte("nova"), []byte(""))` succeeds and returns an empty prefix slice, the original content as `after`, and `true`.

## Implementation Implications

- The next `nova-go` slice can keep ordinary expressions single-valued while adding an explicit multi-result call path for assignments, short declarations, and returns.
- Function signatures, package contracts, bytecode metadata, and VM return handling should all expose result arity explicitly instead of inferring it from a boolean `returns_value` flag.
- The staged multi-binding surface can stay narrower than full Go by focusing on identifier/blank bindings instead of general tuple-like assignment targets that need broader evaluation-order machinery.
- `strings.Cut` and `bytes.Cut` are strong proof points because they exercise multi-result package contracts without forcing interfaces, errors, or filesystem-backed imports.

## Deferred Questions

- When comma-ok receive is added later, whether it should reuse the same multi-result call plumbing or remain a dedicated checked/runtime form like the current staged `map` lookup.
- Whether named result parameters and naked returns should wait for a later scope that can model result slots and zero-value initialization deliberately.
- Whether later multi-result expansion should allow the final-argument forwarding rules that Go permits in some call expressions.
