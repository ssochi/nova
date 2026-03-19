# Map Runtime Groundwork Research

## Goal

Capture the official Go behavior baseline needed for the first staged map runtime slice in `nova-go`, with emphasis on `map[K]V` type syntax, `make`, indexing, assignment, and `len`.

## Sources Reviewed

- Official Go language specification section `Map types`
- Official Go language specification section `Properties of types and values`
- Official Go language specification section `Index expressions`
- Official Go language specification section `Length and capacity`
- Official Go language specification section `Making slices, maps and channels`

## Confirmed Findings

- `map[K]V` is a distinct reference-like type. The zero value of a map is `nil`.
- Map keys must support `==` and `!=`; slices, maps, and functions are not valid key types.
- `make(map[K]V)` and `make(map[K]V, hint)` produce a non-nil ready-to-write map value. The size hint is optional and does not change the map length.
- `len(map)` returns the current number of defined entries. `len(nilMap)` is `0`.
- `value := m[key]` returns the element value when the key exists; otherwise it returns the zero value of the element type.
- Assigning through `m[key] = value` inserts or replaces an entry, but assigning into a nil map triggers a runtime failure in real Go.
- The single-result form `m[key]` is enough for the current slice; the comma-ok lookup form can be deferred without blocking basic map usability.

## Implementation Implications

- The first `nova-go` map slice can stay focused on scalar comparable key types already present in the runtime model: `int`, `byte`, `bool`, and `string`.
- Map support should model nil-vs-empty state explicitly so typed zero values differ from `make`-allocated maps.
- `len` and indexing need central semantic rules for maps instead of runtime-only special cases.
- VM lowering should expose map allocation and lookup explicitly so `dump-bytecode` remains a useful debugging surface.

## Deferred Questions

- When map literals, `delete`, and comma-ok lookups are added, decide whether the checked layer should use separate node kinds or extend the current map-index representation.
- Real Go leaves iteration order unspecified; if later plans expose map printing or iteration more heavily, decide whether deterministic debug rendering should remain intentionally non-Go-like.
