# M2: Frontend and VM Expansion

- Status: `in_progress`
- Current Main Plan: none yet

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

## Current Risks

- Scope can expand quickly if type-system work is mixed into the first control-flow push.
- Parser growth without semantic layering would increase coupling.
- The remaining control-flow gap is now concentrated in loops; delaying it too long would leave return-path analysis incomplete.

## Next-Round Recommendations

- Open the next `M2` plan around looping control flow and semantic path analysis.
- Tighten diagnostics and bytecode inspection ergonomics once `for` support exists.
