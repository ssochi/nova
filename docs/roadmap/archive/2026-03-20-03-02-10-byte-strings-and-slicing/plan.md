# Plan: Byte-Oriented Strings and String Slicing

## Basic Information

- Plan ID: `2026-03-20-03-02-10-byte-strings-and-slicing`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Move runtime strings to a byte-oriented model so the VM can support Go-like string slicing without pretending Rust `String` semantics are equivalent.
- Add semantic and runtime support for string indexing and slicing, with string indexing producing `byte`.
- Make the new byte surface practically usable by supporting `byte`, `[]byte`, and the `copy([]byte, string)` builtin special case.

## Scope

- Extend the existing slice research note with Go string indexing, string slicing, `byte`, and `copy([]byte, string)` behavior.
- Add `byte` to the semantic type model, checked expressions, bytecode value types, and runtime values.
- Change runtime string storage to a byte-oriented representation and keep `len`, concatenation, equality, printing, and the current `strings` package seam working on that representation.
- Support string index expressions, simple string slice expressions, and `copy([]byte, string)`.
- Add examples, layered tests, CLI inspection coverage, reports, roadmap synchronization, and `BOOT.md` updates.

## Non-Goals

- General Go conversion syntax such as `[]byte("text")` or `string(bytes)`
- `append([]byte, string...)`, variadic forwarding, or rune-aware string helpers
- Map or channel `make`, arrays, structs, interfaces, or panic/recover semantics
- Full untyped-constant compatibility for assigning integer literals to `byte`

## Phase Breakdown

1. Open the new `M3` plan and extend the existing slice research / design baseline for byte-oriented strings.
2. Implement `byte`, byte-oriented runtime strings, string slicing/indexing, and the `copy([]byte, string)` path across semantic analysis, lowering, and VM execution.
3. Add examples plus focused parser, semantic, runtime, and CLI coverage for positive and negative cases.
4. Run formatting and serial CLI validation, then sync reports, roadmap docs, `BOOT.md`, archive the plan if complete, and commit plus push the full working tree.

## Acceptance Criteria

- `cargo test` passes with new coverage for string slicing, string indexing, byte locals / slices, and `copy([]byte, string)`.
- `cargo run -- run` executes a realistic example that slices strings, inspects indexed bytes, and copies string bytes into `[]byte`.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` clearly expose the new string / byte paths without reading implementation details.
- `cargo run -- check` rejects at least one invalid string-slice or byte-copy program with a targeted diagnostic.
- Research, design, tech, verification, experience, roadmap, and `BOOT.md` all describe the new byte-oriented string surface and its remaining limits.

## Risks

- Moving runtime strings away from Rust `String` can leak implementation shortcuts unless all string consumers are updated together.
- Adding `byte` without a full untyped-constant system can create misleading partial support unless docs and diagnostics stay explicit.
- Reusing generic `index` and `slice` syntax across slices and strings can blur debug surfaces unless bytecode and CLI dumps remain readable.
