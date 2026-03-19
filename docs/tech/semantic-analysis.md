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
- Validate slice and string windows plus indexed slice assignment through the checked-program model instead of runtime-only checks.
- Resolve builtin calls through a centralized contract table instead of hardcoded name checks spread across the analyzer.
- Model `make([]T, len[, cap])` explicitly because its first argument is a type, then lower it into checked slice-allocation expressions before bytecode generation.
- Model explicit conversion syntax separately from ordinary calls because `T(x)` uses a type in callee position rather than a runtime function value.
- Model `byte` explicitly so string indexing and `[]byte` paths do not collapse into ad hoc `int` behavior.
- Validate the builtin `copy` special case for `[]byte` <- `string` centrally instead of hiding it in the runtime.
- Validate loop conditions and model loop bodies as scoped blocks.
- Ensure non-void functions do not fall through on any reachable path in the supported subset.

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

- Supported types are limited to `int`, `byte`, `bool`, `string`, `[]T`, and `void`, and the current type-valued syntax surface is limited to builtin `make` plus explicit `[]byte(string)` / `string([]byte)` conversions.
- Package loading is still single-file and does not model imports.
- Loop support is limited to `for <condition> { ... }`.
- Termination analysis only treats the literal `for true { ... }` as definitely non-fallthrough because `break` and `continue` do not exist yet.
- Builtin coverage is still intentionally small, and conversions are now deliberately modeled outside the builtin table.
- Slice support is still staged: simple slice expressions on `[]T` and `string` are supported, while full slice expressions remain deferred.
- General conversion syntax beyond the narrow `[]byte(string)` / `string([]byte)` pair is still deferred.
