### DD-M2-P5-002 — Dependency collection mechanism

**Status:** Accepted

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
