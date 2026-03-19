# docs Directory Conventions

This directory stores the persistent documentation for the current repository. It carries framework governance, technical explanations, the plan system, validation evidence, and SOPs, and is not responsible for replacing all business facts of the target project itself.

You do not need to be limited by the current directories. As long as a new type of documentation needs to be included, create the corresponding directory yourself.
## Directory Responsibilities

- `design/`: feature and subsystem design baselines
- `roadmap/`: milestones, active plans, archives, and roadmap progress state
- `reports/`: validation and real experience evidence
- `research/`: external behavior baselines and compatibility research
- `sop/`: high-frequency, reusable, and easy-to-miss processes
- `tech/`: framework structure, boundary descriptions, and migration mapping

## When It Must Be Updated

- When adding, migrating, or deleting first-level directories or stable documents under `docs/`
- When the responsibility boundary of the `docs/` root directory changes

## File Format Convention

- The `docs/` root should only contain cross-domain explanations and the main index, not pile up specific implementation process details
- Directory convention files are uniformly named `AGENTS.md`

## Document Structure

- If a new cross-domain master index document is added under the `docs/` root, it must include at least:
  - Document purpose
  - Subdirectory entry points
  - Current applicable boundary
  - The places that need to be written back into subdirectories

## File Index (only important files and all directories)

- `design/`: design documentation root
- `roadmap/`: roadmap documentation root
- `reports/`: validation and experience report root
- `research/`: external behavior research and compatibility notes root
- `sop/`: SOP documentation root
- `tech/`: technical documentation root
