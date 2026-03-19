# tests Directory Conventions

- This directory is used to validate runner behavior and framework integrity.
- Current test focus:
  - Prompt rereading and argument passthrough in `codex_loop.py`
  - Existence of key framework directories and files
  - No card-specific default terminology leaking into generic documents
- When runner behavior or root-level document contracts change, the tests must be synchronized.
