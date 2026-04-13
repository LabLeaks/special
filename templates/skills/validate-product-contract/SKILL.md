---
name: validate-product-contract
description: Use this skill when checking whether a product claim is honestly supported. Inspect the exact claim with `special spec SPEC.ID --verbose`, read the attached support, and decide whether the contract and verification truly match.
---

# Validate Product Contract

Use this skill when you need to judge whether a live product claim is honestly supported.

1. Start from one exact claim, not a whole subtree.
2. Run `special spec SPEC.ID --verbose` and read the claim text before reading the attached support.
3. Review each attached `@verifies` or `@attests` body and decide whether it proves the exact claim rather than a nearby one.
4. If the verify hides the important setup behind helpers, treat that as a quality problem and tighten it.
5. If the claim is not actually ready to ship, move it back to `@planned` instead of pretending the support is good enough.
6. Run `special lint` if the support looks malformed or the references seem inconsistent.
7. Treat missing intermediate ids as a tree-structure issue: they should usually exist as explicit `@group` nodes.
8. Remember the parser constraints while reviewing support: each `@verifies` block targets exactly one spec id and attaches to the next supported item.
9. Keep architecture validation separate. If the real question is whether code honestly implements a `@module`, switch to `special modules` and the architecture-validation workflow instead of forcing it through product specs.

Read [references/validation-checklist.md](references/validation-checklist.md) for the review rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
