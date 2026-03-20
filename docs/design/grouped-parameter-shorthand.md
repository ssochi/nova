# Grouped Parameter Shorthand

## Goal

Add staged support for grouped input parameter declarations such as `func pair(a, b int)` while preserving readable source inspection and the current flattened semantic/runtime function-signature pipeline.

## Constraints

- Rust standard library only
- Preserve the CLI-first workflow and keep `dump-ast` useful without reading Rust implementation details
- Reuse the existing ordered parameter-slot model in semantic analysis, lowering, and runtime execution
- Keep variadic final-parameter rules explicit and unchanged
- Do not mix this syntax slice with named results, naked returns, or unrelated function-type work

## Current Scope

- Frontend AST support for grouped input parameter declarations with one shared type and one or more names
- Parser support for ordinary grouped declarations and grouped declarations ahead of a single final variadic parameter
- Parser diagnostics for invalid grouped variadic declarations such as `func broken(values, more ...int)`
- Semantic flattening from grouped declarations into the existing ordered parameter type list and local parameter bindings
- CLI examples and tests that prove parsing, rendering, semantic analysis, and execution still agree on parameter order

## Deferred Scope

- Named result parameters and naked `return`
- Grouped result declarations
- Methods, receivers, function types as values, or interface-backed call surfaces
- Any changes to bytecode or VM call mechanics beyond consuming the existing flattened parameter list

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: introduce an explicit grouped input-parameter node so grouped names survive source rendering
- `src/frontend/parser.rs`: parse one parameter declaration as `name[, name]* [ ... ] type`, but keep `...` valid only when the declaration contains one final name
- `src/semantic/registry.rs`: flatten grouped declarations into the existing ordered `Vec<Type>` signature metadata
- `src/semantic/analyzer.rs`: build local parameter bindings from flattened parameter names in declaration order and reuse the existing duplicate-name diagnostic path
- `tests/` and `examples/`: keep one focused CLI slice for grouped parameters instead of extending the already large broad integration files again
