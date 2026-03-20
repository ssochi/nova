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
  - `tests/cli_builtin_clear.rs` and `tests/cli_builtin_clear_diagnostics.rs`: focused coverage for builtin `clear(slice|map)` success and failure paths
  - `tests/cli_panic.rs` and `tests/cli_panic_diagnostics.rs`: focused coverage for builtin `panic`, explicit panic bytecode, panic-aware unwind, and invalid panic arity
  - `tests/cli_recover.rs` and `tests/cli_recover_diagnostics.rs`: focused coverage for builtin `recover()`, direct deferred recovery, helper/deferred-builtin non-recovery, and invalid recover arity
  - `tests/cli_any_interface.rs` and `tests/cli_any_interface_diagnostics.rs`: focused coverage for staged empty-interface values, explicit boxing, `fmt` `[]any...` spread, and invalid spread diagnostics
  - `tests/cli_type_assertions.rs` and `tests/cli_type_assertions_diagnostics.rs`: focused coverage for staged single-result `x.(T)` assertions, typed-nil assertion payloads, runtime mismatch panics, invalid operands, and rejected `.(type)` syntax
  - `tests/cli_type_switches.rs` and `tests/cli_type_switches_diagnostics.rs`: focused coverage for staged comma-ok assertions, explicit type-switch execution, and invalid type-switch guards/cases
  - focused one-feature files such as `tests/cli_strings_bytes_last_index.rs` and `tests/cli_strings_bytes_last_index_diagnostics.rs` when the large baseline integration files are already near the repository size ceiling
  - `tests/support/mod.rs`: shared CLI invocation and temporary source helpers
- When a feature adds an explicit staged surface such as `range`, comma-ok `map` lookup, comma-ok type assertions, type switches, or `if` headers, cover both `dump-ast` and `dump-bytecode` so users can inspect it without reading the implementation.
- When a feature relies on staged lowering rather than a dedicated instruction for every case, use CLI `dump-bytecode` assertions on hidden locals or explicit helper instructions so the new path remains discoverable.
- Manual serial CLI validation
  - Real `cargo run -- ...` commands remain the evidence trail for subcommand experience

## Update Rules

- Add a unit test first when a change lives in one layer and can be validated without the full CLI.
- Add or update CLI integration coverage when the user-visible compiler surface changes.
- Prefer a new focused integration file over extending `tests/cli_execution.rs` or `tests/cli_diagnostics.rs` once those broad files are already near 1000 lines.
- Keep real CLI validation serial when collecting evidence for reports.
- Do not let `check` assertions depend on `run`-only entrypoint assumptions.

## Known Gaps

- There is no fixture-driven golden test system yet; integration assertions still live in Rust source.
- Cross-file package loading and richer runtime data structures will likely need additional support helpers.
- Performance or stress validation is not yet part of the automated loop.
