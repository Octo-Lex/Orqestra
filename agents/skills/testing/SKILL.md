# Skill: Testing

## Purpose
Write effective unit and integration tests.

## Steps
1. **Identify the contract** — what should the code do? Read the function signature and docstring.
2. **Enumerate edge cases** — happy path, empty input, boundary values, error conditions.
3. **Write tests** — one assertion per test case. Use descriptive names: `test_<unit>_<scenario>_<expected>`.
4. **Run and verify** — all tests green, no flakes.
5. **Measure coverage** — aim for >90% on the changed module.

## Constraints
- Tests must be deterministic — no random sleeps, no external service calls.
- Use mocks for filesystem, network, and time.
- Follow the project's existing test framework and conventions.

## Output Format
- Test file paths
- Coverage delta
- Confidence score based on: edge case coverage, assertion quality, determinism
