# Switch Statements Verification

## Basic Information

- Date: `2026-03-20`
- Plan: `2026-03-20-05-46-33-switch-statements`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged expression `switch` statements, tagless `switch`, shared control-flow headers, duplicate-clause diagnostics, and readable lowering across parsing, semantic analysis, bytecode generation, VM execution, and CLI inspection.

## Execution Method

- Ran `cargo fmt`
- Ran `cargo test`
- Ran `cargo fmt --check`
- Ran `cargo run -- run examples/switch_statements.go`
- Ran `cargo run -- dump-ast examples/switch_statements.go`
- Ran `cargo run -- dump-bytecode examples/switch_statements.go`
- Ran `cargo run -- check <temp-source with leaked switch-header binding>`
- Ran `cargo run -- check <temp-source with duplicate switch default>`
- Ran `wc -l` on the modified parser, AST, semantic, compiler, and CLI test files

## Results

- `cargo test` passes with 74 unit tests, 41 CLI diagnostic tests, and 65 CLI execution tests, including new parser, semantic, and CLI coverage for staged `switch` statements and shared control-flow headers.
- `cargo fmt --check` passes after the lexer, AST, parser, semantic, compiler, example, and test changes, confirming the repository remains formatted.
- `cargo run -- run examples/switch_statements.go` prints:
  - `3`
  - `two`
  - `probe`
  - `done`
  - `go`
  This confirms shared-header comma-ok map lookups work in tagless switches, expression switches match the correct clause, expression-statement headers execute before clause dispatch, and assignment-style headers feed later tag matching.
- `cargo run -- dump-ast examples/switch_statements.go` shows `switch value, ok := counts["nova"]; {`, `case ok:`, `switch score {`, and `case 0, 1:`, keeping the staged `switch` surface explicit without reading implementation code.
- `cargo run -- dump-bytecode examples/switch_statements.go` shows explicit hidden locals `switch$tag5` / `switch$tag6`, `equal`, `jump-if-false`, and the earlier `lookup-map map[string]int`, confirming expression-switch tags are evaluated once and clause dispatch stays readable in bytecode.
- The invalid path `cargo run -- check <temp-source with leaked switch-header binding>` reports `unknown variable \`value\``, confirming switch-header bindings do not leak past the enclosing statement.
- The invalid path `cargo run -- check <temp-source with duplicate switch default>` reports `switch statement may only contain one \`default\` clause`, confirming duplicate defaults fail during semantic analysis before lowering.
- File-size checks show the modified code remains under the repository ceiling: `src/frontend/parser.rs` 650 lines, `src/frontend/parser/statements.rs` 393, `src/frontend/ast.rs` 659, `src/semantic/model.rs` 270, `src/semantic/analyzer.rs` 447, `src/semantic/analyzer/ifs.rs` 117, `src/semantic/analyzer/switches.rs` 150, `src/bytecode/compiler.rs` 703, `tests/cli_execution.rs` 576, and `tests/cli_diagnostics.rs` 552.

## Remaining Risks

- Switch support is still intentionally staged: there is no type switch, `fallthrough`, `break`, or `continue`.
- The current duplicate-case diagnostic only covers the currently modeled scalar literal cases, not broader constant expressions.
- Tagless switch lowering is readable but still jump-heavy; later control-flow work may want a small shared clause-lowering helper if `break` or `fallthrough` arrives.
