# Plan: Empty Interface Any and Fmt Spread

## Basic Information

- Plan ID: `2026-03-20-12-55-53-empty-interface-any-and-fmt-spread`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary agent

## Goals

- Add the first staged empty-interface runtime and type-system slice through both `any` and `interface{}`.
- Model explicit interface boxing and nil-interface zero values without hiding them inside ad hoc runtime coercions.
- Extend package-call lowering and VM execution so `fmt.Print*` can consume `[]any` through explicit `...` spread calls.

## Scope

- Research current Go behavior for `any` aliasing, `interface{}` zero values, interface equality edges needed for this slice, and `fmt.Println(args...)` over `[]any`.
- Extend the lexer, parser, AST rendering, semantic type model, bytecode type model, and runtime values for empty-interface support.
- Add explicit checked/lowered boxing so assignments, returns, slice literals, and call arguments targeting `any` remain inspectable.
- Add package-call spread handling for the staged variadic `fmt` functions using `[]any`.
- Add focused examples, tests, CLI validation traces, and file-size governance updates for touched near-limit files.

## Non-Goals

- Method-bearing interfaces, type assertions, type switches, or dynamic dispatch in this round.
- `recover` in this round, even though this slice prepares its type/runtime groundwork.
- Interface-backed map keys, interface method sets, or generalized package `...` support beyond the staged `fmt` variadic helpers.
- Rich `fmt` formatting verbs or reflection-like runtime services.

## Phase Breakdown

1. Research and design
   - Verify the minimal real-Go semantics needed for `any` / `interface{}` and `fmt` spread.
   - Record explicit scope limits so the slice does not overclaim method/interface support.
2. Frontend and semantic work
   - Parse `interface{}` and resolve `any` / `interface{}` into an explicit empty-interface type.
   - Add explicit checked boxing and equality rules needed for nil checks and `fmt` usage.
3. Bytecode and runtime work
   - Add explicit runtime/value-type support for interface values and boxing.
   - Add package spread lowering/execution for staged `fmt` variadics.
4. Validation and synchronization
   - Add focused tests, example programs, CLI traces, doc updates, and line-count checks.

## Acceptance Criteria

- `var value any`, `var value interface{}`, `[]any{...}`, and `fmt.Println(args...)` all work through `run`, `dump-ast`, `dump-bytecode`, and `check`.
- The checked and bytecode layers keep interface boxing explicit enough that `dump-bytecode` exposes the new path without reading implementation code.
- Nil-interface zero values are distinct from boxed typed-nil composite values in the runtime behavior shipped this round.
- Equality covers the staged interface cases documented for this slice, including `value == nil` and comparable dynamic payloads, without silently claiming unsupported interface features.
- Documentation, roadmap state, and validation reports explain that `recover` is now unblocked by type groundwork but still deferred.

## Risks

- Near-limit files such as `src/runtime/vm.rs`, `src/bytecode/compiler.rs`, and `src/semantic/analyzer/expressions.rs` will likely require helper extraction in the same round.
- Interface equality can easily overclaim Go behavior; the staged rule must stay explicit and runtime-visible where payload comparability matters.
- Reusing raw runtime values for `any` without an explicit nil-interface model would produce incorrect `value == nil` behavior.
