# Skill: Debugging

## Purpose
Systematically diagnose and fix bugs in source code.

## Steps
1. **Read the failing test or stack trace** — identify the exact line and error type.
2. **Reproduce** — confirm the failure locally with a minimal test case.
3. **Trace the root cause** — follow the data flow backwards from the failure point.
4. **Write the minimal fix** — change only what is necessary. No refactoring.
5. **Add a regression test** — the test must fail before the fix and pass after.
6. **Verify** — run the full test suite to ensure no side effects.

## Constraints
- Never change more than one concern per commit.
- Never modify documentation files.
- If the fix requires a design change, escalate (propose, not auto-commit).

## Output Format
- Changed files list with line counts
- Regression test file path
- Confidence score (0.0–1.0) based on: reproducibility, test coverage, blast radius
