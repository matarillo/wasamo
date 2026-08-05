# M4-Phase 2 — Event routing, focus model, and generic click handling: Architecture Decisions

**Phase:** M4-Phase 2 (event routing, focus model, and generic click
handling — **the milestone's centre of gravity**)
**Date:** 2026-08-05
**Status:** Accepted (owner approval 2026-08-05; DD-001 … DD-005 all
Accepted)

## Context

M4 acceptance criterion **AC1** (see
[../../../_roadmap.md M4](../../../_roadmap.md#m4-interaction-stack),
[../../plan.md §Acceptance criteria](../../plan.md)):

> **AC1** — Input handling: keyboard, mouse, touch; focus model and
> event routing. Includes click handling on non-`Button` widgets (with
> per-item handlers inside repetition) and a **structure-independent
> modal focus scope** — attachable to any subtree, so a root `ZStack`
> branch and a top-layer overlay are both consumers of one concept.

The milestone thesis is that input, single-line text editing, IME,
multi-window and accessibility are not five features but **five
consumers of one focus model**, and that the model is settled once
([plan.md](../../plan.md) §Purpose). This phase is where that settling
happens. It is deliberately second, while the interaction surface is
still small enough to reason about, rather than after five widget
surfaces have accumulated implicit expectations.

Two consequences follow, and they shape the whole set:

- **This ADR fixes rules it does not implement.** The modal focus
  scope has three consumers — the `ZStack` branch (here), the top layer
  (M4-Phase 9), and screen-reader modality (M4-Phase 11) — and the
  disposition agreed at milestone planning is *one design, three
  implementations* ([plan.md](../../plan.md) §Cross-phase dispositions
  1). DD-004 therefore states binding rules for Phase 9 and Phase 11
  without building either. If a later phase cannot be built on the
  concept unchanged, the correct response is a supersede of DD-004, not
  a local special case (milestone-end criterion 3).
- **The design was tested against running code before it was written
  down.** The framing required a pre-ADR spike
  ([../requirements/framing.md](../requirements/framing.md) agreement
  5); its report is
  [exploration/focus-traversal-spike.md](exploration/focus-traversal-spike.md),
  and its measurements are cited by number below rather than restated.

### The starting state (verified against the workspace at drafting time)

- **There is no event model.** `wnd_proc` maps three mouse messages
  onto two entry points:
  [`widget.rs`](../../../../wasamo-runtime/src/widget.rs)
  `hit_test_click` (fire the first Button-family widget whose rect
  contains the point) and `update_hover` (walk the tree setting hover /
  pressed background state). There is no dispatch table, no
  propagation, no notion of a target, and **no keyboard path at all** —
  `WM_KEYDOWN` is not handled.
- **There is no focus.** No widget can be focused, nothing is
  focusable, Tab does nothing. `ButtonData.enabled` exists and
  suppresses click dispatch (DD-M3-P1-005), which is the only
  interaction-state concept in the runtime.
- **Hit geometry is read back off the live Visual.** `visual_rect_dip`
  reads `Visual.Offset` / `Visual.Size` and divides by the traversal
  root's scale. Layout computes each node's rectangle and does not
  retain it.
- **The two conversions on that path cancel.** The pointer is divided
  by the window's scale in `wnd_proc` and the readback is divided
  again; because hit-testing sources its geometry from the visual tree,
  the two are symmetric and no test can distinguish a correct
  conversion from a missing one
  ([M4-Phase 1 handoff](../../phase-1/implementation/handoff.md), T5).
  **This phase is where that stops being true.**
- **Occlusion does not exist.** `hit_test_click_inner` recurses into
  every child and fires *every* matching Button-family widget it finds
  on the way — there is no "topmost wins" rule, because in M3 no two
  interactive widgets ever overlapped.
- **The reactive drain is synchronous and non-batched** (M3-Phase 7),
  and a handler's state write runs to quiescence before the handler
  returns. `emit::flush_layout`'s layout phase re-lays out the window
  root and refreshes text at the window's scale (corrected at
  M4-Phase 1 T3, finding F-23).
- **Six `WindowState` callback slots exist and none has an installer.**
  Four carry coordinates and all four are DIP `f32`
  (M4-Phase 1 DD-M4-P1-004). The first host- or ABI-facing function to
  install one fixes that unit as shipped API.

### What the phase inherits as settled

Not re-litigated here
([../requirements/constraints.md](../requirements/constraints.md)):
the coordinate space is DIP with conversion confined to the boundary
(M4-Phase 1 DD-M4-P1-002); every Composition geometry write happens in
`sync_visuals` alone; the per-node geometry-scale cache has exactly one
writer; `ButtonData.label_size` has exactly two writers, both of which
also write `label_text` and the node's `SizeConstraint::Fixed` pair.
Each is an invariant a careless event model can break silently.

## Summary of decisions

| DD | Question | Recommendation |
|---|---|---|
| [DD-001](dd-m4-p2-001-event-routing-model.md) | How does an input event reach a handler? | **Target then bubble, no capture phase**, with high-level signals as the authored surface and one drain per dispatch (option R3) |
| [DD-002](dd-m4-p2-002-hit-testing-and-generic-click.md) | How is a pointer target resolved, and which widgets are eligible? | **Layout-derived DIP rectangles** (option H2), **topmost-single-target** selection — from which scrim occlusion follows rather than being a rule — and `clicked` generalised from Button to any widget |
| [DD-003](dd-m4-p2-003-focus-model-and-traversal.md) | Where does focus live and how does it move? | One `FocusState` per `WindowState`; focusable = **Button-family by default, extensible by annotation**; **tree-order** traversal; the spike's core adopted as the traversal implementation |
| [DD-004](dd-m4-p2-004-modal-focus-scope.md) | What is a structure-independent modal focus scope? | An **annotated subtree plus an explicit enter/exit**, because the restore target cannot be derived from the tree (measured). Esc is consumed by the innermost scope; **screen-reader modality attaches to the scope, not the layer** |
| [DD-005](dd-m4-p2-005-dsl-handler-surface.md) | How is all of this spelled in `.ui`? | `clicked` on any widget, `item` / `index` readable in handler position, and **exactly two new annotations** — a focus group and a modal scope — which is the gap the spike measured |

## Out of scope

Sent onward with a trigger, per
[../requirements/framing.md](../requirements/framing.md) §含まないもの:
the top-layer implementation of the modal scope (M4-Phase 9); the
screen-reader implementation of scope modality (M4-Phase 11); the text
field's focus stop and the text-input / shortcut precedence
(M4-Phase 5 / 6); cross-window focus (M4-Phase 8); wheel and drag
scrolling (M4-Phase 4); `Row` overflow semantics (M4-Phase 4, per
framing agreement 4); the visible completion of Left/Right stepping,
which needs predicate expressions (M4-Phase 3); any new ABI function
(M4-Phase 7); the official group spelling and RadioButton / ComboBox
(M5); and a general declarative shortcut surface, which AC1 does not
require.

## Revisions

*(None yet. Revisions after acceptance follow the supersede rule, and a
qualification that does not change what a reader would implement is
recorded as a dated annotation instead —
[workflow.md §凍結文書](../../../procedures/workflow.md).)*

Per framing agreement 5, a spike may also run **during** the phase, and
a finding from it may reopen an Accepted decision here through the same
two mechanisms.
