# Context: String Runtime and Builtin Contracts

## Completed Steps

1. Ran the startup checklist and confirmed milestone `M3-standard-library-and-runtime-model` is `in_progress` with no active plan attached.
2. Read the latest archived context, milestone state, active plan index, `todo.md`, startup SOP, architecture docs, and the current VM execution notes.
3. Inspected the frontend, semantic, bytecode, runtime, tests, and examples to find the first worthwhile `M3` slice.
4. Chose a combined runtime and tooling plan: add string literals, centralize builtin contracts, and validate the richer CLI surface.
5. Opened this active plan and attached it to milestone `M3-standard-library-and-runtime-model`.
6. Added string literals across tokens, AST rendering, parser expressions, semantic types, checked expressions, bytecode lowering, and VM execution.
7. Introduced shared builtin identifiers plus a centralized semantic builtin contract table for `print`, `println`, and `len`.
8. Added `examples/strings.go`, expanded CLI integration coverage, ran `cargo test`, and exercised the real CLI through `run`, `dump-tokens`, `dump-ast`, `dump-bytecode`, and `check`.
9. Updated design, tech, verification, experience, milestone, and boot documents for the first `M3` runtime slice.

## Current Status

- This plan is complete and ready to archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; only the first runtime-value slice is finished.

## Key Information for the Next Trigger

- Builtin identity now lives in `src/builtin.rs`, while call contracts live in `src/semantic/builtins.rs`; keep future builtin expansion flowing through those two seams.
- VM output is now an accumulated string buffer, which future formatting or package-backed IO work should preserve intentionally.
- The next `M3` plan should add either a composite runtime value or an import / standard-library seam; strings alone are not enough for realistic Go packages.
