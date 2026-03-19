# Slice Expressions, Assignment, Typed Zero Values, and Builtin Semantics Research

## Goal

Capture the official Go behavior baseline for the currently targeted slice surface: simple slice expressions, indexed assignment, typed `var` zero values, nil-slice behavior, `cap`, `copy`, and slice-backed `append` reuse.

## Sources Reviewed

- Official Go language specification section `Slice expressions`
- Official Go language specification section `Assignment statements`
- Official Go language specification section `Variable declarations`
- Official Go language specification section `Appending to and copying slices`
- Official Go language specification section `Length and capacity`

## Confirmed Findings

- Go has simple slice expressions of the form `a[low:high]`; `low` and `high` may be omitted and default to `0` and `len(a)` respectively.
- Simple slice expressions are valid on strings, arrays, pointers to arrays, and slices. Full slice expressions `a[low:high:max]` are a distinct form.
- For slices, simple slice expressions may use an upper bound up to `cap(a)`, not just `len(a)`.
- When a valid slice expression produces a slice, the result shares the operand's underlying array. Updating an element through one slice view is observable through overlapping views.
- Indexed assignment requires the left-hand side to be assignable. A slice element assignment such as `values[i] = x` replaces the current element value.
- String slice expressions are byte-oriented in real Go, not rune-oriented.
- Out-of-range slice bounds or indexed writes fail at runtime in real Go.
- A `var` declaration with an explicit type and no initializer stores the zero value for that type.
- The zero value of a slice type is `nil`, with both length and capacity equal to zero.
- Appending to a nil slice is valid and yields a regular non-nil slice result.
- Copying between slices involving nil operands copies zero elements when one visible length is zero.
- `append` and `copy` are defined so their results do not depend on whether the referenced memory overlaps.
- `append` returns a slice of the same type. If the destination slice has enough spare capacity, the existing underlying array is reused; otherwise a new sufficiently large underlying array is allocated.
- `copy(dst, src)` returns the minimum of `len(dst)` and `len(src)` and requires matching slice element types, except for the special `[]byte` <- `string` case.
- `cap(slice)` returns the current slice capacity. For slices, `0 <= len(s) <= cap(s)` always holds.
- Real Go also accepts arrays, pointers to arrays, channels, and some generic type-parameter cases for `cap`, but those surfaces are outside the current compiler subset.

## Implementation Implications

- The current VM should add only the simple slice form this round and reject full slice expressions explicitly.
- Supporting slice expressions on `[]T` is feasible if runtime slices keep shared backing storage plus independent start/length metadata.
- Indexed assignment should reuse the same shared-slice representation so overlapping slice windows observe mutations.
- String slice execution should stay deferred for now because the current runtime stores strings as Rust `String`, which does not model Go's byte-addressed string slicing cleanly.
- The existing slice runtime already tracks capacity metadata, so `cap(slice)` and capacity-aware `append` should extend that path instead of adding a second storage model.
- Explicit typed `var` declarations are a good entry point for nil slices because they provide type context without forcing untyped `nil` expression support yet.
- Zero-value synthesis can stay compile-time-driven for the current subset by lowering explicit typed declarations into concrete zero-producing instructions.
- `copy` should snapshot the visible source elements before writing so overlapping slice windows behave independently of aliasing order, matching the spec.
- The `[]byte` <- `string` special case for `copy` and `append(slice, string...)` should remain deferred until the compiler grows a byte-oriented runtime type.

## Deferred Questions

- Whether a later allocation-oriented plan should add `make` and type-valued builtin arguments on top of the typed zero-value declaration path
- Whether string runtime values should move to a byte-oriented representation before broader `strings` or slicing support expands
- Whether a future allocation model should introduce `make` plus spare-capacity growth heuristics beyond the minimal "reuse if capacity allows" rule
