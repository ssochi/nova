# Defer First Slice Playtest

## Basic Context

- Date: `2026-03-20`
- Related plan: `2026-03-20-11-58-27-defer-first-slice`
- Entry point: `cargo run -- <command> ...`

## Experience Path

1. Ran `cargo run -- run examples/defer.go` to exercise staged `defer` through the real CLI path with builtin, package, and user-defined deferred calls.
2. Ran `cargo run -- dump-ast examples/defer.go` to confirm defer stays visible as source-level syntax.
3. Ran `cargo run -- dump-bytecode examples/defer.go` to confirm defer lowering remains explicit and readable.
4. Ran `cargo run -- check` on two invalid programs to inspect the parenthesized-form and builtin statement-context diagnostics.

## Positive Experience

- The feature fits the existing CLI flow cleanly: `run`, `dump-ast`, `dump-bytecode`, and `check` all expose the defer slice without a new command mode or hidden debug flag.
- `dump-bytecode` is especially useful for this round because it shows deferred execution as dedicated instructions instead of forcing the reader to infer control-flow rewrites from raw jumps.
- The real CLI output proves the two most important user-visible semantics quickly: deferred arguments are captured immediately, and execution order is LIFO.
- The staged diagnostics are concrete enough to guide the user toward the supported surface instead of failing later in lowering or runtime execution.

## Issues and Severity

- Low: the staged defer surface still excludes closures, method values, and panic/recover-aware unwinding, so users can only defer the currently supported direct-call forms.
- Low: builtin statement-context filtering is now correct for `defer`, but ordinary expression statements outside defer are still more permissive than real Go in some builtin cases.

## Conclusion and Next Recommended Steps

The real CLI path now feels materially closer to Go function-exit behavior: `defer` is visible in debug surfaces, executes in the right order, and composes with the current package/user-function subset. The strongest next continuation is another `M3` plan that either deliberately extends unwind semantics (`panic` / `recover`) or keeps broadening realistic Go call surfaces without abandoning the explicit frame-level defer model.
