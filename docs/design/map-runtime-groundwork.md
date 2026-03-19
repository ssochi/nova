# Map Runtime Groundwork

## Goal

Add staged `map[K]V` support to the VM-first compiler pipeline without overcommitting to full Go map semantics in a single iteration.

## Constraints

- Only the Rust standard library may be used.
- The language surface must stay explicit through AST, checked model, bytecode, and VM layers.
- The implementation must preserve the repository's layered validation and keep individual source files under the 1000-line limit.

## Current Scope

- Parse and render `map[K]V` type syntax in declarations and `make(map[K]V[, hint])`.
- Represent typed zero-value maps as nil maps and `make`-allocated maps as writable non-nil maps.
- Support `len(map)`, map indexing in expression position, and map index assignment.
- Support staged `map[K]V{...}` literals with keyed elements and explicit bytecode construction.
- Support builtin `delete(map, key)` with nil-map no-op behavior.
- Restrict keys to the currently modeled comparable scalar set and make that rule explicit in semantic validation.
- Emit dedicated bytecode for map allocation, lookup, and assignment so CLI debug surfaces expose the new runtime path.

## Deferred Scope

- Comma-ok map lookup, equality with `nil`, and `range`.
- Full Go duplicate-constant-key diagnostics for map literals if the staged literal path only guarantees deterministic last-write-wins behavior.
- Keys that depend on broader type-system work such as structs, interfaces, or arrays.
- Backend-oriented map lowering or realistic Go hash/iteration behavior.

## Interfaces and Extension Hooks

- Extend `TypeRef`, checked `Type`, and runtime `ValueType` with explicit map forms.
- Keep `make` generalized at the AST boundary so slices and maps share typed-builtin syntax without pretending their argument rules are identical.
- Keep map literals as an explicit checked expression kind rather than disguising them as `make` plus assignments.
- Use a dedicated runtime map value wrapper with shared storage and explicit nil state so later `delete` and map passing semantics reuse the same container model.
- Keep key comparability checks centralized in semantic support helpers rather than duplicating them across parser, builtin, and VM layers.
