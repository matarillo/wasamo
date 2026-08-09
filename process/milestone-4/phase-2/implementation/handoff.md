---
title: M4-Phase 2 handoff
status: closing
source-phase: M4-Phase 2
date: 2026-08-09
---

# M4-Phase 2 — Handoff

> **Status: closing.** This document was written after the CF-* phase-end
> classification in [phase-end.md](../retrospectives/phase-end.md). T13's
> task-branch CI is green, but the owner-selected no-ff integration means
> the merged `feat/m4-phase-2` branch still owes its own phase-end CI run.
> It carries constraints and open questions, not unearned later-phase
> acceptance claims.

## Main learnings

The phase landed one input model rather than three device-specific ones:
mouse and touch differ at the Win32 boundary, but both reach one
layout-derived target in DIP; keyboard starts from the one per-window
focus record; every handled event walks the same captured target-to-root
chain. Later input consumers must join those seams instead of installing
a second target resolver, focus record or scope concept.

A modal focus scope is structural state: an annotated subtree's presence
enters it, entry captures restoration and moves focus in, removal exits
it, and traversal is confined by changing its root. Pointer confinement
is separate and comes from ordinary one-target occlusion by an authored
covering widget. Phase 9 and Phase 11 are consumers of this same concept,
not permission to create top-layer or accessibility-specific modality.

The evidence also established a reusable limit. Pure traversal tests,
mock-free runtime integration and rendered positive controls cover
different failure classes. A frame must exclude a plausible look-alike:
T12's group claim is carried by the second Tab landing on Scroll down,
not by the first frame merely showing focus on All. A comparison must
also validate the band that judges it; whether to turn that lesson into
a standing obligation remains deliberately open (CF-T12-5).

An allocator-dependent test can be a useful residual sensor without being
a stable regression gate. When T13's sensor finally observed address reuse,
the implementation defect was repaired and a deterministic forced-state
fixture was added while the natural allocator observer remained. Do not
weaken a conditioned assertion merely because most runs do not reach its
condition.

## Immediate intake for M4-Phase 3

| Source | Input to Phase 3 framing | Re-trigger / completion |
|---|---|---|
| **CF-T9-2** | The public IR spec already makes string handler expressions binding-only, but checker, lowering and loader accept string assignment until evaluator invocation rejects it. Phase 3 owns the **diagnostic intake**, not the capability (Phase 5). Preserve this as an unenforced normative statement until the assigned owner acts; do not weaken §8.9 to match missing validation. | Pre-doc must decide the diagnostic shape and whether it is in Phase 3 scope. Capability remains M4-Phase 5 per milestone-plan revision 1. |
| **CF-T10-3** | `selected_index` can become −1 or N. The owner expects Phase 3 to deliver both: out-of-range indexed reads fail with a runtime diagnostic, and a handler can guard its write. The second half is not derivable from the current Phase 3 scope text, so framing must decide whether it fits or requires a gated plan revision. | The phase that lands collection index reads cannot close without deciding the out-of-range result; the handler-predicate half must be answered explicitly even if rejected. T9's `EvalError::ItemOutOfRange` is precedent, not a mandated implementation. |
| **CF-T12-5 — open question, not policy** | At Phase 3 pre-doc, decide whether any task should be obliged to demonstrate that positive-control comparisons can fail. **No intent to adopt is carried. “No rule” is an equally valid answer.** The placement is not the task plan. | If the answer is a rule, it is structural: write a successor to DD-V-029 and make it narrower than the already-rejected “falsify every green/identical observation” proposal. If the answer is no rule, close the question explicitly. |

## Named later-phase inputs

| Source | Destination and constraint | Re-trigger |
|---|---|---|
| **Accepted-DD seeded consequence** | A disabled Button remains the one resolved target, suppresses its own dispatch, and therefore lets propagation continue to a clickable ancestor while still occluding lower siblings. This follows from the accepted routing and hit rules and is normative in §4.8 / §4.19; whether the official widget set wants the consequence is not decided without a consumer. | M5's widget set, or M4-Phase 9 if a dialog places a disabled control inside a clickable container. Swallowing at the disabled control would change the consume-on-handle rule and requires a successor decision, not a local exception. |
| **CF-T10-1** | M4-Phase 4 owns `Row` / `HStack` overflow semantics. Today a non-clipping overflowing toolbar container remains a hit candidate across its arranged rectangle and swallows clicks aimed at overlapping tabs; `dsl_spec.md` §4.19 now states that input consequence without choosing layout policy. | G7 going red, or any narrow-client toolbar capture / fixture. |
| **CF-T11-2, CF-T11-6** | M4-Phase 4 is the first drag consumer: pointer capture, drag and gesture are unbuilt; interactive injection is intentionally outside CI today. | First moving-contact surface, or a CI runner known to support injection. Adding the tier requires a capability probe and an owner-authorized push. |
| **CF-T11-3** | Multi-contact has only a conservative policy: a non-primary contact is claimed and inert. | First pinch, two-finger scroll or other multi-contact meaning, no earlier than Phase 4. |
| **CF-T8-1** | Decide Button keyboard activation with M5's whole keyboard-operable widget family. The runtime does not currently turn Space / Enter into `clicked`, and public text no longer says it does. | M5 widget-set keyboard contract. Decide keys-kept interaction with authored `key-down("Enter")`, not only Button dispatch. |
| **CF-T9-2 capability half** | M4-Phase 5 owns handler assignment to scalar string state. | Text-state write surface design and its typed evaluator/writer path. |
| **CF-T8-6** | The authored `key-down` host-listener instantiation is untested although the shared host-listener branch is covered through `clicked`. | M4-Phase 7 ABI work or the first host connecting a key signal. |
| **CF-T7-3, CF-T9-5** | M4-Phase 9 must exercise nested scopes and a per-item modal scope through real runtime integration. Pure logic covers nested projection today; no M4 app composes scopes or repeats them. | First dialog-from-menu / per-item overlay. Use the existing presence-entry and restoration seam unchanged or supersede DD-M4-P2-004. |
| **CF-T12-1** | A semi-transparent cover attenuates visible changes behind it; a no-change claim needs a sensor leg and a metric able to see through the cover. | M4-Phase 9 top-layer evidence, and any earlier overlay/background no-reaction claim. Derive attenuation from the authored alpha rather than guessing. |
| **CF-T6-2** | A container carrying both `focus-group` and `modal-scope` currently warns and behaves as a scope because the runtime role is one-of. The candidate pool owns whether the combination should exist or whether the public surface should become exclusive. | First app needing a group that is also a scope, or M6 public-surface freeze. |

## Cross-phase implementation constraints

| Source | Constraint that remains in force | Re-trigger |
|---|---|---|
| **CF-4** | Entering routing on a subtree supplies neither ancestors above that root nor their clip bounds. Production window routing enters at the window root; a narrower entry is a deliberately bounded model. | Any production resolver called on other than the window root. |
| **CF-5** | A native `set_clicked` closure that destroys its own node frees the closure while it is executing. Current inline and host producers avoid that shape by construction. | First native closure with structural side effects. |
| **CF-6** | Registry lookup uses raw pointer identity; an address could be freed, reused and re-queried before enqueue. | Any handler whose synchronous drain allocates a widget before the registry re-query. |
| **CF-T4-1** | Hover retains an index path; a structural shift can make a still-in-range path name a different node. Bounds checks prevent unsafety but not stale presentation. | Button-family node under a mutating `for` or any sibling reorder while the pointer is over it. |
| **CF-T4-2** | `set_button_state_at`'s equal-state guard is not independently pinned; its unique avoided effect is redundant mid-animation brush work. | Any change to Button state transition or brush animation. |
| **CF-T4-3** | Replacing a live window root resets hover but no production example or test replaces a root after pointer input. | First live-root replacement API consumer. |
| **CF-T6-4** | Checker and loader intentionally duplicate the seven focus-annotation container names across crates with no mechanical tie. | Any new container kind, first candidate M4-Phase 4 `Image` only if it is a container; otherwise M5's widget set. |
| **CF-T7-1 / CF-T9-1 — presentation repaired at T13a; identity remainder carried** | Focus anchors remain node addresses, so allocator reuse can still make a fresh same-address node the retained focus target. T13a closes the additional divergence the cold suite exposed: after structural restoration / succession / entry, the existing single focus writer reconciles that final target's presentation. A deterministic forced-state fixture is red without the repair; full independent review and the replacement cold suite are green. | First requirement for stable logical identity across regeneration, or a user-visible wrong-target report. Do not remove the natural allocator observer or the deterministic presentation fixture while pointer anchors remain. A generation token or different identity policy requires a new decision; T13a did not select one. |
| **CF-T7-2** | Four direct-ABI child mutators do not run the focus projection/rebase seam. Exposure is bounded because production does not read `focused_path` and ABI-created nodes cannot carry focus annotations. | First production `focused_path` reader or ABI surface for focus annotation. |
| **CF-T8-4** | Unknown bare signal names are accepted and may never fire; the public spec deliberately leaves their diagnostic semantics unspecified instead of turning T13 into a new reject decision. | Fourth defined signal or first bug report caused by a silently misspelled handler. |
| **CF-T9-4** | Invocation-time binder resolution has one discriminating integration test; no other test reddens if attachment-time snapshots replace it. | Any change to handler attachment, loop-scope snapshots or `ForItemHandlerEvalContext`; do not delete/narrow that test without replacement. |

## Doc-folded operational constraints

These remain useful but do not need a second normative restatement here:

- **CF-T5-4** — real keyboard capture must earn foreground activation,
  read it back and retry; see
  `docs/notes/verification-environments.md` Observation 4.
- **CF-T8-5** — the key walk is upward-only; gallery scope handlers sit
  above the focused lightbox control. The implementation comments and
  `dsl_spec.md` §4.19 own the rule.
- **CF-T10-4 / CF-T10-5** — gallery integration fixtures read the shipped
  `.ui` by labels/text, and `__resolve_topmost_for_test` accepts DIP. Their
  call-site comments are the operative warnings.
- **CF-T11-5** — GUI scripts poll to a deadline and report owned windows;
  a single fixed sleep is not readiness evidence.

## Verification residue

- **CF-T11-1:** synthesized touch proves the Win32 pointer path used by
  the fixture, not equivalence to physical digitizer delivery.
- **CF-T11-4:** `pointer_physical` still has callers in client and screen
  coordinate spaces with no type-level distinction. A third caller or an
  actual unit defect reopens the Phase 1 `Dip<T>` / `Px<T>` reserve; the
  reserve has not fired yet.
- **CF-T12-2:** a comparison script can be false-green because the band
  is derived from the measured quantity even when each comparison arm is
  self-checked. T12's reusable mechanism registers every verdict,
  mechanically rejects coverage gaps and uses chosen independent bands.
  This is a hazard and example to copy, not a standing process rule.

## Closed-index pointer

The exhaustive phase-end table in
[phase-end.md §15](../retrospectives/phase-end.md#15-t1t12-carry-forward-classification)
also records every CF identifier closed or doc-folded in this phase. Those
entries are intentionally absent from the forward-work tables above so a
later phase does not mistake closed work for an obligation.

## Merge and CI residue

The repaired local clean rebuild, complete suite, consumer checks and full
independent review are recorded in
[log.md §T13a end gate](./log.md#t13a-end-gate--final-local-verification-2026-08-09).
Owner-authorized T13 task-branch GitHub Actions
[run 31298945418](https://github.com/matarillo/wasamo/actions/runs/31298945418)
passed on `feat/m4-phase-2-t13` HEAD
`11f77b689bc234453d2e9ff2f6a1a540c879320a`, so T13 is complete. It is
direct evidence for the unchanged code tree, but it is not the branch named
by phase procedure item 16. The no-ff merge produced local phase HEAD
`b23e27e`; `feat/m4-phase-2` still requires an owner-authorized push and
green `workflow_dispatch` before this handoff returns to `recorded`.
Phase→`main` merge remains another explicit owner gate.
