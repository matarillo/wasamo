# DD-M3-P6-005 — Conditional subtree effect lifecycle + reactive-drain proof contract

**Status:** Proposed
**Phase:** M3-Phase 6
**AC:** A7 (conditional rendering grammar — binding drives the present /
absent state of a subtree)

## Context

DD-M3-P6-004 makes a conditional subtree present/absent via
`insert_child` / `remove_child` on a `BindingTarget::ConditionalSubtree`
Effect. This DD settles the **lifecycle of the bindings/effects inside
that subtree** and the **reactive-drain observation contract** the
toggle relies on. It discharges the framing's FD-E (binding lifetime +
toggle-observe contract), the FD-CR requirement that "effect / binding
lifetimes inside a conditional subtree must not be left ambiguous", and
the plan-level reactive-drain residual obligation
([../requirements/constraints.md §7](../requirements/constraints.md),
[../../../milestone-2/handoff.md §3](../../../milestone-2/handoff.md)).
Per FD-E this is an **active decision**, not a defensive carry-forward:
the conditional subtree is the first place effects live inside a
structurally toggled region, so its lifecycle boundary is a Phase 6
core deliverable.

Two faces (framing FD-E):

- **(a) effect lifecycle policy** — when the subtree is absent, what
  happens to the binding/effects declared inside it?
- **(b) drain proof contract** — M2 handoff §3 **item 4**: a write at
  `BATCH_DEPTH == 0` drains dirty Effects before control returns (the
  M3-Phase 1 T13 synchronous non-batched contract). Conditional
  rendering directly exercises this: *after toggling the `bool`, when
  can a host/test observe the new subtree presence?*

Relevant end-state mechanics:

- Effects are owned by the hosting widget (`WidgetNode.bindings:
  Vec<EffectHandle>`); disposal is **structural** — every teardown
  path (`remove_child` + drop, `replace_child`, subtree sweep) drops
  each `Box<WidgetNode>`, and binding disposal piggy-backs on that
  walk, removing the `EffectId` from every Signal's dependent set
  ahead of the rest of teardown
  ([architecture.md §6.8.6](../../../../docs/architecture.md#686-effect-lifetime-dd-m2-p5-003--a)).
  "Re-attach … just creates fresh Effects on the new widgets; old
  widgets' Effects dispose through the same path. No explicit hook."
- The drain loop `drain_dirty_effects` runs up to `MUTATION_CAP` (16)
  iterations, **re-scanning `DIRTY_EFFECTS` each iteration**, so
  Effects enqueued *during* a drain are processed within the **same**
  outermost drain (until quiescence or the cap)
  ([reactive.rs](../../../../wasamo-runtime/src/reactive.rs),
  [architecture.md §6.8.3](../../../../docs/architecture.md#683-drain-ordering-inside-drain_if_outermost)).
- M2 handoff §3 names four inherited obligations: **1** cycle
  detection, **2** ordering ties, **3** fan-out × `MUTATION_CAP`, **4**
  the synchronous non-batched drain proof contract, and flags that
  **bool-dependent display structure (notably conditional rendering)**
  directly hits item 4.

## Options

### (a) Effect lifecycle of an absent subtree

- **LA-1 — absent = disposed; present = recreated (recommended).**
  When the condition goes true→false, the subtree is removed and
  dropped; its Effects dispose via the existing structural teardown.
  When it goes false→true, a **fresh** subtree is built and **fresh**
  Effects are registered. There is no "paused effect" state. This is
  exactly the behaviour architecture.md §6.8.6 already describes for
  re-attach.
- **LA-2 — absent = paused/disconnected; present = reconnected.** Keep
  the Effect objects across absence, detached from the dependency
  graph, and reconnect on re-present (preserving any captured state).
- **LA-3 — absent subtree's Effects keep running.** Leave Effects live
  while the subtree is not displayed (approach-1 flavour).

### (b) Drain proof contract under structural toggle

- **DB-1 — preserve the synchronous non-batched contract
  (recommended).** Keep item 4: a condition write at `BATCH_DEPTH == 0`
  (e.g. inside a Button click handler) drains before control returns,
  so the subtree present/absent change — **and** the initial run of any
  freshly-inserted subtree Effects — is complete and observable when
  the toggling call returns.
- **DB-2 — revise the observation boundary.** Declare that subtree
  presence is only guaranteed observable at a later explicit
  flush / next frame, and update the M3-Phase 1 contract accordingly.

### (c) Structural-mutation ordering / transaction model (items 1–3)

M2 handoff §3 asks M3 to *decide* — not silently carry — cycle
detection (item 1), ordering ties (item 2), and fan-out × `MUTATION_CAP`
(item 3). Structural rendering is the first feature where an Effect's
side-effect is a **tree mutation** (insert/remove), not a property
write, so the question is concretely: *what ordering / transaction
guarantees does the runtime make about structural mutations relative to
property writes and to each other?* The options are not "decide vs
defer" but a spectrum of how much model to commit:

- **SM-1 — status quo: structural Effects ride the same topological
  drain, no special ordering contract.** Insert/remove happen wherever
  the existing topological order places the condition Effect; observable
  ordering ties between independent Effects stay implementation-defined,
  exactly as they already are for property Effects. Safety against
  use-after-free comes from the existing structural-disposal invariant
  (§6.8.6: unregister ahead of teardown). This is the current
  carry-forward.
- **SM-2 — normatise ordering for structural targets only.** Define an
  observable rule such as "structural mutations drain after all pending
  property writes in the same drain" (so a subtree is never inserted
  with half-applied sibling state), making *structural* ordering a
  contract while leaving property–property ties implementation-defined.
- **SM-3 — two-phase / transactional structural drain.** Split the
  drain: property Effects settle, then structural mutations apply as a
  batch, then re-drain for the newly-inserted subtree's Effects — a
  transaction boundary around tree shape. Strongest guarantee; largest
  reactive-architecture change.
- **SM-4 — separate effect budget for subtree insertion.** Give the
  fan-out from inserting an N-binding subtree its own budget rather than
  charging it against the single `MUTATION_CAP`, so a large conditional
  subtree cannot trip the divergence guard that exists to catch genuine
  reactive loops.

## Comparison

### (a) lifecycle

LA-3 is the approach-1 anti-pattern FD-CR rejects: an absent subtree
whose Effects keep firing is "built but hidden", contradicting
structural absence and leaking work (and potentially writing to
detached widgets). LA-2 (pause/reconnect) preserves captured state
across absence — genuinely useful for **in-progress input / focus /
scroll position** — but that is precisely the **Element-level identity**
DD-M3-P6-004 defers (ID-2): pausing and reconnecting an Effect implies
a stable subtree identity across absence, i.e. a reconciler. Phase 6
has no driver for state retention across close→open (the lightbox
photo placeholder is stateless; reopening fresh is correct), and LA-2
would smuggle in the identity layer through the back door.

LA-1 is the policy the architecture **already documents** (§6.8.6
re-attach) and the natural partner of DD-M3-P6-004's ID-1 (full
rebuild): absent destroys the entity subtree (Effects included),
present rebuilds it fresh. It makes the lifecycle boundary
**unambiguous** — the FD-CR requirement — with a one-line rule: *an
absent subtree has no live effects; a present subtree's effects are
freshly created and run.* The future LA-2 behaviour (state retention)
arrives **with** the future reconciler (Phase 7 `for` keys), not
before, and DD-M3-P6-004's stable declared tree keeps it reachable.

### (b) drain contract

DB-2 (revise the boundary) would break the verification strategy:
Phase 6's entire positive-control discipline is "toggle the state,
then observe the result" (constraints §3), and the assistant/owner
visible proof reads the post-toggle frame. If presence were only
guaranteed at a later flush, the toggle-then-observe tests would race.
DB-2 also has **no driver**: M3-Phase 1 already established and
T13-verified the synchronous contract, and nothing in conditional
rendering forces its revision — the insert/remove happens *inside* the
condition Effect, which runs *inside* the same drain.

DB-1 keeps the contract, with **one load-bearing requirement made
explicit**: when the condition Effect inserts the subtree, the subtree's
fresh Effects must be **enqueued into the current drain** (marked dirty
on registration) so they run — and thus initialise their bound
properties — **before quiescence**, within the same outermost drain.
The drain loop already re-scans `DIRTY_EFFECTS` each iteration, so this
works **provided binding registration dirties the new Effect** (or runs
its initial pass within the drain). This is the precise behaviour to
pin with a test, and the one place a naive implementation could leave a
freshly-inserted subtree with stale (uninitialised) bound properties
for one frame.

### (c) structural-mutation ordering / transaction model

The genuine Phase-6 hazard is **not** "lightbox is small" — it is the
interleaving of a structural mutation with a property write on an
**overlapping target**: a property Effect could be poised to write a
widget that a structural Effect is about to remove, or a subtree could
be inserted before a sibling's state has settled. Two facts bound this
hazard in Phase 6:

1. **Safety is already covered** by the §6.8.6 disposal invariant
   (binding disposal unregisters from every Signal's dependent set
   *ahead* of teardown), so a captured-reference Effect cannot fire
   against a half-torn-down widget — no use-after-free regardless of
   ordering.
2. **Observability** of inter-Effect ordering is *already*
   implementation-defined for property Effects (item 2 was open before
   conditional rendering). The Phase-6 author surface **does** admit
   multiple sibling / nested conditionals (DD-M3-P6-003 admits `if`
   wherever `member*` is), so the honest question is not "the lightbox
   has one conditional" but "what is observable when several toggle
   together". The answer is bounded by the **quiescent child-order
   invariant** (DD-M3-P6-004): whichever conditionals are present at
   quiescence appear in **declared document order** among the static
   siblings, *independent of effect-/drain-evaluation order*. So the
   **final, observable layout** is fully specified by the declared tree;
   what remains implementation-defined is only the **transient
   drain order** of independent Effects — exactly as it already was for
   property Effects, and not something conditional rendering makes newly
   observable. Item 4's quiescence guarantee is what the verification
   depends on.

So SM-1 is safe **and** does not regress any existing contract. The
cost of SM-2/SM-3 is committing a structural-ordering / transaction
model **before the family's full shape is known** — multiple sibling
conditionals, nested control flow, and Phase 7 `for` are what generate
real ordering requirements (e.g. a `for` that reorders keyed items
needs a defined mutation order), and designing the transaction boundary
against only the single-`if` case risks locking the wrong model. SM-3
in particular is a large reactive-architecture change (a two-phase
drain) with no Phase-6 driver. SM-4's budget split matters only when an
inserted subtree's binding count approaches `MUTATION_CAP` (16), which
the lightbox is far from; committing a budget scheme now would also be
guessing at the `for`-era requirement.

The **owner-impact** of carrying 1–3 forward is therefore: the owner is
*not* leaving a safety gap (point 1) and *not* regressing observability
(point 2); they are declining to freeze a structural-transaction model
on insufficient evidence, with the named re-ignition points (multiple
conditionals / `for` / large subtrees) recorded so the next phase
inherits the decision rather than rediscovering it.

## Recommendation

**(a) LA-1 + (b) DB-1.**

### (a) Effect lifecycle policy (normative for Phase 6)

- **An absent conditional subtree has no live effects.** On
  true→false, `remove_child` drops the subtree; its Effects dispose
  through the existing structural teardown (§6.8.6), unregistering from
  every Signal's dependent set ahead of the rest of teardown so a
  captured-reference Effect cannot fire against a half-torn-down
  widget.
- **A present conditional subtree's effects are freshly created.** On
  false→true, the subtree is built fresh from the declared children
  (DD-M3-P6-004 ID-1) and its bindings register fresh Effects. No
  paused/reconnected effects; no state carried across absence in Phase
  6.
- This is the **minimal lifecycle policy** FD-E asks Phase 6 to make
  explicit; it is recorded normatively in `dsl_spec.md` §4.14
  (conditional chapter) and `architecture.md` (reactive section).

### (b) Drain proof contract (preserve item 4)

- **The M3-Phase 1 synchronous non-batched drain contract is
  preserved, not revised.** A condition write at `BATCH_DEPTH == 0`
  drains before control returns; after the toggling call (e.g. a
  Button click handler) the subtree present/absent change is complete
  and observable.
- **Freshly-inserted subtree Effects run within the same outermost
  drain.** Registering the new subtree's bindings enqueues their
  initial run into the current `drain_dirty_effects` loop (which
  re-scans `DIRTY_EFFECTS` each iteration), so the inserted subtree's
  bound properties are initialised before quiescence — no
  one-frame-stale window. This is pinned by the drain integration test
  (verification closure item 4): toggle open, assert (within the same
  synchronous return) that the subtree is present **and** its bound
  text/properties hold their evaluated values.
- **Bound: "before quiescence" means "within `MUTATION_CAP`
  iterations".** DB-1's same-drain initialisation guarantee holds **as
  long as the drain reaches quiescence within the existing
  `MUTATION_CAP` (16) budget**. An `if`-block body is arbitrary-size
  (DD-M3-P6-003 admits `member*`), so a single insertion can in
  principle fan out more fresh Effects than the cap allows before
  quiescence. The observable behaviour at the cap is **not** silent
  staleness: the existing `MUTATION_CAP` divergence guard fires (the
  documented backstop — `drain_dirty_effects` stops and the runtime
  surfaces the divergence per the established cap path), the same way it
  does for any other Effect fan-out. Phase 6 does **not** add a separate
  insertion budget (SM-4 declined, item 3 below); the cap stays the
  single convergence guarantee, and DB-1 is stated as "initialised
  before quiescence, for subtrees that reach quiescence within the cap",
  not as an unconditional guarantee for unbounded subtrees.

### (c) structural-mutation ordering / items 1–3 disposition

**Recommendation: SM-1** (status quo ordering; carry items 1–3
forward), for the owner-impact reasons in the (c) comparison — SM-1 is
safe (the §6.8.6 disposal invariant) and regresses no existing
contract, while SM-2/SM-3/SM-4 would freeze a structural-transaction
model before the family (`for`, multiple/nested conditionals) reveals
its real requirements. Per the constraints §7 / M2 handoff §3
obligation to decide **fix-or-carry** explicitly, each item:

- **item 1 (cycle detection)** — **carry-forward (no SM change).** A
  conditional toggle introduces no Signal/Effect cycle by itself; the
  condition Effect writes to the widget tree, not back to its own
  Signal. Cycle policy stays the open M3 question (§6.8.8); the
  `MUTATION_CAP` divergence guard remains the backstop.
- **item 2 (ordering ties)** — **carry-forward, SM-2/SM-3 considered
  and declined.** SM-2 (normatise structural-after-property ordering)
  and SM-3 (transactional two-phase drain) were weighed; declined
  because the **quiescent child-order invariant** (DD-M3-P6-004) already
  fixes the observable result — multiple sibling / nested conditionals
  settle into **declared document order** regardless of drain order — so
  structural ordering is not newly observable even though the surface
  admits several conditionals. Only the transient inter-Effect drain
  order stays implementation-defined, exactly as it already was for
  property Effects. The real driver for a *contracted* mutation order
  (a keyed `for` reorder, where present-set order is data-driven, not
  declared) belongs to Phase 7; the named re-ignition points are
  recorded for it.
- **item 3 (fan-out × `MUTATION_CAP`)** — **carry-forward, SM-4
  considered and declined.** Inserting an N-binding subtree fans out N
  fresh Effects into the current drain; for the lightbox N ≪
  `MUTATION_CAP` (16). SM-4 (separate insertion budget) was weighed;
  declined because committing a budget scheme now guesses at the
  `for`-era requirement, and the existing cap remains a correct
  convergence guarantee for Phase-6-scale subtrees. **The cap-reaching
  behaviour is specified, not left silent:** a subtree large enough to
  exhaust the cap before quiescence trips the existing `MUTATION_CAP`
  divergence guard (the documented backstop), so DB-1's same-drain
  initialisation is the guarantee *up to the cap* and the divergence
  path is the observable behaviour beyond it (see DB-1 above). The
  large-subtree-approaching-cap interaction — and whether the `for`-era
  family warrants SM-4's separate budget — is recorded as the
  re-ignition point.

## Forward-compat exposure

- **State retention across absent→present (LA-2 behaviour)** lands
  with the Element-level identity layer (DD-M3-P6-004 ID-2 / Phase 7
  `for` keys), not before. The stable declared tree keeps it reachable
  without an IR change.
- **`untrack` / explicit `engine.flush()`** (post-M2, §6.8.8) — if
  ever added, they would give an author a way to opt out of the
  synchronous drain; Phase 6's DB-1 contract is the default that such
  primitives would refine, not replace.
- **items 1–3 resolution** — cycle policy, ordering-tie contract, and
  fan-out cap strategy remain M3+/M4 open questions with named
  carriers (§6.8.8); after the SM-1..SM-4 comparison, Phase 6 declines
  to freeze a structural-transaction model and carries them forward
  (their resolution is not forced in Phase 6), without foreclosing any
  resolution.

## Technical risk re-evaluation

- **LA-1 reuses the documented re-attach behaviour** (§6.8.6), so the
  lifecycle policy is the architecture's existing shape made
  normative, not a new mechanism — low risk.
- **The drain-ordering of freshly-inserted Effects is the load-bearing
  risk.** If binding registration does **not** enqueue the new Effect
  into the current drain, an inserted subtree would render one frame
  with uninitialised bound properties. Mitigation: the item-4 drain
  test asserts post-toggle property values within the same synchronous
  return; if the registration path does not already dirty new Effects,
  that is the one implementation change the test will force, and it is
  contained to the binding-registration seam (DD-M3-P1-007).
- **Cap interaction (item 3)** is bounded for realistic subtrees and
  guarded by the existing `MUTATION_CAP` divergence path; the residual
  (pathologically large conditional subtree) is carried forward, not a
  Phase 6 blocker.
- **No batched-write path is introduced**, so item 4's `BATCH_DEPTH ==
  0` precondition holds for the Button-click toggle that the
  verification uses; a future batched toggle would re-enter the
  deferred-drain path already covered by the existing batch tests.
