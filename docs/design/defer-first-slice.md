# Defer First Slice

## Goal

Add a first staged `defer` statement that improves Go function-exit behavior while preserving the project's explicit VM-first architecture and readable inspection surfaces.

## Constraints

- Rust standard library only
- Keep deferred calls explicit across AST, checked model, bytecode, and VM execution
- Reuse the existing direct-call surfaces instead of inventing function values or closures
- Preserve `dump-ast` and `dump-bytecode` readability for deferred execution paths
- Stay within the repository file-size limit by splitting near-limit files when needed

## Current Scope

- A source-level `defer` statement whose operand is a non-parenthesized supported direct call
- Semantic reuse of builtin, package, and user-defined call validation, with an added builtin statement-context filter for deferred builtins
- Explicit deferred-call bytecode instructions for direct-call targets, including spread-aware variants where the current call surface already supports them
- VM frame state that records deferred calls in LIFO order and drains them after return-value evaluation but before frame removal
- Focused examples and validation that prove eager argument capture, LIFO ordering, and clear staged diagnostics

## Deferred Scope

- `panic`, `recover`, and panic-triggered unwind behavior
- Function literals, method values, interfaces, or arbitrary callee expressions in `defer`
- Named-result mutation through deferred closures or pointer-based aliasing beyond the currently modeled direct-call subset
- Broad cleanup of the general expression-statement rules outside the new defer path

## Interfaces and Extension Hooks

- `src/frontend/ast.rs` should keep `defer` visible as its own statement kind so `dump-ast` remains source-oriented
- `src/semantic/model.rs` should reuse `CheckedCall` for deferred payloads instead of creating a second call-contract model
- `src/bytecode/instruction.rs` should add dedicated defer instructions rather than hiding the feature behind ordinary `call-*` plus synthetic jumps
- `src/runtime/vm.rs` should store deferred calls on each frame and reuse that stack for future panic-aware unwinding instead of scattering return-site special cases
- Focused CLI tests should live in their own files under `tests/` so the broad execution and diagnostic suites do not keep growing toward the size limit
