# Import Aliases and Bytes Package Seam Playtest

## Basic Context

- Date: `2026-03-20`
- Plan: `2026-03-20-07-46-15-import-aliases-and-bytes-package`
- Milestone: `M3-standard-library-and-runtime-model`

## Experience Path

1. Ran `cargo run -- run examples/imports_bytes.go` to exercise grouped imports, alias imports, and the staged `bytes` functions through the real CLI entrypoint.
2. Ran `cargo run -- dump-ast examples/imports_bytes.go` to confirm grouped imports and alias bindings remain readable in the source-facing inspection path.
3. Ran `cargo run -- dump-bytecode examples/imports_bytes.go` to confirm `bytes` calls stay explicit as package-backed bytecode operations.
4. Ran `cargo run -- check` on two failure paths: unsupported dot imports and an invalid `bytes.Join` argument shape.

## Positive Experience

- Grouped imports and alias imports now feel native in the CLI instead of like a narrow bootstrap syntax; the source sample looks like ordinary Go code.
- The new `bytes` seam is useful immediately because it composes with the existing `[]byte` conversion path rather than introducing a synthetic one-off helper.
- `dump-ast` remains a strong debugging tool here because it preserves `import (...)`, the `b` alias, and the explicit `[]byte(...)` call sites without collapsing them into hidden metadata.
- `dump-bytecode` stays readable because package calls remain visible as `call-package bytes.*`, which makes the staged runtime boundary obvious.

## Issues and Severity

- Medium: import support is still deliberately narrower than real Go because dot imports, blank imports, and filesystem package graphs remain unsupported.
- Medium: `bytes` failures still surface as runtime errors instead of panic/recover behavior, so the seam is pragmatic rather than fully Go-accurate.
- Medium: broader package growth is still constrained by the missing multi-result model, which many standard-library APIs depend on.

## Conclusion and Next Recommended Steps

This round materially improves the real-program surface: ordinary grouped imports, explicit alias bindings, and a practical `bytes` seam now work end to end through parsing, semantic analysis, bytecode, and execution. The strongest next continuation is to plan the first staged multi-result model so more package seams and channel follow-up work can expand without relying on statement-specific exceptions.
