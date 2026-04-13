# Architecture Validation Checklist

- Read the exact `@module` text before judging the code that claims to implement it.
- Compare attached `@implements` bodies to the module’s stated responsibility, not to nearby files or names.
- Treat `@area` as structure-only. A structural node should not need direct implementation.
- Prefer direct ownership. A live `@module` without direct `@implements` is architecture drift unless it is explicitly `@planned`.
- If multiple files implement one module, decide whether that split is intentional or a smell worth refactoring.
- Do not use architecture validation to prove product behavior. Switch to `special specs` when the question is what the product ships.
