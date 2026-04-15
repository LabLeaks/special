# PRODUCT

## Status

Working product vision and positioning document for `special`.

This file is strategic and descriptive. It is not the product contract. Shipped
behavior still lives in `special` specs and modules.

## Product Vision

`special` gives agentic software development a path from repo truth, to managed
validation, to provable correctness.

As coding agents make local implementation cheaper, the scarce resource shifts to
something else:

- durable project truth
- honest separation of live vs planned behavior
- inspectable evidence for what is claimed
- visible ownership of what code implements what architecture
- stakeholder-visible attestation and review context

`special` exists to make that truth explicit without forcing teams into a giant
new methodology.

## Product Thesis

AI coding tools accelerate local change but magnify four failure modes:

1. intent drifts across sessions
2. planned work gets mistaken for shipped behavior
3. code changes outrun the repo's shared understanding of what is true
4. important stakeholders cannot reliably inspect or attest to that truth

The winning product in this space is not necessarily the biggest workflow
framework. It is the lightest trustworthy layer that makes repo truth durable,
inspectable, and usable by both humans and agents.

That is the wedge `special` should own.

## Product Arc

### Phase 1: local developer tool

The first phase is the current wedge.

The job is:

- keep repo truth explicit for agents and reviewers
- make live vs planned behavior visible
- attach support to claims
- attach ownership to modules
- keep adoption lightweight in brownfield repos

This is where `special` wins on local DX.

### Phase 2: `special cloud`

The second phase is the team product and the paid wedge.

The job is:

- make the contract visible without local checkout
- give teams managed validation over spec/test and module/implementation
- support cloud attestation from stakeholders who are not reading source
- preserve revision history and remote concurrency over contract state

This is where `special` wins on team UX.

### Phase 3: provable correctness breakthrough

The third phase is the long-term moat.

The job is:

- upgrade critical claims from heuristic review to machine-checkable proof
- bridge natural-language contracts, implementation, tests, and proof artifacts
- make formal assurance usable inside agentic software development rather than
  separate from it

This is where `special` wins on trust.

## Positioning

### Primary position

`special` is a repo-native contract and evidence layer for AI-assisted
development.

### Secondary expansion

It combines:

- contract materialization
- support and attestation inspection
- module ownership and architecture evidence
- agent skill installation

### Full-arc position

`special` starts as the repo-native truth layer for coding agents, grows into
the managed validation and attestation layer for software teams, and ultimately
becomes the bridge from AI-assisted development to provably correct systems.

### Short product description candidates

- Repo-native contract and evidence for AI-assisted development.
- Keep AI coding grounded with live specs, support, and module ownership.
- Durable repo truth for agents and reviewers.
- Managed contract validation and attestation for AI-built software.
- From agentic speed to provable trust.

## Who It Is For

### Primary users

#### 1. AI-forward staff engineer or tech lead in a brownfield repo

This user is already using coding agents and needs the repo to hold up under
faster change. They care about drift, cross-cutting consistency, and durable
project truth.

#### 2. OSS maintainer or high-volume reviewer

This user cares about clarity, inspectability, and repo-local conventions that
do not depend on a hosted system.

### Secondary users

#### 3. Product engineer shipping many small AI-assisted changes

This user will adopt if the tool feels lightweight and useful on day one.

#### 4. Platform or developer productivity team

This user becomes more relevant once `special` has stronger diff, drift, and
policy-adjacent stories.

#### 5. Non-source-reading stakeholder in cloud workflows

This user may sit in PM, QA, security, compliance, or leadership. They need to
inspect, review, and sometimes attest to product truth without reading source
files or using the CLI directly.

### Anti-persona

Teams shopping for a full workflow suite, orchestration layer, or project
manager are not the right front-door fit today.

## Core Jobs To Be Done

`special` should be sold against a small number of clear jobs.

### Keep durable project truth in the repo

When an agent starts fresh, the repo should still be able to answer:

- what is supposed to be true
- what is only planned
- what evidence supports the live claims

### Make support inspectable

When a claim exists, engineers should be able to inspect the support surface
rather than trust prose or memory.

### Make module ownership explicit

When teams ask what code actually implements a module, the repo should answer in
a concrete inspectable way.

### Teach agents how the repo wants work done

When a repo has a preferred contract workflow, agents should be able to install
and follow repo-native skills instead of relearning that workflow from scratch.

### Make stakeholder attestation possible without source reading

When the repo needs manual or external attestations, stakeholders should be able
to review the relevant contract and evidence in a managed surface rather than by
reading source comments and raw tests.

## Honest Current Product Story

Today `special` already does enough to support a real product story:

- materializes live and planned specs
- renders specs as text, JSON, and HTML
- surfaces attached verifies and attests
- reports unsupported live claims
- materializes architecture modules and areas
- shows implementation ownership and module analysis metrics
- lints structural and reference errors
- installs bundled skills for contract-centric agent workflows

Today `special` does not do:

- task orchestration
- review or approval gates
- hosted collaboration
- project management
- built-in contract diffing
- attestation enforcement or signing

That boundary should remain explicit in user-facing messaging.

The important product consequence is that local `special` is the source-of-truth
and authoring surface today, while `special cloud` is still a strategy layer
rather than a shipped product.

## Narrative

### The problem

AI coding is making code generation abundant, but trustworthy repo context is
still scarce.

Teams can generate implementation faster than they can preserve:

- product truth
- support evidence
- architectural intent
- consistent agent behavior across fresh sessions
- stakeholder-visible review and attestation context

### The answer

`special` keeps that truth in the repo as a living contract with evidence and
architecture ownership attached.

It is not the agent. It is not the tracker. It is not the orchestration engine.

It is the durable contract layer that agents and humans can both inspect.

Then `special cloud` turns that contract into a managed validation and
attestation surface for teams.

Later, a proof layer can turn selected claims from "this appears aligned" into
"this has machine-checkable support."

## Messaging Pillars

### 1. Durable truth beats prompt memory

Fresh sessions are normal. The truth should survive them.

### 2. Live and planned should stay separate

The repo should not blur current behavior with aspiration.

### 3. Claims need inspectable support

Evidence should sit next to the contract, not in someone's head.

### 4. Architecture needs evidence too

The repo should show not just what is claimed, but what code claims ownership of
the implementation.

### 5. Repos should be able to teach agents their workflow

Bundled skills turn repo-local expectations into something agents can actually
follow.

### 6. Attestation should be legible to non-developers

Many important attestations are non-code and non-automated. A managed product
needs to let the right stakeholders inspect the contract and evidence without
turning them into source readers.

## Category Strategy

### Recommended category stack

Primary:

- repo-native contract and evidence layer

Supporting phrases:

- repo-native living contract
- support and ownership for AI-assisted development
- durable project truth for coding agents

### Terms to de-emphasize

- semantic spec tool
- workflow framework
- agent orchestration
- agent memory
- approval gates

`semantic spec tool` is accurate, but it is not the best front-door language.
It sounds like an implementation detail instead of a resolved developer pain.

The best message family stays closer to concrete questions:

- what is live?
- what is planned?
- what supports this claim?
- which code implements this module?

## Adoption Strategy

### Land with one concrete use case

The first adoption story should be:

add a live/planned contract and support surface to an existing repo without
changing the rest of your stack.

The strongest initial ICP is the brownfield-accountable engineer who is already
using AI and now needs more trust and coherence, not more speed theater.

### Expand into architecture evidence

Once teams trust the contract surface, the next differentiator is architecture
ownership and metrics.

### Expand into agent enablement

`special skills` is a meaningful wedge because it lets a repo install its own
preferred task-shaped workflows into agent runtimes.

### Expand into cloud validation and attestation

The first managed-service story should not be generic hosting. It should be:

- hosted materialized contract view
- managed spec/test alignment review
- managed module/implementation alignment review
- stakeholder-facing attestation and revision history

This is the clearest paid product wedge because it gives teams better, faster,
cheaper review judgments than running ad hoc LLM passes themselves.

### Only later expand into policy and stronger assurance

Policy and enforcement become compelling after visibility and diff/drift review
are already strong.

## Roadmap Guidance

### Highest-priority evolution

#### 1. Contract diffs and drift review

This is the strongest near-term product move. It answers a universal question:

what changed in the contract?

It also makes `special` more central to day-to-day review without making it
heavier.

#### 2. Deeper architecture evidence

The architecture module and metrics surface is already differentiated. Extending
it is consistent with the trust story and likely more defensible than generic
spec tooling alone.

#### 3. Attestation integrity

Signed attestations, claim-hash binding, and TTL enforcement fit the long-term
trust thesis, but they should follow stronger everyday visibility features.

#### 4. Managed validation and cloud attestation

Before hard policy, the strongest cloud move is managed judgment:

- does this verify block appear to support this claim?
- does this declared module appear to match the implementation slice?
- does this attestation still look current and adequate for this change?

This is where `special cloud` can create paid value without becoming generic
workflow software.

#### 5. Support policy enforcement

Useful later, especially for platform teams, but only after the product clearly
shows value as an inspection layer rather than a compliance burden.

#### 6. Proof-backed correctness

Longer term, a specialized proof system can become the product breakthrough:

- natural-language claim to formal proof artifact
- proof-aware bridges between spec text, tests, and implementation slices
- selective upgrade from heuristic confidence to machine-checkable assurance

This should be treated as a later moat, not the first monetization story.

### What to avoid

`special` should not evolve first into:

- a giant AI workflow framework
- a multi-agent planner
- a task board
- a mandatory process layer for tiny changes

## Proof Needed To Win Adoption

The product story will land only if teams can see near-immediate proof:

- a tiny brownfield bootstrap path
- fast answers to live vs planned questions
- direct visibility into attached support
- module ownership that helps reviewers reason about change faster than grep
- repo-local skills that agents can follow without a hosted system
- a plausible managed-service path that helps non-developer stakeholders inspect
  and attest without reading source

## `special cloud` Strategy

### What cloud is

`special cloud` should be the managed remote for the contract surface:

- materialized specs and modules pushed from CI
- hosted revision history
- managed validation passes
- stakeholder-facing attestation and review flows

### What cloud is not

It should not begin as:

- a generic AI dev platform
- a task orchestrator
- a project-management system
- a blind source-rewriting service

### Why cloud attestation matters

Many `@attests` are non-code and non-automated. They often involve people who:

- do not use the CLI
- do not read source files
- still need to judge whether a claim is adequately reviewed or supported

Cloud makes those attestations first-class instead of forcing them through ad hoc
docs, screenshots, or PR comments.

### Cloud v1

`special cloud` v1 should focus on:

- read-only hosted contract and module views
- revision history per spec and module id
- managed spec-vs-verify validation
- managed module-vs-implementation validation
- stakeholder-visible attestation surfaces

### Cloud v2

After v1 proves value, expand into:

- semantic contract diffs
- stale attestation surfacing
- advisory support-policy views
- stronger reviewer workflows

### Cloud v3

Only after the validation layer is strong:

- stricter policy hooks
- signed and bound attestations
- proof-backed correctness workflows for selected claims

## Messaging Guardrails

### Say this

- repo-native
- contract and evidence
- living contract
- live vs planned
- support and attestation
- architecture ownership
- module ownership
- durable project truth
- brownfield-friendly
- predictability without ceremony
- managed validation
- cloud attestation

### Do not say this

- approval gates
- workflow control plane
- autonomous delivery engine
- full SDLC replacement
- AI project manager
- agent memory system
- drift detection platform
- generic hosted dashboard
- blind source sync magic

## The Position To Win

`special` should aim to own this sentence:

When AI coding speeds up implementation, `special` keeps the repo's contract,
support, module ownership, and eventually proof surface durable enough for
humans and agents to trust.

That is a sharper and more honest position than "spec tool," and much safer than
"workflow framework."
