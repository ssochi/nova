# Slice Expressions, String Slicing, Typed Zero Values, and Slice Allocation Research

## Goal

Capture the official Go behavior baseline for the currently targeted slice surface: simple slice expressions on slices and strings, string indexing, typed `var` zero values, nil-slice behavior, `cap`, `copy`, slice-backed `append` reuse, and `make([]T, len[, cap])`.

## Sources Reviewed

- Official Go language specification section `Slice expressions`
- Official Go language specification section `Index expressions`
- Official Go language specification section `Assignment statements`
- Official Go language specification section `Variable declarations`
- Official Go language specification section `Appending to and copying slices`
- Official Go language specification section `Length and capacity`
- Official Go language specification section `Making slices, maps and channels`
- Official Go builtin package documentation for `make`

## Confirmed Findings

- Go has simple slice expressions of the form `a[low:high]`; `low` and `high` may be omitted and default to `0` and `len(a)` respectively.
- Simple slice expressions are valid on strings, arrays, pointers to arrays, and slices. Full slice expressions `a[low:high:max]` are a distinct form.
- For slices, simple slice expressions may use an upper bound up to `cap(a)`, not just `len(a)`.
- For arrays or strings, valid simple-slice bounds satisfy `0 <= low <= high <= len(a)`.
- When a valid slice expression produces a slice, the result shares the operand's underlying array. Updating an element through one slice view is observable through overlapping views.
- Indexed assignment requires the left-hand side to be assignable. A slice element assignment such as `values[i] = x` replaces the current element value.
- `byte` is a predeclared alias for `uint8`.
- String slice expressions are byte-oriented in real Go, not rune-oriented.
- For a string operand, `text[i]` yields the non-constant byte value at index `i`, has type `byte`, and may not be assigned to.
- Out-of-range slice bounds or indexed writes fail at runtime in real Go.
- A `var` declaration with an explicit type and no initializer stores the zero value for that type.
- The zero value of a slice type is `nil`, with both length and capacity equal to zero.
- Appending to a nil slice is valid and yields a regular non-nil slice result.
- Copying between slices involving nil operands copies zero elements when one visible length is zero.
- `append` and `copy` are defined so their results do not depend on whether the referenced memory overlaps.
- `append` returns a slice of the same type. If the destination slice has enough spare capacity, the existing underlying array is reused; otherwise a new sufficiently large underlying array is allocated.
- `copy(dst, src)` returns the minimum of `len(dst)` and `len(src)` and requires matching slice element types, except for the special `[]byte` <- `string` case.
- The `[]byte` <- `string` `copy` special case copies raw bytes from the source string into the destination byte slice.
- `cap(slice)` returns the current slice capacity. For slices, `0 <= len(s) <= cap(s)` always holds.
- `make([]T, len)` allocates a new, non-nil slice with length and capacity both equal to `len`.
- `make([]T, len, cap)` allocates a new, non-nil slice with length `len` and capacity `cap`.
- Slice elements created by `make` are initialized to the zero value of the element type.
- `make` requires a slice, map, or channel type as its first argument in real Go; for slices, the length must not exceed the capacity.
- Negative slice lengths or capacities and length-greater-than-capacity failures are rejected; some cases are compile-time failures when the values are constant, and otherwise they fail at runtime.
- Real Go also accepts arrays, pointers to arrays, channels, and some generic type-parameter cases for `cap`, but those surfaces are outside the current compiler subset.

## Implementation Implications

- The current VM should keep the existing simple-slice form and extend it to strings without widening into full-slice syntax.
- Supporting slice expressions on `[]T` is feasible if runtime slices keep shared backing storage plus independent start/length metadata.
- Indexed assignment should reuse the same shared-slice representation so overlapping slice windows observe mutations.
- Supporting string slicing and indexing requires a byte-oriented runtime string representation instead of a Rust `String`-only model.
- `byte` needs to become a first-class semantic and runtime type so `text[i]` and `[]byte` storage stay explicit across semantic analysis, bytecode, and VM execution.
- The existing slice runtime already tracks capacity metadata, so `cap(slice)` and capacity-aware `append` should extend that path instead of adding a second storage model.
- `make([]T, len[, cap])` can reuse the current slice storage model if allocation reserves hidden capacity slots and fills the visible prefix with element zero values.
- The first argument to `make` is a type, not a value, so the compiler should model that path explicitly instead of forcing it through ordinary expression arguments.
- Explicit typed `var` declarations are a good entry point for nil slices because they provide type context without forcing untyped `nil` expression support yet.
- Zero-value synthesis can stay compile-time-driven for the current subset by lowering explicit typed declarations into concrete zero-producing instructions.
- `copy` should snapshot the visible source elements before writing so overlapping slice windows behave independently of aliasing order, matching the spec.
- Once `byte` exists, `copy([]byte, string)` is a high-value narrow special case because it makes byte slices usable without implementing general conversion syntax.
- `append(slice, string...)` should remain deferred until the parser and call model can represent variadic forwarding explicitly.

## Deferred Questions

- Whether later slice work should add compile-time constant folding strong enough to reject more invalid `make` sizes before runtime
- Whether a future allocation model should grow beyond slice-only `make` into maps, channels, or more Go-like append growth heuristics
- Whether later iterations should add general conversion syntax so `[]byte("text")` and `string(bytes)` stop relying on staged builtin special cases
