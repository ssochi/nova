# Type Assertions First Slice Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-13-55-46-type-assertions-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that staged single-result `x.(T)` assertions work across parsing, semantic analysis, explicit bytecode lowering, runtime interface execution, CLI inspection, and repository file-size governance without claiming comma-ok assertions, type switches, or non-empty-interface parity.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/type_assertions.go`
- `cargo run -- dump-ast examples/type_assertions.go`
- `cargo run -- dump-bytecode examples/type_assertions.go`
- `cargo run -- check examples/type_assertions.go`
- `cargo run -- run <temp source with var value any; println(value.(string))>`
- `cargo run -- run <temp source with var value any = "go"; println(value.([]byte))>`
- `cargo run -- check <temp source with var value int = 7; println(value.(int))>`
- `cargo run -- check <temp source with _ = value.(type)>`
- `wc -l BOOT.md docs/design/AGENTS.md docs/design/type-assertions-first-slice.md docs/research/AGENTS.md docs/research/2026-03-20-empty-interface-any-and-fmt-spread.md docs/tech/runtime-values-and-builtins.md docs/tech/semantic-analysis.md docs/tech/testing-strategy.md docs/tech/vm-execution-pipeline.md docs/reports/verification/AGENTS.md docs/reports/verification/2026-03-20-type-assertions-first-slice.md docs/reports/experience/AGENTS.md docs/reports/experience/2026-03-20-type-assertions-first-slice-playtest.md docs/roadmap/milestones/M3-standard-library-and-runtime-model.md docs/roadmap/milestones/index.md docs/roadmap/plans/index.md docs/roadmap/archive/2026-03-20-13-55-46-type-assertions-first-slice/plan.md docs/roadmap/archive/2026-03-20-13-55-46-type-assertions-first-slice/todo.md docs/roadmap/archive/2026-03-20-13-55-46-type-assertions-first-slice/context.md examples/type_assertions.go src/frontend/ast.rs src/frontend/parser.rs src/frontend/parser/tests_type_assertions.rs src/semantic/model.rs src/semantic/analyzer.rs src/semantic/analyzer/expressions.rs src/semantic/analyzer/interfaces.rs src/semantic/analyzer/tests_type_assertions.rs src/bytecode/compiler.rs src/bytecode/instruction.rs src/runtime/vm.rs src/runtime/vm/interfaces.rs src/runtime/vm/tests.rs src/runtime/vm/tests/assertions.rs tests/AGENTS.md tests/cli_type_assertions.rs tests/cli_type_assertions_diagnostics.rs`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeeded after the type-assertion slice landed.
- `cargo test` passed with 184 library/unit tests plus the full CLI integration suite, including 2 new parser tests, 2 new semantic tests, 4 new VM assertion tests, 6 new CLI assertion happy-path/runtime-error tests, and 2 new CLI diagnostic tests.
- `cargo run -- run examples/type_assertions.go` printed:
  - `go`
  - `true 0`
  - `true`
  This confirms concrete string assertion success, typed-nil `[]byte` assertion preservation, and the staged `value.(any)` success rule over a non-nil boxed payload.
- `cargo run -- dump-ast examples/type_assertions.go` renders `text.(string)`, `payload.([]byte)`, and `count.(any)` explicitly, keeping the new source surface inspectable.
- `cargo run -- dump-bytecode examples/type_assertions.go` shows `type-assert string`, `type-assert []byte`, and `type-assert any`, confirming assertions stay visible through lowering.
- `cargo run -- check examples/type_assertions.go` succeeded, confirming package-only validation accepts the new staged assertion surface.
- The nil-interface probe reports `panic: interface conversion: interface {} is nil, not string`.
- The mismatched-dynamic-type probe reports `panic: interface conversion: interface {} is string, not []byte`.
- The non-interface operand probe reports `type assertion requires interface operand, found \`int\``.
- The `.(type)` probe reports `type switches are not supported at 5:13 (found \`identifier(type)\`)`.
- File-size checks remained within repository limits after the feature and analyzer split: `src/frontend/ast.rs` is 940 lines, `src/frontend/parser.rs` 853, `src/semantic/analyzer/expressions.rs` 907, `src/bytecode/compiler.rs` 943, `src/runtime/vm.rs` 913, and all other touched files remain below the 1000-line limit.

## Remaining Risks

- Comma-ok assertions such as `value, ok := x.(T)` and type switches still remain deliberately deferred; future work must keep them explicit on top of the new assertion seam instead of introducing tuple runtime values.
- Interface-conversion panic text is Go-like but not byte-for-byte identical for every future runtime type spelling, especially once richer interface hierarchies or additional runtime categories arrive.
- The current runtime still only models the empty interface, so assertions involving method-bearing interfaces remain out of scope until non-empty interface semantics exist.
