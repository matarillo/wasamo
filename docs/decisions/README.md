# Architecture Decision Records

## Scope and relation to RFCs

This directory holds **Architecture Decision Records (ADRs)**: per-phase implementation decisions agreed between the project owner and collaborators during M1-M2 (BDFL governance).

From M3 onward, the project transitions to RFC-based consensus for substantial feature proposals. Community-facing RFCs will live in `docs/rfcs/`. ADRs in this directory remain the authoritative record for decisions made before that transition.

The governance policy is defined in [VISION.md §9.2](../../VISION.md#92-decision-making).

Most ADRs are bound to a single phase: one file per phase, recording
all decisions made during that phase's pre-implementation agreement
step. A second category exists for vision/roadmap revisions that take
place outside any single phase.

## File naming

```
phase-<N>-<short-topic>.md      (phase pre-implementation decisions; most common)
vision-<short-topic>.md         (revisions to VISION.md / ROADMAP.md outside any single phase)
```

Examples:

- Phase ADRs: `phase-2-runtime-foundation.md`, `phase-3-layout-engine.md`
- Vision ADRs: `vision-m1-acceptance-criteria.md`

A vision ADR records changes to VISION.md or ROADMAP.md that are
not contained within a single phase's pre-implementation review. It
should reference the phase ADR(s) that triggered the revision (if any),
and the phase ADRs in turn reference the vision ADR rather than
restating vision-level decisions.

## Entry format

Each decision entry uses the following structure:

```
### DD-<scope>-<seq> — <title>

**Status:** Accepted | Superseded by DD-<scope>-<seq>

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

`<scope>` is:

- `P<N>` for phase ADRs (e.g. `DD-P3-001` is the first decision in the Phase 3 ADR)
- `V` for vision ADRs (e.g. `DD-V-001` is the first decision in any vision ADR; vision ADRs share a single `V` numbering space)

## Revision rule

When a later phase overrides a prior decision:

1. In the **original entry**, change `Status` to  
   `Superseded by DD-P<M>-<seq> (<phase-M-file>.md)`
2. In the **new entry**, add a `Supersedes` line:  
   `Supersedes: DD-P<N>-<seq> (<phase-N-file>.md)`
3. Never delete or rewrite old entries — keep them as historical record.
