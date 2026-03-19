# For Loop Control Flow

## Goal

Expand `nova-go` loop control beyond the original condition-only `for` slice by adding classic `for` clauses, staged simple-statement init / post support, and unlabeled `break` / `continue`, while keeping the frontend -> semantic -> bytecode layering explicit and inspectable.

## Constraints

- Rust standard library only
- Preserve the checked-program boundary used by bytecode lowering
- Keep `break` / `continue` targets explicit instead of hiding them in ad hoc jump rewriting
- Support only the currently staged simple-statement subset for classic `for` init / post work

## Current Scope

- Source parsing and AST rendering for:
  - infinite `for { ... }`
  - condition-only `for <expr> { ... }`
  - classic `for init; condition; post { ... }` with optional init / condition / post
  - staged short declarations in classic `for` init
  - explicit `++` / `--` in ordinary statements, headers, and classic `for` post clauses
  - staged `range` loops over `slice` and `map`
  - unlabeled `break` and `continue`
- Semantic validation that:
  - classic `for` init bindings live in a dedicated scope shared by condition, post, and body
  - staged short declarations stay explicit and create fresh locals in the current scope
  - `++` / `--` stay statement-only and require `int` or `byte` targets
  - `break` requires an enclosing `for`, `range`, or `switch`
  - `continue` requires an enclosing `for` or `range`
  - `for` conditions remain boolean when present
- Loop-aware and switch-aware termination analysis that keeps infinite-loop and switch-return reasoning conservative when modeled `break` paths exist
- Bytecode lowering that reuses the existing jump instruction set while tracking explicit loop / switch patch targets for `break` and `continue`
- CLI inspection through `dump-ast` and `dump-bytecode`

## Deferred Scope

- Labels, labeled `break` / `continue`, `goto`, `fallthrough`, `defer`, `go`, and `select`
- Send statements, compound assignments, and general multi-binding short variable declaration support in loop clauses
- String / channel / integer / iterator-function `range`
- Flow-sensitive reasoning beyond omitted conditions and the literal `true`

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: `ForStatement` and `ForPostStatement` keep classic `for` clauses explicit in the frontend model
- `src/frontend/ast.rs`: `IncDecOperator` plus explicit short-declaration/header/post variants keep staged simple statements inspectable at the source layer
- `src/semantic/model.rs`: `CheckedForStatement` and `CheckedForPostStatement` preserve loop structure across semantic analysis and lowering
- `src/semantic/analyzer/loops.rs`: loop-specific scope, control-target validation, and post-statement checking stay separate from generic expression analysis
- `src/semantic/analyzer.rs`: short declarations and inc/dec stay explicit instead of being rewritten into plain assignment before checking
- `src/semantic/support.rs`: loop / switch termination analysis remains semantic, not runtime-driven
- `src/bytecode/compiler.rs`: loop and switch lowering share one explicit control-flow stack for `break` / `continue` patching
