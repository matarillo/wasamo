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

**Status:** Proposed | Accepted | Superseded by DD-<scope>-<seq>

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

- `P<N>` for M1 phase ADRs (e.g. `DD-P3-001` is the first decision in
  the Phase 3 ADR). M1 used global Phase 1–8 numbering.
- `M<N>-P<n>` for phase ADRs from M2 onward (e.g. `DD-M2-P2-001`).
  Phase numbering is local to each milestone from M2; the milestone
  prefix prevents collisions across milestones.
- `V` for vision ADRs (e.g. `DD-V-001` is the first decision in any
  vision ADR; vision ADRs share a single `V` numbering space across
  milestones).

M1 phase ADRs are not renumbered; they remain `DD-P<N>-<seq>` as
historical record.

## Risk evaluation

DD entries evaluate two axes of risk under deliberately distinct
labels:

- **Technical risk** (per option, required) — the risk that
  implementing this option within the phase reveals an unworkable
  premise: build failures, missing platform support, abstraction
  leaks. Converges within the phase via spike / build / test.
  Rated `Low` / `Medium` / `High` on each option's bullet list.

- **Forward-compat exposure** (per DD, conditional) — how exposed
  the recommended option is to revision when post-phase DSL or
  C ABI extensions land. Does not converge within the phase; a
  projected estimate. Written as a paragraph inside the
  Recommendation block, and rated as a column in the DD summary
  table for the recommended option.

The two axes use different labels (`risk` vs `exposure`)
intentionally: a unified `risk` vocabulary would imply equal
epistemic confidence, which the project does not have.

### Conditional rule for the Forward-compat exposure paragraph

The paragraph is written only when candidate options differ on
this axis. Absence of the paragraph signals "options are equally
additive-compatible with foreseeable extensions" — not an
oversight. (The summary-table column is still filled for the
recommended option of every DD.)

The paragraph **must reference the ADR's Out of scope items** as
its source of foreseeable future events, rather than introducing
new ones mid-prose. The Out of scope section is the single truth
source for "events that may revisit this DD"; the exposure
paragraph discusses how each option would survive those events.

### What is *not* a separate axis

**Design quality** (coherence, ergonomics, footgun-avoidance) is
intentionally not formalized as a third axis. It lives in the
Recommendation prose, where it can be argued in context rather
than collapsed into a Low/Medium/High rating. When a DD is
dominated by design quality, the option bullet may say so
explicitly ("the risk axis here is design-quality, not
implementability").

### Adoption

The two-axis evaluation is in effect from the **M2-Phase 3 ADR**
([m2-phase-3-handler-exec-location.md](./m2-phase-3-handler-exec-location.md))
onwards. Earlier ADRs follow the supersede rule and are not
retroactively updated; the absence of the column in older ADRs
reflects the format of their time, not a judgement that the axis
was irrelevant there.

## Revision rule

When a later phase overrides a prior decision:

1. In the **original entry**, change `Status` to  
   `Superseded by DD-P<M>-<seq> (<phase-M-file>.md)`
2. In the **new entry**, add a `Supersedes` line:  
   `Supersedes: DD-P<N>-<seq> (<phase-N-file>.md)`
3. Never delete or rewrite old entries — keep them as historical record.

## Pre-doc discipline

A phase ADR's pre-doc must verify that the proposed approach serves
the **acceptance criterion** the phase is meant to satisfy, not merely
implement the ROADMAP task list literally.

ROADMAP phase task lists are working hypotheses written before pre-doc
began. They are proposals, not constraints. If pre-doc surfaces a
better approach to the same acceptance criterion, the task list is
revised as part of the pre-doc work — the ROADMAP entry is updated
alongside the ADR.

The discipline:

1. Identify the phase's acceptance criterion (from VISION §7 /
   ROADMAP milestone-level goal). This is the constraint.
2. Treat the phase's task list as one candidate path to that
   criterion. Ask: are there other paths? Does the task list actually
   serve the criterion, or does it conflate means with ends?
3. If pre-doc finds the acceptance criterion itself was misframed
   (e.g. it elevates a particular implementation tactic to the level
   of the criterion), file a vision ADR (`vision-<topic>.md`)
   revising VISION/ROADMAP at that level before continuing with the
   phase ADR.

The Phase 5 ADR pair is a worked example. The original ROADMAP task
list ("ImplicitAnimationCollection animates Offset/Size/Opacity")
served a misframed acceptance criterion, requiring both a
vision-level revision ([vision-m1-acceptance-criteria.md](./vision-m1-acceptance-criteria.md))
and a phase-level redirection ([phase-5-implicit-animations-dev-api.md](./phase-5-implicit-animations-dev-api.md)
superseded by its replacement). Without pre-doc discipline the
original task list would have been implemented as written, embedding
a behavior that contradicts the milestone's actual acceptance
criterion.

## Task lists

Implementation task lists — the ordered steps to carry out the Accepted
decisions — belong in the milestone plan's Progress section
(`docs/plans/<M>-plan.md`), not in the phase ADR.

The rationale: even a task list agreed in pre-doc remains a hypothesis.
Build failures, linker errors, CI surprises, and direct application
observation can all reveal that a step needs to be split, reordered,
added, or dropped after the design decisions themselves are settled.
Keeping task lists in the plan's Progress section (which is explicitly
mutable throughout implementation) allows those adjustments without
implying that a design decision has changed. DD entries follow the
supersede rule and stay stable; task steps are operational detail and
stay flexible.

An ADR may include:
- DD entries (the decision record)
- A summary table of DD entries at the end
- An explicit out-of-scope list

An ADR must not include:
- A per-step implementation checklist
