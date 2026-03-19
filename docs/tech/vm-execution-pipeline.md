# VM Execution Pipeline

## Purpose

Describe the concrete execution pipeline shipped in the bootstrap milestone, including its module boundaries and known extension seams.

## Module Flow

1. `src/cli.rs`
   - Parses subcommands and optional execution overrides.
   - Produces a typed `Command` enum instead of passing raw strings downstream.
2. `src/driver.rs`
   - Orchestrates file loading, lexing, parsing, compilation, and VM execution.
   - Normalizes error categories into user-facing CLI diagnostics.
3. `src/source.rs`
   - Encapsulates source loading and path retention.
4. `src/frontend/lexer.rs`
   - Converts source text into tokens.
   - Performs lightweight newline-to-semicolon insertion for the supported subset.
5. `src/frontend/parser.rs`
   - Builds a small AST for package/function/block/expression constructs.
6. `src/bytecode/compiler.rs`
   - Validates entrypoint selection.
   - Lowers the entry function into stack-machine bytecode.
7. `src/runtime/vm.rs`
   - Executes bytecode with local slots, operand stack, and buffered output.

## Current Data Boundaries

- Source boundary: `SourceFile`
- Frontend boundary: `Token` -> `SourceFileAst`
- Compilation boundary: `SourceFileAst` -> `Program`
- Runtime boundary: `Program` -> `ExecutionResult`

## Current Semantic Rules

- Execution requires the configured package and function to exist.
- Local variables must be declared before assignment or use.
- Only builtin calls are currently supported.
- `println` is the only builtin wired into the VM.
- Expressions used in value positions must lower to stack values.

## Near-Term Extension Path

1. Add semantic analysis between parser and bytecode compiler.
2. Add typed values beyond integers.
3. Add user-defined function calls and call frames.
4. Add control flow lowering.
5. Separate bytecode IR from runtime-specific instruction encoding if the VM grows significantly.
