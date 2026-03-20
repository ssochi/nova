# Defer First Slice Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-11-58-27-defer-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that staged `defer` statements now work across parsing, semantic analysis, bytecode lowering, VM execution, CLI inspection, and repository file-size governance without claiming unsupported closure or panic-aware defer semantics.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/defer.go`
- `cargo run -- dump-ast examples/defer.go`
- `cargo run -- dump-bytecode examples/defer.go`
- `cargo run -- check examples/defer.go`
- `cargo run -- check <temp source with defer (println("tail"))>`
- `cargo run -- check <temp source with defer len("go")>`
- `wc -l src/frontend/ast.rs src/frontend/parser.rs src/frontend/parser/statements.rs src/frontend/parser/tests.rs src/frontend/parser/tests_defer.rs src/frontend/token.rs src/frontend/lexer.rs src/semantic/analyzer.rs src/semantic/analyzer/defer.rs src/semantic/analyzer/tests.rs src/semantic/analyzer/tests_defer.rs src/semantic/builtins.rs src/semantic/model.rs src/semantic/support.rs src/bytecode/compiler.rs src/bytecode/instruction.rs src/runtime/vm.rs src/runtime/vm/builtins.rs src/runtime/vm/packages.rs src/runtime/vm/tests.rs src/runtime/vm/tests/defer.rs tests/cli_defer.rs tests/cli_defer_diagnostics.rs tests/AGENTS.md examples/defer.go docs/research/2026-03-20-defer-first-slice.md docs/design/defer-first-slice.md docs/tech/semantic-analysis.md docs/tech/runtime-values-and-builtins.md docs/tech/vm-execution-pipeline.md docs/roadmap/archive/2026-03-20-11-58-27-defer-first-slice/plan.md docs/roadmap/archive/2026-03-20-11-58-27-defer-first-slice/todo.md docs/roadmap/archive/2026-03-20-11-58-27-defer-first-slice/context.md BOOT.md`

## Results

- `cargo test` passed with 162 unit tests plus all focused CLI suites, including 5 new defer CLI execution tests, 2 new defer CLI diagnostic tests, and a new VM defer unit test.
- `cargo fmt` and `cargo fmt --check` both succeeded; no environment repair was needed in this round.
- `cargo run -- run examples/defer.go` printed:
  - `body 9`
  - `package 1`
  - `pair`
  - `builtin`
  This confirms eager argument capture (`value` stayed `1` in the deferred package call), LIFO execution order, multi-result deferred user-call execution, and builtin/package/user defer coverage through the real CLI path.
- `cargo run -- dump-ast examples/defer.go` renders `defer println("builtin")`, `defer pair()`, and `defer fmt.Println("package", value)` directly, confirming the new statement remains visible in the source-facing inspection surface.
- `cargo run -- dump-bytecode examples/defer.go` shows `defer-builtin println 1`, `defer-function 0 0`, and `defer-package fmt.Println 2`, confirming defer lowering stays explicit rather than disappearing into synthetic tail blocks.
- `cargo run -- check examples/defer.go` succeeded, confirming package-level validation accepts the staged defer subset without requiring runtime execution.
- Invalid checks report direct staged diagnostics:
  - parenthesized defer: `expression in defer must not be parenthesized`
  - builtin statement-context violation: `builtin \`len\` is not permitted in defer statement context`
- File-size checks remained within repository limits after the new defer slice: `src/frontend/ast.rs` is 932 lines, `src/semantic/analyzer.rs` 947, `src/bytecode/compiler.rs` 957, `src/runtime/vm.rs` 931, `src/runtime/vm/tests.rs` 963, and all newly added focused files remain small.

## Remaining Risks

- The staged defer surface is intentionally limited to the currently modeled direct-call forms; closures, method values, and arbitrary function expressions remain out of scope.
- Deferred execution currently handles ordinary returns only; `panic` / `recover` and panic-triggered unwinding still need a deliberate plan.
- Builtin statement-context filtering is now explicit for `defer`, but general expression-statement compatibility remains broader than real Go outside this new path.
