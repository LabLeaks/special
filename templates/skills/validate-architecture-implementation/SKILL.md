---
name: validate-architecture-implementation
description: Use this skill when checking whether a concrete architecture module is honestly implemented by the code that claims to implement it. Inspect one module with `special arch MODULE.ID --verbose`, use `--metrics` when you need unknown-unknowns evidence, and decide whether the module intent and implementation really match.
---

# Validate Architecture Implementation

Use this skill when you need to judge whether a concrete architecture module is honestly implemented by the code that claims to implement it.

1. Start from one exact `@module` id, not a whole subtree.
2. Run `special arch MODULE.ID --verbose` and read the module text before reading the attached `@implements` bodies.
3. Use the command family consistently:
   - `special arch MODULE.ID` for the normal ownership tree
   - `special arch --metrics` for the architecture-wide grouped summary first
   - `special arch MODULE.ID --metrics` for the deeper evidence on one boundary without dumping the whole repo
   - `special arch MODULE.ID --metrics --verbose` for the full detailed drilldown
4. Use `special arch MODULE.ID --metrics --verbose` when you need evidence about unknown unknowns inside the claimed boundary:
   - hidden or uncovered implementation
   - broad `@fileimplements` ownership that may hide odd items
   - unusually complex, outbound-heavy, isolated, or panic-heavy items
5. Treat `special arch` as the centralized view, not the central source of truth. Prefer architecture declarations near the owning code and use the CLI to assemble the whole picture.
6. When architecture intent itself is wrong or incomplete, update the tracked source-local `@module` text instead of editing incidental runtime copies or burying the design elsewhere.
7. Review each attached implementation body and decide whether it implements the exact module intent rather than a nearby module or a purely organizational `@area`.
8. Treat `@area` as structure only. If a node is conceptual grouping, it should not carry direct implementation expectations.
9. Read metrics at two levels:
   - plain-language explanation first
   - exact structural definition second
10. Treat metrics as evidence, not verdicts. They can surface suspicious items or coarse boundaries, but they do not prove a dependency is wrong or that an annotation must be moved.
11. Use `special arch --unimplemented` to find current modules that still have no direct `@implements`.
12. Run `special lint` if ownership looks malformed or the attachments seem inconsistent.
13. Treat duplicate `@implements`, unknown module ids, and area-targeted `@implements` as architecture integrity problems.
14. Keep product-contract questions separate. If the real question is shipped behavior or whether a test proves a claim, switch to `special specs` and the product-spec workflow instead.
15. Remember that `@implements` is traceability, not behavior proof. A module may have multiple implementing sites, but broad spread is a smell worth calling out.
16. If you delegate or fan this review out to subagents, explicitly instruct each agent to run `special arch MODULE.ID --verbose` and `special arch MODULE.ID --metrics --verbose` for their exact boundary. Use `special arch --metrics` only for the broad architecture-wide summary, and use `special health` only when the question is genuinely cross-cutting rather than module-bound. Do not rely on freehand code reading alone for architecture correspondence audits.

Read [references/validation-checklist.md](references/validation-checklist.md) for the review rubric and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
