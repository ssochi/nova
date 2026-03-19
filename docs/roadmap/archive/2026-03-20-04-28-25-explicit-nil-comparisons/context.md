# Context: Explicit Nil Comparisons

## Completed Steps

1. Ran the startup checklist and confirmed milestone `M3-standard-library-and-runtime-model` is still `in_progress`.
2. Read the latest archived plan, roadmap indexes, startup SOP, `todo.md`, and the current map/slice/runtime docs.
3. Confirmed there was no active plan, so the next highest-priority action was to open a new `M3` plan instead of drifting.
4. Reviewed the current parser, semantic, bytecode, runtime, and CLI surfaces to confirm that `nil` only exists as an internal zero-value state, not as a source-level expression.
5. Chose explicit `nil` plus composite equality as the next leverage point because it strengthens the existing slice/map runtime model and is a prerequisite for later Go-like composite semantics.
6. Verified the Go compatibility edge case `nil == nil` with the local Go toolchain (`go1.21.5`), which reports `invalid operation: nil == nil (operator == not defined on untyped nil)`.
7. Opened this active plan for explicit `nil` expressions, assignment rules, and `slice/map` nil comparisons under `M3`.
8. Added a focused research note and design baseline for explicit `nil`, covering assignability, `slice/map` nil equality, and the intentional rejection of `nil == nil`.
9. Extended the frontend, checked model, and semantic layer with explicit `nil` syntax, an untyped-`nil` checked expression, typed nil coercion for slice/map contexts, and nil-aware equality validation.
10. Kept lowering explicit by reusing `push-nil-slice` and `push-nil-map` for typed `nil` contexts instead of introducing a generic runtime nil value.
11. Added `examples/nil_values.go` plus parser, semantic, CLI execution, and CLI diagnostic coverage for explicit `nil`, typed user-defined/package call arguments, and invalid untyped-nil paths.
12. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, and serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and two `check` failure paths.
13. Wrote verification and experience reports, updated tech docs and `BOOT.md`, and prepared the plan for archive.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; explicit `nil` is now available for slice/map declarations, assignments, returns, typed call arguments, and nil comparisons, while broader nilable-type work, comma-ok lookups, `range`, and channels remain open.

## Key Information for the Next Trigger

- Keep the scope centered on the currently modeled composite types: slices and maps.
- Preserve the runtime distinction between nil and allocated-empty values; explicit `nil` should reuse that model, not erase it.
- Do not silently broaden equality support; the current explicit path is `slice/map` with `nil`, while `nil == nil` and general composite equality still remain rejected.
- The semantic layer now carries untyped `nil` explicitly and resolves it before bytecode lowering. Reuse that same hook for future channel/interface/pointer work instead of reintroducing ad hoc nil cases.
- Fixed-arity typed package functions can now accept `nil` when their signature provides the missing composite type context; variadic output-style calls still do not.
