# Simple Statements, Short Declarations, and Inc/Dec Verification

## Basic Information

- Date: `2026-03-20`
- Plan: `2026-03-20-06-34-01-simple-statements-incdec`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged single-expression short declarations and explicit `++` / `--` statements across parsing, semantic analysis, bytecode lowering, VM execution, CLI inspection, and user-facing diagnostics.

## Execution Method

- Ran `cargo fmt`
- Ran `cargo test`
- Ran `cargo fmt --check`
- Ran `cargo run -- run examples/simple_statements.go`
- Ran `cargo run -- dump-ast examples/simple_statements.go`
- Ran `cargo run -- dump-bytecode examples/simple_statements.go`
- Ran `cargo run -- check <temp-source with repeated short declaration>`
- Ran `cargo run -- check <temp-source with string ++ target>`
- Ran `cargo run -- check <temp-source with short declaration in for post>`
- Ran `wc -l` on the modified parser, semantic, bytecode, and CLI test files

## Results

- `cargo test` passes with 83 unit tests, 47 CLI diagnostic tests, and 73 CLI execution tests, including new parser, semantic, and CLI coverage for short declarations plus explicit inc/dec statements.
- `cargo fmt` and `cargo fmt --check` both succeed with the current local toolchain; no environment repair was required.
- `cargo run -- run examples/simple_statements.go` prints:
  - `6 3`
  - `2 1`
  - `ready`
  This confirms ordinary short declarations, `if` / `switch` headers with `:=`, classic `for i := 0; ...; i++`, local `--`, and map-index `++` all execute correctly through the real CLI path.
- `cargo run -- dump-ast examples/simple_statements.go` renders `total := 0`, `for i := 0; (i < len(values)); i++ {`, `counts["go"]++`, and `switch probe := current; {`, confirming the staged simple-statement surface remains explicit in the source-facing CLI view.
- `cargo run -- dump-bytecode examples/simple_statements.go` shows hidden locals `incdec$target7`, `incdec$index8`, and `incdec$value9` plus `push-int 1`, `add`, `subtract`, and `set-map-index`, confirming index-target inc/dec lowers through single-evaluation temporaries instead of ad hoc repeated expression evaluation.
- The invalid path with `value := 1` followed by `value := 2` reports `short declaration \`:=\` requires a new variable name, but \`value\` already exists in the current scope`, confirming the staged short-declaration freshness rule is enforced centrally.
- The invalid path with `var label string = "go"; label++` reports ``++` requires `int` or `byte`, found `string``, confirming inc/dec typing is rejected during semantic analysis.
- The invalid path with `for i := 0; i < 3; i := 1 { ... }` reports `for post statement does not support \`:=\``, confirming the classic `for` post restriction fails early with a targeted diagnostic.
- File-size checks confirm the repository remains under the 1000-line limit after the required compiler split: `src/frontend/ast.rs` 788, `src/frontend/parser/statements.rs` 610, `src/frontend/parser/tests.rs` 656, `src/semantic/analyzer.rs` 551, `src/semantic/analyzer/ifs.rs` 137, `src/semantic/analyzer/loops.rs` 132, `src/semantic/model.rs` 327, `src/bytecode/compiler.rs` 805, `src/bytecode/compiler/simple_statements.rs` 221, `tests/cli_execution.rs` 648, and `tests/cli_diagnostics.rs` 630.

## Remaining Risks

- General short declarations are still intentionally staged to one named binding plus one expression; a broader multi-result model is still missing.
- Compound assignments such as `+=` remain unsupported, so many idiomatic Go loops still require explicit reassignment even though `i++` now works.
- Byte arithmetic is still not a general expression feature; the new runtime byte add/sub path is only exercised through explicit inc/dec lowering in this round.
