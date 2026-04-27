# Pattern Workflow

Use this checklist when deciding whether repeated code should define or apply a pattern.

1. Identify the problem shape in plain language.
2. Check existing code examples.
3. Run `special patterns` if Special is configured.
4. Compare the candidate code to the pattern's rationale, constraints, and source applications.
5. Apply the pattern only when the code is intentionally following the approach.
6. If the code is a near-exact repeat, extract a helper or component.
7. If the code shares only a broad value, use docs/module text/lint instead.
8. Run `special patterns --metrics --target PATH` after changes when available.

Good pattern definition:

```md
### `@pattern ADAPTER.FACTS_TO_MODEL`
@strictness high
Adapt analyzer-specific fact collections into stable public model structs at the language-pack boundary. Use this when internal analysis facts need to cross into shared Special output without leaking parser or tool-specific types.
```

Good source application:

```rust
// @applies ADAPTER.FACTS_TO_MODEL
fn summarize_repo_traceability(facts: LanguageFacts) -> RepoTraceabilitySummary {
    // ...
}
```

Poor pattern candidates:

- "Prefer clear names." This is style guidance.
- "Keep modules small." This is an engineering principle.
- Three identical parsing functions. This is probably helper extraction.
- One unusual workaround. This belongs in local code comments or module prose until it recurs.

Useful commands:

```sh
special patterns
special patterns PATTERN.ID --verbose
special patterns --metrics
special patterns --metrics --target src/foo.rs
special patterns --metrics --target src/foo.rs --symbol parse_config
special arch MODULE.ID
```
