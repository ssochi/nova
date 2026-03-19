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
- Model typed `make` expressions explicitly because their first argument is a type, then lower slices and maps into dedicated checked allocation expressions before bytecode generation.
- Model staged map literals explicitly in the checked layer instead of hiding them behind synthetic `make` plus index assignment trees.
- Model explicit conversion syntax separately from ordinary calls because `T(x)` uses a type in callee position rather than a runtime function value.
- Model explicit `nil` as a dedicated untyped checked expression and coerce it centrally when typed slice/map context is available.
- Model `byte` explicitly so string indexing and `[]byte` paths do not collapse into ad hoc `int` behavior.
- Validate the builtin `copy` special case for `[]byte` <- `string` centrally instead of hiding it in the runtime.
- Validate staged map key comparability centrally so unsupported key types fail during semantic analysis before reaching the VM.
- Validate staged comma-ok map lookup statements centrally, including map-only right-hand sides, typed `=` assignments, same-block `:=` freshness rules, and blank-identifier handling.
- Validate staged `if` statement headers centrally, including the current simple-statement subset, dedicated header scopes shared by the condition / `then` / `else` path, and explicit `else if` chaining.
- Validate staged expression `switch` statements centrally, including shared header scopes, tagless `switch`, clause-local scopes, duplicate `default` rejection, and the current duplicate scalar literal-case diagnostics.
- Validate staged short declarations centrally so they remain explicit, create fresh locals in the current scope, and reject current-scope reuse without a broader multi-binding model.
- Validate staged classic `for` clauses centrally, including dedicated init scope, optional condition / post handling, and the current post-statement subset.
- Validate explicit `++` / `--` centrally so they remain statement-only and only apply to assignable `int` / `byte` targets.
- Validate unlabeled `break` / `continue` centrally against the nearest enclosing modeled control-flow target instead of leaving invalid jumps to bytecode lowering.
- Validate builtin `delete(map, key)` centrally so map mutation rules stay aligned with map indexing and assignment typing.
- Validate `slice/map == nil` and `slice/map != nil` centrally while continuing to reject broader composite equality.
- Validate staged `range` loops over `slice` and `map`, including no-binding, `:=`, and `=` forms, nil zero-iteration behavior, and typed iteration-variable assignments.
- Validate loop conditions, classic-clause scoping, and loop bodies as scoped blocks.
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

- Supported concrete runtime types are limited to `int`, `byte`, `bool`, `string`, `[]T`, `map[K]V`, and `void`; semantic analysis also carries a dedicated untyped `nil` marker until typed slice/map context resolves it.
- Package loading is still single-file and does not model imports.
- Loop support is staged but broader: infinite `for`, condition-only `for`, classic `for init; condition; post`, single-expression short declarations in `for` init, explicit `++` / `--`, unlabeled `break`, unlabeled `continue`, and `range` over `slice` / `map` are supported.
- Termination analysis remains conservative: infinite or literal-`true` loops only count as non-fallthrough when no modeled `break` can escape the loop, and terminating `switch` clauses fail that classification when a clause can `break`.
- Builtin coverage is still intentionally small, and conversions are now deliberately modeled outside the builtin table.
- Slice support is still staged: simple slice expressions on `[]T` and `string` are supported, while full slice expressions remain deferred.
- Map support is still staged: explicit `nil`, map literals, duplicate constant literal-key diagnostics, single-result indexing, statement-scoped comma-ok lookups, `len`, nil-map zero values, `make`, `delete`, index assignment, `nil` equality, and staged `range` loops are supported, while general tuple expressions, broader constant folding, and richer lookup contexts remain deferred.
- Explicit `nil` still needs typed slice/map context; `var value = nil`, `nil == nil`, and broader nilable-type work remain deferred.
- General conversion syntax beyond the narrow `[]byte(string)` / `string([]byte)` pair is still deferred.
- Range support is still staged: only `slice` and `map` are iterable, assignment-form left sides are identifier-only, and string/channel/integer/function ranges remain deferred.
- Statement-header support is still staged: `if` and expression `switch` support header simple statements, and that support is limited to expression statements, assignments, `var` declarations, single-expression short declarations, explicit `++` / `--`, and staged comma-ok `map` lookups.
- Short declarations are still staged: the general form is intentionally limited to one named binding plus one expression, while the existing explicit comma-ok map lookup path continues to cover the current two-binding lookup case.
- Switch support is still staged: only expression and tagless `switch` are supported, with no type switches, `fallthrough`, labels, or broader constant-expression duplicate detection.
