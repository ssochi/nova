# Imports and Package Contracts

## Purpose

Describe the narrow import and package-function contract model introduced during milestone `M3-standard-library-and-runtime-model`.

## Frontend Boundary

- `SourceFileAst` now keeps top-level `import "path"` declarations before function declarations.
- Expressions now support selector-call syntax such as `fmt.Println(message)`.
- The parser remains intentionally narrow:
  - imports are single-line declarations only
  - selector expressions are only useful as call targets
  - package paths are string literals, not filesystem lookups

## Shared Identity Model

- Shared package identity lives in `src/package.rs`
- Current imported package set:
  - `fmt`
  - `strings`
- Shared package-function identity also lives in `src/package.rs`
- Current package-function set:
  - `fmt.Print(...value) -> void`
  - `fmt.Println(...value) -> void`
  - `fmt.Sprint(...value) -> string`
  - `strings.Contains(string, string) -> bool`
  - `strings.HasPrefix(string, string) -> bool`
  - `strings.Join([]string, string) -> string`
  - `strings.Repeat(string, int) -> string`

## Semantic Contract Model

- Semantic import lookup and package-function contracts live in `src/semantic/packages.rs`
- Import validation is metadata-backed:
  - unsupported import paths fail during semantic analysis
  - selector calls require the target package binding to be imported first
  - unsupported package members fail during semantic analysis before lowering
- Builtin contracts and package contracts stay separate:
  - builtins remain in `src/semantic/builtins.rs`
  - imported package functions live in `src/semantic/packages.rs`
- Package-function validation is now per-function instead of package-wide:
  - `fmt` keeps variadic any-value contracts
  - `strings` introduces typed fixed-arity contracts
  - later packages should add new validators instead of hardcoding type rules in the analyzer

## Runtime Execution Notes

- Bytecode introduces `call-package <package.member> <arity>`
- VM dispatch currently supports:
  - `fmt.Print`: append rendered arguments without a trailing newline
  - `fmt.Println`: join rendered arguments with spaces and append a trailing newline
  - `fmt.Sprint`: join rendered arguments without separators and push the resulting string
  - `strings.Contains`: return whether one string contains another
  - `strings.HasPrefix`: return whether one string begins with another
  - `strings.Join`: join `[]string` with a separator
  - `strings.Repeat`: repeat a string `count` times, with invalid counts surfaced as runtime errors
- The `fmt` behavior is intentionally approximate and does not yet implement Go formatting verbs
- The `strings` seam is intentionally narrow and does not yet model the full Go panic surface

## Extension Hooks

- Add new package IDs or package-function IDs in `src/package.rs`
- Extend `src/semantic/packages.rs` before changing lowering or runtime dispatch
- Keep filesystem package loading out of the semantic analyzer until a later plan introduces a real import graph
- Use `docs/research/` notes to document the official behavior baseline before selecting the next package slice
