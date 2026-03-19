# Map Comma-Ok Lookups and Literal Diagnostics

## Goal

Capture the official Go behavior baseline needed for the next `nova-go` `map` slice, with emphasis on comma-ok lookups, staged short redeclaration rules, and duplicate constant-key diagnostics for map literals.

## Sources Reviewed

- Official Go language specification section `Index expressions`
- Official Go language specification section `Assignment statements`
- Official Go language specification section `Short variable declarations`
- Official Go language specification section `Composite literals`
- Local Go 1.21.5 spot checks for nil-map comma-ok reads, short redeclaration, and duplicate literal-key failures

## Confirmed Findings

- A `map` index expression used in assignment or initialization may produce an additional untyped boolean result that reports whether the key is present.
- The single-result form `m[key]` still yields the element zero value when the key is absent or the map is nil; the comma-ok form on a nil map yields that zero value plus `false`.
- The staged comma-ok surface most often appears as `value, ok := m[key]` or `value, ok = m[key]`; it does not require general tuple expressions to be useful.
- Short variable declarations allow redeclaration in the same block only when the names were declared earlier in that block with the same types and at least one non-blank identifier on the left side is new.
- The blank identifier does not count as a new variable for `:=`.
- Real Go rejects duplicate constant keys within a map literal; in the current `nova-go` subset, the comparable scalar literal keys already modeled directly in the AST are enough to capture the most important part of that rule.

## Implementation Implications

- The next `nova-go` slice can keep comma-ok lookup explicit as a statement form instead of introducing a generic tuple value category.
- The checked layer should model the second `bool` result explicitly so bytecode lowering and CLI debug surfaces can expose how the presence check is computed.
- Short-declaration freshness checks should stay centralized and shared with other staged binding forms when possible instead of duplicating ad hoc parser-only rules.
- Duplicate constant-key diagnostics can run during semantic analysis by inspecting currently supported scalar literal key expressions after type checking, while non-literal or future constant-expression cases stay deferred.

## Deferred Questions

- Whether later `if` / `switch` initializer support should reuse the same comma-ok lookup statement node or a broader statement-header abstraction.
- Whether duplicate-key diagnostics should grow into a general constant-evaluation pass once richer constant expressions enter the language.
