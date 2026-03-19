# Simple Statements

## Goal

Expand the current statement and header surface with a deliberate simple-statement model that supports staged short declarations, explicit `++` / `--`, and staged compound assignments without collapsing them into generic expressions.

## Constraints

- Rust standard library only
- Preserve the explicit AST -> checked model -> bytecode pipeline
- Keep `++` / `--` statement-only
- Keep staged short declarations narrower than full Go until broader multi-result support exists
- Keep compound assignments explicit so index targets still evaluate only once

## Current Scope

- Ordinary statement support for single-expression short declarations such as `count := len(values)`
- Header support for short declarations in `if`, `switch`, and classic `for` init positions
- Explicit increment / decrement statements over identifier and index targets
- Explicit compound assignments `+=`, `-=`, `*=`, and `/=` over identifier and index targets in ordinary statements, `if` / `switch` headers, and classic `for` init / post positions
- Classic `for` post support for increment / decrement while keeping post statements explicit
- Semantic validation for same-block short redeclaration, required new named bindings, and assignment compatibility to reused bindings
- Semantic validation for staged compound-assignment operator coverage across the currently modeled runtime surface (`int`, `byte`, and `string` for `+=`; `int` / `byte` for the numeric operators)
- Bytecode lowering that keeps short declarations visible as local-slot creation, keeps inc/dec visible as load-add/store or load-sub/store sequences, and lowers indexed compound assignments through hidden temporaries so the left side is evaluated once

## Deferred Scope

- General multi-binding short declarations beyond the existing staged map lookup surface
- Prefix increment/decrement or expression-valued increment/decrement
- Modulo, bitwise, and shift compound assignments
- Send statements, labels, `goto`, `fallthrough`, `defer`, `go`, and `select`
- Short declarations in classic `for` post clauses

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: explicit statement and header variants for short declarations, inc/dec, and compound assignments keep the source surface inspectable
- `src/frontend/parser/statements.rs`: one shared simple-statement parser should decide whether a context allows short declarations, compound assignments, inc/dec, or both
- `src/semantic/analyzer.rs`: short declarations need dedicated same-block redeclaration handling rather than reuse of plain assignment
- `src/semantic/analyzer/ifs.rs` and `src/semantic/analyzer/loops.rs`: header and loop-clause analysis should reuse the shared simple-statement path without leaking bindings
- `src/bytecode/compiler/simple_statements.rs`: inc/dec and compound-assignment lowering should remain explicit enough that `dump-bytecode` still reveals loop progression and single-evaluation index handling clearly
