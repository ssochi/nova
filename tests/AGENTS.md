# tests Directory Conventions

- This directory is used to validate runner behavior and framework integrity.
- Current test focus:
  - CLI subcommands such as `check`, `dump-ast`, `dump-bytecode`, and `run`
  - End-to-end compiler and VM behavior for the currently supported Go subset
  - Semantic error coverage for invalid source programs
- When the supported language slice, CLI contract, or framework document rules change, the tests must be synchronized.
