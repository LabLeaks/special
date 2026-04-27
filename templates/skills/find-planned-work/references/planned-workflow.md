# Planned Work Workflow

Use this workflow when you need planned work:

1. Run `special specs --planned` if Special is installed and configured.
2. Scope with `special specs --planned SPEC.ID` when needed.
3. Treat release target strings as exact labels only.
4. If future work only exists in backlog/prose, move durable product behavior into planned specs.
5. Keep planned work separate from current supported behavior.

Example:

```text
@spec EXPORT.CSV.FILTER_SUMMARY
@planned
CSV exports include a summary row showing the active filters.
```
