# Type Switches and Comma-Ok Assertions Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-14-17-12-type-switches-and-comma-ok-assertions`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that staged comma-ok type assertions and the first empty-interface type-switch slice work across parsing, semantic analysis, explicit bytecode lowering, runtime interface execution, CLI inspection, and file-size governance without implying non-empty interfaces or tuple runtime values.

## Execution Method

- `cargo test --lib`
- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/type_switches_and_comma_ok.go`
- `cargo run -- dump-ast examples/type_switches_and_comma_ok.go`
- `cargo run -- dump-bytecode examples/type_switches_and_comma_ok.go`
- `cargo run -- check examples/type_switches_and_comma_ok.go`
- `cargo run -- run <temp source with typed-nil []byte boxed in any plus comma-ok assertion and type switch>`
- `cargo run -- check <temp source with non-interface type-switch guard>`
- `cargo run -- check <temp source with duplicate type-switch case>`
- `cargo run -- check <temp source with blank type-switch binding>`
- `wc -l BOOT.md docs/design/AGENTS.md docs/design/type-switches-and-comma-ok-assertions.md docs/research/AGENTS.md docs/research/2026-03-20-empty-interface-any-and-fmt-spread.md docs/tech/runtime-values-and-builtins.md docs/tech/semantic-analysis.md docs/tech/testing-strategy.md docs/tech/vm-execution-pipeline.md docs/reports/verification/AGENTS.md docs/reports/experience/AGENTS.md docs/roadmap/milestones/M3-standard-library-and-runtime-model.md docs/roadmap/milestones/index.md docs/roadmap/plans/index.md docs/roadmap/archive/2026-03-20-14-17-12-type-switches-and-comma-ok-assertions/plan.md docs/roadmap/archive/2026-03-20-14-17-12-type-switches-and-comma-ok-assertions/todo.md docs/roadmap/archive/2026-03-20-14-17-12-type-switches-and-comma-ok-assertions/context.md examples/type_switches_and_comma_ok.go src/bytecode/compiler.rs src/bytecode/compiler/simple_statements.rs src/bytecode/compiler/switches.rs src/bytecode/instruction.rs src/frontend/ast.rs src/frontend/ast/render.rs src/frontend/parser.rs src/frontend/parser/statements.rs src/frontend/parser/type_switches.rs src/frontend/parser/tests_type_switches.rs src/runtime/vm.rs src/runtime/vm/interfaces.rs src/runtime/vm/tests/assertions.rs src/semantic/analyzer.rs src/semantic/analyzer/ifs.rs src/semantic/analyzer/interfaces.rs src/semantic/analyzer/loops.rs src/semantic/analyzer/switches.rs src/semantic/analyzer/tests_type_switches.rs src/semantic/model.rs src/semantic/support.rs tests/AGENTS.md tests/cli_type_switches.rs tests/cli_type_switches_diagnostics.rs`

## Results

- `cargo test --lib`, `cargo test`, and `cargo fmt --check` all succeeded after the interface-consumption slice landed.
- The automated suite passed with 189 library/unit tests and the full CLI integration suite, including new parser, semantic, VM, and focused CLI coverage for comma-ok assertions plus type switches.
- `cargo run -- run examples/type_switches_and_comma_ok.go` printed:
  - `false`
  - `true nova`
  - `bytes nova`
  - `false`
  - `nil true`
  - `multi true`
  This confirms failed comma-ok assertions return `false`, successful assertions preserve payloads, typed `[]byte` dispatch wins over `case nil`, nil-interface type switches reach `case nil`, and multi-type cases keep a usable interface binding.
- `cargo run -- dump-ast examples/type_switches_and_comma_ok.go` renders `word, ok := boxed.(string)`, `bytes, ok := boxed.([]byte)`, and `switch current := boxed.(type)` explicitly, keeping both new source surfaces inspectable.
- `cargo run -- dump-bytecode examples/type_switches_and_comma_ok.go` shows explicit `type-assert-ok string`, `type-assert-ok []byte`, and `type-assert-ok bool` instructions plus hidden type-switch locals, confirming non-panicking assertion checks and type-switch lowering stay readable.
- `cargo run -- check examples/type_switches_and_comma_ok.go` succeeded, confirming package-only validation accepts the new staged surface.
- The typed-nil probe printed `true` then `typed`, confirming boxed typed-nil `[]byte` values still count as successful comma-ok assertions and match `case []byte` instead of `case nil`.
- The non-interface guard probe reports `type switch guard requires interface operand, found \`int\``.
- The duplicate-case probe reports `duplicate case int in type switch`.
- The blank-binding probe reports `type switch guard requires a named identifier before \`:=\``.
- File-size checks remained within repository limits after splitting render and parser helpers: `src/frontend/ast.rs` is 550 lines, `src/frontend/ast/render.rs` 539, `src/frontend/parser.rs` 839, `src/frontend/parser/statements.rs` 943, `src/semantic/analyzer.rs` 961, `src/bytecode/compiler.rs` 877, `src/runtime/vm.rs` 913, and all other touched files remain below the 1000-line limit.

## Remaining Risks

- Type switches are still limited to empty-interface operands and staged runtime types; non-empty interfaces, method sets, and richer implementation checks remain deferred.
- `case any` currently works only through the empty-interface model; broader interface-to-interface type-switch matching still needs deliberate design.
- Comma-ok assertions remain statement-scoped in this slice; expression-level tuple contexts and comma-ok receive are still intentionally deferred.
