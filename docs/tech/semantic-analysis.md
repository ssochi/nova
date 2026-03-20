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
- Model `chan` explicitly so send statements, receive expressions, and nil/equality behavior remain visible in the checked layer instead of being hidden inside builtin dispatch.
- Validate the builtin `copy` special case for `[]byte` <- `string` centrally instead of hiding it in the runtime.
- Validate staged map key comparability centrally so unsupported key types fail during semantic analysis before reaching the VM.
- Validate staged comma-ok map lookup statements centrally, including map-only right-hand sides, typed `=` assignments, same-block `:=` freshness rules, and blank-identifier handling.
- Validate staged `if` statement headers centrally, including the current simple-statement subset, dedicated header scopes shared by the condition / `then` / `else` path, and explicit `else if` chaining.
- Validate staged expression `switch` statements centrally, including shared header scopes, tagless `switch`, clause-local scopes, duplicate `default` rejection, and the current duplicate scalar literal-case diagnostics.
- Validate staged short declarations centrally so they remain explicit, support the current same-block redeclaration rules when at least one named binding is new, and keep multi-binding result flow separate from plain assignment targets.
- Validate staged multi-result returns, assignment-like usage, and single-call-argument forwarding centrally so broader package seams can reuse the same non-tuple result model.
- Validate staged variadic function declarations and explicit final-argument `...` calls centrally so user-defined helpers and `append` spread behavior stay explicit instead of disappearing into generic flat argument lists.
- Validate staged compound assignments centrally so they remain explicit, reuse assignable-target checking, keep operator support aligned with the modeled runtime surface, and preserve single-evaluation index semantics during lowering.
- Validate staged classic `for` clauses centrally, including dedicated init scope, optional condition / post handling, and the current post-statement subset.
- Validate explicit `++` / `--` centrally so they remain statement-only and only apply to assignable `int` / `byte` targets.
- Validate unlabeled `break` / `continue` centrally against the nearest enclosing modeled control-flow target instead of leaving invalid jumps to bytecode lowering.
- Validate builtin `delete(map, key)` centrally so map mutation rules stay aligned with map indexing and assignment typing.
- Validate builtin `close(chan)` centrally so channel-closing rules stay aligned with channel typing instead of leaking into runtime-only checks.
- Validate `slice/map/chan == nil` and `slice/map/chan != nil` centrally while continuing to reject broader composite equality beyond the currently modeled channel identity case.
- Validate staged send statements and receive expressions centrally so channel typing, nil coercion, and the current runtime limits are explicit before lowering.
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
  - optional variadic element type for the final parameter
  - return type list
  - linear local-slot name list
  - checked body
- `CheckedExpression`
  - resolved type
  - local-slot or call target resolution
- `CheckedValueSource`
  - explicit expression list or explicit multi-result call source used by staged `return`, `:=`, and `=` forms
- `CheckedCall`
  - explicit ordinary-arguments vs expanded-call-argument vs explicit spread-argument source so call forwarding and `...` do not disappear into flat expression lists too early

## Driver Contract

- `check` uses package analysis only and does not assume a runtime entrypoint.
- `dump-bytecode` and `run` require semantic analysis plus entrypoint validation through `ExecutionConfig`.
- The bytecode compiler no longer performs symbol discovery; it assumes semantic output is already valid.

## Current Limits

- Supported concrete runtime types are limited to `int`, `byte`, `bool`, `string`, `[]T`, `chan T`, `map[K]V`, and `void`; semantic analysis also carries a dedicated untyped `nil` marker until typed slice/map/channel context resolves it.
- Package loading is still single-file and does not model imports.
- Loop support is staged but broader: infinite `for`, condition-only `for`, classic `for init; condition; post`, single-expression short declarations in `for` init, staged compound assignments, explicit `++` / `--`, unlabeled `break`, unlabeled `continue`, and `range` over `slice` / `map` are supported.
- Termination analysis remains conservative: infinite or literal-`true` loops only count as non-fallthrough when no modeled `break` can escape the loop, and terminating `switch` clauses fail that classification when a clause can `break`.
- Builtin coverage is still intentionally small, and conversions are now deliberately modeled outside the builtin table.
- Function and package calls now support zero, one, or multiple results explicitly, but those results are still not first-class tuple expressions.
- Call forwarding is still staged: a multi-result call may feed another call only when it is the entire argument list by itself, while prefixed forms such as `f(1, pair())` remain invalid single-value contexts.
- User-defined functions now also support staged final variadic parameters, and calls may use explicit final `...` spreading only for the fixed-prefix-plus-spread shape required by real Go; broader package-backed variadic slice forwarding still remains deferred.
- Slice support is still staged: simple slice expressions on `[]T` and `string` are supported, while full slice expressions remain deferred.
- Map support is still staged: explicit `nil`, map literals, duplicate constant literal-key diagnostics, single-result indexing, statement-scoped comma-ok lookups, `len`, nil-map zero values, `make`, `delete`, index assignment, `nil` equality, and staged `range` loops are supported, while general tuple expressions, broader constant folding, and richer lookup contexts remain deferred.
- Channel support is now staged: bidirectional `chan T`, `make`, `len`, `cap`, builtin `close`, send statements, receive expressions, `nil` equality, and same-type channel equality are supported, while directions, channel `range`, comma-ok receive, and scheduler-aware blocking semantics remain deferred.
- Explicit `nil` still needs typed slice/map/channel context; `var value = nil`, `nil == nil`, and broader nilable-type work remain deferred.
- General conversion syntax beyond the narrow `[]byte(string)` / `string([]byte)` pair is still deferred.
- Range support is still staged: only `slice` and `map` are iterable, assignment-form left sides are identifier-only, and string/channel/integer/function ranges remain deferred.
- Statement-header support is still staged: `if` and expression `switch` support header simple statements, and that support is limited to expression statements, assignments, staged multi-binding `=` / `:=`, staged compound assignments, `var` declarations, explicit `++` / `--`, and staged comma-ok `map` lookups.
- Send statements remain ordinary statements only in the current slice; they are not yet part of the staged header or `for` post-statement subset.
- Short declarations are still staged: multi-binding `:=` now works for expression lists and single multi-result calls, while the existing explicit comma-ok map lookup path remains a dedicated statement form.
- Compound assignments are still staged: `+=`, `-=`, `*=`, and `/=` are supported in explicit statement positions, while modulo, bitwise, and shift assignment operators remain deferred with their wider expression support.
- Switch support is still staged: only expression and tagless `switch` are supported, with no type switches, `fallthrough`, labels, or broader constant-expression duplicate detection.
