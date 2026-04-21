# State Walkthrough

Use this workflow when you need the current state:

1. Run bare `special` if you need a quick orientation pass.
2. Run `special specs`.
3. Use `special specs --metrics` if you need grouped support or lifecycle counts before drilling into one claim.
4. Identify the exact claim or subtree you care about.
5. Use `special specs SPEC.ID` for a focused tree view.
6. Use `special specs SPEC.ID --verbose` when you need the actual support body.
7. Use `special specs --current` when you want only current claims; plain `special specs` includes planned claims too.

Example:

```sh
special specs EXPORT.CSV.HEADERS --verbose
```
