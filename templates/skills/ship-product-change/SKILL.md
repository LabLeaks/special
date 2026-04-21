---
name: ship-product-change
description: Use this skill when adding a feature, fixing a bug, or changing behavior that should update the product contract. Define or revise the relevant product specs first, then keep each current claim matched to one honest verify.
---

# Ship Product Change

Use this skill when a product change needs to stay aligned with the contract you ship, whether the repo already uses `special` or you are introducing it now.

1. Start from the user-visible behavior that changed, not the implementation.
2. If the repo already uses `special`, start with bare `special` if you need quick orientation, then find the relevant claim with `special specs`, `special specs --current`, or `special specs --planned` as appropriate. If it does not, define the first claim before or alongside the code change.
3. If the behavior is not ready to ship, keep the exact claim `@planned` instead of over-claiming.
4. When the claim is current, make sure it has one honest, self-contained `@verifies` or `@attests` artifact.
5. Run `special specs SPEC.ID --verbose` before trusting support. Read the attached body and decide whether it actually proves the claim.
6. Verify at the same boundary as the claim. If the change affects command behavior, prefer command-boundary tests over helper-only tests.
7. Declare missing intermediate ids as `@group` nodes so the tree is explicit. `@group` is structure only; parent `@spec` claims still need their own direct support or `@planned`.
8. Keep `@verifies` narrow: one block targets exactly one spec id and attaches to the next supported item.
9. If the repo already keeps its primary contract in a dedicated spec file or directory, preserve that layout instead of scattering new claim declarations through unrelated implementation files.
10. Keep product specs on true behavior boundaries or the test-owned proving surface. If a file mostly exists to implement internals, use `@module`, `@area`, and `@implements` there instead of `@spec`.
11. If the change also alters architecture intent, locate the relevant source-local `@module` / `@area` declarations with `special arch ... --verbose` or direct file reads and update those texts in tracked source.
12. After the code change, use `special health PATH...` for cross-cutting code-health impact and `special health PATH... --metrics` when you need the deeper grouped counts for the touched files.
13. Use `special arch` when the question is architectural: structure, ownership, missing implementation, whether code honestly implements a module, or refactor targets. Use `special specs` when the question is shipped behavior.
14. Keep the contract on stable, externally meaningful invariants. Avoid promoting incidental implementation details or transient choices into product specs.
15. When you add or revise verifies, prove observable behavior or other durable outcomes. Avoid tests that depend on internal sequencing unless side effects make that the real contract.
16. If you delegate spec validation to subagents, explicitly require `special specs SPEC.ID --verbose`, `special specs --current`, or `special specs --planned` in the prompt and treat direct file reads as confirmation, not the primary source of truth.

Read [references/change-workflow.md](references/change-workflow.md) for the detailed workflow and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
