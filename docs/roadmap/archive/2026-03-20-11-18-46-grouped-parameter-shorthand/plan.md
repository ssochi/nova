# Plan: Grouped Parameter Shorthand

## Basic Information

- Plan ID: `2026-03-20-11-02-46-grouped-parameter-shorthand`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Support Go-style grouped parameter-name shorthand such as `func pair(a, b int)`.
- Keep grouped parameter declarations explicit enough in the frontend so `dump-ast` remains readable.
- Reuse the existing function-signature, variadic, and semantic registry pipeline instead of adding ad hoc parser-only rewrites.

## Scope

- Research the official grouped-parameter declaration behavior and the interaction with staged variadic parameters.
- Extend the frontend AST and parser to preserve grouped parameter declarations.
- Flatten grouped parameters into the existing semantic function-signature model for checking, lowering, and runtime execution.
- Add focused parser, semantic, CLI execution, and diagnostic coverage plus a real example program.
- Update roadmap, design, tech, and report artifacts for the shipped slice.

## Non-Goals

- Named result parameters or naked `return`
- Grouped result declarations such as `(a, b int)`
- Interface-backed variadic package forwarding or any broader function-type syntax work
- Real package import graphs, methods, or receiver syntax

## Phase Breakdown

1. Open the active plan and record the grouped-parameter shorthand scope.
2. Write research and design notes that pin the staged behavior to official Go rules.
3. Implement AST, parser, and semantic changes while preserving the current lowering/runtime contract.
4. Add focused tests, CLI examples, and validation commands.
5. Update reports and roadmap state, then archive the completed plan.

## Acceptance Criteria

- `func f(a, b int)` parses, renders through `dump-ast`, passes semantic analysis, and executes through the VM.
- Grouped declarations interact correctly with a final variadic parameter, such as `func collect(prefix, suffix string, values ...int)`.
- Duplicate parameter names across grouped declarations still fail with the existing semantic diagnostic style.
- Invalid grouped syntax fails early with a targeted parser diagnostic.
- The repository remains under formatting and file-size constraints, and validation evidence is recorded.

## Risks

- Flattening grouped parameters too early could make `dump-ast` regress into a less source-faithful form.
- Variadic parsing and grouped-name parsing share the same signature surface, so careless changes could weaken the final-parameter rule.
- Function-local parameter slot allocation depends on the current flattened parameter ordering; the implementation must preserve declaration order exactly.
