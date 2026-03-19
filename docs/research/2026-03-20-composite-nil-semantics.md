# Composite Nil Semantics Research

## Goal

Capture the Go compatibility baseline for exposing the predeclared `nil` identifier in `nova-go`, with emphasis on the currently modeled composite runtime surface: slices, maps, assignment, calls, returns, and equality checks.

## Sources Reviewed

- Official Go language specification section `Predeclared identifiers`
- Official Go language specification section `Properties of types and values`
- Official Go language specification section `Assignability`
- Official Go language specification section `Comparison operators`
- Local verification with `go1.21.5` for the edge case `nil == nil`

## Confirmed Findings

- `nil` is a predeclared identifier representing the zero value for pointer, function, slice, map, channel, and interface types.
- The predeclared identifier `nil` cannot initialize a variable that has no explicit type.
- An untyped `nil` value is assignable to slice and map types, which covers the currently modeled `nova-go` composite surface.
- Slice and map values are not generally comparable in Go; they may only be compared to `nil`.
- Equality and inequality with `nil` preserve the distinction between nil and allocated-empty composite values.
- `nil == nil` is rejected by the real Go toolchain with `invalid operation: nil == nil (operator == not defined on untyped nil)`.

## Implementation Implications

- `nova-go` should model `nil` as an explicit untyped checked expression rather than pretending it already has a concrete slice or map type.
- Assignability rules should accept untyped `nil` only when the target context provides a supported composite type such as `[]T` or `map[K]V`.
- Equality validation should stay narrow and explicit: allow `slice/map == nil` and `slice/map != nil`, but keep broader composite equality out of scope.
- Bytecode lowering should materialize `nil` through typed runtime instructions such as `push-nil-slice` and `push-nil-map`, so `dump-bytecode` still explains what the VM will execute.
- The runtime must preserve nil-versus-allocated-empty identity for slices and maps so source-level `nil` comparisons remain observable.

## Deferred Questions

- When `chan`, interface, pointer, and function support arrive later, decide whether the same untyped-`nil` representation can be extended cleanly or needs a more general nilable-type abstraction.
- If multi-value map lookups or `range` are added later, keep `nil` semantics centralized instead of introducing special nil cases in each new feature path.
