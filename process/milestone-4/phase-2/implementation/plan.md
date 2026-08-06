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

- [ ] T1

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

- [ ] T2

## T3 — Propagation and the drain boundary

Target, then ancestors, until a handler runs; a handler that runs
consumes the event. No descending phase.

- The ancestor chain is captured **before** dispatch, and the reactive
  drain runs **once after** the walk completes.
- Structural side-effect enumeration: what a handler's state write
  pulls in (drain → re-layout → rectangle store → focus validity), and
  which of those the walk must not observe mid-flight.
- **Evidence:** integration tests that synthesize a click into a nested
  tree and read back which handlers ran, including a handler that
  removes its own subtree, and the consumption control DD-001 names:
  adding a handler to a nested widget is shown to stop the ancestor's
  from firing.

- [ ] T3

## T4 — Hover and pressed behind the routing model

The semantic half DD-001 fixes: hover / pressed become transitions
computed from enter / leave against the resolved target, replacing the
whole-tree walk. Geometry already comes from T1's store (switched in
T2); this task changes who owns the state transitions.

- Single-target consistency: the widget that paints pressed is the
  widget a release would dispatch to; overlapping widgets no longer both
  hover. Hover and pressed gain **no authored surface** — they stay
  Button-family presentation (DD-001).
- `WM_MOUSELEAVE` clears through the same transition path; the
  disabled-Button arm keeps its no-reaction contract while the walk's
  descend-into-children behaviour follows T2's topmost rule.
- The synthesised pointer update after a scale change stays **not
  adopted** (DD-001): hover self-corrects on the next move, and a second
  producer of hover state is the shape DD-M4-P1-002 closed.
- **Evidence:** integration tests driving enter / leave / press /
  release sequences and reading back state transitions, including the
  overlap case where only the topmost widget reacts.

- [ ] T4

## T5 — Per-window focus state and Tab traversal

`FocusState` on `WindowState`; the spike's `focus_core` gains its first
production caller; projection derives `Stop` / `Container` from the
widget kind — the two roles the tree can already supply.

- Nothing is focused at window open; Tab / Shift+Tab in declaration
  order, wrapping both ends, so the first Tab lands on the first stop.
  A disabled Button is skipped. Click focuses the **nearest focusable
  widget at or above** the resolved target and leaves focus unchanged
  when there is none; window activation does not change it.
- The key path integrates with the **existing** `WM_KEYDOWN` arm, which
  today forwards to the uninstalled `key_down_fn` host slot and returns.
  Routing consumes ahead of that slot without installing it — the first
  installer fixes the callback unit as shipped API
  ([constraints §2](../requirements/constraints.md)), and this phase
  must not be that installer. **An unconsumed key now falls through to
  `DefWindowProc`** instead of being swallowed (DD-001); the arm's
  return path changes here and the change has its own assertion.
- The focus indicator is drawn through `sync_visuals` and nowhere else —
  a `SetOffset` / `SetSize` outside that pass silently breaks the
  property DD-002's audit depends on (Phase 1, T3). Close artifact: the
  enumeration of every `SetOffset` / `SetSize` in the runtime with its
  pass (DD-003).
- The indicator must be **distinguishable** (DD-003): M4's only means is
  a background change, and the ToggleButton selected state and hover
  state are also background changes. Evidence includes a frame pair
  showing focused versus hovered/selected as visibly distinct states.
- **Evidence:** integration tests establishing their own initial focus
  state, then asserting the **expected next stop** rather than that
  focus moved; the indicator distinctness frames; the write-site
  enumeration.

- [ ] T5

## T6 — DSL: `focus-group`, `modal-scope`, and `dismiss`

The compiler surface, ahead of the runtime behaviour, because the tree
cannot say "group" or "modal" until this lands
([preamble.md §The sequencing thesis](./preamble.md)).

- Grammar, checker (constant-only `true` / `false`, admitted on
  containers), IR, loader → the node's focus role.
- Three rejects with their own tests: a binding-expression RHS, the
  attribute on a non-admitting widget, and a `dismiss` handler on a
  container that does not carry `modal-scope: true` (DD-005: the request
  is addressed to scopes, so a handler elsewhere could never fire).
- `dismiss` is an ordinary signal name in the existing handler table;
  nothing in the IR distinguishes it.
- No new token, `IrType`, `IrLiteral` or `PropertyValue`.
- **Evidence:** accept-side tests — each attribute parses, round-trips
  through the IR, and reaches the loaded node as its focus role — beside
  the three reject tests, each firing its branch directly
  (implementation-gates trap 4).

- [ ] T6

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
- Reconcile `focus_core` with presence-entry: the core's un-entered
  state has no production constructor; the projection either narrows it
  or the branch carries a test that fires it (implementation-gates
  trap 4) — recorded either way.
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
