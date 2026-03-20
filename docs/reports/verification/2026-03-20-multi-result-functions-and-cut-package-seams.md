# Multi-Result Functions and Cut Package Seams Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-08-14-13-multi-result-functions-cut-seams`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports unnamed multi-result function signatures, staged multi-binding `:=` / `=`, direct multi-result return forwarding, and the new `strings.Cut` / `bytes.Cut` package seams across parsing, semantic analysis, bytecode lowering, VM execution, and CLI diagnostics.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/multi_results.go`
- `cargo run -- dump-ast examples/multi_results.go`
- `cargo run -- dump-bytecode examples/multi_results.go`
- `cargo run -- check <temp source with println(pair())>`
- `find src tests docs examples -type f \( -name '*.rs' -o -name '*.md' -o -name '*.go' \) -print0 | xargs -0 wc -l | sort -n | tail -n 20`

## Results

- `cargo test` passed after adding parser, semantic, package-contract, and CLI coverage for the new multi-result surface; the full suite completed successfully.
- `cargo run -- run examples/multi_results.go` produced:
  - `nova go true`
  - `nova  false`
  - `vm loop true`
  - `vm true false`
  - `alpha 4`
  - `alpha 0`
- `dump-ast` renders multi-result signatures such as `func splitTag(value string) (string, string, bool)` plus staged multi-binding statements like `head, tail, found := splitTag("nova-go")`.
- `dump-bytecode` renders explicit multi-result function metadata and package calls, including `returns=string, string, bool`, `call-package strings.Cut 2`, and `call-package bytes.Cut 2`.
- The invalid CLI path `println(pair())` fails with `call to \`pair\` produces \`(int, int)\` and cannot be used in a single-value expression`, confirming unsupported single-value contexts stay targeted.
- Modified file sizes remain within the repository limit; the largest source file after this slice is `src/runtime/vm.rs` at 981 lines.

## Remaining Risks

- Multi-result calls are still intentionally staged: they do not yet flow through general call-argument forwarding, `var` declarations with multiple names, or broader tuple-like expression positions.
- `bytes.Cut` now preserves the staged nil-vs-allocated distinction for the not-found `after` result, but wider slice aliasing and mutation-heavy package behavior still needs more package/runtime coverage.
- The VM file is now close to the 1000-line ceiling, so the next runtime-heavy slice should split helpers early.
