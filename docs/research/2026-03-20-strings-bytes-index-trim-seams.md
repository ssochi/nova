# Strings and Bytes Index/Trim Package Seams Research

## Goal

Record the official behavior baseline for the next `M3` package-backed slice around `strings` / `bytes` search and prefix/suffix trimming helpers.

## Sources Reviewed

- `go doc strings.Index`
- `go doc strings.HasSuffix`
- `go doc strings.TrimPrefix`
- `go doc strings.TrimSuffix`
- `go doc bytes.Index`
- `go doc bytes.HasSuffix`
- `go doc bytes.TrimPrefix`
- `go doc bytes.TrimSuffix`
- Local Go 1.21.5 probes under `/tmp/nova_pkg_probe.go` and `/tmp/nova_pkg_probe_alias.go`

## Confirmed Findings

- `strings.Index(s, substr)` and `bytes.Index(s, sep)` return the first match offset and `-1` when the needle is absent.
- Empty needles return `0` for both `strings.Index("nova", "")` and `bytes.Index([]byte("nova"), []byte(""))`.
- `strings.HasSuffix` and `bytes.HasSuffix` return `true` for empty suffixes, including empty or nil byte-slice inputs.
- `strings.TrimPrefix` / `strings.TrimSuffix` return the original string unchanged when the prefix or suffix does not match.
- `bytes.TrimPrefix` / `bytes.TrimSuffix` return the original slice unchanged on misses and a subslice view on hits; local probes confirmed the returned slice shares backing storage with the input.
- `bytes.TrimPrefix(nil, []byte{})` and `bytes.TrimSuffix(nil, []byte{})` preserve nilness rather than materializing a non-nil empty slice.
- The currently tempting `Split` / `SplitN` family is not a good next slice yet because empty separators are documented and implemented in terms of UTF-8 sequence boundaries, which would overstate compatibility while `nova-go` still models byte-oriented strings without rune semantics.

## Implementation Implications

- `strings.Index`, `strings.HasSuffix`, `strings.TrimPrefix`, and `strings.TrimSuffix` fit the current byte-oriented `StringValue` model without introducing rune-aware behavior.
- `bytes.Index`, `bytes.HasSuffix`, `bytes.TrimPrefix`, and `bytes.TrimSuffix` fit the current `SliceValue` model and can reuse shared-backing slice windows instead of copying.
- The byte-slice trim helpers should preserve nil-vs-empty distinctions just like the existing staged `bytes.Cut*` helpers.
- This slice can stay inside the existing metadata-backed package-function architecture with no AST or checked-model shape changes.

## Deferred Questions

- Add `Split` / `SplitN` only after the runtime deliberately models empty-separator UTF-8 semantics.
- Consider `LastIndex`, `IndexByte`, and wider trim helpers in a later package slice once this narrower search/trim family is stable.
