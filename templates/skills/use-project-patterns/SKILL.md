---
name: use-project-patterns
description: Use this skill when adding, reviewing, or refactoring code that resembles a recurring project approach. Capture the "why is it done this way?" answer with `@pattern`, attach `@applies` only to real source applications, and use pattern metrics as advisory evidence.
---

# Use Project Patterns

## When To Use

Use this when a developer task involves repeated implementation shape:

- adding code similar to existing code
- reviewing whether a one-off approach should match the project style
- deciding whether repetition is a pattern or should become a helper/component
- documenting why several modules solve a problem the same way
- interpreting `special patterns --metrics`

The repo does not need Special patterns yet. This skill can introduce the first useful pattern.

## How To Use

1. Start with the problem shape: what situation keeps recurring?
2. Look for existing examples in nearby code.
3. If Special is present, run `special patterns` and then `special patterns PATTERN.ID --verbose`.
4. If there is no pattern yet, define one only when it will guide future code.
5. Write the pattern as rationale plus fit criteria, not just a name.
6. Attach `@applies PATTERN.ID` to source items that actually demonstrate the approach.
7. Use `@fileapplies PATTERN.ID` only when the whole file is the application.
8. Use `@strictness high|medium|low` when similarity expectations matter.

Example definition:

```md
### `@pattern ADAPTER.FACTS_TO_MODEL`
@strictness high
Adapt analyzer-specific facts into stable public model structs at the language-pack boundary so parser and tool details do not leak into shared output.
```

Example application:

```rust
// @applies ADAPTER.FACTS_TO_MODEL
fn summarize_repo_traceability(facts: LanguageFacts) -> RepoTraceabilitySummary {
    // ...
}
```

## What To Do With Results

- If code intentionally follows a pattern, add or keep `@applies`.
- If code is almost identical everywhere, extract a helper/component instead of naming a pattern.
- If the shared idea is only a principle or style rule, put it in docs/module text/lint instead.
- If pattern applications are too different, split or rewrite the pattern.
- If `special patterns --metrics` reports possible missing applications, inspect them manually; do not treat them as lint failures.
- After edits, run `special patterns --metrics --target PATH` when available.

Read [references/pattern-workflow.md](references/pattern-workflow.md) for examples and [references/trigger-evals.md](references/trigger-evals.md) for trigger examples.
