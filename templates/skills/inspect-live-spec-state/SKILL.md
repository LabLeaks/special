---
name: inspect-live-spec-state
description: Use this skill when you need the current live validated product-spec state. Materialize the live tree with `special spec`, then scope into exact claims with `special spec SPEC.ID --verbose`.
---

# Inspect Live Spec State

Use this skill when you need to understand what the project currently claims is live and supported.

1. Start with `special spec` to view the live tree only.
2. If you need a narrower view, scope to the exact node with `special spec SPEC.ID`.
3. If you need to understand why a claim is live, use `special spec SPEC.ID --verbose`.
4. Treat groups as navigation only; the real contract lives on direct `@spec` nodes.
5. Use this skill before making statements about what the project currently ships.
6. Learn the repo's main contract layout first so you inspect the authoritative declarations instead of a convenient but secondary file.
7. If the question turns out to be about architecture structure, implementation ownership, or whether code honestly implements a module rather than shipped behavior, switch surfaces instead of forcing it through product specs: use `special modules`, `@module`, `@area`, and `@implements`.

Read [references/state-walkthrough.md](references/state-walkthrough.md) for the walkthrough and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
