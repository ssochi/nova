# Panic-Aware Unwind First Slice Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-12-28-16-panic-aware-unwind-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

- Ran the real CLI against `examples/panic.go` through `run`, `dump-ast`, `dump-bytecode`, and `check`.
- Probed one temporary nil-map runtime-trap program through `run` to confirm deferred output survives panic-triggered failure.
- Probed one temporary `panic(nil)` program through `dump-bytecode` to confirm the nil-special case remains inspectable.
- Scope boundary: this is a focused panic slice playtest, not a milestone-closeout full CLI blackbox pass.

## Positive Experience

- The `run` path now shows the meaningful user-facing sequence for panic cases: normal output first, deferred output next, and the final panic message last.
- `dump-ast` remains source-oriented because the panic call is still rendered as ordinary Go syntax rather than a synthetic statement form.
- `dump-bytecode` is readable enough to explain the new runtime path quickly: `panic`, `panic-nil`, `defer-panic`, and `defer-panic-nil` expose the staged surface directly.
- Package-only validation through `check` still works cleanly and does not require a runtime entrypoint beyond the normal package rules.

## Issues and Severity

- Medium: `recover` is still absent, so the current panic surface is one-way unwind only.
- Medium: only selected runtime traps use panic-aware unwinding today; broader runtime panic parity remains incomplete.
- Low: panic output is intentionally lighter than real Go because there is still no stack-trace or runtime-error-object formatting layer.

## Conclusion and Next Recommended Steps

The staged panic slice is usable and materially improves the CLI-first experience because deferred output is no longer lost when execution fails. The next strongest continuation is either `recover` preparation through interface/`any` groundwork or a broader runtime-panic consolidation pass that moves more existing trap sites onto the same unwind path.
