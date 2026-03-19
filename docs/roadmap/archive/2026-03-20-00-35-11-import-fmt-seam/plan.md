# Plan: Import Declarations and Fmt Package Seam

## Basic Information

- Plan ID: `2026-03-20-00-35-11-import-fmt-seam`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Add the first package-backed standard-library seam by supporting `import` declarations and `fmt` selector calls.
- Keep builtin contracts and package function contracts separated so future standard-library work stays layered.
- Expand the CLI-visible program surface from builtin-only programs to imported-package programs that still run through the VM.

## Scope

- Frontend support for top-level `import "path"` declarations and selector-call syntax like `fmt.Println(...)`
- Semantic validation for imported packages, package member lookup, and package-function call contracts
- Bytecode and VM support for package-function dispatch, including at least one value-returning `fmt` call
- Examples, tests, CLI validation, and synced design / tech / roadmap documents for the new seam

## Non-Goals

- Multi-file package graphs or filesystem-based import resolution
- Import aliases, grouped import blocks, dot imports, or blank imports
- Full Go `fmt` formatting verbs or exact output compatibility
- Composite runtime values such as slices, maps, structs, or interfaces

## Phase Breakdown

1. Open the next active `M3` plan and define the narrow import / package seam being introduced.
2. Extend the frontend AST, lexer, and parser for import declarations plus selector-call expressions.
3. Introduce centralized package-function contracts for `fmt`, then wire semantic validation, lowering, and runtime dispatch through them.
4. Add imported-package examples, automated coverage, and serial CLI validation traces.
5. Sync roadmap, design, tech, verification, and handoff context; archive the plan if all acceptance criteria land.

## Acceptance Criteria

- `cargo test` passes with coverage for imported `fmt` calls and selector-call diagnostics.
- `cargo run -- run` can execute a source file that imports `fmt` and exercises at least one package function returning a value.
- `check` rejects at least one invalid imported-package usage, such as a missing import or unsupported member.
- Package function contracts live in a centralized seam rather than ad hoc string matching across semantic and runtime layers.
- Milestone and technical documents clearly describe the new import / package boundary and the remaining `M3` gaps.

## Risks

- Selector-call parsing can distort the current small expression grammar if call syntax becomes too generic too early.
- Import support can sprawl into package loading unless the plan stays limited to metadata-backed packages.
- `fmt` behavior can become misleading if simplified runtime behavior is not documented explicitly.
