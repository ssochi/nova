# Testing Strategy

## Purpose

Describe the current layered validation strategy so later iterations can expand coverage without collapsing everything into one integration-heavy file.

## Current Layers

- Unit tests inside `src/`
  - Parser coverage for narrow syntax additions
  - Semantic coverage for type rules and diagnostics
  - Builtin-contract coverage for argument validation
  - VM coverage for direct instruction execution
- CLI integration tests under `tests/`
  - `tests/cli_execution.rs`: happy-path runs plus `dump-*` inspection surfaces
  - `tests/cli_diagnostics.rs`: invalid programs and diagnostic expectations
  - `tests/support/mod.rs`: shared CLI invocation and temporary source helpers
- When a feature relies on staged lowering rather than a dedicated instruction for every case, use CLI `dump-bytecode` assertions on hidden locals or explicit helper instructions so the new path remains discoverable.
- Manual serial CLI validation
  - Real `cargo run -- ...` commands remain the evidence trail for subcommand experience

## Update Rules

- Add a unit test first when a change lives in one layer and can be validated without the full CLI.
- Add or update CLI integration coverage when the user-visible compiler surface changes.
- Keep real CLI validation serial when collecting evidence for reports.
- Do not let `check` assertions depend on `run`-only entrypoint assumptions.

## Known Gaps

- There is no fixture-driven golden test system yet; integration assertions still live in Rust source.
- Cross-file package loading and richer runtime data structures will likely need additional support helpers.
- Performance or stress validation is not yet part of the automated loop.
