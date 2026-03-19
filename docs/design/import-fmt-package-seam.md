# Import Declarations and Fmt Package Seam

## Goal

Add the first narrow standard-library seam for `M3` by supporting top-level import declarations and selector-based `fmt` package calls without introducing multi-file package loading yet.

## Constraints

- Rust standard library only
- Preserve the existing frontend -> semantic -> bytecode -> VM layering
- Keep builtin contracts and package-function contracts in separate, centralized seams
- Stay narrow enough to finish in one iteration with real CLI validation

## Current Scope

- Top-level single-line imports such as `import "fmt"`
- Selector-call syntax such as `fmt.Println(...)` and `fmt.Sprint(...)`
- Centralized package identities in `src/package.rs`
- Centralized semantic package contracts in `src/semantic/packages.rs`
- Metadata-backed import validation instead of filesystem package resolution
- `fmt.Print`, `fmt.Println`, and `fmt.Sprint` as the first supported package functions

## Deferred Scope

- Grouped imports, aliases, dot imports, blank imports, or multiple source files per package
- Filesystem package graphs or real standard-library source loading
- Full Go `fmt` formatting verbs and exact compatibility semantics
- Additional standard-library packages beyond the first `fmt` seam
- Composite runtime values such as slices, maps, structs, or interfaces

## Interfaces and Extension Hooks

- `src/frontend/ast.rs`: import declarations plus selector-call expressions
- `src/package.rs`: shared package and package-function identities
- `src/semantic/packages.rs`: package import lookup and package-function contract validation
- `src/semantic/analyzer.rs`: import registry creation and selector-call semantic resolution
- `src/bytecode/instruction.rs`: `call-package` instruction for package-backed runtime services
- `src/runtime/vm.rs`: package-function dispatch for `fmt`
