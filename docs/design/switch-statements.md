# Switch Statements

## Goal

Add the first staged `switch` surface to `nova-go` so common Go control flow can branch on values or boolean case conditions without leaving the current VM-first architecture.

## Constraints

- Rust standard library only
- Preserve explicit AST -> checked -> bytecode layering
- Reuse the shared control-flow header abstraction instead of creating a second incompatible header model
- Keep `dump-ast` and `dump-bytecode` readable enough to debug clause dispatch directly

## Current Scope

- Expression switches with an optional shared header statement from the current supported simple-statement subset
- Tagless switches that model the source-level omitted expression explicitly
- Comma-separated `case` expression lists and one optional `default`
- Clause-local scopes nested under one shared switch-header scope
- Duplicate `default` and staged duplicate scalar literal-case diagnostics
- Return-path analysis for switches with a `default` whose every clause terminates

## Deferred Scope

- Type switches, `fallthrough`, `break`, `continue`, `goto`, `defer`, and `go`
- General short variable declarations beyond the existing staged header forms
- Nil-only composite switches, interface switches, and richer constant-expression duplicate detection
- Control-flow interactions that depend on future statement-control work

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: keep `switch` explicit with optional header and optional tag expression instead of desugaring tagless switches during parsing
- `src/frontend/parser/statements.rs`: parse switch clauses directly so `case` / `default` boundaries remain source-visible and clause bodies stay brace-free like Go
- `src/semantic/analyzer/ifs.rs`: shared control-flow header analysis is now reusable by both `if` and `switch`
- `src/semantic/analyzer/switches.rs`: clause typing, duplicate-clause diagnostics, and clause-scope analysis stay isolated from other statement analyzers
- `src/bytecode/compiler.rs`: lowering must evaluate the switch tag at most once and keep the hidden storage visible through `dump-bytecode`
