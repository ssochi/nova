# Import Fmt Seam CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-00-35-11-import-fmt-seam`
- Entry surface: `dump-tokens`, `dump-ast`, `dump-bytecode`, `check`, `run`

## Experience Path

1. Ran `cargo run -- dump-tokens examples/imports_fmt.go` to verify that the new import and selector syntax is visible at the CLI boundary.
2. Ran `cargo run -- dump-ast examples/imports_fmt.go` to inspect the imported-package source structure before semantic analysis.
3. Ran `cargo run -- dump-bytecode examples/imports_fmt.go` to inspect `call-package` lowering and builtin/package interoperation.
4. Ran `cargo run -- check examples/imports_fmt.go` to confirm that package-level validation accepts the new seam without needing entrypoint execution.
5. Ran `cargo run -- run examples/imports_fmt.go` to verify the real imported-package happy path.
6. Ran `cargo run -- check <temp-missing-import-source>` to verify the missing-import failure path.

## Positive Experience

- The same imported example remains readable through all CLI surfaces, which keeps the new import seam easy to inspect and explain.
- `call-package fmt.*` in the bytecode dump makes the new execution seam obvious instead of hiding it inside builtin dispatch.
- The missing-import failure is direct and appears during `check`, so users get feedback before execution.

## Issues and Severity

- Medium: `fmt` behavior is intentionally partial, so users could still assume more Go compatibility than the current implementation actually provides.
- Low: selector expressions are only meaningful as call targets, which is fine for this slice but still a noticeable subset limitation.
- Low: import support is metadata-backed only; there is still no package graph or filesystem resolution path.

## Conclusion and Next Recommended Steps

This slice materially improves the CLI-first experience because programs can now express a real import line and package-qualified calls without collapsing the architecture. The next best step is either a composite runtime value plan or a second package-service plan that keeps extending the metadata-backed standard-library seam without mixing in full import loading yet.
