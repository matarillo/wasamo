# DD-M3-P6-007 — ScrollView conditional-content policy

**Status:** Proposed
**Phase:** M3-Phase 6
**Surfaced by:** T4 review follow-up semantic-migration audit
(`fix/m3-phase-6-t4-review-followup`); the conditional × exact-one
container interaction the `Vec<IrMember>` migration left undefined.

## Context

Phase 6 introduces the `if` construct and is responsible for defining how
it interacts with every existing container's cardinality invariant. It
already did so for ZStack (placement materialised order, DD-M3-P6-002),
Grid / Cell (reject direct conditionals), Box (at-most-one count), and the
runtime validators (descend into conditional bodies). **ScrollView's
"exactly one content child" (DD-M3-P4-003) × conditional member was
missed.**

The T4 review-follow-up closes the *uncontested* half: `ScrollView {
Content  if c { … } }` (a widget child plus a conditional sibling) is a
potential two-child shape and is rejected at both `wasamoc check` and the
runtime `validate()`. That rejection is correct under either option below.

The *open* question is the conditional-only case:

```
ScrollView { if c { … } }
```

When `c` is false this materialises **zero** content children, violating
"exactly one". Today both gates reject it (the count sees zero widgets,
and the T4-follow-up interim rejects any direct conditional member in a
ScrollView). Should that stay rejected, or should ScrollView be permitted
to be *conditionally empty*?

## Sub-issue

Exact-one-cardinality container semantics when the single content slot is
filled by a conditional whose presence is dynamic (0-or-1 at materialise
time, and toggling at runtime under T5 reactivity).

## Options

- **(a) Reject conditional-only content** — ScrollView rejects a direct
  conditional member entirely (symmetric with Cell); a conditional must be
  wrapped in the content widget (`ScrollView { Box { if c { … } } }`).
  - Gain: DD-M3-P4-003 untouched; minimal; matches the Cell precedent;
    sufficient for the gallery (no conditionally-empty ScrollView needed);
    a one-widget author workaround exists (wrap). This is the interim the
    T4 review-follow-up already ships.
  - Give up: cannot express "a scroll region whose entire content appears
    only under a condition" without a wrapper widget.

- **(b) Allow conditionally-empty content** — relax "exactly one" to "at
  most one materialised"; `ScrollView { if c { … } }` is valid and shows
  empty when `c` is false.
  - Gain: more expressive; uniform with Box's at-most-one tolerance of a
    lone conditional.
  - Give up: **reopens DD-M3-P4-003** (a prior Accepted DD) + dsl_spec /
    architecture sync; needs **reactive toggle-to-empty Windows-runtime
    evidence** (T5-adjacent — should be deliberated before T5 closes so
    the evidence folds into T5 rather than rebuilding the harness);
    generalises into a broader "conditionally-empty containers" design
    question (also touches WrapPanel / other cardinality containers).

## Comparison

(a) is the conservative close that keeps DD-M3-P4-003 intact and is
sufficient for everything Phase 6 actually ships; the cost is a wrapper
widget for an edge case. (b) is more expressive and more uniform with
Box's at-most-one tolerance, but it relaxes a prior Accepted DD, pulls in
reactive evidence and spec sync, and opens the general "may a cardinality
container be conditionally empty" question — none of which the Phase 6
deliverable requires.

## Recommendation

**Pending deliberation.** Status stays `Proposed` — do not flip to
`Accepted` without the owner's comparison. Tentative lean: **(a)** for
Phase 6, recording the conditionally-empty generalisation as a future
option if it becomes a real need. The T4 review-follow-up already ships
the (a)-consistent interim (conditional-only ScrollView content stays
rejected), so accepting (a) is a no-op confirmation (plus an
intent-revealing diagnostic) and accepting (b) is the change that adds a
Phase 6 implementation task (T4b). Deliberation should land **before T5
closes** so a (b) outcome can coordinate its reactive evidence with T5.

## Preamble integration

This DD is not indexed in `preamble.md` §Decisions while it is `Proposed`
(the preamble records accepted decisions only). On acceptance it is added
to the §Decisions index and a Revisions entry records the mid-phase
addition surfaced by the T4 review.
