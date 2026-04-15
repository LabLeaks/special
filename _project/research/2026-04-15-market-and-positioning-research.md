# Market And Positioning Research For `special`

## Status

Working research note for product vision and positioning.

This is not canonical product contract material. Shipped behavior still lives in
`special` specs and modules.

## Scope

This pass was grounded in:

- the successful local `special specs --all` snapshot taken during this session
- the successful local `special modules --all --metrics` snapshot taken during this session
- the existing seed note in `_project/inbox/2026-04-15-external-research-positioning-and-recommendations.md`
- the linked seed material and adjacent public discussions

One caveat: the repo worktree changed during the session and later fresh builds
started failing because `src/modules/analyze.rs` referenced a missing `rust`
module. The capability baseline below is therefore anchored to the earlier
successful dumps, which are the most reliable snapshot collected in this pass.

## Honest Capability Baseline

### Product shape today

The current shipped product is a repo-native CLI that gives a codebase an
inspectable contract and architecture surface.

Successful local snapshots in this session showed:

- 208 total spec claims
- 202 live spec claims
- 6 planned spec claims
- 0 unsupported live spec claims
- 63 architecture nodes total
- 60 concrete modules
- 3 structural areas

### Current wedges

The real product surface today clusters into five wedges.

#### 1. Contract materialization

`special specs` materializes the live spec tree from source-local and markdown
annotations, can include planned items, scopes to subtrees, and renders as text,
JSON, or HTML.

#### 2. Proof and attestation inspection

Claims can carry attached `@verifies` and `@attests`, and verbose views surface
the attached proof bodies for human inspection.

#### 3. Architecture ownership and evidence

`special modules` materializes architecture declarations plus implementation
ownership, and `--metrics` surfaces ownership coverage, module summaries, and
dependency evidence.

#### 4. Structural integrity checks

`special lint` checks malformed annotations, invalid references, duplicate ids,
orphaned support, planned-scope errors, and related structural problems.

#### 5. Agent-runtime skill installation

`special skills` prints bundled skills and installs task-shaped repo/global skills
for product-spec work, contract validation, architecture validation, live-state
inspection, and planned-work discovery.

### Important truths for positioning

`special` does support:

- repo-native contract declarations
- live vs planned distinction
- proof and attestation attachment
- architecture ownership declarations
- architecture analysis metrics
- gradual adoption across ordinary files and markdown
- repo-local and global skill installation for agents

`special` does not currently support:

- workflow checkpoints or approval gates
- task orchestration
- a hosted control plane
- issue tracker or project-management workflows
- automated implementation planning across tasks
- built-in contract diffs
- signed attestations
- claim-hash binding
- attestation TTL enforcement
- enforceable support policy gates

The last five are current roadmap direction, not shipped capability.

## External Market Signals

### 0. Repo-local persistent context is becoming table stakes

There is now a clear adjacent market around persistent repo instructions for
agents:

- `AGENTS.md`
- GitHub Copilot custom instructions
- Claude Code memory files
- Cursor rules

That matters because it proves developers already accept repo-local agent
context as a real need. But those tools mostly encode instructions and
heuristics. They do not materialize product truth, support status, or module
ownership.

Relevant sources:

- https://agents.md/
- https://docs.github.com/en/copilot/how-tos/custom-instructions/adding-repository-custom-instructions-for-github-copilot?tool=visualstudio&trk=public_post_comment-text
- https://code.claude.com/docs/zh-CN/memory
- https://docs.cursor.com/fr/context/rules

### 1. Teams are converging on task packets plus fresh sessions

Across the Matteo Barbero workflow post, HN discussion, Matt Pocock's `skills`,
and adjacent agent-task tools, the recurring pattern is:

- define intent outside the model's short-term memory
- hand the agent a small bounded packet
- run a fresh session for the next slice
- keep durable artifacts in the repo or another persistent system

This matters because `special` already fits the durable-artifact layer of that
workflow better than it fits the orchestration layer.

Relevant sources:

- https://dev.to/maiobarbero/my-ai-assisted-workflow-20ke
- https://news.ycombinator.com/item?id=47775653
- https://github.com/mattpocock/skills
- https://github.com/tim-projects/tasks-ai
- https://michael.roth.rocks/research/543-hours/

### 2. Developers want durability without methodology bloat

The signal from `spec-kit` issues and brownfield-oriented docs is not "no specs."
It is "less ceremony, less artifact sprawl, better small-change support."

The pain shows up as:

- too many generated documents
- heavyweight greenfield flows
- friction for tiny deltas
- process that feels performative rather than operational

That is a strong fit for a repo-native contract layer, but a weak fit for
positioning `special` as a full workflow framework.

Relevant sources:

- https://github.com/github/spec-kit/issues/1174
- https://github.com/github/spec-kit/issues/1191
- https://docs.bmad-method.org/how-to/brownfield/
- https://docs.bmad-method.org/how-to/brownfield/quick-fix-in-brownfield/

### 3. The durable truth is moving into repo-local artifacts

The more teams use AI coding tools, the more they need persistent project truth
that survives session loss and agent drift. The durable objects vary by team,
but the theme is consistent:

- specs
- task definitions
- review findings
- architecture notes
- local skills

This is a direct argument for positioning `special` around durable repo truth,
not around ephemeral prompting tricks.

Relevant sources:

- https://dev.to/maiobarbero/my-ai-assisted-workflow-20ke
- https://github.com/mattpocock/skills
- https://michael.roth.rocks/research/trust-topology/
- https://agents.md/

### 4. Evidence and inspectability matter more than agent theater

The strongest adjacent research does not celebrate maximal autonomy. It focuses
on trust surfaces:

- what does the system claim
- what evidence supports the claim
- what is implemented where
- where does drift accumulate

This maps unusually well to `special`'s shipped capability set, especially the
combination of specs, verifies/attests, and module ownership/metrics.

Relevant sources:

- https://michael.roth.rocks/research/trust-topology/
- https://michael.roth.rocks/research/gate-analysis/
- https://michael.roth.rocks/research/543-hours/
- https://devblogs.microsoft.com/all-things-azure/agentic-devops-practices-principles-strategic-direction/

### 5. Brownfield adoption is the market-critical constraint

Most real teams are not starting from a greenfield demo repo. They need tools
that can be added to an existing codebase without replacing everything else.

The more `special` reads as:

- repo-native
- incremental
- ordinary-file compatible
- light on ceremony

the stronger its adoption prospects become.

The more it reads as:

- new mandatory methodology
- large planning framework
- multi-stage process engine

the weaker its adoption prospects become.

Relevant sources:

- https://docs.bmad-method.org/how-to/brownfield/
- https://spec-weave.com/docs/workflows/brownfield/
- https://github.com/github/spec-kit/issues/860
- https://github.com/github/spec-kit/issues/916

### 6. The trust gap is growing faster than AI adoption

Broad developer data and practitioner writing point in the same direction:

- AI coding adoption is high and rising
- trust in AI accuracy is much lower than usage
- "almost right" output creates debugging and review drag
- real-world repos punish generic assumptions hardest at integration boundaries

This is good for `special` because it can be positioned around trust,
inspectability, and brownfield coherence rather than raw generation speed.

Relevant sources:

- https://survey.stackoverflow.co/2025/ai/
- https://blog.nilenso.com/blog/2025/05/29/ai-assisted-coding/
- https://blog.nilenso.com/blog/2025/09/15/ai-unit-of-work/

## Adjacent Market Map

### Instruction and memory files

Examples:

- `AGENTS.md`
- Copilot instructions
- `CLAUDE.md`
- Cursor rules

These solve persistent guidance and consistent prompting. They do not solve
live/planned truth, support visibility, or module ownership.

### Skills and task-packet systems

Examples:

- Matt Pocock `skills`
- task packet and issue-triage skill repos

These solve execution packaging and repeatable agent behavior. They are adjacent
to `special skills`, but they still sit around the work rather than defining the
product contract itself.

### Workflow and orchestration systems

Examples:

- Sandcastle
- OpenSpec
- SpecWeave
- BMAD flows

These solve process flow, planning artifacts, sandboxed execution, or broader
spec-driven delivery. They are useful comparators, but they are heavier than
`special`'s natural lane and come with more methodology baggage.

### Verification and trust research

Examples:

- Rothrock's trust topology and gate analysis work
- enterprise agentic DevOps guidance

This cluster is strategically important because it validates the long-term wedge:
developers need stronger verification and evidence surfaces, not just better
generation.

## Managed-Service Implications

### Why `special cloud` is a plausible product, not just a hosting layer

The research supports a managed service if it stays centered on contract review,
validation, and attestation rather than generic workflow management.

There are three recurring external signals behind that:

- repo-local truth is valuable, but many stakeholders will not consume it from
  source files or local CLIs
- teams want persistent review surfaces and history, not just raw model calls
- verification is becoming its own product category inside agentic development

This makes a managed contract remote a plausible paid wedge.

### Best cloud job

The strongest cloud job is:

take the repo-native contract and make it reviewable, attestable, and
validator-backed for the broader team.

That means:

- hosted materialized specs and modules
- validation history across revisions
- team-facing attestation workflows
- visibility for non-source-reading stakeholders

### Why cloud attestation matters

Many attestations are not code and are not automated. They may come from PM,
QA, security, compliance, or leadership stakeholders who still need to judge the
state of a claim.

Local `special` can encode and display those attestations, but a managed service
turns them into a real collaboration surface.

### Strong v1 managed-service wedge

The best v1 wedge is not "we host your specs." It is:

- managed spec-vs-verify review
- managed module-vs-implementation review
- stakeholder-visible cloud attestation

This is more defensible than a generic "run an LLM over `special specs --all`"
story because the product can own:

- context extraction
- stable prompt framing
- structured output
- revision-aware history
- lower-noise judgment UX

### Risks

The main risks are:

- drifting into generic dashboard software
- overpromising source-rewriting or cloud-first editing too early
- selling "provable correctness" before the validation layer is already useful

The cleaner arc is:

1. local contract and evidence
2. managed validation and attestation
3. proof-backed correctness

## Persona And Segment Model

### Primary segment: brownfield agentic staff engineer or tech lead

This person is already using Codex, Claude Code, Cursor, or similar tools in a
large existing codebase. Their core problem is not code generation. It is drift
and review burden:

- the agent forgets prior intent
- local changes violate broader product assumptions
- architectural ownership is unclear
- final review catches contradictions late

Why they would adopt `special`:

- it keeps product truth in the repo
- it can be added gradually
- it makes proof and ownership inspectable
- it does not require a new hosted system

Why they would ignore it:

- if it sounds like another process framework
- if small changes need too much ceremony
- if it cannot show value before full adoption

What proof they need:

- a tiny brownfield bootstrap
- fast live/planned inspection
- direct proof attachment to claims
- obvious module-ownership value on day one

Relevant sources:

- https://survey.stackoverflow.co/2025/ai/
- https://blog.nilenso.com/blog/2025/09/15/ai-unit-of-work/
- https://docs.bmad-method.org/how-to/established-projects/

### Primary segment: OSS maintainer or high-volume reviewer

This person wants a durable project contract that contributors and agents can
both inspect. They care about:

- explicit expectations
- easier validation of support
- keeping repo conventions local

They are especially likely to care about `special skills` because installed
skills turn the repo's preferred workflow into something an agent can actually
follow.

Their main fear is AI-shaped review noise. They need proof that `special`
produces smaller, more checkable claims rather than more generated prose.

Relevant sources:

- https://devguide.python.org/getting-started/generative-ai/index.html
- https://docs.kernel.org/process/coding-assistants.html

### Secondary segment: product engineer shipping many small changes with AI help

This person wants fast iteration, not a doctrine. Their pain is that AI is fast
locally but weak on cross-cutting product truth.

They will adopt only if:

- the first use case is small and concrete
- the syntax feels lightweight
- they can refine existing truth in place

Relevant sources:

- https://survey.stackoverflow.co/2025/ai/
- https://docs.bmad-method.org/how-to/brownfield/quick-fix-in-brownfield/

### Secondary segment: developer productivity / platform team evaluating standards

This segment thinks in terms of repeatable engineering practice. They are
interested in:

- repo portability
- evidence surfaces
- team-level consistency
- future policy hooks

They are attractive later, but dangerous to optimize for too early. Enterprise-
sounding messaging can push the product away from its more natural early wedge.

### Weak segment for now: greenfield methodology shoppers

Teams shopping for a full AI workflow framework may try `special`, but they are
not the best initial fit. The product does not currently do orchestration,
workflow state transitions, or tracker integration.

Trying to win this segment through marketing would create avoidable messaging
debt.

### Anti-persona: team shopping for a full AI workflow suite

This team wants planning flows, orchestration, sandboxes, branch strategies,
proposal/apply lifecycles, or tracker-centric execution. Those needs are real,
but they are not `special`'s current product lane.

## Jobs To Be Done

The clearest jobs `special` can honestly claim today are:

### 1. Keep durable project truth where agents can inspect it

When AI sessions are short-lived and context drifts, I want the product contract
to live in-repo so fresh sessions can recover what matters without relying on
someone's memory.

### 2. Separate what ships now from what is only planned

When teams discuss future work in the same places they document current
behavior, I want a live/planned split so the repo can stay honest.

### 3. Show what evidence supports a claim

When a spec exists, I want to inspect the attached verifies and attestations so
I can judge whether support is real, weak, or missing.

### 4. Show what code claims ownership of an architectural module

When architecture intent and implementation drift apart, I want a concrete module
view with implementation ownership and analysis evidence.

### 5. Teach agents the repo's preferred contract workflow

When I drop an agent into the repo, I want installable skills that encode how to
work with the spec and architecture surface without re-explaining the process in
every session.

## Adoption Triggers And Ignore Signals

### Strong triggers

- AI coding use is already spreading through a shared repo
- review burden and inconsistency are rising
- the team needs to answer "what is true today?" faster than code spelunking
- there is recurring pain around stale docs, unclear ownership, or hidden
  contract changes

### Strong ignore signals

- solo prototype work where cleanup debt is acceptable
- teams shopping for a full orchestration or project-management suite
- teams unwilling to maintain any repo-native contract surface at all

## Messaging Language Analysis

### Terms that are legible

- AI-assisted coding
- repo-native
- live vs planned
- durable project truth
- module ownership
- implementation ownership
- contract and evidence

### Terms that need explanation but are usable

- living contract
- attestation
- brownfield-friendly

### Terms to de-emphasize or avoid

- semantic spec
- proof layer
- architecture evidence as a headline term
- agent memory
- workflow engine
- approval gates
- drift detection as a shipped claim

The best phrasing stays close to concrete developer questions:

- what is live?
- what is planned?
- what supports this claim?
- which code implements this module?

## Category Analysis

### Option 1: semantic spec tool

Pros:

- precise for people already bought into the idea
- technically correct

Cons:

- cold and internal
- does not naturally communicate why AI-heavy teams should care
- sounds parser-first rather than workflow-value-first

Verdict:

Keep as secondary implementation language, not primary positioning.

### Option 2: AI coding workflow framework

Pros:

- maps to an obvious budget line
- larger adjacent market

Cons:

- misleading for the actual product
- attracts comparisons with orchestration-heavy systems
- increases the risk of expectations around review gates, planning engines, or
  tracker integrations

Verdict:

Avoid.

### Option 3: repo-native contract and evidence layer

Pros:

- connects directly to durable truth
- naturally fits brownfield repos
- differentiates from hosted systems and long PRDs
- matches current shipped surface

Cons:

- still somewhat novel language
- needs one sentence of explanation

Verdict:

Best primary category phrase family.

### Option 4: repo-native living contract

Pros:

- memorable once explained
- fits the live/planned model well

Cons:

- more abstract on first read
- should not stand alone without clearer supporting language

Verdict:

Best supporting phrase family.

### Option 5: architecture evidence layer

Pros:

- distinctive
- extends the trust story beyond behavior claims

Cons:

- too narrow as the front-door category today

Verdict:

Good expansion wedge, not the only headline.

## Recommended Positioning

### Core framing

`special` is a repo-native contract and evidence layer for AI-assisted
development.

It gives a codebase:

- an honest live vs planned product contract
- attached proof and attestation for claims
- an architecture view with implementation ownership and evidence
- installable repo skills that teach agents how to work with that contract

### Why this framing wins

It maps to real developer pain without implying capabilities the product does not
ship. It is broad enough to cover specs, proof, modules, and skills, but narrow
enough to avoid becoming "another AI workflow framework."

### What to say

Prefer language like:

- repo-native living contract
- contract and evidence layer
- live vs planned
- inspectable support
- architecture evidence
- module ownership
- durable project truth
- inspectable support
- brownfield-friendly
- ordinary repos

### What to avoid

Avoid language like:

- workflow gates
- human approval engine
- agent orchestration
- project management
- autonomous software factory
- end-to-end methodology
- replacement for Jira, Linear, or Notion

## Suggested Message Stack

### Hero direction

Keep AI coding grounded in repo truth.

### Supporting explanation

`special` turns repo annotations into a live contract: what is live, what is
planned, what supports each claim, and which code implements each module.

### Audience qualifier

Best for teams already using AI in shared brownfield repos and feeling the cost
of drift, review thrash, and unclear ownership.

## Messaging Implications

### Best front-door story

The strongest story is not "write specs."

The strongest story is:

AI coding moves faster than repo truth. `special` keeps the truth durable,
inspectable, and attached to evidence inside the repo itself.

### Best proof points

The highest-signal proof points from the current capability set are:

- live vs planned split
- attached verifies and attests
- architecture ownership plus metrics
- support inspection via verbose and unsupported views
- repo-local skill installation for agents

### Best adoption promise

You can add `special` to an existing repo without adopting a giant new process.

That promise should stay explicit.

## Product Evolution Guidance

### Highest-leverage next wedge: contract diffs and drift review

`SPECIAL.ROADMAP.SPEC_DIFFS` is the most market-aligned roadmap item because it
helps answer a universal AI-era question:

what changed in the contract?

It also turns `special` from a static inspection surface into a day-to-day review
surface without requiring orchestration.

### Second wedge: deeper architecture evidence

The current module metrics surface is already a differentiator. Extending it is a
good next move because it answers:

- what code actually implements this area?
- how complete is the declared ownership?
- where are the weak or uncovered boundaries?

This is a strong trust narrative and likely more differentiated than generic spec
authoring alone.

### Third wedge: stronger attestation integrity

Signed attestations, hash binding, and TTL enforcement are strategically aligned,
but they are probably more valuable after the contract-diff story is stronger.

Otherwise the product risks reading like compliance infrastructure before it has
won the day-to-day developer use case.

### Fourth wedge: policy only after visibility is strong

Support-policy enforcement can matter later, especially for platform teams, but
policy before visibility usually feels punitive.

The market signal suggests:

1. show truth
2. show evidence
3. show drift
4. then optionally enforce policy

### What not to optimize for

Do not evolve first toward:

- orchestration-first product strategy
- tracker-specific workflow lock-in
- giant artifact generation
- mandatory multi-step planning flows for tiny deltas

## Bottom Line

The best current market position for `special` is:

a repo-native contract and evidence layer for AI-assisted development, with
module ownership and agent skill installation as the key differentiators.

That framing is honest to the shipped product, legible to current developer pain,
and leaves room for the roadmap to deepen trust without collapsing into workflow
theater.

## Source Set

Seed research:

- https://dev.to/maiobarbero/my-ai-assisted-workflow-20ke
- https://news.ycombinator.com/item?id=47775653
- https://github.com/mattpocock/skills
- https://github.com/tessellate-digital/notion-agent-hive
- https://github.com/tim-projects/tasks-ai
- https://github.com/mattpocock/sandcastle
- https://github.com/mattpocock/sandcastle/issues/191
- https://github.com/github/spec-kit
- https://github.com/github/spec-kit/issues/1174
- https://github.com/github/spec-kit/issues/1191
- https://github.com/github/spec-kit/issues/860
- https://github.com/github/spec-kit/issues/916
- https://docs.bmad-method.org/how-to/brownfield/
- https://docs.bmad-method.org/how-to/brownfield/quick-fix-in-brownfield/
- https://docs.bmad-method.org/how-to/established-projects/
- https://docs.bmad-method.org/explanation/project-context/
- https://michael.roth.rocks/research/trust-topology/
- https://michael.roth.rocks/research/gate-analysis/
- https://michael.roth.rocks/research/543-hours/
- https://agents.md/
- https://docs.github.com/en/copilot/how-tos/custom-instructions/adding-repository-custom-instructions-for-github-copilot?tool=visualstudio&trk=public_post_comment-text
- https://code.claude.com/docs/zh-CN/memory
- https://docs.cursor.com/fr/context/rules
- https://aider.chat/docs/git.html
- https://raw.githubusercontent.com/mattpocock/sandcastle/main/README.md
- https://github.com/Fission-AI/OpenSpec
- https://spec-weave.com/docs/workflows/brownfield/
- https://survey.stackoverflow.co/2025/ai/
- https://blog.nilenso.com/blog/2025/05/29/ai-assisted-coding/
- https://blog.nilenso.com/blog/2025/09/15/ai-unit-of-work/
- https://devguide.python.org/getting-started/generative-ai/index.html
- https://docs.kernel.org/process/coding-assistants.html
- https://devblogs.microsoft.com/all-things-azure/agentic-devops-practices-principles-strategic-direction/

Internal inputs:

- `_project/inbox/2026-04-15-external-research-positioning-and-recommendations.md`
- `/Users/gk/Downloads/Gemini-`@spec` Syntax in Developer Tools.md`
- successful `special specs --all` snapshot captured in this session
- successful `special modules --all --metrics` snapshot captured in this session
