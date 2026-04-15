# External Research: Positioning And Recommendations For `special`

## Status

Directional product research note based on:

- local repo review
- Matteo Barbero's workflow article
- relevant Hacker News discussion
- linked workflow tools and issue threads
- adjacent practitioner research on agent verification workflows

This is not canonical product contract material. If any recommendation here becomes
real planned behavior, it should move into `specs/` as `@planned` work rather than
living here indefinitely.

## Source Set

Primary sources:

- https://www.maiobarbero.dev/articles/ai-assisted-workflow/
- https://news.ycombinator.com/item?id=47775653
- https://github.com/mattpocock/skills
- https://github.com/tessellate-digital/notion-agent-hive
- https://github.com/tim-projects/tasks-ai
- https://github.com/mattpocock/sandcastle
- https://github.com/mattpocock/sandcastle/issues/191
- https://github.com/github/spec-kit
- https://github.com/github/spec-kit/issues/1174
- https://github.com/github/spec-kit/issues/1191
- https://docs.bmad-method.org/how-to/brownfield/
- https://michael.roth.rocks/research/trust-topology/
- https://michael.roth.rocks/research/gate-analysis/
- https://michael.roth.rocks/research/543-hours/

## Sourced Observations

### People are converging on a common workflow

Across the article, HN thread, and adjacent tools, the recurring loop is:

- clarify or rubberduck
- write a spec, PRD, or task packet
- break work into small bounded tasks
- execute each task in a fresh session
- do narrow review during implementation
- do one final whole-feature review at the end

This pattern appears repeatedly even when the implementation surface differs:
markdown files, Notion, Linear, Git-backed task systems, or repo-local skills.

### Users still want strong human control

Even highly agentic workflows keep human authority in the loop:

- humans decide ambiguous product questions
- humans approve risky transitions
- final review or release remains a human-controlled boundary

The market signal is not "remove humans." It is "make human control more efficient
and better informed."

### Final cross-cutting review matters

Per-task review is not enough. Multiple sources explicitly describe a final audit
pass that catches:

- cross-module inconsistencies
- repeated bad patterns
- missing integration work
- assumptions that only break across the full surface

### The biggest pain is workflow heaviness

Users are not rejecting specs. They are rejecting:

- overly long PRDs
- too many generated artifacts
- too much ceremony for small changes
- branch churn and duplicated planning output
- process that feels like productivity theater

The signal from `spec-kit` issues and brownfield-method docs is especially clear:
frameworks need a lighter path for small or iterative work.

### Fresh context and durable memory are both desired

People repeatedly report that long sessions drift. They prefer fresh task-sized
execution contexts.

At the same time, they do not want to lose project memory when a session dies.
That pushes the durable truth into explicit artifacts:

- specs
- task lists
- review findings
- project notes
- knowledge bases

### Deterministic gates beat vague autonomy

The adjacent tools that feel most grounded emphasize:

- state machines
- explicit review gates
- typed transitions
- bounded task scopes
- visible status and approval boundaries

The interest is in reliable workflow control, not in abstract multi-agent theater.

## POV

`special` is well aligned with the strongest research signal.

The external demand is not for a giant methodology product. It is for a lightweight
living contract layer that:

- preserves intent
- shows what changed
- shows what proves it
- reduces drift
- remains usable in ordinary repos

That is already close to `special`'s current public shape. The product risk is not
that `special` is too rigorous. The risk is that it could become too heavy for the
most common brownfield and small-change workflows.

The strongest external opportunity is to make `special` the best place to answer:

- what is the contract?
- what changed in the contract?
- what evidence supports it?
- where is proof weak or drifting?
- what architecture is actually implemented?

## Recommendations

### 1. Keep `special` as the contract and proof layer

Do not expand `special` into a full execution or orchestration platform.

The clearest positioning is:

- repo-native living contract
- proof and attestation surface
- architecture evidence layer

### 2. Build a lightweight small-change path

High-priority product direction:

- refine existing specs in place
- support very small changes without large artifact scaffolding
- keep brownfield updates first-class

Research signal:

- users like specs
- users dislike mandatory ceremony for tiny deltas

### 3. Prioritize spec diffs and drift detection

These are high-leverage next moves because they map directly to observed pain:

- stale assumptions
- hidden contract changes
- weak proof attachments
- proof that depends on drifting ambient context

This also matches the current repo roadmap direction.

### 4. Ship a whole-feature audit view

Users are inventing final audit passes because integration errors surface late.

`special` should help review:

- cross-spec consistency
- missing proofs across a feature slice
- duplicated or conflicting claims
- architecture or implementation coverage gaps

### 5. Lean into brownfield adoption

Users are often working in existing codebases, not greenfield demos.

`special` should make it easy to:

- inspect what is already there
- add specs gradually
- preserve existing conventions
- avoid requiring a top-to-bottom process migration

### 6. Keep messaging anti-bloat

Good external message:

`special` is the living contract and proof layer for AI-assisted coding.

Bad external message:

`special` is a complete new methodology you must adopt everywhere.

### 7. Treat architecture-as-implemented as the next major wedge

The current product and roadmap already point toward richer architecture evidence.
That direction is strategically good because it extends the same trust story:

- not just "what do we claim"
- also "what does the code actually implement"

## What Not To Build Next

- not a giant project-management surface
- not a mandatory multi-phase framework for every tiny change
- not workflow lock-in around one tracker or vendor
- not orchestration-first product messaging

## Recommended Near-Term Sequence

1. Make small-change and brownfield refinement clearly supported.
2. Add spec-diff and drift-warning surfaces.
3. Strengthen whole-feature audit and architecture-evidence views.
4. Keep distribution and repo-native portability simple and boring.

## Suggested External Positioning

`special` gives AI-assisted coding a living contract and proof surface.

Use it to:

- define what should be true
- inspect what is live versus planned
- attach proof and attestation honestly
- review contract drift before implementation drift turns into bugs

