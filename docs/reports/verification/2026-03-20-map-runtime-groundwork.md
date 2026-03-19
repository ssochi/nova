# Map Runtime Groundwork Verification

## Basic Information

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-03-42-30-map-runtime-groundwork`

## Validation Goal

Verify that `nova-go` now supports staged `map[K]V` runtime behavior across parsing, semantic analysis, bytecode lowering, VM execution, and CLI inspection without regressing the existing runtime slices.

## Execution Method

- `cargo fmt`
- `cargo test`
- `cargo fmt --check`
- `cargo run -- run examples/maps.go`
- `cargo run -- dump-ast examples/maps.go`
- `cargo run -- dump-bytecode examples/maps.go`
- `cargo run -- check <temp-source with map[[]int]int>`
- `cargo run -- run <temp-source with nil-map assignment>`

## Results

- `cargo fmt`, `cargo test`, and `cargo fmt --check` all succeed with the current local toolchain.
- `cargo test` now passes with 46 unit tests, 26 CLI diagnostic tests, and 44 CLI execution tests, including new parser, semantic, VM, and CLI coverage for map types, map allocation, nil-map behavior, and map-specific diagnostics.
- `cargo run -- run examples/maps.go` prints:
  - `0 0`
  - `2 3 5 0`
  - `ready 1`
- The run output proves three behaviors together: typed nil maps report zero length and zero-valued reads, `make(map[K]V, hint)` produces a writable map with visible entry growth, and non-string comparable keys also work in the staged runtime slice.
- `cargo run -- dump-ast examples/maps.go` renders `var counts map[string]int`, `counts = make(map[string]int, 2)`, and `var labels = make(map[bool]string)` directly, confirming the new type and allocation syntax remains readable at the source-oriented CLI layer.
- `cargo run -- dump-bytecode examples/maps.go` shows `push-nil-map`, `make-map map[string]int hint=explicit`, `index-map map[string]int`, and `set-map-index`, confirming the map execution path is explicit in VM-facing debug output.
- The failure path `cargo run -- check <temp-source with map[[]int]int>` reports `variable \`counts\` requires a comparable map key type, found \`[]int\``, confirming unsupported key types are rejected during semantic analysis.
- The failure path `cargo run -- run <temp-source with nil-map assignment>` reports `assignment to entry in nil map`, confirming the VM surfaces nil-map writes as runtime errors instead of silently materializing storage.

## Remaining Risks

- Map support is still intentionally staged: map literals, `delete`, comma-ok lookups, `range`, and iteration-order behavior remain deferred.
- Runtime map storage currently uses deterministic sorted rendering for debugging rather than Go's unspecified map iteration behavior.
- Nil-map writes currently map to runtime errors because the VM does not model Go panic or recover yet.
