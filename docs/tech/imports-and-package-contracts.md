# Imports and Package Contracts

## Purpose

Describe the narrow import and package-function contract model introduced during milestone `M3-standard-library-and-runtime-model`.

## Frontend Boundary

- `SourceFileAst` now keeps top-level import declarations before function declarations.
- Import declarations stay explicit in the AST:
  - single imports such as `import "fmt"`
  - alias imports such as `import b "bytes"`
  - grouped imports such as `import ("fmt"; b "bytes")`
- Expressions now support selector-call syntax such as `fmt.Println(message)`.
- The parser remains intentionally narrow:
  - imports support grouped declarations and explicit identifier aliases
  - dot imports and blank imports are rejected explicitly
  - selector expressions are only useful as call targets
  - package paths are string literals, not filesystem lookups

## Shared Identity Model

- Shared package identity lives in `src/package.rs`
- Current imported package set:
  - `fmt`
  - `strings`
  - `bytes`
- Shared package-function identity also lives in `src/package.rs`
- Current package-function set:
  - `fmt.Print(...value) -> void`
  - `fmt.Println(...value) -> void`
  - `fmt.Sprint(...value) -> string`
  - `strings.Compare(string, string) -> int`
  - `strings.Clone(string) -> string`
  - `strings.Contains(string, string) -> bool`
  - `strings.HasPrefix(string, string) -> bool`
  - `strings.HasSuffix(string, string) -> bool`
  - `strings.Index(string, string) -> int`
  - `strings.LastIndex(string, string) -> int`
  - `strings.IndexByte(string, byte) -> int`
  - `strings.LastIndexByte(string, byte) -> int`
  - `strings.Cut(string, string) -> (string, string, bool)`
  - `strings.CutPrefix(string, string) -> (string, bool)`
  - `strings.CutSuffix(string, string) -> (string, bool)`
  - `strings.TrimPrefix(string, string) -> string`
  - `strings.TrimSuffix(string, string) -> string`
  - `strings.Join([]string, string) -> string`
  - `strings.Repeat(string, int) -> string`
  - `bytes.Compare([]byte, []byte) -> int`
  - `bytes.Clone([]byte) -> []byte`
  - `bytes.Equal([]byte, []byte) -> bool`
  - `bytes.Contains([]byte, []byte) -> bool`
  - `bytes.HasPrefix([]byte, []byte) -> bool`
  - `bytes.HasSuffix([]byte, []byte) -> bool`
  - `bytes.Index([]byte, []byte) -> int`
  - `bytes.LastIndex([]byte, []byte) -> int`
  - `bytes.IndexByte([]byte, byte) -> int`
  - `bytes.LastIndexByte([]byte, byte) -> int`
  - `bytes.Cut([]byte, []byte) -> ([]byte, []byte, bool)`
  - `bytes.CutPrefix([]byte, []byte) -> ([]byte, bool)`
  - `bytes.CutSuffix([]byte, []byte) -> ([]byte, bool)`
  - `bytes.TrimPrefix([]byte, []byte) -> []byte`
  - `bytes.TrimSuffix([]byte, []byte) -> []byte`
  - `bytes.Join([][]byte, []byte) -> []byte`
  - `bytes.Repeat([]byte, int) -> []byte`

## Semantic Contract Model

- Semantic import lookup and package-function contracts live in `src/semantic/packages.rs`
- Import validation is metadata-backed:
  - unsupported import paths fail during semantic analysis
  - selector calls resolve through the declared binding name, so alias imports reuse the existing package-call path
  - selector calls require the target package binding to be imported first
  - unsupported package members fail during semantic analysis before lowering
- Builtin contracts and package contracts stay separate:
  - builtins remain in `src/semantic/builtins.rs`
  - imported package functions live in `src/semantic/packages.rs`
- Package-function validation is now per-function instead of package-wide:
  - `fmt` keeps variadic any-value contracts
  - `strings` introduces typed fixed-arity contracts, including search, suffix, trim, and staged multi-result `Cut`, `CutPrefix`, and `CutSuffix`
  - `bytes` introduces typed byte-slice and nested-byte-slice contracts, including search, suffix, trim, and staged multi-result `Cut`, `CutPrefix`, and `CutSuffix`
  - later packages should add new validators instead of hardcoding type rules in the analyzer
- User-defined and package-backed calls now share explicit result lists so zero-result calls, single-result calls, staged multi-result calls, and single-call-argument forwarding all flow through one contract boundary.

## Runtime Execution Notes

- Bytecode introduces `call-package <package.member> <arity>`
- VM dispatch currently supports:
  - `fmt.Print`: append rendered arguments without a trailing newline
  - `fmt.Println`: join rendered arguments with spaces and append a trailing newline
  - `fmt.Sprint`: join rendered arguments without separators and push the resulting string
  - `strings.Compare`: return `-1`, `0`, or `1` for lexicographic byte-oriented string comparison
  - `strings.Clone`: return a byte-preserving copy of the input string
  - `strings.Contains`: return whether one string contains another
  - `strings.HasPrefix`: return whether one string begins with another
  - `strings.HasSuffix`: return whether one string ends with another
  - `strings.Index`: return the first matching substring index or `-1`
  - `strings.LastIndex`: return the last matching substring index or `-1`
  - `strings.IndexByte`: return the first matching byte index or `-1`
  - `strings.LastIndexByte`: return the last matching byte index or `-1`
  - `strings.Cut`: push `(before, after, found)` using the first separator match
  - `strings.CutPrefix`: push `(after, found)` when a prefix match is present, otherwise `(original, false)`
  - `strings.CutSuffix`: push `(before, found)` when a suffix match is present, otherwise `(original, false)`
  - `strings.TrimPrefix`: return the original string or the trimmed suffix view
  - `strings.TrimSuffix`: return the original string or the trimmed prefix view
  - `strings.Join`: join `[]string` with a separator
  - `strings.Repeat`: repeat a string `count` times, with invalid counts surfaced as runtime errors
  - `bytes.Compare`: return `-1`, `0`, or `1` for lexicographic byte-slice comparison while treating nil and empty slices equivalently
  - `bytes.Clone`: return a copied `[]byte` while preserving the nil-vs-empty distinction
  - `bytes.Equal`: compare byte-slice contents, treating nil and empty slices equivalently through the byte-slice view
  - `bytes.Contains`: search a `[]byte` haystack for a `[]byte` subslice
  - `bytes.HasPrefix`: test whether a `[]byte` value begins with a prefix
  - `bytes.HasSuffix`: test whether a `[]byte` value ends with a suffix
  - `bytes.Index`: return the first matching subslice index or `-1`
  - `bytes.LastIndex`: return the last matching subslice index or `-1`
  - `bytes.IndexByte`: return the first matching byte index or `-1`
  - `bytes.LastIndexByte`: return the last matching byte index or `-1`
  - `bytes.Cut`: push `(before, after, found)` while preserving the staged nil `after` result on misses
  - `bytes.CutPrefix`: push `(after, found)` while preserving the original non-nil slice on misses
  - `bytes.CutSuffix`: push `(before, found)` while preserving the original non-nil slice on misses
  - `bytes.TrimPrefix`: return the original slice or the trimmed suffix view while preserving nil-vs-empty behavior
  - `bytes.TrimSuffix`: return the original slice or the trimmed prefix view while preserving nil-vs-empty behavior
  - `bytes.Join`: join `[][]byte` with a separator into a fresh `[]byte`
  - `bytes.Repeat`: repeat a `[]byte` value `count` times, with invalid counts surfaced as runtime errors
- The `fmt` behavior is intentionally approximate and does not yet implement Go formatting verbs
- The `strings` seam is intentionally narrow and does not yet model the full Go panic surface or rune-sensitive split behavior
- The `bytes` seam is intentionally narrow and does not yet model panic/recover-accurate failures or UTF-8-sequence-aware split behavior

## Extension Hooks

- Add new package IDs or package-function IDs in `src/package.rs`
- Extend `src/semantic/packages.rs` before changing lowering or runtime dispatch
- Keep filesystem package loading out of the semantic analyzer until a later plan introduces a real import graph
- Use `docs/research/` notes to document the official behavior baseline before selecting the next package slice
