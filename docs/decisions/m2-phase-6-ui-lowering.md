# M2-Phase 6 — `.ui` → runtime lowering: Architecture Decisions

**Phase:** M2-Phase 6 (`.ui` → runtime lowering)
**Date:** 2026-05-07
**Status:** Accepted

## Context

M2 acceptance criteria **A1** and **A2** ([m2-plan.md](../plans/m2-plan.md#acceptance-criteria),
mirrored from [ROADMAP.md M2](../../ROADMAP.md#m2-foundation)):

> **A1.** `examples/counter/counter.ui` drives the running Hello
> Counter in C, Rust, and Zig — the M1 host-imperative trees in
> `examples/counter-{c,rust,zig}/` are replaced by hosts that load
> the DSL through the agreed wasamoc pipeline.
>
> **A2.** Reactive state propagation works without host-side
> property-set plumbing: `count++` in the host updates the visible
> label through the M2 reactive path, not through a manual
> `wasamo_set_property` call written by the application.

Phase 5 closed A2 *partially*: the reactive engine is verified through
a runtime-internal spike harness that wires a `Signal<i32>` to a `Text`
widget by hand
([wasamoc/src/main.rs `dump-ir`](../../wasamoc/src/main.rs),
[wasamo-runtime/src/experimental_ir_loader.rs](../../wasamo-runtime/src/experimental_ir_loader.rs)).
Phase 6 closes both A1 and A2 permanently by routing reactive
propagation through the `.ui` source path end-to-end. Every other M2
phase contributed structure (Phase 1 cdylib-shim split), the textual
IR shape (Phase 2), the handler-execution model (Phase 3), the
tree-mutation ABI (Phase 4), or the reactive engine (Phase 5);
Phase 6 is where these meet at the host surface.

### Side obligation carried in

DD-M2-P3-002's closing instruction requires `architecture.md` §6
(or its M2-revised equivalent) to document the **signal-dispatch
ordering runtime contract** during this phase. The drain transaction
DD below (DD-M2-P6-001) supplies the substantive content; the
documentation update lands in the same commit that flips this ADR
to `Status: Accepted`.

### Constraints carried in from prior decisions

- **DD-M2-P2-001 = Option B** (textual IR + runtime interpreter). M2
  output format is textual; this ADR commits to a normative grammar
  (DD-M2-P6-002) and the in-IR shape of expressions (DD-M2-P6-003).
- **DD-M2-P2-002 = Option B** (shipping `wasamoc` for M2). Phase 6
  promotes `wasamoc` from the Phase 2 spike-only state to a tool
  whose output drives the running counter (DD-M2-P6-004).
- **DD-M2-P2-003** enumerates 1–7 candidate `wasamoc` activities
  (parse → check → type inference → property-binding lowering →
  handler-body lowering → IR emit → file write-out). DD-M2-P6-004
  resolves which subset is required for A1.
- **DD-M2-P3-001 = Option A** (runtime-side handler interpreter).
  `HandlerExpr` is the in-runtime AST. DD-M2-P6-003 commits to the
  IR-side serialization of that AST.
- **DD-M2-P3-003** (error-reporting via stderr). DD-M2-P6-005
  decides whether `wasamo_load_ui` extends, replaces, or wraps that
  channel.
- **DD-M2-P4-001..004** (tree-mutation ABI). DD-M2-P6-005's loader
  uses these primitives; no new tree-mutation ABI is introduced
  here.
- **DD-M2-P5-001..006** (reactive engine). DD-M2-P5-005's
  `register_binding(target, HandlerExpr)` is marked provisional
  ("revisited at Phase 6 IR-loader implementation time") for the
  `properties` shape; DD-M2-P6-007 settles it. DD-M2-P5-004's
  three-stage drain framing is partially superseded by
  DD-M2-P6-001.
- **DD-P6-003 = Option A** (queued emission). The "no callback fires
  while the host is inside a `wasamo_*` call" rule is unchanged;
  DD-M2-P6-001 alters drain *contents* and *phase ordering*, not
  the firing-timing contract.
- **VISION §4 Principle 2.** Adoption of DD-M2-P6-001 = Option D
  carries a mandatory supplement to this principle (text in
  §11.1 of this ADR), recorded as a structural constraint rather
  than a convention.

### Pre-doc framing input

The owner-aligned framing of this ADR's slate, scope, and
upstream-document update bundling is recorded in
[docs/notes/m2-phase-6/m2-phase-6-pre-doc-framing.md](../notes/m2-phase-6/m2-phase-6-pre-doc-framing.md).
The drain DD's mature draft analysis is folded into DD-M2-P6-001
below, replacing
[docs/notes/m2-phase-6/dd-m2-p6-drain-transaction.md](../notes/m2-phase-6/dd-m2-p6-drain-transaction.md);
that note is archived together with the ADR's `Accepted` flip.

---

### DD-M2-P6-001 — Drain transaction semantics

**Status:** Accepted
**Supersedes:** DD-M2-P5-004 (partial — three-stage drain framing
replaced; deferred-dispatch trigger contract preserved)

**Context:**
DD-M2-P5-004 = B fixed reactive dispatch as deferred to the
outermost-frame drain, and sketched the drain ordering as

> 1. Drain queued observer notifications.
> 2. Drain reactive dirty-set (loop until quiescent or cap).
> 3. Drain layout-dirty windows.

This sketch was sound for the single-Effect counter shape Phase 5
exercises, but left a behavioural path unspecified: a reactive
Effect's body calls internal `set_property`, which queues an
observer notification on the same observer queue whose drain has
already completed in step 1. A literal one-pass reading of the
three-stage drain defers that observer notification to the *next*
outermost-frame cycle (one-frame lag). A loop-the-stages reading
processes it within the current cycle.

The choice surfaces a deeper question that the binary "one-pass vs
loop" framing obscures: **the reactive engine has been treating
three functionally distinct host-↔-runtime callback kinds as if
they were one, by sharing a single queueing mechanism**. They are:

| Kind | Role | Mutation today | VISION §4 P2 |
|---|---|---|---|
| **Signal handler** | Host processes user input events (click, key) | Allowed | "events flow up as host-language callbacks" |
| **Reactive Effect** | Declarative state→property binding | Allowed (internal `set_property`) | "state flows down through property bindings" |
| **Property observer** | Host-registered watcher of property changes | Allowed (current ABI) | **No direct correspondence** |

VISION §4 P2 names two mutation channels (events up, bindings
down). "Property change observed by the host, leading to state
mutation" is *not* in that model; the current ABI happens to permit
it because `set_property` unconditionally enqueues observer
notifications. Whether to keep that latitude — and on what terms —
is the substantive question Phase 6 must settle before the
counter's `.ui` path is wired.

**Design axes.** Four axes generate the option space:

- **α — Frame boundary.** Wasamo's frame unit is the ABI-call
  boundary, not a render pass. This is structural and not in scope
  to revisit; the discussion below uses "same/next cycle" to mean
  "same/next outermost-frame ABI call".
- **β — Observer semantics.** β1 = synchronous mutation channel;
  β2 = mutation deferred one frame; β3 = post-commit pure effect
  (no runtime-state mutation).
- **γ — `set_property` ↔ observer-enqueue coupling.** γ1 = enqueue
  immediately (today's behaviour); γ2 = `set_property` updates value
  + dirty marks only, observer-enqueue is computed in a separate
  phase from a diff or pending set.
- **δ — Convergence-layer separation.** δ1 = dependency-graph
  convergence and event-propagation convergence share one pipeline;
  δ2 = they live in separate phases.

**Options:**

Option A — Single 1-pass three-stage drain  [β2, γ1, δ1]
- Run the three stages literally once each. Reactive-originated
  observer notifications are deferred to the next outermost cycle
  (one-frame lag).
- What you gain: zero implementation cost (Phase 5 ships this
  shape); simplest stop condition; closest reading of DD-M2-P5-004
  as written.
- What you give up: **path-dependent semantics** — the same
  property write reaches observers at different times depending on
  whether the caller was a host signal handler (same cycle) or a
  reactive Effect (next cycle). This is not a stale-by-one-tick
  artefact; it is the framework's notion of "property change"
  being defined non-uniformly. On return from
  `drain_if_outermost`, the system is not quiescent (pending
  observer entries may exist).
- Recasting as "interim" is dishonest: the correct design
  (Option D) is identified and bounded in cost, so calling A
  interim writes a long-lived spec deviation into the ABI surface.
- **Technical risk: Low** to ship; **design risk: High** —
  structurally inconsistent with VISION §4 P2 framed as
  `view = f(state)`.

Option B — Outer loop over the three stages  [β1, γ1, δ1]
- Loop "observer drain → reactive drain → layout drain" until
  observer queue, dirty Effects, and layout-dirty are simultaneously
  empty. The reactive drain keeps its inner cap (16); the outer
  loop adds a second cap.
- What you gain: same-cycle reactive→observer propagation; path
  symmetry restored.
- What you give up: **two caps** (diagnosing divergence requires
  determining which cap fires per case); **convergence-layer
  mixing** institutionalised — pure dependency-graph fixpoint and
  side-effecting event propagation are now interleaved in one
  pipeline; surface area (code + spec) grows for an outcome
  Option C reaches more cleanly.
- **Technical risk: Low–medium**; **design risk: Medium** —
  preserves observer-as-mutation, with extra apparatus.

Option C — Unified side-effect drain  [β1, γ1, δ2 (pipeline-merged)]
- Merge the observer drain and the reactive drain into a single
  alternating loop ("drain one observer entry, drain one dirty
  Effect, repeat") until both queues empty. Run layout once after.
- What you gain: path symmetry; **single cap**; quiescent state
  at return; three stages compress to two.
- What you give up: **non-declarative edges leak into the mutation
  graph** — observers carry host-side mutation through a route
  that is invisible to `.ui`. Mixed in the same pipeline as the
  declarative reactive graph, the system's runtime behaviour stops
  being statically determined by the dependency graph. React Fiber
  and Compose deliberately separate effects from commit precisely
  to avoid this; Option C steps the wrong way across that line.
  "Dynamic determinism" (the loop converges at runtime) is
  achieved, but **structural determinism** (the asymmetry is
  undefinable by construction) is not.
- **Technical risk: Low**; **design risk: Medium-high** —
  predictability and tooling-tractability degrade as binding
  count grows.

Option D — Declarative transaction + post-commit pure observer  [β3, δ2] (recommended)
- Redefine observers as **post-commit pure effects**: callbacks
  that observe a fully-converged frozen state and may perform
  external I/O but **may not mutate runtime state**.
- Drain becomes three phases:
  ```
  Phase 1 (mutation convergence, loop until fixed point):
      while signal_queue ≠ ∅ OR dirty_effects ≠ ∅:
          if signal_queue ≠ ∅:
              pop signal handler, fire host callback
              (callback may freely mutate state via ABI)
          else if dirty_effects ≠ ∅:
              take one dirty Effect, re-run
              (effect body calls internal set_property)
          iter += 1; if iter > MUTATION_CAP: error-log, break

  Phase 2 (layout, 1 pass, terminal):
      for each layout-dirty window: run_layout

  Phase 3 (post-commit observers, 1 pass, terminal):
      IN_OBSERVER_CALLBACK := true
      drain observer queue
      state-mutating ABI returns WASAMO_ERR_OBSERVER_MUTATION
        (panic in debug)
      IN_OBSERVER_CALLBACK := false
  ```
- Return-time invariant: signal_queue, dirty_effects,
  layout-dirty, observer_queue all empty (modulo cap break).
- What you gain (structural):
  - VISION §4 P2 enforced at the ABI surface — mutation flows
    only through events-up and bindings-down; observers are
    read-only + external I/O.
  - No path asymmetry: observers from any source fire in Phase 3.
  - Single MUTATION_CAP; convergence layers truly separated
    (Phase 1 = state-mutation fixpoint; Phase 2 = view consistency;
    Phase 3 = pure side effects).
- Scope of these guarantees: the structural determinism above
  applies *within the runtime boundary*. The dependency graph is
  the ground truth for runtime-state mutation; host-side state
  changes that re-enter the runtime via a subsequent ABI call
  (sanctioned route 1 in the Adoption boundary below) are by
  construction outside the graph and surface only as fresh signal
  emissions on the next cycle. Option D makes the *runtime's*
  mutation graph declarative — the layer the framework controls
  — not the host's full mutation graph. The "what changes when
  this state is set" tooling claim under operational gains is
  scoped accordingly (it answers the question for the runtime's
  own state; host-side state is opaque to it by construction).
- What you gain (operational, not just philosophical):
  - **Predictability** — Phase 1's mutation graph is closed under
    the Signal dependency graph; observers are outside that graph,
    so system behaviour is statically derivable from the
    dependency graph. LSP/devtool can answer "what changes when
    this state is set?" without execution. C and E lose this
    because observer mutation re-enters the graph.
  - **Debuggability** — every mutation source is a signal handler
    or an Effect; both appear in the stack trace. C and E permit
    observer callbacks to mutate, producing causal chains hidden
    from any single stack.
  - **Optimization headroom** — Phase 1 is a pure dependency-graph
    fixpoint, leaving room for parallel evaluation, incremental
    re-eval, and dirty-subgraph scoping. C and E couple observer
    side effects into the same pipeline, narrowing this.
- What you give up — *not* "friction", but **deliberate
  expressiveness reduction**:
  - Patterns like .NET MVVM `INotifyPropertyChanged` →
    ViewModel mutation, Cocoa KVO callback → state update, DOM
    `MutationObserver` → DOM mutation, and Reactive-Extensions
    Subject-mediated host-side state sync are **structurally
    unwriteable in Wasamo**. This is not a migration cost
    softened by a binding guide; it is the framework removing a
    capability by design.
  - Bidirectional sync (external state ↔ UI state) via
    observers is impossible by construction; existing libraries
    of that shape require rewriting onto signal handlers, reactive
    Effects, or the future post-event API (Option F).
  - Affects three audiences: binding authors (community
    bindings transplant patterns that don't apply); external
    integration code (persistence, analytics, log, bidirectional
    sync); tooling (devtool/inspector "read state, write back"
    operations need the special path).
  - Partial collision with VISION's "OSS contribution" /
    "multi-language host" tenets; accepted as part of Wasamo's
    identity.
  - "Observer posts a mutation for next frame" escape hatch is
    not designed in this DD (open question; Option F covers it
    structurally for M3).
  - Adds one error code (`WASAMO_ERR_OBSERVER_MUTATION`) and a
    TLS flag.
- **Technical risk: Low** (TLS flag + state-mutating ABI entry
  guard + queue-bookkeeping change); **design risk: Low** —
  the cost is intentional reduction, not hidden hazard.

Option E — Deferred enqueue + observer-can-mutate  [β2, γ2, δ2]
- `set_property` updates value and dirty marks only; observer
  enqueue is computed from a diff at end of Phase 1. Observer
  mutations are deferred to next outermost cycle.
- What you gain: structural separation of mutation phase and
  notification phase (closest in shape to React commit/effect
  separation among the non-D options); path symmetry holds.
- What you give up: observer mutation is preserved, so
  Option C's non-declarative-edge problem persists, just
  pushed across phase boundaries (deferred mutation still
  feeds back into the graph next cycle); a non-trivial mental
  model ("set in observer, applies later"); meaningful
  implementation cost (diff or pending-set bookkeeping).
- Position: structurally between C ("observer freedom") and
  D ("declarative completeness"); takes half the benefit of
  each. Choosing E codifies "we did not finish making the
  framework declarative, but did not want C's synchronous
  mixing either" — a state that exerts continuous pressure
  to swing toward C or D later.
- **Technical risk: Medium**; **design risk: Medium** — the
  most "undecided" option among the six.

Option F — D + initial-day post-event escape hatch  [β3, δ2]
- Take Option D, and from day one ship a
  `wasamo_post_event(event_id, payload)` ABI: from inside a
  post-commit observer, the host may post an event whose
  handler will run on the next outermost cycle's Phase 1.
- What you gain: D's properties + a structured route for the
  "observer triggers something" cases (analytics/logging
  pipelines, persistence, async bridges) that emerge in M3.
- What you give up: three host-visible concepts (signal
  handler / observer / posted event); the post-event semantics
  must be locked in this DD; design cost pulled forward.
- **Technical risk: Low–medium**; **design risk: Low** for
  D's structure; the locked-in event/payload shape is the
  only added exposure.
- Position: F is **D's standard extension path**, not an
  optional add-on. The "observer wants to trigger something"
  use cases are virtually certain to surface in M3; the only
  question is whether to design F now or after seeing those
  use cases.

#### Comparison

| Axis | A | B | C | D | E | F |
|---|---|---|---|---|---|---|
| Path asymmetry | **yes (path-dep semantics)** | no | no | no | no | no |
| Quiescent on return | × | ○ | ○ | ○ | ○ | ○ |
| Iteration caps | 1 (inner) | 2 (inner + outer) | 1 | 1 (MUTATION_CAP) | 1 | 1 |
| Convergence-layer separation | δ1 | δ1 (degraded) | δ2 (merged pipeline) | δ2 (true) | δ2 (true) | δ2 (true) |
| Observer can mutate runtime state | yes | yes | yes | **no** | yes (deferred) | no (post_event allowed) |
| Mutation graph declarative purity | broken (path-dep) | not maintained | non-declarative edges | **fully declarative** | partial mixing (deferred) | **fully declarative** |
| VISION §4 P2 fit | structurally contradicts | weak | medium-weak | **strong (with supplement)** | medium | **strong (with supplement)** |
| Compatibility with MVVM/KVO patterns | ○ | ○ | ○ | **× (structural break)** | ○ | △ (via post_event) |
| Implementation cost | 0 (status quo) | medium | medium | medium | high | medium-high |
| C ABI surface change | none | none | none | +1 error code | internal only | +`wasamo_post_event` |
| Predictability (static analysis) | × | × | × (graph dynamic) | **○** | × (deferred mutation) | **○** |
| Debuggability (causal chain visibility) | △ | △ | × (hidden mutation) | **○** | △ | **○** |
| Optimization headroom | low | low | low (order-coupled) | **high (pure graph)** | medium | **high** |

**Recommendation:** **Option D** (with a mandatory VISION §4 P2
supplement; see §11.1).

D is the minimum-cost choice that promotes VISION §4 P2 from a
direction of intent to a structural ABI-surface constraint, and
the only option (besides F) that yields *structural* determinism
rather than merely runtime convergence. C achieves the surface
invariants (no path asymmetry, single cap, quiescent return) but
preserves observer mutation, leaving non-declarative edges in the
mutation graph; that defeat is invisible at single-binding scale
(counter) and accumulates as M3 binding-feature growth lands.

The recommendation is *not* "D follows naturally from VISION";
the honest framing is **"choosing D is the decision that
strengthens VISION §4 P2 from convention to structural
constraint"**. The current VISION text declares declarative +
unidirectional as a direction without specifying observer
semantics at the ABI surface; this ADR fills that gap on the
strong side. The §11.1 supplement records that filling. The
self-aware framing matters: without it, future review may push
back ("VISION did not say this") and produce a regression toward
C or E.

**Position relative to F.** F is recommended *as the standard
extension path for M3*, not adopted in M2:

- M2 acceptance does not exercise F (counter completes without
  any observer use).
- The post-event API shape (event_id namespace, payload
  encoding, signal_queue interaction) benefits from a real M3
  use case before being frozen.
- Including F would expand Phase 6 scope into ABI-surface
  design that has no M2 driver.

The ADR commits to F as the named successor: M3 introducing
`wasamo_post_event` is a *planned extension*, not an addition.

**Why C is rejected (explicitly).** C is the most plausible
"compromise" choice — it satisfies every surface-level invariant
this DD started by listing — and rejecting it requires naming
the structural reason. C's failure mode is that the dependency
graph (Signal→Effect) is no longer the ground truth for the
system's runtime behaviour: observer callbacks can mutate state,
producing edges in the mutation graph that are not visible from
`.ui` and that interleave with declarative re-evaluation in the
same pipeline. Tooling that wants to answer static questions
about the UI ("what would change if this state were set?") must
fall back to runtime tracing. React/Compose's commit/effect
separation exists precisely to keep effect side-channels off
this graph; C steps across that line in the wrong direction.
The Wasamo identity statement (Slint philosophy × XAML vocabulary
× multi-language openness, plus declarative-first as Phase 6
adds it) does not survive the loss of structural determinism.

**Why A is rejected.** A's path-dependent semantics is a
structural break of `view = f(state)`, not a tolerable interim:
the same property write produces different observer-firing
behaviour depending on which path triggered it. Calling that
"interim" while D is identified and cheap to implement
(TLS flag + one error code) writes a known-incorrect contract
into the ABI surface. A is acceptable only under extreme schedule
constraint where D is already committed to land within one phase;
M2-Phase 6 has the budget for D directly.

**Why B and E are dismissed.** B reaches the same surface goals
as C with strictly larger surface area (two caps,
institutionalised pipeline mixing). E is structurally cleaner
than C in one respect (true phase separation) but worse in
mental-model terms (deferred observer mutation introduces a
specific delay semantics) and preserves C's
non-declarative-edge problem in deferred form. Neither
dominates among the live options.

**Forward-compat exposure:**

- Out-of-scope items engaged: post-1.0 hot reload, M3
  `wasamo_post_event` (and broader observer→post escape
  hatches), M3 Computed (DD-M2-P5-001's deferred derivation
  layer), animation semantics for compositor-driven animations.
- Option D + planned-F survives all four cleanly: hot reload
  tears down both queues and the dependency graph atomically;
  post-event extends Phase 1 by adding an event source upstream
  of signal handlers; M3 Computed slots into Phase 1's pure
  fixpoint (topological pre-Effect pass); compositor animations
  do not mutate runtime state and are independent.
- A locks in a shape M3 must reverse; B and E lock in caps and
  bookkeeping that conflict with later Phase 1 layering; C
  locks in pipeline merging that obstructs Phase 1
  optimisation work.

This axis reinforces D: maximum forward additivity at low M2
cost, and the planned F path for the foreseeable observer-trigger
needs.

**Technical-risk re-evaluation:** D's incremental risk over the
status quo is small and bounded — a TLS flag, an entry-side guard
on state-mutating ABI calls, queue-bookkeeping for observer
separation, and the new error code. The Phase 5 reactive cap
mechanism is reused as MUTATION_CAP. Risk reinforces D.

#### Adoption boundary (mutation boundary)

"Observer cannot mutate" is a constraint on **runtime state**,
not on host-side activity. The boundary is enforced at the ABI
surface, by TLS flag detection, regardless of callback intent.

**Forbidden during Phase 3 observer callbacks** (TLS-flag
detected; returns `WASAMO_ERR_OBSERVER_MUTATION`, panics in
debug):

- Runtime-state writes: `wasamo_set_property`,
  `wasamo_emit_signal`, `wasamo_signal_set`, and any other ABI
  that updates a Signal value, a property value, or a dirty
  mark.
- Runtime-structure changes: window/element/binding creation,
  destruction, or reparenting.
- Reactive-graph manipulation: Effect register/dispose, Signal
  subscribe/unsubscribe.
- Re-entrant drain: `wasamo_*` calls during observer callbacks
  that would themselves enqueue work.

**Permitted in observer callbacks** (these do not enter the
runtime):

- External I/O: file/network/IPC, log, telemetry/analytics.
- Host-language state changes outside the runtime (host
  globals, in-memory caches, persistence buffers, external
  libraries).
- Pure reads: `wasamo_get_property`, Signal reads.
- Posting work to other threads (the cross-thread ABI call
  itself is forbidden by UI-thread affinity, independently of
  this DD).

**Sanctioned routes back to runtime state from an observer:**

1. Host wires "observer → host event → next ABI call performs
   `wasamo_emit_signal`". Host-side responsibility; no new ABI.
2. Future `wasamo_post_event` (Option F) enqueues to next
   cycle's signal_queue. Not in this DD; M3.

Both sanction "next cycle's Phase 1 processes the event", with
no synchronous return path.

**Phase 1 re-entrancy — host-callback ABI rules.** Inside a
signal handler or Effect body, the host may freely call
state-mutating ABI (`wasamo_set_property`, `wasamo_emit_signal`,
`wasamo_signal_set`); these enqueue further work that the same
drain processes before returning. **Structure-changing ABI is
forbidden during Phase 1**: a nested `wasamo_load_ui` (or any
future tear-down/rebuild call) returns
`WASAMO_ERR_REENTRANT_LOAD` and panics in debug. This separation
keeps Phase 1's mutation graph closed under the dependency graph
(state writes only) while preventing structural changes from
racing the in-progress drain.

A coupled question is what happens when a nested `wasamo_*` ABI
call would itself enter `drain_if_outermost`. DD-P6-003's queued
emission machinery already handles this: the `if_outermost` test
sees an in-progress drain and the nested call skips its own
drain, returning to the outer drain to continue. No new TLS flag
is required; the existing one suffices for both Phase 1
nest-detection and Phase 3 mutation-blocking.

**Boundary maintenance principle.** The boundary is defined at
the ABI surface, not by callback role. Future requests of the
form "this observer is logging-only, surely it can mutate" are
declined: detection is by TLS flag at ABI entry, not by callback
purpose. Capability extensions go in Option F's post-event
namespace, not by widening observer permissions.

#### Drain transaction spec (Option D, accepted form)

```
drain_if_outermost()
  │
  ├─ Phase 1: Mutation convergence  (loop until fixed point)
  │     while signal_queue ≠ ∅ OR dirty_effects ≠ ∅:
  │         if signal_queue ≠ ∅:
  │             pop signal handler, fire host callback
  │             (callback may freely mutate state via ABI)
  │         else if dirty_effects ≠ ∅:
  │             take one dirty Effect, re-run
  │             (effect body calls internal set_property)
  │         iter += 1
  │         if iter > MUTATION_CAP: error-log, break
  │
  ├─ Phase 2: Layout  (1 pass, terminal)
  │     for each layout-dirty window: run_layout
  │
  └─ Phase 3: Post-commit observers  (1 pass, terminal)
        IN_OBSERVER_CALLBACK := true
        drain observer queue;
        state-mutating ABI returns WASAMO_ERR_OBSERVER_MUTATION
          (panic in debug)
        IN_OBSERVER_CALLBACK := false
```

Return-time invariant: `signal_queue` empty, `dirty_effects`
empty, layout-dirty empty, `observer_queue` empty.

#### Phase 1 ordering rules

Structural determinism requires Phase 1 to define an evaluation
order, not merely a fixed point. Three rules fix the order:

1. **`signal_queue` is FIFO** in emission order. Multiple host
   `wasamo_emit_signal` calls within a single ABI entry, or
   chained Effect re-runs that emit further signals, fire their
   handlers in the order they were enqueued.

2. **`dirty_effects` drains in topological order over the Signal
   dependency graph.** An Effect that reads a Signal `a` runs
   after any Effect that writes the Signals on which `a`'s
   readers depend, when both are dirty. Within a topological
   rank, ties resolve by registration order (`EffectHandle`
   allocation order). The topological order exists because the
   Phase 5 dependency graph is acyclic by construction; cycles
   among Effects would imply a divergent dependency, which
   MUTATION_CAP detects (see "Divergence semantics" below).

3. **Same-cycle write-after-write to a Signal: last-wins.** A
   Signal's value at any Phase 1 read is its most recent write;
   intermediate values are not observed. The Phase 3 observer
   queue is computed from the diff between each Signal's value
   at Phase 1 entry and its value at Phase 1 exit; intermediate
   transitions do not produce observer entries (this also
   formalises why repeated writes to the same value are no-ops
   on observer notification).

Rule 1 makes signal-handler firing order portable; rule 2 makes
binding re-evaluation deterministic across runs; rule 3 makes
observer notification volume independent of the intra-Phase-1
trajectory. Together they discharge "structural determinism"
operationally: given an initial state and a sequence of host
events, Phase 1's outcome is a function of that sequence alone,
independent of internal scheduling choices.

The signal/Effect alternation in the spec block ("if signal_queue
≠ ∅: pop … else if dirty_effects ≠ ∅: take one") is one
canonical interleaving compatible with these rules; an
implementation may batch (e.g. drain all signals, then all dirty
Effects in topological order) provided the resulting per-Signal
value sequence is identical to the alternating form. The
alternating form is the spec; batched implementations are
conformant if equivalent.

#### Phase 2 read-only constraint

Phase 2 is strictly read-only with respect to runtime state:
layout reads property values to compute geometry, but layout
code does not subscribe to Signals (no reactive dependency edge
originates in Phase 2) and does not write properties or emit
signals. A layout pass that wrote properties would create a
Phase 1↔Phase 2 cycle outside the fixpoint convergence; a
layout pass that subscribed would extend the dependency graph
with edges no `.ui` construct produced. Both are forbidden by
construction: the layout API surface (internal to the runtime)
takes a read-only view of property values and returns geometry,
nothing more.

#### Divergence semantics (MUTATION_CAP exhaustion)

The spec block's `iter > MUTATION_CAP: error-log, break` line
is incomplete on its own — it leaves the post-break state
unspecified (partial mutation? Phase 2/3 still run?). Three
options were considered:

- **(a) Transactional rollback.** Snapshot all Signal values
  and dirty marks at Phase 1 entry; on cap break, restore the
  snapshot and return an error; skip Phase 2 and Phase 3.
  - Cost: O(|Signals|) snapshot bookkeeping on every outermost
    frame, in the success path as well as the failure path.
    Disproportionate for a failure that indicates a graph bug,
    not a recoverable runtime error.
- **(b) Quarantine frame.** Keep partial mutations in place;
  skip Phase 2 and Phase 3; return an error code; allow the
  next outermost ABI call to resume with the still-dirty
  queues.
  - Failure mode: if the divergence is structural (true cyclic
    or runaway binding), every subsequent frame exhausts the
    cap as well, producing an unbounded error stream rather
    than a clear stop signal. The state is also "valid but
    unconverged" — observable by the host through reads with
    no contract for what is or is not consistent.
- **(c) Terminal error state (recommended).** On cap break,
  the runtime enters a terminal state: every subsequent ABI
  call returns `WASAMO_ERR_REACTIVE_DIVERGED` as a no-op;
  Phase 2 and Phase 3 are skipped for the offending frame; the
  partially mutated state is no longer observable through the
  ABI (read calls also return the error). The host's recourse
  is to tear down the runtime and rebuild — the same recourse
  it has for any unrecoverable runtime error.

**Recommendation: (c).** MUTATION_CAP exhaustion indicates a
structural defect in the binding/Effect graph (cyclic
dependency, runaway re-entry, or a binding that mutates its own
dependency without converging). It is not an error a host can
sensibly recover from at the call site. A terminal error rather
than a per-frame error makes the failure unambiguous: either
the runtime is healthy or it is dead. Option (b)'s "valid but
unconverged" middle state and (a)'s success-path snapshot cost
both make the framework worse to use overall in exchange for
handling a case that is, by definition, a graph bug.

The terminal state is reached only on cap exhaustion; Phase 3's
`WASAMO_ERR_OBSERVER_MUTATION` violations do not put the
runtime in this state (they are caller errors, recoverable by
the host correcting the offending observer body and continuing).

`WASAMO_ERR_REACTIVE_DIVERGED` is added to the M2 error-code
set alongside `WASAMO_ERR_OBSERVER_MUTATION`,
`WASAMO_ERR_REENTRANT_LOAD`, and `WASAMO_ERR_IR_MALFORMED`
(DD-M2-P6-009). All four surface through the
`wasamo_last_error_message` channel established in
DD-M2-P6-005 = (i).

Calling "fatal" a divergence is not yet a specification; the
following four sub-clauses make it one.

##### Divergence: state machine

The runtime carries a single boolean liveness state, with one
irreversible transition:

```
   Healthy  ──(MUTATION_CAP exhausted in Phase 1)──▶  Diverged
```

- Initial state on `wasamo_runtime_create` success: `Healthy`.
- The only transition into `Diverged` is Phase 1 breaking out
  of its inner loop because `iter > MUTATION_CAP`.
- There is no reverse transition. `Diverged` is absorbing for
  the lifetime of the runtime instance.
- Phase 3 `WASAMO_ERR_OBSERVER_MUTATION` and other recoverable
  caller errors do **not** transition state.

##### Divergence: commit-forbidden conditions

A frame whose Phase 1 breaks on cap exhaustion is **never
committed**:

- Phase 2 (layout) is skipped for that frame.
- Phase 3 (post-commit observers) is skipped for that frame.
- The Signal mutations written during the diverging Phase 1
  remain physically present in runtime memory (no rollback —
  see option (a) rejection above), but they are **never
  observable through the ABI**: every read after the transition
  returns `WASAMO_ERR_REACTIVE_DIVERGED`.

This is the "unobservable partial mutation" contract: the
runtime does not pay for snapshot bookkeeping, and the host
cannot witness inconsistent state.

##### Divergence: post-divergence ABI contract

While the runtime is in `Diverged`:

| ABI                           | Behaviour                                         |
|---|---|
| `wasamo_runtime_create`       | n/a — operates on a different instance            |
| `wasamo_runtime_destroy`      | **Succeeds**; releases resources; returns `WASAMO_OK` |
| All other `wasamo_*` calls    | No-op; return `WASAMO_ERR_REACTIVE_DIVERGED`      |
| Observer execution            | Does not run                                      |
| Layout pass                   | Does not run                                      |

`wasamo_runtime_destroy` is the single ABI carved out of the
no-op rule: a host must always be able to release a diverged
runtime's resources. Its success in `Diverged` is part of the
ABI contract, not an implementation detail.

##### Divergence: recovery

The only sanctioned recovery path is **destroy + recreate**:

```
   wasamo_runtime_destroy(rt);
   rt = wasamo_runtime_create(/* fresh IR / state */);
```

There is no `wasamo_runtime_reset`-style API. A reset API would
require defining which Signals and Effects survive a partial
recovery and which do not, reintroducing the "valid but
unconverged" middle state that option (b) was rejected for.
Recovery is per-runtime-instance; a host that wants to retain
unrelated state across recovery is expected to scope that state
outside the diverged runtime instance, not inside it.

##### Divergence: debug vs release behaviour

Behaviour is **uniform across debug and release builds**: both
transition to `Diverged` and surface `WASAMO_ERR_REACTIVE_DIVERGED`
through the ABI. The runtime does not panic or `abort()` on
divergence in any standard build configuration.

Rationale: MUTATION_CAP exhaustion is a defect in the host's
binding/Effect graph, not a violation of an engine invariant.
A debug-only panic would make the divergence path itself
untestable for hosts that test against debug builds, and would
diverge the ABI contract by build mode.

A diagnostic-only escape hatch is permitted but not required:
if the environment variable `WASAMO_REACTIVE_ABORT_ON_DIVERGE`
is set to `1` at runtime, the runtime may call
`std::process::abort()` instead of (or immediately after)
transitioning state. This is a triage aid for engine
developers; it does not alter the specified state machine for
hosts that do not set the variable.

##### Divergence: diagnostics contract

On the transition to `Diverged`, the runtime populates
`wasamo_last_error_message` (DD-M2-P6-005 = (i)) with a
structured payload sufficient to identify the offending region
of the binding graph:

- The ID of the Effect being executed when the cap was
  exhausted (or a sentinel if the inner-loop exit happened
  outside any Effect body).
- The iteration count at which the loop broke (= MUTATION_CAP).
- The set of Signal IDs that were dirtied during the final
  iteration, capped at an implementation-defined N with
  `+K more` overflow notation.

This payload remains readable until the runtime is destroyed
(reads of `wasamo_last_error_message` are themselves no-ops in
`Diverged`, but the message buffer is set before the transition
takes effect on the ABI).

Richer diagnostics — full dependency-graph snapshots, cycle
visualisation, time-travel into the diverging frame — are
out of scope (see "Out of scope"); the payload above is
designed to be sufficient raw material for a future tool to
attach to.

#### Upstream-document effects (bundled into the Accepted commit)

| Document | Update |
|---|---|
| `VISION.md §4 Principle 2` | **Mandatory** supplement (§11.1 below) — observer = post-commit pure effect; mutation channel is two-route only. |
| `architecture.md §6` (or its M2-revised section) | drain spec rewritten to the three-phase + terminal form above; mutation boundary documented (DD-M2-P3-002 side obligation discharged here). |
| `m2-phase-5-reactive-engine.md` (DD-M2-P5-004) | Status flipped to "Superseded in part by DD-M2-P6-001 (drain stage framing)"; deferred-dispatch trigger contract preserved. |
| `docs/notes/m2-phase-6/dd-m2-p6-drain-transaction.md` | Archived/removed; content folded into this DD. |
| `docs/notes/reactive-drain-cascade-policy.md` | Closed; question resolved by this DD. |

---

### DD-M2-P6-002 — Normative grammar of the textual IR

**Status:** Accepted

**Context:**
DD-M2-P2-001 = B settled "textual IR" as the M2 wasamoc output
shape. It did not specify the surface form normatively; the Phase 2
spike used an s-expression shape sufficient to pass the
`experimental_ir_loader` round-trip test. Phase 6 promotes the
textual IR from "what the spike happens to write" to "the contract
between `wasamoc` and `wasamo-runtime`". This DD picks the surface
form and where the spec lives.

**Options:**

Option A — Promote the Phase 2 spike s-expression form as-is
- Name the spike's grammar in a normative spec; freeze it.
- What you gain: zero design work; the spike's existing
  round-trip is the conformance test; no parser rewrite.
- What you give up: the spike grammar was sized for one
  round-trip on counter, not for human readability or
  versioning. Some node shapes (handler-body emission in
  particular) are tagged-value flavour with thin error
  reporting; promoting them locks in shapes the spike author
  did not optimise for.
- **Technical risk: Low.**

Option B — Design a new normative grammar (recommended)
- Specify a textual grammar fit for the contract: explicit
  productions for tree nodes, properties, bindings, handler
  bodies, and (per DD-M2-P6-002 sub-issue) a header line.
  Re-use the spike's parser implementation where it agrees
  with the new grammar; rewrite where it does not.
- What you gain: the grammar is the spec, not an
  implementation accident. Header line accommodates fail-fast
  on stale-`wasamoc` / new-runtime in post-M2 hot-reload-like
  scenarios. Diagnostics target the grammar, not parser
  internals. The grammar can be written to be diff-friendly
  (one node per line, indented children), which matters for
  reviewing generated IR during M3 binding development.
- What you give up: design + spec-writing time; some
  rewriting of `experimental_ir_loader` and `wasamoc` emit;
  a new freeze surface this ADR creates.
- **Technical risk: Low–medium** (parser rewrite scope; not
  conceptually novel).

Option C — Adopt a third format (JSON, TOML, custom binary stub)
- Replace s-expression with a different surface form.
- What you gain: structural validators may exist off-the-shelf
  (JSON Schema, etc.).
- What you give up: JSON is poor for handler bodies (no
  expression-tree sugar); TOML is structurally wrong for
  trees; binary is a non-goal for M2 (textual is the explicit
  Phase 2 choice). All three discard parser/emitter code that
  already works.
- **Technical risk: Medium** (replacing more than the
  grammar).

**Header / version contract sub-issue.** Whether the IR mandates a
magic + version line at file head (e.g. `;wasamo-ir v0`).
M2 co-builds `wasamoc` and `wasamo-runtime` in a single workspace,
so version skew is not a correctness concern *in M2*. Writing the
contract now is cheap and protects post-M2 scenarios (hot reload,
shipped pre-built IR) from silent acceptance of stale output.
**Recommended: include a header line.** Reject load on
mismatch; document the bump policy.

**Recommendation:** **Option B**, with a header line.

The textual IR is the contract Phase 6 makes load-bearing.
"Whatever the spike emits" is not a contract; specifying the
grammar normatively is the smallest change that makes the
artifact reviewable. Header-line cost is one line of parser
work and one paragraph of spec.

**Spec home.** Extend `docs/dsl_spec.md` with an IR chapter. A
separate `docs/ir_spec.md` is rejected because the IR is bound
tightly to DSL constructs (binding expressions, handler bodies);
splitting the document fragments the per-construct
documentation. The IR chapter cross-references the DSL chapter
where the lowering target maps directly to a DSL form.

**Forward-compat exposure:**

- Out-of-scope items engaged: post-1.0 hot reload, M3 binding
  features (Computed, conditional, for-loop), M5 LSP/diagnostics.
- B + header survives all three: header version bumps with
  grammar additions; M3 binding features add productions
  additively; LSP attaches to the grammar, not the parser
  implementation.
- A's "spike-shape as spec" carries hidden-decision risk into
  every later extension. C front-loads format change with no
  M2 acceptance benefit.

**Technical-risk re-evaluation:** B's risk is bounded to
parser/emitter rewrite scope; the conceptual change is small.
Risk reinforces B.

---

### DD-M2-P6-003 — IR representation of `HandlerExpr` and binding expressions

**Status:** Accepted

**Context:**
DD-M2-P3-001 = A established `HandlerExpr` as the in-runtime AST
for handler bodies. DD-M2-P5-005 = A reuses `HandlerExpr` as the
binding-expression AST. The IR must serialise both in a form the
runtime parser can rebuild as `HandlerExpr` values. The Phase 2
spike used a tagged-value form for handler bodies (e.g.
`(assign root.count (add (read root.count) 1))`); whether to
promote that form, replace it, or unify it with property values
is the question here.

**Options:**

Option A — Promote the Phase 2 spike's tagged-value form (recommended)
- Each `HandlerExpr` variant has a distinct head tag (`assign`,
  `add`, `read`, literal forms). Serialisation is a direct
  recursive walk of the AST. Binding expressions and handler
  bodies share the form; the difference between them is the
  *target* (DD-M2-P6-007), not the expression shape.
- What you gain: parser/emitter pair already exists in the
  spike; conceptual fit with `HandlerExpr` is exact (the AST
  was designed for this lowering); diff-friendly when the
  grammar (DD-M2-P6-002 Option B) puts one node per line.
  Sharing between bindings and handlers exercises the
  evaluator-core sharing established in DD-M2-P5-002.
- What you give up: the form is verbose for trivial property
  literals (a number `1` becomes `(lit 1)` rather than `1`);
  acceptable for M2, addressable in DD-M2-P6-002's grammar by
  permitting bare literals where the position is unambiguous.
- **Technical risk: Low.**

Option B — Custom expression mini-language with infix syntax
- Use infix `+`, `=`, `.` etc.; parse to `HandlerExpr` via a
  small precedence parser.
- What you gain: handler bodies read like the source DSL.
- What you give up: a second precedence parser to maintain
  alongside `wasamoc`'s; ambiguity with property literal
  values that contain operators (strings); tooling cost
  outweighs M2 ergonomic gain.
- **Technical risk: Medium.**

Option C — Distinct schemes for bindings vs handlers
- Bindings serialise with one shape, handler bodies with
  another.
- What you gain: each can be optimised independently.
- What you give up: defeats the evaluator-core sharing
  (DD-M2-P3-001/DD-M2-P5-002); two parsers and two emitters
  to maintain; the runtime evaluator must accept either
  origin.
- **Technical risk: Medium.**

**Recommendation:** **Option A.**

The tagged-value form's verbosity is a trivially addressable
artefact of DD-M2-P6-002's grammar choice; everything else aligns
with prior decisions. Bindings and handlers use one expression
shape, mapping 1:1 to `HandlerExpr` variants.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 binding features (Computed,
  conditional, for-loop expressions); M3 DSL spec finalisation
  (richer expression forms — function call, ternary).
- A is additive: M3 expression forms add `HandlerExpr`
  variants and IR head tags in lockstep; no parser-shape
  change. B's precedence parser fights M3 syntax additions
  (each new operator is a precedence-table edit). C's parallel
  shapes amplify the M3 change cost across both schemes.

**Technical-risk re-evaluation:** A's incremental risk is
near-zero. Risk reinforces A.

---

### DD-M2-P6-004 — M2 scope of `wasamoc` activities

**Status:** Accepted

**Context:**
DD-M2-P2-003 enumerates seven candidate `wasamoc` activities:
parse → check → type inference → property-binding lowering →
handler-body lowering → IR emit → file write-out. M1 `wasamoc`
implements only the first two. M2 acceptance A1 requires whatever
subset is needed to drive `counter.ui`. This DD picks the subset.

A coupled question lives inside the Phase 2 spec deferral: whether
`.ui` carries `state` declarations (Signal ownership in `.ui`) or
leaves Signal ownership on the host. That question directly
determines whether the host needs an element-identity API
(DD-M2-P6-005's sub-issue), so it is decided here, not in
DD-M2-P6-005.

**Options:**

Option A — Full activity set (1–7) including general type inference
- Implement all seven activities; type inference is general
  (not restricted to counter's two types).
- What you gain: M3 binding features gain full type checking
  out of the gate; no follow-up DD on activity scope.
- What you give up: type inference for an unfinalised DSL
  surface (M3 grammar is not done); inference rules locked in
  before there is a spec to align them against; Phase 6 scope
  expands well beyond A1.
- **Technical risk: High** for M2 — designing inference for a
  language whose surface still moves.

Option B — Restricted scope: 1, 2, 4, 5, 6, 7 + minimal type inference (recommended)
- Activities: parse, check, property-binding lowering,
  handler-body lowering, IR emit, file write-out. Type
  inference is restricted to fixed `i32` and string for M2;
  errors-out on anything else.
- `.ui` carries `state` declarations (Signal ownership in
  `.ui`). The IR includes Signal nodes; the runtime
  instantiates Signals from the IR.
- What you gain: covers A1 (counter has only `i32` count and
  string label content); the lowering paths exist as written;
  M3 type-inference design is unconstrained by an M2 inference
  rule set.
- What you give up: handlers/bindings that use other types
  fail at `wasamoc` time. Acceptable for M2; M3 expands.
- **Technical risk: Low–medium** (lowering design for two
  shapes; small).

Option C — Minimum viable: 1, 2, 5, 6, 7 (skip property-binding lowering as a distinct pass; do it during emit)
- Property-binding lowering is folded into IR emit; no
  intermediate lowered form.
- What you gain: smaller `wasamoc` internal pipeline.
- What you give up: handler-body lowering (5) and binding
  lowering (4) share substantial machinery (HandlerExpr
  construction); folding 4 into emit duplicates that machinery
  in the emit step. Saves no work in practice; complicates
  diagnostics.
- **Technical risk: Low**, but design-quality regression.

**Coupled consequence — Signal ownership.**

- Option A and B both place Signal ownership in `.ui` (`.ui`
  declares `state`; host references state by binding name).
  This means the host does **not** need an
  `wasamo_find_element_by_id`-style identity API: the host's
  interaction surface is the named Signal, not the widget tree.
  DD-M2-P6-005 is freed from element-identity scope.
- "Signal ownership stays host-side" (an alternative not
  enumerated above) would require an element-identity API for
  the host to attach Signals to widgets, expanding
  DD-M2-P6-005's surface. Rejected here as it leaks DSL
  responsibility (state declaration) into host-language code,
  defeating A2's "no host-side property-set plumbing"
  acceptance.

**Coupled consequence — name resolution.**

Signal ownership in `.ui` makes name resolution a `wasamoc`
responsibility, not a runtime one. The rules are fixed here so
that DD-M2-P6-007 (`SignalRegistry`) and DD-M2-P6-009 (loader
validation) inherit a defined contract:

- **Scope:** the `.ui` document is a single flat namespace for
  M2. Counter has one `state count: i32` declaration and a
  small set of references; flat scope is trivially sufficient.
- **Resolution time:** compile-time. `wasamoc` rejects undefined
  references and duplicate `state` names at parse/check time.
  The IR carries already-resolved names; "binding references
  state X" appears in IR as a resolved reference to declared
  state X, not as a pending lookup.
- **Shadowing:** prohibited in M2. Two `state` declarations
  with the same name are a `wasamoc` error. M3 component
  scoping (when introduced) revisits this; M2's prohibition is
  the conservative starting point that does not foreclose any
  M3 scoping shape.
- **Runtime side:** the loader (DD-M2-P6-006) reads
  already-resolved names from the IR and indexes
  `SignalRegistry` (DD-M2-P6-007) by them. Reference-resolution
  validation at load (DD-M2-P6-009 = C) verifies every IR-side
  name resolves to a declared registry entry; "unresolved name
  at runtime" is not a possible failure mode beyond malformed
  IR detection.

Component-level scoping, dotted access (`component.state`),
and renaming-on-import are out of scope for M2; they live in
M3's binding-feature DDs.

**Recommendation:** **Option B**, with `state` declarations in
`.ui` (Signal ownership in DSL).

Counter requires `i32` and string only; restricting type
inference to those two avoids designing an inference rule set
for a language whose grammar M3 still moves. Property-binding
lowering as a distinct pass keeps the pipeline diagnosable.
Signal ownership in `.ui` keeps the host surface narrow and
discharges A2's "no host-side plumbing" requirement
structurally rather than by host convention.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 DSL spec finalisation
  (general type inference, expanded type set); M5 LSP /
  diagnostics; post-1.0 hot reload.
- B's restricted inference is replaced (not extended) in M3;
  the replacement is straightforward because B's lowering
  passes are already structured for general types — only the
  inference rule set needs filling in.
- A locks in inference rules likely to conflict with M3 spec.
  C complicates diagnostics for no compensating benefit.

**Technical-risk re-evaluation:** B's risk is the smallest
that satisfies A1; A is high-risk for M2; C is low-risk but
design-degraded. Risk reinforces B.

---

### DD-M2-P6-005 — `wasamo_load_ui` C ABI shape

**Status:** Accepted

**Context:**
The host calls into the runtime to turn a `.ui` (or its
compiled IR) into a running widget tree. The ABI shape is
exposed to all binding languages and is the surface that
hosts write against; small choices here propagate to every
host-language wrapper.

DD-M2-P6-004 = B settles Signal ownership in `.ui`, removing
element-identity from this DD's scope. DD-M2-P6-001 = D
introduces `WASAMO_ERR_OBSERVER_MUTATION`; this DD chooses how
load-time errors and other runtime-loaded errors surface.

Resource-resolution sub-issue: whether the loader takes a
filesystem path, a memory blob, or a build-time-embedded
string. The framing note narrows this to three live
candidates (A/B/C below). Search-path / bundle features are
out of scope for M2 per the framing note.

**Options:**

Option α — Single-function load (recommended)
- One function:
  ```
  WasamoStatus wasamo_load_ui(
      const char* resource,         // path or in-memory pointer per DD-005-r
      WasamoWindowHandle* out_root  // root widget handle on success
  );
  ```
- Returns `WasamoStatus` (existing error-code enum extended
  with the load-related codes). Subsequent host calls use
  `*out_root` and any Signal handles bound to its IR (per
  DD-M2-P6-004 = B, Signal handles are referenced by name
  through a separate accessor — design lives in DD-M2-P6-007).
- What you gain: smallest ABI surface; one round-trip.
- What you give up: a future split into "compile" + "instantiate"
  (e.g. for hot reload pre-loading) requires a new function;
  acceptable because the M2 contract does not require that
  split.

Option β — Split loader / instantiate
- `wasamo_compile_ui(resource) → WasamoIRHandle` and
  `wasamo_instantiate(ir_handle) → WasamoWindowHandle`.
- What you gain: the IR handle can be reused across instances
  (relevant for M3 list rendering and post-1.0 hot reload).
- What you give up: two-phase ABI for an M2 case that doesn't
  exercise reuse (counter loads once); doubles the error-path
  surface; encourages premature lifetime concerns in host
  code.

**Recommendation:** **Option α** for M2.

Multi-instantiation and IR reuse are post-M2 needs; the
single-function form is the smallest shape that satisfies A1.
Split-on-demand is additive: introducing
`wasamo_compile_ui` + `wasamo_instantiate` later does not
break α-shape callers.

**Resource-resolution form (sub-decision):**

- (A) **Absolute path only** — host computes the absolute path
  and passes it. Simplest contract.
- (B) **Path relative to host executable** — runtime resolves
  using the executable directory. Adds platform-specific
  resolution code; useful when the host distributes a `.ui`
  alongside the binary.
- (C) **Compile-time embedded string** — host embeds the
  `.ui` content at compile time and passes a memory blob.
  No filesystem access at runtime.

**Recommended sub-decision: support (A) and (C); defer (B).**
The ABI accepts a path or a `(pointer, length)` blob distinguished
by a small flag. (A) is the lowest-friction shape for a
counter example; (C) is increasingly the right shape for
production deployments and for binding languages where build
systems can embed at compile time. (B) is a small convenience
that adds platform code (Windows: `GetModuleFileNameW` +
path manipulation) for limited M2 benefit; deferral does not
foreclose it.

**Error reporting (sub-decision).** Three live candidates:

- (i) Last-error-string API:
  `const char* wasamo_last_error_message(void);`
- (ii) Continue DD-M2-P3-003's stderr-only convention.
- (iii) Logger callback registration:
  `wasamo_set_error_callback(fn(const char*))`.

**Recommended: (i) for M2; document (iii) as the planned M3 path.**
A last-error string is universally writeable from every binding
language. Stderr-only (ii) is hostile to GUI deployments where
stderr is not visible. (iii) is the right long-term shape but
requires the host to set the callback before the first call;
M2 hosts (counter examples) are simple enough that the
last-error pattern suffices, and (i) does not block (iii) being
added later as a precedence-overriding mechanism.

`WASAMO_ERR_OBSERVER_MUTATION` (DD-M2-P6-001) is consolidated
in this error mechanism: the error code is returned from the
violating ABI call, and the message string identifies the
observer callback in flight (file/line where available). The
other M2-introduced error codes
(`WASAMO_ERR_REACTIVE_DIVERGED`, `WASAMO_ERR_REENTRANT_LOAD`,
`WASAMO_ERR_IR_MALFORMED`, `WASAMO_ERR_WRONG_THREAD`) use the
same channel.

**Lifetime and threading model (sub-decision).**

The single-function load shape leaves four contract points
unspecified that every binding language must agree on. Each
is fixed here:

- **Handle ownership.** `WasamoWindowHandle` is owned by the
  runtime. The host receives an opaque pointer; passing it to
  any `wasamo_*` ABI is the only legal use. The runtime is the
  sole party that mutates or frees the underlying window
  structure.
- **Handle lifetime.** A handle is valid from successful return
  of `wasamo_load_ui` until runtime shutdown. M2 does not
  expose a per-window destroy ABI; the M2 counter's window
  lifetime spans the process. M3 multi-instance scenarios will
  introduce `wasamo_destroy_window` (or equivalent); the M2
  contract is the constant-lifetime base case of that future
  shape, so M2-era hosts do not require revision when explicit
  destruction lands.
- **Last-error message lifetime.** The string returned by
  `wasamo_last_error_message` is owned by the runtime, valid
  until the next `wasamo_*` ABI call from the same thread. The
  storage is thread-local; concurrent calls from different
  threads do not overwrite each other's last error (modulo the
  thread-affinity rule below, which makes "different threads"
  itself a contract violation in M2). The host must copy the
  string if it needs to retain it across ABI calls.
- **Thread affinity (UI-thread confinement).** All `wasamo_*`
  ABI calls must originate from a single thread per runtime
  instance — the thread that called `wasamo_load_ui`. Calls
  from any other thread return `WASAMO_ERR_WRONG_THREAD`
  without performing the requested action and without
  modifying runtime state. This matches the discipline of
  every major retained-mode UI framework (Win32 message
  thread, AppKit main thread, GTK main thread, Slint event
  loop) and is the only model under which the lock-free
  queue / TLS-flag machinery in DD-M2-P6-001 is sound (the
  TLS used by DD-P6-001's IN_OBSERVER_CALLBACK and DD-P6-003's
  IN_DRAIN flags is the same TLS the thread-affinity check
  relies on).

Cross-thread "post a callable to the UI thread" patterns are
the host's responsibility for M2; if a binding-author audience
need surfaces in M3, a `wasamo_post_to_ui_thread` helper can
be added additively. The M2 contract does not foreclose it.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 list rendering (multiple
  instantiations); post-1.0 hot reload (re-load same
  resource); M3 logger callback (iii).
- α + (A)/(C) supports M3 list rendering by introducing
  `wasamo_instantiate` additively; supports hot reload by
  the same path. Logger callback (iii) layers over (i) without
  breaking it.
- β front-loads design for an M2-uncommitted use; (B) and (ii)
  carry costs without M2 benefit.

**Technical-risk re-evaluation:** α + (A) + (C) + (i) is the
smallest ABI satisfying A1; risk concentrates in (C)'s
embedding ergonomics, which is a binding-side concern, not a
runtime concern. Risk reinforces the recommendation.

---

### DD-M2-P6-006 — Productionised placement of the IR loader

**Status:** Accepted

**Context:**
The Phase 2 spike's `wasamo-runtime/src/experimental_ir_loader.rs`
is feature-gated and not part of the default build. Phase 6 makes
the loader load-bearing on M2 acceptance; the question is where
the loader lives in the workspace and what becomes of the
experimental flag.

The malformed-IR validation policy (how defensively the loader
treats input) is decided separately in DD-M2-P6-009 because it
has direct ABI-error-surface impact; this DD cross-references it.

**Options:**

Option A — Inside `wasamo-runtime`, replacing experimental loader (recommended)
- Move loader implementation to `wasamo-runtime/src/ir_loader.rs`
  (or split into a submodule). Remove the
  `experimental_ir_loader` feature flag.
- What you gain: smallest workspace change; the loader lives
  with the runtime types it constructs; single-crate build
  story unchanged for hosts.
- What you give up: future "load IR without instantiating
  runtime" use cases (hot reload pre-loading, IR
  pretty-printer) build into the runtime crate; acceptable
  for M2 since neither use case is in scope.

Option B — Split into `wasamo-loader` crate
- New crate; runtime depends on it.
- What you gain: loader can be used standalone (e.g. for
  diagnostic tools); separation of concerns.
- What you give up: an additional crate to version, build,
  and document; the loader and runtime types are tightly
  coupled (loader constructs runtime types directly), so the
  split would either leak runtime internals or force a thin
  abstraction layer with no current consumer.

**Recommendation:** **Option A**, removing the
`experimental_ir_loader` feature flag.

A single-crate placement matches every M2-acceptance use case
and keeps the loader colocated with the types it builds. The
feature flag was always temporary (Phase 2 spike); removing it
on production-ising is the simplest end state. If a standalone
loader becomes necessary (e.g. for a diagnostic tool), B is
additive.

**Forward-compat exposure:**

- Out-of-scope items engaged: post-1.0 hot reload, M5
  diagnostic tooling.
- A is additive on both: hot reload reuses the same loader
  through repeated calls; diagnostic tooling, when it lands,
  motivates the split (B) — at which point the cost is paid
  for a real consumer.
- B paid up-front carries the maintenance cost without an M2
  consumer.

**Technical-risk re-evaluation:** A is the lower-risk choice;
the experimental code is already present in the runtime crate.
Risk reinforces A.

---

### DD-M2-P6-007 — Final signature of `register_binding`

**Status:** Accepted
**Supersedes:** DD-M2-P5-005 (provisional `properties: Rc<HashMap<String, Signal<i32>>>` parameter shape only; the registration-API surface itself is preserved)

**Context:**
DD-M2-P5-005 = A specified
`pub(crate) fn register_binding(target: BindingTarget, expr: HandlerExpr) -> EffectHandle`,
but marked the surrounding context — specifically the
`properties: Rc<HashMap<String, Signal<i32>>>` argument used by
the binding evaluator to resolve named state references — as
provisional. The shape was sized for the spike's single-`i32`
counter and explicitly flagged for revisit at IR-loader time.
Phase 6 settles it.

**Options:**

Option A — Type-erased `Signal<dyn Any>` map; loader downcasts
- `properties: Rc<HashMap<String, Box<dyn AnySignal>>>` where
  `AnySignal` is a small trait with `get_as_value(&self) ->
  PropertyValue` and dependency-tracking hooks.
- What you gain: one map type for all signal value types;
  scales to the M2 type set (`i32`, string) and to M3 expansion
  with no further signature changes.
- What you give up: dynamic dispatch on every read; trait
  object boilerplate; downcasts at the binding-evaluation
  call site (or trait-method indirection) for every read.

Option B — Per-type maps in a struct (recommended)
- Replace the single map with a struct:
  ```rust
  pub(crate) struct SignalRegistry {
      i32s: HashMap<String, Signal<i32>>,
      strings: HashMap<String, Signal<String>>,
  }
  ```
  `register_binding(target, expr, registry: &SignalRegistry)`.
- What you gain: monomorphic Signal access; no dynamic
  dispatch; type errors caught at name-resolution time
  (M2's restricted type set per DD-M2-P6-004 = B makes this
  a 2-field struct). M3 type expansion adds fields; binding
  callers do not change.
- What you give up: each new type adds a field; minor
  boilerplate; conceptually rigid compared to A.
- **Technical risk: Low.**

Option C — Generic `register_binding<T>` with target-bound type
- `register_binding<T>(target: BindingTarget<T>, expr,
  signal: Signal<T>)`. Each binding registration is
  generic; the Effect closure is monomorphic per binding.
- What you gain: no map at all — the binding holds a direct
  Signal handle; reads are straight Signal `get()` calls.
- What you give up: the binding evaluator (which interprets
  arbitrary `HandlerExpr` over named references) needs the
  map to resolve names anyway; eliminating the map shifts
  resolution to the loader, but the loader must build per-
  expression closure factories for each reference shape, which
  duplicates the evaluator. Defeats the
  evaluator-core sharing of DD-M2-P5-002.

**Recommendation:** **Option B.**

DD-M2-P6-004 = B's type restriction (`i32`, string) makes the
explicit per-type registry trivial (two fields); M3 type
expansion adds fields without changing the registration call
site. Type erasure (A) buys flexibility no M2 binding shape
exercises; per-binding generics (C) defeat the evaluator
sharing. B is the smallest shape that fits both M2 acceptance
and M3 type-set growth.

**Registry key semantics.** `SignalRegistry` keys are the
`wasamoc`-resolved state names defined in DD-M2-P6-004's
name-resolution rules: post-resolution, single flat namespace
per `.ui` document, no shadowing. The runtime does not
interpret the key string; access is `HashMap::get` only. M3
component scoping (when introduced) translates to either a
key-namespacing convention or a nested registry shape; the M2
single-document flat case is compatible with either future
choice and does not foreclose them.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 expanded type set; M3
  Computed (which is signal-shaped); M5 binding-conformance
  tests.
- B is additive on type expansion; Computed slots in as
  another field (`HashMap<String, Computed<...>>`), since
  Computed exposes Signal-shape access. A loses its
  monomorphism advantage as the trait grows; C requires
  binding-side rewrites per type addition.

**Technical-risk re-evaluation:** B's risk is the smallest;
the rewrite from the spike's single-type map is mechanical.
Risk reinforces B.

---

### DD-M2-P6-008 — Migration shape for `examples/counter-{c,rust,zig}`

**Status:** Accepted

**Context:**
M2 acceptance A1 replaces the per-language imperative tree
construction in `examples/counter-{c,rust,zig}/` with hosts that
load `counter.ui` through the agreed pipeline. Two coupled
sub-questions: per-language wrapper API shape, and whether `.ui`
sources are shared or copied per language.

Resource-resolution form is decided in DD-M2-P6-005 (recommended:
absolute path or compile-time embedded blob); this DD picks how
each example uses it.

**Options:**

Option α — Per-language wrapper API: thin direct call
- Each example calls `wasamo_load_ui` directly through its
  language's existing C-ABI binding. No new helper.
- What you gain: smallest example surface; demonstrates the
  raw ABI; binding-author audience sees exactly what their
  binding must expose.
- What you give up: counter examples carry slightly more
  boilerplate (resource path setup) than a polished helper
  would.

Option β — Per-language wrapper API: language-idiomatic helper
- Each binding crate (`wasamo` for Rust, `wasamo.h` for C,
  `wasamo.zig` for Zig) provides a small idiomatic helper
  (e.g. Rust `Wasamo::load_ui_file(path)`).
- What you gain: examples read more naturally; community
  binding authors see a target shape.
- What you give up: helper API surface this ADR creates and
  Phase 6 must specify per language.

**Resource location:**

- (X) Single shared `examples/counter/counter.ui`, all three
  hosts load it (path resolved per Option A or C in
  DD-M2-P6-005).
- (Y) Per-language copies under `examples/counter-c/counter.ui`,
  etc.

**Recommendation:** **Option α with shared (X)**, plus
compile-time embedding (DD-M2-P6-005 = C) for the C and Zig
examples; absolute path (DD-M2-P6-005 = A) for the Rust
example.

α exposes the ABI cleanly to the binding-author audience this
M2 deliverable targets; idiomatic helpers (β) belong in M3
when the wrapper crates' broader API is being designed.
A single shared `.ui` (X) is the canonical "DSL drives all
hosts" demonstration and removes per-host drift risk in copies.

The Rust example uses path-loading because Rust's `cargo run`
ergonomics already point at the workspace; C and Zig use
embedded `.ui` because their build systems make embedding
ergonomic and the resulting binary is self-contained — exactly
the binding-style M3 community bindings will inherit.

**Forward-compat exposure:**

- Out-of-scope items engaged: M3 list/grid examples (additional
  examples), post-M2 search-path / resource-bundle features.
- α + X is additive on both axes: M3 examples add new
  directories; resource-bundle features (when they land)
  augment DD-M2-P6-005's resource-resolution choices, which
  these examples consume rather than define.
- β codifies a per-language helper API before M3 designs the
  full wrapper crates; revisitation likely.

**Technical-risk re-evaluation:** α + X is the lowest-risk
choice (no new API surface); risk reinforces it.

---

### DD-M2-P6-009 — IR loader malformed-input validation policy

**Status:** Accepted

**Context:**
The IR loader (DD-M2-P6-006 = A, in-runtime) reads textual IR
produced by `wasamoc`. M2 co-builds `wasamoc` and
`wasamo-runtime` from the same workspace; in M2 the loader can
trust its input *for correctness*. Post-M2 scenarios (hot
reload, ahead-of-time-built IR shipped with bindings) introduce
the possibility of stale or malformed IR; the validation policy
written now sets the post-M2 defensiveness baseline. Cross-refs
DD-M2-P6-005 (error reporting) for how detected errors surface
to the host.

**Options:**

Option A — Strict
- Every node validated for structure, type, and reference
  resolution; any irregularity fails the load.
- What you gain: maximum safety; clear diagnostics.
- What you give up: validation cost on every load (small for
  M2 IR sizes); two parses (one to validate, one to build) or
  validation interleaved with construction. M2 doesn't need
  this defensiveness against its own emitter.

Option B — Lenient
- Build whatever parses; warn on unknown tags; keep going.
- What you gain: forward compatibility (newer `wasamoc` writes
  tags an older runtime ignores).
- What you give up: malformed IR may produce a partially
  constructed tree; failure modes are silent; post-M2 hot
  reload inherits a debugging hazard. Forward compatibility
  is also a non-goal in M2 (single-workspace co-build).

Option C — Defense-in-depth (recommended)
- `wasamoc` output is *trusted* in the sense that the loader
  performs lightweight checks rather than re-validating
  every invariant the emitter is responsible for. The loader
  strictly verifies:
  - Magic + version line (DD-M2-P6-002 header).
  - Reference resolution (every name referenced by a binding
    or handler resolves to a declared signal/widget).
  - Top-level structure (the IR has the expected document
    shape).
- Anything else the parser would accept structurally is
  trusted; type-level invariants the emitter establishes
  (e.g. binding expression result type matches target
  property type) are *not* re-checked at load.
- The emitter's type-checking pass (DD-M2-P6-004 = B's "check"
  activity, restricted to `i32` and string for M2) is the sole
  guard on binding-expression / property-type integrity. The
  runtime is permitted to assume that every binding
  expression's evaluated `PropertyValue` matches its target
  property's declared type; mismatches indicate a `wasamoc`
  bug, not a recoverable load-time error, and surface as
  whatever evaluation behaviour the type mismatch produces (no
  guaranteed diagnostic). This trust placement is what makes
  DD-M2-P6-007's monomorphic per-type registry sound at load
  time.
- What you gain: cheap; aligned with the M2 trust model
  (single-workspace co-build); correct defensiveness for
  the post-M2 stale-IR scenario (header + reference
  resolution catch the realistic failure modes); diagnostics
  via DD-M2-P6-005's last-error API.
- What you give up: a deliberately unverified surface (the
  emitter's per-node invariants); acceptable because that
  surface is `wasamoc`'s test responsibility, not the
  loader's.

**Recommendation:** **Option C.**

The trust gradient maps the realistic failure modes:
header/version mismatch (post-M2 scenarios), reference
resolution failure (any time), and structural malformation
(parser-level) are all detectable cheaply. Re-validating
emitter-side invariants doubles the spec without catching
failures the test suite for `wasamoc` already addresses.

Detected errors surface through DD-M2-P6-005's last-error
mechanism with a status code distinct from
`WASAMO_ERR_OBSERVER_MUTATION` (suggested:
`WASAMO_ERR_IR_MALFORMED`).

**Forward-compat exposure:**

- Out-of-scope items engaged: post-1.0 hot reload (the
  realistic stale-IR scenario); M5 LSP/diagnostics.
- C is additive: hot reload exercises the existing
  validation paths; LSP attaches to the same diagnostic
  channel.
- A's per-node validation is rebuilt against M3 grammar
  expansion; B's lenient mode is incompatible with hot reload
  defensiveness goals.

**Technical-risk re-evaluation:** C's risk is the smallest
that meets M2 needs without foreclosing post-M2 use. Risk
reinforces C.

---

### DD-M2-P6-010 — `dirty_effects` topological sort fidelity

**Status:** Accepted (2026-05-09) in
[m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-010--dirty_effects-topological-sort-fidelity)
(housing migrated 2026-05-08; resolved Option A — true topological
walk in M2). The Phase 6 ADR's "Forward-compat carry-forward" entry
for DD-010 (line above) refers to the resolved DD; the mandatory
pre-condition for M3 multi-binding is **discharged at acceptance**
(the walk lands in M2; M3 inherits the verified primitive). M3
residuals (cycle detection, ordering ties, fan-out × `MUTATION_CAP`)
are recorded in
[docs/notes/m2-to-m3-handover.md](../notes/m2-to-m3-handover.md).

This DD was opened in the Phase 6 ADR's draft slate and deferred to
M2-Phase 7 per the 2026-05-08 acceptance-criteria revision recorded in
[m2-plan.md](../plans/m2-plan.md)'s Progress section. The full
Context / Options / Recommendation now lives in the Phase 7 ADR; this
stub preserves the section anchor and the Phase 6 chronological record
that the issue surfaced during the DD-M2-P6-001 implementation
retrospective on 2026-05-07. The DD number remains `DD-M2-P6-010` (a
historical surface-time identifier, not a housing-location reference).

---

### DD-M2-P6-011 — String-typed property binding

**Status:** Proposed — housing migrated to
[m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-011--string-typed-property-binding)
(2026-05-08).

This DD was opened in the Phase 6 ADR's draft slate and deferred to
M2-Phase 7 per the 2026-05-08 acceptance-criteria revision recorded in
[m2-plan.md](../plans/m2-plan.md)'s Progress section. The full
Context / Options / Recommendation now lives in the Phase 7 ADR; this
stub preserves the section anchor and the Phase 6 chronological record
that the issue surfaced during the DD-M2-P6-007 implementation step on
2026-05-07. The DD number remains `DD-M2-P6-011`.

---

### DD-M2-P6-012 — Re-entrancy and safety-guard placement principle

**Status:** Proposed — housing migrated to
[m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-012--re-entrancy-and-safety-guard-placement-principle)
(2026-05-08).

This DD was opened in the Phase 6 ADR's draft slate as `Proposed
(decision deferred to Phase 7)` and is now housed in the Phase 7 ADR
per the 2026-05-08 acceptance-criteria revision recorded in
[m2-plan.md](../plans/m2-plan.md)'s Progress section. The full
Context / Options now live in the Phase 7 ADR; this stub preserves the
section anchor and the Phase 6 chronological record that the issue
surfaced from the Phase 5 retrospective and was deliberately deferred
so the full Phase 6 set of re-entrancy states (Diverged / IN_DRAIN /
IN_OBSERVER_CALLBACK / UI-thread confinement) would be available as
evidence. The DD number remains `DD-M2-P6-012`.

---

## Summary of proposed decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P6-001 | Drain transaction semantics | **Option D** — declarative transaction + post-commit pure observer; Phase 1 ordering rules (FIFO + topological + last-wins); MUTATION_CAP exhaustion = terminal error state; Phase 2 strictly read-only; Phase 1 re-entrancy permits state mutation, forbids structure change; VISION §4 P2 supplement mandatory; F as planned M3 extension | Low | Low |
| DD-M2-P6-002 | Normative grammar of textual IR | **Option B** — new normative grammar in `docs/dsl_spec.md`'s IR chapter; mandatory header line | Low–medium | Low |
| DD-M2-P6-003 | IR representation of `HandlerExpr` and bindings | **Option A** — promote tagged-value form; share between bindings and handlers | Low | Low |
| DD-M2-P6-004 | M2 scope of `wasamoc` activities | **Option B** — restricted: parse, check, lower bindings + handlers, emit, write; type inference limited to `i32` + string; `state` declarations in `.ui`; compile-time name resolution (flat namespace, no shadowing) | Low–medium | Low |
| DD-M2-P6-005 | `wasamo_load_ui` C ABI shape | **α + (A)/(C) + (i)** — single function, path or embedded blob, last-error string API; runtime-owned handles for runtime lifetime; UI-thread-confined; `WASAMO_ERR_WRONG_THREAD` on cross-thread call | Low | Low |
| DD-M2-P6-006 | Productionised placement of IR loader | **Option A** — inside `wasamo-runtime`; remove `experimental_ir_loader` flag | Low | Low |
| DD-M2-P6-007 | Final signature of `register_binding` | **Option B** — `SignalRegistry` per-type struct keyed by `wasamoc`-resolved names; supersedes DD-M2-P5-005 provisional `properties` shape only | Low | Low |
| DD-M2-P6-008 | Counter examples migration shape | **α + (X)** — direct ABI calls; shared `examples/counter/counter.ui`; embedded for C/Zig, path for Rust | Low | Low |
| DD-M2-P6-009 | IR loader malformed-input validation policy | **Option C** — defense-in-depth: header/version + reference resolution + top-level structure; trust emitter invariants including type integrity | Low | Low |
| DD-M2-P6-010 | `dirty_effects` topological sort fidelity | Housing migrated to [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-010--dirty_effects-topological-sort-fidelity) | — | — |
| DD-M2-P6-011 | String-typed property binding | Housing migrated to [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-011--string-typed-property-binding) | — | — |
| DD-M2-P6-012 | Re-entrancy and safety-guard placement principle | Housing migrated to [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md#dd-m2-p6-012--re-entrancy-and-safety-guard-placement-principle) | — | — |

**Aggregate impl-risk picture.** DD-M2-P6-001 and DD-M2-P6-005
introduce the new ABI-surface error codes M2 ships
(`WASAMO_ERR_OBSERVER_MUTATION`, `WASAMO_ERR_REACTIVE_DIVERGED`,
`WASAMO_ERR_REENTRANT_LOAD`, `WASAMO_ERR_WRONG_THREAD`,
plus DD-M2-P6-009's `WASAMO_ERR_IR_MALFORMED`). All five share
the `wasamo_last_error_message` channel and the TLS infrastructure
that DD-P6-003's queued-emission machinery and DD-P6-001's
observer-callback flag already require; the marginal
implementation cost per code is small. DD-M2-P6-002's grammar
rewrite is the largest *code-volume* change, but it is a
structured rewrite of an existing parser/emitter pair with the
round-trip test from the spike as a regression baseline. Every
other DD recommends an additive or scope-restricting choice;
the M2 delta is concentrated in the drain transaction (with
its operational sub-rules — ordering, divergence semantics,
re-entrancy boundary) and the loader production-ising.

**Aggregate forward-compat exposure.** All nine DDs recommend
the M3-additive option. The named successor work for M3 is:

- DD-M2-P6-001's Option F (post-event API design with concrete
  use cases).
- DD-M2-P6-002's grammar extensions for new binding shapes.
- DD-M2-P6-003's `HandlerExpr` variant additions.
- DD-M2-P6-004's general type inference rule set (paired with
  the M3 DSL spec).
- DD-M2-P6-005's logger callback (iii) and possibly
  resolution-mode (B).
- DD-M2-P6-006's potential split into `wasamo-loader` (paired
  with M5 diagnostic tooling).
- DD-M2-P6-007's `SignalRegistry` field expansion.
- DD-M2-P6-008's idiomatic per-language helpers (paired with
  M3 wrapper-crate API design).
- DD-M2-P6-009's validation-path reuse for hot reload.
- DD-M2-P6-010, 011, 012 — successor work scoped to M2-Phase 7 and beyond; see [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md) for the per-DD forward-compat treatment.

**Pre-doc validation spike.** Not required for this ADR. The
Phase 2 spike already round-trips the IR through
`experimental_ir_loader`; the Phase 5 reactive engine is
verified pure-logic-side; the C/Rust/Zig binding-author audience
exercise is the GUI checkpoint that closes A1/A2 at Phase 6
implementation completion. The drain transaction's structural
correctness is established by analysis; its operational
behaviour on the counter shape is identical to the Phase 5
single-pass shape (no observer in the M2 acceptance set), so
the regression risk against existing exercise is zero.

## Out of scope

- **Hot reload of the IR.** Post-1.0. DD-M2-P6-002's header,
  DD-M2-P6-005's split-on-demand allowance, DD-M2-P6-006's
  loader colocation, and DD-M2-P6-009's defense-in-depth
  validation collectively keep the M2 architecture amenable;
  no M2 work item enables it. **Assumed model: tear-down +
  full rebuild.** When designed post-1.0, the existing window
  tree, dependency graph, signal_queue, and observer queue are
  atomically destroyed; a new IR is loaded and instantiated.
  State preservation across reload (incremental hot reload) is
  a separate post-1.0 question and is *not* implied by the M2
  choices that keep hot reload amenable. This assumption is
  recorded so the M2 choices (header line, loader placement,
  validation policy, single-thread affinity) can be validated
  against a concrete future shape; the actual hot-reload DD
  will revisit the model and may refine it.
- **Binary IR format.** M2 = textual only (DD-M2-P2-001 = B);
  binary is post-M2.
- **LSP / diagnostics integration.** M5. DD-M2-P6-002's
  grammar-as-spec and DD-M2-P6-005's last-error mechanism are
  the surfaces an M5 LSP attaches to.
- **Resource search paths and bundle systems.** Beyond
  DD-M2-P6-005's recommended (A) and (C), and the deferred
  (B), additional resource-resolution shapes are post-M2.
- **General type inference.** DD-M2-P6-004 = B restricts to
  `i32` and string; M3 takes the general case alongside the
  DSL spec finalisation.
- **`wasamo_post_event` API (Option F).** Not adopted in M2.
  Designed in M3 against concrete observer-trigger use cases.
- **Idiomatic per-language wrapper APIs.** DD-M2-P6-008 = α
  uses direct ABI calls; M3 designs the wrapper crates.
- **Element-identity API
  (`wasamo_find_element_by_id`-style).** Made unnecessary by
  DD-M2-P6-004 = B's choice to put Signal ownership in `.ui`.
- **Logger callback registration ((iii) variant of error
  reporting).** Planned M3 path; DD-M2-P6-005 = (i) ships
  M2.
- **Dependency-cycle visualisation tooling.** DD-M2-P6-001 = D's
  divergence diagnostics emit a structured payload (offending
  Effect ID, iteration count, last-iteration dirty Signal IDs)
  through `wasamo_last_error_message`. Tooling that consumes
  this payload to render the cyclic sub-graph or to time-travel
  into the diverging frame is post-M2; the M2 contract only
  guarantees the raw material is available.

## VISION §4 Principle 2 supplement (mandatory; bundled with Accepted commit)

DD-M2-P6-001 = D's adoption requires the following text appended
to VISION §4 Principle 2 (final wording subject to the same
review pass that flips this ADR to `Accepted`):

> Property observers (host-registered watchers on property
> changes) are post-commit pure effects: they observe a fully
> converged frozen state and perform external side effects
> (logging, telemetry, I/O) without mutating runtime state.
> State mutation **into the runtime** flows exclusively
> through user events (signal handlers) and reactive bindings
> (declarative property bindings). This makes the
> unidirectional model structurally enforced at the runtime
> boundary rather than merely conventional. Host-side state
> external to the runtime may be mutated freely; the
> constraint applies to the runtime's own state — Signals,
> properties, and the dependency graph — and to the channels
> that mutate it.

The supplement is recorded here as inseparable from the DD's
acceptance: choosing D and not writing the supplement leaves
the structural enforcement undocumented at vision level.

## Revisions

- **2026-05-08.** DD-M2-P6-010, 011, 012 housing migrated to
  [m2-phase-7-reactive-foundation.md](./m2-phase-7-reactive-foundation.md).
  These DDs were drafted as part of this ADR's slate but never
  Accepted under it; they were carried as `Proposed` and explicitly
  deferred at Phase 6 closing per the 2026-05-08 acceptance-criteria
  revision (recorded in [m2-plan.md](../plans/m2-plan.md)'s Progress
  section, which also added A5/A6 and scoped Phase 7). The migration
  is a *housing move*, not a content rewrite: full DD bodies are
  preserved verbatim in the Phase 7 ADR, with explicit notes where
  Phase 7 pre-doc may revise the inherited recommendation under the
  A5/A6 framing. This ADR retains stub anchors at the original DD
  section locations for inbound link stability. The Phase 6 ADR
  itself remains `Accepted` for DD-M2-P6-001..009; this revisions
  entry does not modify any Accepted decision.
