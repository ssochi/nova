# Bootstrap CLI Playtest

## Basic Context

- Date: `2026-03-19`
- Entry point: `cargo run -- <subcommand>`
- Scope: bootstrap VM milestone closeout playtest

## Experience Path

1. Ran `cargo run -- run examples/hello.go` to verify the primary happy path.
2. Ran `cargo run -- dump-bytecode examples/arithmetic.go` to inspect the generated stack program.
3. Ran `cargo run -- run /tmp/nova-go-missing-main.TNTfLo.go` with a source missing `main` to validate the failure path.

## Positive Experience

- The CLI surface is small and discoverable.
- The happy path is fast and prints the program result directly.
- The bytecode dump is readable enough to support debugging the VM pipeline.
- The missing-entry failure is explicit about package/function expectations.
- The same executable supports syntax checking, inspection, and execution without changing tools.

## Issues and Severity

- Medium: the language subset is still much smaller than real Go, so unsupported constructs fail early.
- Medium: diagnostics do not yet show source excerpts or multi-location context.
- Low: command help currently appears only through usage errors rather than a dedicated `--help` success path.

## Conclusion and Next Recommended Steps

The CLI is usable as a real bootstrap execution path for the supported subset. The next iteration should expand the frontend and semantic pipeline so the VM can execute functions and control flow, while preserving the current command ergonomics.
