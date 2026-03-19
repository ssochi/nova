# M2: Frontend and VM Expansion

- Status: `completed`
- Current Main Plan: `2026-03-19-23-57-06-for-loops-path-analysis`

## Goals

- Introduce a dedicated semantic analysis stage.
- Add user-defined functions, call frames, and control flow to the VM path.
- Improve diagnostics and coverage while preserving standard-library-only constraints.

## Completion Criteria

- The compiler supports multi-function programs with entrypoint dispatch.
- The VM can execute conditional and looping control flow.
- Semantic errors are reported before bytecode lowering for common cases.
- CLI validation covers both successful and failing multi-function programs.

## Task Breakdown

- Design the semantic analysis layer and symbol tables.
- Extend parsing and lowering for function calls and control flow.
- Add VM call frames and jump instructions.
- Strengthen CLI diagnostics and validation fixtures.

## Related Plans

- `2026-03-19-23-37-05-semantic-functions-branches`
- `2026-03-19-23-57-06-for-loops-path-analysis` (`completed`)

## Current Risks

- Loop control flow is now present, but the supported form is intentionally narrow compared with real Go.
- Diagnostics still stop at message strings without source spans or excerpts.
- Runtime values and builtins remain minimal, which now shifts the main project risk into milestone `M3`.

## Next-Round Recommendations

- Move to milestone `M3-standard-library-and-runtime-model`.
- Prioritize runtime value expansion and builtin contract centralization before backend-facing work.
