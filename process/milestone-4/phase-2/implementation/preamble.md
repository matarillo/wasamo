---
phase: M4-Phase 2
title: Event routing, focus model, and generic click handling
status: draft
adr: process/milestone-4/phase-2/decisions/preamble.md
plan: process/milestone-4/plan.md
opened: 2026-08-05
---

# M4-Phase 2 — Event routing, focus model, and generic click handling: Implementation

This is the execution framing for **the milestone's centre of gravity**.
The design decisions are frozen in the ADR set under
[../decisions/](../decisions/preamble.md) (preamble + DD-M4-P2-001
through DD-M4-P2-005, all Accepted 2026-08-05). This file and its
sibling [plan.md](./plan.md) are mutable during the phase; in-flight
decisions and CI evidence land in [log.md](./log.md); phase residuals
land in [handoff.md](./handoff.md) at phase close. The front-matter
`status` flips `draft` → `active` when the owner approves the task list,
and `active` → `closing` at the phase-end batch commit.

## Phase scope

Five deliverables, one per decision:

- **Route** — a pointer or key event goes to one target and then walks
  its ancestors until a handler runs; a handler that runs consumes the
  event; the reactive drain runs once after the walk rather than between
  steps
  ([DD-M4-P2-001](../decisions/dd-m4-p2-001-event-routing-model.md)).
- **Resolve** — hit geometry comes from layout, in DIP, not from a
  Visual readback; exactly one widget is the target, the topmost
  containing one; every widget with a visual is a candidate
  ([DD-M4-P2-002](../decisions/dd-m4-p2-002-hit-testing-and-generic-click.md)).
- **Focus** — one focus record per window, tree-order traversal, group
  traversal with per-group memory
  ([DD-M4-P2-003](../decisions/dd-m4-p2-003-focus-model-and-traversal.md)).
- **Confine** — a modal focus scope is an annotated subtree plus an
  explicit entry, and it confines the keyboard only
  ([DD-M4-P2-004](../decisions/dd-m4-p2-004-modal-focus-scope.md)).
- **Author it** — `clicked` on any widget, per-item handlers inside
  `for`, and exactly two new container attributes
  ([DD-M4-P2-005](../decisions/dd-m4-p2-005-dsl-handler-surface.md)).
  The normative text **landed at Moment 1**
  ([dsl_spec.md §4.19](../../../../docs/dsl_spec.md),
  [architecture.md §13](../../../../docs/architecture.md)), ahead of
  implementation; it is re-verified against the landed runtime at phase
  close.

## Acceptance relation

AC1 ([_roadmap.md §M4](../../../_roadmap.md#m4-interaction-stack)) is
this phase's, in full. Two further criteria have their **rule halves**
fixed here and their implementations elsewhere: the focus rule set bound
to the top layer (AC10, built at M4-Phase 9) and screen-reader modality
(AC5, built at M4-Phase 11). The phase does not claim those criteria;
it claims that a later phase implementing them needs no new concept.

Two rows of the M3 residual cluster close here — thumbnail hit-testing,
and the lightbox's modal focus and input containment
([M3 handoff](../../../milestone-3/handoff.md)).

## The sequencing thesis: runtime-first, except where the tree cannot say it

Phase 1's ordering thesis was "build the machinery, declare last."
This phase's is different, and it was **measured rather than assumed**.

The spike asked what today's `.ui` can express and counted the answer
([spike §4.1 Q6](../decisions/exploration/focus-traversal-spike.md)):
a widget tree can supply **two of the six focus roles**. `Stop` follows
from the widget kind and `Container` from everything else. `Group` and
`ModalScope` have **no representation at all** — nothing an author
writes today says "these three buttons are one Tab stop" or "this
subtree is modal."

That splits the phase in two:

- **Where the existing tree can already supply the input, the runtime
  lands first.** Hit resolution, propagation and Tab traversal need
  nothing new from the author: the gallery already has Buttons, and
  Button-family widgets are already focus stops. T1–T4 therefore land
  against the gallery as it is, and each is checkable the day it lands.
- **Where the tree cannot say it, the authored annotation lands first.**
  Group traversal and modal scopes have no input until `focus-group` and
  `modal-scope` exist, so T5 (the compiler surface) precedes T6 (the
  runtime behaviour). Building T6 first would mean building it against
  the spike's override map — a test-only stand-in — and then rewiring
  it, which is the shape that leaves two projections in the tree.

The generic-click half follows the first pattern for a second reason:
**T1–T3 are behaviour-preserving for the gallery.** Today only Buttons
carry handlers and no two interactive widgets overlap, so single-target
resolution and consume-on-handle produce the same clicks the gallery
already produces. T7 then widens *who may carry a handler* without
touching how an event travels.

## What "green" is worth in this phase

**A green suite is not evidence that routing is right**, for the same
structural reason Phase 1 recorded at its T1: the existing layout
integration tests drive `WidgetNode`s directly and never through a
window, so no existing test routes an event at all.
`cargo test --workspace` stays in every end gate as a **regression
check** — it must not go red — and is not counted as evidence that a
new rule is correct.

Two specific traps this phase has to name in advance, because both
produce a green suite and a correct-looking gallery:

- **Occlusion is unobservable until T9.** Single-target resolution (T2)
  differs from today's fire-every-Button recursion only when two
  interactive widgets overlap, and nothing in the gallery overlaps until
  the lightbox is wired. T2's evidence is therefore its own pure-logic
  tests over a constructed overlapping tree, **shown to fire** against a
  wrong implementation — not the gallery frame.
- **The two cancelling conversions stop cancelling at T1.** Phase 1
  measured that the pointer division and the `visual_rect` division are
  symmetric today, so *"no test can distinguish a correct row 9 from a
  missing one"*
  ([Phase 1 handoff](../../phase-1/implementation/handoff.md), T5). T1
  removes the second division. Until T1 lands, a wrong conversion is
  invisible at any scale; after it lands, a wrong one is invisible at
  100% and wrong at 125%. Every task from T1 onward whose evidence is a
  captured frame states the scale it was captured at.

## The migration obligation is a task-shape constraint, not advice

DD-002 makes the geometry migration **complete or not made**: no
intermediate state may have one path reading the layout cache and
another reading the Visual, because that is where the cancellation
argument silently stops applying to only some paths. T1 and T2 are
therefore ordered so that **no commit between them leaves a mixed
input path**, and T2's close artifact is a call-site audit showing
**zero** `visual_rect` readers on the input path.

## DD-005 is not accepted, and its key-handling half contradicts DD-004

**DD-005 is `Proposed`.** Its key-handling sub-decision (K1 / K2 / K3)
was accepted without being judged on its merits, and re-reading it
against DD-004 shows the accepted shape does not hold.

K1 ships **no authored key surface at all**: Esc is described as
"handled by the scope" and Left/Right as "handled by the lightbox's own
state". Neither is implementable.

- **Left/Right.** The runtime has no notion of a photo index, and with
  no authored key surface there is nothing to bind `selected_index -= 1`
  to.
- **Esc.** DD-004 is explicit that the scope *names* the Esc recipient
  and **does not define what closing means** — *"the act of closing …
  is authored. The core never mutates the tree."* Naming a recipient is
  not a mechanism for reacting: the author still has to clear the state
  the enclosing `if` reads, and K1 gives no place to write that.

So K1 leaves **two** named behaviours without a mechanism, not one, and
the second is a direct contradiction between two decisions in the same
set. Arrow keys *inside a focus group* are unaffected — DD-003 defines
those as focus movement, and no option here changes that.

This is assigned to **T0** below: the phase does not start until it is
resolved, because it decides whether the authored surface gains a key
signal family and therefore what
[dsl_spec.md §4.19](../../../../docs/dsl_spec.md) says. That section
currently carries the K1 position and is rewritten by T0.
A plain-language walkthrough of the options is in
[private/explainer/](../../../../private/explainer/m4-phase-2-key-handling-options.md).

## Verification means

Per [CLAUDE.md §Testing rules](../../../../CLAUDE.md) and the framing's
verification section:

- **Pure-logic unit tests** — the traversal core (already landed by the
  spike), hit resolution over constructed trees including edge
  containment (a **boundary condition**, so
  [DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)'s
  red-test obligation applies), scope entry / exit / restore, and the
  compiler's accept and reject cases.
- **Windows-only integration tests** — synthesized messages into
  `wnd_proc` with live widget state read back. Fixtures establish their
  own before-state rather than inheriting it (Phase 1 F-47), and the
  shared skip guard's two-conjunct status check is not relaxed (Phase 1,
  `0x80070005` collision).
- **GUI evidence with positive controls** — the four controls in the
  [framing](../requirements/framing.md) §検証方針, each with a leg where
  the two sides must **agree** (Phase 1 F-50). Capture is preceded by
  `cargo build --release --workspace` (Phase 1 F-21) and takes multiple
  frames on each side (F-33).
- **Touch** — synthesized injection only, with the limit stated: no
  touch hardware is available, agreed at framing (agreement ⑥).
