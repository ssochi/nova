# Context: Map Literals and Delete

## Completed Steps

1. Ran the startup checklist and confirmed milestone `M3-standard-library-and-runtime-model` is still `in_progress`.
2. Read the latest archived plan, roadmap indexes, startup SOP, playtest SOP, `todo.md`, and the current map/runtime docs.
3. Confirmed there was no active plan, so the next highest-priority action was to open a new `M3` plan instead of drifting.
4. Chose staged map usability as the next leverage point because map groundwork already exists and the current milestone still calls out deeper composite runtime work.
5. Opened this active plan for `map[K]V{...}` literals and builtin `delete(map, key)`.
6. Extended the existing map research note and design baseline so literal construction, nil-map delete behavior, and current Go-compatibility gaps were explicit before code changes.
7. Split parser tests into a dedicated submodule to keep `src/frontend/parser.rs` comfortably below the repository file-size ceiling before adding new syntax.
8. Added explicit AST and checked-expression support for `map[K]V{...}` literals, including keyed entry modeling and readable AST rendering.
9. Extended builtin contracts, semantic analysis, bytecode lowering, and VM execution for staged map literals and builtin `delete(map, key)`.
10. Added `build-map` bytecode, runtime map deletion support, and tests covering empty literals, nil-map deletes, and deterministic last-write-wins duplicate-key behavior.
11. Added `examples/map_literals.go` plus parser, semantic, VM, CLI execution, and CLI diagnostic coverage for the new map usability slice.
12. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, and serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and two `check` failure paths.
13. Wrote verification and experience reports, updated tech docs and `BOOT.md`, and prepared the plan for archive.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; staged map literals and builtin `delete` are now available, but duplicate-constant-key diagnostics, comma-ok lookups, `range`, channels, and broader runtime/package work remain open.

## Key Information for the Next Trigger

- Reuse and extend `docs/research/2026-03-20-map-runtime-groundwork.md` instead of creating a duplicate map compatibility note.
- Preserve the current explicit nil-map model: nil reads still return zero values, nil writes still fail, and `delete` should follow Go's nil-map no-op behavior rather than implicitly allocating storage.
- Keep the new literal and delete execution path visible in `dump-ast` and `dump-bytecode`; do not hide it behind generic builtin fallback lowering.
- The current map literal implementation intentionally keeps deterministic last-write-wins behavior for duplicate keys; if a later round aims for tighter Go compatibility, extend the same research/design note and decide how to surface duplicate-constant-key diagnostics without scattering constant-folding logic.
