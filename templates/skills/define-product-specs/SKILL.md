---
name: define-product-specs
description: Use this skill when turning requirements, feature ideas, bug reports, roadmap notes, or vague behavior into clear product claims. Create or update Special specs as the durable contract: `@group` for structure, `@spec` for real claims, and planned/current state based on what actually ships.
---

# Define Product Specs

## When To Use

Use this when a developer task starts with unclear behavior:

- scoping a new feature
- turning a bug report into expected behavior
- deciding what a change should promise users
- moving roadmap or backlog prose into a real contract
- introducing Special specs to a repo that does not have them yet

Do not use this for implementation ownership or code organization. Use module/architecture guidance for that.

## How To Use

1. Write the behavior in plain English first.
2. Split it into stable claims that should survive refactors.
3. Use `@group ID` only for navigation.
4. Use `@spec ID` for each real claim.
5. Mark a claim `@planned` if it is not current yet.
6. Make each current claim narrow enough for one honest `@verifies` or `@attests` artifact.
7. If Special already exists, run `special specs --metrics` and place new claims near the existing contract surface.
8. If Special is not set up yet, add the smallest useful spec surface near tests, docs, or a dedicated contract file.

Example:

```md
@group EXPORT
CSV export behavior.

@spec EXPORT.CSV.HEADER
CSV exports include a header row with the selected column names.
```

Example verify:

```rust
// @verifies EXPORT.CSV.HEADER
#[test]
fn csv_export_includes_selected_headers() {
    let csv = export_csv(["id", "status"]);
    assert_eq!(csv.lines().next(), Some("id,status"));
}
```

## What To Do With Results

- If the claim is current, add or tighten a verify.
- If the claim is future work, keep it `@planned`.
- If a claim describes internals, move it to architecture text or pattern guidance.
- If a claim needs exact prose or exact output, say so in the spec.
- After editing, run `special specs --metrics` and `special lint` when available.

Read [references/spec-writing.md](references/spec-writing.md) for the writing rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
