---
name: define-product-specs
description: Use this skill when scoping a feature, defining behavior, or rewriting vague requirements into product specs. Write present-tense claims, use `@group` only for structure, and keep each current spec narrow enough for one self-contained verify.
---

# Define Product Specs

Use this skill when you are turning requirements, roadmap items, or vague behavior into explicit product specs, whether the repo already uses `special` or you are introducing it now.

1. Write claims as clear, present-tense statements about externally meaningful behavior or other durable contract facts.
2. Use `@group` for structure-only nodes and `@spec` for real claims.
3. Keep `@planned` local to the exact claim that is not current yet.
4. Split claims until each current `@spec` can point to one honest, self-contained `@verifies` or `@attests` artifact.
5. Verify at the same boundary as the claim. Prefer public APIs, observable outcomes, and structured outputs over helpers and internal call sequences.
6. If a parent claim says something real, give it direct support. Child support does not justify a parent `@spec`.
7. Declare missing intermediate ids explicitly as `@group` nodes instead of relying on dotted ids alone.
8. Design verifies around parser reality: each `@verifies` block names exactly one spec id and attaches to the next supported item.
9. If the repo already has a primary place for contract declarations, keep new claims there instead of inventing a second contract layout.
10. Keep product specs on true behavior boundaries or the test-owned proving surface. Avoid placing behavior specs on internal implementation modules unless that file is itself the real user-visible or system-visible boundary.
11. Use `@module`, `@area`, and `@implements` for architecture intent instead of product specs. Reach for `special arch` when the question is structure, ownership, refactor shape, or whether code honestly implements a module rather than shipped behavior.
12. Favor claims and verifies that survive refactors. Avoid incidental implementation details, helper structure, and transient choices that are expected to churn.

Read [references/spec-writing.md](references/spec-writing.md) for the writing rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
