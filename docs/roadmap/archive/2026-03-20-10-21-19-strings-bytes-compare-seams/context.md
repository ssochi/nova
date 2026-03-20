# Context: Strings and Bytes Compare Seams

## Completed Steps

1. Ran the startup checklist from `docs/sop/startup-context-refresh.md`.
2. Read the root `AGENTS.md`, latest archived plan context, milestone index, active plan index, `todo.md`, startup SOP, and the recent validation and experience reports because no active plan was open.
3. Confirmed milestone `M3-standard-library-and-runtime-model` remains `in_progress` with no active plan, so opening a new `M3` plan was the required next action.
4. Reviewed milestone recommendations, existing `strings` / `bytes` package seams, runtime helper layout, and file-size pressure across semantic and CLI test files.
5. Chose staged `strings.Compare` / `bytes.Compare` as the next slice because the helpers are low-risk, byte-oriented, and explicitly recommended by the milestone document.
6. Verified with local Go 1.21.5 docs that both helpers are lexicographic integer-returning APIs, and that `bytes.Compare` treats nil the same as an empty slice.
7. Opened this active plan before implementation.
8. Added a dedicated research note, design note, and active-plan artifacts for the `Compare` slice.
9. Extended shared package metadata, semantic package contracts, VM package dispatch, and reusable byte-sequence helper logic for `strings.Compare` and `bytes.Compare`.
10. Added `examples/strings_bytes_compare.go`, focused semantic and runtime unit tests, and two focused CLI integration files so the umbrella integration files did not grow further.
11. Ran `cargo fmt`, `cargo test`, `cargo fmt --check`, serial CLI validation through `run`, `dump-ast`, `dump-bytecode`, and `check`, plus two invalid `check` probes and file-size checks.
12. Updated verification and experience reports, technical docs, milestone state, `BOOT.md`, and archive/index metadata for the completed plan.

## Current Status

- The plan is complete and ready for archive.
- Milestone `M3-standard-library-and-runtime-model` remains `in_progress`.

## Key Information for the Next Trigger

- `strings.Compare` and `bytes.Compare` now work end to end through `check`, `dump-ast`, `dump-bytecode`, and `run`.
- `bytes.Compare` should keep the Go rule that nil and empty byte slices compare as equal; future compare-adjacent helpers should not regress that boundary.
- Keep Unicode- or rune-sensitive helpers deferred, and keep using focused CLI integration files instead of extending the umbrella test files.
