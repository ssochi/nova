# Plan: Type Switches and Comma-Ok Assertions

## Basic Information

- Plan ID: `2026-03-20-14-17-12-type-switches-and-comma-ok-assertions`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add explicit comma-ok type-assertion statement forms such as `value, ok := boxed.(T)` and `value, ok = boxed.(T)`.
- Add the first staged type-switch surface for empty-interface operands with explicit guard syntax and clause typing.
- Keep interface-consumption behavior explicit through AST, checked, bytecode, and VM seams without introducing tuple runtime values or hiding type checks inside generic calls.

## Scope

- Research the real-Go baseline for comma-ok assertions and type switches, then record the staged implementation boundary.
- Add dedicated AST and parser forms for comma-ok assertion statements and type switches, including optional control-flow headers for type switches.
- Extend semantic analysis with explicit checked nodes, interface-only validation, clause-local binding typing, and duplicate-case diagnostics for the staged supported type list.
- Extend bytecode/runtime execution with explicit comma-ok assertion and type-switch support while keeping `dump-ast` and `dump-bytecode` readable.
- Add focused examples, unit tests, CLI happy-path coverage, CLI diagnostic coverage, validation records, and plan/context synchronization.

## Non-Goals

- Non-empty interfaces, methods, method sets, or interface implementation checks beyond the empty interface.
- Type-switch `fallthrough`, labels, or control-flow work outside the existing staged `switch` subset.
- General tuple values, first-class multi-result expressions, or support for arbitrary multi-value use sites beyond the dedicated assertion/type-switch forms.
- Broader runtime categories not already modeled in the current VM.

## Phase Breakdown

1. Research and plan refresh
   - Record local Go findings for comma-ok assertions, type-switch guard forms, nil behavior, clause binding typing, and duplicate-case diagnostics.
   - Add a dedicated design note for the staged interface-consumption slice.
2. Frontend and semantic surface
   - Parse explicit comma-ok assertion statements and type switches.
   - Add checked statement models, guard/case validation, and clause-local binding typing.
3. Bytecode and runtime
   - Add explicit lowering and VM execution for non-panicking type assertions and type-switch matching.
   - Reuse the runtime interface seam so interface type checks stay centralized.
4. Validation and synchronization
   - Add focused tests, serial CLI evidence, touched-file line-count checks, and sync the roadmap/docs/report indices.

## Acceptance Criteria

- `dump-ast` renders comma-ok assertion statements and type switches explicitly instead of hiding them inside generic assignment or expression-switch forms.
- `check` accepts staged comma-ok assertions and type switches on `any` / `interface{}` operands, while rejecting non-interface guards and unsupported case types.
- `run` returns zero-value-plus-`false` behavior for failed comma-ok assertions and dispatches type-switch clauses with real-Go-compatible nil vs typed-nil behavior for the staged empty-interface surface.
- `dump-bytecode` keeps the new assertion/type-switch operations explicit enough to debug matching without reading the compiler implementation.
- All touched files stay within the repository line-count limit.

## Risks

- Parsing `switch [stmt;] x.(type)` must not weaken the existing expression-switch or single-result type-assertion syntax.
- Type-switch clause binding types vary by clause shape, so semantic scope handling must stay explicit to avoid leaking the wrong binding type.
- `src/frontend/ast.rs`, `src/frontend/parser/statements.rs`, `src/semantic/model.rs`, `src/bytecode/compiler.rs`, and runtime VM files are already dense, so helper extraction may be required in the same round.
