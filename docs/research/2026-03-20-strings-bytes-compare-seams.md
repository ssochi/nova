# Strings and Bytes Compare Seams Research

## Goal

Record the official behavior baseline for the next `M3` package-backed slice around `strings.Compare` and `bytes.Compare`.

## Sources Reviewed

- `go doc strings.Compare`
- `go doc bytes.Compare`
- Local Go 1.21.5 probe under `/tmp/nova-go-compare-probe.go`

## Confirmed Findings

- `strings.Compare(a, b)` compares two strings lexicographically and returns `0` when equal, `-1` when `a < b`, and `+1` when `a > b`.
- `bytes.Compare(a, b)` compares two byte slices lexicographically and returns the same `0` / `-1` / `+1` result set.
- `bytes.Compare` treats a nil slice the same as an empty slice, so `bytes.Compare(nil, []byte{}) == 0`.
- Local probes confirmed the expected ordering cases:
  - `strings.Compare("go", "vm") == -1`
  - `strings.Compare("vm", "go") == 1`
  - `bytes.Compare(nil, []byte("go")) == -1`
  - `bytes.Compare([]byte("vm"), nil) == 1`
- Both helpers are byte-oriented and do not require rune-aware or UTF-8-sequence-aware semantics.

## Implementation Implications

- `strings.Compare` fits the current byte-oriented `StringValue` runtime model and only needs a fixed-arity typed package contract plus integer-returning VM dispatch.
- `bytes.Compare` fits the existing byte-slice runtime path and can reuse the current `[]byte` extraction behavior that already treats nil and empty slices equivalently for content-based helpers.
- This slice does not require AST changes, checked-model shape changes, or new bytecode instructions beyond the existing `call-package` path.
- The new helpers should remain visible through `dump-ast`, `dump-bytecode`, and `check` without any hidden lowering.

## Deferred Questions

- `strings.EqualFold` and other case-folding helpers should stay deferred until text semantics are planned deliberately.
- `Compare`-adjacent helpers that depend on rune or Unicode classes should stay deferred until the runtime models those semantics honestly.
- If broader comparison coverage is needed later, each new helper should get its own scope check instead of being bundled opportunistically.
