# M2-Phase 5 — Reactive engine: Architecture Decisions

**Phase:** M2-Phase 5 (reactive state propagation engine)
**Date:** 2026-05-05
**Status:** Proposed

## Context

M2 acceptance criterion **A2** ([m2-plan.md](../plans/m2-plan.md#acceptance-criteria),
mirrored from [ROADMAP.md M2](../../ROADMAP.md#m2-foundation)):

> Reactive state propagation works without host-side property-set
> plumbing: `count++` in the host updates the visible label through
> the M2 reactive path, not through a manual `wasamo_set_property`
> call written by the application.

A2 is the M2 thesis-validation point: every other phase contributes
structure (A3 cdylib-shim split), ABI surface (A4 tree-mutation
primitives), or integration (A1 `.ui`-driven counter), but A2 is where
"property write → bound widget update without host wiring" is exercised
end-to-end for the first time. The other M2 phases either feed into
this engine or consume its output; the engine itself is the foundation
hypothesis the milestone rests on.

Phase 5's job is to design and implement the property-change → binding
re-evaluation → widget property write → invalidate path on top of the
machinery prior phases established, with shape choices that survive
M3 binding-feature growth (Grid cell bindings, List per-item context)
without front-loading fine-grained tracking before there is any DSL
grammar to align against.

### Constraints carried in from prior decisions

- **DD-M2-P3-001 = Option A** (runtime-side handler interpreter).
  Handler bodies mutate property storage through the internal
  `set_property` path
  ([wasamo-runtime/src/widget.rs:334](../../wasamo-runtime/src/widget.rs#L334)).
  The reactive engine observes those internal writes directly; no
  C ABI round-trip is involved. This is the load-bearing argument
  for runtime-side reactivity — see DD-M2-P3-001's reactive-integration
  paragraph.
- **DD-M2-P3-002 = Option B** (separate inline-handler slot vs host
  listener list). The handler evaluator core
  ([wasamo-runtime/src/handler.rs](../../wasamo-runtime/src/handler.rs))
  is already factored as `HandlerExpr` + `EvalContext` trait +
  `evaluate()`. Phase 5 reuses this evaluator for binding-expression
  evaluation, with a read-only context variant — the binding evaluator
  is **not** a parallel implementation.
- **DD-M2-P4-004 = Option A** (no host-visible batching ABI). The
  internal `with_batched_writes` helper
  ([wasamo-runtime/src/reactive.rs:18](../../wasamo-runtime/src/reactive.rs#L18))
  is the runtime-internal coalescing primitive. Phase 5 implements it
  (Phase 4 shipped the skeleton).
- **DD-P6-003 = Option A** (queued emission). The runtime guarantees
  no callback fires while the host is inside a `wasamo_*` call;
  `emit::drain_if_outermost` ([wasamo-runtime/src/abi.rs:369](../../wasamo-runtime/src/abi.rs#L369))
  runs at outermost-frame boundaries. Phase 5's reactive dispatch must
  compose with this rule, not bypass it.
- **DD-P8-002** (size-affecting property writes invalidate layout).
  The runtime already auto-marks layout-dirty on writes to
  `TEXT_CONTENT` / `TEXT_STYLE` / `BUTTON_LABEL`; the layout drain
  runs after the emission drain. Phase 5's binding writes go through
  the same `set_property` path and inherit this behaviour. The
  whole-window dirty granularity is preserved; subtree-grain dirty is
  out of scope (open question in
  [layout-engine note §3.4](../notes/layout-engine.md)).
- **Pre-aligned design axes.** [docs/notes/m2-phase-5-design-axes.md](../notes/m2-phase-5-design-axes.md)
  records owner direction (2026-05-05) on two axes before pre-doc:
  (a) intermediate dependency-tracker depth (Signal + Effect 2-layer,
  read-time auto-collection; Computed deferred to M3); (b) Option A
  verification (pure-logic tests with fake Effect closure; no new
  mirrors; no headless backend). These axes are recorded as DDs in
  this ADR with full options for the record, but the recommendations
  align with the pre-aligned direction.

### What "reactive" means concretely at M2

The smallest set of behaviours that satisfies A2:

1. **Reactive primitive.** A storage cell whose reads are observable
   and whose writes notify dependents.
2. **Binding.** A side-effecting closure that re-runs when any
   primitive it read on its previous run changes. In M2 the only
   binding shape is `Text { content: "Count: \{root.count}" }` and
   similar property-bound expressions; M3 introduces structural
   bindings (conditional / `for` over collections).
3. **Re-evaluation cycle.** Property write → mark dirty bindings →
   (at the appropriate flush point) re-run dirty bindings → new values
   are written into widget property storage → existing layout/render
   invalidation kicks in.
4. **Coalescing.** A burst of property writes from one logical update
   (`count += 1` typically writes once, but reactive writes from a
   binding may cascade) results in each binding re-evaluating at most
   once per cycle, not once per intermediate write.

Computed values (derived primitives that other bindings can read) and
fine-grained dirty propagation through derived nodes are **out of
scope** for M2 — they belong with the M3 DSL spec work that defines
the DSL surface for derivations. The Phase 5 architecture leaves room
for a Computed layer to be added without disturbing Signal or Effect.

### Architectural framing assumption

Reactive GUI architectures span several families: tree-with-bindings
(Slint, QML/Qt), view-function with re-execution (SwiftUI, Jetpack
Compose), coarse subtree-rebuild (Flutter `setState`), and manual
property notification (WPF / WinUI). The cumulative effect of accepted
decisions (DSL × C ABI thesis; DD-M2-P2-001 = B textual IR as a tree
description; DD-P6-001..007 handle-based stable core; DD-M2-P3-001 = A
handler on internal `set_property`) currently fits the
tree-with-bindings family best, but no accepted ADR names that
selection as a long-term commitment.

[docs/notes/architectural-family.md](../notes/architectural-family.md)
tracks the current hypothesis status, the family-neutral vs
family-coupled split of the design, and the re-evaluation triggers
(M3 DSL spec drafting; hot-reload work; binding shapes that don't fit
`BindingTarget`; owner revisitation). This Phase 5 ADR's
recommendations apply within that current hypothesis frame.

The Phase 5 primitive selection is deliberately family-neutral:
Signal + Effect 2-layer with read-time auto-tracking is shared by
families (1) and (2) and degenerates cleanly to (3); only the
`BindingTarget` shape (DD-M2-P5-005) is family-coupled, and it is
`pub(crate)`, revisable through internal refactor without C ABI
churn. The cost of the implicit framing is bounded to that surface.

---

### DD-M2-P5-001 — Reactive primitive layering

**Status:** Proposed

**Context:**
The reactive engine's abstraction surface determines how much
machinery Phase 5 commits to before there is M3 DSL grammar to align
against. Three shapes are in common prior art (Solid.js, Vue ref,
MobX, Knockout): Signal-only, Signal + Effect, and Signal + Computed +
Effect. The choice fixes how easily M3 can layer in derivations
without rewriting Phase 5 internals.

**Options:**

Option A — Minimum viable (global dirty flag; one re-evaluation pass)
- A single thread-local "anything dirty" bit; on flush, every
  registered binding re-runs unconditionally.
- No per-primitive dependency tracking; no Signal abstraction beyond
  "property storage with a write-observer hook".

- What you gain: Smallest code surface that closes A2 — counter's
  one binding always re-runs on any write, which is correct for the
  M2 acceptance set. Ships fastest.
- What you give up: M3 DSL surface (Grid cell bindings, List per-item
  context) re-runs every binding on every write — wasted work scales
  with widget count × binding count. M3 will rewrite to fine-grained
  tracking; Phase 5's API surface is rebuilt rather than extended.
- **Technical risk: Low.** Trivial to implement and verify. Risk is
  rework cost in M3, not implementation correctness in M2.

Option B — Signal + Effect 2-layer with read-time auto-tracking (recommended)
- Two abstractions:
  - `Signal<T>`: storage cell with `get()` (records current effect as
    dependent) and `set()` (marks dependent effects dirty).
  - `Effect`: closure registered with a tracker; on creation it runs
    once, populating its dependency set from `Signal::get()` calls;
    on flush, dirty effects re-run, repopulating dependencies (a
    re-running effect may pick up different dependencies each pass).
- Dependency collection is automatic via a thread-local "current
  effect" stack populated during `Effect` execution; no manual
  `register_dependency` calls.
- No `Computed` (derived primitives) layer; derivations in M2 are
  expressed as effects whose body writes to a widget property.

- What you gain: M3-compatible shape — fine-grained re-evaluation
  works out of the box, Computed is purely additive in M3, and the
  M3 DSL spec can decide derivation grammar without disturbing Phase 5
  internals. Solid.js / Vue ref prior art is well-understood; the
  signal/effect 2-layer is the canonical "small reactive core". Read-
  time auto-tracking eliminates a class of bugs where a binding
  forgets to declare a dependency. Maps cleanly onto property storage
  (each property cell becomes a Signal-equivalent under the hood) and
  onto the existing handler `EvalContext` (binding evaluator gets a
  read-only context variant that records reads).
- What you give up: Larger Phase 5 scope than Option A. Disposal
  semantics, glitch avoidance, and re-entry rules need explicit
  decisions (DD-M2-P5-003, DD-M2-P5-004). A correctness bar (no
  stale-dependency leaks, no double-fires) that Option A doesn't
  have.
- **Technical risk: Low–medium.** The 2-layer pattern is well-
  understood prior art; the runtime side is a few hundred lines of
  bookkeeping (HashMap from Signal id → dependent Effect ids; a
  thread-local effect stack; a dirty set). Risk concentrates in the
  edge cases (DDs 003 / 004 below); not in the basic mechanism.

Option C — Signal + Computed + Effect 3-layer (full Solid-equivalent)
- Adds a `Computed<T>` derived primitive between Signal and Effect.
  `Computed` reads Signals, has its own value and dependents, and is
  itself read by Effects (or other Computeds).

- What you gain: One more layer of the Solid.js mental model — M3
  derivations can be Computed without further engine work.
- What you give up: A glitch-avoidance regime. With Computed in the
  graph, the order of re-evaluation matters: a downstream Effect
  must not see a stale Computed whose Signal inputs already changed.
  Solid handles this with topological ordering; this is real
  implementation work that Phase 5 does not need for the M2
  acceptance set (counter has no derivations between Signal and the
  bound Text). Adds API surface that no M2 binding shape exercises.
- **Technical risk: Medium.** Glitch-free update is the source of
  the risk: implementing topological re-eval ordering correctly for
  a graph that mutates while it runs (an effect re-running may add
  new dependencies) needs careful invariants. Solving it before M3
  has DSL grammar to align against risks getting the API shape
  wrong.

**Recommendation:** **Option B.**

The Signal + Effect 2-layer with read-time auto-tracking is the
smallest shape that is M3-compatible. Option A's rework cost is real
(M3 will rewrite the engine, and Phase 5's API surface to the binding
evaluator and to Phase 6 would have to change) and the saving in
Phase 5 scope is modest. Option C front-loads glitch-avoidance work
that has no M2 driver — the M2 binding shape is Signal → Effect with
no intermediate derivations, and adding Computed before there is a
DSL spec for derivations is premature. Option B sits at the
inflection point where the M3-compatibility benefit dominates the
incremental scope cost.

The 2-layer naming follows the prior-art convention (Solid.js,
Vue ref): `Signal` for the observable cell, `Effect` for the
re-runnable closure. Internal types stay `pub(crate)` in
`wasamo-runtime/src/reactive.rs`; no C ABI symbols are added (Phase
4's DD-M2-P4-004 = A precludes that).

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are M3 DSL spec finalisation (Computed grammar), M3
structural bindings (conditional / for-loop over collections), and
post-M2 hot reload (post-1.0; depends on M2-Phase 2 IR shape).

- Option A locks in a coarse re-eval shape that M3 has to discard.
  Forward-compat exposure is high — every M3 binding-feature DD has
  to revisit Phase 5's engine.
- Option B leaves Computed and structural bindings additive: a
  `Computed<T>` type can be inserted between Signal and Effect
  without changing Signal's or Effect's API; structural bindings
  become a new Effect kind. Hot reload's "tear down old graph,
  build new graph" workflow composes naturally with explicit
  Effect disposal (DD-M2-P5-003).
- Option C commits to a glitch-avoidance strategy whose choice may
  not survive M3 DSL spec. The 3-layer shape is right, but the
  ordering invariants ("each Computed re-runs at most once per
  flush" vs. "each Effect sees a consistent snapshot") are design
  decisions that should land with M3 grammar, not before.

This axis reinforces the Option B recommendation: M3-compatible at
minimum cost, no commitment to invariants that have no M2 driver.

**Technical-risk re-evaluation:** Option B's risk is bounded to the
edge cases enumerated in DDs 003 / 004. Option A is lower-impl-risk
in M2 but has a known M3 rework cost. Option C is medium-impl-risk
without M2 acceptance to amortise it. Risk reinforces Option B.

---

### DD-M2-P5-002 — Dependency collection mechanism

**Status:** Proposed

**Context:**
Given Option B in DD-M2-P5-001, Effects need to know which Signals
they read. Two coherent collection schemes exist: explicit (the
binding declares its dependencies up-front) and implicit (the runtime
records reads during execution, via a thread-local "current effect"
stack). The choice affects API ergonomics, error modes, and how the
binding evaluator integrates with the existing `EvalContext` trait.

**Options:**

Option A — Explicit declaration: binding registers its dependencies
- The binding API requires the caller (Phase 6 IR loader) to enumerate
  the Signal handles a binding depends on at registration time:
  `register_effect(deps: &[SignalId], body: Box<dyn Fn()>)`.
- The binding body has no observation hook on `Signal::get()`; it
  reads values through the same context as the handler evaluator
  but the dependency list is metadata, not derived from execution.

- What you gain: Determinism — the dependency set is fixed at
  registration time, so the engine never has to handle "binding
  picked up a new dependency on this run". Slightly simpler runtime
  (no thread-local effect stack).
- What you give up: Phase 6 (IR loader) must compute the dependency
  set from the binding expression's AST — it has to walk the
  expression and emit the list of `root.count`-shaped reads as ABI-
  facing identifiers. This is a code path Phase 6 doesn't otherwise
  need. Bindings whose dependencies depend on runtime state (M3
  conditional bindings: `if cond { a } else { b }` reads `cond`
  always but `a` or `b` only conditionally) cannot be expressed
  precisely; either over-declare (re-run on a-or-b changes when only
  one is currently read — wasted work) or under-declare (missed
  updates — bug). M3 will replace this scheme.
- **Technical risk: Low** for M2's static binding shape (counter has
  one binding, dependency set is `[root.count]`). Risk shifts to
  Phase 6 (a Phase-6 work item that wouldn't otherwise exist) and
  to M3 (rewrite when conditional bindings appear).

Option B — Read-time auto-tracking via thread-local effect stack (recommended)
- The runtime maintains a thread-local `Option<EffectId>` ("currently
  running effect"). `Signal::get()` reads it and, if present, adds
  the running effect to its dependent set. `Effect::run()` pushes
  itself onto the stack before invoking the body, pops after.
- The binding evaluator's `EvalContext` impl, when used in binding
  mode, calls `Signal::get()` for every property read — dependency
  collection happens automatically as a side effect of evaluation.
- Re-running an effect first clears its previous dependencies (each
  Signal removes the effect from its dependent set, then the effect
  re-adds itself for whatever it reads on the new run). This handles
  conditional bindings correctly without per-binding scaffolding.

- What you gain: Phase 6's IR loader does not enumerate dependencies —
  it just registers `(expression, write_target)` pairs and the engine
  derives the rest. M3 conditional bindings work without a new
  mechanism. The handler evaluator (`HandlerExpr` + `EvalContext`)
  reuses cleanly: a `BindingEvalContext` wraps the same property
  storage but routes reads through `Signal::get()` and forbids
  writes (Phase 5 binding bodies are pure read; mutation happens
  through the binding's bound write target, not through context
  writes). This matches the handler/binding evaluator-core sharing
  noted in the m2-plan Phase 5 boundary section.
- What you give up: A thread-local stack and a tracked effect-id
  invariant ("at most one effect on the stack at a time, except
  during nested Effect creation, which is a programming error in
  M2"). One more thing to specify and test. Re-entrant or async
  evaluation needs an explicit policy (M2 binds dispatch to the
  outermost-frame drain — see DD-M2-P5-004 — so the question is
  bounded).
- **Technical risk: Low–medium.** The thread-local mechanism is
  prior art (Solid signals' core trick is exactly this). Risk is
  in DD-M2-P5-003 (lifetime/disposal of effects whose dependencies
  outlive them) and DD-M2-P5-004 (when reads happen outside an
  Effect — answer: tracked-as-no-dependency, plain read).

Option C — Hybrid: auto-track plus an `untrack` escape hatch
- Same as Option B, plus an `untrack(|| signal.get())` helper that
  reads a Signal without adding a dependency.

- What you gain: Future flexibility for derived bindings that want
  to read a value without subscribing to its changes (e.g. logging
  a current value at update time).
- What you give up: Adds API surface with no M2 binding shape that
  needs it. Counter's Text binding tracks every Signal it reads;
  there is no escape-hatch case. Reads outside any Effect (e.g. an
  imperative host call into the runtime that incidentally calls
  `Signal::get()`) are already untracked — the thread-local stack
  is empty, so no dependency is recorded. Option C's escape hatch
  duplicates that behaviour from inside Effect bodies.
- **Technical risk: Low** mechanically, but the design risk is
  premature optionality — Solid's `untrack` exists for derivation
  cases (Computed reading a Signal it shouldn't subscribe to) that
  M2 doesn't have.

**Recommendation:** **Option B.**

Read-time auto-tracking is the canonical Solid/Vue pattern and
integrates cleanly with the existing handler evaluator core: a
`BindingEvalContext` wrapping the property storage emits
`Signal::get()` calls during expression evaluation, and the
thread-local effect stack picks up the dependencies without Phase 6
or the binding author writing dependency lists. Option A's
explicit-declaration scheme front-loads work onto Phase 6 that the
runtime can do once for free, and is unsound for M3 conditional
bindings; deferring its rejection to M3 would just mean rewriting
Phase 5 then. Option C's `untrack` is additive over Option B and
can land in M3 if a concrete case appears.

The thread-local stack lives in `wasamo-runtime/src/reactive.rs`
alongside `with_batched_writes`. Reads outside any Effect (the
common case for handler-side state writes feeding into binding
re-evaluation, and for host code that happens to query a property
through the experimental layer) are simply untracked: the thread-
local is `None`, `Signal::get()` short-circuits.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are M3 conditional / structural bindings, M3 Computed,
and the post-M2 binding-author-facing API (M3 binding grammar in
the DSL spec).

- Option A's explicit-dependency form is wrong for conditional
  bindings; M3 has to either replace the scheme or layer auto-track
  on top, leaving two parallel forms.
- Option B is forward-compatible with all three: Computed becomes
  an effect-like node that *also* tracks reads of its inputs and
  exposes a `get()` of its own; conditional bindings work as-is
  because dependency sets are recomputed each run; M3 binding
  grammar is decoupled from the runtime mechanism (the grammar
  decides what reads are expressible; the runtime tracks whatever
  is read).
- Option C is forward-compatible like Option B but commits to the
  `untrack` escape hatch shape now, with no M2 case to validate it.

This axis reinforces Option B: maximum forward additivity at no
additional M2 cost.

**Technical-risk re-evaluation:** Option B's risk is bounded to the
adjacent DDs (003 / 004) and is the same risk Solid's prior art has
already exercised at scale. Option A's M2 risk is low but its M3
rework cost is high. Option C is Option B plus a speculative API.
Risk reinforces Option B.

---

### DD-M2-P5-003 — Effect lifetime and disposal

**Status:** Proposed

**Context:**
DD-M2-P5-001 = B introduces Effects (re-runnable closures) registered
with the engine. Each Effect holds a reference into the dependency
graph (Signals point at it; on Signal write, dirty marks propagate to
it). When the bound widget is removed from the tree (Phase 4
`remove_child` / `replace_child` / `widget_destroy`), the Effect must
be disposed so:

1. Signals stop pushing dirty marks at a defunct Effect.
2. The Effect's closure (which captures references into widget
   property storage) is dropped before the widgets it captures.
3. A re-attach of the same widget to a different parent does not
   resurrect a stale Effect.

This DD decides whose responsibility disposal is and how it is
threaded through the existing widget lifecycle.

**Options:**

Option A — Effects are owned by the widget that hosts the binding (recommended)
- Each `WidgetNode` gains a `bindings: Vec<EffectHandle>` field
  ([wasamo-runtime/src/widget.rs](../../wasamo-runtime/src/widget.rs)).
  Phase 6's IR loader, when it lowers `Text { content: "..." }`,
  creates an `Effect` and pushes its handle onto the widget's
  `bindings`.
- Disposal is automatic on widget drop: `Drop for WidgetNode`
  iterates `bindings` and calls `engine.dispose_effect(handle)`,
  which removes the Effect from every Signal's dependent set and
  drops the closure.
- `wasamo_widget_destroy` (Phase 4) and `wasamo_window_destroy`
  drop subtrees through Box ownership, so binding disposal piggy-
  backs on the existing teardown sweep with no new ABI surface.
- The Phase 4 `attached: bool` flag is unrelated to effect
  registration: an Effect is registered the moment the IR loader
  creates it (regardless of whether the widget is yet attached to a
  window). The dependency graph holds the effect live until the
  widget Drop-fires.

- What you gain: Disposal is structural, not bookkeeping —
  ownership of the Effect mirrors ownership of the widget, and
  every existing teardown path (window destroy, widget destroy,
  remove_child + drop) handles bindings without new code paths.
  No "leaked Effect whose target widget is gone" failure mode.
- What you give up: Each `WidgetNode` carries one extra `Vec`
  field (often empty in M2 — only Text widgets with bound content
  get an entry). Trivial.
- **Technical risk: Low.** Existing `Drop` paths and the Phase 4
  subtree-teardown sweep are the integration surface; both already
  exist. The new code is a single iterator in `Drop for WidgetNode`
  plus the `dispose_effect` engine method.

Option B — Effects are owned by the engine; widgets reference by handle
- The engine maintains the authoritative `HashMap<EffectId, Effect>`.
  Widgets store an `EffectId` (an opaque integer); on widget drop,
  some external mechanism is responsible for telling the engine to
  free the corresponding entry.
- The "external mechanism" is either: (a) a Drop impl that calls
  `engine.dispose_effect(id)` (functionally identical to Option A),
  or (b) a sweep at outermost-frame boundaries that walks live
  widgets and reaps orphaned Effects.

- What you gain: Centralised registry shape — useful if Effects
  ever need to be enumerated by the engine (e.g. for a "force
  flush all" debug command).
- What you give up: Sub-option (a) is Option A in disguise; sub-
  option (b) requires the engine to walk the widget tree, which is
  the kind of registry-with-no-clear-owner pattern M2 has been
  avoiding (cf. DD-M2-P4-003 = A's rejection of the limbo registry).
  Adds an integer-handle layer with no benefit Option A doesn't
  also have.
- **Technical risk: Low–medium.** Sub-option (b) introduces a
  reaper sweep that has to run at the right moment and can leak if
  the trigger is missed. Sub-option (a) is just Option A with
  extra indirection.

Option C — Manual disposal via explicit `unbind` calls
- Phase 6's IR loader returns `EffectHandle` to the host (or to a
  binding-tracking layer); explicit cleanup is required at widget
  removal time.

- What you gain: Maximum control.
- What you give up: Phase 6 has to generate disposal calls for
  every `remove_child` / `replace_child` it emits, doubling the
  per-mutation work and risking leaks. Re-attach (M3 conditional
  bindings will rebuild subtrees) becomes "destroy old effects,
  rebuild new effects" with no help from the structural mechanism.
- **Technical risk: Medium.** Manual disposal scales poorly with
  M3 structural bindings. Rejected.

**Recommendation:** **Option A.**

Effect ownership mirrors widget ownership: an Effect bound to a
widget's property is owned by that widget and disposed when the
widget drops. This makes binding lifecycle structural rather than
bookkept — every existing teardown path (Phase 4
`wasamo_widget_destroy` subtree sweep, `wasamo_window_destroy`
whole-tree drop, plain `remove_child` + `drop`) handles binding
disposal correctly with no new ABI and no engine-level reaper.

The `WidgetNode.bindings` field is `pub(crate)` (no C ABI
exposure); Phase 6's IR loader populates it during construction.
The `Drop for WidgetNode` impl forwards each handle to
`reactive::dispose_effect`, which removes the effect from every
Signal's dependent set and drops the closure.

Re-attach (M3 conditional binding rebuilds a subtree at a different
position) just creates fresh Effects on the new widgets; old widgets
go through normal Drop, which disposes their old Effects. No
explicit hook needed.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are M3 structural bindings (conditional / for-loop, which
rebuild subtrees), M3 Computed (which has its own lifetime), and
post-1.0 hot reload (which destroys whole graphs at once).

- Option A's structural ownership extends naturally: Computed nodes
  are owned by whoever creates them (an enclosing Effect, or the
  engine if they outlive the cycle); structural-binding subtree
  rebuilds dispose old Effects via Drop and create new ones; hot
  reload's whole-tree teardown disposes everything via root drop.
- Option B (sub-option b reaper) accumulates risk per future shape:
  Computed adds another registry, structural bindings add another
  trigger, hot reload adds another sweep moment.
- Option C does not scale to structural bindings without additional
  scaffolding.

This axis reinforces Option A: ownership-first design composes with
foreseeable growth without engine-side bookkeeping.

**Technical-risk re-evaluation:** Option A's risk is the smallest;
the integration surface is existing Drop paths plus one engine
method. Option B's reaper sub-option introduces correctness risk
without acceptance benefit. Option C's manual disposal is high-cost
and scales poorly. Risk reinforces Option A.

---

### DD-M2-P5-004 — Reactive dispatch timing

**Status:** Proposed

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

### DD-M2-P5-005 — Phase 6 binding registration API surface

**Status:** Proposed

**Context:**
Phase 6 (`.ui → runtime` lowering) consumes the textual IR and
constructs the widget tree, attaching bindings as it goes. The
binding-registration call shape is the API surface between Phase 5
(reactive engine internals) and Phase 6 (IR loader). Getting it
right matters because:

- Phase 5 wants the registration shape minimal enough that the
  engine can change internals freely.
- Phase 6 wants the shape ergonomic enough that the IR loader's
  binding emission is a few lines per binding shape, not a section.
- M3 will add binding shapes (Computed, conditional, for-loop);
  the M2 registration API should not be the limiting factor.

**Options:**

Option A — One generic `register_binding(target, expression)` (recommended)
- Single internal Rust API:
  ```rust
  pub(crate) fn register_binding(
      target: BindingTarget,
      expr: HandlerExpr,  // shared with Phase 3 evaluator
  ) -> EffectHandle;
  ```
  where `BindingTarget` enumerates the property-write sink (in M2:
  `WidgetProperty { widget: WidgetNodeRef, property_id: u32 }`;
  M3 may add `ConditionalSubtree`, `ForLoopSubtree`, etc.).
- The engine wraps `expr` in an Effect closure that evaluates `expr`
  with a `BindingEvalContext` (read-only, dependency-tracking) and
  writes the result into `target`. Dependency collection is automatic
  per DD-M2-P5-002 = B.
- Phase 6 emits one `register_binding` call per `Text { content:
  "..." }` (or similar bound property); the IR loader does not need
  to know about Effects, Signals, or the dependency graph.

- What you gain: Phase 6's binding emission collapses to a
  one-liner per binding. The `HandlerExpr` reuse from Phase 3 means
  no new IR-side expression language for bindings; binding
  expressions and handler bodies are the same AST (with binding
  context disabling assignment statements). M3 binding shapes
  (conditional / for-loop) become new `BindingTarget` variants; the
  registration API itself does not change.
- What you give up: `BindingTarget` is an internal enum; future
  variants are not pre-specified. M3 has to add them, but additive
  changes to a `pub(crate)` enum are mechanically free.
- **Technical risk: Low.** The wrapping closure is mechanical; the
  evaluator is the existing handler evaluator with a different
  context.

Option B — Per-target-shape registration functions
- Multiple internal APIs:
  ```rust
  pub(crate) fn bind_text_content(widget, expr) -> EffectHandle;
  pub(crate) fn bind_button_label(widget, expr) -> EffectHandle;
  // ... one per bindable property kind
  ```

- What you gain: Each function can validate that the expression's
  result type matches the target property type at registration
  rather than at first run.
- What you give up: API count grows linearly with bindable property
  count. Type validation is also achievable in Option A by having
  `BindingTarget` carry the expected `PropertyValueKind` and
  checking once at registration.
- **Technical risk: Low** mechanically; design risk is API
  proliferation for no acceptance benefit.

Option C — Phase 6 builds the Effect closure itself
- Phase 5 exposes `Signal::get` / `Signal::set` and an
  `Effect::create(body: Box<dyn FnMut()>)` constructor; Phase 6
  builds the binding closure manually:
  ```rust
  let widget_handle = ...;
  let count_signal = ...;
  Effect::create(Box::new(move || {
      let v = count_signal.get();
      widget_handle.set_property(TEXT_CONTENT, format!("Count: {v}"));
  }));
  ```

- What you gain: Maximum flexibility — Phase 6 can compose any
  binding shape it wants from primitives.
- What you give up: Phase 6 carries the IR-walking *and* the
  closure-building responsibility. The textual IR's `Bind` form
  has to be lowered to a Rust closure at runtime, which means the
  IR loader has to interpret the binding expression itself rather
  than handing it off to the evaluator. Forces Phase 6 to build a
  parallel evaluation path; defeats the handler/binding evaluator-
  core sharing.
- **Technical risk: Medium.** The duplicate evaluation path is the
  risk; not in implementing it, but in keeping it consistent with
  the handler evaluator over future evolutions.

**Recommendation:** **Option A.**

The single `register_binding(target, expr)` call collapses the
Phase 5 / Phase 6 boundary to its smallest meaningful shape. The
engine internals (dependency tracker, dirty-set, drain loop) stay
fully `pub(crate)`, free to evolve in M3 without an API break.
Phase 6's IR loader emits one call per binding, with the same
`HandlerExpr` AST it already produces for inline handlers.

The `BindingTarget::WidgetProperty` variant carries enough
information for the engine to perform the write through the
existing `set_property` path; size-affecting properties trigger
layout invalidation as DD-P8-002 already arranges. M3 binding
shapes add `BindingTarget` variants without disturbing this
function's signature.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are M3 conditional / for-loop bindings, M3 Computed,
and M3 DSL spec finalisation (which decides binding-expression
grammar).

- Option A's `BindingTarget` enum is the natural extension point
  for M3 structural bindings — variants are added; existing
  callers are unaffected. M3 grammar additions land in
  `HandlerExpr` (the existing AST), not in the registration API.
- Option B's per-shape functions multiply for every M3 binding
  kind; renaming or generalising them later means churning Phase 6
  call sites.
- Option C externalises the binding-evaluation path, so M3 grammar
  changes (e.g. function-call expressions, ternary) require Phase
  6 to update its closure-builder in lockstep with the evaluator.

This axis reinforces Option A: minimal API, additive on the
M3-foreseeable axes.

**Technical-risk re-evaluation:** Option A is the lowest-impl-risk
of the three. Option B's risk is design-shape API proliferation.
Option C's risk is parallel-evaluator divergence. Risk reinforces
Option A.

---

### DD-M2-P5-006 — Verification strategy

**Status:** Proposed

**Context:**
[docs/notes/headless-verification.md](../notes/headless-verification.md)
records the M2 stance: do not build a general-purpose headless
backend; cover pure-logic surfaces with phase-specific test fixtures;
GUI-observable behaviour is verified by manual exercise on a visible
desktop. Phase 5 is the trigger phase that note flagged for
re-evaluation: "Phase 5 着手時に reactive 経路の検証が unit test
単独で覆えるか再評価". This DD answers the trigger.

The reactive engine has a large pure-logic surface (Signal storage,
dependency tracker, dirty-set, drain loop, evaluator wiring) and a
small Visual-Layer-bound surface (the bound widget actually renders
the new text). Phase 4 established a precedent (Slot/Children mirror
test pattern, [CLAUDE.md](../../CLAUDE.md) testing rule's optional
mirror clause); this DD decides how far to lean on it for Phase 5.

**Options:**

Option A — Pure-logic only; no new mirrors; GUI manual confirms end-to-end (recommended)
- Test surface: Signal `get`/`set`, Effect creation/disposal,
  dependency-graph mutation across re-runs, dirty-set drain loop
  (including iteration-cap), `with_batched_writes` deferral,
  `BindingEvalContext` over `HandlerExpr` (read-only mode rejects
  writes; reads register dependencies). Effects are tested with
  closure bodies that record observable side effects into a
  test-side `Vec` — no widget property writes in unit tests.
- The Phase 4 Slot/Children mirror pattern is **not** extended;
  Phase 5 does not introduce widget-tree mutation that would need
  it. The Effect closures that, in production, write through
  `set_property` are stubbed in tests with closures that push to a
  log Vec.
- GUI verification: at Phase 5 close, run the M1 counter example
  through a Phase-5-aware code path (Phase 6 is not yet present,
  so a small experimental harness wires a `Signal` to a Text
  widget by hand) on a visible desktop and confirm `count++`
  updates the label. Recorded as Phase 5 GUI checkpoint in the
  m2-plan; A2 acceptance is fully confirmed at Phase 6 close
  (counter.ui-driven).

- What you gain: Stays inside [CLAUDE.md](../../CLAUDE.md) testing
  rules without further interpretation. Test fixtures are narrow
  and phase-local. No mirror struct to maintain. The pure-logic
  surface is large enough that a "binding evaluator over Signal +
  Effect with deferred drain" suite gives high confidence; the
  remaining GUI-observable bit (the widget actually re-renders)
  is exercised by the manual checkpoint and by Phase 6 e2e.
- What you give up: A2 is not closed by unit tests alone. The
  manual GUI checkpoint at Phase 5 close is the close criterion;
  CI green is necessary but not sufficient.
- **Technical risk: Low.** The test surface is pure Rust;
  closures-as-side-effect-loggers is a standard pattern. The
  manual checkpoint is the same shape as Phase 6's manual GUI
  verification — owner runs counter on RDP / physical desktop and
  observes the click → label update.

Option B — Extend the Phase 4 mirror pattern to cover bound widget property writes
- Add a test-only mirror of `WidgetNode` (or a narrow sub-struct)
  that supports `set_property` and records writes; bind an Effect
  to the mirror; assert the mirror's recorded write set after a
  Signal write + drain.

- What you gain: Tests demonstrate "Effect ran and wrote the
  property" rather than "Effect ran and incremented our counter
  closure" — closer in shape to the production path.
- What you give up: A new mirror that has to track `WidgetNode`'s
  property storage shape, drift risk against production, and
  maintenance burden as M3 adds property types. The Effect closure
  in production calls the same `set_property` function by name;
  testing through a mirror tests the bridging code, not the
  reactive engine. Diminishing-returns — the bridging code is one
  line per binding (Effect's closure is ~3 lines).
- **Technical risk: Low–medium.** The risk is mirror drift, the
  same issue that motivates the [CLAUDE.md](../../CLAUDE.md) rule
  to prefer extracting free functions over mirrors.

Option C — Build a "no-Compositor" runtime mode and integration-test through it
- Per [headless-verification.md (ii)](../notes/headless-verification.md):
  introduce a runtime mode where `WidgetNode` is fully constructed
  but no Compositor / Visual / DirectWrite is created. Tests
  exercise full property write → reactive drain → property store
  end-to-end without the OS surface.

- What you gain: Higher-fidelity tests; A2-shaped verification in
  CI (modulo the actual rendering bit).
- What you give up: A "Visual on / Visual off" two-mode runtime
  is exactly what
  [headless-verification.md](../notes/headless-verification.md)'s
  long-form analysis rejected for M2 — DD-V-001-era posture
  (no abstraction over Visual Layer) and infrastructure cost.
  Building it as a Phase 5 sub-task expands scope significantly
  and would need its own ADR.
- **Technical risk: Medium.** The risk is scope creep into a
  separate runtime mode that has its own design surface (which
  events fire, how the message loop is faked, which OS-bound
  paths short-circuit). Out of M2-Phase 5 budget; if needed,
  a separate vision-level ADR is the right shape.

**Recommendation:** **Option A.**

Pure-logic test fixtures are sufficient for the engine surface and
align with [CLAUDE.md](../../CLAUDE.md) testing rules without
further reinterpretation. The Phase 4 mirror pattern was used
sparingly (Slot/Children — small enough state to mirror without
drift risk); Phase 5 does not present a target small enough to
mirror cleanly without dragging in `WidgetNode`'s full property
storage. The GUI manual checkpoint at Phase 5 close, plus full A2
verification at Phase 6 close, completes the verification chain.

The binding-evaluator integration with `HandlerExpr` is itself
pure-logic-testable: a `BindingEvalContext` with mock storage,
fake signals, and assertion that read calls are tracked correctly.
This is where the Phase 5 testing lift is concentrated, and it is
done entirely without OS-bound types.

`headless-verification.md` is **not** updated to flag a new
trigger; the M2-stance "do not build a headless backend" survives
Phase 5 by virtue of Option A working.

**Forward-compat exposure:** Options differ. The relevant out-of-
scope items are post-1.0 hot-reload CI verification and post-1.0
binding-conformance test (Swift / Go community track).

- Option A leaves the door open to building a headless mode later
  if a foreseeable future event (hot reload in CI, binding
  conformance tests) demands it; the engine internals don't lock
  in any assumption that prevents a later headless mode.
- Option B's mirror pattern has the same forward-compat
  property; the difference is in the M2 cost, not the M3+ cost.
- Option C builds the headless mode now, which has the same
  long-term value but pulls scope forward into Phase 5 without an
  M2 driver.

This axis reinforces Option A: defer infrastructure that has no
M2 driver; the runtime architecture remains amenable to a
headless mode if a real driver appears.

**Technical-risk re-evaluation:** Option A's risk is the smallest;
the test fixtures are narrow and phase-local. Option B's mirror
drift is bounded but real. Option C's scope is out of phase
budget. Risk reinforces Option A.

---

## Summary of proposed decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P5-001 | Reactive primitive layering | **Option B** — Signal + Effect 2-layer; Computed deferred to M3 | Low–medium | Low |
| DD-M2-P5-002 | Dependency collection mechanism | **Option B** — read-time auto-track via thread-local effect stack | Low–medium | Low |
| DD-M2-P5-003 | Effect lifetime and disposal | **Option A** — owned by host widget; disposed via `Drop for WidgetNode` | Low | Low |
| DD-M2-P5-004 | Reactive dispatch timing | **Option B** — deferred to outermost-frame drain alongside observer + layout drains | Low–medium | Low |
| DD-M2-P5-005 | Phase 6 binding registration API | **Option A** — one generic `register_binding(target, HandlerExpr)`; `BindingTarget` enum | Low | Low |
| DD-M2-P5-006 | Verification strategy | **Option A** — pure-logic tests with side-effect-logger Effect closures; no new mirrors; GUI manual checkpoint | Low | Low |

**Aggregate impl-risk picture.** The non-trivial impl-risk axes are
DD-M2-P5-001's 2-layer engine bookkeeping, DD-M2-P5-002's thread-
local effect stack invariants, and DD-M2-P5-004's drain-loop
quiescence. All three are bounded: the engine is `pub(crate)` and
freely revisable; the thread-local stack is single-threaded by
construction (M2 is single-threaded throughout); the drain loop has
an iteration cap as a divergence trap. No DD introduces a mechanism
the runtime hasn't already exercised in shape (DD-P6-003's
queued-emission drain is the model for DD-M2-P5-004's reactive drain;
DD-M2-P3-001's `EvalContext` is the model for DD-M2-P5-002's
`BindingEvalContext`; Phase 4's subtree teardown is the model for
DD-M2-P5-003's binding disposal).

**Aggregate forward-compat exposure.** All six DDs recommend the
M3-additive option. Phase 5's runtime delta is intentionally
internal: no new C ABI symbols (DD-M2-P4-004 = A precludes that),
one new public-to-Phase-6 internal Rust function
(`register_binding`), one new `pub(crate)` module
(`reactive` already exists from Phase 4). M3's Computed and
structural-binding work lands as additions to `BindingTarget`,
to the drain loop's pre-Effect topological pass, and to the
reactive module — not as rewrites.

**Pre-doc validation spike.** Not required. The handler evaluator
([wasamo-runtime/src/handler.rs](../../wasamo-runtime/src/handler.rs))
is the pre-existing reference for the evaluator-with-context shape
DD-M2-P5-005 = A reuses; the queued-emission drain
([wasamo-runtime/src/abi.rs:369](../../wasamo-runtime/src/abi.rs#L369))
is the pre-existing reference for the deferred-dispatch shape
DD-M2-P5-004 = B extends; the `with_batched_writes` skeleton
([wasamo-runtime/src/reactive.rs:18](../../wasamo-runtime/src/reactive.rs#L18))
is the integration point already in place. The Solid.js / Vue ref
prior art is broadly understood and exercised at scale; no spike is
needed to validate the 2-layer auto-tracking premise.

## Out of scope

- **`Computed<T>` derived primitives.** Deferred to M3 alongside the
  DSL spec public draft, which decides derivation grammar. The
  Phase 5 architecture (DD-M2-P5-001 = B) is shape-compatible with
  a future Computed layer added between Signal and Effect; the
  drain loop (DD-M2-P5-004 = B) extends with a pre-Effect
  topological re-eval pass.
- **Structural bindings (conditional / for-loop / list-rendered).**
  Deferred to M3. M2 binds property values only; subtree shape is
  static. M3 adds new `BindingTarget` variants; DD-M2-P5-005's
  registration API is unchanged.
- **Subtree-grain layout dirty.** Phase 5 inherits DD-P8-002's
  whole-window dirty path; finer granularity remains the open
  question in [layout-engine note §3.4](../notes/layout-engine.md)
  and is revisited only if M2 acceptance demands it (it does not).
- **Host-visible reactive API.** No C ABI symbol is added (per
  DD-M2-P4-004 = A). Hosts in M2 do not interact with the reactive
  engine directly; they observe its effects through the existing
  property-set + observer machinery.
- **`untrack` / read-without-subscribe escape hatch.** Deferred to
  M3 when Computed adds use cases. DD-M2-P5-002 = B's auto-track
  is shape-compatible with an additive `untrack` helper.
- **Explicit `engine.flush()` primitive.** Not added; the drain
  trigger remains the outermost-frame boundary (DD-M2-P5-004 = B).
  M3+ may revisit if a host-driven flush case appears.
- **Headless / "no-Compositor" runtime mode.** Per DD-M2-P5-006 = A
  and [headless-verification.md](../notes/headless-verification.md);
  not built in M2. Re-evaluation triggers (post-1.0 hot reload CI;
  binding-conformance tests) remain as recorded in that note.
- **Multi-threaded Signal access.** M2 is single-threaded; the
  thread-local effect stack assumes single-threaded use. Multi-
  thread support is post-1.0 and would need its own ADR.
- **Updating A4's "required by the reactive engine" wording.**
  Already deferred per the M2-Phase 4 ADR's Out of scope item.
  Phase 5's implementation does not change the situation: the
  reactive engine remains internal Rust and does not call across
  the C ABI. If a vision ADR rewriting A4 is desired, it remains
  a one-line documentation change; Phase 5's work product is
  unchanged either way.
