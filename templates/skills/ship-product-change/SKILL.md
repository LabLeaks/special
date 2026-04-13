---
name: ship-product-change
description: Use this skill when adding a feature, fixing a bug, or changing behavior that should update the product contract. Define or revise the relevant product specs first, then keep each live claim matched to one honest verify.
---

# Ship Product Change

Use this skill when a product change needs to stay aligned with the contract you ship, whether the repo already uses `special` or you are introducing it now.

1. Start from the user-visible behavior that changed, not the implementation.
2. If the repo already uses `special`, find the relevant claim with `special spec` or `special spec --all`. If it does not, define the first claim before or alongside the code change.
3. If the behavior is not ready to ship, keep the exact claim `@planned` instead of over-claiming.
4. When the claim is live, make sure it has one honest, self-contained `@verifies` or `@attests` artifact.
5. Run `special spec SPEC.ID --verbose` before trusting support. Read the attached body and decide whether it actually proves the claim.
6. If the change affects command behavior, prefer command-boundary verifies over helper-only tests.

Read [references/change-workflow.md](references/change-workflow.md) for the detailed workflow and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
