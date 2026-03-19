# Context: Import Declarations and Fmt Package Seam

## Completed Steps

1. Ran the startup checklist and confirmed milestone `M3-standard-library-and-runtime-model` is still `in_progress`.
2. Read the latest archived context, roadmap indexes, `todo.md`, startup SOP, and the current runtime / builtin docs.
3. Reviewed the frontend, semantic, bytecode, runtime, and CLI tests to find the next worthwhile `M3` slice.
4. Chose a narrow standard-library seam plan: top-level imports plus centralized `fmt` package-function contracts.
5. Opened this active plan and attached it to milestone `M3-standard-library-and-runtime-model`.
6. Added `import` tokens, import declarations, selector-call parsing, and AST rendering for imported package usage.
7. Introduced shared package and package-function identities plus centralized semantic contracts for the first metadata-backed `fmt` seam.
8. Extended lowering and VM execution with `call-package`, then added `examples/imports_fmt.go` and automated CLI coverage for imported package calls.
9. Installed `rustfmt`, ran `cargo fmt`, ran `cargo test`, and exercised the real CLI through `dump-tokens`, `dump-ast`, `dump-bytecode`, `check`, and `run`.
10. Updated design, tech, verification, experience, milestone, and boot documents for the import and package-contract slice.

## Current Status

- This plan is complete and ready to archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; the project now has the first metadata-backed package seam but still lacks composite values and broader package coverage.

## Key Information for the Next Trigger

- Keep package-function contracts separate from builtin contracts so both extension paths remain clean.
- Favor the current selector-call model unless a later plan truly needs package values or richer selector semantics.
- Preserve `check` as package-level validation; imported package support must not accidentally depend on `main.main`.
- The next `M3` plan should either add a composite runtime value such as slices or extend metadata-backed package services beyond `fmt` without jumping to filesystem import resolution.
