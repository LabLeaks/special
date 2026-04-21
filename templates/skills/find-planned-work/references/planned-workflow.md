# Planned Work Workflow

Use this workflow when you need planned work:

1. Run `special specs --planned`.
2. Look for `[planned]` nodes.
3. Scope to a subtree with `special specs --planned SPEC.ID` when needed.
4. Treat release target strings as exact matches only.
5. Distinguish clearly between current state and planned work.

Example:

```text
@spec EXPORT.CSV.FILTER_SUMMARY
@planned
CSV exports include a summary row showing the active filters.
```
