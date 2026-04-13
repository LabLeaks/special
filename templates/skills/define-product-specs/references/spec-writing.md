# Spec Writing Rubric

Use this rubric when writing or tightening product specs in `special`.

It is anchored in a small set of standard software-engineering principles:

- Good specs are clear, implementation-independent, and testable.
- Good tests exercise public behavior and observable results.
- Good tests survive refactors and avoid overspecifying internals.

- State the contract, not the implementation.
- Keep the claim narrow enough that one verify can honestly support it.
- Avoid future tense. `@planned` already carries the future state.
- Avoid umbrella claims that only read like folders; use `@group` for those.
- Keep user-facing behavior at the command boundary and verify it there.
- Use exact wording that can stay stable after the claim ships.
- If a claim is not ready, keep it planned rather than overfitting a weak verify.
- Keep specs on stable, externally meaningful invariants rather than incidental details that are likely to churn.
- Write verifies against behavior, interfaces, file layout, or structured output instead of transient implementation details.

Good spec examples:

- `CSV exports include a header row with the selected column names.`
- `The CLI exits with status 2 when the config file contains an unknown key.`
- `Project-local skill installs are written under .agents/skills/.`

Bad spec examples:

- `The implementation uses helper function parse_skill_args first.`
- `The help text says "Print bundled skill help or install bundled skills."`
- `The test fixture is named skills_install_flow.`

Good verify examples:

- Assert the first CSV row equals the selected column names.
- Assert the CLI exits non-zero and reports the unknown config key.
- Assert `special skills install` writes the installed skill under `.agents/skills/<id>/SKILL.md`.

Bad verify examples:

- Assert an internal helper is called in a specific order.
- Assert an exact help-text sentence whose wording is expected to evolve.
- Assert a test-only implementation detail that users never observe.

Interaction-heavy tests are sometimes appropriate, but treat them as the exception. If the real contract is a side effect such as `sendEmail()` or `saveRecord()`, interaction checks can supplement the proof. Otherwise, prefer observable state and outputs.

Source anchors:

- W3C QA Specification Guidelines: clearer, more implementable, better testable specifications.
- Software Engineering at Google: test behaviors, not methods; favor tests that remain valuable through refactors.
- Testing Library guiding principle: the more tests resemble real use, the more confidence they provide.

Example:

```text
@group EXPORT.CSV
CSV export behavior.

@spec EXPORT.CSV.HEADERS
CSV exports include a header row with the selected column names.

@spec EXPORT.CSV.FILENAME
CSV downloads use the workspace name in the default filename.
```
