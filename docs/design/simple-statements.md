# Simple Statements

## Goal

Expand the current statement and header surface with a deliberate simple-statement model that supports staged short declarations plus explicit `++` / `--` without collapsing them into generic expressions.

## Constraints

- Rust standard library only
- Preserve the explicit AST -> checked model -> bytecode pipeline
- Keep `++` / `--` statement-only
- Keep staged short declarations narrower than full Go until broader multi-result support exists

## Current Scope

- Ordinary statement support for single-expression short declarations such as `count := len(values)`
- Header support for short declarations in `if`, `switch`, and classic `for` init positions
- Explicit increment / decrement statements over identifier and index targets
- Classic `for` post support for increment / decrement while keeping post statements explicit
- Semantic validation for same-block short redeclaration, required new named bindings, and assignment compatibility to reused bindings
- Bytecode lowering that keeps short declarations visible as local-slot creation and inc/dec visible as load-add/store or load-sub/store sequences

## Deferred Scope

- General multi-binding short declarations beyond the existing staged map lookup surface
- Prefix increment/decrement or expression-valued increment/decrement
- Compound assignments, send statements, labels, `goto`, `fallthrough`, `defer`, `go`, and `select`
- Short declarations in classic `for` post clauses

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: explicit statement and header variants for short declarations and inc/dec keep the source surface inspectable
- `src/frontend/parser/statements.rs`: one shared simple-statement parser should decide whether a context allows short declarations, inc/dec, or both
- `src/semantic/analyzer.rs`: short declarations need dedicated same-block redeclaration handling rather than reuse of plain assignment
- `src/semantic/analyzer/ifs.rs` and `src/semantic/analyzer/loops.rs`: header and loop-clause analysis should reuse the shared simple-statement path without leaking bindings
- `src/bytecode/compiler.rs`: inc/dec lowering should remain explicit enough that `dump-bytecode` still reveals loop progression clearly
