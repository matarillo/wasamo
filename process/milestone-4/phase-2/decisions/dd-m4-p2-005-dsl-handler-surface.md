# DD-M4-P2-005 — The authored surface: handlers, item references, and focus annotations

**Status:** Proposed
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

The per-item half is a reopening, not a new surface, and the thing being
reopened is **larger than a spelling**.
[dsl_spec.md §4.15](../../../../docs/dsl_spec.md) rejects a
`signal_handler` member *anywhere inside a `for` body* — not merely
binder reads in handler position — and states why, and what has to be
answered together when it is admitted:

> Admitting handlers without binder reads would ship per-item widgets
> whose handlers can only mutate global state — a half-surface the spec
> would have to explain away; admitting binder reads in handlers *is*
> the per-item interaction surface (select-this-item / delete-this-item),
> whose real driver arrives with input work. **Handler admission,
> handler-position binder reads, registration lifecycle, and their
> identity interaction are designed together at that point.**

This is that point. A's thumbnail click is the driver, and the four
concerns M3 named are four of this record's sub-issues rather than one.

## Sub-issues

- **Generic `clicked`** — the spelling on a non-Button widget.
- **Per-item handlers**, which M3 sends here as four coupled questions:
  **admission** of a handler inside a `for` body, the **spelling** of
  binder reads in handler position, the handler's **registration
  lifecycle**, and its **interaction with iteration identity**.
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

### Binder reads in handler position

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

### Per-item handlers: the four coupled answers

**Admission.** A `signal_handler` is admitted inside a `for` body. M3's
stated reason for the rejection was that admitting handlers *without*
binder reads would ship a half-surface; admitting both together removes
the objection rather than overriding it.

**Spelling: I1.** M3 already binds `item` and `index` inside `for` in
*binding* position, and the checker already knows the loop scope there.
I2 would introduce a second spelling for the same values, distinguished
only by which side of a `=>` they appear on — a distinction with no
meaning to an author. The checker rule follows the binding position's
shape: the binders resolve only inside a `for` body, and a reference
outside one is a diagnostic. **Both directions need a test**, the accept
case and the reject case, per the shared-lexer discipline carried from
M3.

**Registration lifecycle.** A handler inside a `for` body is registered
per generated item, so it must be released when that item's subtree is
dropped. It rides the path that already releases the subtree's
bindings — the generated subtree is the unit that owns both, and giving
handlers a second lifecycle would create exactly the parallel-data drift
the runtime keeps eliminating. Nothing new is invented here; what this
decision fixes is that the handler's registration is *not* separately
owned.

**Identity interaction — the one that has to be stated, not assumed.**
M3's iteration identity is **positional and un-keyed** (§4.15). A binder
read inside a handler resolves **when the handler runs**, not when the
subtree was generated. Together these mean a per-item handler belongs to
a **position**, not to an item: after a collection mutation, the handler
at position `n` reads whatever item is now at position `n`.

That is the correct behaviour, and it is correct *because* of the
invocation-time resolution rather than in spite of it. "Delete the item
this row shows" works only if the row's handler reads the index it
currently occupies; a handler that had captured its item at generation
time would delete the wrong row after any preceding removal. The two
choices — positional identity and invocation-time reads — are
consistent, and neither is safe to change alone. A future keyed-identity
opt-in (§4.15 records it as a possible future) is what would reopen it,
and it is named here so that phase re-derives the pairing rather than
discovering it.

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
- **Per-item handlers** — a `signal_handler` is admitted inside a `for`
  body; **I1**, bare `item` / `index` in handler position, with accept
  and reject tests; the registration is released with the generated
  subtree on the path that already releases its bindings; and a binder
  read resolves at invocation time, so a handler belongs to a position
  rather than to an item under the positional identity baseline.
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
- **Keyed iteration identity** is the change that would reopen the
  per-item handler semantics, because "the handler belongs to a
  position" is a joint consequence of positional identity and
  invocation-time reads. §4.15 already records keyed identity as a
  possible future opt-in; this is what it would have to re-decide.

## Technical risk re-evaluation

- **Per-item handlers are the phase's only new IR content**, and the
  hazard is the one M3 named when it deferred the surface: the loop
  scope must be the one in force when the handler *runs*, and repetition
  can re-materialise the subtree in between. The evidence must therefore
  include a click **after a collection mutation**, not only a click on a
  freshly built list — a test that only ever clicks a freshly generated
  row cannot distinguish invocation-time resolution from
  generation-time capture.
- **The registration lifecycle fails silently in one direction.** A
  handler left registered against a dropped subtree is not visible in
  any rendered frame; it surfaces as a stale invocation or a leak. The
  close artifact is the structural side-effect enumeration for subtree
  removal, listing handler registrations beside the bindings that path
  already releases.
- **Two new attributes are two new checker branches**, each needing a
  test that fires it (implementation-gates trap 4): the attribute on a
  widget kind that cannot carry it, and a non-constant value.
- **G1 widens an existing signal name**, so a `.ui` that previously
  produced an "unknown signal" diagnostic may now be accepted. That is
  the intended direction, and the reject-side tests are what pin how far
  it widens.
