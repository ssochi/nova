# Plan: Slice Runtime Values and Layered Test Coverage

## Basic Information

- Plan ID: `2026-03-20-00-55-55-slice-runtime-testing`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Introduce the first composite runtime value with narrow, explicit `slice` support.
- Expand builtin coverage around composite values without collapsing builtin contracts into ad hoc runtime checks.
- Upgrade the test system from a single integration-heavy file into a clearer layered setup with reusable CLI helpers and focused unit coverage.

## Scope

- Frontend support for `[]T` types, slice literals such as `[]int{1, 2}`, and index expressions such as `values[0]`
- Semantic support for slice types, slice indexing, slice-aware equality restrictions, and builtin validation for `append` plus `len`
- Bytecode and VM support for slice literals, indexing, and builtin execution for `append`
- Test-system restructuring into reusable support helpers plus unit and CLI integration coverage
- Design, tech, verification, experience, roadmap, and handoff updates for the new runtime slice

## Non-Goals

- Full Go slicing syntax such as `values[1:3]`
- `make`, `copy`, `cap`, variadic `...`, or nil semantics
- Slice element assignment such as `values[0] = 1`
- Multi-file import graphs or broader standard-library expansion beyond the builtin surface needed for this slice

## Phase Breakdown

1. Open the next active `M3` plan and record the composite-value and test-system slice being introduced.
2. Extend the frontend and semantic layers for slice types, literals, builtin contracts, and index expressions.
3. Lower the new checked forms into bytecode and execute them in the VM with runtime diagnostics.
4. Restructure tests into layered coverage, add unit tests, and keep CLI evidence serial and reproducible.
5. Sync docs, reports, milestone state, and archive the plan when all acceptance criteria land.

## Acceptance Criteria

- `cargo test` passes with unit coverage for slice-oriented parsing, semantic validation, builtin validation, and runtime execution.
- `cargo run -- run` executes a source file that builds and reads a slice through `append`, `len`, and index expressions.
- `check` rejects at least one invalid slice usage such as a non-integer index or mismatched `append` element type.
- Slice support is documented as a narrow staged subset, with deferred Go behaviors called out explicitly.
- The active-plan, milestone, and handoff documents describe both the feature slice and the upgraded validation surface.

## Risks

- Converting the type system from scalar-only to recursive types can spread across semantic and bytecode layers if not kept narrow.
- Slice runtime behavior can become misleading if unsupported Go features such as `cap` or slicing syntax are not documented sharply.
- Test restructuring can create churn unless helper boundaries remain minimal and stable.
