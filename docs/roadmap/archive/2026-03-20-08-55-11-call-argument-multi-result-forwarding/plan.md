# Plan: Call-Argument Multi-Result Forwarding

## Basic Information

- Plan ID: `2026-03-20-08-55-11-call-argument-multi-result-forwarding`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`
- Owner: primary project agent

## Goals

- Extend the staged multi-result model so a single call argument may forward its whole result list into another call without introducing tuple runtime values.
- Spend that new consumer path on more package-backed standard-library seams that depend on multi-result returns.
- Reduce `src/runtime/vm.rs` growth by moving package/builtin dispatch helpers into narrower runtime modules while touching that surface.

## Scope

- Single-argument multi-result forwarding for builtins, user-defined calls, and imported package calls when the entire argument list is one multi-result call.
- Explicit checked-model plumbing for flattened argument counts and the expanded-call source.
- `strings.CutPrefix`, `strings.CutSuffix`, `bytes.CutPrefix`, and `bytes.CutSuffix`.
- Bytecode and VM execution changes required to keep `dump-bytecode` readable for the widened call path.
- Examples, automated coverage, CLI validation, and documentation updates for the new call-consumer and package surfaces.
- Runtime module splitting tied directly to the touched package/builtin dispatch path.

## Non-Goals

- General tuple expressions or multi-result values in arbitrary non-call expression positions.
- Prefix arguments alongside an expanded multi-result call, non-final expansion, or multiple expanded arguments in one call.
- Named results, naked returns, variadic `...` syntax, or broader import graph work.
- Channel comma-ok receive, channel `range`, or scheduler-aware blocking design.

## Phase Breakdown

1. Extend the existing multi-result research note with official behavior for call-argument forwarding and the chosen `CutPrefix` / `CutSuffix` APIs.
2. Record the staged design for explicit expanded-call modeling and the related runtime-module split.
3. Implement semantic and checked-model support for final-argument expansion plus the new package contracts.
4. Implement compiler/runtime/package dispatch changes, examples, and tests for the new call flow and package APIs.
5. Run formatting, unit/integration tests, serial CLI validation, documentation/report sync, and archive the plan if complete.

## Acceptance Criteria

- Calls such as a user-defined wrapper `consume(pair())`, `println(strings.Cut(...))`, and `fmt.Println(bytes.CutPrefix(...))` succeed through `check`, `dump-ast`, `dump-bytecode`, and `run`.
- Unsupported multi-result argument shapes still fail with targeted diagnostics instead of collapsing into generic arity errors.
- The checked model and bytecode keep call-argument forwarding explicit enough that later agents can distinguish ordinary arguments from the staged expansion path.
- `src/runtime/vm.rs` drops comfortably below the repository file-size ceiling after the runtime helper split.

## Risks

- Call-argument forwarding can blur the existing single-value expression rule if the checked model does not keep the expanded call explicit.
- Variadic builtins and package functions must preserve predictable arity/type diagnostics when a final multi-result call is present.
- Adding four more package seams while touching runtime dispatch could let package behavior sprawl unless the helper split stays deliberate.
