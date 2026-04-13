# AGENTS.md

## Test Guidance

- Prefer semantic tests over incidental string matching.
- When output is primarily human prose, assert the smallest semantic surface that proves the contract: required sections, command shapes, ids, options, omissions, and ordering only when order matters.
- Do not lock tests to exact prose by default. Exact wording checks are appropriate only when the wording itself is contractual, such as legal or compliance text, machine-consumed formats, or intentionally fixed UX copy.
- If a stable output keeps needing exact string assertions, prefer introducing or reusing a typed or structured representation and test that structure directly, with a smaller number of rendering tests on the final text surface.
- Regression guards against internal or out-of-spec behavior do not automatically belong in `@spec` claims or `@verifies`; reserve product specs and verifies for real external contract behavior.
- Do not make substantive design choices in spec updates without consulting the user first.
