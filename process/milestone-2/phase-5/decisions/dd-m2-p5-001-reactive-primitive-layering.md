### DD-M2-P5-001 — Reactive primitive layering

**Status:** Accepted

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
