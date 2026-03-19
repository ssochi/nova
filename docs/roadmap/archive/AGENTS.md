# roadmap/archive Directory Conventions

This directory stores complete archives of closed plans. It serves historical continuation, not the current execution surface.

## Directory Responsibilities

- Save directory snapshots of completed or terminated plans
- Preserve historical `plan.md`, `todo.md`, and `context.md`
- Provide the minimum necessary context for later retrospective

## When It Must Be Updated

- When an active plan is completed and moved into the archive
- When archive directory naming rules or retained-file rules change
- When key information for historical plans needs to be supplemented

## File Format Convention

- Archive directories reuse active plan directory naming: `YYYY-MM-DD-HH-MM-SS-<plan-id>`
- Each archived plan directory must keep at least `plan.md`, `todo.md`, and `context.md`
- Do not compress "completed" into a one-sentence conclusion; resumable context must be preserved

## Document Structure

- Each archived plan directory keeps the same trio as an active plan:
  - `plan.md`: retain the original plan structure; do not rewrite it into a summary version
  - `todo.md`: retain the task list with final status
  - `context.md`: retain the context at closeout and reminders for the next round
- If an archive index document is added in the future, it must include at least:
  - Archived plan list
  - Close reason
  - Related milestone

## File Index

- `AGENTS.md`: this directory convention
- `2026-03-19-23-19-47-bootstrap-vm-foundation/`: archived bootstrap execution plan for milestone M1
- `2026-03-19-23-37-05-semantic-functions-branches/`: archived M2 plan for semantic analysis, function calls, and branches
- `2026-03-19-23-57-06-for-loops-path-analysis/`: archived M2 closing plan for loop control flow and path analysis
- `2026-03-20-03-42-30-map-runtime-groundwork/`: archived M3 plan for staged `map[K]V` support with `make`, indexing, and nil-map behavior
- `2026-03-20-04-07-11-map-literals-delete/`: archived M3 plan for staged map literals, empty-map construction, and builtin `delete`
- `2026-03-20-04-28-25-explicit-nil-comparisons/`: archived M3 plan for explicit `nil` expressions and `slice/map` nil comparisons
