# Loop Control Flow Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-06-12-55-for-clauses-break-continue`
- Focus: real CLI experience for classic `for` clauses plus unlabeled `break` / `continue`

## Experience Path

1. Ran `cargo run -- run examples/loop_control.go` to exercise classic `for` init / condition / post, `continue` in a classic loop, `break` in both loop and `switch` contexts, and `continue` / `break` inside a staged `range` loop.
2. Ran `cargo run -- dump-ast examples/loop_control.go` to confirm the source-facing debug layer keeps classic `for` clauses plus loop-control statements visible.
3. Ran `cargo run -- dump-bytecode examples/loop_control.go` to inspect whether the new control-transfer lowering remains understandable without reading Rust code.
4. Ran `cargo run -- check <temp-source with top-level break>` as the first error path.
5. Ran `cargo run -- check <temp-source with switch-only continue>` as the second error path.
6. Ran `cargo run -- check <temp-source with infinite loop that breaks before falling through>` as the return-path regression check.

## Positive Experience

- The CLI now supports the first realistic Go loop shape instead of forcing users into condition-only loops or `range` exclusively.
- The new control-transfer behavior feels coherent: `continue` reaches the classic `for` post step, while `switch`-local `break` does not accidentally escape the surrounding loop.
- The debug surfaces remain useful. `dump-ast` shows the source-level clause structure directly, and `dump-bytecode` still exposes jump-based control transfer well enough to explain a wrong branch target quickly.
- The failure paths are direct and early. Invalid `break` / `continue` usage fails during `check`, and return-path diagnostics now catch loops that can escape through a modeled `break`.

## Issues and Severity

- Medium: classic `for` clauses still use a staged simple-statement subset, so common Go forms such as `i++` and general short declarations remain unavailable.
- Medium: `break` / `continue` are unlabeled only, which still blocks more advanced nested-control patterns from real Go codebases.
- Low: `dump-bytecode` for mixed loop / `switch` control is readable but jump-heavy; if labels or `fallthrough` arrive later, a small shared control-flow rendering helper may become worthwhile.

## Conclusion and Next Recommended Steps

This round closes a core control-flow gap: `nova-go` can now express ordinary counted loops and loop exits directly, not just condition-only loops and staged `range`. The strongest next `M3` direction is either to deepen control flow with labels / broader loop simple statements, or to return to runtime expansion with the first `chan` slice now that the control-flow foundation is materially stronger.
