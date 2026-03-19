# reports Directory Conventions

This directory stores result documents. It records validation evidence and real experience conclusions, and is not responsible for defining the framework truth itself.

## Directory Responsibilities

- `verification/`: test results, command validation, structure checks, risk review, and failure reasons
- `experience/`: validation of real user-path experience, subjective feel, issue grading, and conclusions

## When It Must Be Updated

- When adding, migrating, or deleting report subdirectories
- When the naming convention for validation reports or experience reports changes
- When the reporting standard of the current project changes significantly

## File Format Convention

- Report files uniformly use `YYYY-MM-DD-<topic>.md`
- Reports should include: date, context, execution method, results, and risks or conclusions
- If the current project lacks a real experience entry point, do not fabricate an experience report; explain the fallback approach in validation or in the plan

## Document Structure

- Formal reports under `verification/` follow the validation report structure; see the subdirectory `AGENTS.md` for details
- Formal reports under `experience/` follow the experience report structure; see the subdirectory `AGENTS.md` for details
- If an index document is added at the `reports/` root in the future, it must include at least:
  - Report types
  - Time range
  - Related plan or milestone
  - Entry point for key conclusions

## File Index

- `AGENTS.md`: this directory convention
- `verification/AGENTS.md`: validation report subdirectory convention
- `experience/AGENTS.md`: experience report subdirectory convention
- There are currently no formal report files; after adding reports, this index must be completed
