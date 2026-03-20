# Strings and Bytes LastIndex / Byte Search Research

## Goal

Record the official behavior baseline for the next `M3` package-backed slice around byte-oriented `strings` / `bytes` tail-search helpers.

## Sources Reviewed

- `go doc strings.LastIndex`
- `go doc strings.IndexByte`
- `go doc strings.LastIndexByte`
- `go doc bytes.LastIndex`
- `go doc bytes.IndexByte`
- `go doc bytes.LastIndexByte`
- Local Go 1.21.5 probe under `/tmp/nova-go-byte-search-probe.go`

## Confirmed Findings

- `strings.LastIndex(s, substr)` and `bytes.LastIndex(s, sep)` return the last match offset and `-1` when the needle is absent.
- Empty needles return the source length for `strings.LastIndex("nova", "")` and `bytes.LastIndex([]byte("nova"), []byte(""))`.
- `strings.IndexByte(s, c)` returns the first byte offset of `c` in `s`, or `-1` when absent.
- `strings.LastIndexByte(s, c)` returns the last byte offset of `c` in `s`, or `-1` when absent.
- `bytes.IndexByte(b, c)` and `bytes.LastIndexByte(b, c)` have the same first/last byte-offset behavior on `[]byte`.
- Local probes confirmed `bytes.LastIndex(nil, []byte("")) == 0` and `bytes.LastIndexByte(nil, 'x') == -1`, so nil byte slices behave like empty slices for these integer-returning helpers.
- All six helpers are byte-oriented and do not require rune-aware or UTF-8-sequence-aware semantics.

## Implementation Implications

- `strings.LastIndex`, `strings.IndexByte`, and `strings.LastIndexByte` fit the current byte-oriented `StringValue` model without widening the syntax or checked-expression surface.
- `bytes.LastIndex`, `bytes.IndexByte`, and `bytes.LastIndexByte` fit the existing `SliceValue` byte-slice view and only need integer-returning package dispatch.
- This slice can reuse the existing metadata-backed package contract model and explicit `call-package` bytecode path.
- Because all helpers return `int`, byte-slice nil-vs-empty distinctions stay internal and do not require new runtime value modeling.

## Deferred Questions

- `strings.LastIndexAny`, `strings.IndexAny`, and related Unicode-sensitive helpers should stay deferred until rune semantics are modeled deliberately.
- `bytes.LastIndexAny`, `bytes.IndexAny`, and wider class-based search helpers should stay deferred for the same reason.
- If broader package-backed search coverage is needed later, `Compare` / `EqualFold` / split-family helpers should each get a fresh scope check instead of being bundled opportunistically.
