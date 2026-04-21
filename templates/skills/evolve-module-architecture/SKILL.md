---
name: evolve-module-architecture
description: Use this skill when changing the architecture design of a `special` repo itself. Locate the real `@module` tree first, update tracked source-local module text as the authoritative architecture, and keep repo, modules, and specs responsibilities explicitly separated.
---

# Evolve Module Architecture

Use this skill when the task is to change architecture intent, command-surface responsibility, or ownership boundaries in a repo that already uses `special`.

1. Start by locating the real module tree. In a `special` repo, architecture lives in tracked source-local `@module` / `@area` declarations rooted at the repo's `@module` tree, not in installed runtime copies or loose design notes.
2. Read the relevant subtree with `special arch MODULE.ID --verbose` when possible, then confirm by opening the source files that declare those `@module` ids.
3. Treat the three main surfaces separately:
   - `special specs` is the product-contract view.
   - `special arch` is the ownership and architecture view.
   - `special health` is the code-first cross-cutting analysis view.
4. If the design changes shipped behavior, add or revise planned product specs in `specs/` first.
5. If the design changes architecture intent or projection boundaries, update the authoritative tracked source-local `@module` text before or alongside implementation.
6. Keep `.agents/skills/` as runtime/install output only. Do not treat it as authoring storage.
7. When the task involves bundled skills in this repo, prefer `special skills install ...` over ad hoc copying. Manual copying is only for cases the CLI does not cover.
8. When one shared analysis core feeds multiple command surfaces, define the core once and describe each CLI surface as a projection over that shared evidence rather than duplicating the architecture in multiple places.
9. For `special` itself, prefer this split unless the code proves otherwise:
   - `special health` stays code-first.
   - `special arch` stays ownership-first.
   - `special specs` stays contract-first.
10. If the architecture change is broad, update the smallest set of tracked root texts that honestly explain the new split before touching deeper leaves.
11. Use `special arch` for the normal tree, `special arch --metrics` for the architecture-wide grouped summary, `special arch MODULE.ID --metrics` for deeper evidence on one boundary, and `special arch MODULE.ID --metrics --verbose` when you need the full detailed drilldown on one boundary.
12. Run `special arch`, `special arch MODULE.ID --verbose`, `special health PATH...` when the change crosses ownership seams, and `special lint` after the edit so the projections still materialize cleanly.

Use `validate-architecture-implementation` when the task is to judge whether existing code matches a module. Use `ship-product-change` when the main task is a behavior change rather than an architecture change.
