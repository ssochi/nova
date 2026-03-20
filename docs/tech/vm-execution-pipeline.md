# VM Execution Pipeline

## Purpose

Describe the concrete execution pipeline shipped in the bootstrap milestone, including its module boundaries and known extension seams.

## Module Flow

1. `src/cli.rs`
   - Parses subcommands and optional execution overrides.
   - Produces a typed `Command` enum instead of passing raw strings downstream.
2. `src/driver.rs`
   - Orchestrates file loading, lexing, parsing, semantic analysis, compilation, and VM execution.
   - Normalizes error categories into user-facing CLI diagnostics.
3. `src/source.rs`
   - Encapsulates source loading and path retention.
4. `src/frontend/lexer.rs`
   - Converts source text into tokens.
   - Performs lightweight newline-to-semicolon insertion for the supported subset.
5. `src/frontend/parser.rs`
   - Builds a small AST for package/import/function/block/expression constructs.
6. `src/semantic/analyzer.rs`
   - Resolves names, validates imports and function signatures, and produces a checked program.
7. `src/bytecode/compiler.rs`
   - Lowers the checked program into stack-machine bytecode for every function.
8. `src/runtime/vm.rs`
   - Executes bytecode with call frames, local slots, operand stack, and buffered output.

## Current Data Boundaries

- Source boundary: `SourceFile`
- Frontend boundary: `Token` -> `SourceFileAst`
- Semantic boundary: `SourceFileAst` -> `CheckedProgram`
- Compilation boundary: `CheckedProgram` -> `Program`
- Runtime boundary: `Program` -> `ExecutionResult`

## Current Semantic Rules

- Package analysis validates duplicate function names, variable scopes, call arity, and zero/one/multi-result return compatibility.
- Package analysis also flattens grouped input parameter declarations such as `func f(a, b int)` into the ordered parameter-slot metadata used by lowering and the VM.
- Package analysis also flattens explicit result declarations such as `func f() (left, right string)` into ordered result-slot metadata while keeping named result bindings visible to semantic scope checks.
- Package analysis also validates supported import paths and selector calls to imported package members.
- Package analysis now also validates staged `defer` statements as explicit statement nodes, reusing ordinary call contracts while rejecting parenthesized defer operands and builtins that are not valid in statement context.
- Package analysis now also resolves both `any` and `interface{}` into one explicit empty-interface type and keeps coercions into that type visible through checked boxing nodes instead of silent assignment-only special cases.
- Package analysis now also treats explicit builtin `panic(...)` calls as terminating paths for conservative non-void fallthrough checks, even though a deferred `recover()` may later convert that panic into a normal return.
- Execution additionally requires the configured package and entry function to exist, and the entry function must be `func main()`.
- Local variables must be declared before assignment or use, with nested block scopes mapped to fixed slots during analysis.
- Builtin calls, user-defined function calls, and metadata-backed `fmt` / `strings` / `bytes` package seams are supported.
- Current builtin coverage includes `print`, `println`, `len`, `cap`, `append`, `copy`, `delete`, `close`, `clear`, `panic`, `recover`, and typed `make` handling.
- Current imported package coverage is `fmt.Print`, `fmt.Println`, `fmt.Sprint`, `strings.Compare`, `strings.Clone`, `strings.Join`, `strings.Contains`, `strings.HasPrefix`, `strings.HasSuffix`, `strings.Index`, `strings.LastIndex`, `strings.IndexByte`, `strings.LastIndexByte`, `strings.Cut`, `strings.CutPrefix`, `strings.CutSuffix`, `strings.TrimPrefix`, `strings.TrimSuffix`, `strings.Repeat`, `bytes.Compare`, `bytes.Clone`, `bytes.Equal`, `bytes.Contains`, `bytes.HasPrefix`, `bytes.HasSuffix`, `bytes.Index`, `bytes.LastIndex`, `bytes.IndexByte`, `bytes.LastIndexByte`, `bytes.Cut`, `bytes.CutPrefix`, `bytes.CutSuffix`, `bytes.TrimPrefix`, `bytes.TrimSuffix`, `bytes.Join`, and `bytes.Repeat`.
- The staged empty-interface slice now supports declarations, returns, conversions, and `[]any` literals, while `fmt.Print*` additionally accepts explicit `args...` when `args` is `[]any`.
- Compiled-function metadata now records explicit result lists instead of a boolean `returns_value` flag, so the VM can return zero, one, or several values through the same stack-frame path.
- Bytecode lowering now also emits explicit zero-value stores for named result locals at function entry, because VM local-slot defaults are not type-aware.
- The current multi-result model is explicit rather than tuple-based: staged `return`, multi-binding `:=` / `=`, single-call-argument forwarding, and package seams can consume multi-result calls, while unsupported single-value contexts still fail during semantic analysis.
- Bare `return` now lowers through explicit reads of tracked result locals, and semantic analysis rejects bare returns whose named result bindings are shadowed out of scope.
- Deferred calls now lower through explicit defer instructions instead of synthetic tail blocks, keeping eager argument capture and LIFO execution visible in `dump-bytecode`.
- Explicit builtin `panic` now lowers through dedicated typed panic instructions rather than a generic builtin call so `dump-bytecode` keeps panic entry points readable.
- Explicit builtin `recover()` stays visible as an ordinary builtin call so `dump-bytecode` exposes the difference between recoverable deferred user-function frames and non-recovering helper/builtin paths.
- Empty-interface coercions now lower through explicit `box-any <type>` bytecode, while nil interface zero values lower through `push-nil-interface`.
- Runtime dispatch inside `src/runtime/vm/` is now split between `builtins.rs`, `calls.rs`, `packages.rs`, `support.rs`, and `unwind.rs` so panic/call growth does not keep accumulating in one VM file.
- Runtime interface values now carry explicit nil-vs-boxed state plus the boxed dynamic runtime type, so nil checks and staged interface equality do not collapse boxed typed-nil composites into nil interfaces.
- VM call frames now retain both pending return values and a deferred-call stack, while the VM also tracks a pending panic payload plus unwind depth so staged `defer` can run during ordinary returns and panic propagation through the same frame model.
- Directly deferred user-function frames now also carry explicit recover-eligibility metadata, letting `recover()` stop the active panic only in that precise context while helper calls and deferred builtin `recover()` still produce nil.
- Branch and loop conditions must produce boolean values, staged control-flow headers run before condition or clause dispatch, and expression-switch tags are evaluated once before clause comparison.
- The current branch model supports `if`, `else`, explicit `else if`, and staged expression `switch` lowering with header scopes chosen during semantic analysis.
- Single-expression short declarations, staged compound assignments, and explicit `++` / `--` are now explicit statement forms that survive semantic analysis and lower directly instead of pretending to be expressions.
- Infinite / condition-only / classic-clause `for` loops plus staged `range` loops over `slice` / `map` are lowered into the existing jump and helper instruction set.
- Loop and `switch` control transfer now share an explicit lowering-time control-flow stack so unlabeled `break` / `continue` patch to readable jump targets.
- Index-target inc/dec lowering now caches target / index evaluation into hidden locals so bytecode preserves single-evaluation behavior while keeping `dump-bytecode` readable.
- Index-target compound assignments now reuse the same hidden-local strategy so `x[i] op= y` lowers with single-evaluation behavior instead of recomputing the target and index.
- Strings, slices, channels, and maps are now first-class runtime values with dedicated bytecode instructions and builtin interoperability.

## Near-Term Extension Path

1. Keep broader panic payload fidelity staged; `recover()` now works for directly deferred user functions, but concrete Go runtime panic object types still need deliberate modeling.
2. Do not add channel `range` or comma-ok receive opportunistically; pair them with the blocking-model design they still need on top of the new multi-result path.
3. Keep package-backed services growing without collapsing into ad hoc dispatch tables spread across the VM.
4. Separate bytecode IR from runtime-specific instruction encoding if the VM grows significantly.
