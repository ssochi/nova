# design Directory Conventions

This directory stores feature and subsystem designs. It describes intent, scope boundaries, staged choices, and design trade-offs without replacing roadmap execution records or low-level technical references.

## Directory Responsibilities

- Capture feature-level designs before or alongside implementation
- Record scope boundaries, assumptions, and staged decisions
- Link design decisions to related milestone and plan work

## When It Must Be Updated

- When a new subsystem or feature direction needs a design baseline
- When a shipped implementation changes the design boundary or invalidates an assumption
- When a design becomes important enough to serve as a later handoff interface

## File Format Convention

- Design files use kebab-case, such as `bootstrap-vm-execution.md`
- Each design should state its goal, constraints, current scope, deferred scope, and follow-up hooks

## Document Structure

- Each design document should include:
  - `# <Design Topic>`
  - `## Goal`
  - `## Constraints`
  - `## Current Scope`
  - `## Deferred Scope`
  - `## Interfaces and Extension Hooks`

## File Index

- `AGENTS.md`: this directory convention
- `bootstrap-vm-execution.md`: design baseline for the bootstrap VM execution loop
- `semantic-functions-branches.md`: design baseline for semantic analysis, user-defined calls, and branch control flow
- `for-loop-control-flow.md`: design baseline for condition-only `for` loops and loop-aware path analysis
- `string-runtime-builtins.md`: design baseline for string values, builtin contract centralization, and the first `M3` runtime slice
