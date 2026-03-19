# M3: Standard Library and Runtime Model

- Status: `in_progress`
- Current Main Plan: none yet

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

- None yet

## Current Risks

- Runtime-surface growth can sprawl quickly if types, builtins, and imports are mixed into the same plan.
- Builtin additions can become hardcoded special cases unless their contracts stay centralized.
- Supporting more realistic Go programs will require careful staging so the VM remains understandable.

## Next-Round Recommendations

- Open the first `M3` plan around runtime data expansion and builtin contract centralization.
- Prioritize work that improves real-program viability without collapsing into premature standard library breadth.
