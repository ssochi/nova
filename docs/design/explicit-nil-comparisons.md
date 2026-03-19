# Explicit Nil Comparisons

## Goal

Expose Go-like source-level `nil` handling for the currently modeled composite types without overcommitting to a full generalized nilable-type system.

## Constraints

- Only the Rust standard library may be used.
- The implementation must keep `nil` explicit through AST, checked model, bytecode, and VM layers.
- The design must preserve the existing runtime distinction between nil and allocated-empty slices/maps.

## Current Scope

- Parse and render `nil` as a first-class expression.
- Introduce an explicit untyped-`nil` checked expression kind for semantic analysis.
- Allow untyped `nil` in typed slice/map contexts: variable initialization, assignment, return values, call arguments, and equality checks.
- Restrict equality support to `slice/map` compared with `nil`, while keeping broader composite equality rejected.
- Reuse `push-nil-slice` and `push-nil-map` during lowering so CLI bytecode inspection stays readable.

## Deferred Scope

- `chan`, pointer, interface, and function `nil` support.
- `nil` in untyped contexts such as `var value = nil` or `nil == nil`.
- General tuple/multi-result features that would be needed for comma-ok map lookup.
- Broader composite comparability beyond `nil` checks.

## Interfaces and Extension Hooks

- Keep AST `nil` explicit instead of treating it as an identifier lookup.
- Use a dedicated semantic `Type::UntypedNil` marker so assignability and equality helpers can centralize the compatibility rules.
- Lower typed `nil` contexts into the existing zero-value bytecode instructions for slices and maps rather than inventing a generic boxed-nil runtime value.
- Add shared helper logic for “nilable composite target” checks so later channel support can extend one place instead of scattering conditionals.
