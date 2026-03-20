# Empty Interface Any and Fmt Spread

## Goal

Add a first explicit empty-interface slice that introduces `any` / `interface{}` values, keeps nil-interface behavior correct, and unlocks `fmt` variadic spread calls over `[]any`.

## Constraints

- Keep the feature limited to the empty interface; no methods, assertions, or dynamic dispatch in this round.
- Keep boxing explicit in the checked and bytecode layers so `dump-bytecode` remains useful.
- Preserve the repository file-size ceiling by extracting helpers when the near-limit compiler or VM files need it.

## Current Scope

- Parse `interface{}` and resolve both `interface{}` and `any` into one explicit empty-interface type.
- Add a runtime interface value with nil-vs-boxed state instead of reusing raw `Value` directly.
- Add an explicit checked boxing node and matching bytecode instruction(s) for coercions into `any`.
- Support nil-interface zero values, interface-aware printing, staged interface equality, and `[]any` values.
- Allow explicit package spread only for staged variadic `fmt` helpers so `fmt.Println(args...)` works with `[]any`.

## Deferred Scope

- Non-empty interfaces, type assertions, type switches, and method sets.
- `recover`, even though this slice prepares the payload-carrier type it needs later.
- General package-function `...` support outside the staged `fmt` functions.
- Reflection-like formatting, richer `%` verbs, or interface-backed map keys.

## Interfaces and Extension Hooks

- `src/frontend/signature.rs` and `src/frontend/parser.rs` should keep `interface{}` explicit in the syntax model rather than erasing it into a named alias immediately.
- `src/semantic/model.rs` and `src/semantic/support.rs` should centralize empty-interface type identity plus boxing/coercion helpers so assignability rules do not leak across the analyzer.
- `src/bytecode/instruction.rs` should keep any boxing/runtime-interface operations explicit enough for `dump-bytecode`.
- `src/runtime/value.rs` should hold the nil-vs-boxed interface representation and interface-aware equality/rendering helpers.
- `src/runtime/vm/` should route interface comparisons through a dedicated helper so future `recover` or assertion work has one runtime seam to extend.
