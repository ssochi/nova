# Context: Map Comma-Ok Lookups and Literal Diagnostics

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, latest archived plan context, milestone index, active plan index, `todo.md`, and the latest verification and experience reports tied to the archived `range` plan.
3. Confirmed milestone `M3-standard-library-and-runtime-model` is still `in_progress` and that no active plan existed, so a new plan was the highest-priority next action.
4. Reviewed the current parser, semantic analyzer, bytecode compiler, runtime value model, VM implementation, and map-related design / tech / research docs.
5. Chose staged comma-ok `map` lookups plus duplicate constant-key diagnostics as the next plan because `map` reads are still incomplete for realistic Go code even after the new `range` support.
6. Verified the intended Go behavior baseline with the official specification and local Go 1.21.5 spot checks for nil-map comma-ok reads, short redeclaration rules, and duplicate constant-key diagnostics.
7. Opened this plan under `M3` and updated the roadmap state to point at it.
8. Extended the frontend AST and parser with explicit comma-ok lookup statements while keeping the right-hand side constrained to `map` index syntax.
9. Added semantic analysis for staged comma-ok lookups, same-block short-declaration freshness, and duplicate constant literal-key diagnostics for map literals.
10. Lowered comma-ok lookups into a dedicated `lookup-map` bytecode instruction, added VM execution support, and extracted reusable helpers into `src/runtime/vm/support.rs` so `src/runtime/vm.rs` stayed below the repository file-size limit.
11. Added `examples/map_lookup.go` plus parser, semantic, VM, CLI execution, and CLI diagnostic coverage for the new lookup path and duplicate-key failures.
12. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, and serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and three `check` failure paths.
13. Wrote verification and experience reports, updated design / tech docs plus `BOOT.md`, and prepared the plan for archive.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; staged comma-ok `map` lookups and duplicate constant literal-key diagnostics now work, while broader tuple expressions, `if` initializers, channels, and wider control-flow work remain open.

## Key Information for the Next Trigger

- Keep comma-ok lookup explicit and statement-scoped instead of widening the whole language into generic tuple expressions prematurely.
- Same-block `:=` freshness for comma-ok lookups is now enforced centrally; blank identifiers do not count as new variables, and names from outer scopes still shadow through the current block scope rather than being reused automatically.
- Duplicate constant-key diagnostics currently cover the scalar literal-key forms modeled directly in the AST; if later work adds richer constant expressions, extend the research and semantic layer deliberately instead of silently widening detection.
