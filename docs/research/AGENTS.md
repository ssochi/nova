# research Directory Conventions

This directory stores research notes that inform roadmap choices and implementation slices. Research documents are inputs to design and planning, not replacements for feature specs or validation reports.

## Directory Responsibilities

- Record external behavior baselines, comparisons, and capability surveys that affect project decisions
- Capture what was verified, what remains uncertain, and how the research should influence implementation scope
- Provide indexed research entry points that later agents can quickly reuse

## When It Must Be Updated

- When a plan depends on external behavior research or compatibility notes
- When a stable research note is added, retired, or materially revised
- When the directory needs a stronger indexing or naming convention

## File Format Convention

- Research files use `YYYY-MM-DD-<topic>.md`
- Each note should clearly separate confirmed facts, chosen scope, and deferred questions
- Research notes may cite official sources, but should summarize the conclusions instead of copying large passages

## Document Structure

- Each research note should include:
  - `# <Research Topic>`
  - `## Goal`
  - `## Sources Reviewed`
  - `## Confirmed Findings`
  - `## Implementation Implications`
  - `## Deferred Questions`

## File Index

- `AGENTS.md`: this directory convention
- `2026-03-20-strings-package-contracts.md`: official behavior baseline for the first `strings` package seam
- `2026-03-20-slice-expressions-and-assignment.md`: official behavior baseline for the current slice and string-window surface, including string indexing, typed zero values, `make`, `cap`, `copy`, and append-capacity semantics
- `2026-03-20-map-runtime-groundwork.md`: official behavior baseline for staged `map[K]V` support with `make`, `len`, indexing, and assignment
