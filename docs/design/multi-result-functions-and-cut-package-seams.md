# Multi-Result Functions and Cut Package Seams

## Goal

Add the first staged multi-result model to the VM-first compiler pipeline so broader package APIs and user-defined function patterns become possible without inventing first-class tuple runtime values.

## Constraints

- Only the Rust standard library may be used.
- Multi-result behavior must stay explicit in the AST, checked model, bytecode, and VM layers.
- The implementation must preserve readable `dump-ast` / `dump-bytecode` output and keep individual source files under the 1000-line limit.

## Current Scope

- Support unnamed multi-result function signatures such as `func pair() (int, string)`.
- Support explicit `return` statements with multiple expressions and direct forwarding from a single multi-result call.
- Support staged multi-binding short declarations and identifier/blank assignments when the right side is either an equal-length expression list or one multi-result call.
- Keep ordinary expressions single-valued; multi-result calls are only valid in the staged assignment/return surfaces of this round.
- Extend user-defined call signatures and package contracts so a call can report zero, one, or multiple result types explicitly.
- Add `strings.Cut` and `bytes.Cut` as the first package contracts that return multiple values.

## Deferred Scope

- Named result parameters, naked returns, and result-slot initialization
- General tuple expressions, tuple runtime values, or multi-result values in arbitrary argument/condition positions
- Multi-result builtins, comma-ok receive, and refactoring staged comma-ok `map` lookups into generic tuple expressions
- Multi-target assignments that require single-evaluation index-target lowering across several left-hand sides

## Interfaces and Extension Hooks

- `src/frontend/ast.rs` should keep result lists and staged multi-binding statements explicit enough that `dump-ast` can show the source-level surface directly.
- `src/semantic/registry.rs`, `src/semantic/builtins.rs`, and `src/semantic/packages.rs` should converge on explicit result lists instead of single return types plus ad hoc exceptions.
- `src/semantic/model.rs` should represent multi-result call sources separately from ordinary single-valued expressions so later staged forms can reuse the same boundary.
- `src/bytecode/instruction.rs` and `src/runtime/vm.rs` should replace the current boolean return metadata with explicit result-count or result-type metadata for every compiled function and package call path.
- `strings.Cut` / `bytes.Cut` should be added through the same contract and dispatch tables as the existing `fmt`, `strings`, and `bytes` functions so later package seams do not bypass the centralized model.
