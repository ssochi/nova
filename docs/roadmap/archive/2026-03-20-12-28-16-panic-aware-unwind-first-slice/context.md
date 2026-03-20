# Context: Panic-Aware Unwind First Slice

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, the newest archived plan context, milestone index, active plan index, `todo.md`, and the most relevant tech/design documents.
3. Confirmed milestone `M3-standard-library-and-runtime-model` remains `in_progress` with no active plan, so opening a new `M3` plan was the required next step.
4. Read the latest defer verification report plus the matching playtest naming/index rules and noted that experience reports are stored with the `-playtest` suffix.
5. Reviewed builtin contracts, bytecode lowering, runtime frame management, and file-size pressure to pick the next substantial slice.
6. Verified local Go 1.21.5 behavior for panic-driven defer execution, deferred panic overriding a normal return, and `panic(nil)` runtime failure.
7. Opened this plan for a staged panic-aware unwind slice built on top of the existing frame-level defer model.
8. Wrote the new research note and design note for staged builtin `panic` plus panic-aware unwind, then synchronized the research/design indexes.
9. Added builtin `panic` contracts, explicit `panic` / `panic-nil` / `defer-panic` / `defer-panic-nil` bytecode, and a compiler helper split so `src/bytecode/compiler.rs` stayed under the repository limit.
10. Reworked VM runtime errors into explicit message-vs-panic categories, added panic payload tracking plus unwind depth, preserved buffered output on final runtime failure, and extracted unwind helpers into `src/runtime/vm/unwind.rs`.
11. Routed explicit builtin panic plus selected runtime traps (`close` nil/closed channel, send on closed channel, nil-map assignment) through the new panic-aware defer path.
12. Added `examples/panic.go`, focused VM panic tests, focused CLI panic execution/diagnostic suites, and synchronized the test index plus testing strategy docs.
13. Updated technical docs and `BOOT.md`, then ran `cargo fmt`, `cargo test`, `cargo fmt --check`, serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and `check`, targeted invalid probes, and touched-file line-count checks.
14. Wrote the formal verification and playtest reports for the shipped panic slice.

## Current Status

- Plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`.

## Key Information for the Next Trigger

- Explicit builtin `panic` now exists across semantic validation, bytecode lowering, and the VM, with explicit bytecode instructions for both ordinary payloads and `panic(nil)`.
- VM execution now preserves buffered output on final runtime failure and reuses the defer/frame model for panic-driven unwind through a dedicated `pending_panic + panic_depth` hook in `src/runtime/vm/unwind.rs`.
- `recover` still remains the main adjacent gap, but it should wait for interface/`any` groundwork rather than being approximated with an ad hoc type placeholder.
