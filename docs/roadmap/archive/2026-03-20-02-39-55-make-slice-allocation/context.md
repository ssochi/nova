# Context: Make-Based Slice Allocation

## Completed Steps

1. Ran the startup checklist and confirmed milestone `M3-standard-library-and-runtime-model` is still `in_progress`.
2. Read the latest archived plan for typed declarations and nil-slice zero values plus its verification and CLI experience reports.
3. Reviewed the current slice research, design, runtime docs, and source layout to identify the next highest-value `M3` gap.
4. Chose builtin `make([]T, len[, cap])` as the next iteration because the compiler now has nil slices and capacity-aware append but still lacks general slice allocation.
5. Opened this active plan and attached it to milestone `M3-standard-library-and-runtime-model`.
6. Extended the existing slice research baseline with official `make` semantics instead of creating a duplicate research note.
7. Added explicit AST and checked-model support for `make`, keeping the type argument separate from ordinary runtime-valued call arguments.
8. Lowered `make([]T, len[, cap])` into `make-slice` bytecode with runtime element-type descriptors and implemented zero-filled spare-capacity allocation in the VM.
9. Added `examples/make_slices.go` plus parser, semantic, runtime, and CLI coverage for `make` allocation and invalid constant bounds.
10. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, and serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and a negative `check` case.
11. Synced research, design, tech, reports, roadmap docs, and `BOOT.md` for the new allocation surface.

## Current Status

- The plan is complete and ready to archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`; slice allocation is materially stronger, but map/channel allocation, string slicing, and broader byte-oriented runtime work remain open.

## Key Information for the Next Trigger

- Reuse the existing slice research note instead of creating a parallel note for `make`; this is the same semantic surface.
- This round kept `make` slice-only on purpose. Do not imply map or channel `make` support in parser, runtime, or docs until their runtime categories exist.
- The explicit AST / checked-model path for type-argument builtins is now in place and should be reused if later builtins need type syntax.
- Hidden capacity slots are zero-filled in runtime storage, so later slice work can build on that when revisiting full append growth, stronger constant handling, or byte-oriented slices.
