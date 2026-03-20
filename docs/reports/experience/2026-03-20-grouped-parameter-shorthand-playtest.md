# Grouped Parameter Shorthand Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-11-02-46-grouped-parameter-shorthand`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/grouped_parameters.go` to exercise grouped ordinary parameters plus grouped-prefix variadic signatures through the real CLI path.
2. Ran `cargo run -- dump-ast examples/grouped_parameters.go` to confirm grouped declarations stay visible as source-level syntax.
3. Ran `cargo run -- dump-bytecode examples/grouped_parameters.go` to confirm the VM-facing metadata remains understandable after grouped declarations flatten into slots.
4. Ran `cargo run -- check` on two invalid programs to inspect grouped-variadic and duplicate-name diagnostics.

## Positive Experience

- The feature fits the current CLI flow cleanly: no new command or hidden lowering mode is required, and the example reads like ordinary Go source.
- `dump-ast` is materially better than a flattened-only implementation would be because grouped declarations remain visible exactly where a user expects to inspect source structure.
- `dump-bytecode` stays useful because it explains the runtime truth plainly: grouped declarations become ordinary ordered parameters, and the variadic tail is still explicit as `params=2 + ...int`.
- Diagnostics stay early and understandable. The parser catches grouped variadic misuse before semantic analysis, while duplicate names still use the existing function-signature diagnostic style.

## Issues and Severity

- Low: grouped result declarations and named results remain unsupported, so some common Go signature shorthand is still unavailable even after this improvement.
- Low: bytecode inspection intentionally shows flattened parameter slots rather than preserving grouped markers, so users need `dump-ast` for source-faithful signature shape.

## Conclusion and Next Recommended Steps

The real CLI path is better in a practical way: ordinary Go function signatures no longer need the more verbose `a int, b int` workaround, and the debug surfaces remain coherent from source to bytecode. The strongest next continuation is to return to a broader `M3` runtime or package seam while keeping grouped results and named returns as a separate, deliberately designed function-signature slice.
