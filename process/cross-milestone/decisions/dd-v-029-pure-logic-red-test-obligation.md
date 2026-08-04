---
title: Vision Decision Record — Pure-logic red-test evidence
---

# Vision Decision Record — Pure-logic red-test evidence

**ID:** DD-V-029
**Status:** Accepted 2026-08-04
**Scope:** Process rule for implementation-gate artifacts

## Context

M4-Phase 1's T2 and T3 reviews showed that a green pure-logic test can be
structurally present while its assertion is redundant, too weak, or shadowed
by an earlier assertion. The phase-end owner decision asked for a narrow
"show it goes red" obligation. The broader proposal — requiring every green,
identical, or passing observation to be falsified — would codify a much wider
and less proportionate evidence policy and was explicitly rejected.

## Decision

For a newly added pure-logic rounding rule, unit-conversion rule, or boundary-
condition branch, the implementation-gate close artifact must include:

1. the test name that exercises the branch directly; and
2. a deliberately wrong implementation (or equivalent mutation) that was
   shown to make that test fail.

This rule applies only to pure logic. It does not require mutation evidence
for Win32/WinRT surfaces, does not widen the GUI screenshot/positive-control
rule, and does not require a universal "green/identical observation must go
red" proof.

## Enforcement

**Tier: Forcing.** The gate artifact names the branch test and the wrong
implementation's observed failure, which a reviewer can compare with the
source and test output. `implementation-gates.md` is updated in the same
accepted-decision batch. The review lane remains branch/test-focused for
diagnostic, reject, and size branches; this rule adds the pure-logic mutation
artifact without changing the high-risk runtime/schema review policy.

## Evidence and boundary

T2's seven-mutation table is the phase evidence for rounding and unit
conversion. T3's three-mutation frame set is evidence for the value of a
positive control, but its GUI form is deliberately outside this decision.
The first implementation of this rule must cite the exact mutation and the
failing test output; a prose assertion that a test is meaningful is not the
artifact.

## Revision rule

If a later phase wants to extend this obligation beyond the three pure-logic
families above, it must file a successor vision decision record first. A
qualification that leaves this scope unchanged belongs in a dated annotation,
not a new status vocabulary.
