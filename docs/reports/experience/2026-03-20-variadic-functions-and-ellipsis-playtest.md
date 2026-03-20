# Variadic Functions and Explicit Ellipsis Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-09-21-38-variadic-functions-ellipsis`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/variadic.go` to exercise user-defined variadic functions, ordinary variadic arguments, final `slice...` spreading, nil-slice spreading, and both slice/string append spread paths in one CLI flow.
2. Ran `cargo run -- dump-ast examples/variadic.go` to confirm the source-facing story stayed readable without reading implementation details.
3. Ran `cargo run -- dump-bytecode examples/variadic.go` to confirm variadic metadata and explicit spread calls stayed inspectable in the bytecode output.
4. Ran `cargo run -- check <temp source with println(total(1, values...))>` to confirm the fixed-prefix rule still fails with a targeted diagnostic when a spread call includes an extra variadic scalar before the final `...` argument.

## Positive Experience

- The CLI now supports a common Go helper shape that was previously missing: user-defined functions can declare `...` parameters and consume either ordinary scalars or a final slice spread.
- Zero-variadic-argument and nil-spread behavior are easy to inspect from the CLI because the example shows both `values == nil` and `len(values)` inside the callee.
- `dump-bytecode` remains useful after the change because the spread path is explicit through `call-function-spread` and `call-builtin-spread` instead of being lowered into opaque helper code.
- `append([]byte, string...)` now makes the byte/string interoperability slice feel less artificial and more like real Go source.

## Issues and Severity

- Medium: explicit `...` is still narrower than the full language surface because package-backed variadic slice forwarding remains unavailable until an interface or `[]any` model exists.
- Medium: grouped parameter-name shorthand still remains missing, so some ordinary Go signatures continue to require more verbose examples than real Go would.

## Conclusion and Next Recommended Steps

The real CLI path is materially better: user-defined variadic helpers work, explicit spread calls are readable in both AST and bytecode views, and the long-deferred append spread cases now execute end to end. The strongest next continuation is to decide whether `M3` should spend this new function-signature groundwork on grouped parameter shorthand or on another package/API slice that benefits from the explicit variadic model without requiring interfaces.
