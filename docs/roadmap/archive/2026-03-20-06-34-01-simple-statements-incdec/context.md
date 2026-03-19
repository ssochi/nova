# Context: Simple Statements, Short Declarations, and Inc/Dec

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, latest archived plan context, milestone index, active plan index, `todo.md`, the startup SOP, and the newest verification / experience reports tied to the archived `for` / loop-control plan.
3. Confirmed milestone `M3-standard-library-and-runtime-model` remains `in_progress` and that no active plan existed, so opening a new `M3` plan was the highest-priority next action.
4. Reviewed the current control-flow design, semantic simple-statement seams, bytecode lowering hooks, and parser structure to choose the next substantial `M3` slice.
5. Chose staged short declarations plus explicit `++` / `--` because they unlock idiomatic `if`, `switch`, and classic `for` usage with less implementation risk than channel work that still lacks `go` and `select`.
6. Created this active plan, attached it to milestone `M3`, and recorded the initial execution boundary for the round.
7. Added lexer tokens, explicit AST / checked-model nodes, parser support, semantic analysis, bytecode lowering, runtime arithmetic support, and tests for single-expression short declarations plus explicit inc/dec statements.
8. Added `examples/simple_statements.go` and CLI coverage for ordinary short declarations, `if` / `switch` headers, classic `for i := 0; ...; i++`, local `--`, and map-index `++`.
9. Split `src/bytecode/compiler.rs` by moving simple-statement lowering into `src/bytecode/compiler/simple_statements.rs` after the new feature pushed the original file over the 1000-line repository limit.
10. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and three `check` failure paths, then checked modified file sizes.
11. Wrote verification and experience reports, updated design / tech docs plus `BOOT.md`, and prepared the plan for archive.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; staged single-expression short declarations and explicit `++` / `--` now work, while labels, compound assignments, channels, and broader runtime / package work remain open.

## Key Information for the Next Trigger

- Keep short declarations explicit in the AST and checked model; do not hide them inside assignment statements because even the staged single-binding form has different scope rules from `=`.
- Keep `++` / `--` statement-only all the way through parsing and lowering so later expression work does not inherit a misleading model.
- Preserve the split `src/bytecode/compiler/simple_statements.rs` seam if later work adds compound assignments or broader simple statements; this round created that module specifically to hold the file-size ceiling after inc/dec lowering.
