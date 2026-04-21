---
name: inspect-current-spec-state
description: Use this skill when you need the current validated product-spec state. Start with bare `special` when you need quick orientation, then materialize the current tree with `special specs --current` and scope into exact claims with `special specs SPEC.ID --verbose`.
---

# Inspect Current Spec State

Use this skill when you need to understand what the project currently claims is current and supported.

1. Start with bare `special` if you need a quick overview of where to look next.
2. Run `special specs --current` to view the current tree only.
3. Use `special specs --current --metrics` when you need grouped support and lifecycle counts across the current surface.
4. If you need a narrower view, scope to the exact node with `special specs --current SPEC.ID`.
5. If you need to understand why a claim is current, use `special specs SPEC.ID --verbose`.
5. Treat groups as navigation only; the real contract lives on direct `@spec` nodes.
6. Use this skill before making statements about what the project currently ships.
7. Learn the repo's main contract layout first so you inspect the authoritative declarations instead of a convenient but secondary file.
9. If the question turns out to be about architecture structure, implementation ownership, or whether code honestly implements a module rather than shipped behavior, switch surfaces instead of forcing it through product specs: use `special arch`, `@module`, `@area`, and `@implements`.
10. If the question is broader code health rather than contract state, switch to `special health` instead of trying to infer it from the spec tree.

Read [references/state-walkthrough.md](references/state-walkthrough.md) for the walkthrough and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
