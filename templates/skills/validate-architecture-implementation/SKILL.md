---
name: validate-architecture-implementation
description: Use this skill when checking whether a concrete architecture module is honestly implemented by the code that claims to implement it. Inspect one module with `special modules MODULE.ID --verbose`, read the attached `@implements` bodies, and decide whether the module intent and implementation really match.
---

# Validate Architecture Implementation

Use this skill when you need to judge whether a concrete architecture module is honestly implemented by the code that claims to implement it.

1. Start from one exact `@module` id, not a whole subtree.
2. Run `special modules MODULE.ID --verbose` and read the module text before reading the attached `@implements` bodies.
3. Treat `special modules` as the centralized view, not the central source of truth. Prefer architecture declarations near the owning code and use the CLI to assemble the whole picture.
4. Review each attached implementation body and decide whether it implements the exact module intent rather than a nearby module or a purely organizational `@area`.
5. Treat `@area` as structure only. If a node is conceptual grouping, it should not carry direct implementation expectations.
6. Use `special modules --unsupported` to find live modules that still have no direct `@implements`.
7. Run `special lint` if ownership looks malformed or the attachments seem inconsistent.
8. Treat duplicate `@implements`, unknown module ids, and area-targeted `@implements` as architecture integrity problems.
9. Keep product-contract questions separate. If the real question is shipped behavior or whether a test proves a claim, switch to `special specs` and the product-spec workflow instead.
10. Remember that `@implements` is traceability, not behavior proof. A module may have multiple implementing sites, but broad spread is a smell worth calling out.

Read [references/validation-checklist.md](references/validation-checklist.md) for the review rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
