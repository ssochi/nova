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
- `slice`
  - Stored as an ordered vector of runtime values
  - Built by slice literals and returned by `append`
  - Currently rendered in a Go-like `[value value]` form for builtin and package output
  - Supports `len(slice)` and index expressions such as `values[0]`

## Builtin Contract Model

- Shared builtin identity lives in `src/builtin.rs`
- Semantic builtin contracts live in `src/semantic/builtins.rs`
- Current builtin set:
  - `print(...value) -> void`
  - `println(...value) -> void`
  - `len(string|slice) -> int`
  - `append(slice, ...element) -> slice`
- Variadic output builtins accept any value-producing expression in the current type system
- `len` validates one string or slice target before lowering
- `append` validates a slice first argument and matching appended element types before lowering

## Package Contract Model

- Shared package and package-function identity live in `src/package.rs`
- Semantic package contracts live in `src/semantic/packages.rs`
- Current imported package support:
  - `import "fmt"`
- Current package-backed function set:
  - `fmt.Print(...value) -> void`
  - `fmt.Println(...value) -> void`
  - `fmt.Sprint(...value) -> string`
- Selector calls require the package binding to be imported before semantic analysis will lower them
- Unsupported import paths and unsupported package members fail during semantic analysis

## Runtime Execution Notes

- Bytecode uses `push-string` for literals and `concat` for string addition
- Bytecode now also uses `build-slice <count>` for slice literals and `index` for slice element reads
- Equality still reuses the generic value comparison path because runtime values are tagged
- VM output is an accumulated string buffer instead of newline-separated records
- `print` appends rendered arguments without an automatic trailing newline
- `println` appends rendered arguments plus a newline
- `append` returns a new slice runtime value and does not mutate earlier values in place
- Bytecode now also uses `call-package` for metadata-backed package functions
- `fmt.Sprint` returns a runtime string value without mutating the output buffer
- `fmt` formatting is intentionally approximate and does not yet support format verbs

## Extension Hooks

- Add new builtin IDs in `src/builtin.rs`, then extend `src/semantic/builtins.rs` before touching lowering or runtime
- Add new package IDs and package-function IDs in `src/package.rs`, then extend `src/semantic/packages.rs`
- Keep new runtime value categories reflected in both `src/runtime/value.rs` and semantic `Type`
- If output behavior becomes more realistic or package-backed, extract builtin execution helpers from `src/runtime/vm.rs`
- If slice behavior expands beyond literals and indexing, consider separating slice-specific lowering and VM helpers from the core scalar path
