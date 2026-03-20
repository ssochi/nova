# Context: Call-Argument Multi-Result Forwarding

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, latest archived plan context, milestone index, active plan index, `todo.md`, startup SOP, and the latest verification / experience reports tied to the archived multi-result plan.
3. Confirmed milestone `M3-standard-library-and-runtime-model` remains `in_progress` with no active plan, so opening a new `M3` plan was the next required action.
4. Reviewed the current multi-result research note, milestone recommendations, technical docs, and the semantic / compiler / runtime seams around calls and package dispatch.
5. Chose staged call-argument multi-result forwarding plus more `Cut*` package seams as the next slice because the new multi-result plumbing is now the main limiter on more realistic call chains.
6. Verified with local Go probes that multi-result call expansion in another call only applies when the whole argument list is that single call; prefixed arguments such as `f(1, pair())` still fail as single-value misuse.
7. Opened this active plan before implementation and recorded the coupled runtime-helper split so the VM file does not grow past the repository limit while this path is being touched.
8. Extended the existing multi-result checked model so calls now distinguish ordinary argument lists from an explicit expanded-call argument source.
9. Updated semantic analysis, package contracts, bytecode lowering, and diagnostics so `consume(pair())` and direct package-backed call forwarding work, while `f(1, pair())` still fails as a single-value misuse.
10. Added `strings.CutPrefix`, `strings.CutSuffix`, `bytes.CutPrefix`, and `bytes.CutSuffix` across package identity, semantic validation, VM dispatch, runtime helpers, and the new CLI example `examples/call_forwarding.go`.
11. Split VM builtin/package dispatch into `src/runtime/vm/builtins.rs` and `src/runtime/vm/packages.rs`, reducing `src/runtime/vm.rs` to 742 lines.
12. Added unit, semantic, runtime, CLI execution, and CLI diagnostic coverage for staged call forwarding and the new package seams.
13. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and a failing `check` path, then checked modified file sizes.
14. Updated research, tech docs, `BOOT.md`, verification and experience reports, and prepared the completed plan for archive.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`.

## Key Information for the Next Trigger

- Keep multi-result behavior explicit and narrow: only a single call argument may expand, and only when the entire argument list is that multi-result call.
- Reuse and extend the existing multi-result research note instead of creating a duplicate document for the same semantic surface.
- The next package- or syntax-driven `M3` slice can now spend this forwarding seam on more realistic APIs, but it should not relax the prefixed-argument rule without fresh research.
