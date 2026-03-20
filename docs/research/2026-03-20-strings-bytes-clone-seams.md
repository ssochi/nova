# Strings and Bytes Clone Package Seams Research

## Goal

Record the official behavior baseline for the next `M3` package-backed slice around `strings.Clone` and `bytes.Clone`.

## Sources Reviewed

- `go doc strings.Clone`
- `go doc bytes.Clone`
- Local Go 1.21.5 probe under `/tmp/nova-go-clone-probe.go`

## Confirmed Findings

- `strings.Clone(s)` returns a fresh copy of the input string, but the only currently observable behavior in `nova-go` is that the returned string has the same byte content as `s`.
- `strings.Clone("")` returns the empty string.
- `bytes.Clone(b)` returns a copy of `b[:len(b)]`.
- `bytes.Clone(nil)` returns `nil`.
- Local probes confirmed `bytes.Clone([]byte{})` returns a non-nil empty slice, so nil and non-nil empty inputs must stay distinguishable.
- Local probes also confirmed `bytes.Clone([]byte("nova"))` preserves the source bytes while allowing implementation-defined spare capacity.

## Implementation Implications

- `strings.Clone` fits the current byte-oriented `StringValue` model with no syntax or checked-model changes.
- `bytes.Clone` fits the existing `[]byte` runtime path, but the helper must preserve the staged nil-vs-empty distinction instead of routing both cases through the same empty-slice constructor.
- This slice can stay inside the existing metadata-backed package-function architecture with explicit `call-package` bytecode.
- Because the VM does not expose allocation identity, `strings.Clone` only needs content-preserving behavior, while `bytes.Clone` needs content plus nil-state preservation.

## Deferred Questions

- Do not claim pointer-identity or allocation-uniqueness guarantees until the runtime exposes observability that would make those promises meaningful.
- Keep rune-aware or UTF-8-sequence-aware `strings` / `bytes` helpers in separate research notes instead of bundling them into this slice.
