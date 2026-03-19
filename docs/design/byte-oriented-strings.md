# Byte-Oriented Strings

## Goal

Move runtime strings to a byte-oriented model so the compiler can support Go-like string indexing and simple string slicing while keeping the VM-first pipeline layered and debuggable.

## Constraints

- Rust standard library only
- Preserve the existing frontend -> semantic -> bytecode -> VM layering
- Keep `byte` explicit in the type system instead of smuggling string indexing through `int`
- Finish in one iteration with real CLI validation and without opening full conversion or panic semantics

## Current Scope

- Add semantic and runtime support for the predeclared `byte` type
- Store runtime strings as bytes instead of only Rust `String`
- Keep existing string features working: literals, equality, concatenation, `len`, `fmt`, and the narrow `strings` package seam
- Support string index expressions that produce `byte`
- Support simple string slice expressions that produce `string`
- Support `copy([]byte, string)` as the first byte-specialized builtin path
- Keep `dump-ast` and `dump-bytecode` readable for the new string and byte paths

## Deferred Scope

- General Go conversion syntax such as `[]byte("hello")` and `string(bytes)`
- `append([]byte, string...)`, variadic forwarding syntax, or rune-aware iteration
- Full untyped-constant compatibility for `byte`
- Exact Go panic formatting or output behavior for invalid UTF-8 byte sequences

## Interfaces and Extension Hooks

- `src/semantic/model.rs` and `src/semantic/analyzer.rs`: add `byte`, type resolution, string-index typing, and string-slice validation
- `src/semantic/builtins.rs`: teach `copy` about the `[]byte` <- `string` special case while keeping the contract table centralized
- `src/bytecode/instruction.rs` and `src/bytecode/compiler.rs`: carry byte values and string-aware index / slice instructions clearly through debug output
- `src/runtime/value.rs`: store strings as byte-oriented values and expose helpers for byte indexing, byte slicing, concatenation, and rendering
- `src/runtime/vm.rs`: execute the new string / byte instructions, preserve existing builtin behavior, and keep runtime errors explicit for invalid bounds
- `docs/tech/runtime-values-and-builtins.md`: record that strings are now byte-oriented runtime values rather than Rust `String` wrappers
