# Empty Interface Any and Fmt Spread Research

## Goal

Verify the minimal real-Go behavior needed to stage empty-interface support through `any` / `interface{}` and to unlock `fmt.Print*` calls that consume `[]any` via explicit `...`.

## Sources Reviewed

- Local Go 1.21.5 probes run with `go run`
  - `any` and `interface{}` value declarations plus `%T` output
  - `fmt.Println(args...)` where `args` is `[]any`
  - zero-value `any` printing and `== nil`
  - interface equality with a comparable payload
  - interface equality with an uncomparable payload
  - single-result type assertions from `any` to comparable scalars, `[]byte`, and `any`
  - failed type assertions on nil interfaces and mismatched dynamic types
  - comma-ok type assertions on successful typed-nil payloads and nil-interface failures
  - type switches with single-type, multi-type, `nil`, and `any` cases
  - duplicate type-switch case diagnostics
  - compile-time rejection for non-interface type-switch guards
  - invalid type assertions on non-interface operands through `go build`
- Official Go language specification
  - `Type assertions` section in `https://go.dev/ref/spec`

## Confirmed Findings

- `any` behaves as the empty interface alias; `var left any = 1` and `var right interface{} = "go"` both compile and preserve the dynamic value type.
- `fmt.Println(args...)` works when `args` has type `[]any`; the observed output for `[]any{"prefix", 7, nil}` is `prefix 7 <nil>`.
- The zero value of `any` prints as `<nil>` through `fmt` and compares equal to `nil`.
- An interface value holding a comparable payload can compare directly to a matching concrete comparable value; `var value any = "boom"; value == "boom"` evaluates to `true`.
- Comparing an interface value whose dynamic payload is uncomparable can panic at runtime; `var value any = []int{1}; _ = value == value` fails with `panic: runtime error: comparing uncomparable type []int`.
- A type assertion `value.(T)` requires the operand to have interface type at compile time; `var x int = 7; _ = x.(int)` fails with `invalid operation: x (variable of type int) is not an interface`.
- Successful single-result assertions preserve the dynamic payload, including typed nil composites; `var boxed any = []byte(nil); boxed.([]byte) == nil` evaluates to `true`.
- Asserting an interface value to `any` succeeds when the interface holds any dynamic value and returns that boxed payload as another interface value; the observed `%T` / `%v` output for `var x any = 7; x.(any)` is `int 7`.
- A failed single-result assertion panics at runtime with interface-conversion wording that distinguishes nil interfaces from mismatched dynamic types:
  - `var x any; _ = x.(string)` panics with `interface conversion: interface {} is nil, not string`
  - `var x any = "go"; _ = x.([]byte)` panics with `interface conversion: interface {} is string, not []uint8`
- A comma-ok assertion `value, ok := x.(T)` returns the asserted value plus `true` on success and the zero value of `T` plus `false` on failure; typed-nil payloads still count as success:
  - `var x any = []byte(nil); value, ok := x.([]byte)` yields `[]uint8 true true`
  - `var x any; value, ok := x.(string)` yields `"" false`
- A type switch guard may appear as `switch x.(type)` or `switch value := x.(type)` and the guard operand must have interface type; `var x int; switch x.(type) {}` fails with `x (variable of type int) is not an interface`.
- `case nil` matches only a nil interface value, not a boxed typed-nil payload. A typed-nil slice boxed in `any` matches `case []byte`, while a zero-value `any` matches `case nil`.
- In a type-switch clause binding:
  - a single concrete case such as `case []byte:` gives the clause binding that concrete type
  - a multi-type case such as `case string, bool:` keeps the clause binding at the guard's interface type
  - `case any:` also keeps the clause binding at interface type and matches any non-nil dynamic payload in the current empty-interface model
- Current Go rejects duplicate type-switch cases, including duplicate `nil`, at compile time.

## Implementation Implications

- The runtime needs an explicit nil-interface state separate from boxed typed-nil composite values such as `[]int(nil)`, otherwise `value == nil` will be wrong.
- Empty-interface support should be explicit in the checked and bytecode layers through boxing rather than treated as a silent assignability rule.
- The staged equality surface can include:
  - `any` or `interface{}` against `nil`
  - `any` or `interface{}` against concrete comparable scalar values
  - `any` / `interface{}` against other interface values, with runtime panic on uncomparable dynamic payloads of the same boxed runtime type
- The runtime printer should render nil interface values as `<nil>` and boxed interface values through the underlying value display path.
- Package-call spread should stay narrow in this round: support the staged variadic `fmt` functions once `[]any` exists instead of claiming general package `...` support.
- The first type-assertion slice can stay narrow and still unlock real interface use:
  - keep assertion syntax explicit in the AST and checked model instead of pretending it is a call or conversion
  - require an `any` / `interface{}` operand in semantic analysis
  - allow currently modeled destination runtime types, including `int`, `byte`, `bool`, `string`, `[]T`, `map[K]V`, `chan T`, and `any`
  - preserve typed-nil slice/map/chan payloads when the dynamic type matches
  - lower success/failure through dedicated bytecode and VM interface helpers so `dump-bytecode` stays readable
- The follow-up interface-consumption slice should keep comma-ok assertions explicit as statement-level AST / checked / bytecode forms rather than smuggling them through generic multi-result expression plumbing.
- The first staged type-switch implementation can stay narrow and still reflect real Go behavior if it:
  - keeps type switches explicit instead of overloading expression switches
  - requires an `any` / `interface{}` guard operand in semantic analysis
  - supports the currently modeled runtime-type surface plus `case nil`
  - keeps clause binding types explicit per clause instead of erasing everything back to `any`
  - diagnoses duplicate staged case types after canonicalizing `interface{}` and `any`

## Deferred Questions

- Whether the project should expose empty-interface conversions as a dedicated checked node or reuse a generalized boxing node for all `any` coercions.
- How comma-ok type assertions and type switches should share runtime type-check helpers without introducing first-class tuple runtime values.
- How broader interface equality, non-empty interfaces, method sets, and `recover` should build on top of this slice without reworking the runtime representation again.
