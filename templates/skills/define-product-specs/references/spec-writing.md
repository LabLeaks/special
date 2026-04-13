# Spec Writing Rubric

Use this rubric when writing or tightening product specs in `special`:

- State the contract, not the implementation.
- Keep the claim narrow enough that one verify can honestly support it.
- Avoid future tense. `@planned` already carries the future state.
- Avoid umbrella claims that only read like folders; use `@group` for those.
- Keep user-facing behavior at the command boundary and verify it there.
- Use exact wording that can stay stable after the claim ships.
- If a claim is not ready, keep it planned rather than overfitting a weak verify.
