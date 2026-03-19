# Project Architecture

## Purpose

Define the current top-level architecture of `nova-go` so future rounds can extend the compiler without collapsing module boundaries.

## Current Architecture Layers

1. CLI layer
   - `src/main.rs`
   - `src/cli.rs`
   - Responsibility: accept commands, parse options, and expose a stable user entrypoint.
2. Driver layer
   - `src/driver.rs`
   - Responsibility: orchestrate source loading, frontend, lowering, runtime execution, and error normalization.
3. Frontend layer
   - `src/frontend/token.rs`
   - `src/frontend/lexer.rs`
   - `src/frontend/ast.rs`
   - `src/frontend/parser.rs`
   - Responsibility: source text to structured syntax.
4. Semantic layer
   - `src/semantic/model.rs`
   - `src/semantic/analyzer.rs`
   - Responsibility: symbol resolution, scope validation, entrypoint validation, and typed checked-program construction.
5. Lowering layer
   - `src/bytecode/compiler.rs`
   - `src/bytecode/instruction.rs`
   - Responsibility: transform checked programs into VM-ready bytecode.
6. Runtime layer
   - `src/runtime/value.rs`
   - `src/runtime/vm.rs`
   - Responsibility: execute bytecode programs.
7. Shared support layer
   - `src/config.rs`
   - `src/source.rs`
   - Responsibility: configuration and source loading primitives used across layers.

## Architectural Rules

- The CLI layer may depend on the driver layer, but not on frontend/runtime internals directly.
- Frontend modules must not depend on bytecode or runtime modules.
- The semantic layer may depend on frontend structures, but not on runtime execution details.
- Bytecode lowering must consume semantic output rather than re-resolving source names from raw AST.
- Runtime modules must not depend on parser internals.
- Configuration is centralized in `src/config.rs` instead of being duplicated across commands.
- Execution-facing validation should prefer the real CLI path, with direct library tests only as a supplement.

## Current Execution Surface

- `check`: lex + parse + package-level semantic validation
- `dump-tokens`: token inspection
- `dump-ast`: AST inspection
- `dump-bytecode`: bytecode inspection
- `run`: bytecode execution on the VM

## Near-Term Evolution

- Expand the runtime and semantic surface for richer values, builtins, and standard-library-oriented execution.
- Expand the VM with more value categories and runtime data structures.
- Keep backend work behind a later stable IR boundary instead of coupling it to the current VM instruction set.

## Related Documents

- `docs/design/bootstrap-vm-execution.md`
- `docs/design/semantic-functions-branches.md`
- `docs/design/for-loop-control-flow.md`
- `docs/tech/semantic-analysis.md`
- `docs/tech/vm-execution-pipeline.md`
- `docs/roadmap/milestones/M2-frontend-expansion.md`
