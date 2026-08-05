# DD-M4-P2-004 — The structure-independent modal focus scope

**Status:** Accepted
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
  it must be captured when the scope opens (§4.1 Q5).
- **"Modal scope" is two separable mechanisms wearing one name** (S-3).
  In the spike's core, *containment* came from the entry act and *"not
  tabbable while closed"* came from the annotation. The spike's first
  version checked neither against the other, and the mechanism fixture's
  containment assertions passed against a projection that had dropped
  the annotation entirely. That was found by mutation, not by review.

S-3 is the finding this decision is built around, because it names the
state in which the concept can silently fall apart: a subtree that is
present but whose two halves are not held together.

## Sub-issues

- **What a scope is** — annotation, structure, or event.
- **Entry** — what act enters a scope, and what entering does.
- **What it contains** — traversal, pointer input, both.
- **Dismissal** — who receives the request and what closing means.
- **Restoration** — where the target comes from.
- **Nesting.**
- **Screen-reader modality** — the rule, fixed here, implemented in
  M4-Phase 11.
- **The second structure** — what Phase 9 must be able to do.

## Options

- **S1 — annotated subtree plus a separate explicit enter / exit act.**
  The annotation marks the subtree; an act distinct from the subtree's
  presence enters it and captures the restore target.
- **S2 — annotated subtree whose presence is the entry.** Materialising
  the subtree enters the scope and removing it exits; the capture
  happens at materialisation, where the runtime is present.
- **S3 — a property of the top layer.** Content on the top layer traps
  focus; the root-`ZStack` case is handled separately.

## Comparison

**S3 is excluded by the framing**, and the reason is worth restating
because it is the whole point of the AC: A's lightbox is a root `ZStack`
branch and must trap focus. S3 would need a second mechanism for it, and
"two mechanisms for one concept" is exactly the failure the milestone
thesis exists to prevent. Recorded as considered-and-rejected rather
than omitted, since it is what most toolkits do.

**S1 versus S2 is decided by one measured fact and one structural one.**

The measured fact (Q5) is that the restore target cannot be recovered
from the tree after the scope opens. That argues for *capture at a
defined moment*, not for a separate act: the runtime owns the
materialisation path — the structural drain that makes an `if` true,
and the initial build — and the focused widget is still known at that
moment. Presence supplies the capture point; no second act is needed to
have somewhere to stand.

The structural fact is S-3. With entry separate from presence,
"present but not entered" is a reachable state, and the concept splits
into two mechanisms — confinement from the entry, unreachability from
the annotation — that can each fail alone while ordinary tests stay
green; the spike measured exactly that. Deriving entry from presence
removes the state in which the two can drift: **every present scope is
entered, reachability is presence, and a closed modal subtree is simply
absent from the tree.** The annotation keeps the one job it should
have — making presence mean entry.

S1 would also need a spelling for the act, and DD-005 adds none: the
spike's `enter_modal` was test-side scaffolding, and an act with no
authored surface is an internal call the DSL cannot reach.

Two things S1 appeared to buy survive under S2, stated so the choice is
not misread as losing them:

- **Nesting order is still a stack of what happened.** Entry order is
  materialisation order — document order within one build — not tree
  depth, so two sibling scopes opened in sequence nest by their opening
  order.
- **An ill-formed entry cannot exist.** The only path onto the stack is
  the materialisation of an annotated subtree; there is nothing to
  reject.

S2's costs, stated rather than argued away:

- **A present scope always confines.** An author cannot materialise a
  modal subtree in a disabled state; the way not to confine is not to be
  present. No M4 or planned consumer wants a disabled-but-present
  scope, and `if` is exactly the switch the author already holds.
- **A scope in the initial tree is entered at startup.** That is the
  intended reading — a startup dialog — not an accident, and it is
  recorded as behaviour.

### What entry does

Entry is part of the same structural mutation that materialises the
subtree — the drain that makes an `if` true, iteration generating the
subtree, or the initial build — and it does three things:

1. **Pushes the scope** onto the per-window stack, in materialisation
   order.
2. **Captures the restore target**: the widget focused at that moment,
   possibly none.
3. **Moves focus to the scope's first focus stop** in tree order — or
   to none when the scope has no stops, in which case key delivery
   starts at the scope itself (DD-001).

The third step is load-bearing, not a courtesy. Key delivery starts at
the focused widget, so focus left outside the scope's subtree would make
the scope's own authored handlers unreachable — Left / Right on A's
lightbox would be dead until the user's first Tab. Moving focus in on
entry is what makes the authored key surface live the moment the scope
opens, and it is what every reference toolkit's dialog does.

Exit is the removal of the subtree; what it restores is under
Restoration below.

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

### Dismissal

The scope **names** the recipient of a dismissal request; it does not
define what closing means. The innermost entered scope receives the
request, and the act of closing — removing the subtree, clearing the
state the conditional reads — is authored. The core never mutates the
tree.

**Esc is a source, not the concept.** The scope receives a *dismissal
request*, and Esc is the only thing that raises one in M4. M4-Phase 9's
click-away raises the same request, and M5's Dialog widget consumes the
same contract — which is why the recipient is defined here in terms of
the request rather than the key. The authored spelling is DD-005's.

The request is **addressed to the innermost entered scope and stops
there**; it does not continue to outer scopes. A dialog that ignores a
dismissal must not close the menu underneath it. If no scope is entered,
nothing addresses the request and Esc is an ordinary key.

### Restoration

Captured at entry — the materialisation of the subtree — restored at
exit, held on the scope's stack entry. When the entered scope's subtree
is removed, **restoration wins over structural succession** — the spike
pins this with a test that goes red when the restore branch is deleted
(mutation M7). Without that precedence, closing the lightbox would drop
focus onto whatever survived, typically the first toolbar tab, rather
than the widget the user was on. A removal's successor is computed
**before** the mutation, because node identity does not survive a
rebuild (spike Q5).

### Nesting

Scopes stack in materialisation order, innermost wins, and entry / exit
are balanced by construction — presence is the entry, so an unbalanced
stack requires a removal path that bypasses the structural seam. M4 has
no nested case — B4's dialog-from-a-menu is Phase 9 — but the stack
costs nothing over a single slot and removes a Phase 9 redesign.
Recorded as "supported, unexercised in M4", which is an honest state
rather than a claim.

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
than discover it: the top-layer overlay must be usable as a scope **by
carrying the same annotation**, with no top-layer-specific rule in the
traversal, the entry path, or the Esc path. Concretely, realizing the
top-layer subtree must run the **same materialisation seam** that enters
any other scope — stack push, capture, focus move — with no new concept.

The property Phase 9 must verify first rather than last: authored key
handlers on the top-layer container depend on that container being an
**ancestor of the focused widget**, because that is how DD-001's walk
reaches it. Entry's focus move is what establishes the dependency —
focus lands inside the realized subtree — so the concrete check is that
the realization path runs entry, and that the walk from a widget inside
the realized subtree reaches the realized container. Dismissal is
deliberately **not** exposed to that dependency: the request is
addressed to the innermost entered scope rather than walked to from the
focused widget, so it holds even where the walk does not. That is a
second reason to keep dismissal separate from key delivery, beyond the
one DD-005 argues from the sources.

## Recommendation

**S2 — an annotated subtree whose presence is the entry**, with:

- **Entry at materialisation** (structural drain or initial build):
  push in materialisation order, capture the restore target, and move
  focus to the scope's first stop — or to none when it has no stops,
  with key delivery then starting at the scope.
- **Containment = traversal only.** Pointer confinement comes from
  DD-002's topmost-target rule plus an authored scrim.
- **Reachability is presence.** A closed modal subtree is absent from
  the tree; nothing present is un-entered.
- **Dismissal** is addressed to the innermost entered scope and stops
  there; the scope names the recipient, the author defines closing. Esc
  is M4's only source of a dismissal request, and later sources
  (click-away, a Dialog's close control) reuse the same recipient.
- **Restoration** is captured at entry and takes precedence over
  structural succession when the subtree is removed; a removal's
  successor is computed before the mutation.
- **Nesting** by stack in materialisation order; supported, unexercised
  in M4.
- **Screen-reader modality attaches to the scope**, binding on
  M4-Phase 11.
- The `ZStack` consumer (A's lightbox) is implemented in this phase;
  the other two are not.

## Forward-compat exposure

- **M4-Phase 9** consumes this unchanged or supersedes it. Realizing its
  subtree must run the same entry seam, and the ancestor property of
  authored key handlers is checked first (see the second structure).
- **M4-Phase 9 also adds the second dismissal source** (click-away),
  which is where a declarative policy attribute becomes necessary: with
  two sources, "Esc only" and "Esc or click-away" stop being the same
  thing. DD-005 records the value ladder HTML already settled.
- **M4-Phase 11** consumes the stack to compute the accessibility
  tree's visible subtree.
- **M4-Phase 5 / 6** put a text field inside a scope; the text field
  will consume keys before they bubble, and the precedence is
  M4-Phase 6's decision. Bubbling is what makes it expressible.
- **A dangling stack entry** requires a removal path that bypasses the
  structural seam. `window_add_widget`-shaped attach paths are already
  outside the layout boundary (DD-002) and stay cleanup candidates; the
  re-trigger is any new subtree-removal path that does not run the seam.
- **A disabled-but-present scope is not expressible.** A consumer that
  wants one would be changing "presence is the entry", not adding to it,
  and the change is recorded as such in advance.
- **"Modal" is narrower here than in common usage** — no click blocking
  without a scrim, no window-level modality. A later phase wanting
  either adds it; neither is silently implied.

## Technical risk re-evaluation

- **The concept could still break at the second structure.** That is
  the milestone's headline risk and it cannot be closed here. It is
  mitigated by naming the entry-seam and ancestor checks in advance and
  by the supersede path being written down, so Phase 9 cannot resolve it
  with a quiet special case.
- **Confinement evidence still needs its agreement leg.** "The
  background is unreachable" is equally produced by an empty background.
  The phase's evidence must include the leg where the scope is *absent*
  and the same Tab reaches the background — the must-agree half of the
  control (M4-Phase 1 F-50).
- **Restoration is state, and state can disagree with the tree.**
  Narrowed by construction: entry and exit ride the structural seam, so
  disagreement requires a path that bypasses it, and that residual is
  recorded above with its re-trigger rather than claimed closed.
- **Entry runs inside a drain.** The focus move writes runtime focus
  state, not authored state, so it enqueues no further drain work. That
  is an invariant the implementation asserts rather than assumes.
