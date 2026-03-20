# Strings and Bytes Compare Seams

## Goal

Extend the metadata-backed `strings` and `bytes` package seams with staged lexicographic comparison helpers that match the current byte-oriented runtime model.

## Constraints

- Rust standard library only
- Preserve the existing frontend -> semantic -> bytecode -> VM layering
- Reuse the centralized package metadata and validator tables instead of adding package-specific analyzer shortcuts
- Keep the scope limited to byte-oriented comparison behavior that the current runtime can model honestly

## Current Scope

- Shared package IDs and typed package contracts for:
  - `strings.Compare(string, string) -> int`
  - `bytes.Compare([]byte, []byte) -> int`
- VM package dispatch for both helpers through the existing `call-package` path
- A small runtime helper for lexicographic byte-slice comparison so `bytes.Compare` stays centralized and testable
- CLI examples and focused tests that keep `Compare` visible in `dump-ast`, `dump-bytecode`, and `check`

## Deferred Scope

- Unicode-aware or case-folding APIs such as `strings.EqualFold`
- Richer comparison helpers that depend on rune or UTF-8-sequence semantics
- New syntax, operator overloading, or rewrites of ordinary `==`, `<`, and `>` expression handling
- Panic-accurate runtime behavior beyond the current staged VM error surface

## Interfaces and Extension Hooks

- `src/package.rs`: extend shared package-function identities for `strings.Compare` and `bytes.Compare`
- `src/semantic/packages.rs`: centralize fixed-arity type validation and integer result contracts
- `src/runtime/vm/packages.rs`: keep `strings` package dispatch explicit in the package-call path
- `src/runtime/vm/support.rs`: keep byte-slice comparison logic reusable and separate from semantic validation
- `docs/research/2026-03-20-strings-bytes-compare-seams.md`: official behavior baseline that limits this slice to byte-oriented compare semantics
