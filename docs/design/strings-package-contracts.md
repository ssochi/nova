# Strings Package Contracts

## Goal

Add a second standard-library package seam under `M3` by supporting a narrow `strings` package slice while upgrading package contracts from variadic-any-value metadata to per-function typed validation.

## Constraints

- Rust standard library only
- Preserve the existing frontend -> semantic -> bytecode -> VM layering
- Reuse the import and package-call surface introduced by the earlier `fmt` seam
- Keep the slice narrow enough to validate through real CLI commands in one round

## Current Scope

- `import "strings"` as a metadata-backed imported package
- Typed package contracts for:
  - `strings.Contains(string, string) -> bool`
  - `strings.HasPrefix(string, string) -> bool`
  - `strings.Join([]string, string) -> string`
  - `strings.Repeat(string, int) -> string`
- VM package execution for the supported `strings` helpers
- Runtime error mapping for invalid `strings.Repeat` counts that the VM cannot model as Go panics yet

## Deferred Scope

- Full `strings` package coverage
- Grouped imports, aliases, or filesystem-backed package loading
- Exact Go panic semantics, including stack traces and recovery
- `nil` slices and other behaviors that need a broader runtime model
- Rich selector expressions beyond imported package call targets

## Interfaces and Extension Hooks

- `src/package.rs`: shared package and package-function identities for both `fmt` and `strings`
- `src/semantic/packages.rs`: centralized package-function validators, including typed fixed-arity checks
- `src/semantic/analyzer.rs`: import registry lookup and selector-call semantic resolution
- `src/bytecode/instruction.rs`: unchanged `call-package` lowering surface reused by the new seam
- `src/runtime/vm.rs`: package-function dispatch and runtime error mapping for the supported `strings` helpers
- `docs/research/2026-03-20-strings-package-contracts.md`: official behavior baseline that explains why these functions were chosen first
