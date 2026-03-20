# Recover First Slice Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-13-28-50-recover-first-slice`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that staged builtin `recover()` works across semantic analysis, typed panic bytecode, panic-aware unwind, direct deferred recovery eligibility, CLI inspection, and repository file-size governance without claiming unsupported closure- or runtime-object-level parity.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/recover.go`
- `cargo run -- dump-ast examples/recover.go`
- `cargo run -- dump-bytecode examples/recover.go`
- `cargo run -- check examples/recover.go`
- `cargo run -- run <temp source with defer recover(); panic("boom")>`
- `cargo run -- run <temp source with deferred helper() calling recover()>`
- `cargo run -- check <temp source with recover(1)>`
- `wc -l BOOT.md docs/design/AGENTS.md docs/design/recover-first-slice.md docs/research/AGENTS.md docs/research/2026-03-20-panic-aware-unwind-first-slice.md docs/tech/runtime-values-and-builtins.md docs/tech/semantic-analysis.md docs/tech/testing-strategy.md docs/tech/vm-execution-pipeline.md docs/reports/verification/AGENTS.md docs/reports/experience/AGENTS.md docs/roadmap/milestones/M3-standard-library-and-runtime-model.md docs/roadmap/milestones/index.md docs/roadmap/plans/index.md docs/roadmap/archive/2026-03-20-13-28-50-recover-first-slice/plan.md docs/roadmap/archive/2026-03-20-13-28-50-recover-first-slice/todo.md docs/roadmap/archive/2026-03-20-13-28-50-recover-first-slice/context.md src/builtin.rs src/semantic/builtins.rs src/semantic/support.rs src/semantic/analyzer/tests_defer.rs src/bytecode/compiler/calls.rs src/bytecode/instruction.rs src/runtime/vm.rs src/runtime/vm/calls.rs src/runtime/vm/builtins.rs src/runtime/vm/unwind.rs src/runtime/vm/tests/panic.rs tests/AGENTS.md tests/cli_recover.rs tests/cli_recover_diagnostics.rs examples/recover.go`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeeded after the recover slice landed.
- `cargo test` passed with 176 unit tests plus all focused CLI suites, including 2 new VM recover tests, 6 new CLI recover happy-path tests, 2 new CLI recover diagnostic tests, and 1 new semantic fallthrough test for `panic(...)`.
- `cargo run -- run examples/recover.go` printed:
  - `<nil>`
  - `<nil>`
  - `boom`
  - `0`
  - `[103 111]`
  - `7`
  This confirms nil recovery outside panic, helper-call nil recovery, direct deferred recovery of string and `[]byte` payloads, unnamed-result zero return after recovery, and named-result preservation after recovery.
- `cargo run -- dump-ast examples/recover.go` renders `recover()` directly in ordinary functions, deferred user functions, and helper calls, keeping the staged source surface inspectable.
- `cargo run -- dump-bytecode examples/recover.go` shows `call-builtin recover 0`, `panic string`, and `panic []byte`, confirming recover stays visible as a builtin while panic payload typing stays explicit in bytecode.
- `cargo run -- check examples/recover.go` succeeded, confirming package-only validation accepts direct recovery, `panic(...)` termination analysis, and the new example surface.
- The deferred-builtin probe reports `panic: boom`, confirming `defer recover()` remains non-recovering in the staged model just like real Go.
- The helper-call probe reports:
  - `<nil>`
  - `panic: boom`
  confirming only directly deferred user-defined function frames can recover.
- The invalid-arity probe reports `builtin \`recover\` expects 0 arguments, found 1`.
- File-size checks remained within repository limits after the VM refactor and recover work: `src/runtime/vm.rs` is 911 lines, `src/runtime/vm/unwind.rs` 221, `src/runtime/vm/builtins.rs` 218, `src/runtime/vm/calls.rs` 199, and all other touched files remain below the 1000-line limit.

## Remaining Risks

- Recovered runtime panic payloads and `panic(nil)` still reenter the staged language as boxed `string` values instead of Go's concrete runtime panic object types.
- Closure-based recovery patterns and outer-local mutation through deferred closures remain unavailable because function literals and closures are still deferred.
- The current named-result recovery behavior depends on the existing explicit result-slot initialization pattern; if compiled-function metadata changes later, that inference should be revisited deliberately.
