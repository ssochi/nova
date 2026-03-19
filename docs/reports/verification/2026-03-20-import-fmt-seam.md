# Import Fmt Seam Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-00-35-11-import-fmt-seam`

## Validation Goal

Verify that `nova-go` now supports top-level imports, selector-based `fmt` package calls, centralized package-function contracts, and package-backed VM execution without breaking the existing CLI and builtin paths.

## Execution Method

- `rustup component add rustfmt`
- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- dump-tokens examples/imports_fmt.go`
- `cargo run -- dump-ast examples/imports_fmt.go`
- `cargo run -- dump-bytecode examples/imports_fmt.go`
- `cargo run -- check examples/imports_fmt.go`
- `cargo run -- run examples/imports_fmt.go`
- `cargo run -- check <temp-missing-import-source>`

## Results

- `cargo test` passes with twenty-five integration tests, including imported `fmt` execution, selector-call rendering, package-call bytecode dumps, and negative import/member diagnostics.
- `cargo fmt` and `cargo fmt --check` both succeed after installing `rustfmt`, so the repository now has a working formatting path on this machine.
- `cargo run -- dump-tokens examples/imports_fmt.go` shows `import`, `string("fmt")`, and `.` tokens, proving the lexer exposes the new import and selector surface.
- `cargo run -- dump-ast examples/imports_fmt.go` renders `import "fmt"` plus nested package calls such as `fmt.Print(fmt.Sprint("bytes=", len(message)))`.
- `cargo run -- dump-bytecode examples/imports_fmt.go` shows `call-package fmt.Sprint`, `call-package fmt.Println`, and `call-package fmt.Print`, confirming package-backed lowering.
- `cargo run -- check examples/imports_fmt.go` succeeds, proving package-level validation accepts the supported metadata-backed import seam.
- `cargo run -- run examples/imports_fmt.go` prints `hello, nova` and then `bytes=11`, proving imported package calls, value-returning `fmt.Sprint`, and builtin/package interoperation on the VM path.
- An invalid source that calls `fmt.Println` without importing `fmt` is rejected during semantic analysis with `package \`fmt\` is not imported`.

## Remaining Risks

- Import support is intentionally narrow and does not yet include aliases, grouped imports, or filesystem package graphs.
- `fmt` behavior is only an approximation and does not yet implement formatting verbs or Go-exact spacing behavior.
- Selector expressions are intentionally only useful as call targets; package values and richer selector semantics remain unimplemented.
- The project still lacks composite runtime values, so many real standard-library APIs remain out of reach.
