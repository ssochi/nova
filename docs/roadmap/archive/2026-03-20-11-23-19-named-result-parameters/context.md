# Context: Named Result Parameters

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, latest archived plan context, milestone index, active plan index, `todo.md`, and the latest roadmap/report context.
3. Confirmed milestone `M3-standard-library-and-runtime-model` remains `in_progress` with no active plan, so opening a new `M3` plan was the required next action.
4. Reviewed the existing function-signature, multi-result, parser, semantic, and bytecode surfaces to choose the next compatibility slice.
5. Chose grouped named result declarations plus bare `return` because they are the strongest adjacent gap after grouped input parameters and the current explicit multi-result support.
6. Verified the scope against the Go specification plus local Go 1.21.5 compiler probes covering grouped named results, mixed named/unnamed result lists, bare-return behavior, signature-name conflicts, blank result identifiers, and shadowed result-parameter diagnostics.
7. Added the research note, design note, and active-plan artifacts for the new slice.
8. Split `src/frontend/signature.rs` out of the near-limit AST file, changed function signatures to store explicit `ResultDecl` entries, and taught the parser to preserve grouped named results while rejecting mixed named/unnamed result lists.
9. Added result-declaration flattening in semantic support, extended function analysis to allocate named result slots alongside parameters, and added explicit semantic lowering for bare `return` with shadowing diagnostics.
10. Added checked-function result-local metadata plus compiler-side zero-value initialization prologues so named result slots do not depend on the VM's non-type-aware local defaults.
11. Added `examples/named_results.go`, focused parser and semantic tests, and dedicated CLI execution/diagnostic suites for named results.
12. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and `check`, three invalid `check` probes, and touched-file line-count checks.
13. Updated technical docs, verification and experience reports, and `BOOT.md` for the shipped slice.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`.

## Key Information for the Next Trigger

- Named result declarations now live explicitly in `src/frontend/signature.rs` as `ResultDecl`, while `dump-ast` preserves grouped result syntax and the checked/bytecode layers still flatten into ordered result slots.
- Bare `return` is implemented by semantic lowering into ordered reads of tracked result locals, and named result slots are initialized explicitly in bytecode before the function body because VM local defaults are not type-aware.
- Blank named results are supported through hidden locals such as `result$0`; that synthetic name is visible in `dump-bytecode` but not in the source-facing AST.
