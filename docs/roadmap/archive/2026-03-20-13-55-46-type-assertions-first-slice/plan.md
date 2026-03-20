# Plan: Type Assertions First Slice

## Basic Information

- Plan ID: `2026-03-20-13-55-46-type-assertions-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add the first staged `x.(T)` type-assertion slice for empty-interface values.
- Make boxed `any` payloads usable as concrete runtime values through explicit AST, semantic, bytecode, and VM paths.
- Keep CLI inspection, validation, and file-size governance intact while extending the interface runtime seam.

## Scope

- Extend the existing empty-interface research baseline with real-Go type-assertion rules needed for the first slice.
- Add postfix type-assertion syntax to the parser and rendering pipeline with a dedicated expression node.
- Add semantic checking for interface-only operands, explicit destination type resolution, and checked assertion nodes.
- Add explicit bytecode and VM execution for successful unboxing and interface-conversion panic failures.
- Add focused unit tests, CLI happy-path tests, CLI diagnostic tests, an example program, and documentation updates for the new assertion surface.

## Non-Goals

- Comma-ok type assertions, type switches, non-empty interfaces, methods, or dynamic dispatch.
- Exact Go runtime type-name spelling for every mismatch panic when the staged runtime type renderer differs from Go internals.
- Broader reflection-like interface services or package APIs that depend on richer interface behavior.

## Phase Breakdown

1. Research and plan refresh
   - Extend the existing empty-interface note with type-assertion probes and record the staged scope boundary.
   - Add a dedicated design note for the first assertion slice.
2. Frontend and semantic surface
   - Parse `expression.(Type)` explicitly.
   - Add checked assertion nodes and reject non-interface operands.
3. Bytecode and runtime
   - Add explicit assertion lowering and VM execution with success unboxing and panic-on-failure behavior.
   - Reuse the runtime interface helper seam instead of scattering assertion logic.
4. Validation and synchronization
   - Add focused unit and CLI coverage, serial CLI evidence, line-count checks, and document synchronization.

## Acceptance Criteria

- `dump-ast` renders `x.(T)` explicitly without hiding it inside calls or conversions.
- `check` accepts successful assertions from `any` values to currently modeled runtime types and rejects assertions on non-interface operands.
- `run` returns the concrete payload on success, preserves typed-nil composite payloads, and reports interface-conversion panics for nil-interface and mismatched-type failures.
- `dump-bytecode` shows an explicit assertion instruction with the asserted runtime type.
- All touched files remain within the repository line-count limit.

## Risks

- Parser postfix ambiguity between selector expressions and type assertions must be handled without weakening existing package-selector behavior.
- Assertion success for typed-nil slice/map/chan payloads depends on preserving boxed runtime type metadata exactly.
- `src/frontend/parser.rs`, `src/semantic/analyzer/expressions.rs`, and runtime VM files are already dense, so helper extraction may be required in the same round.
