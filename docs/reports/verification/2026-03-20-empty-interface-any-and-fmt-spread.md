# Empty Interface Any and Fmt Spread Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-12-55-53-empty-interface-any-and-fmt-spread`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that staged empty-interface `any` / `interface{}` support, explicit boxing bytecode, nil-interface runtime behavior, interface-aware equality, and `fmt` `[]any...` spread all work together across parsing, semantic analysis, bytecode inspection, VM execution, and repository file-size governance.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/empty_interface_any.go`
- `cargo run -- dump-ast examples/empty_interface_any.go`
- `cargo run -- dump-bytecode examples/empty_interface_any.go`
- `cargo run -- check examples/empty_interface_any.go`
- `cargo run -- check <temp source with fmt.Println(1, args...)>`
- `cargo run -- check <temp source with fmt.Println(args...) where args is []string>`
- `cargo run -- run <temp source with var value any = []int{1}; println(value == value)>`
- `wc -l BOOT.md docs/design/AGENTS.md docs/design/empty-interface-any-and-fmt-spread.md docs/research/AGENTS.md docs/research/2026-03-20-empty-interface-any-and-fmt-spread.md docs/reports/verification/AGENTS.md docs/reports/experience/AGENTS.md docs/roadmap/milestones/M3-standard-library-and-runtime-model.md docs/roadmap/milestones/index.md docs/roadmap/plans/index.md docs/roadmap/archive/2026-03-20-12-55-53-empty-interface-any-and-fmt-spread/plan.md docs/roadmap/archive/2026-03-20-12-55-53-empty-interface-any-and-fmt-spread/todo.md docs/roadmap/archive/2026-03-20-12-55-53-empty-interface-any-and-fmt-spread/context.md docs/tech/runtime-values-and-builtins.md docs/tech/testing-strategy.md docs/tech/vm-execution-pipeline.md src/frontend/token.rs src/frontend/lexer.rs src/frontend/signature.rs src/frontend/parser.rs src/frontend/parser/tests.rs src/frontend/parser/tests_any_interface.rs src/semantic/model.rs src/semantic/support.rs src/semantic/packages.rs src/semantic/analyzer/tests.rs src/semantic/analyzer/expressions.rs src/bytecode/instruction.rs src/bytecode/compiler.rs src/bytecode/compiler/types.rs src/runtime/value.rs src/runtime/vm.rs src/runtime/vm/interfaces.rs src/runtime/vm/support.rs src/runtime/vm/packages.rs src/runtime/vm/builtins.rs src/runtime/vm/tests.rs src/runtime/vm/tests/any.rs tests/AGENTS.md tests/cli_any_interface.rs tests/cli_any_interface_diagnostics.rs examples/empty_interface_any.go`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeeded after the empty-interface slice landed.
- `cargo test` passed with 171 unit tests plus all focused CLI suites, including 2 new VM interface tests, 5 new CLI happy-path interface tests, and 2 new CLI diagnostic tests.
- `cargo run -- run examples/empty_interface_any.go` printed:
  - `true`
  - `false`
  - `true`
  - `boom`
  - `go 7 <nil>`
  This confirms nil-interface checks, boxed typed-nil distinction, boxed scalar equality, interface printing, and `fmt.Println(args...)` over `[]any`.
- `cargo run -- dump-ast examples/empty_interface_any.go` renders both `any` and `interface{}` syntax directly, keeping the staged source surface inspectable.
- `cargo run -- dump-bytecode examples/empty_interface_any.go` shows explicit `push-nil-interface`, `box-any string`, `box-any []byte`, `box-any int`, and `call-package-spread fmt.Println 0`, confirming the feature is not hidden in ad hoc runtime coercions.
- `cargo run -- check examples/empty_interface_any.go` succeeded, confirming package-only validation accepts the staged empty-interface and `fmt` spread surface.
- The invalid spread-prefix probe reports: `package function \`fmt.Println\` with \`...\` requires 0 fixed arguments before the spread value, found 1`.
- The invalid non-`[]any` spread probe reports: `spread argument in call to \`fmt.Println\` requires \`[]any\`, found \`[]string\``.
- The runtime equality panic probe reports: `panic: runtime error: comparing uncomparable interface value`, confirming the staged interface-equality path keeps that failure explicit instead of silently returning a wrong result.
- File-size checks remained within repository limits after the new slice: `src/frontend/parser/tests.rs` is 992 lines, `src/runtime/vm.rs` 984, `src/runtime/vm/tests.rs` 965, `src/bytecode/compiler.rs` 934, and all other touched files stay below the 1000-line limit.

## Remaining Risks

- The current interface equality slice is intentionally narrow for direct interface-vs-concrete comparisons; scalar cases are covered, but broader comparable runtime categories still need deliberate runtime-type metadata before being claimed.
- Method-bearing interfaces, type assertions, type switches, and `recover` remain deferred.
- `fmt` spread support is intentionally limited to staged `[]any` calls with no extra prefix arguments before the spread value.
