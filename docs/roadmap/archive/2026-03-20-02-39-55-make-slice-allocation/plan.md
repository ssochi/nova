# Plan: Make-Based Slice Allocation

## Basic Information

- Plan ID: `2026-03-20-02-39-55-make-slice-allocation`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Add builtin `make([]T, len[, cap])` for slices so programs can allocate non-zero-length or spare-capacity slices without relying on literals or repeated `append`.
- Introduce a layered representation for builtin calls that need type arguments, starting with `make`, without scattering parser or semantic hacks.
- Keep CLI debug surfaces useful so `dump-ast` and `dump-bytecode` expose the allocation path clearly.

## Scope

- Extend the existing slice research baseline with official `make` semantics for slice length, optional capacity, and runtime-failure cases.
- Add AST and semantic support for `make` with a first-class slice type argument plus integer length / capacity expressions.
- Lower `make([]T, len[, cap])` into explicit bytecode and execute it in the VM with zero-filled slice allocation.
- Support both `len` and `cap` observations plus indexed assignment on `make`-allocated slices.
- Add examples, unit tests, CLI integration coverage, validation reports, and roadmap/doc synchronization.

## Non-Goals

- `make` support for maps, channels, or other non-slice runtime categories
- Untyped `nil`, conversions, or general type-valued expressions outside builtin `make`
- Full Go panic formatting; runtime failures may still use project-specific error text
- Byte-oriented string slicing or `[]byte` interoperability

## Phase Breakdown

1. Open the active `M3` plan and extend the existing slice research / design baseline for `make`.
2. Implement builtin `make` across parser, semantic analysis, checked-model lowering, bytecode, and VM execution.
3. Add example programs plus focused unit and CLI coverage for positive and negative allocation paths.
4. Run formatting and serial CLI validation, then sync reports, roadmap docs, `BOOT.md`, archive the plan if complete, and commit plus push the full working tree.

## Acceptance Criteria

- `cargo test` passes with new coverage for `make([]T, len[, cap])` happy paths and diagnostics.
- `cargo run -- run` executes a realistic example using `make`-allocated slices and prints observable `len` / `cap` behavior.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` expose the `make` allocation path clearly enough for debugging.
- `cargo run -- check` rejects at least one invalid `make` invocation with a targeted diagnostic.
- Research, design, tech, verification, experience, roadmap, and `BOOT.md` all reflect the new allocation surface and its current limits.

## Risks

- `make` is the first builtin with a type argument, so parser and semantic changes can sprawl unless the special form is kept narrowly modeled.
- Runtime allocation must preserve existing slice invariants, especially `len <= cap`, zero-filled elements, and compatibility with existing slice builtins.
- A partial `make` implementation can mislead future work if map / channel semantics are implied instead of explicitly deferred.
