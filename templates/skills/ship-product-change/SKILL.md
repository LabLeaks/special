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
6. Verify at the same boundary as the claim. If the change affects command behavior, prefer command-boundary tests over helper-only tests.
7. Declare missing intermediate ids as `@group` nodes so the tree is explicit.
8. Keep `@verifies` narrow: one block targets exactly one spec id and attaches to the next supported item.
9. If the repo already keeps its primary contract in a dedicated spec file or directory, preserve that layout instead of scattering new claim declarations through unrelated implementation files.
10. Keep product specs on true behavior boundaries or the test-owned proving surface. If a file mostly exists to implement internals, use `@module`, `@area`, and `@implements` there instead of `@spec`.
11. Use `special modules` when the question is architectural: structure, ownership, missing implementation, whether code honestly implements a module, or refactor targets. Use `special specs` when the question is shipped behavior.
12. Keep the contract on stable, externally meaningful invariants. Avoid promoting incidental implementation details or transient choices into product specs.
13. When you add or revise verifies, prove observable behavior or other durable outcomes. Avoid tests that depend on internal sequencing unless side effects make that the real contract.

Read [references/change-workflow.md](references/change-workflow.md) for the detailed workflow and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
