# Map Comma-Ok Lookups and Literal Diagnostics Verification

## Basic Information

- Date: `2026-03-20`
- Plan: `2026-03-20-05-11-15-map-comma-ok-and-literal-diagnostics`
- Milestone: `M3-standard-library-and-runtime-model`

## Validation Goal

Verify that `nova-go` now supports staged comma-ok `map` lookup statements and duplicate constant-key diagnostics across parsing, semantic analysis, bytecode lowering, VM execution, and CLI inspection, while keeping modified source files under the repository size limit.

## Execution Method

- Ran `cargo fmt`
- Ran `cargo test`
- Ran `cargo fmt --check`
- Ran `cargo run -- run examples/map_lookup.go`
- Ran `cargo run -- dump-ast examples/map_lookup.go`
- Ran `cargo run -- dump-bytecode examples/map_lookup.go`
- Ran `cargo run -- check <temp-source with duplicate "go" map literal keys>`
- Ran `cargo run -- check <temp-source with value, ok := values[0]>`
- Ran `cargo run -- check <temp-source with no new variables on comma-ok :=>`

## Results

- `cargo test` passes with 68 unit tests, 36 CLI diagnostic tests, and 58 CLI execution tests, including new parser, semantic, VM, and CLI coverage for staged comma-ok lookup statements, short-declaration freshness, and duplicate constant-key failures.
- `cargo fmt --check` passes after the parser, semantic, compiler, runtime, example, and test changes, confirming the repository remains formatted.
- `cargo run -- run examples/map_lookup.go` prints:
  - `0 false`
  - `3 true`
  - `false`
  - `2 true`
  This confirms nil-map comma-ok reads produce zero-plus-false, populated maps report true for present keys, blank left-hand-side bindings discard the value cleanly, and same-block short redeclaration works when at least one new name exists.
- `cargo run -- dump-ast examples/map_lookup.go` shows `value, ok := counts["nova"]`, `value, ok = counts["nova"]`, and `_, ok = counts["missing"]`, keeping the staged statement form explicit in the source-oriented debug surface.
- `cargo run -- dump-bytecode examples/map_lookup.go` shows `lookup-map map[string]int` plus the expected local stores and discards, confirming comma-ok lowering stays readable instead of disappearing into generic tuple machinery.
- The invalid path `cargo run -- check <temp-source with duplicate "go" map literal keys>` reports `map literal contains duplicate constant key "go"`, confirming duplicate constant-key diagnostics now fail during semantic analysis before runtime execution.
- The invalid path `cargo run -- check <temp-source with value, ok := values[0]>` reports `comma-ok lookup requires \`map\` target, found \`[]int\``, confirming the staged lookup surface rejects non-map right-hand sides centrally.
- The invalid path `cargo run -- check <temp-source with no new variables on comma-ok :=>` reports `comma-ok lookup \`:=\` requires at least one new named variable`, confirming same-block short-declaration freshness is enforced centrally.
- `src/runtime/vm.rs` now sits at 816 lines after extracting reusable VM support helpers into `src/runtime/vm/support.rs`, so the runtime stays below the repository size ceiling despite the new lookup instruction.

## Remaining Risks

- Comma-ok lookup is still intentionally staged as a statement-only surface; `if value, ok := m[key]; ok { ... }` and general tuple expressions remain deferred.
- Duplicate-key diagnostics currently cover the scalar literal-key forms modeled directly in the AST; broader constant-expression detection still needs a future constant-evaluation pass.
- Map iteration order remains deterministic for debugability because runtime storage is backed by sorted keys, which intentionally differs from real Go's unspecified order.
