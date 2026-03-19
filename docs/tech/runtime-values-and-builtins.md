# Runtime Values and Builtins

## Purpose

Describe the current runtime value categories and builtin execution model introduced in milestone `M3-standard-library-and-runtime-model`.

## Runtime Value Categories

- `int`
  - Stored as `i64`
  - Used by arithmetic, comparisons, and `len` results
- `byte`
  - Stored as `u8`
  - Produced by string index expressions and used as the element type of `[]byte`
  - Rendered as its decimal byte value in builtin and package output
- `bool`
  - Used by branch and loop conditions plus equality
- `string`
  - Produced by string literals, user functions, concatenation, and builtin arguments
  - Stored as byte-oriented runtime data rather than only Rust `String`
  - `len(string)` returns the byte length
  - Supports byte-oriented index expressions such as `text[0]` and simple slice expressions such as `text[1:3]`
  - Supports explicit `string([]byte)` conversion by copying bytes out of a runtime byte slice
  - Current CLI rendering is lossy for invalid UTF-8 byte sequences because the output buffer is still a Rust `String`
- `slice`
  - Stored as shared backing storage plus start / length / capacity metadata
  - Built by slice literals, `make([]T, len[, cap])`, and returned by `append`
  - Also used as the zero value for explicit typed slice declarations via a nil-slice runtime state
  - Currently rendered in a Go-like `[value value]` form for builtin and package output
  - Supports `len(slice)`, `cap(slice)`, `copy(dst, src)`, index expressions such as `values[0]`, simple slice expressions such as `values[1:3]`, element assignment such as `values[0] = 1`, and explicit `[]byte(string)` conversion by copying string bytes into a new non-nil byte slice

## Builtin Contract Model

- Shared builtin identity lives in `src/builtin.rs`
- Semantic builtin contracts live in `src/semantic/builtins.rs`
- Current builtin set:
  - `print(...value) -> void`
  - `println(...value) -> void`
  - `len(string|slice) -> int`
  - `make([]T, len[, cap]) -> []T`
  - `cap(slice) -> int`
  - `copy(slice, slice|string) -> int`
  - `append(slice, ...element) -> slice`
- Variadic output builtins accept any value-producing expression in the current type system
- `len` validates one string or slice target before lowering
- `make` validates a slice type argument plus one required and one optional integer size argument before lowering
- `cap` validates one slice target before lowering
- `copy` validates destination and source slice types centrally before lowering, including the `[]byte` <- `string` special case
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

- Bytecode uses `push-string` for literals, `push-byte` for byte zero values, and `concat` for string addition
- Bytecode now also uses `push-nil-slice` for typed zero-value slice declarations, `build-slice <count>` for slice literals, `make-slice <type>` for slice allocation, `index <slice|string>` for element reads, `slice <slice|string>` for window creation, and `set-index` for slice element writes
- Bytecode now also uses `convert string->[]byte` and `convert []byte->string` for the narrow explicit conversion surface
- Equality still reuses the generic value comparison path because runtime values are tagged
- VM output is an accumulated string buffer instead of newline-separated records
- `print` appends rendered arguments without an automatic trailing newline
- `println` appends rendered arguments plus a newline
- `cap` returns the current slice capacity metadata tracked by the runtime
- `copy` snapshots source slice elements before writing, so overlapping slice windows behave predictably
- `copy([]byte, string)` copies raw string bytes into the destination slice and returns the copied byte count
- `append` now reuses existing backing storage when spare capacity is available; otherwise it allocates a fresh slice value
- `make([]T, len[, cap])` lowers into dedicated allocation bytecode instead of a generic runtime builtin call because its first argument is a type
- `make` allocation reserves hidden capacity slots filled with the element zero value, so reslicing into spare capacity exposes zero-initialized elements and later `append` can reuse that storage
- Slice windows share backing storage, so updating one slice view is visible through overlapping slice values
- `[]byte(string)` currently returns a non-nil byte slice with exact-length capacity; real Go leaves the capacity implementation-specific
- `string([]byte)` copies the visible byte slice elements into a new runtime string value
- Explicit typed local declarations are lowered into concrete zero-producing instructions, so `var total int`, `var marker byte`, `var ready bool`, `var label string`, and `var values []int` all produce Go-like zero values without runtime type reflection
- Bytecode now also uses `call-package` for metadata-backed package functions
- `fmt.Sprint` returns a runtime string value without mutating the output buffer
- `fmt` formatting is intentionally approximate and does not yet support format verbs
- `strings` package functions now operate on the byte-oriented runtime string representation instead of converting through Rust-only string semantics
- `strings.Join` currently requires a runtime `[]string` value and returns a joined string
- `strings.Repeat` maps negative-count or repeated-size overflow failures into runtime errors because the VM does not model Go panic yet

## Extension Hooks

- Add new builtin IDs in `src/builtin.rs`, then extend `src/semantic/builtins.rs` before touching lowering or runtime
- If a builtin needs type arguments, keep that path explicit in the AST and checked model instead of pretending type syntax is an ordinary runtime value
- Add new package IDs and package-function IDs in `src/package.rs`, then extend `src/semantic/packages.rs`
- Keep new runtime value categories reflected in both `src/runtime/value.rs` and semantic `Type`
- If output behavior becomes more realistic or package-backed, extract builtin execution helpers from `src/runtime/vm.rs`
- Keep package-function validation metadata centralized; do not reintroduce package-specific ad hoc type checks inside `src/semantic/analyzer.rs`
- If string behavior expands into broader conversions, rune-aware iteration, or invalid-UTF-8-preserving printing, add that on top of the current byte-oriented representation instead of reverting to Rust `String`-only storage
- If slice behavior expands beyond the current window / builtin subset, consider separating slice-specific lowering and VM helpers from the core scalar path
