# CLI Blackbox Playtest

## Trigger Conditions

Use this SOP when a milestone is declared complete or when a CLI-facing change needs a real user-path walkthrough.

## Prerequisite Checks

- The CLI binary can be launched through `cargo run -- <args>`.
- At least one realistic input program exists under a stable location such as `examples/`.
- Validation commands have already covered obvious build failures so the playtest can focus on experience.

## Execution Steps

1. Choose at least one happy-path sample and one error-path sample.
2. Run the CLI exactly as a user would, such as:
   - `cargo run -- run examples/hello.go`
   - `cargo run -- dump-bytecode examples/hello.go`
   - `cargo run -- run <broken-source>`
   - Run the commands serially when using `cargo run -- ...`; parallel cargo invocations add lock noise that pollutes the observed CLI output.
3. Observe and record:
   - command
   - actual output
   - whether the output is understandable without reading the code
   - any friction in discovery, ergonomics, or diagnostics
4. Confirm the CLI can be used in sequence, not just as isolated commands.
5. Write an experience report under `docs/reports/experience/`.

## Completion Criteria

- At least one full happy path is executed through the real CLI.
- At least one failure path is exercised through the real CLI.
- The experience report includes strengths, issues, and next recommendations.

## Common Mistakes

- Replacing the real CLI with direct library calls
- Running multiple `cargo run -- ...` commands in parallel and mistaking cargo lock output for user-facing CLI behavior
- Recording only pass/fail without noting user-facing clarity
- Declaring a milestone complete without exercising an error path

## Related Entry Points

- `docs/reports/experience/`
- `examples/`
