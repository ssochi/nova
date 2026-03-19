# Slice Expression and Assignment Research

## Goal

Capture the official Go behavior baseline for simple slice expressions and indexed assignment so the next slice-runtime expansion stays explicit and defensible.

## Sources Reviewed

- Official Go language specification section `Slice expressions`
- Official Go language specification section `Assignment statements`

## Confirmed Findings

- Go has simple slice expressions of the form `a[low:high]`; `low` and `high` may be omitted and default to `0` and `len(a)` respectively.
- Simple slice expressions are valid on strings, arrays, pointers to arrays, and slices. Full slice expressions `a[low:high:max]` are a distinct form.
- For slices, simple slice expressions may use an upper bound up to `cap(a)`, not just `len(a)`.
- When a valid slice expression produces a slice, the result shares the operand's underlying array. Updating an element through one slice view is observable through overlapping views.
- Indexed assignment requires the left-hand side to be assignable. A slice element assignment such as `values[i] = x` replaces the current element value.
- String slice expressions are byte-oriented in real Go, not rune-oriented.
- Out-of-range slice bounds or indexed writes fail at runtime in real Go.

## Implementation Implications

- The current VM should add only the simple slice form this round and reject full slice expressions explicitly.
- Supporting slice expressions on `[]T` is feasible if runtime slices keep shared backing storage plus independent start/length metadata.
- Indexed assignment should reuse the same shared-slice representation so overlapping slice windows observe mutations.
- String slice execution should stay deferred for now because the current runtime stores strings as Rust `String`, which does not model Go's byte-addressed string slicing cleanly.
- `append` and explicit capacity management remain separate concerns from this slice-window iteration and should stay documented as future work.

## Deferred Questions

- Whether a later runtime slice model should expose `cap` and nil slices directly once more builtin coverage lands
- Whether string runtime values should move to a byte-oriented representation before broader `strings` or slicing support expands
