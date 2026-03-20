# Plan: Named Result Parameters

## Basic Information

- Plan ID: `2026-03-20-11-23-19-named-result-parameters`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Support grouped named result declarations such as `func split() (head, tail string, ok bool)`.
- Support bare `return` for named-result functions without weakening the existing unnamed multi-result path.
- Keep result declarations source-visible in `dump-ast` while preserving the current explicit ordered result-slot model through semantic analysis and lowering.

## Scope

- Research and design the staged behavior against the Go spec and local Go 1.21.5 probes.
- Extend the frontend AST and parser to model explicit result declarations and reject mixed named/unnamed result lists.
- Extend semantic analysis so named results create zero-value result slots at function entry and bare returns read those slots with shadowing checks.
- Add focused parser, semantic, CLI execution, and diagnostic coverage plus a dedicated example program.
- Keep touched files under the repository line ceiling, splitting near-limit files where needed.

## Non-Goals

- `defer`, panic/recover, or broader Go control-flow semantics tied to named returns
- Methods, receivers, function values, or interface-backed call surfaces
- Tuple runtime values or arbitrary multi-result expressions outside the current staged model
- Broader bytecode metadata redesign beyond what this slice needs for readable inspection

## Phase Breakdown

1. Write research and open the new `M3` active plan for named result parameters.
2. Extend the frontend signature model for explicit result declarations and mixed-result diagnostics.
3. Add semantic result-slot initialization and bare-return shadowing checks while preserving current explicit return lowering.
4. Add focused examples, tests, CLI validation, and file-size governance fixes for touched near-limit files.
5. Update reports and roadmap state, archive the completed plan, and commit the full working tree.

## Acceptance Criteria

- `func f() (left, right string, ok bool)` parses, renders through `dump-ast`, passes semantic analysis, and executes through the VM.
- Bare `return` works for named-result functions and still fails for unnamed-result functions.
- Mixed named/unnamed result lists fail with a targeted parser diagnostic.
- Bare return under a shadowed named-result binding fails with a targeted semantic diagnostic.
- The repository remains under formatting and file-size constraints, and validation evidence is recorded.

## Risks

- Result declarations are more ambiguous than input parameters because identifiers may be either names or types; careless parser changes could regress existing unnamed-result parsing.
- Named results live at the boundary between signature metadata and local-variable scope, so slot allocation must stay ordered and explicit.
- Bare return shadowing checks can become ad hoc if they are bolted onto lowering instead of being modeled in semantic analysis.
