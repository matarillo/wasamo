# DD-M3-P6-007 — ScrollView conditional-content policy

**Status:** Accepted
**Phase:** M3-Phase 6
**Surfaced by:** T4 review follow-up semantic-migration audit
(`fix/m3-phase-6-t4-review-followup`); the conditional × exact-one
container interaction the `Vec<IrMember>` migration left undefined.

## Context

Phase 6 introduces the `if` construct and is responsible for defining how
it interacts with every existing container's cardinality invariant. It
already did so for ZStack (placement materialised order, DD-M3-P6-002),
Grid (reject — children must be `Cell`-wrapped, a structural rule) and
`Cell` (reject — `Cell` requires exactly one content child, DD-M3-P5-001),
Box (at-most-one count, DD-M3-P2-001), and the runtime validators (descend
into conditional bodies). **ScrollView's "exactly one content child"
(DD-M3-P4-001, loader gate DD-M3-P4-006) × conditional member was missed.**

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
  - Gain: DD-M3-P4-001 untouched; implementation-minimal; matches the
    `Cell` precedent (Cell rejects a direct conditional for the same
    exactly-one-content-child reason, DD-M3-P5-001); sufficient for the
    gallery (no conditionally-empty ScrollView needed); a one-widget author
    workaround exists (wrap). This is the interim the T4 review-follow-up
    already ships.
  - Give up: cannot express "a scroll region whose entire content appears
    only under a condition" without a wrapper widget — and that wrapper is
    **not** a semantic Fragment (Phase 6 has none); it is a real content
    widget (`Box` / `VStack` / …) with its own layout/measurement
    behaviour inside the scroll viewport, and a real node that M4's
    AccessKit / UIA accessibility mapping must account for — carry or
    deliberately flatten
    ([_roadmap.md M4](../../../_roadmap.md#m4-interaction-stack)) — whereas a
    neutral Fragment would add no node to account for in the first place.
    The author therefore inherits a real structural node, not a zero-cost
    syntactic shim, until an explicit empty / fragment content model exists
    (see [Deferred design space](#deferred-design-space)).

- **(b) Allow conditionally-empty content (direct-conditional form)** —
  relax "exactly one" to "at most one materialised"; `ScrollView { if c {
  … } }` is valid and shows empty when `c` is false. Call this the
  **direct-conditional form**: an `else`-less `if` directly in the content
  slot emits a 0-or-1 materialised child. It is only one way to reach
  conditionally-empty content; a future relaxation need not take this shape
  and could instead arrive through a different content-value model (see
  [Deferred design space](#deferred-design-space)).
  - Gain: more expressive; uniform with Box's at-most-one tolerance of a
    lone conditional.
  - Give up: **reopens DD-M3-P4-001's exact-one content invariant** (a
    prior Accepted DD; loader gate DD-M3-P4-006) plus the dependent
    content-size / offset-clamp assumptions in DD-M3-P4-003 / DD-M3-P4-005
    that presume content is present — but **not** DD-M3-P4-003's offset-y
    literal shape or read-only binding-direction decisions, which stay
    intact; plus dsl_spec / architecture sync. Needs **ScrollView-specific
    reactive toggle-to-empty Windows-runtime evidence** — a direct-
    conditional ScrollView content toggling 1→0→1 while viewport / clip,
    content-size, and offset-clamp behaviour stay correct and same-drain
    observation holds, under the DD-M3-P6-004 / DD-M3-P6-005 conditional
    runtime contract (this is the ScrollView-specific delta, distinct from
    the *general* conditional present/absent those DDs already settle).
    T5-adjacent — deliberate before T5 closes so the evidence folds into T5
    rather than rebuilding the harness. It also generalises into a broader
    "conditionally-empty containers" design question (also touches
    WrapPanel / other cardinality containers).

## Comparison

(a) is the conservative close that keeps DD-M3-P4-001 intact and is
sufficient for everything Phase 6 actually ships; the cost is a wrapper
widget for an edge case — and, because Phase 6 has no neutral Fragment,
that wrapper is a real layout node (with its own measurement behaviour, and
a node M4's AccessKit / UIA accessibility mapping must account for —
carry or deliberately flatten), not a zero-cost syntactic shim. (b) is more
expressive and more uniform with Box's at-most-one tolerance of a lone
conditional (Box admits a 0-child body outright, DD-M3-P2-001), but it
relaxes a prior Accepted DD, pulls in reactive evidence and spec sync, and
opens the general "may a cardinality container be conditionally empty"
question — none of which the Phase 6 deliverable requires.

Crucially, choosing (a) rejects only the **direct-conditional form** for
ScrollView (`ScrollView { if c { … } }`); it does **not** reject
conditionally-empty scroll content as a semantic direction. The (b) weighed
here is specifically that form, not the direction itself. What (a) actually
does is *defer* the larger question of how conditional content should be
typed and normalised in the DSL (see
[Deferred design space](#deferred-design-space)) — a deferral, not a verdict
against conditionally-empty containers.

## Deferred design space

Choosing (a) for Phase 6 settles ScrollView's *current* surface syntax; it
does not settle how conditional content is typed and normalised in the DSL,
and it does not reject conditionally-empty scroll content as a future
semantic direction. That larger question is deferred, not decided against.

The deferral exists because two readings of a content-position `if` pull in
different directions, and Phase 6 does not need to pick between them:

- **Imperative member emission** — `if` is a member-emitting construct, so
  an `else`-less `if` naturally yields a 0-or-1 materialised member, and a
  conditionally-empty ScrollView is the direct continuation. This is the
  model option (b) assumes today.
- **Typed / expression-like content** — as the DSL grows more typed and
  expression-oriented, an `else`-less `if` in content position is not
  necessarily the right final primitive. Absence could instead be expressed
  as an optional / `Maybe` content *value*, as an explicit empty-content
  *identity* (cf. SwiftUI `EmptyView`), or via a *grouping* construct (cf.
  React fragments, SwiftUI `Group`) that gives authors an explicit place to
  host conditional content. These are not interchangeable — a value-level
  `Maybe`, an empty-view identity, and a grouping wrapper differ in what
  they make first-class — which is exactly why the base model has to be
  chosen deliberately rather than inherited from today's syntax.

Which of these is the base content model determines what "conditionally
empty" even means and how a relaxed (b) would actually be spelled. The
candidate base models to choose among when (b) is reconsidered are
therefore: **imperative member emission**, **optional typed content**, and
**explicit empty / fragment content**. Phase 6's current IR already
*mechanically* uses member-level conditional emission, but adopting (a)
deliberately declines to canonise that mechanism as the DSL's final content
model — it settles ScrollView's surface syntax for now and commits to none
of these as the answer. These future content-value models need not be an IR
replacement: an optional / typed / fragment author surface could still
lower into the accepted O1 member-level IR (DD-M3-P6-004) — the deferred
question is the author-facing and typing model, not the Phase-6 IR shape.

## Recommendation

**Accepted: (a)** (owner comparison 2026-06-04). For Phase 6, ScrollView
rejects the direct-conditional form — **not** rejecting conditionally-empty
scroll content as a semantic direction.
Reconsideration of (b) should be tied to the DSL's broader
conditional-content model rather than treated as a ScrollView-only
ergonomics tweak: the DSL should first decide whether conditional content
is modelled as imperative member emission, optional typed content, or
explicit empty / fragment content (see
[Deferred design space](#deferred-design-space)). The T4 review-follow-up
already ships the (a)-consistent interim (conditional-only ScrollView
content stays rejected), so accepting (a) is a *code-path* no-op relative
to that interim — but **not** a design no-op: it confirms a **normative**
ScrollView rule (a direct conditional-only member is rejected) plus its
diagnostic wording, adding that rule to the public spec and the preamble
index (see [Preamble integration](#preamble-integration)). Accepting the
direct-conditional form of (b) instead is the change that adds a Phase 6
implementation task (T4b).

Only the Phase-6 **direct-conditional (a-vs-b)** choice is time-boxed: it
should land **before T5 closes** — not because Phase 6 leans toward (b), but
because that is the only window in which a (b) outcome could fold its
reactive toggle-to-empty evidence into T5 rather than rebuilding the
harness. The broader content-model choice in
[Deferred design space](#deferred-design-space) carries **no** such T5
deadline; it stays deferred.

## Preamble integration

This DD is not indexed in `preamble.md` §Decisions while it is `Proposed`
(the preamble records accepted decisions only). On acceptance it is added
to the §Decisions index and a Revisions entry records the mid-phase
addition surfaced by the T4 review.

**Accepted consequence (a).** Acceptance does **not** modify DD-M3-P6-003's
general member-placement rule (`if` is admitted inside a widget body).
Wasamo's model is two-layer: that general rule says *where* a conditional
may appear; each container then decides whether a *direct* conditional
survives, and it does so for **different reasons**. `Cell` (DD-M3-P5-001)
and ScrollView (DD-M3-P4-001) reject it on **cardinality** grounds — their
exactly-one-content-child invariant cannot absorb a dynamically-zero
member; `Grid` rejects it on a **structural** ground instead (children must
be `Cell`-wrapped); `Box` (DD-M3-P2-001) tolerates it because its
at-most-one count admits a 0-child body. DD-007 records ScrollView's answer
under that existing model, on the same cardinality basis as the `Cell`
precedent — it is not a new exception to DD-003, and it does not touch
DD-M3-P4-003's offset-y / binding-direction surface. The public-spec sync
is therefore narrow: `docs/dsl_spec.md` §4.11 (ScrollView) gains one
sentence (**any** direct conditional member under ScrollView is rejected —
the conditional-only centre case, plus the conditional *sibling* the T4
follow-up already closed; wrap the conditional inside the single content
widget) and the §4.14 rejected-shape / diagnostic list gains the matching
entry — touching the §4.14 diagnostics prose but **not** the conditional
grammar *production* (the `conditional_member` BNF rule), which already
admits `conditional_member` wherever `member` appears and then restricts it
semantically (so the spec does not over-claim unconditional validity).

## Implementation handoff if (a)

Acceptance of (a) is expected to be a doc/process change, not a code change.
The checklist of record is
[implementation/plan.md T4b](../implementation/plan.md); this section is the
consequence map, **not** a second checklist.

**Touch**
- This DD — `Status: Proposed → Accepted` + a Revision history line.
- `decisions/preamble.md` — add DD-007 to the §Decisions index and a
  Revisions entry (the mid-phase addition surfaced by the T4 review).
- `docs/dsl_spec.md` — one sentence in §4.11 (ScrollView) per the
  Accepted-consequence wording above, plus the matching rejected-shape entry
  in the §4.14 conditional-rendering diagnostics list. plan.md T4b's (a)
  bullet names this dsl_spec §4.11 / §4.14 sync (reconciled during this
  deliberation — the diagnostic was added by the T4 follow-up *after* the
  preamble verification-closure list was written, so it was absent from the
  original bullet).

**No-touch**
- Code — the T4 review-follow-up interim *is* the final rule; no `wasamoc` /
  runtime change (barring a micro-fix should the shipped diagnostic wording
  diverge from the §4.11 spec sentence).
- Prior DDs — DD-M3-P4-001 / DD-M3-P4-003 / DD-M3-P5-001 / DD-M3-P2-001 are
  untouched (this is the per-container reading of an existing model, not a
  revision of any of them).

**Final evidence (already shipped — both gates; (a) adds no new test).**
- `wasamoc check` — `scrollview_conditional_member_rejected` (sibling) and
  `scrollview_conditional_only_member_rejected` (the centre case).
- runtime `validate()` — `validate_rejects_scrollview_with_conditional_member`
  and `validate_rejects_scrollview_with_conditional_only_member`, returning
  `IrLoadError::Validate` → `WASAMO_ERR_IR_MALFORMED` at the C ABI boundary
  (DD-M2-P6-005 C ABI shape / DD-M2-P6-009 malformed-IR policy — the mapping
  the loader comment cites), carrying the intent-revealing "a conditional
  member is not valid directly in ScrollView … — see DD-M3-P6-007"
  diagnostic. The
  ControlFlow gate fires ahead of the child-count gate, so the conditional
  diagnostic (not a bare "got 0") is what an author sees.

## Revision history

All review passes kept `Status: Proposed`.

- Strategic / owner-alignment review: scoped (a) to rejecting only the
  current direct-conditional syntax, not the conditionally-empty direction;
  added §Deferred design space.
- Owner refinement (ChatGPT): tightened that section's vocabulary; recast the
  defer as design-driven, not demand-driven.
- Recommendation-choice review: made the Accepted consequence explicit — the
  two-layer placement-vs-cardinality model, code-path vs design no-op, and
  the T5 deadline scoped to the a-vs-b choice.
- Recommendation-choice review #2 (repo-verified): corrected the exact-one
  citation to DD-M3-P4-001 (not DD-M3-P4-003, the offset-y DD); separated
  Grid's structural rejection from Cell/ScrollView's cardinality rejection.
- Implementation-readiness review: broadened the spec-sync to *any* direct
  conditional member; added §Implementation handoff if (a) with the
  dual-gate evidence.
- Implementation-readiness review #2: clarified the §4.14 sync as diagnostics
  prose, not grammar production; confirmed the DD-M2-P6-005 / DD-M2-P6-009
  error-mapping citation.
- Implementation-readiness review #3: reconciled plan.md T4b (named the
  dsl_spec sync; fixed the (b) prior-DD reference).
- **Accepted (a)** — owner comparison 2026-06-04; `Status: Proposed →
  Accepted`. Synced `docs/dsl_spec.md` §4.11 / §4.14, preamble §Decisions +
  Revisions, and plan.md T4b.
