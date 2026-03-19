# Switch Statements Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-05-46-33-switch-statements`
- Focus: real CLI experience for staged expression `switch`, tagless `switch`, shared header scope, and switch-specific diagnostics

## Experience Path

1. Ran `cargo run -- run examples/switch_statements.go` to exercise tagless `switch` with a comma-ok map lookup header, an expression `switch` with multiple `case` expressions, an expression-statement header, and an assignment-style header.
2. Ran `cargo run -- dump-ast examples/switch_statements.go` to confirm the source-facing debug layer keeps the new switch surface visible.
3. Ran `cargo run -- dump-bytecode examples/switch_statements.go` to inspect whether single-evaluation tag lowering and clause dispatch stay understandable.
4. Ran `cargo run -- check <temp-source with leaked switch-header binding>` as the first error path.
5. Ran `cargo run -- check <temp-source with duplicate switch default>` as the second error path.

## Positive Experience

- The CLI now supports a much more realistic Go branching shape. Ordinary value dispatch no longer has to be spelled as stacked `if` chains.
- The shared header model feels coherent: the same comma-ok map lookup form works in a tagless switch without introducing a second special-case syntax.
- The debug surfaces stay useful. `dump-ast` preserves the source form, while `dump-bytecode` makes the hidden `switch$tag` locals and clause jumps obvious enough for debugging.
- The failure paths are direct and early. Header-scope leaks and duplicate defaults are rejected during `check` with diagnostics that are understandable without reading Rust code.

## Issues and Severity

- Medium: `switch` is still a narrow slice because there is no `break`, `fallthrough`, or type-switch support.
- Medium: duplicate-case diagnostics only cover the currently modeled scalar literal cases, so broader constant-expression duplicates still remain an open gap.
- Low: `dump-bytecode` uses a verbose jump pattern for multi-expression cases; it is readable, but later control-flow features may warrant a small abstraction to reduce noise.

## Conclusion and Next Recommended Steps

This round materially improves real CLI usability because `nova-go` can now express common Go dispatch directly instead of forcing nested `if` ladders. The strongest next `M3` continuation is to decide whether to deepen statement control with `break` / `continue` and richer `for` syntax, or to return to runtime expansion with the first `chan` slice now that both `if` and `switch` headers exist.
