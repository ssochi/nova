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
- Explicit typed `var` declarations such as `var values []int` and `var total int`
- Slice literals such as `[]int{1, 2}`
- Index expressions such as `values[0]`
- Simple slice expressions on slices such as `values[1:3]`, `values[:2]`, and `values[1:]`
- Slice element assignment such as `values[0] = 1`
- Builtin `append(slice, ...elements)` with backing-storage reuse when capacity permits
- Builtin `len(slice)` and `cap(slice)` support plus existing `len(string)`
- Builtin `copy(dstSlice, srcSlice)` returning the copied element count with overlap-safe behavior
- Zero-value synthesis for typed locals, including nil slices plus scalar zero values
- Runtime rendering of slices in a Go-like `[1 2 3]` form
- Shared backing storage for slice windows so overlapping slice views observe element updates
- Layered validation with unit tests in `src/` and CLI integration tests under `tests/`

## Deferred Scope

- Full slice expressions such as `values[1:3:4]`
- String slice execution, `make`, explicit backing-array allocation APIs, or byte-specialized slice builtins
- Variadic forwarding with `...`
- Full Go comparability rules beyond rejecting slice equality in the current semantic layer
- Array, pointer-to-array, or channel operands for `cap`

## Interfaces and Extension Hooks

- `src/frontend/token.rs` and `src/frontend/parser.rs`: bracket tokens, recursive type parsing, typed `var` declarations, slice literals, and index expressions
- `src/semantic/model.rs` and `src/semantic/analyzer.rs`: recursive `Type`, typed zero-value declaration checking, slice validation, slice-aware expression checking, and index-assignment targets
- `src/semantic/builtins.rs`: centralized contracts for slice-aware `len`, `cap`, `copy`, and `append`
- `src/bytecode/instruction.rs` and `src/bytecode/compiler.rs`: `push-nil-slice`, `build-slice`, `slice`, `index`, and `set-index` instructions plus zero-value lowering
- `src/runtime/value.rs` and `src/runtime/vm.rs`: shared slice runtime values, nil-slice zero values, rendering, indexing, slice-window construction, overlap-safe copy, and capacity-aware append execution
- `tests/support/` plus CLI integration files: reusable helpers and layered automated coverage
