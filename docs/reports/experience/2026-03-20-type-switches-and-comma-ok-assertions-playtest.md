# Type Switches and Comma-Ok Assertions Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-14-17-12-type-switches-and-comma-ok-assertions`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

- Ran the real CLI against `examples/type_switches_and_comma_ok.go` through `run`, `dump-ast`, `dump-bytecode`, and `check`.
- Probed one temporary typed-nil interface program through `run` to confirm comma-ok success plus `case []byte` versus `case nil` behavior.
- Probed temporary invalid programs through `check` for non-interface guards, duplicate type-switch cases, and blank type-switch bindings.
- Scope boundary: this is a focused interface-consumption slice playtest, not a milestone-closeout full CLI blackbox pass.

## Positive Experience

- The new surface is readable from the terminal: `dump-ast` shows both `value, ok := boxed.(T)` and `switch current := boxed.(type)` exactly as written, while `dump-bytecode` keeps the runtime checks visible through `type-assert-ok`.
- The example is compact but hits the key behavior edges: failed comma-ok assertion, successful concrete assertion, concrete type-switch case, nil-interface type-switch case, and multi-type case binding all show up in one path.
- Diagnostics are sharper than the old deferred boundary because invalid guards, duplicate cases, and blank bindings now fail with type-switch-specific messages instead of falling back to unrelated parser noise.
- The file-size governance change improved maintainability during the same round: parser and AST splits kept the new work below the repository line cap without hiding the feature.

## Issues and Severity

- Medium: type switches still only cover empty-interface operands and staged runtime types, so interface-heavy Go code remains blocked.
- Medium: clause binding behavior is correct but still subtle; users need `dump-ast` or `dump-bytecode` to see why single-type clauses bind concrete payloads while multi-type clauses bind `any`.
- Low: `case any` currently reflects only the empty-interface world, so broader interface implementation semantics are still missing.

## Conclusion and Next Recommended Steps

The interface-consumption path is materially stronger now because boxed `any` values can be handled through both non-panicking assertions and source-level type switches without leaving the staged VM-first model. The strongest adjacent continuation is broader interface typing, especially non-empty-interface groundwork or the first package/runtime seams that benefit from type-switchable interface payloads.
