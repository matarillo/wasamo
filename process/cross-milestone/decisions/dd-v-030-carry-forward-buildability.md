---
title: Vision Decision Record — Carry-forward buildability
---

# Vision Decision Record — Carry-forward buildability

**ID:** DD-V-030
**Status:** Proposed
**Scope:** Process rule for implementation-gate artifacts

## Context

M4-Phase 2's T2 close gate recorded a carry-forward naming a concrete
authored shape — a `Button` carrying a `WidgetNode` child — and made it a
**required evidence item** for T3: a click on such a child had to be shown
activating the Button through the ancestor walk, "or the narrowing ships".
The conclusion was reached by reading three layers that all admit the
shape: `wasamoc check` accepts it, the IR loader builds the child, and
`dsl_spec.md` §4.16 shows one in an example. The shape was never built.

A fourth layer refuses it. `build_layout_tree` maps a Button to a childless
layout node, so the child is never arranged, has no rectangle, and is not a
hit candidate. T3 measured the consequences: in a release build the click
resolves to the **Button** and fires it — the opposite of what the
carry-forward asserted — and in a debug build the shape aborts during
`wasamo_load_ui` on T2's own child-count assertion, so **a fixture that
builds it cannot run at all**. The evidence T3 was required to produce was
unbuildable, and T3 spent its start gate discovering that rather than
building the behaviour it owned.

Trap #5 already requires "evidence" and a re-trigger criterion for every
carry-forward, and T2's entry satisfied that as written: its evidence was
the structural side-effect enumeration. What the trap does not distinguish
is a carry-forward **about code that ran** from one **about a shape that
was reasoned about**. The first is a description; the second is a
prediction, and a prediction handed to a later task as a required
obligation is where the cost lands.

## Decision

When a task's close gate records a carry-forward that **obliges a later
task to produce evidence for a named shape**, the close artifact must also
record that the shape was **built and run once** in the recording task: the
fixture or probe used, and the result observed.

If the shape cannot be built or cannot be run, it is **not a
carry-forward**. It is a finding, and the close gate records it as one,
with an owner — a named task, or an owner decision. "The next task closes
this" is false in that case, and the classification must say so.

Bound: this applies only to a carry-forward that **requires evidence of a
later task**. A carry-forward that records an invariant to preserve — an
ordering, an identity, a validation rule — is unchanged and needs no run;
those describe code that already executes. A throwaway probe is sufficient
evidence and is expected to be discarded; the artifact is the recorded
result, not a retained test.

## Enforcement

**Tier: Forcing.** The carry-forward's evidence cell names the run and what
it produced, which a reviewer can compare against the shape the
carry-forward names and against the obligation it places on a later task.
A Hard tier is not feasible — no check can tell which carry-forwards oblige
a later task — and a Soft tier is what this failure already passed through:
trap #5's prose asked for evidence and got a reading.
`implementation-gates.md` §2 artifact #5 is updated in the same
accepted-decision batch.

## Evidence and boundary

The T2 → T3 case above is the origin, recorded with its measurements in
[M4-Phase 2 log.md](../../milestone-4/phase-2/implementation/log.md) §T3
start gate (finding 1) and close gate (CF-1). It is the first case in this
project where a plan-required evidence item turned out to be unbuildable,
which is what distinguishes it from the ordinary case of a carry-forward
that is merely imprecise.

**Falsifier:** a later task finds that an evidence item its plan requires
cannot be built. If that recurs after this rule is in force, the rule did
not fire and its shape is wrong.

This decision does not require a task to build every shape it mentions, and
it does not widen trap #5's ordinary evidence requirement. It adds one
obligation on the narrow class of carry-forwards that spend a later task's
budget.

## Revision rule

If a later phase wants to extend this obligation to carry-forwards that do
not oblige a later task, it must file a successor vision decision record
first. A qualification that leaves this scope unchanged belongs in a dated
annotation.
