# Panic-Aware Unwind First Slice Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-12-28-16-panic-aware-unwind-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that staged builtin `panic`, panic-aware deferred unwinding, selected runtime-trap escalation, CLI-visible panic bytecode, and repository file-size governance all hold together without claiming unsupported `recover` behavior.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/panic.go`
- `cargo run -- dump-ast examples/panic.go`
- `cargo run -- dump-bytecode examples/panic.go`
- `cargo run -- check examples/panic.go`
- `cargo run -- check <temp source with panic()>`
- `cargo run -- run <temp source with deferred nil-map assignment panic>`
- `cargo run -- dump-bytecode <temp source with defer panic(nil); panic(nil)>`
- `wc -l BOOT.md docs/design/AGENTS.md docs/design/panic-aware-unwind-first-slice.md docs/research/AGENTS.md docs/research/2026-03-20-panic-aware-unwind-first-slice.md docs/roadmap/milestones/M3-standard-library-and-runtime-model.md docs/roadmap/milestones/index.md docs/roadmap/plans/index.md docs/roadmap/plans/2026-03-20-12-28-16-panic-aware-unwind-first-slice/plan.md docs/roadmap/plans/2026-03-20-12-28-16-panic-aware-unwind-first-slice/todo.md docs/roadmap/plans/2026-03-20-12-28-16-panic-aware-unwind-first-slice/context.md docs/tech/runtime-values-and-builtins.md docs/tech/testing-strategy.md docs/tech/vm-execution-pipeline.md src/builtin.rs src/semantic/builtins.rs src/bytecode/compiler.rs src/bytecode/compiler/calls.rs src/bytecode/instruction.rs src/runtime/vm.rs src/runtime/vm/builtins.rs src/runtime/vm/unwind.rs src/runtime/vm/tests.rs src/runtime/vm/tests/panic.rs tests/AGENTS.md tests/cli_panic.rs tests/cli_panic_diagnostics.rs examples/panic.go`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeeded after the new panic slice landed.
- `cargo test` passed with 166 unit tests and all focused CLI suites, including 2 new VM panic tests, 7 new CLI panic execution tests, and 2 new CLI panic diagnostic tests.
- `cargo run -- run examples/panic.go` printed:
  - `body`
  - `inner defer`
  - `outer defer`
  - `panic: boom`
  This confirms explicit builtin panic now unwinds through deferred calls and preserves buffered CLI output.
- `cargo run -- dump-ast examples/panic.go` renders `panic("boom")` directly, keeping the panic entry visible in the source-facing inspection surface.
- `cargo run -- dump-bytecode examples/panic.go` shows an explicit `panic` instruction after `defer-builtin println 1`, confirming the feature is not hidden behind a generic builtin call.
- `cargo run -- check examples/panic.go` succeeded, confirming package-level validation accepts the staged panic surface without requiring runtime execution.
- The invalid `check` probe reports the expected direct diagnostic: `builtin \`panic\` expects 1 arguments, found 0`.
- The deferred nil-map runtime-trap probe printed:
  - `inner`
  - `outer`
  - `panic: assignment to entry in nil map`
  This confirms the selected runtime-trap path now shares panic-aware deferred unwinding instead of aborting before defers run.
- The `panic(nil)` bytecode probe shows both `defer-panic-nil` and `panic-nil`, confirming the nil-special case remains explicit in bytecode and does not disappear into hidden runtime markers.
- File-size checks remained within repository limits after the new slice: `src/bytecode/compiler.rs` is 964 lines, `src/runtime/vm.rs` 965, `src/runtime/vm/tests.rs` 964, and all other touched files stay well below the 1000-line limit.

## Remaining Risks

- `recover` is still intentionally unsupported because the current type system cannot carry panic payloads through an `interface{}` / `any`-like surface.
- Only a focused subset of runtime traps now enter the panic-aware unwind path; other runtime failures still report directly without pretending to be Go panics.
- CLI panic formatting is intentionally lightweight and does not yet model real Go stack traces or richer runtime panic value formatting.
