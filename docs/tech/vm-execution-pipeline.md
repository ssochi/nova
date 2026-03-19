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
   - Builds a small AST for package/function/block/expression constructs.
6. `src/semantic/analyzer.rs`
   - Resolves names, validates function signatures and branches, and produces a checked program.
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
- Execution additionally requires the configured package and entry function to exist, and the entry function must be `func main()`.
- Local variables must be declared before assignment or use, with nested block scopes mapped to fixed slots during analysis.
- Builtin calls and user-defined function calls are both supported.
- `println` is still the only builtin wired into the VM.
- Branch conditions must produce boolean values.

## Near-Term Extension Path

1. Add looping control flow and path analysis for `for`.
2. Add richer values and runtime data structures beyond integers and booleans.
3. Expand builtin coverage and lay runtime groundwork for standard library support.
4. Separate bytecode IR from runtime-specific instruction encoding if the VM grows significantly.
