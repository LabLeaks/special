---
name: ship-product-change
description: Use this skill when adding a feature, fixing a bug, or changing behavior. Keep the product contract current: define or revise the relevant claim, implement the change, attach one honest proof, then run the focused Special checks.
---

# Ship Product Change

## When To Use

Use this for ordinary product work:

- adding a feature
- fixing a behavior bug
- changing CLI/API/output behavior
- preparing a release-impacting change
- introducing Special while making a real change

Do not start from implementation details. Start from what users or downstream systems can observe.

## How To Use

1. State the behavior change in one sentence.
2. Find the existing claim with `special specs` when available.
3. If no claim exists, add a narrow `@spec`.
4. If the behavior is not ready, mark it `@planned`.
5. Implement the change.
6. Attach one honest `@verifies` or `@attests` artifact to the current claim.
7. If ownership changed, update `@module` / `@implements`.
8. If the implementation follows an adopted approach, check or add `@pattern` / `@applies`.

Useful commands:

```sh
special specs --metrics
special specs SPEC.ID --verbose
special arch MODULE.ID --verbose
special patterns --metrics --target src/foo.rs
special health --target src/foo.rs --metrics
special lint
```

## What To Do With Results

- If specs are unverified, add or repair the proof.
- If lint reports unknown ids or orphan support, fix the annotation.
- If health flags touched code, inspect whether the issue is real before widening scope.
- If pattern metrics show a possible missing application, apply the pattern or explain why the code is intentionally different.
- If the change only affects internals, keep product specs focused on observable behavior and use architecture/pattern annotations for implementation intent.

Read [references/change-workflow.md](references/change-workflow.md) for the detailed workflow and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
