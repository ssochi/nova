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

- Package analysis validates duplicate function names, variable scopes, call arity, and return compatibility.
- Package analysis also validates supported import paths and selector calls to imported package members.
- Execution additionally requires the configured package and entry function to exist, and the entry function must be `func main()`.
- Local variables must be declared before assignment or use, with nested block scopes mapped to fixed slots during analysis.
- Builtin calls, user-defined function calls, and metadata-backed `fmt` / `strings` package seams are supported.
- Current builtin coverage includes `print`, `println`, `len`, `cap`, `append`, `copy`, `delete`, and typed `make` handling.
- Current imported package coverage is `fmt.Print`, `fmt.Println`, `fmt.Sprint`, `strings.Join`, `strings.Contains`, `strings.HasPrefix`, and `strings.Repeat`.
- Branch and loop conditions must produce boolean values, and staged `if` headers now run their initializer instructions before the condition jump sequence.
- The current branch model supports `if`, `else`, and explicit `else if` lowering with header scopes chosen during semantic analysis.
- Condition-only `for` loops and staged `range` loops over `slice` / `map` are lowered into the existing jump and helper instruction set.
- Strings, slices, and maps are now first-class runtime values with dedicated bytecode instructions and builtin interoperability.

## Near-Term Extension Path

1. Add the next runtime category or service seam, most likely `chan` support or broader package-backed runtime helpers.
2. Expand control-flow ergonomics beyond the current staged `if` headers, such as `switch`, `break`, `continue`, and richer `for` syntax.
3. Keep package-backed services growing without collapsing into ad hoc dispatch tables spread across the VM.
4. Separate bytecode IR from runtime-specific instruction encoding if the VM grows significantly.
