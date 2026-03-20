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

## Confirmed Findings

- `any` behaves as the empty interface alias; `var left any = 1` and `var right interface{} = "go"` both compile and preserve the dynamic value type.
- `fmt.Println(args...)` works when `args` has type `[]any`; the observed output for `[]any{"prefix", 7, nil}` is `prefix 7 <nil>`.
- The zero value of `any` prints as `<nil>` through `fmt` and compares equal to `nil`.
- An interface value holding a comparable payload can compare directly to a matching concrete comparable value; `var value any = "boom"; value == "boom"` evaluates to `true`.
- Comparing an interface value whose dynamic payload is uncomparable can panic at runtime; `var value any = []int{1}; _ = value == value` fails with `panic: runtime error: comparing uncomparable type []int`.

## Implementation Implications

- The runtime needs an explicit nil-interface state separate from boxed typed-nil composite values such as `[]int(nil)`, otherwise `value == nil` will be wrong.
- Empty-interface support should be explicit in the checked and bytecode layers through boxing rather than treated as a silent assignability rule.
- The staged equality surface can include:
  - `any` or `interface{}` against `nil`
  - `any` or `interface{}` against concrete comparable scalar values
  - `any` / `interface{}` against other interface values, with runtime panic on uncomparable dynamic payloads of the same boxed runtime type
- The runtime printer should render nil interface values as `<nil>` and boxed interface values through the underlying value display path.
- Package-call spread should stay narrow in this round: support the staged variadic `fmt` functions once `[]any` exists instead of claiming general package `...` support.

## Deferred Questions

- Whether the project should expose empty-interface conversions as a dedicated checked node or reuse a generalized boxing node for all `any` coercions.
- How broader interface equality, type assertions, method sets, and `recover` should build on top of this slice without reworking the runtime representation again.
