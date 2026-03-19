# roadmap/milestones Directory Conventions

This directory stores milestones. Milestones are responsible for defining the stage main line and completion criteria, and do not directly carry single-round implementation details.

## Directory Responsibilities

- `index.md`: current milestone, milestone queue, and switching rules
- `M*.md`: individual milestone documents, recording goals, completion criteria, related plans, current risks, and next-round recommendations

## When It Must Be Updated

- When opening, closing, or switching a milestone
- When the current milestone's main plan, completion criteria, or stage goals change
- When milestone numbering, naming, or status conventions change

## File Format Convention

- The index file is fixed as `index.md`
- Individual milestone files use `M<number>-<topic>.md`
- Milestone documents must include at least: status, goals, completion criteria, related plans, current risks, and next-round recommendations

## Document Structure

- `index.md` must always include:
  - `# Milestone Index`
  - `## Current Milestone`
  - `## Planned Milestones`
  - `## Switching Rules`
- `M<number>-<topic>.md` must always include:
  - Title: `# M<number>: <topic>`
  - `- Status:`
  - `- Current Main Plan:`
  - `## Goals`
  - `## Completion Criteria`
  - `## Task Breakdown`
  - `## Related Plans`
  - `## Current Risks`
  - `## Next-Round Recommendations`

## File Index

- `AGENTS.md`: this directory convention
- `index.md`: milestone index
