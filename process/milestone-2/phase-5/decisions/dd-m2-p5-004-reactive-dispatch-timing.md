### DD-M2-P5-004 — Reactive dispatch timing

**Status:** Superseded in part by [DD-M2-P6-001](./m2-phase-6-ui-lowering.md#dd-m2-p6-001--drain-transaction-semantics) (drain stage framing). The deferred-dispatch trigger contract — `Signal::set` writes value + marks dirty, re-evaluation runs at the outermost-frame boundary — is preserved. The three-stage `observer → reactive → layout` framing is replaced by DD-M2-P6-001's three-phase + terminal form (Phase 1 mutation convergence loop / Phase 2 layout / Phase 3 post-commit observers).

**Context:**
DD-P6-003 = A guarantees no callback fires while the host is inside
a `wasamo_*` call; `emit::drain_if_outermost` runs queued observer
notifications at the outermost-frame boundary. Phase 5's reactive
dispatch (re-run dirty Effects, write new values to widget property
storage) must integrate with this rule. The question is whether
Effect re-evaluation runs synchronously inside `Signal::set()`,
deferred to the next outermost-frame drain (alongside observer
notifications), or scheduled by a separate trigger.

The downstream behaviour the answer determines: whether a handler
that writes `count += 1` inside a click callback sees the bound
Text update before the handler returns (sync), or only after the
outermost frame returns (deferred).

**Options:**

Option A — Synchronous inside `Signal::set()`
- `Signal::set(new_value)` immediately runs all dependent Effects
  before returning. The binding evaluator runs in the same call
  stack as the write.

- What you gain: Updates propagate eagerly; reads done immediately
  after a write see the new bound state. Simplest mental model.
- What you give up: A handler doing `count += 1; count += 1; count
  += 1;` re-evaluates every binding three times. Layout invalidation
  fires three times per write (DD-P8-002 marks layout-dirty inside
  `set_property`; sync re-eval would make a binding's
  `set_property` call mark layout-dirty again, repeat). Re-entrancy
  hazards: an Effect that writes a Signal it also reads from
  triggers immediate recursion. The `with_batched_writes` skeleton
  becomes a no-op shape (nothing to batch if every write fires
  inline).
- **Technical risk: Medium.** Re-entrancy and amplification are
  both real; the existing queued-emission rule was introduced
  precisely to avoid this category of bug at the C ABI surface, and
  reintroducing it inside the reactive engine would be a step
  backward.

Option B — Deferred to outermost-frame drain (recommended)
- `Signal::set(new_value)` writes the new value into property
  storage, marks dependent Effects dirty in a thread-local dirty-
  set, and returns. The dirty-set is drained by the outermost-frame
  machinery alongside observer notifications: `drain_if_outermost`
  runs first the queued observer callbacks, then the reactive
  re-evaluation pass, then the layout drain (preserving DD-P8-002's
  size-affecting → re-layout chain).
- `with_batched_writes(f)` (already present as a skeleton)
  increments a thread-local depth counter; while depth > 0, the
  drain at the end of each `wasamo_*` call is suppressed. On the
  outermost-frame `f` exit, a single drain processes the
  accumulated dirty-set.
- The order inside the drain is: (1) deduplicate dirty Effects;
  (2) re-run each dirty Effect once, in registration order;
  (3) Effect bodies write through the same `set_property` path,
  which queues observer notifications — those run on the next
  cycle of the same outer drain, until quiescence.

- What you gain: Composes with the existing queued-emission rule
  (DD-P6-003 = A) — same drain trigger, same re-entrancy
  guarantee. A handler doing N writes to the same Signal results
  in one re-evaluation pass per binding, not N. Re-entrancy
  hazards bounded: an Effect's writes mark new dirty effects but
  do not re-run mid-pass; the drain loop catches the new dirty
  effects on the next iteration until the dirty-set stabilises.
  The existing `with_batched_writes` skeleton lights up cleanly.
- What you give up: A handler that writes a Signal and immediately
  reads the bound widget's new property value sees the *old* value
  (the binding hasn't re-run yet). Mitigated by: (a) handlers that
  need post-write reads can read the Signal, not the bound widget;
  (b) the M2 acceptance set has no such case (counter handler does
  not read its own bound display).
- **Technical risk: Low–medium.** The drain-loop quiescence
  guarantee needs explicit specification (a binding evaluator that
  diverges — keeps writing dirty inputs of its own dependency set —
  must be detectable). Pragmatic answer: cap the drain loop at N
  iterations (N small, e.g. 100), log on overflow, treat as a
  binding-author bug. The cap is policy, not correctness for any
  M2 binding shape (counter binding converges in one pass).

Option C — Explicit flush primitive
- `Signal::set` only marks dirty; an explicit `engine.flush()` call
  triggers re-evaluation. Observer drain and reactive drain are
  separately schedulable.

- What you gain: Maximum control over when re-eval runs.
- What you give up: Phase 6's IR loader (or its callers) has to
  call `flush` at the right moment; getting it wrong leaves the UI
  showing stale values. Adds an API surface no M2 case demands.
- **Technical risk: Low** mechanically, but the design risk is
  surface bloat with no acceptance hook.

**Recommendation:** **Option B.**

Deferred dispatch via the existing outermost-frame drain integrates
with DD-P6-003 = A as a single coherent emission rule rather than
introducing a parallel one. The `with_batched_writes` skeleton was
designed for exactly this shape; Phase 5 implements the dirty-set
side. The acceptance hook (counter writes `count += 1` from a click
handler; the bound Text updates before control returns to the host)
is satisfied because `wasamo_run`'s message dispatcher *is* the
outermost frame — the drain runs after the click handler returns
into the dispatcher, before the next message is pumped. From the
host's observable perspective, "click → label updates" holds.

The drain ordering inside `drain_if_outermost` becomes:

1. Drain queued observer notifications (existing).
2. Drain reactive dirty-set: re-run each dirty Effect once;
   Effect-side writes feed back into the dirty-set; loop until
   quiescent or iteration cap exceeded.
3. Drain layout-dirty windows (existing, DD-P8-002).

The reactive drain is bounded: dirty-set deduplicates per-Effect
(an Effect dirtied by N upstream writes runs once per pass), and
the iteration cap traps divergent bindings. Convergence in one
iteration is the M2 expectation (counter's binding reads `count`,
writes the Text content; the Text widget is not itself a Signal
input).

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are M3 Computed (introduces multi-pass propagation),
M3 structural bindings (subtree rebuild during a drain), and
post-1.0 hot reload.

- Option A leaves no batching surface for M3 Computed to interpose
  on; topological re-eval ordering, when it lands, would clash
  with sync inline propagation.
- Option B's drain-loop shape is the natural place for M3 Computed
  to add topological ordering: dirty Computeds re-run before dirty
  Effects that depend on them. Structural-binding subtree rebuilds
  during a drain become "Drop the old subtree's Effects; create
  new Effects; the new Effects re-run on next iteration of the
  same drain" — composes with the loop. Hot reload's whole-graph
  swap fits the same shape.
- Option C is forward-compatible but its M2 cost is unmotivated.

This axis reinforces Option B: the drain shape is the canonical
extension point for foreseeable reactive-engine growth.

**Technical-risk re-evaluation:** Option B's risk (drain loop
quiescence) is bounded by an iteration cap and is the same risk
shape every dirty-set reactive engine has solved. Option A's
re-entrancy / amplification risk is unbounded without separate
mitigation. Option C is safe but adds API. Risk reinforces
Option B.

---
