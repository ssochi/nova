# Self-Experience SOP (Incomplete)

## Trigger Conditions

Applicable when the current project has a real user path and this round's changes need to confirm whether "actual use holds up."

## Definition

Real experience validation means: using the project's real external entry point and operating step by step in the real way it is used, observing the results and forming conclusions. It is not the same as calling internal interfaces, nor is it the same as pure code inference.

## Execution Steps

1. First confirm whether a real user path exists
   - CLI interaction
   - Web page
   - API consumption flow
   - SDK example or minimal integration program
2. Clarify the experience goal for this round
   - Validate the complete flow
   - Validate a key scenario
   - Validate whether a boundary behavior is reachable through the real entry point
3. Advance through the real entry point
   - Avoid letting test fixtures directly replace the user path whenever possible
   - If automation tools are needed, the entry point should still remain consistent with the user
4. Record four categories of information
   - Basic context: date, version, entry point, environment
   - Actual path: what the user did
   - Positive experience: what worked and why
   - Issues and severity: what did not work and how much it matters
5. If the current project has no real user path
   - Do not generate an experience report
   - State the gap and fallback phrasing in the validation report or plan

## Acceptance Focus

1. The report must be based on a real entry point
2. Conclusions must be tied to specific paths and scenarios
3. If no real experience was performed in this round, clearly state why
