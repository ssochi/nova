# Builtin Clear and Runtime File Split Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-10-46-29-builtin-clear-and-runtime-file-split`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/builtin_clear.go` to exercise slice-window clearing, nil-slice no-op behavior, string-slice zeroing, and shared-map alias clearing through the real CLI path.
2. Ran `cargo run -- dump-ast examples/builtin_clear.go` to confirm builtin `clear` stays visible in the source inspection surface.
3. Ran `cargo run -- dump-bytecode examples/builtin_clear.go` to confirm the mutating path stays readable as explicit `call-builtin clear 1`.
4. Ran `cargo run -- check` on two invalid programs to inspect string and channel misuse diagnostics.

## Positive Experience

- The new builtin fits the existing CLI flow cleanly: there is no hidden syntax or lowering surprise, and the example demonstrates the important aliasing and nil behaviors in a small program.
- The AST and bytecode views remain useful because `clear` is visible end to end instead of being rewritten into synthetic assignment loops.
- The runtime file split pays off immediately: the feature landed without pushing `src/runtime/value.rs` over the repository ceiling, and the focused CLI files keep the broad integration suites stable.
- Diagnostics stay direct and early, with `check` reporting unsupported `clear` targets before execution starts.

## Issues and Severity

- Low: `src/runtime/vm/tests.rs` is still near the repository size ceiling, so another VM-heavy feature should continue the same focused-submodule pattern.
- Low: generic `clear` behavior is not modeled yet because the project still lacks generics.

## Conclusion and Next Recommended Steps

The real CLI path improved in a concrete way: builtin `clear` now behaves like Go for the staged slice/map surface, and the runtime test layout is in a better state for the next iteration. The strongest next continuation is another core builtin/runtime slice or a package-backed API that benefits from the newly reduced runtime file pressure.
