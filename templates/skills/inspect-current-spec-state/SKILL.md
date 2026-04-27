---
name: inspect-current-spec-state
description: Use this skill when you need to know what behavior the project currently claims and supports. Use `special specs --current` when available, or identify that the project lacks a tracked current contract.
---

# Inspect Current Spec State

## When To Use

Use this before saying what the product currently does:

- preparing release notes
- answering “is this supported?”
- checking current behavior before changing it
- auditing whether claims are verified
- introducing Special because current behavior is only implicit

## How To Use

1. If Special is present, run `special specs --current`.
2. Use `special specs --current --metrics` for counts and gaps.
3. Scope with `special specs --current SPEC.ID` for one area.
4. Use `special specs SPEC.ID --verbose` to read the proof for one claim.
5. If Special is not present, inspect tests/docs/public behavior and report that the current contract is untracked.

Useful commands:

```sh
special
special specs --current
special specs --current --metrics
special specs SPEC.ID --verbose
```

## What To Do With Results

- If a current claim is verified, you can cite it.
- If a current claim lacks support, add proof or mark it planned.
- If the behavior is real but untracked, add a spec.
- If the question is about ownership, use architecture skills.
- If the question is about code health, use `special health`.

Read [references/state-walkthrough.md](references/state-walkthrough.md) for the walkthrough and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
