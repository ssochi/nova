# M3: Standard Library and Runtime Model

- Status: `in_progress`
- Current Main Plan: none currently

## Goals

- Expand the runtime value model beyond integers and booleans.
- Increase builtin coverage and start building stable seams for standard library support.
- Keep the VM-first execution path while preparing for broader Go program compatibility.

## Completion Criteria

- The semantic and runtime layers support at least one new non-scalar runtime category or interoperability seam needed for future standard library work.
- Builtin coverage expands beyond `println` in a way that remains layered and testable.
- The CLI validation path covers richer runtime behavior than the current arithmetic / branch / loop subset.
- Technical and roadmap documents describe the runtime model clearly enough for backend-bridge work to build on later.

## Task Breakdown

- Define the next runtime value and builtin slice with explicit non-goals.
- Extend semantic analysis, lowering, and VM execution for the chosen runtime model changes.
- Add CLI-first validation, diagnostics, and documentation for the richer runtime surface.
- Keep standard-library-oriented work decomposed into resumable plans instead of one large jump.

## Related Plans

- `2026-03-20-02-39-55-make-slice-allocation`: completed builtin `make([]T, len[, cap])` and the first type-argument builtin path
- `2026-03-20-00-09-59-string-runtime-builtins`: completed first richer runtime value slice with builtin contract centralization
- `2026-03-20-00-35-11-import-fmt-seam`: completed the first narrow import and package-backed standard-library seam
- `2026-03-20-00-55-55-slice-runtime-testing`: completed the first composite runtime value and layered test coverage
- `2026-03-20-01-17-51-strings-package-contracts`: completed typed package contracts, a second standard-library seam, and the research baseline
- `2026-03-20-01-33-44-slice-expressions-assignment`: completed slice windows, shared slice storage, and indexed slice assignment
- `2026-03-20-01-54-16-slice-builtins-capacity`: completed `cap`, `copy`, and capacity-aware `append`
- `2026-03-20-02-18-45-typed-var-zero-values`: completed explicit typed `var` declarations and nil-slice zero-value runtime behavior

## Current Risks

- Runtime-surface growth can sprawl quickly if types, builtins, and imports are mixed into the same plan.
- Builtin additions can become hardcoded special cases unless their contracts stay centralized.
- Supporting more realistic Go programs will require careful staging so the VM remains understandable.
- The current runtime now has richer slice operations including `make`, but string slicing, fuller append growth semantics, real import graphs, and broader package-backed runtime services remain open.

## Next-Round Recommendations

- Open the next `M3` plan around byte-oriented string work and string slicing, or broaden allocation/runtime coverage beyond slices once the next compatibility gap is chosen.
- Reuse the `docs/research/` flow before locking the next compatibility-sensitive slice.
