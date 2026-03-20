# Call-Argument Multi-Result Forwarding Verification

## Basic Information

- Date: `2026-03-20`
- Related plan: `2026-03-20-08-55-11-call-argument-multi-result-forwarding`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged multi-result call forwarding when a single call supplies the whole argument list, plus the new `strings.CutPrefix` / `strings.CutSuffix` / `bytes.CutPrefix` / `bytes.CutSuffix` seams across semantic analysis, bytecode lowering, VM execution, and CLI diagnostics.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/call_forwarding.go`
- `cargo run -- dump-ast examples/call_forwarding.go`
- `cargo run -- dump-bytecode examples/call_forwarding.go`
- `cargo run -- check <temp source with take(1, pair())>`
- `find src tests docs examples -type f \( -name '*.rs' -o -name '*.md' -o -name '*.go' \) -print0 | xargs -0 wc -l | sort -n | tail -n 20`

## Results

- `cargo test` passed after adding checked-model, package-contract, runtime, CLI, and unit coverage for call forwarding and the new `Cut*` seams.
- `cargo fmt --check` passed after formatting.
- `cargo run -- run examples/call_forwarding.go` produced:
  - `nova:go`
  - `nova|go|true`
  - `go:true`
  - `nova:true`
  - `go:true`
  - `nova:true`
  - `nova  false`
  - `[110 111 118 97] false`
- `dump-ast` keeps the source path readable, including wrapper calls such as `fmt.Println(joinPair(pair()))` and the new `strings.CutPrefix(...)` / `bytes.CutSuffix(...)` package seams.
- `dump-bytecode` shows the staged forwarding path explicitly, including `call-function 0 0`, `call-function 1 2`, `call-function 2 3`, `call-package fmt.Println 3`, `call-package strings.CutPrefix 2`, and `call-package bytes.CutSuffix 2`.
- The invalid CLI path `take(1, pair())` fails with `call to \`pair\` produces \`(int, int)\` and cannot be used in a single-value expression`, confirming prefixed arguments still stay outside the staged forwarding rule.
- Modified file sizes remain within the repository limit; `src/runtime/vm.rs` is now 742 lines after splitting builtin/package dispatch helpers into submodules.

## Remaining Risks

- Call forwarding is still intentionally narrow: only a single multi-result call may supply another call's full argument list, while prefixed arguments and broader tuple-like contexts remain unsupported.
- The new `CutPrefix` / `CutSuffix` seams are still narrow, metadata-backed package contracts; broader package growth will need the same explicit staging to avoid hardcoded runtime sprawl.
- Grouped parameter names in function signatures are still outside the current parser surface, so examples and diagnostics must keep parameter types explicit per name.
