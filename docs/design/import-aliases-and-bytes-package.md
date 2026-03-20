# Import Aliases and Bytes Package Seam

## Goal

Extend the current import/package subsystem with grouped imports, explicit import aliases, and a staged `bytes` package seam while preserving the metadata-backed package model and CLI-visible syntax.

## Constraints

- Rust standard library only
- Keep import/package metadata centralized instead of scattering special cases through the analyzer or VM
- Preserve readable `dump-ast` and `dump-bytecode` output
- Do not imply dot imports, blank imports, or real filesystem package loading
- Reuse the current byte-slice runtime model instead of introducing new string/rune representations

## Current Scope

- AST support for import declarations that can represent:
  - `import "fmt"`
  - `import alias "bytes"`
  - `import ("fmt"; alias "bytes")`
- Parser support for grouped imports and explicit identifier aliases
- Semantic import binding lookup keyed by the declared binding name
- Shared package IDs and typed package-function contracts for:
  - `bytes.Equal`
  - `bytes.Contains`
  - `bytes.HasPrefix`
  - `bytes.Join`
  - `bytes.Repeat`
- Runtime package dispatch for the staged `bytes` subset
- CLI examples and tests that show alias imports and grouped imports through both execution and inspection paths

## Deferred Scope

- Dot imports, blank imports, or import-side-effect behavior
- Selector expressions on non-package values
- Filesystem package graphs or multi-file package loading
- Broader `bytes` APIs that need interfaces, errors, runes, or multi-result support
- Panic-accurate runtime behavior for `bytes.Repeat`

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: keep import declarations explicit, including optional binding names
- `src/frontend/parser.rs`: parse single and grouped import declarations without hiding grouped structure from the AST
- `src/package.rs`: extend shared imported-package and package-function identities for `bytes`
- `src/semantic/registry.rs`: resolve import bindings through the declared alias or package default name
- `src/semantic/packages.rs`: centralize `bytes` argument/return validation alongside `fmt` and `strings`
- `src/runtime/vm.rs`: execute `bytes` package functions through the existing `call-package` dispatch path
