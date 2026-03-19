# Strings Package Contract Research

## Goal

Capture the official behavior baseline for the first `strings` package seam so the implementation can stay narrow, explicit, and reusable for later package expansion.

## Sources Reviewed

- Official Go package documentation for `strings`
- Official Go language specification sections that define strings and slices as the value categories used by the selected functions

## Confirmed Findings

- `strings.Contains(s, substr)` reports whether `substr` occurs within `s` and returns `bool`.
- `strings.HasPrefix(s, prefix)` reports whether `s` begins with `prefix` and returns `bool`.
- `strings.Join(elems, sep)` concatenates the elements of a string slice using `sep` between adjacent elements and returns `string`.
- `strings.Repeat(s, count)` returns a new string containing `count` copies of `s`.
- Real Go treats negative repeat counts and repeated-size overflow as failures for `strings.Repeat`; the current VM does not model panic, so this project should surface those cases as runtime errors instead of silently producing incorrect results.

## Implementation Implications

- The selected function set fits the current runtime model because it only requires `string`, `bool`, `int`, and `[]string`.
- `strings.Join` is the most useful contract test for the current package layer because it requires a typed slice argument instead of the existing variadic any-value shape used by `fmt`.
- `strings.Contains` and `strings.HasPrefix` exercise typed fixed-arity boolean-returning package calls.
- `strings.Repeat` exercises typed mixed arguments and forces the runtime layer to define how Go-like failure cases map into the VM error model.

## Deferred Questions

- Whether later `strings` expansion should prioritize transformation helpers such as `TrimSpace` / `ToUpper` or search / split helpers that interact with slices more heavily
- Whether future runtime plans should model Go panic behavior directly instead of translating selected package failures into runtime errors
