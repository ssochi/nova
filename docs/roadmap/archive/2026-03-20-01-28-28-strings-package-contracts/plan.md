# Plan: Strings Package Contracts and Research Baseline

## Basic Information

- Plan ID: `2026-03-20-01-17-51-strings-package-contracts`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Add a second metadata-backed standard-library seam by introducing a narrow `strings` package surface.
- Strengthen the package-contract layer so imported package functions can describe typed fixed-arity calls instead of only variadic any-value output helpers.
- Create a reusable research directory and capture the official behavior baseline that guides this package slice and later package expansion.

## Scope

- Add `import "strings"` support with a first small function set chosen from existing runtime capabilities.
- Extend shared package identity, semantic package contracts, and VM package dispatch for typed `strings` helpers.
- Add example programs plus layered unit, integration, and serial CLI validation for the new package seam.
- Create `docs/research/` with directory conventions and a research note for the selected `strings` behaviors.
- Sync roadmap, design, tech, verification, experience, and boot documents for the new package-contract slice.

## Non-Goals

- Filesystem-backed import graphs or multi-file package loading
- General selector expressions beyond imported package call targets
- Full Go `strings` package coverage
- `nil`, `panic`, and allocation semantics that require a broader runtime model

## Phase Breakdown

1. Open the next active `M3` plan and record the package-contract plus research slice.
2. Capture official-behavior research for the selected `strings` functions and create the new research directory baseline.
3. Extend package identities, semantic validation, and VM execution for the chosen `strings` functions.
4. Add examples, tests, and serial CLI validation for happy-path and error-path package usage.
5. Sync docs, update milestone and plan state, and archive the plan if all acceptance criteria land.

## Acceptance Criteria

- `cargo test` passes with unit and CLI coverage for the new `strings` package seam.
- `cargo run -- run` executes at least one realistic program that imports `strings` and uses multiple supported functions.
- `check` rejects at least one invalid `strings` package call with a typed package-contract diagnostic.
- `docs/research/` exists with an indexed research note that later agents can reuse.
- Roadmap, design, tech, verification, experience, and boot artifacts describe both the feature slice and the new research workflow.

## Risks

- The package-contract layer can become brittle if typed and variadic rules are added ad hoc instead of through shared metadata.
- `strings` behavior in real Go includes panic and edge semantics that the current VM cannot fully reproduce; unsupported parts must stay explicit.
- Creating a research directory without indexing discipline would add noise instead of handoff value.
