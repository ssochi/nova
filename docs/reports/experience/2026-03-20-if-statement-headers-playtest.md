# If Statement Headers Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-05-29-16-if-initializers-else-if`
- Focus: real CLI experience for staged `if` statement initializers, shared header scope, and `else if` chaining

## Experience Path

1. Ran `cargo run -- run examples/if_headers.go` to exercise comma-ok `map` lookup headers, assignment-style headers, expression-statement headers, and `else if` chaining through the real CLI entrypoint.
2. Ran `cargo run -- dump-ast examples/if_headers.go` to confirm the new header surface remains visible at the source-oriented debug layer.
3. Ran `cargo run -- dump-bytecode examples/if_headers.go` to inspect whether initializer execution and nested branch jumps stay visible without reading implementation code.
4. Ran `cargo run -- check <temp-source with leaked if-header binding>` as the first error path.
5. Ran `cargo run -- check <temp-source with missing if-header semicolon>` as the second error path.

## Positive Experience

- The new flow feels much closer to ordinary Go. Comma-ok `map` checks can now live directly in branch headers instead of requiring prelude statements or awkward sentinel comparisons.
- The staged control-flow path stays inspectable. `dump-ast` shows the initializer and condition in one header, and `dump-bytecode` makes it obvious that initializer instructions run before each branch jump.
- The scope failure path is understandable: users get a direct `unknown variable` diagnostic instead of a confusing runtime symptom when they try to use a header binding after the `if`.

## Issues and Severity

- Low: `dump-ast` prints `else` on its own line before nested `if` content instead of the more idiomatic single-line `else if` presentation.
- Medium: Statement headers are still incomplete because only the current simple-statement subset is supported; general short declarations in headers remain unavailable.
- Medium: This round improves control-flow ergonomics but does not yet help `switch` headers or channel-driven control flow, which many larger Go programs still rely on.

## Conclusion and Next Recommended Steps

This round meaningfully improves real CLI usability because the earlier comma-ok `map` work now reaches a common Go branch shape, and branch-local setup no longer has to be spelled as separate statements. The strongest next `M3` continuation is to decide whether to keep widening control-flow ergonomics with staged `switch` headers or to pivot into the first `chan` runtime slice now that `if` headers are no longer a blocker for ordinary map-driven branches.
