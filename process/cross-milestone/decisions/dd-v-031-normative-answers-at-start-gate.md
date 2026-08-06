---
title: Vision Decision Record — Normative answers at the start gate
---

# Vision Decision Record — Normative answers at the start gate

**ID:** DD-V-031
**Status:** Accepted 2026-08-07
**Scope:** Process rule for implementation-gate artifacts

## Context

Since M3-Phase 2's framing, a phase may synchronise its normative text at
**Moment 1** — when its decision set is accepted, *ahead* of implementation
— leaving a re-verification against the landed runtime for the phase close
(Moment 2). M4-Phase 2 did exactly that: `dsl_spec.md` §4.19 and
`architecture.md` §13 were written and committed before T1.

M4-Phase 2's T3 then had to decide whether a **disabled** Button consumes a
click or lets it reach its ancestors. The decision set does not answer it.
DD-M4-P2-001 fixes the consumption rule in general terms; DD-M4-P2-002's
sentence about a disabled Button "stopping the walk" is about hit-test
descent, not propagation. Reading only the decision set, the question looks
open and looks like a new semantic worth escalating to the owner.

It was not open. `dsl_spec.md` §4.8 and §4.19 both already state the
answer — *"Having run no handler, it also does not end propagation: the
event continues to its ancestors as it would from any widget without a
handler"* — because the Moment 1 sync had settled it. The task reached that
text, but only after treating the question as an open design decision
first.

The general shape: **when the normative text is written ahead of
implementation, the decision record is the reasoning and the specification
is the answer.** Nothing in the gates says so, and an implementer who
reads the decision set — the natural reading order, since the decisions are
the phase's own work product — can conclude that a settled question is open.
The cost is not only a wasted escalation: an implementer who does *not*
notice can also land behaviour that contradicts the shipped text, which the
phase close then finds as a divergence.

## Decision

In a phase whose normative text is synchronised **ahead of
implementation**, an implementation task's **start-gate artifact** must
additionally list the **normative statements that already answer the
behaviour this task is about to build** — the document, the section, and
what each fixes — or record explicitly that none does.

A question with such an answer is not an escalation and not a new design
decision. Where the normative text and the decision set appear to disagree,
this rule does not choose between them: the disagreement is **recorded** as
a divergence for the phase-close re-verification to settle, not resolved
silently in either direction.

Bound: only the statements bearing on the behaviour this task builds — not
a re-read of the specification. In a phase that synchronises its normative
text at phase close rather than ahead of implementation, this rule does not
fire.

## Enforcement

**Tier: Forcing.** The list is an auditable artifact: a reviewer can check
that the cited statements exist, say what the task claims they say, and
agree with what landed. It also gives the phase-close re-verification a
list to compare against rather than a re-read, which is where the same
information is needed a second time. A Hard tier cannot judge relevance; a
Soft tier is the state this failure already occurred under.
`implementation-gates.md` §1's start-gate artifact paragraph is updated in
the same accepted-decision batch.

## Evidence and boundary

M4-Phase 2 T3's disabled-Button question is the origin, recorded in
[M4-Phase 2 log.md](../../milestone-4/phase-2/implementation/log.md) §T3
start gate and in its
[retrospective](../../milestone-4/phase-2/retrospectives/t3.md) learning
(b). The landed behaviour agrees with the specification, so this case cost
a detour rather than a defect — which is why it is worth fixing cheaply now
rather than after it costs one.

**Falsifier:** a task treats as an open design decision, or escalates to
the owner, something the phase's already-synchronised normative text
fixes. If that recurs after this rule is in force, the rule did not fire.

This decision does not change which document governs, does not add a
reading obligation to phases that synchronise at close, and does not
displace the phase-close re-verification (Moment 2), which remains the
place where specification and implementation are reconciled.

## Revision rule

If a later phase wants this obligation to extend to phases that
synchronise their normative text at close, or to make the specification
authoritative over the decision set on disagreement, it must file a
successor vision decision record first. A qualification that leaves this
scope unchanged belongs in a dated annotation.
