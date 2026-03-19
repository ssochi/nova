# String and Byte Conversion Expressions

## Goal

Add first-class `[]byte(string)` and `string([]byte)` conversion syntax without treating conversions as builtin calls or leaking runtime-only rules into parsing and semantic analysis.

## Constraints

- Preserve the existing frontend -> semantic -> bytecode -> VM layering
- Keep conversion syntax explicit in the AST and checked model because the callee position is a type, not a runtime value
- Reuse the byte-oriented `StringValue` and existing `[]byte` slice runtime model introduced in earlier `M3` plans
- Keep the scope narrow enough to finish in one iteration with real CLI validation

## Current Scope

- Parse explicit conversion expressions for named and slice types when the source form is `T(x)`
- Support only the narrow pair `[]byte(string)` and `string([]byte)`
- Lower conversions into dedicated bytecode instructions so `dump-bytecode` remains readable
- Execute conversions by copying bytes between runtime string and byte-slice representations
- Reject unsupported conversion targets or source types during semantic analysis with targeted diagnostics

## Deferred Scope

- Numeric conversions such as `byte(65)` or `int(value)`
- Rune-oriented conversions such as `[]rune(string)` or `string([]rune)`
- Alias-aware conversion coverage beyond the currently supported predeclared types
- Implementation-specific capacity growth for `[]byte(string)` beyond the current exact-length allocation
- Integer-to-string historical conversions and the broader Go conversion matrix

## Interfaces and Extension Hooks

- `src/frontend/ast.rs` and `src/frontend/parser.rs`: conversion expressions are separate from ordinary calls and reuse `TypeRef`
- `src/semantic/model.rs` and `src/semantic/analyzer.rs`: checked conversions carry both the target type and a narrow conversion kind
- `src/bytecode/instruction.rs` and `src/bytecode/compiler.rs`: dedicated `convert` instructions preserve debuggability
- `src/runtime/value.rs` and `src/runtime/vm.rs`: conversions reuse byte-oriented string helpers and byte-slice helpers instead of reintroducing Rust `String` shortcuts
- Future conversion work should extend the explicit conversion model rather than adding builtin special cases
