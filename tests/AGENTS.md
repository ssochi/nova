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
- When the supported language slice, CLI contract, or framework document rules change, the tests must be synchronized.
