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

The keyboard half of this record is written against the reference
toolkits rather than against M4's consumer list alone, because two of
its choices outlive the phase: **an overlay's dismissal contract is
what M5's Dialog widget inherits**, and **an authored key surface is
what a 1.0 release must not have to apologise for**. HTML's `<dialog>`
and Popover API and Slint's `PopupWindow` and `FocusScope` are the
desktop references; Compose and SwiftUI are read for shape, not for
their mobile-specific sources.

## Sub-issues

- **Generic `clicked`** — the spelling on a non-Button widget.
- **Per-item handlers**, which M3 sends here as four coupled questions:
  **admission** of a handler inside a `for` body, the **spelling** of
  binder reads in handler position, the handler's **registration
  lifecycle**, and its **interaction with iteration identity**.
- **Dismissal** — how an overlay learns the user wants it closed. This
  is separate from key input and is the half M5's Dialog widget
  inherits.
- **Authored key input** — how an application reacts to a key at all,
  and at what granularity.
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

### Dismissal — how an overlay learns the user wants it closed

- **D1 — the runtime closes it.** Esc makes the runtime clear the state
  the enclosing conditional reads.
- **D2 — a dismissal request signal** on the scope (`dismiss => { … }`),
  raised by whatever gesture means "close this", with the author
  deciding what closing is.
- **D3 — D2 plus a declarative policy attribute now**
  (`dismiss-policy: none | close-request | any`, the shape HTML's
  `closedby` uses).

### Authored key input

- **K1 — no authored key surface in M4.** Left/Right stepping is a
  built-in behaviour of the scope.
- **K2 — one signal per key** (`key-left => { … }`, `key-right`,
  `key-escape`), delivered by DD-001's walk.
- **K3 — one signal that names its key as a string**
  (`key-down("ArrowLeft") => { … }`), same delivery.
- **K4 — one signal that receives every key**, with the handler body
  deciding which key it was (Slint's `key-pressed(event) -> EventResult`
  shape).
- **K5 — a declarative shortcut table** at window or scope level.
- **K6 — K3's shape with a structured key value** rather than a string
  (`key-down(Key.ArrowLeft) => { … }`, with a modifier set as part of
  the value).

K3 / K4 / K6 are **not three points on one line**. Two independent axes
are in play, and conflating them is how a reader ends up thinking the
string is the only alternative to Slint's callback:

| | key selected in the **declaration** | key selected in the **body** |
|---|---|---|
| key denoted by a **string** | K3 | (possible; same body blocker as K4) |
| key denoted by a **structured value** | K6 | K4 (Slint's shape) |

The two axes have **different blockers in M4**, which is why they are
listed separately and excluded separately.

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
A's lightbox controls its *existence* with `if`, and under DD-004 the
subtree's presence is what enters the scope, so a bindable attribute
would be a second switch for the same thing. Recorded so a later phase
that wants a bindable scope knows it is a change, not an oversight.

**Focusability opt-in** (DD-003's F3) is deliberately **not spelled in
M4**. No M4 widget needs it: A's focusable widgets are all Button
family. M4-Phase 5's text field is the first case, and it is better
spelled alongside the widget that needs it than invented one phase early
against a hypothetical. The extension point is in the *derivation*
(DD-003), which is what makes the later spelling additive.

### Dismissal: D2

**Dismissal and key input are two decisions, not one.** "Esc closes the
lightbox" and "Left/Right steps the photo" look like one question and
are not: every reference toolkit separates them, and the separation is
design content rather than taxonomy.

- **HTML `<dialog>`** raises a `cancel` event on Esc and takes a
  declarative `closedby` attribute with three values: `none` (only a
  developer-provided mechanism), `closerequest` (a *platform-specific
  user action* — Esc on desktop), `any` (adds light dismiss, i.e. a
  click outside). The spec's vocabulary is **"close request"**, not
  "Escape", precisely so the same path covers a platform's other
  dismissal gestures.
- **Slint's `PopupWindow`** takes `close-policy` with
  `close-on-click` / `close-on-click-outside` / `no-auto-close`, and
  general key input is a *different* mechanism (`FocusScope`'s
  `key-pressed` / `key-released` with an accept-or-reject result).
- **Compose** pairs `onDismissRequest` with `DialogProperties`'
  `dismissOnBackPress` / `dismissOnClickOutside`; **SwiftUI** pairs a
  `dismiss` action with `interactiveDismissDisabled()`.

What is mobile-specific is only the *source* (a back gesture); the
*shape* — a request, a declarative policy, and the application deciding
what closing means — is what the two desktop references have too.

**The decisive fact for Wasamo is that Esc is not the only source.**
Click-outside arrives at M4-Phase 9 with the top layer, and a close
button arrives with M5's Dialog widget. If Esc is authored as an
ordinary key, Phase 9 needs a second unrelated hook and M5's Dialog
cannot offer one "on dismiss" contract — which is the divergence this
decision exists to prevent.

- **D1 is excluded.** It requires withdrawing DD-004's "the act of
  closing is authored; the core never mutates the tree", and it burns an
  application policy into the runtime: a scope that must *not* close on
  Esc — an unsaved-changes confirmation — becomes inexpressible. All
  four references can express it (`closedby="none"`, `no-auto-close`,
  `dismissOnBackPress = false`, `interactiveDismissDisabled`). D1 would
  ship strictly less than the state of the art.
- **D3 is premature.** With Esc as the only source in M4, two of the
  three policy values are indistinguishable. The attribute becomes
  necessary at Phase 9, and adding it then is additive.
- **D2**, therefore: the scope raises `dismiss`, and **whether a handler
  exists is the policy** while there is exactly one source. Phase 9
  splits policy from handling by adding the attribute.

**Admission follows the recipient.** `dismiss` is admitted only on a
container that also carries `modal-scope: true`. The request is
addressed to scopes, so a `dismiss` handler anywhere else could never
fire — the same silently-never-fires failure mode the key-name
validation exists to prevent — and the checker rejects it for the same
reason.

Two properties follow that are worth stating because they are free here
and expensive elsewhere:

- **Vetoing is automatic.** HTML needs `cancel` to be cancelable so an
  author can keep a dialog open. Wasamo needs nothing: the runtime never
  mutates the tree, so not writing the state *is* not closing. DD-004's
  discipline is what buys this.
- **The request is addressed, not bubbled.** It goes to the innermost
  entered scope and stops there, matching HTML's "the close request goes
  to the topmost item in the top layer". Bubbling would let a dialog
  that ignores Esc close the menu underneath it.

### What kind of event this is — a physical key press, not a character

Fixed here because the answer constrains every option below, and
because the web platform's own history shows what happens when it is
left implicit.

**This surface is the equivalent of the DOM `keydown` event, not of the
deprecated `keypress`.** The distinction the web settled on is:

- **`keydown`** — a physical key went down, carrying the key's identity
  and the modifier state. Used for *commands*.
- **`beforeinput`** — text is about to be inserted, whatever produced
  it. Used for *content*.
- **`keypress`** — deprecated, because it tried to be both and could
  represent neither the modifier-bearing command nor the text produced
  by an IME, a flick keyboard, dictation, or a paste.

Wasamo takes the same split, and the two halves land in different
phases: **this signal is the command half**, and the content half is
M4-Phase 5's text field and M4-Phase 6's TSF integration, which receive
text through the text-store path and **not** through key signals.

Two rules follow that are fixed here and implemented later, in the same
way this record fixes the screen-reader modality rule for M4-Phase 11:

- **While an IME composition is active, key events belong to the
  composition and are not delivered to authored handlers.** This is not
  a detail: without it, pressing Left to move within a Japanese
  candidate list inside A's lightbox would also step the photo. The web
  platform needs `isComposing` for exactly this, and it is the concrete
  reason `keypress` could not survive.
- **Auto-repeat is delivered.** Holding Left scans back through photos,
  which is the behaviour A wants and which an author cannot reconstruct
  without state. The cost is that a repeat-hostile action — a save — is
  not well served, and that case is not expressible in M4 anyway (see
  the character-key limit below). A structured event (K4 / K6) is where
  a repeat flag would live.

The signal is therefore named **`key-down`**, not `key-pressed`.
Slint's callback is spelled `key-pressed`, and following it would
collide with the one web name that means something different from what
we mean. `key-down` also pairs with a later `key-up`.

### Authored key input: K3

**K2 is excluded on the language's lifetime rather than on M4's consumer
list.** Three fixed key names would have to be either removed or
grandfathered when the general mechanism arrives — and a general
mechanism is certain before 1.0, because every toolkit in the reference
set has one. Shipping three names buys a phase and costs a permanent
explanation.

**A hard constraint decides the shape.** `docs/dsl_spec.md` §3 gives
`statement ::= assign_stmt ";"` — **a handler body contains assignments
and nothing else.** There is no `if`, and comparison does not enter the
expression language until M4-Phase 3's predicates. So **K4 — Slint's
shape, one callback that receives every key and branches in the body —
is not authorable in M4 at all**, however the runtime delivers it. This
is a measured property of the language, not a preference.

If the body cannot select the key, the declaration must. That is K3, and
it is **not** K2 with better marketing:

- **K3 adds one signal, not one per key.** Any key is expressible on the
  first day — `key-down("F5")`, `key-down("Delete")` — so nothing
  has to be deleted or grandfathered at 1.0. This is the whole of the
  owner's objection, answered structurally.
- **A key named by *identifier*** (`key-down.ArrowLeft`, reusing the
  `slot.*` precedent) would be cheaper today — no new production — and
  would have to be replaced on the day modifiers arrive, because
  `Ctrl+S` is not an identifier.
- **K5 is out**, as before: the intake classification puts a general
  shortcut mechanism outside M4, and a table detached from the widget
  tree brings its own precedence rules. K3 needs none — it rides
  DD-001's walk, first match consumes.
- **An unrecognised key name is a diagnostic.** A misspelled
  `"ArrowLef"` that silently never fires is the failure mode this
  surface would otherwise ship with. M4 recognises a small named-key
  table; widening it is additive.

#### The named-key limit

Modifier combinations are **not** a free widening of the key-name value,
and the reason decides what M4's recognised table contains.

`keydown` exposes two different identities for the same physical press:
the **logical key** the layout produces (DOM `event.key` — pressing the
physical `Q` position on AZERTY yields `"a"`) and the **physical
position** (DOM `event.code` — `"KeyQ"` regardless of layout). A
shortcut is conventionally spelled against the logical key so it matches
the labels on the user's keyboard; a game's WASD wants the physical one.
A string key name does not say which.

**M4 defers that question by construction**, and this is a decision
rather than an oversight: the recognised table contains **named
non-character keys only** — `Escape`, the four arrows, `Home`, `End`,
`PageUp`, `PageDown`, `Enter`, `Tab`-adjacent names, function keys —
for which the logical key and the physical position **coincide**. All of
M4's consumers are inside that set.

Two consequences, stated so a later phase does not inherit a false
premise:

- **`"Ctrl+S"` is not expressible in M4.** Admitting a character key is
  what forces the logical-versus-physical decision, and modifiers are
  mostly wanted *with* character keys. So modifiers and character keys
  are one extension in practice, not two, and neither is free.
- The extension is **additive** — a wider table plus one settled
  question — but it is not merely a wider value grammar.

#### Why K4 and K6 are both deferred, for different reasons

A structured key representation is **not** the Slint-style callback
under another name, and separating them is what makes each exclusion
checkable.

- **K4 (body-filtered) is blocked by the statement grammar.**
  `statement ::= assign_stmt ";"` — a handler body holds assignments and
  nothing else. No `if`, and no comparison in the expression language
  until M4-Phase 3's predicates. K4 is not authorable in M4 **however
  the runtime delivers the event**.
- **K6 (structured denotation) is blocked by the value grammar.** It
  needs a typed constant — a `Key` namespace with an `IrLiteral` to
  carry it — which is a new kind of value in the language, not a new
  spelling of an existing one. The `TypedValue` hold
  ([M4 framing](../../requirements/framing.md) §M4 に入れないもの) is
  the neighbouring decision, and inventing a one-off enum beside it is
  the sort of fait accompli this milestone avoids elsewhere.
- **They are deferred *together*, and that is the substantive point.**
  With the key selected in the declaration, a string and a structured
  value do exactly the same work: neither is compared, neither is
  passed anywhere, and the typo protection an enum would give is
  recovered by validating the name at `check`. **A structured key's
  payoff is almost entirely in the body-filtered form** — comparing
  against `Key.ArrowLeft`, reading a modifier set, testing a repeat
  flag. So K6 without K4 buys type-safety Wasamo already has by another
  route, at the cost of a new value kind; and K4 without K6 would want
  K6 immediately. The pair is one future decision, not two.

**Recorded conclusion: neither K4 nor K6 is supported in M4-Phase 2.**
The reopening condition is the same for both — a handler body that can
branch, which needs M4-Phase 3's comparison plus a branching statement
form that no phase currently schedules. K3 and K4 then coexist rather
than one replacing the other: HTML ships `accesskey` beside `onkeydown`,
and Flutter ships `Shortcuts` beside `KeyboardListener`. A declarative
filter and a catch-all are different tools.

### Which keys the runtime keeps

Authored keys and the focus machinery want the same keys, so the split
is fixed here rather than discovered:

| Key | Recipient |
|---|---|
| Tab / Shift+Tab | **Always the runtime.** Never delivered to an authored handler — an author must not be able to break traversal |
| Arrows, focus inside a focus group | **The runtime** (group movement, DD-003) |
| Arrows, anywhere else | The authored walk — A's Left/Right is this case |
| Esc, with a scope entered | Converted to a **dismissal request** on the innermost entered scope |
| Esc, otherwise | The authored walk (`key-down("Escape")`) |

The rule underneath is DD-001's, not a new one: a built-in behaviour
consumes at the focused widget, and only unconsumed keys walk to
ancestors.

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
- **`dismiss`**: an ordinary signal name in the existing per-node
  handler table. Nothing in the IR distinguishes it; what differs is
  that the runtime raises it rather than a pointer message doing so, and
  that `check` admits it only on a container carrying
  `modal-scope: true`.
- **`key-down("<key>")`**: the one shape that needs new **grammar** —
  a signal handler whose name carries an argument. The cost is
  acknowledged rather than hidden: `slot.*` showed a dotted identifier
  needs no new production, and that cheaper spelling was rejected
  because modifiers (`"Ctrl+S"`) are not identifiers. The key name is
  validated at `check` against a recognised table, so the IR carries a
  key name that the loader can map without re-parsing.

**`docs/dsl_spec.md`** gains: `clicked` generalised from the Button
section to a common signal; `item` / `index` availability in handler
position; the two attributes; the `dismiss` and `key-down` signals;
the table of which keys the runtime keeps; and a short statement of the
focus and modal-scope semantics an external implementor would need.
`docs/abi_spec.md` is **not** touched — no new entry point (framing
agreement 7).

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
- **D2** — the scope raises a `dismiss` signal and the author decides
  what closing means; while Esc is the only source, the presence of a
  handler is the policy. `dismiss` is admitted only beside
  `modal-scope: true`, where it can fire; elsewhere it is a diagnostic.
  The declarative policy attribute lands at M4-Phase 9 with the second
  source. **This is a dismissal concept, not a key concept**, and it is
  the contract M4-Phase 9's click-away and M5's Dialog widget both
  consume.
- **K3** — one `key-down("<key>")` signal whose key is named in the
  declaration as a string, delivered by DD-001's walk, first match
  consuming. It is the **`keydown` equivalent**: a physical key press
  with modifier state, for *commands* — **not** the deprecated
  `keypress`, and **not** a text-input path. An unrecognised key name is
  a diagnostic. Tab is never delivered to an authored handler, and
  arrows are the runtime's only while focus is inside a focus group.
- **The recognised key table holds named non-character keys only** in
  M4, which keeps the logical-key versus physical-position question
  closed. `"Ctrl+S"` is therefore **not** expressible in M4; character
  keys and modifiers are one later extension, not a free value widening.
- **Two rules fixed here, implemented later**: an active IME composition
  owns the keyboard and its keys are not delivered to authored handlers
  (M4-Phase 6); auto-repeat **is** delivered.
- **K4 (a catch-all callback) and K6 (a structured key value) are both
  out of M4-Phase 2**, for different reasons — no branching in handler
  bodies, and no typed-constant kind in the value grammar — and they
  reopen **together**, because a structured key's payoff is almost
  entirely in the catch-all form.
- **`docs/dsl_spec.md` moves; `docs/abi_spec.md` does not.**

## Forward-compat exposure

- **The dismissal policy attribute is additive and is expected at
  M4-Phase 9**, when click-away gives the second source. Its values are
  known in advance because HTML has already settled the ladder —
  `none` / `close-request` / `any` — and Phase 9 inherits the naming
  question, not the design.
- **`key-up` is additive**, a sibling signal with no buyer today.
- **Character keys and modifier combinations are one later extension.**
  It is additive in grammar — a wider recognised table — but it carries
  a question M4 does not answer: whether a key name denotes the
  **logical key** the layout produces or the **physical position**.
  M4's table is named non-character keys only, where the two coincide.
- **A catch-all key signal (K4) and a structured key value (K6) are both
  additive, and they reopen together.** K4 needs a handler body that can
  branch — comparison arrives with M4-Phase 3's predicates, a branching
  statement form is on no phase's list. K6 needs a typed-constant kind
  in the value grammar, adjacent to the held `TypedValue` work. Adding
  K4 without K6 would want K6 immediately, which is why the reopening
  condition is written for the pair. K3 and K4 then coexist rather than
  one superseding the other.
- **Text input never rides this surface.** M4-Phase 5's field and
  M4-Phase 6's TSF take content through the text-store path; `key-down`
  stays the command half. A later phase that routes characters through
  key signals would be re-making the mistake that deprecated the web's
  `keypress`.
- **A focus stop that wants Tab itself** — a text field that inserts a
  tab character, a grid that moves between cells — is the case the
  "Tab is always the runtime's" rule forecloses. No M4 widget wants it;
  M4-Phase 5's text field is where it would first be argued, and it
  would arrive as an opt-out on the widget rather than as an authored
  Tab handler.
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
  widget kind that cannot carry it, a non-constant value, and — for
  `dismiss` — a handler on a container that is not a scope.
- **G1 widens an existing signal name**, so a `.ui` that previously
  produced an "unknown signal" diagnostic may now be accepted. That is
  the intended direction, and the reject-side tests are what pin how far
  it widens.
