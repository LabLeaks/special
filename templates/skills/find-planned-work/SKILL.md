---
name: find-planned-work
description: Use this skill when looking for product-spec work that is planned but not live yet. Materialize the full tree with `special spec --all`, then focus on `[planned]` claims and their release targets when present.
---

# Find Planned Work

Use this skill when you need to see what the project intends to ship later but has not made live yet.

1. Start with `special spec --all` so planned claims appear.
2. Focus on `[planned]` nodes rather than live ones.
3. If the tree is large, scope with `special spec --all SPEC.ID`.
4. If a planned claim has a release target string, treat it as an exact tag to compare against the current version, not an ordered version range.
5. Use this skill for backlog discovery, release readiness, and “what is still planned?” questions.

Read [references/planned-workflow.md](references/planned-workflow.md) for the walkthrough and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
