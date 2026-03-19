# Semantic Analysis Layer

## Purpose

Describe the semantic boundary introduced during milestone `M2-frontend-expansion`, including the checked program model consumed by bytecode lowering.

## Pipeline Position

1. Lexer produces tokens.
2. Parser produces `SourceFileAst`.
3. Semantic analysis produces `CheckedProgram`.
4. Bytecode lowering produces `Program`.
5. The VM executes the bytecode program.

## Current Responsibilities

- Collect function signatures before body analysis so forward calls and recursion can resolve by name.
- Validate package-level structure independently from runtime entrypoint rules.
- Track block scopes and map variables to stable local slots.
- Infer the type of each supported expression and reject incompatible assignments, returns, and branch conditions.
- Ensure non-void functions return on every reachable path currently expressible in the supported subset.

## Data Model

- `CheckedProgram`
  - package name
  - entry function index
  - checked functions
- `CheckedFunction`
  - function name
  - parameter count
  - return type
  - linear local-slot name list
  - checked body
- `CheckedExpression`
  - resolved type
  - local-slot or call target resolution

## Driver Contract

- `check` uses package analysis only and does not assume a runtime entrypoint.
- `dump-bytecode` and `run` require semantic analysis plus entrypoint validation through `ExecutionConfig`.
- The bytecode compiler no longer performs symbol discovery; it assumes semantic output is already valid.

## Current Limits

- Supported types are limited to `int`, `bool`, and `void`.
- Package loading is still single-file and does not model imports.
- Return-path analysis only covers the currently supported control-flow forms.
