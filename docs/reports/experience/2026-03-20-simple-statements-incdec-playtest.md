# Simple Statements, Short Declarations, and Inc/Dec Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-06-34-01-simple-statements-incdec`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

1. Ran `cargo run -- run examples/simple_statements.go` to exercise ordinary short declarations, `if` / `switch` headers, classic `for i := 0; ...; i++`, local `--`, and map-index `++` through the normal CLI entrypoint.
2. Ran `cargo run -- dump-ast examples/simple_statements.go` to confirm the source-facing debug path stays readable for the new syntax.
3. Ran `cargo run -- dump-bytecode examples/simple_statements.go` to confirm hidden locals and index-target inc/dec lowering stay inspectable without reading Rust code.
4. Ran `cargo run -- check` on three failure paths: repeated short declaration in one scope, `string` target for `++`, and `:=` in a classic `for` post clause.

## Positive Experience

- The happy path now feels materially closer to ordinary Go: `for i := 0; i < len(values); i++` works directly, which removes a noticeable amount of staging friction from real examples.
- `dump-ast` remains easy to read because short declarations and inc/dec are rendered exactly as source-level constructs instead of being disguised as assignments.
- `dump-bytecode` is still debuggable even for `counts["go"]++` because the hidden `incdec$*` locals make the single-evaluation lowering obvious instead of magical.
- Error paths are understandable from the CLI alone; the repeated-short-declaration and wrong-type-incdec diagnostics are specific enough to act on immediately.

## Issues and Severity

- Medium: general short declarations are still narrow, so code that expects ordinary multi-binding `a, b := ...` remains outside the current subset except for the existing explicit comma-ok map lookup path.
- Medium: lack of compound assignments such as `+=` is now more visible because loops can use `i++`, but accumulation still needs verbose `total = total + value` rewrites.
- Low: the `for post statement does not support \`:=\`` parse diagnostic includes token-position detail, which is helpful, but it still reads more parser-oriented than a polished Go compiler diagnostic.

## Conclusion and Next Recommended Steps

This round materially improves the core CLI experience because `nova-go` can now express common Go headers and counting loops directly instead of forcing `var` declarations and assignment-only rewrites everywhere. The strongest next continuation is another explicit simple-statement slice for compound assignments, because that would pair naturally with the newly shipped short declarations and inc/dec support and unlock more idiomatic loop bodies without changing the current VM architecture.
