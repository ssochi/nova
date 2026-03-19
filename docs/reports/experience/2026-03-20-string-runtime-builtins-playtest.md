# String Runtime and Builtin CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-00-09-59-string-runtime-builtins`
- Entry surface: `run`, `dump-tokens`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/strings.go` to verify the end-to-end happy path for strings and builtins.
2. Ran `cargo run -- dump-tokens examples/strings.go` to inspect literal tokenization and builtin call spelling.
3. Ran `cargo run -- dump-ast examples/strings.go` to inspect the source-level structure before semantic lowering.
4. Ran `cargo run -- dump-bytecode examples/strings.go` to inspect `push-string`, `concat`, and builtin dispatch.
5. Ran `cargo run -- check examples/strings.go` to confirm package-level semantic validation on the same source.
6. Ran `cargo run -- check <temp-invalid-len-source>` to verify the builtin error path.

## Positive Experience

- The same example source is understandable across tokens, AST, bytecode, and runtime output, which makes the new `M3` slice easy to inspect.
- The bytecode dump is concrete enough to explain how string concatenation and builtin calls are lowered.
- The invalid `len(1)` path fails early with a direct type error instead of a runtime failure.

## Issues and Severity

- Medium: `print` / `println` behavior is coherent but still simplified relative to real Go formatting, so users could over-assume compatibility.
- Low: string literals currently support only a narrow escape subset and no raw-string form.
- Low: CLI diagnostics still do not include source snippets or spans, which slows down debugging once programs become larger.

## Conclusion and Next Recommended Steps

This slice is viable as the first `M3` runtime expansion because it improves real CLI usage without collapsing the existing layers. The next best step is another runtime-oriented plan that adds either composite values or an import / standard-library seam, while keeping builtin contracts centralized.
