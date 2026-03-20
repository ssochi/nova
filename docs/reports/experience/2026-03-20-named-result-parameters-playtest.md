# Named Result Parameters Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-11-23-19-named-result-parameters`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/named_results.go` to exercise grouped named results, blank result identifiers, and bare `return` through the real CLI path.
2. Ran `cargo run -- dump-ast examples/named_results.go` to confirm grouped result declarations stay visible as source-level syntax.
3. Ran `cargo run -- dump-bytecode examples/named_results.go` to confirm result-slot zero-value initialization is explicit and readable in the lowered program.
4. Ran `cargo run -- check` on three invalid programs to inspect mixed-result, shadowed bare-return, and unnamed-result bare-return diagnostics.

## Positive Experience

- The feature fits the existing CLI flow without any special command mode. Named results show up naturally in `run`, `dump-ast`, `dump-bytecode`, and `check`.
- `dump-ast` is the right inspection surface for this slice because it keeps grouped result declarations visible instead of flattening them away.
- `dump-bytecode` stays readable even after the semantic/runtime fix for zero-value result slots because the initialization prologue is short and explicit.
- The shadowing diagnostic is practical: when a nested short declaration hides a named result, the CLI reports the bare-return problem directly instead of failing later or returning the wrong slot.

## Issues and Severity

- Low: the current unnamed-result bare-return diagnostic is staged rather than Go-exact; it reports the existing `must return a <type> value` wording instead of Go's `not enough return values`.
- Low: blank named results appear as synthetic locals such as `result$0` in `dump-bytecode`, which is accurate for the current lowering but not source-level syntax.

## Conclusion and Next Recommended Steps

The real CLI path is materially better: common Go function forms such as grouped named results and bare `return` now work end-to-end, and the debug surfaces still explain the implementation choices clearly. The strongest next continuation is another `M3` compatibility slice that builds on the richer function-signature surface, such as narrowing the gap on call/use sites or additional standard-library seams that benefit from named-result support.
