# Map Literals and Delete Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-04-07-11-map-literals-delete`

## Validation Goal

Verify that `nova-go` now supports staged `map[K]V{...}` literals and builtin `delete(map, key)` across parsing, semantic analysis, bytecode lowering, VM execution, and CLI inspection without regressing the existing compiler surface.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/map_literals.go`
- `cargo run -- dump-ast examples/map_literals.go`
- `cargo run -- dump-bytecode examples/map_literals.go`
- `cargo run -- check <temp-source with map literal value type mismatch>`
- `cargo run -- check <temp-source with delete key type mismatch>`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeed with the current local toolchain.
- `cargo test` now passes with 55 unit tests, 28 CLI diagnostic tests, and 47 CLI execution tests, including new parser, semantic, builtin, VM, and CLI coverage for map literals, empty-map literals, delete semantics, and duplicate-key runtime behavior.
- `cargo run -- run examples/map_literals.go` prints:
  - `1 3 0`
  - `0`
  - `0 0`
- The run output proves three behaviors together: keyed map literals allocate writable non-nil maps, `delete` removes present keys without breaking later lookups, and deleting from both allocated-empty and nil maps is a no-op.
- `cargo run -- dump-ast examples/map_literals.go` renders `var counts = map[string]int{"nova": 3, "go": 2}`, `delete(counts, "go")`, and `var empty = map[string]int{}`, confirming the new source-facing syntax remains readable through the CLI.
- `cargo run -- dump-bytecode examples/map_literals.go` shows `build-map map[string]int 2`, `build-map map[string]int 0`, `call-builtin delete 2`, and `push-nil-map`, confirming literal construction and deletion remain explicit at the VM-facing debug layer.
- The failure path `cargo run -- check <temp-source with map literal value type mismatch>` reports `map literal value 1 requires \`int\`, found \`string\``, confirming map literal entries are type-checked during semantic analysis.
- The failure path `cargo run -- check <temp-source with delete key type mismatch>` reports `argument 2 in call to builtin \`delete\` requires \`string\`, found \`int\``, confirming builtin `delete` key typing is validated before runtime.

## Remaining Risks

- Real Go rejects duplicate constant keys in a single map literal, but the staged `nova-go` implementation currently keeps deterministic last-write-wins behavior instead of issuing that diagnostic.
- Map support is still intentionally staged: comma-ok lookups, `range`, nil equality, and channel/runtime adjacency remain deferred.
- Runtime map rendering remains deterministic for debugging rather than following Go's unspecified iteration order.
