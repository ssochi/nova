# Strings Package Contracts CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-01-17-51-strings-package-contracts`
- Entry surface: `dump-tokens`, `dump-ast`, `dump-bytecode`, `check`, `run`

## Experience Path

1. Ran `cargo run -- dump-tokens examples/strings_package.go` to verify that the imported `strings` surface is visible before parsing and semantic analysis.
2. Ran `cargo run -- dump-ast examples/strings_package.go` to inspect the readable source shape for nested `strings` package calls.
3. Ran `cargo run -- dump-bytecode examples/strings_package.go` to confirm the new functionality remains obvious through `call-package strings.*` instructions.
4. Ran `cargo run -- check examples/strings_package.go` to validate the happy path without entrypoint execution assumptions.
5. Ran `cargo run -- run examples/strings_package.go` to verify the real imported-package execution path.
6. Ran `cargo run -- check <temp-invalid-strings-join-source>` to inspect the typed error path.

## Positive Experience

- The example is readable across all CLI surfaces, so the second package seam still feels inspectable instead of magical.
- `call-package strings.*` makes the bytecode output more expressive; users can see that package-backed behavior is distinct from builtin dispatch.
- The failure path is more precise than the earlier `fmt` seam because it points directly to the typed contract mismatch instead of only saying the member is unsupported.

## Issues and Severity

- Medium: the current `strings` support is still a curated subset, so users may overestimate compatibility if they assume the full standard package is present.
- Low: the runtime maps some real-Go `strings.Repeat` failure cases into VM errors rather than panic behavior, which is pragmatic for now but not exact.
- Low: package loading remains metadata-backed, so imports still do not reflect the real filesystem or module graph experience.

## Conclusion and Next Recommended Steps

This slice materially improves the standard-library path because imported packages are no longer limited to variadic output helpers; the project now has a typed package-contract model that future packages can reuse. The next best step is either deeper slice behavior such as slice expressions or another typed package seam that benefits from the same validator structure.
