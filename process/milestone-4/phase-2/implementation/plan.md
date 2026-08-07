---
phase: M4-Phase 2
title: Implementation task list
status: draft
adr: process/milestone-4/phase-2/decisions/preamble.md
---

# M4-Phase 2 — Implementation task list

Mutable during the phase. Task splits, additions and reorderings are
recorded here as they happen rather than left as a frozen prediction
([AGENTS.md §Commit rules](../../../../AGENTS.md#commit-rules)). Two
framing allowances ([framing.md](../requirements/framing.md) agreement
⑤) are part of that mutability rather than exceptions to it: a spike may
be **inserted mid-phase** when a task finds it is missing the
information a decision needs, and a finding that shakes an Accepted
decision **reopens the decision** — supersede when shipped behaviour
changes, dated annotation when only the explanation narrows
([workflow.md](../../../procedures/workflow.md)) — instead of being
absorbed as a local workaround. The execution framing, the sequencing
thesis and what a green suite is worth are in
[preamble.md](./preamble.md).

Each task runs the implementation gates at **both** start and close
([implementation-gates.md](../../../procedures/implementation-gates.md)),
records its gate selection with reasons for the rows judged
non-applicable, and — because a start-gate selection is a prediction
that goes stale (Phase 1 F-53) — **re-decides that selection at close**
whenever the task built something it did not expect to build.

---

## T1 — Layout-derived hit rectangles

Each node retains its arranged rectangle in DIP, absolute within the
window's client area. The writer is the **lockstep walk that applies
layout results to the Visual** (`sync_visuals`) — one walk, two stores —
because the arranged `LayoutNode` tree is transient and does not outlive
the layout entry (DD-002). No consumer yet — this task adds the source,
T2 switches the readers.

- The write inherits the walk's audited single-pass discipline; the
  enumeration that closes that audit now covers both stores (the
  physical write to the Visual, the DIP rectangle to the node).
- The node also records whether its kind clips (`ScrollView`, `Grid`,
  `ZStack` install `InsetClip{0,0,0,0}`); T2's resolution consumes it.
- A subtree attached but never laid out has no rectangle and is not
  hit-testable. That is the intended failure and is asserted, not
  assumed.
- **Evidence:** integration assertions that the retained rectangle
  matches the arranged result, plus a writer audit naming every site
  that could write the field.

- [x] T1

## T2 — Single-target hit resolution and the complete geometry migration

Replace the fire-every-Button recursion with reverse-order topmost
resolution reading T1's store, **bounded by ancestor clips**: a clipping
container's subtree is resolvable only inside that container's
rectangle; a non-clipping container's overflowing children stay
resolvable, matching paint (DD-002). Every widget with a visual is a
candidate; whether anything reacts is T3's question.

- The resolution is a **free function over plain data** (rectangles +
  order + clip bounds → target), so occlusion, clipping and per-item
  resolution are pure logic with no Compositor.
- **The migration completes in this task** (DD-002): both input-path
  readers switch source — `hit_test_click` and `update_hover`, the
  latter entered from three window messages. `update_hover` changes
  geometry source only here; its semantic redesign is T4. The close
  artifact is a call-site audit table showing **zero** `visual_rect`
  readers on the input path. No commit between T1 and T2 may leave a
  mixed path.
- **Three existing test files hit-test against a hand-pinned Visual
  rectangle rather than a laid-out tree** (T1 finding):
  `button_enabled.rs`, `bool_binding_live_propagation.rs` and
  `togglebutton_runtime_integration.rs` each write `SetOffset` /
  `SetSize` directly so today's readback lands inside. Those nodes have
  never been through the layout pass, so they hold no rectangle and
  their clicks resolve to nothing once the readers switch. Laying their
  trees out is part of the migration: re-pinning the store or skipping
  the tests reintroduces the mixed path the obligation above exists to
  prevent.
- `clips_children` gains its first production caller here, so the
  `__clips_children_for_test` accessor T1 needed stops being a second
  entry point. **It is retained rather than removed** (recorded
  deviation, [log.md](./log.md) §T2 close gate #3): eight of the eleven
  widget kinds cannot have children, so no production click can reach
  their arm of the predicate, and deleting the accessor would delete
  T1's per-kind agreement pin for those eight instead of replacing it.
- Edge containment is a **boundary condition**, so a deliberately wrong
  implementation must be shown to make the named test fail
  ([DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)).
- **Staleness control** (DD-002's named mitigation): a fixture asserts
  that a click resolves correctly **after** a property write triggers
  re-layout — the path where a cache goes stale.
- **The conversion the cancellation used to hide is asserted at a
  non-unit scale.** At 100% a missing pointer conversion cannot fail any
  test ([preamble.md §What "green" is worth](./preamble.md)), so an
  integration fixture drives the window to a scale ≠ 1 through the
  M4-Phase 1 synthesized propagation machinery and asserts that a click
  at physical coordinates resolves to the widget whose DIP rectangle
  contains the converted point. Touching the DPI-fixture environment
  fires the Phase 1 handoff's recorded desktop-range dependency
  ([constraints §10](../requirements/constraints.md)), which is read
  before this fixture is written.
- **Evidence:** pure-logic tests over a constructed *overlapping* tree —
  not the gallery, where occlusion is unobservable until T10
  ([preamble.md](./preamble.md)) — including a clip case (a rectangle
  outside its clipping ancestor does not resolve); the audit table; the
  staleness fixture; the non-unit-scale resolution fixture.
- **Two existing test files beyond the three named above also stood on
  the old geometry source** (found at T2's start gate):
  `dpi_scale_matrix_integration.rs`'s one-divisor test pins a property
  this task deletes and stays green with a false reason, so it is
  re-documented with its assertions kept; `iteration_mutation_integration.rs`
  derives a click point from a Visual readback, which is a test-side
  physical coordinate and is classified rather than changed.

- [x] T2

## T3 — Propagation and the drain boundary

Target, then ancestors, until a handler runs; a handler that runs
consumes the event. No descending phase.

- The ancestor chain is captured **before** dispatch, and the reactive
  drain runs **once after** the walk completes. The chain needs no new
  traversal: T2's `resolve_topmost` returns the **path of child indices**
  to the target, so the ancestors are that path's prefixes.
- **The drain is not where DD-001's wording implies** (T2 measured):
  `wnd_proc`'s message arms never call `emit::drain_if_outermost` — the
  production call site is the line after `DispatchMessageW` in
  `wasamo_runtime::run`, so today a drain runs after *every* dispatched
  message rather than once per dispatch. Reconciling "one drain per
  dispatch, after the walk completes" with that boundary is this task's,
  and a fixture that expects a synthesised message to re-layout must
  pump `run` ([log.md](./log.md) §T2 close gate #2). **Resolved: the
  drain stays at the message-loop boundary.** One drain per message is a
  superset of one drain per dispatch, the walk contains no drain point,
  and draining inside `wnd_proc` would invoke host callbacks while the
  runtime holds a `&mut WindowState` it returns through — a callback may
  call `wasamo_window_destroy` (abi_spec §6). Full argument in
  [log.md](./log.md) §T3 close gate #2.
- **The task's real shape was measured at its start gate, and two of
  T2's forward assumptions did not survive it** ([log.md](./log.md) §T3
  start gate). `clicked` on a non-Button widget is already accepted by
  `wasamoc check` and attached by the IR loader, so this task is what
  makes an already-authorable handler fire rather than an addition to a
  working generic dispatch; and **the Button-child narrowing T2 carried
  forward does not exist** — `build_layout_tree` maps Button to a
  childless `LayoutNode`, so such a child never gets a rectangle, is
  never a hit candidate, and the click resolves to the Button. The
  evidence item that stood on it ("a click on a Button's child
  activating the Button through the ancestor walk") is **unbuildable**:
  in a debug build the shape aborts at load on T2's `sync_visuals`
  child-count assertion. It is replaced by the general property it was
  a special case of — a click on a widget with no handler activates the
  nearest ancestor that has one — and the defect is carried to T8,
  which rejects the shape at both gates (close gate CF-1; the withheld
  capability has its own [candidate pool](../../../candidate-pool.md)
  row).
- Structural side-effect enumeration: what a handler's state write
  pulls in (drain → re-layout → rectangle store → focus validity), and
  which of those the walk must not observe mid-flight.
- **Evidence:** integration tests that synthesize a click into a nested
  tree and read back which handlers ran — a non-Button widget's handler
  firing at all; a click on a handler-less widget activating its
  ancestor; a handler that removes its own subtree; a host listener
  connected through `wasamo_signal_connect` consuming the walk; a
  disabled Button suppressing its own dispatch **without** consuming;
  and the consumption control DD-001 names: adding a handler to a nested
  widget is shown to stop the ancestor's from firing.

- [x] T3

## T4 — Hover and pressed behind the routing model

The semantic half DD-001 fixes: hover / pressed become transitions
computed from enter / leave against the resolved target, replacing the
whole-tree walk. Geometry already comes from T1's store (switched in
T2); this task changes who owns the state transitions.

- Single-target consistency: the widget that paints pressed is the
  widget a release would dispatch to; overlapping widgets no longer both
  hover. Hover and pressed gain **no authored surface** — they stay
  Button-family presentation (DD-001). The two descriptions coincide for
  every shape the widget set can build, and that is measured rather than
  assumed: `build_layout_tree` maps Button-family to a childless
  `LayoutNode`, so a Button-family node on the dispatch chain is always
  the target ([log.md](./log.md) §T4 start gate fact 3).
- **Replacing the walk needs a retained record, not a narrowed walk.**
  The normative text names the whole-tree walk as the thing being
  replaced, so the window holds *which node currently paints a
  non-`Normal` state* and each pointer message is a leave/enter against
  it. The record and the painted state are a derived pair written in one
  primitive (implementation-gates trap #3), and the paths that can
  invalidate the record from outside — a binding write that disables a
  Button, a root swap, a handler's rebuild mid-message — are enumerated
  at the close gate rather than left to chance.
- `WM_MOUSELEAVE` clears through the same transition path; the
  disabled-Button arm keeps its no-reaction contract, and the old walk's
  descend-into-children behaviour is **deleted rather than adapted** —
  under topmost resolution a Button-family widget's `WidgetNode` children
  hold no rectangle and were never reachable candidates.
- The synthesised pointer update after a scale change stays **not
  adopted** (DD-001): hover self-corrects on the next move, and a second
  producer of hover state is the shape DD-M4-P1-002 closed. This needs no
  code — [architecture.md §12.5](../../../../docs/architecture.md)
  already states it, which discharges
  [constraints §5](../requirements/constraints.md).
- **Evidence:** integration tests driving enter / leave / press /
  release sequences and reading back state transitions, including the
  overlap case where only the topmost widget reacts — **plus a GUI
  positive control**, because the gallery's overlap is reachable today:
  with the lightbox open its stretch/stretch scrim covers the toolbar,
  and the checked `ToggleButton` underneath hovered through it before
  this task. The lane is correspondingly **full independent review**, not
  the branch/test-focused review [preamble.md](./preamble.md) predicted.

- [x] T4

## T5 — Per-window focus state and Tab traversal

`FocusState` on `WindowState`; the spike's `focus_core` gains its first
production caller; projection derives `Stop` / `Container` from the
widget kind — the two roles the tree can already supply.

- Nothing is focused at window open; Tab / Shift+Tab in declaration
  order, wrapping both ends, so the first Tab lands on the first stop.
  A disabled Button is skipped. Click focuses the **nearest focusable
  widget at or above** the resolved target and leaves focus unchanged
  when there is none; window activation does not change it. The click's
  focus write is ordered **before** dispatch, because a handler's
  synchronous rebuild can invalidate the path the click resolved.
- The key path integrates with the **existing** `WM_KEYDOWN` arm, which
  today forwards to the uninstalled `key_down_fn` host slot and returns.
  Routing consumes ahead of that slot without installing it — the first
  installer fixes the callback unit as shipped API
  ([constraints §2](../requirements/constraints.md)), and this phase
  must not be that installer. **An unconsumed key now falls through to
  `DefWindowProc`** instead of being swallowed (DD-001); the arm's
  return path changes here and the change has its own assertion.
- **The key *walk* is not built here** (decided at the start gate). No
  authored key handler can exist until T8 adds `key-down("<key>")`, so a
  dispatch built now would be a branch no test could fire. T5 lands the
  consumption half and the fallthrough half; T8 lands the dispatch
  between them.
- **`DefWindowProc`'s return value cannot distinguish the fallthrough**
  (measured over eight candidate keys: every one returns 0 either way).
  What discriminates is the **host key slot**, which the arm reaches only
  after traversal declines — a fixture installs a recorder there, and
  that is what makes "routing consumes ahead of the slot" falsifiable.
- The focus indicator adds **no** `SetOffset` / `SetSize`: it is a brush
  colour written through the same primitive hover uses, and creates no
  `Visual`. Close artifact: the enumeration of every `SetOffset` /
  `SetSize` in the runtime with its pass (DD-003) — still the six inside
  `sync_visuals`. [architecture.md §13.3](../../../../docs/architecture.md)'s
  "applied by the same pass that writes visual geometry" cannot be
  implemented literally, because a focus change triggers no layout pass;
  the divergence is recorded for T13 rather than resolved here.
- The indicator must be **distinguishable** (DD-003): M4's only means is
  a background change, and the ToggleButton selected state and hover
  state are also background changes. Evidence includes a frame pair
  showing focused versus hovered/selected as visibly distinct states.
  The gallery's **first** Tab stop is a checked `ToggleButton`, so the
  hardest of the three comparisons is the one the evidence meets first.
- **A stop disabled or removed while focused applies the successor rule
  lazily**, at the next traversal, landing on the domain's first stop.
  Computing the correct successor *before* the mutation needs the
  materialisation seam, which is T7's; both halves are carried forward
  rather than left implicit.
- **A retained focus id outliving its projection was a reachable crash**,
  found in review and fixed here rather than carried: the id is the
  pre-order index of a projection rebuilt per operation, and a handler
  that removes its own subtree shrinks the projection out from under it.
  The shipped gallery reaches it through the lightbox's close control.
- **Evidence:** integration tests establishing their own initial focus
  state, then asserting the **expected next stop** rather than that
  focus moved; the indicator distinctness frames; the write-site
  enumeration.

- [x] T5

## T6 — DSL: `focus-group`, `modal-scope`, and `dismiss`

The compiler surface, ahead of the runtime behaviour, because the tree
cannot say "group" or "modal" until this lands
([preamble.md §The sequencing thesis](./preamble.md)).

- Checker (constant-only `true` / `false`, admitted on containers),
  loader → the node's focus role.
- **The grammar, the lexer, `lower` and `emit` needed no change at all**
  (measured at the start gate, [log.md](./log.md) §T6 fact 1). Both names
  already lex as a single `Ident` under §2.2's hyphen rule — the one
  `item-cross-size` uses — parse as `property_bind`, lower through the
  generic `Member::PropertyBind` arm to `IrProp { value: IrLiteral::Bool }`
  and emit as `prop focus-group = true`; `dismiss => { … }` already
  parses as a `signal_handler`. The item's original "Grammar, …, IR"
  over-predicted two of its four; the work was checker + loader + role
  derivation + tests. This does **not** generalise to T8's
  `key-down("<key>")`, which still needs the phase's one new production.
- Three rejects with their own tests: a binding-expression RHS, the
  attribute on a non-admitting widget, and a `dismiss` handler on a
  container that does not carry `modal-scope: true` (DD-005: the request
  is addressed to scopes, so a handler elsewhere could never fire).
- **The same rules are gated at the loader as well** (recorded deviation,
  [log.md](./log.md) §T6 start gate). `wasamo_load_ui` admits memory IR
  that never passed through `wasamoc`, the loader must read these props
  anyway to write the annotation, and without the gate the failure is
  *silent* — an annotation on a `Button`, or `prop modal-scope = 1`, is
  dropped with no diagnostic, which is the class T3's CF-2 already
  recorded. The two-gate shape §4.9 / §4.12 / §4.16 already use.
- **Four existing per-kind gates already rejected the new names and had
  to be passed through or relaxed**, none of them enumerable by the
  compiler: `check_zstack_unknown_attr`, `check_scrollview_unknown_attr`
  and `check_grid`'s attribute arm on the checker side, and
  `validate_phase6_zstack_node_invariants` on the loader side. The last
  is the one that mattered — without relaxing it the loader refuses IR
  the checker accepts, which the integration fixture is what notices
  (witness W6).
- **A per-kind signal admission rule does exist, in two places**, so
  §T8's premise below is false for them: `check_grid` rejected *every*
  handler on a `Grid` and the loader rejects every handler on a `ZStack`.
  This task relaxed both for `dismiss` only; widening `clicked` is T8's
  (close gate CF-T6-3).
- `dismiss` is an ordinary signal name in the existing handler table;
  nothing in the IR distinguishes it.
- No new token, `IrType`, `IrLiteral` or `PropertyValue`.
- **No behaviour is built, and the intermediate state that creates is
  named rather than left to be found.** Until T7 adds the entry seam, a
  *present* `modal-scope` subtree is *un-entered* and `tab_stops` skips
  it — the state DD-004 says must not be reachable. It is unreachable
  from any shipped `.ui` (T10 adds the first one, after T7) and is
  carried forward as CF-T6-1.
- **A container carrying both annotations collapses to one role**, since
  `focus_core::FocusRole` is one-of-six. `modal-scope` takes precedence,
  which is asserted rather than incidental; DD-005 records the
  both-at-once case as expressible and untested in M4, and what a
  composite role should mean is T7's (CF-T6-2).
- **Evidence:** accept-side tests — each attribute parses, round-trips
  through the IR, and reaches the loaded node as its focus role — beside
  the reject tests, each firing its branch directly
  (implementation-gates trap 4), on **both** gates; plus the mutation
  witnesses, two of which restore the pre-T6 behaviour rather than break
  the new code.

- [x] T6

## T7 — Group traversal and modal scopes in the runtime

Now that roles have an authored source, the behaviour DD-003 and DD-004
fix:

- Group: one Tab stop, arrows within, **per-group memory with a single
  writer** — the primitive that moves focus is the primitive that writes
  the memory.
- Scope: **presence is the entry** (DD-004). The materialisation seam —
  structural drain or initial build — pushes the scope in
  materialisation order, captures the restore target, and moves focus to
  the scope's first stop (or none, with key delivery starting at the
  scope). Removal exits: **restoration takes precedence over structural
  succession**, and a removal's successor is computed **before** the
  mutation because node identity does not survive a rebuild. The entry's
  focus move writes runtime focus state only and enqueues no further
  drain work — asserted, not assumed.
- Reconcile `focus_core` with presence-entry. **The core's un-entered
  state now has a production constructor** — T6 landed it, so this is a
  real branch with a real input rather than a hypothetical: a `.ui`
  carrying `modal-scope: true` projects as `FocusRole::ModalScope` with
  nothing entered, and `collect_stops` returns early for it, so the
  subtree is reachable by neither Tab nor click-to-focus until the entry
  seam lands (close gate CF-T6-1). The projection either narrows the
  un-entered state away or the branch carries a test that fires it
  (implementation-gates trap 4) — recorded either way.
- **Two landing paths disagree about a group, and the fix goes in one
  primitive** (close gate CF-T6-5). `FocusTree::tab` resolves a group
  landing through `resolve_stop`, so Tab already lands on the group's
  first or remembered member; `focus::focus_on_click` derives its
  landing from `tab_stops` + `nearest_focusable` and never calls it, so a
  click inside a group focuses the **group container**. Making the click
  path agree must go through the same primitive the memory is written
  by, not a second landing resolver.
- **The seam is enumerated before it is trusted**: a start-gate audit
  lists every path that materialises or removes a subtree — initial
  build, conditional drain, `for` regeneration — and shows each runs the
  entry / exit seam, which is the enumeration DD-004's balanced-stack
  argument stands on. If the seam turns out not to be one place, that is
  new information for the plan (a spike or task split), not something a
  local patch absorbs.
- Dismissal: `Escape` becomes a request **addressed** to the innermost
  entered scope and stopping there; the runtime delivers it and never
  acts on it.
- **Retire the spike scaffolding**: `focus_spike`, the `__focus_spike`
  seam, and the override map go, replaced by the real projection.
- **Retiring the override map narrows coverage deliberately**:
  `ActiveItemList` / `ActiveItem` have no authored source in M4, so
  focus / active-item separation falls back to `focus_core`'s unit
  tests — recorded as the intended state rather than silent loss (the
  capacity ships at the pure-logic level, DD-003). Scope nesting is in
  the same position (supported, unexercised by any M4 app — DD-004), so
  its ordering and innermost-addressing keep — or gain — pure-logic
  pins for Phase 9 to inherit.
- **Evidence:** the mechanism fixture, re-pointed at the authored
  annotations; the mutation that deletes the restore branch must go red
  (the spike's M7); an entry test driven by a state write flipping the
  `if` (the production seam, not a test-side call) **and** one driven by
  the initial build — a scope present at startup is entered, which
  DD-004 records as behaviour, so it is asserted rather than implied;
  and the spike's S-3 leg carried over: a **present but unannotated**
  subtree does not confine. T12's control C cannot stand in for that
  leg — its agreement side removes the subtree entirely, so an
  implementation that confines *any* conditional subtree while ignoring
  the annotation would pass it.

- [ ] T7

## T8 — DSL: generic `clicked` and `key-down("<key>")`

`clicked` admitted on any widget. Routing is already generic from T3, so
this widens *who may carry a handler* — a checker rule over the existing
handler table — not how an event travels.

- Button keeps its `enabled` suppression and its keyboard activation;
  both are Button behaviour, documented on Button.
- Reject-side tests pin how far `clicked` widens (DD-005: a previously
  diagnosed `.ui` may now be accepted; the reject tests are what bound
  the widening).
- **`clicked` widens on two kinds and nowhere else** (T3 close gate
  CF-3, corrected by T6 close gate CF-T6-3). For `Box`, the stacks,
  `WrapPanel` and `ScrollView` there is no per-kind signal admission
  rule, so `Box { clicked => … }` has always been accepted, lowered and
  attached, and T3 landed the runtime half — for those kinds the reject
  tests are the whole of the work. **`Grid` and `ZStack` each carry a
  blanket handler rejection, and the two are not symmetric**:
  `check_grid`'s signal arm rejects on the **compiler** side only — the
  loader has no Grid handler gate and never has, so
  `Grid { clicked => … }` is rejected by `check` and *accepted* by
  `wasamo_load_ui` — while `validate_phase6_zstack_node_invariants`
  rejects on the **loader** side only, the checker admitting it. T6
  relaxed each for `dismiss` alone, so this task decides **three**
  things: widen `check_grid`, widen the ZStack loader gate, and whether
  the Grid rule gains the loader half it never had. Leaving any of them
  narrows the authored surface against §4.19's "`clicked` — any widget"
  and hands T13 a divergence
  ([log.md](./log.md) §T6 close gate CF-T6-3).
- **Two Button-family loader defects land here** (T3 close gate CF-1 /
  CF-2), both measured, neither in T3's lane, and both dispositioned by
  the owner on 2026-08-07 ([log.md](./log.md) §T3 owner disposition):
  - **A literal `enabled: false` on a plain `Button` is silently
    dropped** — `ir_loader.rs`'s `"Button"` arm never reads an `enabled`
    prop, unlike its `"ToggleButton"` sibling — so only a state-bound
    `enabled` disables a Button today. Fixed here by reading the prop
    the way the sibling arm does, with a test that drives the literal
    through `.ui` → IR → loader and asserts the widget constructs
    disabled. The checker already has a test that the literal is
    **accepted**; the missing half was a test that it **takes effect**,
    and that pairing is what the new test restores.
  - **A `Button` carrying a `WidgetNode` child becomes a named
    diagnostic.** The shape is accepted by `check`, built by the loader,
    and unknown to layout, so today it renders nothing in release and
    aborts a debug build during `wasamo_load_ui` on T2's `sync_visuals`
    child-count assertion. Rejected here at **both** gates — the checker
    rule plus the loader's re-check, the two-gate shape §4.9 / §4.16
    already use, because `wasamo_load_ui` admits memory IR that never
    passed through `wasamoc`. The direct C path
    (`wasamo_widget_append_child`) stays ungated, matching how `Box`'s
    child-count rule is enforced. [dsl_spec.md
    §4.16](../../../../docs/dsl_spec.md)'s placement example is
    corrected in the same change — it illustrates `slot.*`, and needs no
    Button child to do so. **This narrows the authored surface
    deliberately**, and the capability it withholds is recorded as its
    own row in the [candidate pool](../../../candidate-pool.md) so
    re-opening it is a milestone decision rather than a rediscovery.
- `key-down` needs the phase's **one new grammar production** — a signal
  handler whose name carries an argument. The key name is validated at
  `check` against the recognised non-character table, and an
  unrecognised name is a diagnostic with its own test.
- The keys the runtime keeps (`Tab` always; arrows inside a group;
  `Escape` while a scope is entered) are asserted, since each is a way
  for an authored handler to silently never fire. The group and scope
  behaviours these assertions run against exist from T7.
- **Evidence:** a `Box` with a handler fires; a disabled Button still
  occludes what is behind it; a `key-down("ArrowLeft")` handler fires
  outside a group and does not fire inside one.

- [ ] T8

## T9 — DSL: per-item handlers inside `for`

The four coupled answers M3-Phase 7 routed here:

- **Admission** of a `signal_handler` inside a `for` body.
- **Bare `item` / `index` in handler position**, with accept *and*
  reject tests (the reject: a binder read outside the body).
- **Registration lifecycle** — released with the generated subtree, on
  the path that already releases its bindings. Close artifact: the
  structural side-effect enumeration for subtree removal, listing
  handler registrations beside those bindings. This fails silently in
  one direction, so the enumeration is the check, not a rendered frame.
- **Identity** — a binder resolves at invocation time, so the handler
  belongs to a position. **Evidence must include a click after a
  collection mutation**; a test that only clicks a freshly generated row
  cannot distinguish invocation-time resolution from generation-time
  capture.
- **The M3-Phase 7 drain residuals are dispositioned in DD-001** (they
  do not fire). This task's click-after-mutation evidence is what
  exercises the nearest case, so the disposition is checked against
  something rather than asserted; a divergence goes in
  [log.md](./log.md).

- [ ] T9

## T10 — Gallery slice (consumer A)

Wire the `.ui` end to end through `.ui` → IR → runtime, from at least
one example host:

- Thumbnail click opens the lightbox, carrying which thumbnail.
- The lightbox is a **root `ZStack` branch** and stays one; its scrim is
  an authored covering widget, and it is what blocks background clicks —
  the scope confines the keyboard only.
- Esc closes through an authored `dismiss` handler and Tab is contained.
  **Focus restores to the widget focused before the lightbox opened**
  (or to none, when nothing was). The thumbnail itself becomes the
  restore target when M4-Phase 5's focusability attribute makes a
  clicked thumbnail focusable; that is recorded as Phase 5's consumer,
  not silently claimed here.
- Left/Right step the photo through `key-down` handlers. **The visible
  result is a bound value changing, not the finished picture** — the
  caption and the selected thumbnail need index reads and equality
  selection, which are M4-Phase 3
  ([framing.md](../requirements/framing.md) §範囲の縫い目).
- **Scrolled hit-testing is exercised**, because the gallery is the
  clip rule's consumer: with `scroll_y` non-zero, a thumbnail inside the
  viewport resolves and opens the lightbox, and a toolbar click above
  the viewport resolves to the toolbar — not to the invisible thumbnail
  whose rectangle sits under it.
- **Host artifacts rebuild in order**: the new grammar and attributes
  change `.uic`, so `wasamoc` builds before the C / Zig gallery hosts
  re-embed their IR (the M4-Phase 1 T9 shape); A stays runnable on all
  three hosts.

- [ ] T10

## T11 — Touch

Synthesized injection only; no touch hardware is available (framing
agreement ⑥).

- The path under test is touch message → DIP conversion → hit
  resolution → handler, i.e. that touch rides the same seam as the
  pointer.
- **The injected press fires its handler exactly once.** DD-001 takes
  `WM_POINTER*` rather than mouse promotion, and the OS synthesizes
  mouse messages for pointer input the window does not handle — so the
  discriminating assertion is single delivery through the pointer path,
  not merely "the handler fired", which promotion would also produce.
- `EnableMouseInPointer` is **not** called (DD-001: a library does not
  change its host process's input mode); the mouse stays on the mouse
  messages, and the shared seam is the DIP conversion.
- **Where it runs is probed, not assumed.** Pointer injection needs
  capabilities a CI runner may lack, so a feasibility probe on the dev
  box and on CI comes before the assertions are written. On CI the test
  follows the standing rule: it **fails rather than silently skips**
  when the capability is missing, and the skip guard is verified on an
  environment that actually lacks it
  ([CLAUDE.md §Testing rules](../../../../CLAUDE.md)). If the probe
  finds injection infeasible everywhere, the fallback — posting
  `WM_POINTER*` frames directly, which does not exercise the OS pointer
  machinery — is a **weaker claim**, and swapping to it is an
  owner-visible plan change (framing agreement ⑥ named the evidence
  form), not a silent substitution.
- **The limit is stated, not implied**: this does not establish that a
  physical touch digitizer produces the same messages. Recorded in the
  same shape as Phase 1's synthesized-`WM_DPICHANGED` limit.
- Confirm whether
  [verification-environments.md](../../../../docs/notes/verification-environments.md)
  needs a taxonomy entry for synthesized touch, per the framing's
  verification section.

- [ ] T11

## T12 — GUI evidence with positive controls

The four controls from the [framing](../requirements/framing.md)
§検証方針, each with its agreement leg:

| Control | Difference leg | Agreement leg |
|---|---|---|
| A — click routing and item identity | clicking thumbnail N and thumbnail M give different lightbox content | clicking N twice gives the same content |
| B — traversal order | Tab ×1 / ×2 / ×3 and Shift+Tab reach the expected stops in reverse | two frames with no input agree within the measured text-pixel jitter (F-33: up to 13/channel), with the tolerance and comparison recorded — not asserted as bit-identical |
| C — containment and occlusion | with the lightbox open, a background click does nothing and Tab cycles inside | with it closed, the same coordinate fires **and the same Tab reaches the background** (the DD-004 agreement leg — containment distinguished from an empty background) |

- **Control C also discharges T4's CF-T4-5** (owner-settled 2026-08-07,
  [log.md](./log.md) §Owner disposition of CF-T4-5). T2 resolved
  hit-testing to a single target without taking a gallery frame, because
  the phase predicted the gallery had no reachable overlap; T4 measured
  that prediction false. T2 is not reopened — its rule is pinned by
  pure-logic tests over a constructed overlapping tree, which bounds the
  rule rather than one instance of it — and the gallery frame it did not
  take is taken **here**, once, in exactly the shape control C already
  has. Recorded on this row so the obligation is visible where it is
  executed.
| D — Esc | Esc closes the lightbox | an unrelated key does not |

- Capture is preceded by `cargo build --release --workspace`, takes
  **multiple frames on each side**, uses the **client** rectangle, and
  states the display scale (Phase 1 F-21 / F-33 / T10).
- The capture width keeps the toolbar's content within the client, or
  names the known width-driven overlap as a known observation
  ([constraints.md §6](../requirements/constraints.md)).
- **At least one control repeats at a display scale ≠ 100%** (A or C):
  a wrong pointer conversion is invisible at 100%
  ([preamble.md §What "green" is worth](./preamble.md)), so a capture
  set taken only at 100% cannot distinguish the migrated input path
  from a broken one. Every capture states its scale either way.
- Any tool that reads window geometry or cursor position declares
  Per-Monitor-Aware V2 first (Phase 1 F-48).
- **Control B is Tab-driven, so its capture acquires foreground
  activation before sending a key, verifies it, and retries** — keyboard
  input goes to the focused window of the foreground thread, unlike the
  cursor-routed input every earlier control used
  ([verification-environments.md](../../../../docs/notes/verification-environments.md)
  Observation 4). Each capture **records which input path it used**: real
  key presses, or posted `WM_KEYDOWN`, which is the weaker claim. The
  numbers look identical either way, so a silent fallback would be
  invisible in the artifact. T5's
  [capture-t5-focus.ps1](./evidence/capture-t5-focus.ps1) is the working
  shape.
- **The owner smoke guide is written out here** (framing §オーナー目視)
  and verified against the target commit before it is used.

- [ ] T12

## T13 — Close gate

- Local clean rebuild and full suite in the evidence profile; CI run id
  recorded at phase end (the split of ownership Phase 1 carried: the
  step that changed code owns the local rebuild, the phase end owns the
  CI id).
- **Moment 2 implementation sync** — re-verify
  [dsl_spec.md §4.19](../../../../docs/dsl_spec.md) and
  [architecture.md §13](../../../../docs/architecture.md) against the
  landed runtime and flip their phase-status markers; record any
  divergence rather than silently correcting it. The named check items,
  so the re-verification is a list and not a re-read:
  - the disabled-Button contract in
    [dsl_spec.md §4.8](../../../../docs/dsl_spec.md) agrees with §4.19's
    occlusion rule (a disabled Button occludes; the "child hit-test
    traversal is preserved" reading must not survive);
  - §4.8's M4-deferred items that this phase settled are stated where
    they land: disabled widgets are skipped in traversal (the spike's
    `a_disabled_stop_is_skipped`);
  - §4.8's disabled contract states that a disabled Button is skipped by
    Tab (DD-003 discharges that deferral) alongside its occlusion
    behaviour, and §4.19's focus section says the same;
  - §4.19's `for` example parses under §4.15's grammar;
  - §4.15's per-item handler text — admission, invocation-time binder
    reads, position-not-item identity, registration released with the
    generated subtree — matches what T9's click-after-mutation evidence
    landed;
  - the recognised key table in §4.19 matches the checker's table, and
    §4.19 records that an unconsumed key reaches the default window
    procedure;
  - §4.19's `dismiss` admission rule (only beside `modal-scope: true`)
    and its keys-the-runtime-keeps table match the landed checker and
    runtime behaviour;
  - **§13.3's focus-indicator sentence matches the landed runtime.** Its
    second half ("not a visual written at focus-change time") is
    satisfied literally — no `Visual` is created and the runtime still
    has exactly six `SetOffset` / `SetSize` calls, all inside
    `sync_visuals`. Its first half ("applied by the same pass that writes
    visual geometry") is not implementable: that pass runs only from
    layout, and a Tab press triggers none, so an indicator applied there
    would not appear until something else re-laid the tree out. The
    landed indicator is presentation state on the node applied by the
    same means hover and pressed use, which is what DD-003's own
    "the same shape as Button hover / pressed" says. Owner-settled
    2026-08-07: correct the wording here (T5 close gate CF-T5-6);
  - the entry rule (presence is the entry; focus moves in) and the clip
    bound in §13 match the landed runtime;
  - **no fixture spelling appears in `docs/dsl_spec.md`** (DD-005 /
    framing R2).
- Phase-end retrospective, verification closure mapping, and
  [handoff.md](./handoff.md).

- [ ] T13

---

## Cross-task obligations

- **The stretch re-evaluation checkpoint** ([plan.md](../../plan.md)
  §Cross-phase dispositions 3) is discharged: owner re-read 2026-08-05,
  **both stretch intakes retained** — `Image` / direct-value `fill` stay
  at M4-Phase 4, multi-line text editing stays ride-if-room. No
  disposition line to the candidate pool or M5 is owed.
- **No new ABI function** (framing agreement ⑦, a hypothesis rather than
  a constraint). If a task finds it needs one, it proposes the tier-2
  plan revision before adding it.
- **Every task that measures something re-reads the whole task list at
  its close gate**, not only the item it was assigned (the re-audit
  discipline; recurred at M4-Phase 1 T2).
