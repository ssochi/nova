# sop Directory Conventions

This directory stores high-frequency, reusable, and easy-to-miss processes. What belongs here are fixed practices, check items, and expiration conditions, not temporary conclusions for a single plan.

## Directory Responsibilities

- Capture startup recovery flows
- Capture validation and experience flows
- Provide stable checklists and common error handling for later agents

## When It Must Be Updated

- When a type of process repeatedly goes wrong and is already worth turning into a fixed SOP
- When an existing SOP becomes ambiguous, has missing items, or conflicts with the current framework rules
- When stable process documents are added, migrated, or deprecated

## File Format Convention

- SOP files use kebab-case, such as `self-validation.md`
- Recommended contents: trigger conditions, prerequisite checks, execution steps, completion criteria, and common mistakes
- If an SOP depends on fixed entry points or indexes, the source location must be written out

## Document Structure

- Every SOP must always include:
  - `# <Process Name>`
  - `## Trigger Conditions`
  - `## Prerequisite Checks`
  - `## Execution Steps`
  - `## Completion Criteria`
  - `## Common Mistakes`
- If the SOP needs to reference a fixed document or entry point, `## Related Entry Points` may be added

## File Index

- `AGENTS.md`: this directory convention
- `cli-blackbox-playtest.md`: CLI milestone closeout playtest SOP
- `startup-context-refresh.md`: startup context refresh SOP
- `self-validation.md`: self-validation SOP
- `self-experience.md`: self-experience SOP
