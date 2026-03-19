# Strings Package Contracts Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-01-17-51-strings-package-contracts`

## Validation Goal

Verify that `nova-go` now supports a second metadata-backed imported package through typed `strings` package contracts, while keeping automated coverage and CLI-facing inspection surfaces coherent.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- dump-tokens examples/strings_package.go`
- `cargo run -- dump-ast examples/strings_package.go`
- `cargo run -- dump-bytecode examples/strings_package.go`
- `cargo run -- check examples/strings_package.go`
- `cargo run -- run examples/strings_package.go`
- `cargo run -- check <temp-invalid-strings-join-source>`

## Results

- `cargo test` passes with unit coverage for typed package validation and VM package execution plus CLI coverage for the new `strings` example and diagnostics.
- `cargo fmt` and `cargo fmt --check` both succeed with the current local toolchain; no environment repair was needed in this round.
- `cargo run -- dump-tokens examples/strings_package.go` shows both `import "strings"` and selector tokens for `Join`, `Repeat`, `Contains`, and `HasPrefix`, confirming the frontend surface remains visible at the CLI boundary.
- `cargo run -- dump-ast examples/strings_package.go` renders the imported package program with nested `strings` calls inside slice literals and `fmt.Println` calls, confirming the AST surface stays readable.
- `cargo run -- dump-bytecode examples/strings_package.go` shows `call-package strings.Join 2`, `call-package strings.Repeat 2`, `call-package strings.Contains 2`, and `call-package strings.HasPrefix 2`, confirming lowering reuses the shared package-call instruction.
- `cargo run -- check examples/strings_package.go` succeeds, proving the semantic layer accepts the supported `strings` contracts without requiring execution.
- `cargo run -- run examples/strings_package.go` prints:
  - `nova-gogo-vm`
  - `true`
  - `true`
- An invalid source that calls `strings.Join("oops", ",")` is rejected during semantic analysis with `argument 1 in call to \`strings.Join\` requires \`[]string\`, found \`string\``, proving typed package diagnostics are working.

## Remaining Risks

- The `strings` seam is intentionally narrow and does not yet cover most of the real package surface.
- Real Go uses panic semantics for some `strings.Repeat` failures, while the current VM maps those cases to runtime errors.
- Imported packages are still metadata-backed only; there is no filesystem package graph yet.
