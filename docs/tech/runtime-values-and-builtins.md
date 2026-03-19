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
  - Stored as shared backing storage plus start / length / capacity metadata
  - Built by slice literals and returned by `append`
  - Also used as the zero value for explicit typed slice declarations via a nil-slice runtime state
  - Currently rendered in a Go-like `[value value]` form for builtin and package output
  - Supports `len(slice)`, `cap(slice)`, `copy(dst, src)`, index expressions such as `values[0]`, simple slice expressions such as `values[1:3]`, and element assignment such as `values[0] = 1`

## Builtin Contract Model

- Shared builtin identity lives in `src/builtin.rs`
- Semantic builtin contracts live in `src/semantic/builtins.rs`
- Current builtin set:
  - `print(...value) -> void`
  - `println(...value) -> void`
  - `len(string|slice) -> int`
  - `cap(slice) -> int`
  - `copy(slice, slice) -> int`
  - `append(slice, ...element) -> slice`
- Variadic output builtins accept any value-producing expression in the current type system
- `len` validates one string or slice target before lowering
- `cap` validates one slice target before lowering
- `copy` validates destination and source slice types centrally before lowering
- `append` validates a slice first argument and matching appended element types before lowering

## Package Contract Model

- Shared package and package-function identity live in `src/package.rs`
- Semantic package contracts live in `src/semantic/packages.rs`
- Current imported package support:
  - `import "fmt"`
  - `import "strings"`
- Current package-backed function set:
  - `fmt.Print(...value) -> void`
  - `fmt.Println(...value) -> void`
  - `fmt.Sprint(...value) -> string`
  - `strings.Contains(string, string) -> bool`
  - `strings.HasPrefix(string, string) -> bool`
  - `strings.Join([]string, string) -> string`
  - `strings.Repeat(string, int) -> string`
- Selector calls require the package binding to be imported before semantic analysis will lower them
- Unsupported import paths and unsupported package members fail during semantic analysis

## Runtime Execution Notes

- Bytecode uses `push-string` for literals and `concat` for string addition
- Bytecode now also uses `push-nil-slice` for typed zero-value slice declarations, `build-slice <count>` for slice literals, `index` for slice element reads, `slice` for slice-window creation, and `set-index` for slice element writes
- Equality still reuses the generic value comparison path because runtime values are tagged
- VM output is an accumulated string buffer instead of newline-separated records
- `print` appends rendered arguments without an automatic trailing newline
- `println` appends rendered arguments plus a newline
- `cap` returns the current slice capacity metadata tracked by the runtime
- `copy` snapshots source elements before writing, so overlapping slice windows behave predictably
- `append` now reuses existing backing storage when spare capacity is available; otherwise it allocates a fresh slice value
- Slice windows share backing storage, so updating one slice view is visible through overlapping slice values
- Explicit typed local declarations are lowered into concrete zero-producing instructions, so `var total int`, `var ready bool`, `var label string`, and `var values []int` all produce Go-like zero values without runtime type reflection
- Bytecode now also uses `call-package` for metadata-backed package functions
- `fmt.Sprint` returns a runtime string value without mutating the output buffer
- `fmt` formatting is intentionally approximate and does not yet support format verbs
- `strings.Join` currently requires a runtime `[]string` value and returns a joined string
- `strings.Repeat` maps negative-count or repeated-size overflow failures into runtime errors because the VM does not model Go panic yet
- String slice execution is still deferred because the current runtime stores strings as Rust `String` instead of a byte-addressed Go string model

## Extension Hooks

- Add new builtin IDs in `src/builtin.rs`, then extend `src/semantic/builtins.rs` before touching lowering or runtime
- Add new package IDs and package-function IDs in `src/package.rs`, then extend `src/semantic/packages.rs`
- Keep new runtime value categories reflected in both `src/runtime/value.rs` and semantic `Type`
- If output behavior becomes more realistic or package-backed, extract builtin execution helpers from `src/runtime/vm.rs`
- Keep package-function validation metadata centralized; do not reintroduce package-specific ad hoc type checks inside `src/semantic/analyzer.rs`
- If slice behavior expands beyond the current window / builtin subset, consider separating slice-specific lowering and VM helpers from the core scalar path
