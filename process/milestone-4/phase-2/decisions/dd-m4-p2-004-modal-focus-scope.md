# DD-M4-P2-004 — The structure-independent modal focus scope

**Status:** Proposed
**Phase:** M4-Phase 2
**AC:** AC1 (modal focus scope), and the rule halves of AC10 and AC5

## Context

AC1 names this explicitly: a modal focus scope is *"attachable to any
subtree, so a root `ZStack` branch and a top-layer overlay are both
consumers of one concept."* The milestone's disposition is **one design,
three implementations** ([plan.md](../../plan.md) §Cross-phase
dispositions 1): the concept is settled here, the `ZStack` consumer is
built here, the top-layer consumer in M4-Phase 9, the screen-reader
consumer in M4-Phase 11.

This is the decision the milestone is most likely to be judged on. If
M4-Phase 9's top layer cannot use the concept unchanged, the thesis is
materially weakened even if both apps work — which is why the correct
response there is a supersede of this record, not a local special case
(milestone-end criterion 3).

The framing adds a constraint that is easy to state and easy to violate:
the scope is **not a property of a layer**. A design in which "things on
the top layer trap focus" would satisfy Phase 9 and fail Phase 2,
because A's lightbox stays a root `ZStack` branch and never touches the
top layer ([M4 framing](../../requirements/framing.md) §粒度の定義
〔重ね表示〕, design condition 1).

## What the spike measured

Three results shape the options
([exploration/focus-traversal-spike.md](exploration/focus-traversal-spike.md)):

- **Containment needs no modal-specific branch.** Confining traversal is
  "enumerate Tab stops from a different root". The walk itself has no
  modal case (§4.1 Q4).
- **The restore target cannot be derived from the tree.** Nothing in the
  structure records which node was focused before the scope opened, so
  it must be captured at entry (§4.1 Q5).
- **"Modal scope" is two separable mechanisms wearing one name** (S-3).
  *Containment* comes from the entry, and *"not tabbable while closed"*
  comes from the annotation. The spike's first version checked neither
  against the other, and the mechanism fixture's containment assertions
  passed against a projection that had dropped the annotation entirely.
  That was found by mutation, not by review.

S-3 is the finding this decision is built around, because it is the one
a reader would otherwise get wrong: the natural mental model is that a
scope is one thing.

## Sub-issues

- **What a scope is** — annotation, structure, or event.
- **What it contains** — traversal, pointer input, both.
- **Esc** — who consumes it and what closing means.
- **Restoration** — where the target comes from.
- **Nesting.**
- **Screen-reader modality** — the rule, fixed here, implemented in
  M4-Phase 11.
- **The second structure** — what Phase 9 must be able to do.

## Options

- **S1 — annotated subtree plus explicit enter / exit.** A subtree
  carries a scope annotation; entering is an act that captures the
  restore target.
- **S2 — derived from existence.** A scope is active whenever its
  annotated subtree is present in the tree; there is no enter act.
- **S3 — a property of the top layer.** Content on the top layer traps
  focus; the root-`ZStack` case is handled separately.

## Comparison

**S3 is excluded by the framing**, and the reason is worth restating
because it is the whole point of the AC: A's lightbox is a root `ZStack`
branch and must trap focus. S3 would need a second mechanism for it, and
"two mechanisms for one concept" is exactly the failure the milestone
thesis exists to prevent. Recorded as considered-and-rejected rather
than omitted, since it is what most toolkits do.

**S2 is genuinely attractive and fails on one measured point.** It is
the most declarative option: the author writes a conditional subtree,
and its presence *is* the modal state — no enter act, no state to keep
in sync with the tree, and closing is deleting. Containment works
(traversal enumerates from the innermost present scope). It fails on
**restoration**: when the lightbox closes, focus must return to the
thumbnail that opened it, and no amount of looking at the tree afterwards
can recover which node that was. S2 would have to store the restore
target somewhere anyway — at which point it needs an act to store it at,
and it is S1 with the act hidden.

Two weaker points, recorded because they support the same conclusion:

- Under S2 there is no place to reject an ill-formed scope, so any
  container annotated as a scope becomes modal by existing — including
  one whose visibility is controlled by something other than the
  conditional the author had in mind.
- Nesting order under S2 is derived from tree depth. That is right for
  a dialog inside a menu, and wrong for two sibling scopes opened in
  sequence — a stack records what happened, tree depth records only
  where things are.

**S1's cost is real**: it introduces state that must agree with the
tree, and if a scope's subtree is removed without an exit the stack
holds a dangling entry. That is a genuine hazard and is handled below
rather than argued away.

### What containment covers

Traversal confinement is the core. **Pointer input is confined by
DD-002's target selection, not by this scope** — a full-bleed scrim
inside the scope's subtree is the topmost widget over everything behind
it, so background clicks resolve to the scrim and go no further. That is
why the scrim is a `Box` an author writes, not a concept the scope
supplies: the scope confines the *keyboard*, and the scrim confines the
*pointer*, and they are separate because they are separately visible to
the author.

This is a deliberate narrowing of "modal" and it is stated so a later
phase does not assume more: a scope with no scrim traps Tab and does not
block clicks. B4's rename dialog (Phase 9) will need both, and will
compose both.

### Esc

The scope **names** the Esc target; it does not define what closing
means. `esc_target` returns the innermost entered scope, and the act of
closing — removing the subtree, clearing the state the conditional reads
— is authored. The core never mutates the tree.

This matters for the DSL: Esc is delivered by DD-001's bubble walk, and
it reaches the scope because the scope is an ancestor of the focused
widget. Nothing special-cases Esc in the routing model; what the scope
adds is a well-defined recipient. If no scope is entered, Esc bubbles to
the root and is unhandled, which is correct.

### Restoration

Captured at entry, restored at exit, held on the scope's stack entry.
When the entered scope's subtree is removed, **restoration wins over
structural succession** — the spike pins this with a test that goes red
when the restore branch is deleted (mutation M7). Without that
precedence, closing the lightbox would drop focus onto whatever
survived, typically the first toolbar tab, rather than the thumbnail the
user was on.

### The two mechanisms (S-3), stated as a requirement

The annotation and the entry are tied together by requiring that
**only an annotated subtree may be entered**. Without that check the two
mechanisms drift: any container could be entered and confine traversal,
and the annotation would silently be doing nothing but hiding closed
subtrees from Tab. That is not a hypothetical — it is what the spike
measured before the check was added.

Both halves are therefore part of this decision:

1. An un-entered scope contributes **no Tab stops**, so a closed modal
   subtree that is still in the tree is not reachable.
2. Only a subtree annotated as a scope may be entered.

### Nesting

Scopes stack, innermost wins, and entry / exit are balanced. M4 has no
nested case — B4's dialog-from-a-menu is Phase 9 — but the stack costs
nothing over a single slot and removes a Phase 9 redesign. Recorded as
"supported, unexercised in M4", which is an honest state rather than a
claim.

### Screen-reader modality — the rule

**Background content is hidden from the screen reader by focus scope,
not by layer.** Fixed here as binding on M4-Phase 11, implemented there.
This is the framing's design condition 3 and the third consumer of the
one-concept claim.

Its practical content for Phase 11: the accessibility tree's visible
subtree is the innermost entered scope, computed from the same stack
this decision defines — not from a layer test, and not from a per-widget
`hidden` flag maintained in parallel.

### The second structure — what Phase 9 must be able to do

Stated as an obligation on this decision, so Phase 9 can check it rather
than discover it: the top-layer overlay must be usable as a scope
**by carrying the same annotation**, with no top-layer-specific rule in
the traversal or in the Esc path. Concretely, Phase 9 must be able to
open its menu by annotating the realized subtree and entering it, and
must get containment, Esc and restoration with no new concept.

The spike gives grounds to expect this and does not prove it: its scopes
were ordinary subtrees, and a top-layer subtree differs in *where it is
realized* (window level) rather than in what the traversal sees. But the
top layer's tree position is exactly what Phase 9 designs, and if it
turns out that the realized subtree is not an ancestor of the focused
widget, DD-001's bubble path for Esc breaks. **That is the specific
falsifier**, named here so Phase 9 tests it first rather than at the end.

## Recommendation

**S1 — an annotated subtree plus an explicit enter / exit**, with:

- **Containment = traversal only.** Pointer confinement comes from
  DD-002's topmost-target rule plus an authored scrim.
- **Two tied mechanisms**: an un-entered scope contributes no Tab
  stops, and only an annotated subtree may be entered.
- **Esc** is delivered by ordinary bubbling to the innermost entered
  scope; the scope names the recipient, the author defines closing.
- **Restoration** is captured at entry and takes precedence over
  structural succession when the subtree is removed.
- **Nesting** by stack; supported, unexercised in M4.
- **Screen-reader modality attaches to the scope**, binding on
  M4-Phase 11.
- The `ZStack` consumer (A's lightbox) is implemented in this phase;
  the other two are not.

## Forward-compat exposure

- **M4-Phase 9** consumes this unchanged or supersedes it. The named
  falsifier is whether the realized top-layer subtree is an ancestor of
  the focused widget for Esc's bubble path.
- **M4-Phase 11** consumes the stack to compute the accessibility
  tree's visible subtree.
- **M4-Phase 5 / 6** put a text field inside a scope; the text field
  will consume keys before they bubble, and the precedence is
  M4-Phase 6's decision. Bubbling is what makes it expressible.
- **A dangling stack entry** is the shape of failure to watch for: a
  scope's subtree removed without an exit. The recommendation's
  restoration-on-removal rule covers the case the runtime can see;
  a scope removed by a path that does not consult focus at all is the
  residual, and its re-trigger is any new subtree-removal path.
- **"Modal" is narrower here than in common usage** — no click blocking
  without a scrim, no window-level modality. A later phase wanting
  either adds it; neither is silently implied.

## Technical risk re-evaluation

- **The concept could still break at the second structure.** That is
  the milestone's headline risk and it cannot be closed here. It is
  mitigated by naming the falsifier and by the supersede path being
  written down in advance, so Phase 9 cannot resolve it with a quiet
  special case.
- **The two-mechanism split is invisible in ordinary testing.** A test
  that checks only "the background is unreachable" passes with either
  mechanism broken. The phase's evidence must include the leg where the
  scope is *closed* and the same Tab reaches the background — the
  must-agree half of the control (M4-Phase 1 F-50).
- **Restoration is state, and state can disagree with the tree.**
  Mitigated by capturing at entry and by the removal precedence; the
  residual is recorded above rather than claimed closed.
