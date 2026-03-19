# For Loop Control Flow

## Goal

Add the first looping construct to `nova-go` without collapsing the current frontend -> semantic -> bytecode layering, and keep the implementation narrow enough to finish milestone `M2`.

## Constraints

- Rust standard library only
- Preserve the checked-program boundary introduced earlier in `M2`
- Reuse the current stack-machine VM instead of inventing a second control-flow representation
- Support only the condition form `for <expr> { ... }` in this slice

## Current Scope

- Source parsing and AST rendering for condition-only `for` loops
- Semantic validation that loop conditions are boolean and loop bodies own a nested scope
- Loop-aware function termination analysis:
  - conditional loops do not guarantee fallthrough elimination
  - `for true { ... }` is treated as non-fallthrough because `break` and `continue` do not exist yet
- Bytecode lowering that reuses `jump` and `jump-if-false`
- CLI inspection through `dump-ast` and `dump-bytecode`

## Deferred Scope

- `for` init / post clauses
- `range` loops
- `break`, `continue`, labels, and loop-carried control-flow diagnostics
- Flow-sensitive reasoning about conditions beyond the literal `true`

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: `Statement::For` keeps loop syntax frontend-only
- `src/semantic/analyzer.rs`: loop condition typing and non-fallthrough analysis stay semantic, not runtime-driven
- `src/bytecode/compiler.rs`: loop lowering is expressed entirely in terms of existing jump instructions
- `docs/tech/semantic-analysis.md`: records the current loop termination approximation so later control-flow work can refine it intentionally
