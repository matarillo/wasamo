# DD-M4-P2-005 — The authored surface: handlers, item references, and focus annotations

**Status:** Accepted
**Phase:** M4-Phase 2
**AC:** AC1 (generic click handling, per-item handlers, modal focus
scope), and phase-end criterion 4 (spec synchronization)

## Context

This is the phase's only normative DSL surface, and the only decision in
the set an author sees. It has to add exactly what DD-001 … DD-004
require and nothing else — every spelling added here is spelling M5's
widget set inherits and M6 freezes around.

The spike measured the size of the gap rather than leaving it to
estimate ([spike §4.1 Q6](exploration/focus-traversal-spike.md)):
today's `.ui` can express **two of the six focus roles**. `Stop` follows
from the widget kind (Button family) and `Container` from everything
else; `Group`, `ModalScope`, `ActiveItemList` and `ActiveItem` have no
representation at all. Of those four, DD-003 puts the active-item pair
in M5 (no widget in M4 owns a list), so **M4 needs exactly two new
annotations**.

The per-item half is a reopening, not a new surface: M3 rejected
`item` / `index` in handler position and recorded that it would land at
*"M4's per-item interaction"* ([M3 handoff](../../../milestone-3/handoff.md)).
A's thumbnail click is that case.

## Sub-issues

- **Generic `clicked`** — the spelling on a non-Button widget.
- **`item` / `index` in handler position** — spelling and checker rule.
- **Key handlers** — whether Esc / arrows are authored at all.
- **The focus group annotation.**
- **The modal scope annotation.**
- **Focusability opt-in** (DD-003's F3 extension point).
- **IR and compiler impact**; what moves in `docs/dsl_spec.md`.
- **Keeping the fixture's spelling out of the normative surface.**

## Options

### Generic `clicked`

- **G1 — the same `clicked` signal on any widget**, identical spelling
  to Button's.
- **G2 — a distinct name** (`pointer_click`, `tapped`) for non-Button
  widgets, leaving `clicked` Button-specific.

### `item` / `index` in handler position

- **I1 — bare `item` / `index`**, the same identifiers the binding
  position already uses inside `for`.
- **I2 — a qualified form** (`for.item`, `loop.index`).

### Focus annotations

- **A1 — attributes on the existing container** (`focus-group: true`,
  `modal-scope: true`).
- **A2 — wrapper widgets** (`FocusGroup { … }`, `ModalScope { … }`).
- **A3 — a single attribute with an enumerated value**
  (`focus-role: group` / `focus-role: modal-scope`).

### Key handlers

- **K1 — no authored key surface in M4.** Esc closing and Left/Right
  stepping are built-in behaviours of the scope and the app's state.
- **K2 — key handlers as signals** (`escape => { … }`,
  `key-left => { … }`) on any widget, delivered by DD-001's bubble.
- **K3 — a general declarative shortcut surface** (key combinations
  bound to actions, at window or scope level).

## Comparison

### Generic `clicked`: G1

G2's only argument is that a Button's click carries semantics a `Box`'s
does not — a Button is activated by Space and Enter as well as by the
pointer, and a `Box` is not. That is real, and it is a difference in
*what raises the signal*, not in what the signal means. Two names would
force every consumer — M5's widget set, the accessibility layer, an
author reading someone else's file — to learn which spelling a given
widget uses, to express a distinction that is already visible from the
widget's kind.

G1 also keeps DD-002's decision legible: one signal, raised by whatever
the hit test resolved. The Button's extra activation paths are Button
behaviour, documented on Button.

### `item` / `index`: I1

M3 already binds `item` and `index` inside `for` in *binding* position,
and the checker already knows the loop scope there. I2 would introduce a
second spelling for the same values, distinguished only by which side of
a `=>` they appear on — a distinction with no meaning to an author.

M3's rejection was about the **handler evaluator** not having the loop
scope available, not about the spelling. What this decision adds is the
scope's availability at handler-invocation time; the identifiers are the
ones already in the language.

The checker rule follows the same shape as the binding position: `item`
and `index` resolve only inside a `for` body, and a reference outside
one is a diagnostic. **Both directions need a test** — the accept case
and the reject case — per the shared-lexer discipline carried from M3.

### Focus annotations: A1

- **A2 (wrapper widgets) is the wrong shape twice over.** It adds
  widget kinds to a phase whose scope explicitly adds none, and it puts
  a node in the layout tree whose only job is annotation — which then
  has to be given layout semantics (does `FocusGroup` stack? pad?
  stretch?), none of which anyone wants to decide. It would also make
  the group a *structure*, when DD-004's whole point is that the scope
  is structure-independent.
- **A3 (one enumerated attribute) is tidier on paper** and worse in
  use: a container that is both a group and a scope becomes
  inexpressible, and the enumeration invites future roles to be added as
  values rather than considered as concepts. M4 has no both-at-once
  case, but designing a surface that cannot express one is a gratuitous
  limit.
- **A1** matches how every other M4-era attribute is spelled, composes
  freely, and defaults to absent — an unannotated container behaves
  exactly as today, which is what keeps every existing `.ui` file
  unchanged.

The concrete spelling proposed is `focus-group: true` and
`modal-scope: true`, both boolean, both constant-only in M4 (matching
the `Box.fill` / `WrapPanel` precedent for attributes that do not
traverse the binding path). Constant-only is a real limit: **a scope
cannot be turned on and off by a binding.** It does not need to be —
A's lightbox controls its *existence* with `if`, and DD-004's entry is
an act, not an attribute. Recorded so a later phase that wants a
bindable scope knows it is a change, not an oversight.

**Focusability opt-in** (DD-003's F3) is deliberately **not spelled in
M4**. No M4 widget needs it: A's focusable widgets are all Button
family. M4-Phase 5's text field is the first case, and it is better
spelled alongside the widget that needs it than invented one phase early
against a hypothetical. The extension point is in the *derivation*
(DD-003), which is what makes the later spelling additive.

### Key handlers: K1 for M4, with K2's shape recorded

- **K3 is out** — the intake classification puts a general shortcut
  mechanism outside M4's acceptance, and AC1 is satisfied by Esc, Tab
  and Left/Right.
- **K2 is the honest general answer** and buys nothing A needs. A's
  Left/Right steps between photos, which is a state change the lightbox
  already owns; Esc closes the scope, which DD-004 delivers by bubbling
  to a recipient that is already well-defined. Adding a key-signal
  family would put a new normative surface into the spec for zero
  consumers in this phase.
- **K1's cost** is that "what happens on Left/Right" becomes runtime
  behaviour rather than authored behaviour, which is less discoverable
  and less flexible.

The recommendation is K1 **with the seam left where K2 would attach**:
keys are delivered by the same bubble walk as clicks (DD-001), so adding
a key-signal family later is an addition to the signal vocabulary and
not a change to routing. What M4 ships is Esc handled by the scope and
Left/Right handled by the lightbox's own state.

**A caveat this decision must not hide.** A's Left/Right stepping
changes which photo is shown, and *showing* it — the caption, the
selected thumbnail — needs index reads and equality selection, which are
M4-Phase 3. This phase's evidence for Left/Right is therefore a bound
value changing, not the finished picture
([framing.md](../requirements/framing.md) §範囲の縫い目).

### IR and compiler impact

- **`clicked` on any widget**: the handler table is already per-node and
  keyed by signal name; the change is in `check` (accepting the signal
  on non-Button kinds) and in the runtime's dispatch, not in the IR
  shape.
- **`item` / `index` in handler position**: `lower` must carry the loop
  scope into the handler body, and the runtime's handler evaluation
  context must supply it. This is the only genuinely new IR content in
  the phase.
- **Two boolean attributes**: constant-only, so they follow the
  `Box.fill` precedent — a per-kind field, no new `PropertyValue`
  variant, no binding path.

**`docs/dsl_spec.md`** gains: `clicked` generalised from the Button
section to a common signal; `item` / `index` availability in handler
position; the two attributes; and a short statement of the focus and
modal-scope semantics an external implementor would need. `docs/abi_spec.md`
is **not** touched — no new entry point (framing agreement 7).

### Keeping the fixture out of the normative surface

The mechanism fixture composes existing widgets and supplies its group
and scope annotations through a test-side map, not through `.ui`. Once
this decision lands, the fixture can use the real attributes. Either way
**the fixture is not a widget**: nothing it does appears in
`docs/dsl_spec.md` or the widget list, and the group spelling for
RadioButton-like widgets remains M5's. The phase-end check includes
confirming the fixture introduced no normative spelling.

## Recommendation

- **G1** — one `clicked` signal, authorable on any widget; Button
  keeps its `enabled` suppression and its keyboard activation.
- **I1** — bare `item` / `index` in handler position inside `for`, with
  accept and reject tests.
- **A1** — `focus-group: true` and `modal-scope: true` as constant-only
  boolean attributes on any container.
- **No focusability opt-in spelling in M4**; the derivation is the
  extension point.
- **K1** — no authored key-signal family; Esc is the scope's, Left/Right
  is the lightbox's own state; the seam where K2 would attach is DD-001's
  bubble walk.
- **`docs/dsl_spec.md` moves; `docs/abi_spec.md` does not.**

## Forward-compat exposure

- **Key signals (K2) remain additive** — a new signal family on the
  existing routing.
- **A focusability attribute remains additive** and is expected at
  M4-Phase 5.
- **A bindable scope is a change, not an addition** — the constant-only
  choice is recorded as a limit with that consequence stated.
- **M5's group spelling for real widgets** is unconstrained by
  `focus-group`, which annotates a container rather than declaring a
  widget kind. A RadioGroup widget in M5 can carry the semantics
  internally without the author writing the attribute.
- **The both-at-once case** (a container that is a group and a scope) is
  expressible under A1 and untested in M4.

## Technical risk re-evaluation

- **`item` / `index` at handler-invocation time is the phase's only new
  IR content**, and its hazard is the one M3 named when it deferred the
  surface: the loop scope must be the one in force when the handler
  *runs*, not when it was authored, and repetition can re-materialise
  the subtree in between. The evidence must include a click after a
  collection mutation, not only a click on a freshly built list.
- **Two new attributes are two new checker branches**, each needing a
  test that fires it (implementation-gates trap 4): the attribute on a
  widget kind that cannot carry it, and a non-constant value.
- **G1 widens an existing signal name**, so a `.ui` that previously
  produced an "unknown signal" diagnostic may now be accepted. That is
  the intended direction, and the reject-side tests are what pin how far
  it widens.
