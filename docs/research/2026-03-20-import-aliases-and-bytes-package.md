# Import Aliases and Bytes Package Seam

## Goal

Lock the official behavior baseline for grouped import declarations, explicit import aliases, and a staged `bytes` package seam so `nova-go` can extend real-project compatibility without implying filesystem package loading, dot imports, or broader interface/error support.

## Sources Reviewed

- Go language specification: import declarations and import specs (`https://go.dev/ref/spec`)
- Go package docs: `bytes` package API contracts (`https://pkg.go.dev/bytes@go1.25.4`)
- Local behavior probes with `go version go1.21.5 darwin/arm64`

## Confirmed Findings

- Go import declarations allow either one `ImportSpec` or a parenthesized group of `ImportSpec` entries.
- Each `ImportSpec` may optionally start with `.` or a package name alias before the import path string.
- Without an alias, the imported binding defaults to the package name exposed by the imported package.
- Alias imports bind the package under the chosen file-block name; grouped imports do not change binding behavior, they only compress multiple specs under one `import (...)` declaration.
- Dot imports and blank imports are part of the real grammar, but they imply additional name-resolution and side-effect behavior that the current single-file model does not support safely.
- `bytes.Equal(a, b)` reports equality by byte content and treats a nil argument as equivalent to an empty slice.
- `bytes.Contains(s, subslice)` reports whether `subslice` appears in `s`; an empty `subslice` matches even when `s` is nil.
- `bytes.HasPrefix(s, prefix)` reports whether `s` begins with `prefix`; an empty prefix matches even when `s` is nil.
- Local Go probes confirm:
  - `bytes.Equal(nil, []byte{}) == true`
  - `bytes.Equal(nil, nil) == true`
  - `bytes.Contains(nil, []byte{}) == true`
  - `bytes.Contains(nil, []byte("a")) == false`
  - `bytes.HasPrefix(nil, []byte{}) == true`
  - `bytes.HasPrefix(nil, []byte("a")) == false`
  - `bytes.Join(nil, sep)` and `bytes.Join([][]byte{}, sep)` both produce `[]byte{}`
  - `bytes.Repeat([]byte("go"), 0)` produces `[]byte{}`
  - `bytes.Repeat([]byte("go"), -1)` panics in real Go with `bytes: negative Repeat count`
- `bytes.Join` takes `[][]byte` plus a separator `[]byte` and returns a freshly joined `[]byte`.
- `bytes.Repeat` returns a fresh `[]byte`; negative counts fail, while zero returns an empty byte slice.

## Implementation Implications

- The staged parser can add grouped imports and explicit package-name aliases while continuing to reject dot/blank imports with targeted diagnostics.
- Import bindings should stay explicit in the AST so `dump-ast` can render both grouped and alias forms instead of normalizing them away invisibly.
- Semantic import lookup should resolve by the bound name, not only by the package default name, so alias-aware selector calls reuse the existing package-call path.
- The staged `bytes` seam can remain metadata-backed alongside `fmt` and `strings`; it does not require filesystem loading or analyzer-specific type checks.
- `bytes.Join` needs nested byte-slice support through the existing generic `[]T` type model and runtime slice values, but it should stay isolated to the package seam rather than turning into a general deep-composite feature push.
- The VM can map `bytes.Repeat` failures into the current runtime-error path instead of panic/recover modeling, but that approximation must stay documented.

## Deferred Questions

- When the project adds dot imports, blank imports, or real package graphs, should import bindings move from a simple map into a richer file-block package model?
- When panic/recover semantics exist, should package-backed runtime failures such as `bytes.Repeat` negative counts become panic-accurate instead of runtime errors?
