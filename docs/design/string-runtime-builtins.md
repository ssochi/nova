# String Runtime and Builtin Contracts

## Goal

Add the first richer runtime value slice for `M3` by supporting strings end to end and centralizing builtin call contracts before the runtime surface grows further.

## Constraints

- Rust standard library only
- Preserve the existing frontend -> semantic -> bytecode -> VM layering
- Keep builtin identity and contracts centralized instead of scattering string name checks
- Stay narrow enough to finish in one iteration with real CLI validation

## Current Scope

- Double-quoted string literals with a small escape set: `\\`, `\"`, `\n`, `\t`, `\r`
- Semantic `string` typing for parameters, returns, locals, literals, equality, and concatenation
- Builtin expansion from `println` to `print`, `println`, and `len`
- Centralized builtin contract lookup under `src/semantic/builtins.rs`
- VM output modeled as a stream string so `print` and `println` can coexist correctly
- `len(string)` defined as UTF-8 byte length to match Go's string-length behavior

## Deferred Scope

- Raw string literals and broader escape coverage
- Composite runtime values such as slices, maps, structs, or interfaces
- Import resolution and standard library package loading
- Go-exact `print` / `println` formatting behavior and diagnostic spans

## Interfaces and Extension Hooks

- `src/builtin.rs`: shared builtin identifiers used by semantic analysis, lowering, and runtime dispatch
- `src/semantic/builtins.rs`: builtin contract table and validation rules
- `src/semantic/analyzer.rs`: string typing, builtin resolution, and string-aware operator checks
- `src/bytecode/instruction.rs`: `push-string`, `concat`, and shared builtin call encoding
- `src/runtime/vm.rs`: output stream behavior plus builtin execution for `print`, `println`, and `len`
