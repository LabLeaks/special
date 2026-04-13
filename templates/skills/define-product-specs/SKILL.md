---
name: define-product-specs
description: Use this skill when scoping a feature, defining behavior, or rewriting vague requirements into product specs. Write present-tense claims, use `@group` only for structure, and keep each live spec narrow enough for one self-contained verify.
---

# Define Product Specs

Use this skill when you are turning requirements, roadmap items, or vague behavior into explicit product specs, whether the repo already uses `special` or you are introducing it now.

1. Write claim text in present tense. Shipping a planned claim should not require rewriting the sentence.
2. Use `@group` for structure-only nodes and `@spec` for real claims.
3. Keep `@planned` local to the exact claim that is not live yet.
4. Split claims until each live `@spec` can point to one honest, self-contained `@verifies` or `@attests` artifact.
5. Prefer product-boundary verifies for product-boundary behavior. Do not let helper tests carry a command-level claim.
6. If a parent claim says something real, give it direct support. Child support does not justify a parent `@spec`.

Read [references/spec-writing.md](references/spec-writing.md) for the writing rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
