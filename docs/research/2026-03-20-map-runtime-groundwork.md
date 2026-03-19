# Map Runtime Groundwork Research

## Goal

Capture the official Go behavior baseline needed for staged map work in `nova-go`, with emphasis on `map[K]V` type syntax, `make`, indexing, assignment, `len`, composite literals, and builtin `delete`.

## Sources Reviewed

- Official Go language specification section `Map types`
- Official Go language specification section `Properties of types and values`
- Official Go language specification section `Index expressions`
- Official Go language specification section `Length and capacity`
- Official Go language specification section `Making slices, maps and channels`
- Official Go language specification section `Composite literals`
- Official Go language specification section `Deletion of map elements`

## Confirmed Findings

- `map[K]V` is a distinct reference-like type. The zero value of a map is `nil`.
- Map keys must support `==` and `!=`; slices, maps, and functions are not valid key types.
- `make(map[K]V)` and `make(map[K]V, hint)` produce a non-nil ready-to-write map value. The size hint is optional and does not change the map length.
- `len(map)` returns the current number of defined entries. `len(nilMap)` is `0`.
- `value := m[key]` returns the element value when the key exists; otherwise it returns the zero value of the element type.
- Assigning through `m[key] = value` inserts or replaces an entry, but assigning into a nil map triggers a runtime failure in real Go.
- `map[K]V{}` is a non-nil empty map value; keyed map literals allocate a writable map and evaluate each key/value pair in source order.
- Map literal elements always use `key: value` form. Real Go rejects duplicate constant keys in the same literal, while non-constant duplicates are decided by evaluation order.
- `delete(m, key)` removes the entry when present. Deleting from a nil map or deleting a missing key is a no-op.
- The single-result form `m[key]` is enough for the current slice; the comma-ok lookup form can be deferred without blocking basic map usability.

## Implementation Implications

- The first `nova-go` map slice can stay focused on scalar comparable key types already present in the runtime model: `int`, `byte`, `bool`, and `string`.
- Map support should model nil-vs-empty state explicitly so typed zero values differ from `make`-allocated maps.
- `len` and indexing need central semantic rules for maps instead of runtime-only special cases.
- VM lowering should expose map allocation and lookup explicitly so `dump-bytecode` remains a useful debugging surface.
- A staged literal implementation can lower into a dedicated map-construction bytecode instruction so AST and bytecode views show map creation directly.
- `delete` fits the existing builtin contract table, but nil-map no-op behavior must remain explicit in the runtime rather than piggybacking on insertion logic.

## Deferred Questions

- If the staged literal implementation does not reject duplicate constant keys yet, record that gap explicitly in reports and keep the runtime behavior deterministic.
- Real Go leaves iteration order unspecified; if later plans expose map printing or iteration more heavily, decide whether deterministic debug rendering should remain intentionally non-Go-like.
