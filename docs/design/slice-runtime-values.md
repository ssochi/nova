# Slice Runtime Values

## Goal

Introduce the first composite runtime value for `M3` with a narrow, testable slice feature set that can support richer builtins and later standard-library work.

## Constraints

- Rust standard library only
- Preserve the current frontend -> semantic -> bytecode -> VM layering
- Keep builtin contracts centralized instead of scattering slice checks across semantic and runtime code
- Limit the scope to one resumable iteration with real CLI validation and stronger automated coverage

## Current Scope

- Recursive semantic type support for `[]T`
- Slice literals such as `[]int{1, 2}`
- Index expressions such as `values[0]`
- Builtin `append(slice, ...elements)` returning a new slice value
- Builtin `len` support for both `string` and `slice`
- Runtime rendering of slices in a Go-like `[1 2 3]` form
- Layered validation with unit tests in `src/` and CLI integration tests under `tests/`

## Deferred Scope

- Slice expressions such as `values[1:3]`
- Slice element assignment such as `values[0] = 1`
- `make`, `copy`, `cap`, nil, or backing-array semantics
- Variadic forwarding with `...`
- Full Go comparability rules beyond rejecting slice equality in the current semantic layer

## Interfaces and Extension Hooks

- `src/frontend/token.rs` and `src/frontend/parser.rs`: bracket tokens, recursive type parsing, slice literals, and index expressions
- `src/semantic/model.rs` and `src/semantic/analyzer.rs`: recursive `Type`, slice validation, and slice-aware expression checking
- `src/semantic/builtins.rs`: centralized contracts for slice-aware `len` and `append`
- `src/bytecode/instruction.rs` and `src/bytecode/compiler.rs`: `build-slice` and `index` instructions
- `src/runtime/value.rs` and `src/runtime/vm.rs`: slice runtime values, rendering, indexing, and builtin execution
- `tests/support/` plus CLI integration files: reusable helpers and layered automated coverage
