# Context: Switch Statements

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, latest archived plan context, milestone index, active plan index, `todo.md`, and the latest verification and experience reports tied to the archived `if`-header plan.
3. Confirmed milestone `M3-standard-library-and-runtime-model` is still `in_progress` and that no active plan existed, so opening a new plan was the highest-priority next action.
4. Reviewed the current AST, parser, semantic analyzer, checked model, bytecode compiler, runtime pipeline docs, and test layout to choose the next substantial `M3` slice.
5. Chose staged `switch` statements as the next plan because they materially improve realistic Go control-flow coverage, while standalone `chan` work would still be bottlenecked by the absence of `go` and `select`.
6. Opened this active plan under `M3` and updated roadmap state to point at it.
7. Wrote a research note for official expression-switch behavior, including shared header scope, tagless `switch`, single-expression evaluation, duplicate `default`, and staged duplicate literal-case diagnostics.
8. Generalized the old `if`-only header abstraction into shared `HeaderStatement` / `CheckedHeaderStatement` structures so both `if` and `switch` use one control-flow header model.
9. Added lexer keywords, explicit AST and checked-model `switch` nodes, and extracted parser statement handling into `src/frontend/parser/statements.rs` so parser growth stayed below the repository file-size limit.
10. Implemented dedicated semantic switch analysis with shared header scopes, clause-local scopes, duplicate `default` rejection, duplicate scalar literal-case diagnostics, and switch-aware return-path analysis.
11. Lowered staged `switch` statements into explicit bytecode jumps, including hidden `switch$tag*` locals so expression-switch tags are evaluated once and remain visible through `dump-bytecode`.
12. Added `examples/switch_statements.go` plus parser, semantic, CLI execution, CLI token, CLI AST, CLI bytecode, and CLI diagnostic coverage for expression and tagless switch paths.
13. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and two `check` failure paths, then checked modified file sizes.
14. Wrote verification and experience reports, updated design / tech docs plus `BOOT.md`, and prepared the plan for archive.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; staged expression `switch` and tagless `switch` now work, while `break`, `continue`, `fallthrough`, richer `for` syntax, channels, and wider runtime/package work remain open.

## Key Information for the Next Trigger

- The shared control-flow header abstraction now lives in `HeaderStatement` / `CheckedHeaderStatement`; reuse it for future `for`-header or related statement work instead of reintroducing `if`- or `switch`-specific header enums.
- Expression-switch lowering currently evaluates the tag once into visible `switch$tag*` locals. Preserve that property if later `break`, `fallthrough`, or type-switch work changes clause dispatch.
- Parser statement extraction is now the pattern for keeping `src/frontend/parser.rs` small; continue splitting helper modules when new control-flow forms threaten the line-count ceiling.
