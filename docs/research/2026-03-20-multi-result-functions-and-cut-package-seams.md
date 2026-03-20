# Multi-Result Functions and Cut Package Seams

## Goal

Capture the official Go behavior baseline for the staged `nova-go` multi-result surface, with emphasis on unnamed result lists, assignment / return usage, call-argument forwarding, and the `strings` / `bytes` `Cut*` package contracts that depend on multi-result call plumbing.

## Sources Reviewed

- Official Go language specification sections `Function declarations`, `Return statements`, and `Assignments`
- Official `strings` package documentation for `Cut`, `CutPrefix`, and `CutSuffix`
- Official `bytes` package documentation for `Cut`, `CutPrefix`, and `CutSuffix`
- Local Go 1.21.5 probes for multi-result short declarations, reassignment, direct return forwarding, call-argument forwarding, and `Cut*` success/failure edge cases

## Confirmed Findings

- Go functions may return zero, one, or multiple values; unnamed result lists are written either as a single type or a parenthesized comma-separated type list.
- A call that produces multiple results is not a general tuple value. It is valid in assignment-like and return contexts that expect matching arity, but it is rejected in ordinary single-value expression contexts.
- Multi-result assignment and short declaration allow a single multi-result call on the right side when the left side count matches the produced result count.
- The multi-result return path allows direct forwarding such as `return pair()` when the callee result list matches the enclosing function result list.
- A multi-result call may supply arguments to another call only when it is the entire argument list by itself, such as `consume(pair())`; prefixed forms such as `consume(1, pair())` still fail because `pair()` remains in a single-value expression context there.
- `strings.Cut(s, sep)` returns `before`, `after`, and `true` when `sep` is found; otherwise it returns `s`, `""`, and `false`.
- `strings.Cut("nova", "")` succeeds and returns `""`, `"nova"`, and `true`.
- `strings.CutPrefix(s, prefix)` returns `s` without `prefix` and `true` when `s` starts with `prefix`; otherwise it returns `s` and `false`. An empty prefix succeeds and returns `s`, `true`.
- `strings.CutSuffix(s, suffix)` returns `s` without `suffix` and `true` when `s` ends with `suffix`; otherwise it returns `s` and `false`. An empty suffix succeeds and returns `s`, `true`.
- `bytes.Cut(s, sep)` returns `before`, `after`, and `true` when `sep` is found; otherwise it returns the original slice, `nil`, and `false`.
- `bytes.Cut([]byte("nova"), []byte(""))` succeeds and returns an empty prefix slice, the original content as `after`, and `true`.
- `bytes.CutPrefix(s, prefix)` returns a slice of the original `s` without the leading `prefix` and `true` when found; otherwise it returns the original slice and `false`.
- `bytes.CutSuffix(s, suffix)` returns a slice of the original `s` without the trailing `suffix` and `true` when found; otherwise it returns the original slice and `false`.
- `bytes.CutPrefix([]byte("nova"), []byte("go"))` and `bytes.CutSuffix([]byte("nova"), []byte("go"))` both return the original non-nil slice and `false`; unlike `bytes.Cut`, they do not synthesize a nil slice on misses.

## Implementation Implications

- The next `nova-go` slice can keep ordinary expressions single-valued while adding an explicit multi-result call path for assignments, short declarations, returns, and single-call-argument forwarding.
- Function signatures, package contracts, bytecode metadata, and VM return handling should all expose result arity explicitly instead of inferring it from a boolean `returns_value` flag.
- The staged multi-binding surface can stay narrower than full Go by focusing on identifier/blank bindings instead of general tuple-like assignment targets that need broader evaluation-order machinery.
- Call forwarding should stay explicit in the checked model rather than flattening expanded calls into ordinary expression lists too early.
- `strings.Cut*` and `bytes.Cut*` are strong proof points because they exercise several multi-result package contracts without forcing interfaces, errors, or filesystem-backed imports.

## Deferred Questions

- When comma-ok receive is added later, whether it should reuse the same multi-result call plumbing or remain a dedicated checked/runtime form like the current staged `map` lookup.
- Whether named result parameters and naked returns should wait for a later scope that can model result slots and zero-value initialization deliberately.
- Whether later multi-result expansion should broaden beyond Go's current single-call-argument forwarding rule into more general call or statement contexts.
