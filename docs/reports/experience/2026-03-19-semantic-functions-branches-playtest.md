# Semantic Functions and Branches Playtest

## Basic Context

- Date: `2026-03-19`
- Entry point: `cargo run -- <subcommand>`
- Scope: CLI walkthrough for semantic analysis, multi-function execution, and branch diagnostics

## Experience Path

1. Ran `cargo run -- run examples/functions_and_branches.go` to verify the new happy path.
2. Ran `cargo run -- dump-bytecode examples/functions_and_branches.go` to inspect the multi-function bytecode output.
3. Ran `cargo run -- check /tmp/nova-go-bad-if.go` with an invalid `if` condition to confirm the semantic failure path.

## Positive Experience

- The same CLI surface now handles a meaningfully richer program without adding command complexity.
- The bytecode dump remains readable even after moving to a per-function execution model.
- Semantic failures appear before runtime and use direct language-level wording such as `if condition requires bool`.
- Entry-point execution and package-level checking now feel distinct in practice, which makes `check` more useful.

## Issues and Severity

- Medium: loop constructs are still absent, so the control-flow story is only partially complete.
- Medium: semantic diagnostics identify the rule clearly but still do not show source snippets.
- Low: bytecode output is longer now, and there is no filtering option for a single function yet.

## Conclusion and Next Recommended Steps

The CLI now supports the first credible multi-function Go subset instead of only a single-entry bootstrap. The next round should finish milestone `M2` by adding looping control flow and then strengthen diagnostics around semantic spans and bytecode inspection ergonomics.
