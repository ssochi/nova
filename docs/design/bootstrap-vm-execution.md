# Bootstrap VM Execution Design

## Goal

Establish the first real end-to-end execution path for `nova-go`: a CLI command can read a `.go` file, lex it, parse a minimal Go subset, lower it into bytecode, and execute it on a Rust VM implemented with the standard library only.

## Constraints

- The implementation must use the Rust standard library only.
- The first milestone should prioritize a VM path over a native backend.
- The initial slice must be narrow enough to complete and validate in one round, but broad enough to prove the layered architecture.
- Configuration should be surfaced as explicit inputs instead of being scattered as literals.

## Current Scope

- CLI subcommands: `check`, `run`, `dump-tokens`, `dump-ast`, `dump-bytecode`
- Configurable entry package and entry function for execution-facing commands
- Frontend subset:
  - `package` declaration
  - parameterless `func` declarations
  - block statements
  - `var` declarations
  - assignment statements
  - `return`
  - integer literals, identifiers, `+ - * /`, parentheses
  - builtin call expressions for `println`
- Bytecode subset:
  - stack push/pop
  - local load/store
  - arithmetic ops
  - builtin invocation
  - return
- VM subset:
  - integer values
  - local slots
  - stack execution
  - stdout-like buffered output for CLI rendering

## Deferred Scope

- Full Go syntax, typing, scoping, and package loading
- User-defined function calls and call frames
- Control flow (`if`, `for`, `switch`, `range`, `defer`, `go`, `select`)
- Heap values, structs, slices, maps, interfaces, and channels
- Standard library emulation
- Native code generation backend

## Interfaces and Extension Hooks

- `src/cli.rs`: command surface and execution overrides
- `src/frontend/`: tokenization, AST, and parsing layers kept separate for future semantic analysis
- `src/bytecode/compiler.rs`: current lowering seam; future IR stages can sit between AST and bytecode
- `src/runtime/vm.rs`: isolated interpreter runtime; later call frames and heap state can extend here without reshaping CLI or parsing
- `docs/tech/vm-execution-pipeline.md`: technical reference for the current execution pipeline
