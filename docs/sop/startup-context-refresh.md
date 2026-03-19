# Startup Context Refresh

## Trigger Conditions

Use this SOP at the start of every new autonomous run.

## Prerequisite Checks

- Confirm the repository root.
- Confirm whether archived plans exist.
- Confirm whether a milestone is already `in_progress`.

## Execution Steps

1. Read the root `AGENTS.md`.
2. Read the newest archived plan directory under `docs/roadmap/archive/`.
   - Prioritize `context.md`.
   - If no archive exists yet, record that the project is at a cold start.
3. Read `docs/roadmap/milestones/index.md` and identify the active milestone.
4. Read `docs/roadmap/plans/index.md`.
   - If an active plan exists, read that plan's `plan.md`, `todo.md`, and `context.md`.
   - If no active plan exists, record that the next round must either continue the milestone with a new plan or close the milestone.
5. Read `todo.md`.
6. Read the research, design, tech, SOP, and test documents that apply to the likely task.
7. Decide whether to continue the current plan, archive it, or start a new one.

## Completion Criteria

- The current milestone and current execution surface are clear.
- The last round's next-step context is captured.
- Any missing roadmap artifact is treated as the highest-priority documentation gap.

## Common Mistakes

- Skipping archived `context.md` and losing momentum
- Treating an empty active plan index as permission to improvise without a new plan
- Starting implementation before confirming whether the milestone should continue or switch

## Related Entry Points

- `BOOT.md`
- `docs/research/`
- `docs/roadmap/milestones/index.md`
- `docs/roadmap/plans/index.md`
