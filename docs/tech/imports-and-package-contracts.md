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
  - `strings.Contains(string, string) -> bool`
  - `strings.HasPrefix(string, string) -> bool`
  - `strings.Cut(string, string) -> (string, string, bool)`
  - `strings.Join([]string, string) -> string`
  - `strings.Repeat(string, int) -> string`
  - `bytes.Equal([]byte, []byte) -> bool`
  - `bytes.Contains([]byte, []byte) -> bool`
  - `bytes.HasPrefix([]byte, []byte) -> bool`
  - `bytes.Cut([]byte, []byte) -> ([]byte, []byte, bool)`
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
  - `strings` introduces typed fixed-arity contracts, including staged multi-result `Cut`
  - `bytes` introduces typed byte-slice and nested-byte-slice contracts, including staged multi-result `Cut`
  - later packages should add new validators instead of hardcoding type rules in the analyzer
- User-defined and package-backed calls now share explicit result lists so zero-result calls, single-result calls, and staged multi-result calls all flow through one contract boundary.

## Runtime Execution Notes

- Bytecode introduces `call-package <package.member> <arity>`
- VM dispatch currently supports:
  - `fmt.Print`: append rendered arguments without a trailing newline
  - `fmt.Println`: join rendered arguments with spaces and append a trailing newline
  - `fmt.Sprint`: join rendered arguments without separators and push the resulting string
  - `strings.Contains`: return whether one string contains another
  - `strings.HasPrefix`: return whether one string begins with another
  - `strings.Cut`: push `(before, after, found)` using the first separator match
  - `strings.Join`: join `[]string` with a separator
  - `strings.Repeat`: repeat a string `count` times, with invalid counts surfaced as runtime errors
  - `bytes.Equal`: compare byte-slice contents, treating nil and empty slices equivalently through the byte-slice view
  - `bytes.Contains`: search a `[]byte` haystack for a `[]byte` subslice
  - `bytes.HasPrefix`: test whether a `[]byte` value begins with a prefix
  - `bytes.Cut`: push `(before, after, found)` while preserving the staged nil `after` result on misses
  - `bytes.Join`: join `[][]byte` with a separator into a fresh `[]byte`
  - `bytes.Repeat`: repeat a `[]byte` value `count` times, with invalid counts surfaced as runtime errors
- The `fmt` behavior is intentionally approximate and does not yet implement Go formatting verbs
- The `strings` seam is intentionally narrow and does not yet model the full Go panic surface
- The `bytes` seam is intentionally narrow and does not yet model panic/recover-accurate failures

## Extension Hooks

- Add new package IDs or package-function IDs in `src/package.rs`
- Extend `src/semantic/packages.rs` before changing lowering or runtime dispatch
- Keep filesystem package loading out of the semantic analyzer until a later plan introduces a real import graph
- Use `docs/research/` notes to document the official behavior baseline before selecting the next package slice
