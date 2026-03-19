# If Statement Headers Verification

## Basic Information

- Date: `2026-03-20`
- Plan: `2026-03-20-05-29-16-if-initializers-else-if`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged `if` statement initializers and `else if` chains across parsing, semantic analysis, bytecode lowering, VM execution, and CLI inspection while preserving explicit header scoping and repository file-size limits.

## Execution Method

- Ran `cargo fmt`
- Ran `cargo test`
- Ran `cargo fmt --check`
- Ran `cargo run -- run examples/if_headers.go`
- Ran `cargo run -- dump-ast examples/if_headers.go`
- Ran `cargo run -- dump-bytecode examples/if_headers.go`
- Ran `cargo run -- check <temp-source with leaked if-header binding>`
- Ran `cargo run -- check <temp-source with missing if-header semicolon>`
- Ran `wc -l` on the modified parser, AST, semantic, compiler, and CLI test files

## Results

- `cargo test` passes with 71 unit tests, 38 CLI diagnostic tests, and 61 CLI execution tests, including new parser, semantic, and CLI coverage for staged `if` initializers, `else if` chains, and header-scope diagnostics.
- `cargo fmt --check` passes after the AST, parser, semantic, compiler, example, and test changes, confirming the repository remains formatted.
- `cargo run -- run examples/if_headers.go` prints:
  - `3 true`
  - `2`
  - `probe`
  - `7`
  This confirms comma-ok `map` lookups now work directly in `if` headers, assignment-style headers execute before their conditions, expression-statement headers run on the real CLI path, and `else if` control flow reaches the expected branch.
- `cargo run -- dump-ast examples/if_headers.go` shows `if value, ok := counts["nova"]; ok {`, `else if fallback = counts["fallback"]; (fallback > 0) {`, and `else if var ready bool = false; ready {`, keeping the staged header surface readable without source-code inspection.
- `cargo run -- dump-bytecode examples/if_headers.go` shows the header initializer instructions before each `jump-if-false`, including `lookup-map map[string]int`, `store-local` for header bindings, and nested branch jumps for the `else if` chain.
- The invalid path `cargo run -- check <temp-source with leaked if-header binding>` reports `unknown variable \`ok\``, confirming header-bound names do not leak past the enclosing `if` statement.
- The invalid path `cargo run -- check <temp-source with missing if-header semicolon>` reports `expected \`;\` at 5:31 (found \`{\`)`, confirming the staged parser requires explicit `if`-header separation.
- File-size checks show the modified code remains under the repository ceiling: `src/frontend/parser.rs` 916 lines, `src/frontend/ast.rs` 598, `src/semantic/analyzer.rs` 445, `src/semantic/analyzer/ifs.rs` 116, `src/bytecode/compiler.rs` 624, `tests/cli_execution.rs` 538, and `tests/cli_diagnostics.rs` 513.

## Remaining Risks

- Statement-header support is still intentionally staged to `if`; `switch` and richer `for` headers remain deferred.
- General short variable declarations are still not implemented beyond the existing staged forms, so `if value := 1; value > 0 { ... }` remains unsupported.
- `dump-ast` currently renders `else` on a fresh line before nested `if` text, which is readable but not yet fully Go-style pretty-printing.
