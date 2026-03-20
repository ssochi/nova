# Context: Variadic Functions and Explicit Ellipsis

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, latest archived plan context, milestone index, active plan index, `todo.md`, startup SOP, and the latest verification / experience reports tied to the archived call-forwarding plan.
3. Confirmed milestone `M3-standard-library-and-runtime-model` remains `in_progress` with no active plan, so opening a new `M3` plan was the next required action.
4. Reviewed the current slice, multi-result, builtin, package-contract, parser, semantic, compiler, and VM seams that touch function signatures and call arguments.
5. Chose staged variadic declarations plus explicit final-argument `...` as the next slice because they unlock realistic helper APIs and close the long-standing `append(slice, other...)` / `append([]byte, string...)` gap.
6. Verified with local Go 1.21.5 probes that variadic parameters are visible inside the callee as `[]T`, zero variadic arguments produce a nil slice, `prefix, values...` calls are valid, `append([]byte, string...)` is valid, and `...` is rejected for non-variadic functions.
7. Added the new variadic research note and opened this active plan before implementation.
8. Extended the lexer, tokens, AST, parser, and parser tests so variadic parameters and explicit final `...` call arguments remain visible in the source model.
9. Extended function-signature metadata, the checked call model, builtin validation, bytecode metadata, and VM call entry so user-defined variadic calls and builtin `append` spread handling execute without hiding the feature behind flat argument lists.
10. Added focused builtin, semantic, runtime, parser, CLI execution, and CLI diagnostic coverage plus the new real example `examples/variadic.go`.
11. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and a failing `check` path, then checked updated file sizes.
12. Updated research, tech docs, `BOOT.md`, verification / experience reports, and prepared the completed plan for archive.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`.

## Key Information for the Next Trigger

- Keep variadic declarations and explicit `...` spread calls explicit in the AST and checked model; do not flatten them into ordinary fixed-arity calls too early.
- Reuse the existing slice runtime when materializing variadic tails so zero-argument calls naturally become nil slices and `values...` can preserve shared backing behavior.
- Keep `...` separate from the staged multi-result expanded-call path; they are both explicit call-shape features but follow different Go rules.
- When a call uses explicit `...`, preserve the real fixed-prefix rule: only the non-variadic prefix arguments may appear before the spread value, while extra individual variadic arguments before `slice...` remain invalid.
