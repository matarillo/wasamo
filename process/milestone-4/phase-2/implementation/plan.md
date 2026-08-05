---
phase: M4-Phase 2
title: Implementation task list
status: draft
adr: process/milestone-4/phase-2/decisions/preamble.md
---

# M4-Phase 2 — Implementation task list

Mutable during the phase. Task splits, additions and reorderings are
recorded here as they happen rather than left as a frozen prediction
([AGENTS.md §Commit rules](../../../../AGENTS.md#commit-rules)). The
execution framing, the sequencing thesis and what a green suite is worth
are in [preamble.md](./preamble.md).

Each task runs the implementation gates at **both** start and close
([implementation-gates.md](../../../procedures/implementation-gates.md)),
records its gate selection with reasons for the rows judged
non-applicable, and — because a start-gate selection is a prediction
that goes stale (Phase 1 F-53) — **re-decides that selection at close**
whenever the task built something it did not expect to build.

---

## T0 — Settle DD-005's key handling (blocking, no code)

**DD-005 is `Proposed`.** The phase does not open until its key-handling
sub-decision closes, because that decides whether the authored surface
gains a key signal family — and therefore what
[dsl_spec.md §4.19](../../../../docs/dsl_spec.md) says. §4.19 currently
carries the K1 position and is rewritten by this task.

**Why it reopened.** K1 was accepted without being judged against the
rest of the set, and it leaves **two** named behaviours with no
mechanism:

- **Left/Right** — the runtime has no notion of a photo index, and K1
  ships no place to write `selected_index -= 1`.
- **Esc** — DD-004 says the scope *names* the recipient and that *"the
  act of closing … is authored. The core never mutates the tree."*
  Naming a recipient is not a way to react to it. Under K1 there is no
  place to write the state clear that closes the lightbox, so DD-004 and
  DD-005 contradict each other.

Arrow keys *inside a focus group* are unaffected (DD-003 defines those
as focus movement) and no option changes that.

**The rewrite splits it into two questions**, because the reference
toolkits all do and mixing them is what produced the wrong answer:

- **Dismissal** (D1 / D2 / D3) — how an overlay learns the user wants it
  closed. Esc is one *source*; click-away (M4-Phase 9) and a Dialog's
  close control (M5) are others. Recommended **D2**: the scope raises a
  `dismiss` signal, the author decides what closing means, and the
  policy attribute lands at Phase 9 with the second source.
- **Authored key input** (K1 … K6) — how an application reacts to a key.
  Recommended **K3**: one `key-down("<key>")` signal whose key is an
  argument. It is the **`keydown` equivalent** — a physical key press
  for *commands* — explicitly not the web's deprecated `keypress`, and
  not a text-input path; text arrives through the text-store at
  M4-Phase 5 / 6. `statement ::= assign_stmt ";"` means a handler body
  cannot branch, so **K4** (Slint's catch-all) is not authorable in M4
  at all, and **K6** (a structured key value) needs a typed-constant
  kind the value grammar does not have. Both are excluded, and they
  **reopen together** because a structured key's payoff is in the
  catch-all form.
- **M4's recognised key table is named non-character keys only**, which
  keeps the logical-key versus physical-position question closed;
  `"Ctrl+S"` is consequently not expressible in M4.

The full walkthrough with `.ui` examples and the reference comparison is
in
[private/explainer/m4-phase-2-key-handling-options.md](../../../../private/explainer/m4-phase-2-key-handling-options.md);
the decision is recorded in the rewritten DD-005.

**Adopting D2 also rewrites DD-004's dismissal paragraph** (already
done in the draft): the request is *addressed* to the innermost entered
scope rather than bubbled to it. DD-004's decision is unchanged, and one
of its Phase 9 falsifiers gets weaker as a result — dismissal no longer
depends on the top-layer subtree being an ancestor of the focused
widget, though authored key handlers on it still do.

**Task shape.** No code. Per the rewrite discipline, and because no
downstream work has started, the outcome is applied by **rewriting**
DD-005's key-handling section and §4.19 rather than annotating them,
after which DD-005 is re-accepted. Close artifact: the rewritten
sections plus the owner's decision recorded in [log.md](./log.md).

- [ ] T0

---

## T1 — Layout-derived hit rectangles

Arrange retains each node's arranged rectangle in DIP; the arrange pass
is its **only** writer, in the same sense as the per-node geometry scale
(Phase 1). No consumer yet — this task adds the source, T2 switches the
reader.

- Rectangle is absolute within the window's client area, so hit
  resolution needs no accumulation walk.
- A subtree attached but never laid out has no rectangle and is not
  hit-testable. That is the intended failure and is asserted, not
  assumed.
- **Evidence:** pure-logic tests over constructed trees, plus a writer
  audit naming every site that could write the field.

- [ ] T1

## T2 — Single-target hit resolution

Replace the fire-every-Button recursion with reverse-order topmost
resolution reading T1's cache. Every widget with a visual is a
candidate; whether anything reacts is T3's question.

- **The migration completes in this task** (DD-002): the close artifact
  is a call-site audit table showing **zero** `visual_rect` readers on
  the input path. No commit between T1 and T2 may leave a mixed path.
- Edge containment is a **boundary condition**, so a deliberately wrong
  implementation must be shown to make the named test fail
  ([DD-V-029](../../../cross-milestone/decisions/dd-v-029-pure-logic-red-test-obligation.md)).
- **Evidence:** pure-logic tests over a constructed *overlapping* tree —
  not the gallery, where occlusion is unobservable until T9
  ([preamble.md](./preamble.md)).

- [ ] T2

## T3 — Propagation and the drain boundary

Target, then ancestors, until a handler runs; a handler that runs
consumes the event. No descending phase.

- The ancestor chain is captured **before** dispatch, and the reactive
  drain runs **once after** the walk completes.
- Structural side-effect enumeration: what a handler's state write
  pulls in (drain → re-layout → rectangle cache → focus validity), and
  which of those the walk must not observe mid-flight.
- **Evidence:** integration tests that synthesize a click into a nested
  tree and read back which handlers ran, including a handler that
  removes its own subtree.

- [ ] T3

## T4 — Per-window focus state and Tab traversal

`FocusState` on `WindowState`; the spike's `focus_core` gains its first
production caller; projection derives `Stop` / `Container` from the
widget kind — the two roles the tree can already supply.

- Tab / Shift+Tab in declaration order, wrapping both ends; click moves
  focus when the resolved widget is focusable and leaves it unchanged
  otherwise; window activation does not change it.
- The focus indicator is drawn through `sync_visuals` and nowhere else —
  a `SetOffset` / `SetSize` outside that pass silently breaks the
  property DD-002's audit depends on (Phase 1, T3).
- **Evidence:** integration tests establishing their own initial focus
  state, then asserting the **expected next stop** rather than that
  focus moved.

- [ ] T4

## T5 — DSL: `focus-group` and `modal-scope`

The compiler surface, ahead of the runtime behaviour, because the tree
cannot say "group" or "modal" until this lands
([preamble.md §The sequencing thesis](./preamble.md)).

- Grammar, checker (constant-only `true` / `false`, admitted on
  containers), IR, loader → the node's focus role.
- Two rejects with their own tests: a binding-expression RHS, and the
  attribute on a non-admitting widget.
- No new token, `IrType`, `IrLiteral` or `PropertyValue`.

- [ ] T5

## T6 — Group traversal and modal scopes in the runtime

Now that roles have an authored source, the behaviour DD-003 and DD-004
fix:

- Group: one Tab stop, arrows within, **per-group memory with a single
  writer** — the primitive that moves focus is the primitive that writes
  the memory.
- Scope: annotation and entry as two mechanisms; an un-entered scope
  contributes no Tab stops; only an annotated subtree may be entered;
  restore target captured at entry; **restoration takes precedence over
  structural succession**, and a removal's successor is computed
  **before** the mutation because node identity does not survive a
  rebuild.
- Esc reaches the innermost entered scope by T3's ordinary walk.
- **Retire the spike scaffolding**: `focus_spike`, the `__focus_spike`
  seam, and the override map go, replaced by the real projection.
- **Evidence:** the mechanism fixture, re-pointed at the authored
  annotations; the mutation that deletes the restore branch must go red
  (the spike's M7).

- [ ] T6

## T7 — DSL: generic `clicked`

`clicked` admitted on any widget. Routing is already generic from T3, so
this widens *who may carry a handler*, not how an event travels.

- Button keeps its `enabled` suppression and its keyboard activation;
  both are Button behaviour, documented on Button.
- **Evidence:** a `Box` with a handler fires; a disabled Button still
  occludes what is behind it.

- [ ] T7

## T8 — DSL: per-item handlers inside `for`

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

- [ ] T8

## T9 — Gallery slice (consumer A)

Wire the `.ui` end to end through `.ui` → IR → runtime, from at least
one example host:

- Thumbnail click opens the lightbox, carrying which thumbnail.
- The lightbox is a **root `ZStack` branch** and stays one; its scrim is
  an authored covering widget, and it is what blocks background clicks —
  the scope confines the keyboard only.
- Esc closes and Tab is contained, both per T0's outcome; focus returns
  to the thumbnail that opened it.
- Left/Right per T0's outcome. **Its visible result is a bound value
  changing, not the finished picture** — the caption and the selected
  thumbnail need index reads and equality selection, which are
  M4-Phase 3 ([framing.md](../requirements/framing.md) §範囲の縫い目).

- [ ] T9

## T10 — Touch

Synthesized injection only; no touch hardware is available (framing
agreement ⑥).

- The path under test is touch message → DIP conversion → hit
  resolution → handler, i.e. that touch rides the same seam as the
  pointer.
- **The limit is stated, not implied**: this does not establish that a
  physical touch digitizer produces the same messages. Recorded in the
  same shape as Phase 1's synthesized-`WM_DPICHANGED` limit.

- [ ] T10

## T11 — GUI evidence with positive controls

The four controls from the [framing](../requirements/framing.md)
§検証方針, each with its agreement leg:

| Control | Difference leg | Agreement leg |
|---|---|---|
| A — click routing and item identity | clicking thumbnail N and thumbnail M give different lightbox content | clicking N twice gives the same content |
| B — traversal order | Tab ×1 / ×2 / ×3 and Shift+Tab reach the expected stops in reverse | two frames with no input are identical |
| C — containment and occlusion | with the lightbox open, a background click does nothing | with it closed, the same coordinate fires |
| D — Esc | Esc closes the lightbox | an unrelated key does not |

- Capture is preceded by `cargo build --release --workspace`, takes
  **multiple frames on each side**, uses the **client** rectangle, and
  states the display scale (Phase 1 F-21 / F-33 / T10).
- The capture width keeps the toolbar's content within the client, or
  names the known width-driven overlap as a known observation
  ([constraints.md §6](../requirements/constraints.md)).
- Any tool that reads window geometry or cursor position declares
  Per-Monitor-Aware V2 first (Phase 1 F-48).

- [ ] T11

## T12 — Close gate

- Local clean rebuild and full suite in the evidence profile; CI run id
  recorded at phase end (the split of ownership Phase 1 carried: the
  step that changed code owns the local rebuild, the phase end owns the
  CI id).
- **Moment 2 implementation sync** — re-verify
  [dsl_spec.md §4.19](../../../../docs/dsl_spec.md) and
  [architecture.md §13](../../../../docs/architecture.md) against the
  landed runtime and flip their phase-status markers; record any
  divergence rather than silently correcting it.
- Phase-end retrospective, verification closure mapping, and
  [handoff.md](./handoff.md).

- [ ] T12

---

## Cross-task obligations

- **The stretch re-evaluation checkpoint** fired at the ADR Accepted
  flip ([plan.md](../../plan.md) §Cross-phase dispositions 3). It is a
  milestone-level management item, not a task here, but it is recorded
  so it is not lost: `Image` / direct-value `fill` (M4-Phase 4) and
  multi-line text editing await the owner's re-read.
- **No new ABI function** (framing agreement ⑦, a hypothesis rather than
  a constraint). If a task finds it needs one, it proposes the tier-2
  plan revision before adding it.
- **Every task that measures something re-reads the whole task list at
  its close gate**, not only the item it was assigned (the re-audit
  discipline; recurred at M4-Phase 1 T2).
