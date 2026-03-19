# roadmap/plans Directory Conventions

This directory stores active plans. Active plans are responsible for answering "What exactly is being done in this round, how far has it progressed, and what should the next agent continue with?"

## Directory Responsibilities

- `index.md`: current active plan and recent updates
- `<timestamp>-<plan-id>/plan.md`: plan goals, scope, phase breakdown, acceptance criteria, and risks
- `<timestamp>-<plan-id>/todo.md`: task list and status
- `<timestamp>-<plan-id>/context.md`: completed steps, current status, and key context for the next round

## When It Must Be Updated

- When opening, switching, or closing an active plan
- When the plan's goals, scope, methods, or priorities change significantly
- When the todo status or key context of the current plan changes

## File Format Convention

- Plan directories are uniformly named `YYYY-MM-DD-HH-MM-SS-<plan-id>`
- Each plan directory must contain `plan.md`, `todo.md`, and `context.md` at the same time
- `todo.md` uses explicit status markers, such as `[done]` and `[todo]`
- `context.md` records facts step by step and does not write vague conclusions

## Document Structure

- `index.md` must always include:
  - `# Active Plan Index`
  - `## Current Active Plan`
  - `## Recent Updates`
- `plan.md` must always include:
  - `# Plan: <topic>`
  - `## Basic Information`
  - `## Goals`
  - `## Scope`
  - `## Non-Goals`
  - `## Phase Breakdown`
  - `## Acceptance Criteria`
  - `## Risks`
- `todo.md` is fixed as a task list, with a status marker on each line:
  - `- [todo] ...`
  - `- [in_progress] ...`
  - `- [done] ...`
  - `- [blocked] ...`
- `context.md` must always include:
  - `# Context: <topic>`
  - `## Completed Steps`
  - `## Current Status`
  - `## Key Information for the Next Trigger`

## File Index

- `AGENTS.md`: this directory convention
- `index.md`: active plan index
