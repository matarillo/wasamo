---
phase: M4-Phase 2
title: Implementation task list
status: closing
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
  which is asserted rather than incidental, and `wasamoc` **warns** that
  the `focus-group` half has no effect — the shape stays accepted, so the
  surface DD-005 chose is not narrowed, but the author is told rather
  than left to find out. Whether the combination should be supported at
  all, or the two booleans should become one enumerated attribute, is a
  surface question **no M4-Phase 2 task owns**; it goes to the
  [candidate pool](../../../candidate-pool.md) (owner-settled
  2026-08-07, CF-T6-2).
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
- Scope: **presence is the entry** (DD-004). The materialisation seam
  pushes the scope, captures the restore target, and moves focus to the
  scope's first stop (or none, with key delivery starting at the scope).
  Removal exits: **restoration takes precedence over structural
  succession**. The entry's focus move writes runtime focus state only
  and enqueues no further drain work — a `debug_assert_eq!` over the
  queue and dirty-set lengths around the entry step, not a comment.
- **What has to happen before the mutation is the capture, not the
  succession** (measured at the start gate, [log.md](./log.md) §T7 fact
  5). `focus_core::focus_after_removing`'s structural succession is the
  domain's *first surviving stop*, which is exactly what T5's lazy
  landing already produced — so it is derivable from the post-mutation
  tree. The half that genuinely cannot be recovered afterwards is the
  restore target, and that is captured at entry and held on the scope's
  stack entry. CF-T5-2's observable content is therefore restoration
  precedence alone, and the seam runs **after** the mutation without
  weakening DD-004.
- **The retained record needs a coordinate system that survives a
  structural mutation** — the precondition this item did not name. A
  `FocusId` is the pre-order index of a projection rebuilt per
  operation, and a modal stack entry is the longest-lived retained id in
  the runtime, so entry / exit built on raw ids would be built on a key
  that silently renames nodes. `FocusProjection` carries an anchor
  (node address) per id, `FocusState::remap` re-expresses every id-keyed
  store, and `WindowFocus::rebase` writes the ids and their coordinate
  system together. That closes CF-T5-1's in-range half, which T5 recorded
  as unbuildable and which now has a fixture.
- Reconcile `focus_core` with presence-entry. **Resolved: the branch
  keeps its test rather than being narrowed away by the projection.**
  `collect_stops`'s `ModalScope if !is_entered` arm stays reachable in
  principle — through the direct-ABI child mutators, which reach no seam
  (DD-002's "outside the layout boundary") — and is fired by
  `focus_core`'s own `an_unentered_modal_scope_is_not_reachable_by_tab`.
  Through every seam-running path, present implies entered.
- **Two landing paths disagree about a group** (close gate CF-T6-5), and
  **`resolve_stop` is not the fix** (start gate fact 8): it returns the
  group's *remembered* member, which is right for Tab and wrong for a
  click — clicking "Favorites" must focus "Favorites". The landing rule a
  click needs is per-node, so `focus_core::FocusTree::focus_landing`
  lands it beside `collect_stops`, the two differing in exactly one place
  — a group's members — which is the rule rather than drift.
  `focus::nearest_focusable` is deleted; the landing still reaches focus
  through the primitive that writes the memory, so no second resolver
  exists.
- **The seam was enumerated before it was trusted, and it is one hop
  later than this item predicted.** It is the layout-invalidation seam:
  `window::set_root` for the initial build, and `emit::flush_layout` —
  drain Phase 2 — for every reactive structural mutation, which all
  reach it through `mark_layout_dirty_for`. It cannot sit at
  `insert_structural_child` / `remove_structural_child`, because those
  run inside `Signal::set`'s synchronous drain, which for a click is
  inside `hit_test_click`, which holds `&mut WidgetNode` on the window
  root; forming `&mut WindowState` there would alias it. The four
  direct-ABI child mutators are outside the seam, which is DD-004's own
  recorded residual rather than a new one.
- **One primitive writes the focus record and the painted indicator.**
  Entry and arrow movement are two further writers of the focused id, so
  `move_focus` becomes a wrapper over `with_focus_write`, which snapshots
  the previous id, performs an arbitrary write on the core, and repaints
  the transition. `set_button_focused_at` still has exactly one caller.
- Dismissal: `Escape` becomes a request **addressed** to the innermost
  entered scope and stopping there; the runtime delivers it and never
  acts on it. It is consumed whether or not the scope carries a
  `dismiss` handler — writing none is how a scope declines to close — and
  falls through to the host key slot when no scope is entered. Delivery
  reuses the `clicked` dispatcher's snapshot-then-run split rather than
  adding a second one.
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
  annotations and driven through real window messages, keeping its
  read-every-result-back-as-a-Button-label discipline and narrowed to the
  one shape no other fixture has — a group **and** a scope in one tree;
  the mutation that deletes the restore branch goes red on exactly the
  two fixtures that claim restoration (the spike's M7); an entry test
  driven by a state write flipping the `if` (the production seam, not a
  test-side call) **and** one driven by the initial build — a scope
  present at startup is entered, which DD-004 records as behaviour, so it
  is asserted rather than implied; and the spike's S-3 leg carried over:
  a **present but unannotated** subtree does not confine. T12's control C
  cannot stand in for that leg — its agreement side removes the subtree
  entirely, so an implementation that confines *any* conditional subtree
  while ignoring the annotation would pass it.
- **A frame pair is taken here rather than deferred to T12**, because no
  state read-back can show that the indicator *paints* on a node the same
  drain created moments earlier: `__button_focused_for_test` reports the
  same boolean whatever colour the brush reaches (T5's CF-T5-3). Two
  builds of the same tree, annotated and not, with the scope's first stop
  as the difference leg and a second, never-focused Button as the
  agreement leg. The gallery `.ui` carries the annotation only as a
  **throwaway probe** — landing it is T10's.
- **The mechanism fixture's fourth test is deleted rather than
  re-pointed.** It asserted that without annotations every Button is its
  own stop, which measured the gap DD-005 had to close; the gap is
  closed and there is no unannotated variant of that tree left to build.
  The property it guarded is the S-3 leg above.

- [x] T7

## T8 — DSL: generic `clicked` and `key-down("<key>")`

`clicked` admitted on any widget. Routing is already generic from T3, so
this widens *who may carry a handler* — a checker rule over the existing
handler table — not how an event travels.

- **That framing is true of `clicked` and false of `key-down`** (measured
  at the start gate, [log.md](./log.md) §T8). `key-down` is a five-layer
  addition ending in the runtime: the member-dispatch production and an
  AST field; the checker's recognised-key table and three rejects; an IR
  field, its emission and its loader parse; the loader's second gate; and
  **the key walk itself**, which T5 deferred here in as many words ("T5
  lands the consumption half and the fallthrough half; T8 lands the
  dispatch between them"). The **review lane is therefore corrected from
  the branch/test-focused review [preamble.md](./preamble.md) predicted to
  a full independent review** — runtime structural change plus a schema /
  IR migration, the Phase 1 F-12 / T12 precedent for correcting a stale
  lane at the start gate.

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
  **The three questions resolved as one answer: both rules are removed
  rather than widened**, so per-kind signal admission ceases to exist and
  admission is by signal name alone. That answers the third question with
  *no* — no per-kind handler rule is left for either gate to hold.
  Widening instead would keep two allow-lists that the next signal name
  has to be added to twice, which is the drift CF-T6-3 records. Two
  consequences are named rather than left to be found:
  `Grid { totally_unknown => … }` becomes accepted, which is what the
  other ten kinds already did (the uniform question — whether an
  unrecognised *signal* name should be a diagnostic anywhere — is
  CF-T8-4, not this task's); and `ZStack { clicked => … }` becomes
  accepted at the loader, which is what §4.19's table requires.
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
    **The defect spans four widget kinds, and all four are rejected**
    (start gate fact 4; owner disposition 2026-08-08):
    `build_layout_tree` maps `Rectangle`, `Text`, `Button` and
    `ToggleButton` alike to a childless `LayoutNode::rectangle`, and
    `check` accepted a widget child on all four — `Text { Button {} }`
    was measured aborting a debug build on the same assertion the Button
    shape does. **The rule is built so a later phase can re-open any of
    these kinds in one edit**: the four are named only in
    `wasamo_ir::LAYOUT_CHILDLESS_WIDGET_KINDS`, both gates read
    `layout_treats_as_childless`, the const's doc carries the re-opening
    recipe, and `build_layout_tree`'s childless arm points back at it.
    Measured rather than claimed — removing two entries reddens exactly
    the four per-kind reject tests across both gates and nothing else.
- `key-down` needs the phase's **one new grammar production** — a signal
  handler whose name carries an argument. The key name is validated at
  `check` against the recognised non-character table, and an
  unrecognised name is a diagnostic with its own test.
  - **Three rules, not one, and both gates carry all three**: a bare
    `key-down` with no argument, an unrecognised key name, and an
    argument on a signal that takes none. All three are the
    silently-never-fires class, and a bare `key-down` parsed and was
    accepted before this task (start gate fact 2).
  - **The 22 recognised names get one owner**, `wasamo-ir`, read by both
    the checker and the runtime's virtual-key map, with a sweep test
    asserting every name is producible — a name the checker accepts with
    no virtual-key mapping is a handler that can never fire. The
    handler's canonical storage spelling gets the same treatment
    (`wasamo_ir::signal_key`, composed nowhere else).
  - **The runtime half is the key walk**, a fourth consumption arm in
    `WM_KEYDOWN` between `dismiss_on_key` and the host key slot,
    reusing `hit::dispatch_chain` and `run_signal_handlers` rather than
    adding a second dispatcher. It projects and rebases like the other
    three `*_on_key` functions rather than calling `focused_path`, which
    cannot rebase — so this task does **not** become the first production
    reader CF-T7-2 names as its re-trigger.
  - **The walk starts at the focused widget, or at `traversal_root`
    when nothing is focused** — §4.19's "or, when nothing is focused, at
    the innermost modal focus scope", which is that function's own
    definition. The walk is upward-only, so a handler below the start can
    never fire; carried to T10 as CF-T8-5.
  - **A disabled Button-family node suppresses its own `key-down` and
    does not consume**, the same disposition §4.8 gives a click. This
    item first shipped the opposite, on the false reasoning that the case
    was unreachable — true of the `focus.focused()` start, false of the
    `traversal_root` one, because `collect_stops` never gates the tree
    root itself. **The independent review found it and the lead
    re-measured it** with a probe: a disabled root Button's own
    `key-down("Enter")` handler ran ([log.md](./log.md) §T8 review F1).
  - **Adding the `(` route collided with a shared sub-parser.**
    `parse_grid_track_list`'s word-continuation lookahead absorbed
    `key-down(` as a trailing track word; the stop set gained `LParen`
    and is pinned at that sub-parser's own layer with both legs.
- The keys the runtime keeps (`Tab` always; arrows inside a group;
  `Escape` while a scope is entered) are asserted, since each is a way
  for an authored handler to silently never fire. The group and scope
  behaviours these assertions run against exist from T7 — **and so does
  the tripwire**: `arrow_keys_two_legs` and `escapes_two_legs` each
  assert the consumption leg *and* the fallthrough leg through the host
  key slot, so a `key-down` dispatch inserted on the wrong side of that
  slot breaks a named test rather than silently swallowing the key
  (CF-T5-5, armed).
- **Evidence:** a `Box` with a handler fires; a disabled Button still
  occludes what is behind it; a `key-down("ArrowLeft")` handler fires
  outside a group and does not fire inside one.
  - **A fourth piece was added after a mutation measured the third to be
    unpinned in one direction.** Removing the consuming arm's early
    `return` left the whole suite green: every key fixture reads handler
    *effects*, so none of them constrained where the arm sits relative to
    the host key slot. `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot`
    asserts both legs against the slot's recorder and goes red under that
    mutation ([log.md](./log.md) §T8 close gate W12).
  - **A fifth was added at the independent review**, for the same reason
    in a different place: `wasamo_ir::signal_key`'s *argument* was not
    observable at the behaviour level at all. Both the loader and the
    dispatcher compose the storage key through that one function, so
    dropping the argument symmetrically reddened only its own unit test
    and nothing else in 1,201 tests. Two `key-down` handlers for
    different keys on one node is the smallest shape where the argument
    has to survive ([log.md](./log.md) §T8 review W9). **The single-owner
    design that prevents drift is also what hides a symmetric error** —
    a shape worth carrying to any later task that gives two sides one
    shared encoder.
- **Button keyboard activation does not exist and is not built here**
  (start gate fact 6). This item's "Button keeps … its keyboard
  activation" presupposed a behaviour the runtime has never had —
  `rg "VK_RETURN|VK_SPACE"` over the runtime returns nothing and
  `run_clicked_handlers` has one caller. Building it would put `Enter`
  and `Space` into §4.19's keys-the-runtime-keeps table, which does not
  list them, so an authored `key-down("Enter")` would silently never fire
  while a Button is focused. Recorded as CF-T8-1 for an owner
  disposition of the same kind CF-1 / CF-2 received.

- [x] T8

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
  **The enumeration measured that a handler has no separate lifecycle to
  release at all** — an inline body is owned data on the node and is
  freed with it — so the only releasable thing a `for`-body child can
  now hold is a host `wasamo_signal_connect` token, which
  `widget_destroy` already severs. That is what the fixture drives.
- **Identity** — a binder resolves at invocation time, so the handler
  belongs to a position. **Evidence must include a click after a
  collection mutation**; a test that only clicks a freshly generated row
  cannot distinguish invocation-time resolution from generation-time
  capture. **The discriminator is a same-length whole-value reset**,
  which dsl_spec §4.15 says makes no structural edit at all, so the rows
  under test are provably the original ones (pinned by pointer identity
  in the fixture, not assumed). It was measured rather than asserted: a
  working generation-time-capture implementation reddens that fixture
  **and nothing else in 1,244 tests** ([log.md](./log.md) §T9 close gate,
  witness W-F).
- **"The phase's only new IR content" was false as written, and the
  review lane is corrected on the other ground** (start gate facts 2 and
  3). `lower` already threaded the loop context into handler bodies,
  `ItemRead` / `IndexRead` already existed, `emit` already wrote them,
  and the IR text parser is shared between bindings and handlers — so
  **no IR type, no grammar production and no lowering change** was
  needed. What was outstanding is the half DD-005 names beside it: the
  runtime's handler evaluation context. The lane stays **full
  independent review** on the runtime structural trigger — a new retained
  field on every `WidgetNode`, a change to the one snapshot all three
  signal dispatchers share, and new arms in the evaluator.
- **The scope rule is the only new authored rule; there is no type
  rule.** Handler-body assignments are not type-checked at all today
  (`root.n = "abc"` on an `i32` state passes `check` with exit 0), so a
  type rule for binder reads alone would make handler position stricter
  for a binder than for a literal. The consequence is recorded as
  CF-T9-2: a scalar `string` state cannot be written from a handler at
  all, for any right-hand side.
- **The binder read had to reach three evaluators, not one** (found by
  probe, closed here). Only `i32` collection appends go through
  `evaluate`; a `string[]` append goes through `evaluate_binding` and
  `evaluate_binding_part`, a `bool[]` append through
  `evaluate_bool_assignment_value`. `labels = labels.append(label)`,
  `labels = labels.append("row \{i}")` and `flags = flags.append(f)`
  were accepted by both gates and could only ever log at click time. All
  three now read their binder, each with the same out-of-range boundary.
- **The loader had a two-gate divergence this task created**, measured
  red before it was fixed: `validate_collection_element_expr` passed
  `None` for the loop scope, which was correct only while no writable
  expression could sit inside a `for` body, so `xs = xs.append(item)` was
  accepted by `wasamoc check` and rejected by the loader.
- **Two loader test premises were falsified rather than left to survive
  silently.** The `dismiss` gate's `ControlFlow::For` arm was documented
  as unreachable through `parse_ir` because the handler rejection
  short-circuited ahead of it; admitting the handler makes it reachable,
  and the bare-`key-down` gate with it. Both are now accept-and-reject
  pairs.
- **The focus record's anchors are node addresses, and `for` regeneration
  is where an address can be reused** (T7 close gate CF-T7-1). Freeing a
  subtree and allocating a new one inside the same drain is the nearest
  shape that could hand a retained anchor an address belonging to a
  different node. Bounded — a wrong focus target, never an unsound read,
  since nothing dereferences an anchor — and narrow, because the seam
  rebases at the end of that same drain. This task is where the shape
  exists, so it is where the residual is checked rather than assumed.
  **Checked, and the collision was not reached**: a fixture focuses the
  last generated row and runs `xs.drop-last()` then `xs.append(9)` in one
  handler body, and the run records the freed row and the row allocated
  in the same message at *different* addresses. CF-T7-1 is therefore
  **narrowed, not closed** — M4-Phase 2's nearest expressible shape does
  not reproduce the reuse — and the fixture is retained as its tripwire,
  asserting per arm so a collision becomes a named observation rather
  than an allocator-dependent red (CF-T9-1).
- **The M3-Phase 7 drain residuals are dispositioned in DD-001** (they
  do not fire). This task's click-after-mutation evidence is what
  exercises the nearest case, so the disposition is checked against
  something rather than asserted; a divergence goes in
  [log.md](./log.md). **There is one, and it is in the reason rather than
  the result.** DD-001 says the case is safe because "the handler has
  already returned when regeneration runs"; the fixture measures the
  clicked row's own subtree already destroyed *and* the next statement
  already failed, from one synchronous message — regeneration runs
  **during** the handler. The conclusion (no cycle: regeneration
  re-invokes no handler) is unaffected, so this is the "explanation
  narrows" case, carried to T13 for the owner rather than settled from a
  task close gate.
- **The `for`-body handler rejection is intact on both gates** — T8
  touched neither `check_members_inner`'s `inside_for_template` arm nor
  `validate_node_references_in_scope`'s, so admission is still wholly
  this task's. What T8 changes for it: `Member::SignalHandler` and
  `IrHandler` now carry `arg`, so the loop scope travels beside an
  existing optional field; and `wasamo_ir::signal_key` is the function a
  per-item `key-down` would compose its storage key through, if
  admission reaches that far. **Both held**: `arg` needed no special
  handling on the loop-scope path, and the loop scope is a *separate*
  field on the node rather than something baked into the storage key, so
  no second composer appeared.

- [x] T9

## T10 — Gallery slice (consumer A)

Wire the `.ui` end to end through `.ui` → IR → runtime, from at least
one example host:

- Thumbnail click opens the lightbox, carrying which thumbnail.
  **It carries an index, not a label** (T9 close gate CF-T9-2): a handler
  cannot write a scalar `string` state at all — there is no `set_string`
  in the runtime for any right-hand side — so the per-item handler writes
  `root.selected_index = index;`, which is §4.19's own example shape and
  is what T9 landed end to end.
  - **The click resolves to the thumbnail's `Text`, not to the `Box` that
    carries the handler**, so the shipped app is the gallery's first
    production exercise of T3's ancestor walk rather than of target
    dispatch alone. Measured, not inferred (fixture G1).
- The lightbox is a **root `ZStack` branch** and stays one; a covering
  widget inside the scope blocks background clicks — the scope confines
  the keyboard only.
  - **Which covering widget was wrong in this item and is now measured.**
    It said the scrim blocks them. At a background point the resolver
    returns the lightbox's own `Grid`: it is declared after the scrim and
    is also stretch/stretch, so it wins the reverse-order walk, and the
    scrim never gets the chance. Both are inside the scope subtree, so
    the blocking behaviour §4.19 describes holds either way — but T12's
    control C rests on this, so it is recorded as measured rather than
    left as the prediction it was (fixture G5).
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
  **The key walk is upward-only** (T8 close gate CF-T8-5), so those
  handlers must sit **at or above** whatever the scope's entry focuses.
  On the lightbox's `modal-scope` container they do: entry moves focus to
  the scope's first stop, a descendant, so the walk reaches them. That is
  a property of entry rather than an accident, and moving the handlers
  below the focused stop would make them unreachable.
  - **The `<` / `>` Buttons gained `clicked` handlers too**, which this
    item did not ask for. They are the visible prev/next affordance and
    were inert; leaving them inert while the arrow keys worked would fail
    the owner's smoke for a reason that has nothing to do with the phase.
    They use only the M3 Button surface, and the two routes are asserted
    to reach the same bound value.
  - **The `x` close Button is fired too** (owner-settled 2026-08-08). It
    predates this task and was therefore outside trap #4, but it is the
    lightbox's other authored closing route and the only one with no test;
    the line drawn around it cost more to defend than to cross.
- **Scrolled hit-testing is exercised**, because the gallery is the
  clip rule's consumer: with `scroll_y` non-zero, a thumbnail inside the
  viewport resolves and opens the lightbox, and a toolbar click above
  the viewport resolves to the toolbar — not to the invisible thumbnail
  whose rectangle sits under it.
- **Host artifacts rebuild in order**: the new grammar and attributes
  change `.uic`, so `wasamoc` builds before the C / Zig gallery hosts
  re-embed their IR (the M4-Phase 1 T9 shape); A stays runnable on all
  three hosts. **No `.uic` is tracked**, so this is a build-order
  obligation with nothing to keep in step in the repository.
- **The fixtures drive the shipped file itself.**
  `gallery_slice_integration.rs` reads `examples/gallery/gallery.ui` with
  `include_str!` rather than building a gallery-shaped miniature, so what
  is under test is the artifact the hosts embed. It finds nodes by label
  and rendered text, so a later `.ui` edit reddens a named assertion
  instead of silently measuring a different node.
- **This task writes production Rust after all**, which its start gate
  forbade and the lead re-decided mid-task:
  `WidgetNode::__resolve_topmost_for_test` forwards to the production
  `hit::resolve_topmost`, because `ffi::__hover_target_for_test` reports
  only *enabled Button-family* targets and so cannot witness a resolved
  `Box` / `Text` / `Grid` at all. It is what turned this item's own
  scrim claim from a prediction into a measurement.
- **The gallery's tab strip is one Tab stop** (`focus-group: true` on the
  toolbar-left `HStack`, owner-settled 2026-08-08). This item did not ask
  for it and the attribute would otherwise have shipped with no `.ui`
  carrying it, which §4.19's own `focus-group` example — literally these
  three tab `ToggleButton`s — makes look like an oversight.
- **`selected_index` is unclamped at both ends** because this phase has no
  conditional expression to guard with; §4.19's own example has the same
  shape. Ships as is (owner-settled), carried to M4-Phase 3 with the two
  questions that phase must answer.
- **The known toolbar overlap has an input-side half nobody had
  measured**: where the toolbar `HStack`s overflow their `Grid` columns,
  every tab `ToggleButton` becomes unclickable, because a non-clipping
  container is itself a hit target across its whole rectangle. Recorded
  as a finding with M4-Phase 4 (which owns the overflow semantics per
  [constraints.md §6](../requirements/constraints.md)) and pinned by a
  fixture that says, in its own failure message, that a red there means
  the overflow was fixed.

- [x] T10

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
  - **"The shared seam is the DIP conversion" was true of the division
    and false of the space** (start gate fact 3). The `WM_POINTER*`
    family carries **screen** coordinates where the mouse family carries
    client coordinates, so the arm adds a `ScreenToClient` translation
    ahead of the shared division — a new conversion site in the class
    [architecture.md §12](../../../../docs/architecture.md) keeps
    enumerable. It is invisible at a window position of `(0,0)`, so the
    fixture parks its window off the desktop origin and asserts the move.
    The **review lane is correspondingly corrected** from the
    branch/test-focused review [preamble.md](./preamble.md) predicted to
    a full independent review.
- **Where it runs was probed, and the probe reshaped the task.** Both
  injection APIs work on the development desktop, so the plan's
  "infeasible everywhere" fallback does not fire. What the probe found
  instead is that injection is **desktop-scoped**: the contact goes to
  whatever window is at the screen point, so an injection-driven cargo
  test would depend on its own window being visible, foreground and
  unobstructed — the *GUI / interactive* environment class, not the
  *headless runtime with live Compositor* class the integration suite
  runs in. The two halves of the claim therefore land in the two tiers
  that can carry them, which is **not** the plan's fallback swap
  (injection is used, not replaced):
  - a **CI-gated message-level fixture** (`touch_pointer_integration.rs`)
    for conversion, resolution, dispatch, focus and hover — labelled in
    its own header as the weaker claim, because a `SendMessageW`-borne
    pointer message carries no real pointer id;
  - a **desktop-tier injection artifact** for the OS-level claim, which
    the CI-gated tier provably cannot make: mutation witness W2 removed
    the suppression and left the whole suite green.
  - The injection half is **deliberately outside the CI gate**
    (owner-settled 2026-08-09: not needed for now, revisited when it is —
    a reservation, not a closed question). Adding it later is additive and
    needs a GitHub Actions capability probe, which needs a push.
- **The single-delivery rule is measured, and it is per contact rather
  than per message.** Claiming `WM_POINTERDOWN` **or** `WM_POINTERUP`
  suppresses the whole contact's promotion, including the `WM_MOUSEMOVE`
  an unclaimed `WM_POINTERUPDATE` would otherwise produce on a moving
  contact; claiming only `WM_POINTERENTER` or only `WM_POINTERLEAVE`
  suppresses nothing. All five are claimed anyway, so no member of a
  contact the runtime has taken responsibility for reaches
  `DefWindowProc`.
- **Two behaviours no normative text fixes are decided here**, rather
  than inherited by omission (CF-T4-4 and the T5 / T7 / T10 focus line):
  a touch contact **moves focus exactly as a click does**, and it
  **writes no hover or pressed state** — with the limit stated, that a
  touch user gets no press feedback in M4.
- **Only the primary contact activates a widget** (found at the
  independent review). The arm read no `wParam`, so two fingers landing
  on one Button produced two dispatches for one perceived tap. A
  non-primary contact is still *claimed* — claiming must not become
  contact-dependent or promotion returns — and simply dispatches nothing.
  Multi-contact gestures stay outside M4-Phase 2.
- **The limit is stated, not implied**: this does not establish that a
  physical touch digitizer produces the same messages. Recorded in the
  same shape as Phase 1's synthesized-`WM_DPICHANGED` limit.
- The taxonomy entry
  ([verification-environments.md](../../../../docs/notes/verification-environments.md))
  **was needed**, and the answer is a measurement rather than a
  judgement: synthesized touch injection gets its own row and an
  observation recording the desktop-scoped requirement, the two-tier
  split, and the mechanics that fail silently.

- [x] T11

## T12 — GUI evidence with positive controls

The four controls from the [framing](../requirements/framing.md)
§検証方針, each with its agreement leg:

| Control | Difference leg | Agreement leg |
|---|---|---|
| A — click routing and item identity | clicking thumbnail N and thumbnail M give different lightbox content | clicking N twice gives the same content |
| B — traversal order | Tab ×1 / ×2 / ×3 and Shift+Tab reach the expected stops in reverse | two frames with no input agree within the measured text-pixel jitter (F-33: up to 13/channel), with the tolerance and comparison recorded — not asserted as bit-identical |
| C — containment and occlusion | with the lightbox open, a background click does nothing and Tab cycles inside | with it closed, the same coordinate fires **and the same Tab reaches the background** (the DD-004 agreement leg — containment distinguished from an empty background) |
| D — Esc | Esc closes the lightbox | an unrelated key does not |

**All four are taken in one sitting** — one `cargo build --release
--workspace`, one launch, one window geometry, one measured scale —
by [capture-t12-controls.ps1](./evidence/capture-t12-controls.ps1),
with the frames and their reading in
[evidence/t12-frames/](./evidence/t12-frames/). The order is forced
rather than chosen: B first, because its baseline needs *nothing
focused* and only a fresh launch supplies that; C last, because it
leaves a different tab checked.

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
- **Control D's "unrelated key" leg uses a *recognised* key with no
  handler** (T8 close gate re-audit). Before T8 an unrelated key had no
  authored path to fire on at all, so the leg could not have
  discriminated; now it can, and the state-level equivalent is pinned by
  `the_authored_key_down_walk_consumes_ahead_of_the_host_key_slot`. The
  key is `Home`: one of `wasamo_ir::RECOGNISED_KEY_NAMES`' 22 entries,
  authored nowhere in the gallery, and absent from §4.19's
  keys-the-runtime-keeps table. `Enter` was avoided deliberately —
  whether Button keyboard activation should exist is CF-T8-1, open for
  T13, and a leg standing on an open question is not a leg.

- **Control A was re-taken rather than cited, and the ground is not
  staleness.** T10 took it first
  ([evidence/t10-frames/](./evidence/t10-frames/)) and the runtime has
  not moved on any path it exercises since — the whole diff is T11's
  `WM_POINTER*` arms, which no mouse or key path passes through — so
  citing was available. Re-taking buys the one thing citing cannot: the
  four controls share a window, so they are mutually comparable and one
  `-Compare` run reads the whole artifact. T10's set stands beside this
  one as an independent earlier sitting, and the two agree.
- **Control C's blocker is the lightbox's own `Grid`, not the scrim**
  (T10 close gate, measured through `__resolve_topmost_for_test`). The
  `Grid` is declared after the scrim and is also stretch/stretch, so it
  wins the reverse-order walk at a background point. Both are inside the
  scope subtree, so containment holds either way and the control is
  unchanged — but the row's *description* should not say "the scrim
  blocks it", because that is not what the runtime does.
- **Control C's containment leg needed a sensor, and the sensor changed
  how the leg is judged.** "The toolbar did not change while the scope
  was entered" is a no-change claim about a region seen *through* the
  scrim, and a no-change claim is free if the instrument cannot see the
  region. Measured: the scrim (`fill: #101820cc`, alpha 0.8) leaves
  `px_differing_at_all` untouched — 2608 px either way for the same
  state change — and divides `max_channel` from **157 to 31**, a 5.06×
  attenuation against the 5 its own alpha predicts. That is below the
  60-summed visible-change bar every other leg uses, so the two
  lightbox-open toolbar-band legs are judged on `px_differing_at_all`.
  **This is a tightening** — the agreement bar moves from "under 40 px
  over a 60-summed threshold" to "under 40 px differing by *any*
  amount", and the leg meets it at 0 — and it must not be "corrected"
  back.
- **A band must not be computed from the quantity it judges.** The
  bands were first written as `max(measured noise × 4, floor)`, which
  made the eleven "two frames with no input agree" legs tautologies —
  each was compared against a multiple of its own maximum — and let a
  noisy sitting simultaneously redden the difference legs and widen
  every agreement leg in the same region. Found at the independent
  review, demonstrated by painting 2,000 pixels into a copy of one
  frame. The bands are now **independent constants with stated
  reasons**, and the measured noise is a **checked** quantity: each
  region's within-set jitter must sit inside F-33's own 13/channel with
  no pixel over the visible-change bar, or the run fails rather than
  absorbing the noise into a band.
- **"The same coordinate fires" cannot be read off the button that was
  clicked.** A click on a Button moves focus to it (§4.19 Focus), so the
  clicked tab ends up checked *and* focused, which is a third colour
  again. The leg therefore lives on the **previously** checked tab,
  which is never clicked and never focused in the sequence, so only the
  handler's `tab_all_selected = false` can change its face; the
  look-alike it has to exclude ("focus landed there instead") is
  excluded against control B's own measurement of what a checked and
  focused tab looks like.
- **Every verdict is shown able to go red, and the coverage is enforced
  by the script rather than claimed.** A comparison over a wrong region
  or with an over-generous tolerance passes silently and looks like a
  measurement, which is the T11 retrospective's lesson (c) in a new
  place. `-SelfCheck` feeds every verdict a deliberately wrong pairing
  drawn from the committed frames and requires each to fail — and it
  **fails the run if any verdict `-Compare` registers has no row**,
  because the first version of this pass claimed "every" while covering
  23 of 37, and the 14 it missed were exactly the ones that could not
  have gone red. Where a wrong pairing can be chosen so the two frames
  differ **elsewhere but not in the sampled region**, it is, so the row
  tests the region too; where no such pairing exists (whole-client legs)
  the row says so rather than looking like an oversight.
- **The measured within-set jitter in this sitting was 0**, and every
  agreement leg is byte-identical by SHA-256 rather than merely inside a
  band. Recorded as a measurement of this host and this sitting, not as
  a property to rely on: the control-B row's tolerance rule stands, and
  F-33's 13/channel was available and not needed.

- Capture is preceded by `cargo build --release --workspace`, takes
  **multiple frames on each side**, uses the **client** rectangle, and
  states the display scale (Phase 1 F-21 / F-33 / T10).
- The capture width keeps the toolbar's content within the client, or
  names the known width-driven overlap as a known observation
  ([constraints.md §6](../requirements/constraints.md)). **That rule now
  has an input-side reason as well as a visual one** (T10's G7): where the
  toolbar overflows, the overlapped tab `ToggleButton`s stop being
  clickable at all, so a control that clicks one at a narrow width would
  measure the overlap rather than the property it is aimed at. T10's own
  captures at a 785.6 DIP client show the toolbar **not** overlapping, so
  a comparable width is the safe one.
- **At least one control repeats at a display scale ≠ 100%** (A or C):
  a wrong pointer conversion is invisible at 100%
  ([preamble.md §What "green" is worth](./preamble.md)), so a capture
  set taken only at 100% cannot distinguish the migrated input path
  from a broken one. Every capture states its scale either way. The
  development desktop is a single 120-DPI monitor, so **all four**
  controls are at 1.25 and the row needs no second sitting — read from
  the window under capture, because `GetDpiForWindow(GetDesktopWindow())`
  answers 96 on the same machine.
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
  and verified against the target commit before it is used:
  [evidence/owner-smoke/protocol.md](./evidence/owner-smoke/protocol.md),
  in Japanese because the owner runs it (the Phase 1 T11 shape). Each of
  its ten steps is checked performable at the target commit against
  either a frame in this task's own set or a named `gallery_slice_integration.rs`
  fixture, and the mapping is the artifact
  ([log.md](./log.md) §T12 close gate). It prescribes what to do and
  what to look at; what the owner concludes is the owner's.

- [x] the capture script, all four controls in one run, and the frames
- [x] `-Compare`: every difference and agreement leg
- [x] `-SelfCheck`: all 23 verdicts shown able to go red
- [x] the owner smoke protocol, verified against the target commit
- [x] T12

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
  - **§4.15's Diagnostics table still lists two rows T9 makes false**
    (T9 close gate CF-T9-3): "Handler inside a `for` body" and "Binder
    read in handler position" are listed as rejected shapes three
    paragraphs above the subsection stating they are admitted. False
    statements rather than gaps, so they are where this pass starts;
  - **§8.9 marks the string expression forms binding-only, and three
    layers do not enforce it** (T9 close gate CF-T9-2, sharpened at the
    owner disposition). §8.9's mapping table marks `StrLit` /
    `Interpolation` / `StrPropRead` binding-only and its `(assign …)` row
    admits `i32`, `bool` and collections only — yet `check` accepts
    `s = "abc"`, the compiler emits `(assign s "abc")`, and the loader
    passes it; only the evaluator rejects, at invocation. **The runtime
    matches the spec; the compiler does not.** Owner-settled 2026-08-08:
    the *capability* lands at M4-Phase 5 ([milestone plan](../../plan.md)
    revision 1) and the *diagnostic* is an M4-Phase 3 pre-doc intake, so
    T13 **records the divergence** rather than resolving it — and records
    it as an unenforced normative statement, not as a gap in the text;
  - **DD-M4-P2-001's reason for residual 1 not firing does not match the
    runtime** (T9 close gate). "The handler has already returned when
    regeneration runs" is false — a collection write regenerates inside
    `Signal::set`, during the handler's own statement, which T9's F5
    measures directly. The decision's *conclusion* (no cycle) holds, so
    this is the "explanation narrows" case. **Owner-settled 2026-08-08:
    a dated annotation on DD-M4-P2-001, not a supersede.** T13 writes it,
    citing F5 as the measurement;
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
  - **§13.4's "a removal's successor is computed before the mutation"
    matches a runtime that captures at entry and succeeds after** (T7
    start-gate fact 5). The pre-mutation half —
    the restore target, which the tree cannot supply afterwards — is
    captured at entry; structural succession is the domain's first
    surviving stop, which the post-mutation tree yields. The sentence is
    satisfied in substance and not in sequence; T13 decides whether the
    wording narrows;
  - **§4.19 does not fix which arrow axis moves which way inside a
    group** (T7 close gate CF-T7-4). The landed mapping is Left / Up →
    previous, Right / Down → next, with both axes accepted; either the
    spec gains the sentence or the mapping is recorded as unspecified;
  - **§4.19 does not say what a click *outside* an entered scope does to
    focus** (T7 close gate CF-T7-5). The landed answer is "nothing" — the
    click landing is bounded by the traversal root, so it takes the same
    arm as a background click, which is the reading consistent with "no
    widget outside it can be reached by the keyboard";
  - **§3's grammar has no production for `key-down`'s argument, and
    §8.8's IR grammar has none either** (T8 close gate CF-T8-3).
    `signal_handler ::= IDENT "=>" block` and
    `handler ::= "on" IDENT "{" expr "}"` both predate the Moment 1 sync
    that added §4.19's `key-down("ArrowLeft")` example, and §3's
    §Disambiguation table has no `IDENT` `(` row. The landed production
    is `IDENT ( "(" STRING_LIT ")" )? "=>" block`, and the IR text form
    is `on key-down("ArrowLeft") { … }`;
  - **§4.5 still reads "The only recognized signal name is `clicked`"**
    (T8 close gate CF-T8-3), which T6's `dismiss` already falsified and
    `key-down` falsifies again;
  - **§4.19's and §4.8's Button keyboard-activation sentences describe a
    behaviour the runtime does not have** (T8 close gate CF-T8-1). Either
    the runtime gains it — which puts `Enter` and `Space` into the
    keys-the-runtime-keeps table — or the wording narrows. Owner-owned
    before T13 records the outcome;
  - **whether an unrecognised *signal* name should be a diagnostic**
    (T8 close gate CF-T8-4). Today `Box { totally_unknown => … }` is
    accepted on every kind and silently never fires; T8 removed the one
    kind-specific exception rather than adding a rule;
  - **no section says which widget kinds admit children** (T8's owner
    follow-up, 2026-08-08). §4.9 fixes Box's count and §4.11
    ScrollView's, but "`Rectangle` / `Text` / `Button` / `ToggleButton`
    admit none" — now a diagnostic at both gates — is stated nowhere.
    §4.4's registry is where the container / leaf distinction is visible
    and is what the diagnostic cites, so either it gains the sentence or
    the rule is recorded as unspecified;
  - **§4.19's "every widget with a visual is a candidate" is accurate and
    its consequence is unstated** (T10 close gate, measured by G7). A
    *layout container* is such a widget, so a non-clipping one is a hit
    target across its whole arranged rectangle — including the part that
    overflows its parent's cell, where it silently takes clicks aimed at
    the siblings it overlaps. The shipped gallery reproduces this at a
    narrow client: every toolbar tab `ToggleButton`'s own centre resolves
    to a scroll `Button` instead. The runtime matches the specification;
    what is missing is a sentence saying that a container is a candidate
    too, which is what makes overflow an input problem and not only a
    visual one. The overflow *semantics* stay M4-Phase 4's
    ([constraints.md §6](../requirements/constraints.md)); T13 decides
    only whether §4.19 gains the sentence;
  - **§4.16's placement example is corrected by T8**, not here, so this
    is a confirmation rather than a repair;
  - **§13.2's touch paragraph is satisfied, and the runtime is now
    measured more precisely than the sentence** (T11 close gate fact 9).
    "Handling the pointer message is what suppresses that promotion — one
    delivery per contact" is true; what was measured per member is that
    the suppression is keyed on `WM_POINTERDOWN` / `WM_POINTERUP` alone,
    with `ENTER` / `UPDATE` / `LEAVE` claimed for a different reason. T13
    decides whether the sentence gains that precision;
  - **§13.2 says nothing about whether a touch contact moves focus or
    paints hover** (T11). Both are decided — it moves focus exactly as a
    click does, and it writes no hover or pressed state, with the limit
    that a touch user gets no press feedback in M4 — and neither is
    normative anywhere. T13 decides whether §13.2 gains the two
    sentences, and the same pass covers **only the primary contact
    activates a widget**, which is decided and unstated for the same
    reason;
  - **§12.3's four-kind conversion enumeration does not mention that the
    pointer family arrives in screen space** (T11). Row 2 says pointer
    coordinates are divided at the window procedure and is silent on the
    `ScreenToClient` translation ahead of that division, which is now a
    real site in the class §12.3 exists to keep enumerable;
  - **§12.3 row 2's second sentence is false, and was in no task's
    re-verification list until now** (found at T11's start gate). "Where
    hit-testing reads a widget's rectangle back off its Visual (§7.5),
    that readback is converted alongside them" was falsified by T2's
    migration;
  - **no section says that the presentation states compose** (T12 close
    gate, CF-T12-3). §4.19 fixes that a click moves focus to the nearest
    focusable widget at or above the target, and §13.3 fixes what the
    focus indicator is; neither says that a **checked `ToggleButton`
    which is also focused renders a third appearance**, distinct from
    checked and from focused. M4 expresses all three as background
    changes and DD-003 requires focus to be visibly distinct from
    selected *and* hovered, so the composition is in that rule's
    territory. T12's frames measure it — `(52,121,214)` checked,
    `(144,153,150)` checked and focused, `(67,67,67)` neither. The
    runtime is not in question; T13 decides only whether §13.3 gains the
    sentence;
  - **no fixture spelling appears in `docs/dsl_spec.md`** (DD-005 /
    framing R2).
- **T12's frames are available to this pass as rendered evidence**, which
  earlier tasks' re-verification items did not have: §4.19's traversal
  order, the `focus-group` single-stop rule, scope containment, focus
  restoration and `dismiss` each have a committed frame pair in
  [evidence/t12-frames/](./evidence/t12-frames/) beside their fixture
  name. A confirmation, not a repair.
- Phase-end retrospective, verification closure mapping, and
  [handoff.md](./handoff.md) — which carries **CF-T12-5 as an open
  question**, not as a settled intention: whether a task should be
  *obliged* to show its positive control's comparisons can fail is
  decided at the **next phase's pre-doc**, and M4-Phase 2 closes with no
  rule change (owner-settled 2026-08-09, [log.md](./log.md) §T12).

### T13a — CF-T7-1 focus-presentation repair (owner-authorized 2026-08-09)

Added after T13's cold full suite fired the exact allocator-address reuse
residual. This is a repair inside T13, not permission to redesign focus
identity:

- keep the accepted pointer-anchor / rebase semantics and their bounded
  possibility that a same-address fresh node becomes the retained focus
  target;
- after rebase and after restoration / succession / modal entry have
  settled the final target, reconcile any retained focused id through the
  existing `with_focus_write` presentation primitive;
- add an allocator-independent, mock-free Windows integration witness
  that forces the equivalent state — retained focus record plus a fresh
  node's default `focused = false` presentation — and proves the
  structural seam repaints it; retain the stochastic allocator witness as
  supporting evidence, not the only regression gate;
- re-run the implementation start/end gates for the expanded runtime
  boundary, including call-site / derived-state / side-effect artifacts,
  and complete **full independent review** before close;
- repeat the clean evidence-profile suite from `cargo clean`, then run the
  remaining CMake / Zig / DSL checks. Push and actual CI remain separate
  owner gates.

- [x] T13a

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
