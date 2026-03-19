# Slice Runtime Values CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Focus: first composite runtime value plus the CLI-visible slice flow
- Entry example: `examples/slices.go`

## Experience Path

1. Inspected token output with `cargo run -- dump-tokens examples/slices.go`.
2. Inspected AST output with `cargo run -- dump-ast examples/slices.go`.
3. Inspected bytecode output with `cargo run -- dump-bytecode examples/slices.go`.
4. Checked the source with `cargo run -- check examples/slices.go`.
5. Executed the program with `cargo run -- run examples/slices.go`.
6. Ran a negative `check` case for `values[true]`.

## Positive Experience

- The slice flow is readable end to end: syntax, AST, bytecode, and runtime output all line up cleanly.
- The bytecode view exposes the new behavior clearly through `build-slice`, `append`, and `index`.
- `check` remains package-level validation and reports slice type errors without depending on `main.main` execution.
- The new test split mirrors the CLI surface more cleanly than the previous single integration file.

## Issues and Severity

- Low: the current slice subset is intentionally small and does not yet include slicing syntax, `make`, `cap`, or element assignment.
- Low: slice rendering is approximate and runtime behavior still favors clarity over efficiency.

## Conclusion and Next Recommended Steps

The slice slice is usable and coherent for the current VM stage. The next round should build on it with either more slice operations such as element assignment / slicing, or leverage the composite-value foundation for broader package-backed runtime services.
