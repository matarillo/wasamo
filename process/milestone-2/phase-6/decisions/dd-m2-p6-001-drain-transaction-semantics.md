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
