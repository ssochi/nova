# Semantic Functions and Branches

## Goal

Add the first non-trivial semantic layer to `nova-go`, then use it to unlock multi-function execution, value returns, boolean comparisons, and `if` / `else` control flow on the VM path.

## Constraints

- Rust standard library only
- Preserve the existing CLI-first workflow and module layering
- Keep the type surface intentionally narrow so the milestone does not balloon into full Go typing
- Leave obvious extension seams for loops, richer types, and a later backend-oriented IR

## Current Scope

- Function signatures with named parameters and optional return types
- Supported source-level types: `int`, `bool`, and omitted return type as `void`
- Semantic checks for function lookup, variable scoping, assignment compatibility, call arity, return consistency, and boolean branch conditions
- User-defined function calls with VM call frames
- Comparison operators and `if` / `else` lowering

## Deferred Scope

- `for`, `range`, `switch`, `defer`, `go`, and other control-flow forms
- Composite types, interfaces, pointers, and package imports
- Source-span-rich diagnostics and multi-file package loading
- Backend-facing IR that is decoupled from the current VM bytecode encoding

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: syntax tree now carries signatures and branch statements without depending on semantic concepts
- `src/semantic/analyzer.rs`: package analysis and entrypoint validation are separate so `check` can stay package-oriented while `run` remains execution-oriented
- `src/bytecode/compiler.rs`: lowers `CheckedProgram` instead of raw AST, which keeps symbol resolution and type checks out of the VM-facing compiler
- `src/runtime/vm.rs`: call frames and branch jumps are isolated runtime concerns that later loop instructions can reuse
