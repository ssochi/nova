# Self-Validation SOP (Incomplete)

## Trigger Conditions

Applicable to any iteration that needs to prove the change holds.

## Goal

Without assuming the project shape in advance, choose an appropriate primary validation surface and secondary validation surface for the current project, and leave reviewable evidence.

## Execution Steps

1. First explore the existing validation entry points in the project
   - Unit tests
   - Integration tests
   - Build or type check
   - Command-line validation
   - Browser automation
   - API probes or sample programs
2. Choose the primary validation surface
   - Prefer automated validation that is closest to the actual delivery surface
   - If there is no automated entry point, execute at least one lowest-cost reviewable command
3. Choose the secondary validation surface
   - Use it to cover risks omitted by the primary validation surface, such as configuration, documentation, structural completeness, or specific boundary conditions
4. Record the results
   - Execution method
   - Success or failure
   - Whether uncovered risks remain
5. If the current project lacks executable validation
   - Do not pretend it has been "validated"
   - Clearly write the gap, the impact, and the next plan to fill it

## Recommended Evidence

- Test commands pass
- Build or static checks pass
- Key file structure exists
- Key configuration can be read by the real entry point

## Common Mistakes

- Reusing old commands without exploring first
- Running only the easiest checks and skipping core risks
- Failing validation without recording the blockers
