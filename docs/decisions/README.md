# Architecture Decision Records

## Scope and relation to RFCs

This directory holds **Architecture Decision Records (ADRs)**: per-phase implementation decisions agreed between the project owner and collaborators during M1-M2 (BDFL governance).

From M3 onward, the project transitions to RFC-based consensus for substantial feature proposals. Community-facing RFCs will live in `docs/rfcs/`. ADRs in this directory remain the authoritative record for decisions made before that transition.

The governance policy is defined in [VISION.md §9.2](../../VISION.md#92-decision-making).

One file per phase. Each file records all decisions made during that phase's
pre-implementation agreement step.

## File naming

```
phase-<N>-<short-topic>.md
```

Examples: `phase-2-runtime-foundation.md`, `phase-3-layout-engine.md`

## Entry format

Each decision entry uses the following structure:

```
### DD-P<N>-<seq> — <title>

**Status:** Accepted | Superseded by DD-P<M>-<seq>

**Context:** Why this decision needed to be made.

**Options:**

Option A — <name>
- What you gain: …
- What you give up: …

Option B — <name>
- What you gain: …
- What you give up: …

**Decision:** Option <X> — <one-line rationale>
```

## Revision rule

When a later phase overrides a prior decision:

1. In the **original entry**, change `Status` to  
   `Superseded by DD-P<M>-<seq> (<phase-M-file>.md)`
2. In the **new entry**, add a `Supersedes` line:  
   `Supersedes: DD-P<N>-<seq> (<phase-N-file>.md)`
3. Never delete or rewrite old entries — keep them as historical record.
