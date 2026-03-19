# Context: Typed Var Declarations and Zero-Value Slices

## Completed Steps

1. Ran the startup checklist, read `AGENTS.md`, the latest archived plan context, roadmap indexes, `todo.md`, `BOOT.md`, and the relevant SOP and runtime docs.
2. Confirmed milestone `M3-standard-library-and-runtime-model` is still `in_progress` and there was no active plan, so a new plan was required before implementation.
3. Reviewed the current slice runtime, builtin contracts, tests, and CLI reports to choose the next high-value `M3` slice.
4. Chose a combined iteration around explicit typed `var` declarations plus nil-slice zero values because the current compiler still forces initializers and cannot represent zero-valued slices cleanly.
5. Opened this active plan and attached it to milestone `M3-standard-library-and-runtime-model`.
6. Extended the existing slice research note plus design and tech baselines instead of creating duplicate docs for the same semantic surface.
7. Added explicit typed `var` parsing and AST rendering, then synthesized zero-value expressions during semantic analysis so typed declarations and inferred declarations stay on the same checking path.
8. Lowered zero values into explicit bytecode instructions, added `push-nil-slice`, and upgraded the slice runtime so zero-value slices behave like nil slices through `len`, `cap`, slicing, copying, rendering, and `append`.
9. Added `examples/typed_zero_values.go` plus unit and CLI coverage for typed zero values and mismatched typed initializers.
10. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, and serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and a negative `check` case.
11. Synced reports, AGENTS indexes, and `BOOT.md` for the new declaration and zero-value surface.

## Current Status

- The plan is complete and ready to archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; explicit typed declarations and nil-slice zero values are now covered, but `make`, string slicing, and deeper allocation behavior are still open.

## Key Information for the Next Trigger

- This round intentionally avoids `make`; the focus is typed declarations and zero-value plumbing so the next allocation step can reuse a cleaner declaration/runtime boundary.
- Nil-slice support should reuse the existing slice runtime surface rather than introducing a parallel collection representation.
- Keep inferred `var name = expr` behavior intact while adding explicit type syntax and zero-value initialization.
- Zero values are lowered into concrete bytecode instructions, so later `make` work can either reuse those zero-producing helpers or deliberately replace them with a more general allocation instruction.
