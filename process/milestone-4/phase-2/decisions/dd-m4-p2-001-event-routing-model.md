# DD-M4-P2-001 — Event routing model

**Status:** Accepted
**Phase:** M4-Phase 2
**AC:** AC1 ("focus model and event routing")

## Context

Today there is no routing: `wnd_proc` calls `hit_test_click`, which
walks the tree and fires **every** Button-family widget whose rectangle
contains the point, and `update_hover`, which walks the tree setting
background state. The keyboard reaches nothing: the `WM_KEYDOWN` arm
forwards the virtual key to a `key_down_fn` host callback slot that has
no installer, and returns without consulting the widget tree or the
default window procedure. That was adequate while no two interactive
widgets overlapped and nothing but a Button reacted.

This decision chooses how an input event finds a handler. It is the
milestone's foundational choice: the scrim's occlusion rule, Esc
consumption by a modal scope, per-item handlers, and every later
consumer (text field, IME, top layer, screen reader) stands on it.
[m4-interaction-intake.md](../../../../docs/notes/m4-interaction-intake.md)
case 6 is the open-question list this discharges.

**The spike removed one axis from this decision.** The traversal core
answers "which node is next" without knowing how an event arrived, so
**DD-001 and DD-003 are independently decidable**
([spike §4.3](exploration/focus-traversal-spike.md)). The routing model
therefore has to be chosen on the merits of propagation itself, not on
what the focus model needs.

## Sub-issues

- **Phases** — capture / target / bubble, or fewer.
- **Signal vocabulary** — which high-level signals exist, and whether a
  raw pointer-event surface is exposed at all.
- **Keyboard delivery** — where a key event starts, and what becomes of
  a key nothing consumes.
- **Drain coupling** — when the reactive drain runs relative to
  propagation.
- **Delivery to a removed subtree** — the guarantee M3-Phase 7 residual
  work asks about.
- **Pointer capture, hover enter/leave, pressed ownership.**
- **Touch** — which message family, and how it reaches the DIP space.

## Options

### R1 — Three phases (capture → target → bubble)

The DOM model. An event descends from the root to the target giving
each ancestor a chance to intercept, fires at the target, then ascends
giving each ancestor a chance to react.

### R2 — Target only, plus high-level signals

The event fires on the resolved target and nowhere else. Ancestors
never see it. Anything an ancestor wants to know about is expressed as
a signal the target raises deliberately.

### R3 — Target then bubble, no capture

The event fires at the target, then walks ancestors until one consumes
it. No descending phase.

### R4 — Asymmetric: pointer target-only, keyboard target-then-bubble

Pointer events stop at the widget hit-testing resolved; key events
start at the focused widget and bubble.

## Comparison

Judged on product merit first, with implementation and revision cost as
tie-breakers only (the M3-Phase 7 discipline, carried in
[framing.md](../requirements/framing.md) §再検討しない前提).

**What the milestone actually demands of propagation.** Four concrete
requirements, each traced to a document rather than assumed:

1. **A scrim blocks clicks to lower `ZStack` siblings**
   ([M4 framing](../../requirements/framing.md) §粒度の定義〔重ね表示〕
   design condition 2).
2. **Esc closes the innermost modal scope** while focus is on a Button
   inside it (AC1; intake case 5).
3. **Left / Right step between photos** while focus is on a lightbox
   button — an application-level meaning for a key the focused widget
   does not want ([spec.md](../../requirements/spec.md) §アプリ仕様 A).
4. **A row click is handled by the row**, including inside repetition
   (AC1's per-item handlers).

Requirement 1 is **not** a propagation requirement. It is settled by
DD-002's rule that hit-testing resolves **one** target, the topmost:
lower siblings are never candidates, so nothing needs to be blocked.
Reading it as a propagation problem is what makes a capture phase look
necessary; it is not.

Requirement 3 **is** a propagation requirement: a key the focused widget
has no meaning for must reach something above it. That is bubbling.
Without it, every lightbox button would have to carry its own arrow
handlers, which is precisely the per-consumer special-casing the
milestone thesis exists to prevent.

The requirement is only expressible because focus is *inside* the
lightbox when it opens: DD-004's entry moves focus to the scope's first
stop, which is what makes the scope's container an ancestor of the
focused widget and therefore reachable by the walk. A design in which
the scope opened without taking focus would leave an authored key
handler on it unreachable until the user pressed Tab.

Requirement 2 is **not** one, despite looking like it. DD-005 makes Esc
a source of a *dismissal request*, which DD-004 addresses directly to
the innermost entered scope rather than walking to it from the focused
widget, so requirement 2 is satisfied however propagation is designed.
It is listed here as considered-and-not-load-bearing, so a later reader
does not treat it as support for bubbling; requirement 3 alone is
decisive.

Requirement 4 is satisfied by target-only *if* hit-testing resolves the
row rather than a descendant. It is satisfied by bubbling in either
case.

**So the load-bearing question is whether pointer events also bubble**
(R3 versus R4), and whether a capture phase has any consumer at all
(R1).

- **R1's capture phase has no consumer in M4.** The one case that
  looks like interception — the scrim — is resolved by target
  selection. Building a phase whose only justification is that other
  frameworks have one is the over-design failure mode the project keeps
  as a standing hazard. It also doubles the surface an author must
  understand and the surface M5's widgets must respect, before a single
  requirement asks for it.
- **R2 costs requirements 2 and 3.** They would return as either
  per-widget duplication or a special "unhandled key" side channel —
  which is bubbling, spelled worse and reachable only for keys.
- **R4 is honest about what each family needs** and is the smallest
  thing that works. Its cost is a permanent asymmetry in the model that
  every later consumer must learn: two dispatch rules instead of one.
  It also forecloses cheaply-bought behaviour — a row wanting to react
  to a click on a button inside it has no path.
- **R3 gives one rule for every event family.** Its cost over R4 is
  that a click on a Button inside a clickable row reaches the row too,
  unless propagation is stopped. That is a real semantic the author must
  know, and it is the one thing R3 buys that could surprise.

The tie-break is uniformity: this model is consumed by five later
phases and by M5's whole widget set, and a single rule ("the event
starts at the target and walks up until something consumes it") is the
one an author and an implementer can both hold. The surprising case is
also the one every developer has already met on the web.

**Consumption, not stopping.** R3 needs a way to end propagation. The
recommendation makes **handling** the terminator: a handler that runs
consumes the event, and propagation ends there. There is no separate
`stop_propagation` verb in M4. This is weaker than the DOM (where a
handler can run and still let the event continue) and stronger than
nothing; it is chosen because no M4 requirement asks for
"handle and continue", and because the alternative adds a verb to the
DSL that DD-005 would have to spell.

**Drain coupling.** A handler's state write drains synchronously to
quiescence (M3-Phase 7). If that drain ran *between* propagation steps,
an ancestor could be re-materialised or removed while the event is
still walking toward it — the "delivery to a removed subtree" hazard,
arriving through the runtime's own machinery rather than through
authoring. The recommendation is therefore **one drain per dispatch**,
after propagation completes. This also bounds the answer to the
removed-subtree question: the ancestor chain is captured when the event
is dispatched, and a node removed during that dispatch is not visited.

**The M3-Phase 7 drain residuals**, whose firing this decision is
required to state
([constraints §3](../requirements/constraints.md)):

- **Cycle detection (residual 1) does not fire.** Its reopen signal is
  written as *an effect in a generated subtree writing state*. A handler
  is a user-initiated write, not an effect. The near case — a per-item
  handler that mutates the very collection its subtree rides — stays
  inside the existing machinery for the same reason the drain is placed
  after the walk: the handler has already returned when regeneration
  runs, so nothing re-invokes it and no cycle is created.

  **Qualified in part (2026-08-09, M4-Phase 2 T13;
  owner-approved 2026-08-08).** The conclusion and reopen signal above
  are unchanged, but the final causal sentence is too strong. T9's F5
  measurement shows that a collection write regenerates the `for`
  subtree inside `Signal::set`, during the handler statement rather than
  after the handler returns. No cycle is created because that synchronous
  regeneration updates the materialised subtree without re-invoking the
  dispatching handler. See
  [implementation log §T9](../implementation/log.md#t9--dsl-per-item-handlers-inside-for).
- **Effect-ordering ties (residual 2) do not fire**, because this phase
  introduces no observable ordering contract between effects.

Both remain residuals under their existing signals rather than being
closed here.

**What becomes of a key nothing consumes.** The walk can end with no
handler having run. That key **falls through to `DefWindowProc`**
rather than being swallowed, which is a change from today's arm: a
routing model should not claim keys it does not use, and the default
path is where system-level keyboard behaviour lives. Swallowing every
key would make the runtime's silence indistinguishable from its
handling, and would hand M4-Phase 5 / 6 a keyboard already consumed
before their surfaces exist.

**The host key callback slot stays unwired.** `WindowState`'s
`key_down_fn` has no installer, and this phase adds none: the walk is
the key path. Installing the first one fixes a host-facing surface
([constraints §2](../requirements/constraints.md)), and ABI work is
M4-Phase 7's (framing agreement 7).

**Touch.** M4's touch obligation is discharged by synthesized injection
with the limit stated ([framing.md](../requirements/framing.md)
agreement 6, owner-settled: no touch hardware is available). The
recommendation is to consume `WM_POINTER*` rather than rely on mouse
promotion, because promotion loses the distinction the model will need
in M5 and because the injection API drives the pointer family
directly. Coordinates cross the same DIP boundary as the mouse
(M4-Phase 1 DD-M4-P1-002), adding no new conversion site.

Handling the pointer message also **suppresses the mouse messages the
system would otherwise synthesize** for that contact, which is what
makes "the handler ran exactly once" a checkable property rather than a
hope — a touch that arrived by promotion and one that arrived through
the pointer path are otherwise indistinguishable from the handler's
side.

The converse switch is **not** taken: `EnableMouseInPointer` would route
*mouse* input onto the same family and unify the two paths, and it is
tempting for exactly the reason this milestone likes — one path instead
of two. It is refused because it is a **process-wide, one-way mode**,
and `wasamo.dll` runs inside a host process whose own windows would
inherit it. A library does not change its host's input mode. The mouse
therefore stays on the mouse messages, and the shared seam is the DIP
conversion rather than the message family.

**Pointer capture, hover, pressed.** Hover and pressed stay
Button-family **presentation state** and gain no authored surface; what
changes is who computes them. Today a whole-tree walk sets state on
every Button. Under this model they are enter / leave transitions
against the **resolved target**, so the widget that paints pressed is
the widget a release would dispatch to — under overlap the two can
otherwise disagree, which is the defect single-target resolution exists
to remove. Pointer capture (a drag that leaves the widget) is required
by *nothing* in this phase and is deferred with a trigger: the first
drag-based surface, which is M4-Phase 4's scrollbar.

**The synthesised pointer update after a scale change** (carried
forward from M4-Phase 1 T5) is resolved as **not adopted**: hover
correctness across a DPI change is cosmetic, self-correcting on the
next mouse move, and adopting it would add a second producer of hover
state — the exact shape DD-M4-P1-002 §Row 6 closed.

## Recommendation

**R3 — target then bubble, with no capture phase**, plus:

- **High-level signals only.** The authored vocabulary is `clicked`,
  `dismiss` and `key-down` (DD-005). Hover and pressed are Button-family
  presentation state, **not** authored signals — nothing raises them to
  an author. No raw pointer-event surface in M4 — permitted by the
  intake classification, and withheld because nothing in A or B4 needs
  it and it would be ABI-visible surface before M4-Phase 7.
- **Keyboard starts at the focused widget**, or at the innermost entered
  modal scope when nothing is focused, and bubbles. **An unconsumed key
  falls through to `DefWindowProc`**, and the uninstalled host key
  callback stays uninstalled.
- **Handling consumes.** No separate stop verb.
- **One drain per dispatch**, after propagation completes; the
  ancestor chain is captured at dispatch. The M3-Phase 7 drain residuals
  do not fire and keep their existing signals.
- **Touch via `WM_POINTER*`**, converted at the existing DIP boundary,
  with the promotion it suppresses as the single-delivery check;
  `EnableMouseInPointer` is refused as a host-wide mode change.
- **Hover and pressed are computed against the resolved target**, not
  by a whole-tree walk.
- **Pointer capture deferred** to M4-Phase 4.

## Forward-compat exposure

- **A capture phase remains addable.** Adding a descending phase later
  is additive for every existing handler, because no handler observes
  the absence of one. What is *not* cheap later is removing bubbling,
  so the risk is carried on the side that can be undone.
- **"Handle and continue" remains addable** as an explicit verb; the
  M4 rule is the conservative default and nothing depends on
  propagation ending.
- **Raw pointer events remain addable** and would arrive as a new
  signal family, not as a reinterpretation of `clicked`.
- **The ABI is untouched.** Signals continue through the existing
  registry; no new C entry point (framing agreement 7).
- **M4-Phase 6 inherits an unresolved precedence**: when a text field
  has focus, which keys it consumes before bubbling. Fixed there by
  plan, not pre-empted here; the bubbling rule is what makes it
  expressible.

## Technical risk re-evaluation

- **The consumption rule is the sharpest edge.** "The first handler
  that runs ends the walk" means adding a handler to a widget silently
  removes an ancestor's. Mitigation: DD-005's spelling makes the
  handler's location visible at the authoring site, and the phase's
  evidence includes a control where a nested handler is added and the
  ancestor is shown to stop firing.
- **One drain per dispatch changes an existing timing.** Today the
  drain runs inside `hit_test_click`'s handler invocation. Moving it to
  the end of propagation changes when re-layout happens relative to a
  handler's return. Nothing in M3 observes that ordering, but the
  assumption is recorded so a later contract can be checked against it.
- **`WM_POINTER*` is a wider surface than the three mouse messages.**
  Mitigation: only the subset the fixture and the two apps exercise is
  handled, and unhandled members fall through to `DefWindowProc` — which
  is also what re-enables promotion for the contacts this phase does not
  claim, so an unhandled pointer message degrades to today's behaviour
  rather than to silence.
- **The `DefWindowProc` fallthrough is a behaviour change on a path with
  no current consumer.** No key reaches the default procedure today, so
  nothing can regress visibly; the risk is the opposite direction — a
  later phase assuming keys are swallowed. Recorded so M4-Phase 6 reads
  it as a property rather than discovering it.

## Revision history

- 2026-08-06: Initial Accepted record.
- 2026-08-09: Added the owner-approved dated qualification to the
  residual-1 explanation. T9 F5 falsified only the claim that
  regeneration waits for handler return; the decision, implementation,
  and no-cycle conclusion remain unchanged.
