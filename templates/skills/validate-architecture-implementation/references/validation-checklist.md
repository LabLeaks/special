# Architecture Validation Checklist

- Read the exact `@module` text before judging the code that claims to implement it.
- Compare attached `@implements` bodies to the module’s stated responsibility, not to nearby files or names.
- Use `special arch MODULE.ID --metrics --verbose` when you need architecture evidence beyond direct ownership tracing.
- If the module lists pattern applications, read `special patterns PATTERN.ID --verbose` before judging whether the implementation approach is intentional.
- Treat uncovered files, weak coverage, isolated items, outbound-heavy items, and complexity hotspots as inspection cues, not automatic violations.
- Treat `@area` as structure-only. A structural node should not need direct implementation.
- Prefer direct ownership. A current `@module` without direct `@implements` is architecture drift unless it is explicitly `@planned`.
- Treat `@fileimplements` as honest but coarse ownership. Use item signals to decide whether the file likely hides functions that deserve narrower attachment.
- If multiple files implement one module, decide whether that split is intentional or a smell worth refactoring.
- Keep `@applies` and `@implements` distinct: pattern application explains approach; module implementation explains ownership.
- Do not use architecture validation to prove product behavior. Switch to `special specs` when the question is what the product ships.
