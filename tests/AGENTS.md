# tests Directory Conventions

- This directory is used to validate runner behavior and framework integrity.
- Current test focus:
  - Unit coverage inside `src/` for narrow parser, semantic, builtin, and VM behaviors
  - CLI subcommands such as `check`, `dump-ast`, `dump-bytecode`, and `run`
  - End-to-end compiler and VM behavior for the currently supported Go subset
  - Semantic error coverage for invalid source programs
- Test structure:
  - `support/`: shared CLI and temporary-source helpers for integration tests
  - `cli_execution.rs`: happy-path CLI and dump-surface coverage
  - `cli_diagnostics.rs`: invalid-program and diagnostic coverage
  - `cli_strings_bytes_clone.rs`: focused CLI coverage for the `strings.Clone` / `bytes.Clone` helper slice
  - `cli_strings_bytes_clone_diagnostics.rs`: focused diagnostic coverage for mistyped `strings.Clone` / `bytes.Clone` calls
  - `cli_strings_bytes_last_index.rs`: focused CLI coverage for the `strings` / `bytes` last-index and byte-search helper slice
  - `cli_strings_bytes_last_index_diagnostics.rs`: focused diagnostic coverage for mistyped `strings` / `bytes` last-index and byte-search calls
- When the supported language slice, CLI contract, or framework document rules change, the tests must be synchronized.
