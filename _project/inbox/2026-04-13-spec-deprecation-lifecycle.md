# Proposal: Spec Deprecation / Removal Lifecycle

## Problem

`special` is good at two states today:
- live
- planned

There is a missing lifecycle state when a claim is currently live but the product has decided to remove or invert it.

This shows up when a team wants to back out of a live claim cleanly:
- the old live claim is no longer directionally correct
- the replacement behavior may not be fully implemented yet
- the team may want removal-proof tests that pass when the old behavior is gone
- deleting the old claim immediately can erase useful change intent

Recent example shape:
- a live claim says a command emits a full retrieval packet
- product direction changes and that behavior should go away
- the team wants an explicit removal path instead of pretending the old claim is still healthy until the last minute

## What Seems Needed

A claim lifecycle that can express:
- this currently ships
- this is on the way out
- removal is intended for a target release
- support may temporarily focus on proving absence/removal behavior

## Recommendation

Add `@deprecated <release>` as metadata, but do not let it replace honest behavior claims.

Suggested semantics:
- `@deprecated <release>` means:
  - the claim is still part of the current live contract
  - the team intends to remove or replace it by the named release
  - `special spec --all` should surface it distinctly from plain live and plain planned claims
- deprecation should not by itself authorize inverted proofs against the old positive claim text
- if the replacement behavior is specific enough, teams should still write a separate `@planned` successor claim with honest replacement wording

Example:

```text
@spec RETRIEVAL.DO_PACKET_INCLUDES_TASK_CONTEXT_AND_INTENT
Opening a do emits a retrieval packet that includes task context and local intent.
@deprecated 0.1.0
```

paired with

```text
@spec TASK_ENTRY.RENDERS_FULL_TASK_CONTEXT_ON_START_OR_RESUME
Entering a task via start or resume renders the full durable task context.
@planned
```

and possibly

```text
@spec LOCAL.DO.RENDERS_ONLY_LOCAL_INTENT_AND_MODE
Opening a do renders local intent and mode without re-emitting the full task context.
@planned
```

## Why This Is Better Than Pure Inversion

If teams only invert the old claim, the spec becomes confusing:
- the old claim text says behavior exists
- the test passes only when behavior is removed
- support and contract are now pointing in opposite directions

That should stay invalid.

So the healthy pattern is:
- old claim may be marked `@deprecated <release>` while it is still shipped
- new behavior gets its own planned/live claim with honest text
- once the removal lands and the replacement is live, the deprecated claim is deleted

## Optional Future Addition

If `special` wants a stronger lifecycle, it could later add a distinct tombstone state such as `@removed <release>` for short-lived release-transition visibility in `spec --all`.

But that feels less important than first adding:
- visible deprecation metadata
- lint rules that keep deprecated claims honest
- better release-target reporting for deprecations

## Lint / UX Suggestions

Possible lint behavior:
- allow `@deprecated <release>` only on live `@spec` claims
- require an explicit release string
- reject `@deprecated` on already-planned claims
- keep requiring support to match the literal claim text
- optionally warn when a deprecated claim has no adjacent successor planned claim

Possible `special spec --all` rendering:
- `[deprecated 0.1.0]` next to the live claim
- still show normal support counts

## Bottom Line

`@deprecated <release>` looks useful as contract metadata.

But it should not be treated as "the opposite of planned," and it should not let teams prove the negation of a still-positive claim.

The honest flow should remain:
- mark old live claim deprecated
- write the replacement as its own planned/live claim
- remove the deprecated claim once the replacement ships
