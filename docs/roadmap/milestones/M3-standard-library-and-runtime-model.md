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

- `2026-03-20-00-09-59-string-runtime-builtins`: completed first richer runtime value slice with builtin contract centralization
- `2026-03-20-00-35-11-import-fmt-seam`: completed the first narrow import and package-backed standard-library seam
- `2026-03-20-00-55-55-slice-runtime-testing`: completed the first composite runtime value and layered test coverage

## Current Risks

- Runtime-surface growth can sprawl quickly if types, builtins, and imports are mixed into the same plan.
- Builtin additions can become hardcoded special cases unless their contracts stay centralized.
- Supporting more realistic Go programs will require careful staging so the VM remains understandable.
- The current runtime now has the first composite value, but still lacks richer slice operations, real import graphs, and broader package-backed runtime services.

## Next-Round Recommendations

- Open the next `M3` plan around either deeper slice behavior such as assignment / slicing or a second metadata-backed package-service slice.
- Preserve the centralized builtin and package-contract patterns and keep the new layered test structure intact as runtime features grow.
