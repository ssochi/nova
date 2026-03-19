# If Statement Headers

## Goal

Add Go-style `if` header ergonomics on top of the current branch model so common comma-ok `map` checks and preparatory statements can run in-place without widening the language into full statement-header generality everywhere at once.

## Constraints

- Rust standard library only
- Preserve the VM-first pipeline and current AST -> checked -> bytecode layering
- Keep the current staged comma-ok lookup statement explicit instead of turning it into a generic tuple expression
- Keep `dump-ast` and `dump-bytecode` readable enough to show the new branch-header path directly

## Current Scope

- `if <simple-stmt>; <condition> { ... }` for the currently modeled simple-statement subset
- Shared header scope visible to the condition, `then` block, and `else` block
- Explicit `else if` chaining in the source-facing representation
- Parser, semantic analysis, lowering, and CLI inspection updates for the new header form

## Deferred Scope

- `switch` and `for` initializers
- Inc/dec statements, send statements, `go`, `defer`, and general short variable declarations
- A generalized statement-header abstraction shared by every control-flow form
- Source span tracking and richer parser recovery around header semicolons

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: `if` statements should carry optional initializer data and an explicit else-branch representation instead of synthetic surrounding blocks
- `src/frontend/parser.rs`: header parsing should reuse simple-statement parsing helpers without silently consuming statement terminators that belong to outer blocks
- `src/semantic/analyzer.rs`: branch analysis should create one dedicated header scope that wraps condition analysis and both branch blocks
- `src/bytecode/compiler.rs`: lowering should emit initializer instructions before condition evaluation while preserving the scoped local layout chosen by semantic analysis
