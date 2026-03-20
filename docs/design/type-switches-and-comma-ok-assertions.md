# Type Switches and Comma-Ok Assertions

## Goal

Add the next explicit interface-consumption slice for empty-interface values so `nova-go` can use non-panicking type assertions and source-level type switches without collapsing the feature into generic multi-result expressions or ordinary expression switches.

## Constraints

- Keep comma-ok assertions explicit as statement-scoped forms across parsing, semantic analysis, lowering, and runtime execution.
- Keep type switches explicit and distinct from expression switches in the AST and checked model.
- Preserve nil-interface versus boxed typed-nil behavior across comma-ok assertions, `case nil`, and concrete typed cases.
- Limit the slice to the existing empty-interface model; do not imply non-empty interfaces, method sets, or wider implementation checks.
- Keep touched files under the repository line-count ceiling by splitting helpers when parser/compiler/runtime files approach 1000 lines.

## Current Scope

- Statement, header, and `for`-post support for `value, ok := boxed.(T)` and `value, ok = boxed.(T)` through dedicated AST and checked forms.
- Explicit type-switch syntax `switch [header;] [name :=] value.(type)` with clause lists over currently modeled runtime types plus `nil`.
- Clause binding typing that follows the staged real-Go rule:
  - single non-`nil` case type binds the unboxed concrete value
  - multi-type, `nil`, and `any` cases bind the original interface value
- Duplicate staged case diagnostics after canonicalizing `any` and `interface{}` to one runtime type.
- Explicit bytecode/runtime operations for non-panicking interface type checks and readable type-switch lowering.

## Deferred Scope

- Non-empty interfaces, interface implementation checks, interface-to-interface matching beyond `any`, and methods.
- Type-switch `fallthrough`, labels, and control-flow features outside the current `switch` subset.
- Comma-ok type assertions as expression values, general tuple expressions, or arbitrary multi-value contexts outside dedicated statement surfaces.
- Broader runtime categories or reflection-like type metadata.

## Interfaces and Extension Hooks

- `src/frontend/ast.rs` and `src/frontend/parser/statements.rs` should keep comma-ok assertions and type switches explicit instead of hiding them behind generic assignment or expression-switch nodes.
- `src/semantic/model.rs`, `src/semantic/analyzer/interfaces.rs`, and `src/semantic/analyzer/switches.rs` should preserve clause-local binding type decisions explicitly so later non-empty-interface work has a clear seam to extend.
- `src/bytecode/instruction.rs` and compiler helpers should expose non-panicking interface checks as readable instructions so `dump-bytecode` remains useful for debugging dispatch.
- `src/runtime/vm/interfaces.rs` should stay the single owner of interface type matching, assertion execution, and zero-value-on-failure behavior so later assertion or type-switch slices reuse one runtime seam.
