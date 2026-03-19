# Make-Based Slice Allocation CLI Playtest

## Basic Context

- Date: `2026-03-20`
- Related milestone: `M3-standard-library-and-runtime-model`
- Related plan: `2026-03-20-02-39-55-make-slice-allocation`
- Entry surface: `run`, `dump-ast`, `dump-bytecode`, `check`

## Experience Path

1. Ran `cargo run -- run examples/make_slices.go` to validate slice allocation, reslicing into spare capacity, zero-filled elements, and append reuse through the normal execution path.
2. Ran `cargo run -- dump-ast examples/make_slices.go` to inspect whether the type-argument builtin still reads like ordinary Go source in the CLI.
3. Ran `cargo run -- dump-bytecode examples/make_slices.go` to confirm the allocation path stays visible and debuggable at the VM-facing surface.
4. Ran `cargo run -- check /tmp/nova-go-bad-make.go` to inspect the failure path for invalid `make` bounds.

## Positive Experience

- The new allocation path feels like a real Go step forward because slice creation no longer depends on literals, nil slices, or append-only growth tricks.
- The AST output remains readable; `make([]int, 2, 4)` appears almost exactly as written instead of collapsing into a special internal form.
- The bytecode dump is still actionable because `make-slice ...` makes the allocation decision obvious without reading compiler internals.
- The spare-capacity story feels coherent in the CLI: users can allocate, reslice into hidden capacity, observe zero values, and then append into the same backing storage.

## Issues and Severity

- Medium: `make` is slice-only for now, so Go users will still hit gaps on map and channel allocation.
- Medium: constant-bound rejection is intentionally narrow; more Go-like constant evaluation for `make` sizes is still missing.
- Low: the CLI clearly shows local names in bytecode dumps, but it still does not surface local types alongside the new allocation instructions.

## Conclusion and Next Recommended Steps

This round materially improves the core slice workflow because programs can now allocate non-nil slices with chosen length and capacity through the normal CLI path. The next strongest `M3` follow-up is to decide between byte-oriented string work for string slicing and `[]byte`-adjacent compatibility, or a broader allocation / runtime expansion such as map-channel groundwork and stronger constant handling.
