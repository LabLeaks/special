---
name: validate-product-contract
description: Use this skill when checking whether a product claim is honestly supported. Inspect the exact claim with `special specs SPEC.ID --verbose`, read the attached support, and decide whether the contract and verification truly match.
---

# Validate Product Contract

Use this skill when you need to judge whether a current product claim is honestly supported.

1. Start from one exact claim, not a whole subtree.
2. Use `special specs --current` for the normal current tree, `special specs --current --metrics` for grouped support/lifecycle analysis, and `special specs SPEC.ID --verbose` when you need the full attached support for one claim.
3. Run `special specs SPEC.ID --verbose` and read the claim text before reading the attached support.
4. Review each attached `@verifies` or `@attests` body and decide whether it proves the exact claim rather than a nearby one.
5. If the verify hides the important setup behind helpers, treat that as a quality problem and tighten it.
6. If the claim is not actually ready to ship, move it back to `@planned` instead of pretending the support is good enough.
7. Run `special lint` if the support looks malformed or the references seem inconsistent.
8. Treat missing intermediate ids as a tree-structure issue: they should usually exist as explicit `@group` nodes.
9. Remember the parser constraints while reviewing support: each `@verifies` block targets exactly one spec id and attaches to the next supported item.
10. Keep architecture validation separate. If the real question is whether code honestly implements a `@module`, switch to `special arch` and the architecture-validation workflow instead of forcing it through product specs.

Read [references/validation-checklist.md](references/validation-checklist.md) for the review rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
