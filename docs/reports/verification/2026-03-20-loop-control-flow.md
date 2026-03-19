# Loop Control Flow Verification

## Basic Information

- Date: `2026-03-20`
- Plan: `2026-03-20-06-12-55-for-clauses-break-continue`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged classic `for` clauses plus unlabeled `break` / `continue` across parsing, semantic analysis, bytecode lowering, VM execution, CLI inspection, and conservative return-path analysis.

## Execution Method

- Ran `cargo fmt`
- Ran `cargo test`
- Ran `cargo fmt --check`
- Ran `cargo run -- run examples/loop_control.go`
- Ran `cargo run -- dump-ast examples/loop_control.go`
- Ran `cargo run -- dump-bytecode examples/loop_control.go`
- Ran `cargo run -- check <temp-source with top-level break>`
- Ran `cargo run -- check <temp-source with switch-only continue>`
- Ran `cargo run -- check <temp-source with infinite loop that breaks before falling through>`
- Ran `wc -l` on the modified frontend, semantic, compiler, and CLI test files

## Results

- `cargo test` passes with 79 unit tests, 44 CLI diagnostic tests, and 69 CLI execution tests, including new parser, semantic, and CLI coverage for classic `for` clauses plus unlabeled `break` / `continue`.
- `cargo fmt --check` passes after the AST, lexer, parser, semantic, compiler, example, and test changes, confirming the repository remains formatted.
- `cargo run -- run examples/loop_control.go` prints:
  - `5`
  - `nova`
  This confirms classic `for` init / condition / post execution, `continue` skipping directly to the loop post step, `break` exiting the loop, `switch`-local `break` not escaping the outer loop, and `range` loop `continue` / `break` behavior on the VM path.
- `cargo run -- dump-ast examples/loop_control.go` shows `for var i int = 0; (i < 5); i = (i + 1) {`, `continue`, and `break`, keeping the staged loop-control surface explicit without reading implementation code.
- `cargo run -- dump-bytecode examples/loop_control.go` shows jump-based loop back-edges, `jump-if-false` exits, `switch$tag6`, and explicit jump targets around both loop post-step and `range` increment paths, confirming unlabeled `break` / `continue` lower into readable control-transfer points.
- The invalid path `cargo run -- check <temp-source with top-level break>` reports `` `break` requires an enclosing `for`, `range`, or `switch` ``, confirming semantic rejection before lowering.
- The invalid path `cargo run -- check <temp-source with switch-only continue>` reports `` `continue` requires an enclosing `for` or `range` loop ``, confirming `continue` does not treat `switch` as a loop target.
- The invalid path `cargo run -- check <temp-source with infinite loop that breaks before falling through>` reports `function \`helper\` must return a \`int\` on every path`, confirming termination analysis no longer treats every infinite-looking loop as non-fallthrough once modeled `break` can escape it.
- File-size checks show the modified code remains under the repository ceiling: `src/frontend/ast.rs` 735 lines, `src/frontend/parser/statements.rs` 516, `src/semantic/analyzer.rs` 453, `src/semantic/analyzer/loops.rs` 116, `src/semantic/analyzer/range.rs` 126, `src/semantic/analyzer/switches.rs` 152, `src/semantic/model.rs` 292, `src/semantic/support.rs` 287, `src/bytecode/compiler.rs` 923, `tests/cli_execution.rs` 611, and `tests/cli_diagnostics.rs` 591.

## Remaining Risks

- Classic `for` clauses still use the repository's staged simple-statement subset; labels, `++`, `--`, and general short variable declarations remain out of scope.
- `break` / `continue` are currently unlabeled only, so more advanced Go control-flow graphs still need explicit future work.
- Return-path analysis is still intentionally conservative and does not reason about broader constant expressions beyond omitted or literal-`true` loop conditions.
