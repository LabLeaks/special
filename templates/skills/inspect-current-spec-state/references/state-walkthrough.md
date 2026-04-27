# State Walkthrough

Use this workflow when you need the current product-contract state.

1. Run `special specs --current` if Special is installed and configured.
2. Use `special specs --current --metrics` for support counts and gaps.
3. Use `special specs SPEC.ID --verbose` to inspect one claim and its proof.
4. If there is no Special surface, inspect tests/docs/public behavior and report that current behavior is untracked.
5. Add specs later through `define-product-specs` or `ship-product-change`; do not invent unsupported claims in the report.

Example:

```sh
special specs EXPORT.CSV.HEADERS --verbose
```
