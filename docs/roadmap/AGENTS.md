# roadmap Directory Conventions

This directory is responsible for stage progress state. It answers "What is the current milestone?" "What is the current active plan?" and "Which plans have already been completed and archived?"

## Directory Responsibilities

- `milestones/`: stage main line, completion criteria, current risks, and next-round suggestions
- `plans/`: active plans, execution context, task breakdown
- `archive/`: complete archives of closed plans

## When It Must Be Updated

- When opening, switching, or closing an active plan
- When opening, switching, or closing a milestone
- When the plan and milestone indexes can no longer accurately describe the current state
- When the responsibility boundaries of `milestones/`, `plans/`, and `archive/` change

## File Format Convention

- `milestones/` is responsible for stage goals and does not write specific implementation steps
- `plans/` is responsible for the current round's execution surface, and each plan is packaged in a directory
- `archive/` stores closed plans and retains `plan.md`, `todo.md`, and `context.md`
- `index.md` must be able to directly answer the current main line and current execution surface

## Document Structure

- `milestones/index.md` must always include:
  - `# Milestone Index`
  - `## Current Milestone`
  - `## Planned Milestones`
  - `## Switching Rules`
- `plans/index.md` must always include:
  - `# Active Plan Index`
  - `## Current Active Plan`
  - `## Recent Updates`
- If a stage overview document is added at the `roadmap/` root in the future, it must include at least:
  - Background
  - Current conclusions
  - Impact on lower-level `milestones/` or `plans/`

## File Index

- `AGENTS.md`: this directory convention
- `milestones/AGENTS.md`: milestone subdirectory convention
- `milestones/index.md`: current milestone index
- `plans/AGENTS.md`: active plan subdirectory convention
- `plans/index.md`: active plan index
- `archive/AGENTS.md`: archive subdirectory convention
