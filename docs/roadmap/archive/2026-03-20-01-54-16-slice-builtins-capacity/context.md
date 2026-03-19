# Context: Slice Builtins and Capacity-Aware Append

## Completed Steps

1. Ran the startup checklist, read `AGENTS.md`, the latest archived plan context, roadmap indexes, `todo.md`, `BOOT.md`, and the relevant SOP and runtime docs.
2. Confirmed milestone `M3-standard-library-and-runtime-model` is still `in_progress` and there was no active plan, so a new plan was required before implementation.
3. Reviewed the current slice runtime, builtin contracts, tests, and CLI reports to choose the next high-value `M3` slice.
4. Chose a combined iteration around `cap`, `copy`, and capacity-aware `append` because it deepens the shared slice model without mixing in string runtime work.
5. Opened this active plan and attached it to milestone `M3-standard-library-and-runtime-model`.
6. Extended the existing slice research note and design baseline with official `cap`, `copy`, and append-capacity semantics instead of creating duplicate same-surface docs.
7. Added builtin IDs and centralized contracts for `cap` and `copy`, then upgraded the slice runtime so `copy` is overlap-safe and `append` reuses shared storage when capacity allows.
8. Added `examples/slice_builtins.go` plus unit and CLI coverage for happy-path slice builtins, append reuse, and targeted diagnostics.
9. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, and serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and a negative `check` case.
10. Synced tech, report, AGENTS, and `BOOT.md` documentation for the new slice builtin surface.

## Current Status

- The plan is complete and ready to archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; slice runtime behavior is materially stronger, but string slicing, `make` / nil slices, and a deeper allocation model are still open.

## Key Information for the Next Trigger

- The current runtime already stores slice capacity metadata plus shared backing storage, so reuse that model instead of introducing a second append path.
- `copy` now snapshots source elements before writing, which is the right hook to preserve if later work adds `[]byte` or other slice categories.
- `append` now reuses storage within capacity but still allocates with a minimal capacity rule once it overflows; a future `make`-oriented plan can revisit growth heuristics cleanly.
- Array, pointer-to-array, channel, and string-backed builtin semantics are intentionally out of scope for this iteration and should remain documented rather than half-implemented.
