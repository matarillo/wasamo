# DD-M4-P2-003 — Focus model: location, eligibility, and traversal

**Status:** Proposed
**Phase:** M4-Phase 2
**AC:** AC1 ("focus model"), and the M5 semantics the milestone
requires be settled once

## Context

Nothing in Wasamo can be focused today. This decision creates the
concept, and it is the one the milestone thesis names: input, text
editing, IME, multi-window and accessibility are *"five consumers of one
model"*, and the model is settled here or it is settled five times.

The framing adds an obligation beyond M4's own needs: the focus
semantics **M5's official widget set** will require must be fixed now,
without building the widgets ([M4 framing](../../requirements/framing.md)
§粒度の定義〔フォーカスモデル〕). Those are (a) arrow-key movement
within a group that Tab treats as one stop, and (b) separation of focus
from the active item in an open list. If they are not settled here, the
thesis breaks in M5 — and it breaks at the point where it is most
expensive to fix, with a widget catalogue already built on the wrong
shape.

Both were required to be demonstrated on running code before this ADR
could be Accepted. They were:
[exploration/focus-traversal-spike.md](exploration/focus-traversal-spike.md)
answers Q1–Q5 and Q7, and its findings are cited below rather than
restated.

## Sub-issues

- **Where the focus state lives**, and whether that survives
  M4-Phase 8's per-window model.
- **What is focusable.**
- **Traversal order** — tree order or visual order.
- **Group traversal** and the roving memory it needs.
- **Focus / active-item separation.**
- **Click-to-focus.**
- **The focus indicator**, and its interaction with the single-pass
  Visual write.
- **What happens when the focused node disappears.**
- **Relation to Win32 window activation.**

## Options

### Where the state lives

- **L1 — one process-global focus.** Simplest today; there is one
  window.
- **L2 — one `FocusState` per `WindowState`.**

### Eligibility

- **F1 — derived from widget kind.** Button-family is focusable,
  nothing else.
- **F2 — explicit annotation only.** Every focusable widget says so.
- **F3 — kind-derived default, extensible by annotation.**

### Traversal order

- **O1 — tree order** (the order the `.ui` declares).
- **O2 — visual order** (derived from arranged geometry).

## Comparison

### Location: L2

L1 is cheaper by exactly one field and is wrong by construction: AC2
puts a second window in M4-Phase 8, and cross-window focus is that
phase's problem only if focus is per-window to begin with. M4-Phase 1
made the identical choice for the DPI scale and recorded the reason —
*"any second source of a window's scale reintroduces the drift"* — and
the same argument applies unchanged. L2 costs nothing now and makes
Phase 8 additive.

The spike's state object is already shaped for this: one `FocusState`
holds the focused id and three derived stores, with no static and no
global ([spike §4.1 Q1](exploration/focus-traversal-spike.md)).

### Eligibility: F3

- **F1 is what the spike measured today's tree can express** — Button
  family and nothing else. It is adequate for A's lightbox and toolbar
  and inadequate the moment a non-Button widget must take focus, which
  is M4-Phase 5's text field, one phase later.
- **F2 is verbose for the common case** and would make every Button in
  every example carry an attribute to keep behaviour it already has.
- **F3** keeps Button-family focusable by default and lets a widget
  opt in. The opt-in spelling is DD-005's; what this decision fixes is
  that the default is *derived*, so a phase that adds a focusable widget
  kind changes the derivation rather than every author's file.

The spike's `spike_focus_role` is deliberately exhaustive over
`WidgetData` with no `_` arm, so a later widget kind cannot become
silently non-focusable. That property is adopted as a requirement, not
just as spike scaffolding.

### Traversal order: O1

O2 is what a user perceives — Tab should go left to right, top to
bottom, regardless of declaration order. It is also unavailable at the
moment traversal runs without further decisions: a visual order needs a
reading direction, a rule for overlapping rectangles (`ZStack`), and a
rule for a `WrapPanel`'s rows, none of which this phase opens.

O1 is chosen with its limit stated rather than as an assumption: **in
every layout Wasamo has, tree order and visual order coincide**, because
every container lays its children out in declaration order. `ZStack` is
the one place they can diverge, and there the tree order (bottom to
top) is arguably the right reading order anyway. The spike used tree
order throughout and no case pushed back
([spike §4.3](exploration/focus-traversal-spike.md)).

The re-trigger is recorded: a layout primitive whose arranged order
differs from its declaration order — an author-controlled `order`
attribute, or right-to-left text — reopens this.

### Group traversal and the roving memory

The spike answered this as one mechanism, not two: a single `Group`
annotation makes the container contribute **itself, once** to the Tab
stop list and not be descended into, and makes arrows move within its
members. No second concept was needed
([spike §4.2 S-1](exploration/focus-traversal-spike.md)).

It also answered a question the framing left open: **the memory is
required and it belongs inside the model.** Without it, leaving a group
and returning lands on the first member rather than the one the user
left — measured, and pinned by a test that goes red when the memory
write is removed (spike mutation M3).

The memory is data parallel to the focus pointer, which is the
implementation-gates trap-3 shape. The decision therefore includes its
discipline: **the focus pointer and the group memory are written by one
primitive**, and the pointer is not independently writable. In the spike
this is enforced by visibility — `focused` is private and `set_focus`
writes both. That enforcement is adopted, not just the behaviour.

### Focus / active-item separation

The M5 dropdown semantics require a pointer that is **not** focus:
focus stays on the owner while the active item moves. The spike
implemented it as a per-list entry in a separate store, and measured
that focus does not move (mutation M6 makes the assertion red).

Two questions it settles, both stated because they are easy to get
backwards:

- **The active item belongs to the widget, not to the scope.** Nothing
  in the spike pushed toward scope ownership, and scope ownership would
  make two open lists in one scope impossible.
- **Active items are not Tab stops.** Reaching them is what the active
  pointer is for.

This is fixed as a rule and **not implemented as a widget** in M4: no
list widget exists to own one. What ships is the model's capacity for
it, demonstrated by the spike, so M5 does not have to change the model.

### Click-to-focus

A click moves focus to the resolved target when that target is
focusable, and leaves focus alone when it is not. The second half
matters: clicking the background must not clear focus, or every
click-away in B4 would strand the keyboard.

### The focus indicator

Focus must be visible. The constraint that shapes it is inherited, not
chosen: **every Composition geometry write in the runtime happens in
`sync_visuals`** (M4-Phase 1 T3), and that property is what makes
DD-M4-P1-002's conversion audit complete rather than approximate. A
focus ring drawn by a new `SetOffset` / `SetSize` outside that pass
breaks it silently.

The decision is therefore that the indicator is **presentation state on
the node, applied by the existing sync pass** — the same shape as
Button hover / pressed — and not a new visual written at focus-change
time. The concrete appearance is implementation, not ADR content.

### The focused node disappearing

Conditional rendering removes subtrees while they hold focus; A's
lightbox does exactly this. The spike measured two things here
([§4.1 Q5](exploration/focus-traversal-spike.md)):

- **The successor must be computed before the removal**, because ids do
  not survive a rebuild — the conditional and `for` paths materialise
  fresh subtrees, so an id stored across the mutation can name a
  *different* node afterwards. This is not a theoretical hazard: the
  spike's own test failed to exercise its membership check for exactly
  this reason and the mutation battery caught it (S-2).
- **When the disappearing subtree is an entered modal scope,
  restoration wins over structural succession** — that belongs to
  DD-004 and is cross-referenced rather than duplicated.

### Win32 activation

Focus in this model is *within* a window; Win32's `WM_SETFOCUS` /
`WM_KILLFOCUS` are *between* windows. They are kept separate: losing
window activation does not clear the internal focus, so re-activating
restores what the user had. Cross-window focus is M4-Phase 8's, and this
separation is what leaves it additive.

## Recommendation

- **L2** — one `FocusState` per `WindowState`; no global, no static.
- **F3** — Button-family focusable by default via a derivation that is
  exhaustive over widget kinds; extensible by annotation (DD-005).
- **O1** — tree order, with the divergence re-trigger recorded.
- **Group** — one annotation gives both "one Tab stop" and "arrows
  move inside"; the roving memory is required and is written by the
  same primitive that writes focus, which is not independently
  writable.
- **Active item** — a separate per-widget pointer; active items are
  not Tab stops; the capacity ships, no widget does.
- **Click-to-focus** on focusable targets only; a click elsewhere does
  not clear focus.
- **The indicator is presentation state applied by `sync_visuals`.**
- **Successor-on-removal is computed before the mutation.**
- **Win32 activation is separate** from internal focus.
- **The spike's traversal core is adopted as the implementation**, with
  its two recorded limits carried as documented rather than closed: a
  focusable widget is a traversal leaf (a focusable widget inside a
  focusable widget is not expressible), and traversal is tree order.

## Forward-compat exposure

- **M4-Phase 5** adds the first non-Button focus stop; F3 is what makes
  that an annotation rather than a model change.
- **M4-Phase 8** consumes L2 additively; the check at that point is
  that no second source of focus crept in.
- **M4-Phase 11** reads focus and scope to build the accessibility
  tree; the `ActiveItem` concept is what `aria-activedescendant`-shaped
  reporting will need, and it exists now.
- **M5's widget set** consumes group traversal and focus / active-item
  separation. The `.ui` spelling of a group is **M5's decision**; what
  M4 fixes is the semantics, and DD-005 adds only the minimum
  annotation the fixture and A need.
- **Focusable containers** (a focusable widget with focusable children)
  are the recorded limit. M5's list-item-with-a-button is the case that
  fires it.

## Technical risk re-evaluation

- **The parallel-memory hazard is real and pinned.** It is carried by
  visibility rather than by discipline, which is the stronger of the
  two; the risk is that a later task widens `focused` for convenience.
  Recorded as a carry-forward with that exact re-trigger.
- **Tree order will eventually be wrong.** It is right for every layout
  that exists and its re-trigger is written down, which is the most that
  can be bought without opening layout.
- **The indicator is the likeliest place to break the single-pass
  invariant**, because drawing at focus-change time is the obvious
  implementation. Mitigation: the close artifact for any task touching
  it is the same enumeration DD-M4-P1-002 used — every `SetOffset` /
  `SetSize` in the runtime, with its pass.
- **The spike's core is not yet load-bearing.** It has no production
  caller, so adopting it is a plan, not a measurement. What *is*
  measured is that its semantics survive contact with a real tree built
  through the production `.ui` path
  ([spike §4.1 Q6](exploration/focus-traversal-spike.md)).
