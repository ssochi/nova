# Builtin Clear for Slices and Maps Research

## Goal

Record the official behavior baseline for the next `M3` slice around builtin `clear` on `slice` and `map` values.

## Sources Reviewed

- Official `builtin` package docs on `pkg.go.dev`
- Official Go language spec on `go.dev/ref/spec`
- Local Go probe under `/tmp/nova-go-clear-probe.go`

## Confirmed Findings

- `clear(m)` deletes all entries from a map and is a no-op for a nil map.
- `clear(s)` sets slice elements up to `len(s)` to their zero value while preserving the slice length and capacity.
- `clear` operates on the visible slice window, so clearing a subslice mutates the shared backing storage only for that subslice range.
- Clearing a nil slice is a no-op and keeps the slice nil with `len == 0` and `cap == 0`.
- Clearing a map mutates shared map state, so aliases observe the emptied map immediately.
- The current Go spec also describes `clear` over type parameters whose set contains only map or slice types, but that generic surface is outside the current `nova-go` subset.

## Implementation Implications

- `clear` should be added as an ordinary builtin rather than lowered into synthetic loops or package calls.
- Semantic validation should accept exactly one argument of type `slice` or `map` and reject scalars, `string`, and `chan`.
- VM execution needs explicit runtime branches:
  - `slice`: overwrite each visible element with the element zero value without changing `len`, `cap`, or nil state
  - `map`: remove all entries while preserving the nil-vs-allocated distinction
- The bytecode surface can stay as explicit `call-builtin clear 1`, keeping `dump-bytecode` readable.
- Because `clear` mutates shared slice/map state, the runtime helpers must work through existing shared storage rather than returning rebuilt values.

## Deferred Questions

- Do not extend `clear` to type-parameter inputs until generics are modeled deliberately.
- Do not widen `clear` beyond `slice` and `map`; channel or string support would be incorrect.
- Keep any larger runtime-file refactor scoped to staying under the repository file-size ceiling rather than redesigning the whole runtime layer.
