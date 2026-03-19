# Runtime Values and Builtins

## Purpose

Describe the current runtime value categories and builtin execution model introduced in milestone `M3-standard-library-and-runtime-model`.

## Runtime Value Categories

- `int`
  - Stored as `i64`
  - Used by arithmetic, comparisons, and `len` results
- `bool`
  - Used by branch and loop conditions plus equality
- `string`
  - Produced by string literals, user functions, concatenation, and builtin arguments
  - `len(string)` returns the UTF-8 byte length

## Builtin Contract Model

- Shared builtin identity lives in `src/builtin.rs`
- Semantic builtin contracts live in `src/semantic/builtins.rs`
- Current builtin set:
  - `print(...value) -> void`
  - `println(...value) -> void`
  - `len(string) -> int`
- Variadic output builtins accept any value-producing expression in the current type system
- Exact-arity builtins validate both argument count and argument types before lowering

## Runtime Execution Notes

- Bytecode uses `push-string` for literals and `concat` for string addition
- Equality still reuses the generic value comparison path because runtime values are tagged
- VM output is an accumulated string buffer instead of newline-separated records
- `print` appends rendered arguments without an automatic trailing newline
- `println` appends rendered arguments plus a newline

## Extension Hooks

- Add new builtin IDs in `src/builtin.rs`, then extend `src/semantic/builtins.rs` before touching lowering or runtime
- Keep new runtime value categories reflected in both `src/runtime/value.rs` and semantic `Type`
- If output behavior becomes more realistic or package-backed, extract builtin execution helpers from `src/runtime/vm.rs`
