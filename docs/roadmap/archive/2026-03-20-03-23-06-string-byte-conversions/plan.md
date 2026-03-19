# Plan: String and Byte Conversions

## Basic Information

- Plan ID: `2026-03-20-03-23-06-string-byte-conversions`
- Milestone: `M3-standard-library-and-runtime-model`
- Status: `completed`

## Goals

- Promote `[]byte(string)` and `string([]byte)` from deferred compatibility gaps into first-class language syntax.
- Keep type-valued call syntax explicit in the AST and checked model instead of hiding conversions behind ad hoc builtin behavior.
- Strengthen the byte-oriented runtime string model with realistic conversion workflows that unblock more Go code.

## Scope

- Extend the existing slice/string research note with official conversion behavior for `[]byte(string)` and `string([]byte)`.
- Add a dedicated design baseline for typed conversion expressions.
- Parse conversion syntax where the callee position contains a type rather than a runtime expression.
- Type-check and lower the narrow conversion set `[]byte(string)` and `string([]byte)` through the semantic and bytecode layers.
- Execute conversion bytecode in the VM using the existing byte-oriented `StringValue` and slice runtime representations.
- Add examples, layered tests, serial CLI validation, reports, roadmap synchronization, and `BOOT.md` updates.

## Non-Goals

- General conversion coverage for arbitrary named types or numeric conversions
- Variadic byte/string bridging such as `append([]byte, string...)`
- Rune-oriented conversions, invalid-UTF-8-preserving CLI output, or panic formatting work
- Map/channel runtime expansion or multi-file package loading

## Phase Breakdown

1. Open the new `M3` plan, extend the existing compatibility research, and add a feature design for typed conversions.
2. Implement typed conversion expressions across parser, semantic analysis, bytecode lowering, and VM execution for `[]byte(string)` and `string([]byte)`.
3. Add examples plus focused parser, semantic, runtime, and CLI coverage for positive and negative conversion cases.
4. Run formatting and serial CLI validation, sync reports and roadmap documents, archive the completed plan, and commit plus push the full working tree.

## Acceptance Criteria

- `cargo test` passes with new coverage for parsing, checking, lowering, and executing `[]byte(string)` and `string([]byte)`.
- `cargo run -- run` executes a realistic example that round-trips through both conversions.
- `cargo run -- dump-ast` and `cargo run -- dump-bytecode` expose the conversion path clearly enough to debug without reading implementation code.
- `cargo run -- check` rejects at least one invalid conversion with a targeted diagnostic.
- Research, design, tech, verification, experience, roadmap, and `BOOT.md` all describe the new conversion surface and remaining limits.

## Risks

- Type-valued syntax can blur with ordinary calls unless the AST and checked model keep conversions distinct from value calls.
- Conversion support can sprawl into a fake general cast system unless this round stays narrow and explicitly documented.
- Slice/string conversion execution must preserve the current byte-oriented runtime model instead of silently reintroducing Rust string assumptions.
