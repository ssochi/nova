# Type Assertions First Slice

## Goal

Add the first explicit `x.(T)` type-assertion slice for empty-interface values so boxed `any` payloads can be consumed as concrete staged runtime values without collapsing assertions into calls, conversions, or hidden runtime special cases.

## Constraints

- Keep the slice limited to single-result assertions on `any` / `interface{}` operands.
- Keep assertion syntax explicit in the AST, checked model, bytecode, and VM runtime path so `dump-ast` and `dump-bytecode` stay useful.
- Preserve nil-vs-boxed interface state and typed-nil composite payload fidelity.
- Keep touched files under the repository line-count ceiling by splitting parser, semantic, compiler, or VM helpers if needed.

## Current Scope

- Parse postfix type assertions `expression.(Type)` as a dedicated expression form.
- Reject assertions on non-interface operands during semantic analysis with a compile-time diagnostic.
- Support assertion targets across the currently modeled runtime-type surface: scalar types, `string`, `[]T`, `map[K]V`, `chan T`, and `any`.
- Lower assertions through a dedicated bytecode instruction that names the asserted runtime type.
- Execute assertions in the VM through the existing interface helper seam, returning the unboxed payload on success and raising a user-visible interface-conversion panic on failure.
- Keep nil-interface assertion failures and mismatched dynamic-type failures distinct in the runtime diagnostic text.

## Deferred Scope

- Comma-ok assertions such as `value, ok := x.(T)`.
- Type switches, non-empty interfaces, methods, method sets, and dynamic dispatch.
- Assertions from broader interface hierarchies once non-empty interfaces exist.
- Exact Go runtime type names in every panic message beyond the currently modeled runtime type rendering.

## Interfaces and Extension Hooks

- `src/frontend/ast.rs` and `src/frontend/parser.rs` should keep type assertions explicit as postfix expressions rather than lowering them into selector or call nodes.
- `src/semantic/model.rs` and `src/semantic/analyzer/expressions.rs` should expose a dedicated checked assertion node with the asserted destination type kept intact for bytecode lowering.
- `src/bytecode/instruction.rs` and `src/bytecode/compiler.rs` should use an explicit assertion instruction so inspection surfaces show the target runtime type directly.
- `src/runtime/vm/interfaces.rs` should own assertion execution and panic construction so future comma-ok assertions or richer interface work reuse one runtime seam.
