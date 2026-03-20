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
- The first slice should stop short of comma-ok assertions and type switches; those need explicit multi-result and statement-surface planning instead of being hidden behind generic call results.

## Deferred Questions

- Whether the project should expose empty-interface conversions as a dedicated checked node or reuse a generalized boxing node for all `any` coercions.
- How comma-ok type assertions and type switches should build on the same checked and bytecode representation without introducing first-class tuple runtime values.
- How broader interface equality, non-empty interfaces, method sets, and `recover` should build on top of this slice without reworking the runtime representation again.
