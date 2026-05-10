# M2-Phase 7 — Reactive Foundation Hardening & Contract Finalization: Architecture Decisions

**Phase:** M2-Phase 7 (Reactive Foundation Hardening & Contract Finalization)
**Date:** 2026-05-08 (ADR opened; DDs remain Proposed pending per-DD pre-doc cycles); 2026-05-09 (DD-M2-P6-010 Accepted; DD-M2-P6-010 minor implementation clarification recorded); 2026-05-10 (DD-M2-P6-012 Accepted; DD-M2-P6-011 Accepted); 2026-05-11 (M2 completed)
**Status:** Accepted (DD-M2-P6-010 / 011 / 012)

## Context

M2 acceptance criteria **A5** (Reactive Foundation Hardening) and **A6**
(Type-Agnostic Reactive Binding) — added by the 2026-05-08 acceptance-
criteria revision recorded in
[m2-plan.md](../plans/m2-plan.md#progress) — are discharged by this
phase. Phase 6 closed A1/A2 (counter `.ui` drives the running counter
through the reactive path end-to-end); Phase 7 closes A5/A6 by
upgrading the foundation guarantees that distinguish "the pipeline
runs" from "the pipeline is a Foundation other layers can rely on".

This ADR houses three DDs that were drafted as part of the Phase 6 ADR
slate but deferred at Phase 6 closing because their resolution depends
on full Phase 6 implementation evidence rather than mid-implementation
judgement:

- **DD-M2-P6-010** — `dirty_effects` topological sort fidelity. Surfaced
  during the DD-M2-P6-001 implementation retrospective (2026-05-07).
- **DD-M2-P6-011** — String-typed property binding. Surfaced during the
  DD-M2-P6-007 implementation step (2026-05-07).
- **DD-M2-P6-012** — Re-entrancy and safety-guard placement principle.
  Surfaced from the Phase 5 retrospective; deliberately deferred so the
  full Phase 6 set of re-entrancy states (Diverged / IN_DRAIN /
  IN_OBSERVER_CALLBACK / UI-thread confinement) is available as
  evidence.

The DDs retain their original `DD-M2-P6-NNN` numbering. The number is a
historical identifier that records when the issue surfaced; the ADR
file is the housing for resolution. Decoupling the two avoids
renumbering churn across `docs/plans/m2-plan.md`, prior memory, and
git history.

### Order of work (agreed 2026-05-08)

Per the Phase 7 entry in [m2-plan.md](../plans/m2-plan.md), the three
DDs are processed sequentially as independent pre-doc cycles, in the
order:

1. **DD-M2-P6-010** (topo sort fidelity) — pre-doc → agreement →
   Accepted → implementation. Discharges the Phase 6 closing
   constraint that any multi-binding work must replace the
   `EffectId`-numeric-order approximation.
2. **DD-M2-P6-012** (guard placement principle) — pre-doc → agreement
   → Accepted → `architecture.md` update; the principle is recorded
   as a global runtime invariant. Discharges A5.
3. **DD-M2-P6-011** (String-typed property binding) — pre-doc →
   agreement → Accepted → implementation. Discharges A6 with an
   end-to-end `Signal<String>` binding demonstration.

The order reflects a deliberate sequencing choice: 010 settles the
ordering primitive that A6's binding evaluator depends on; 012
establishes the placement principle that any new evaluator code in
011 should follow; 011 lands last so it benefits from both. The
sequence is not implied by the DD numbers and is not reversible
without re-pre-doc.

### Side obligations carried in

- DD-M2-P6-012 acceptance must update `docs/architecture.md` to record
  the chosen guard-placement principle as a global runtime invariant.
  This update lands in the same commit that flips DD-M2-P6-012 to
  `Accepted`.
- DD-M2-P6-010's mandatory-pre-condition language (currently in the
  Phase 6 ADR's Forward-compat exposure paragraph) is reconciled at
  010 acceptance. Under the recommended Option A, the constraint is
  discharged at acceptance time (the walk lands in M2; M3 inherits it).
  The reconciled text replaces the Phase 6 paragraph in the same
  commit that flips DD-010 to `Accepted`.

---

### DD-M2-P6-010 — `dirty_effects` topological sort fidelity

**Status: Accepted (2026-05-09)**

#### Context

DD-M2-P6-001 = Option D specifies Phase 1 ordering as
"topological-by-dependency-graph" for `dirty_effects`. The M2
implementation in `drain_dirty_effects()` uses `sort_unstable()` on
`EffectId` values, which are monotonically increasing integers assigned
at Effect creation time. The numeric-ID order coincides with
topological order **only** under the precondition: every Effect's
dependencies were created before the Effect itself. The runtime does
not enforce this precondition; it holds in M2 because the IR loader
emits `state` declarations before the binding Effects that read them,
and the counter shape has exactly one Effect per Signal.

The approximation holds for the M2 acceptance set (single binding, one
handler, one reactive value). It breaks silently as soon as two Effects
depend on the same Signal in a non-trivial order, or when an Effect
created earlier is a downstream consumer of one created later. The
correctness condition "ID-numeric ≡ topological" is therefore a
property of the IR loader's emit discipline, not a structural property
of the runtime — a distinction that matters once Phase 7 evaluates the
gap against a structural criterion.

This gap surfaced during the DD-M2-P6-001 implementation cycle and was
recorded as `Proposed` for later settlement; the Phase 6 ADR draft and
working notes through DD-M2-P6-007 carried **Option B** (defer to M3
pre-doc with an explicit constraint record) as the drafter
recommendation, on the criterion "M2-acceptance-observable". Phase 7
re-evaluates DD-010 under acceptance criterion **A5** (Reactive
Foundation Hardening), whose operative clause is "the implementation
no longer relies on the counter case happening to converge". Read
literally, A5 is a property of the **shipped binary**: any option that
ships M2 with a runtime path whose correctness depends on the counter
shape happening to satisfy the ID-order precondition does not
discharge A5 by its own wording. The criterion change — not new
technical evidence — is what reopens the recommendation.

#### Options

**Option A — True topological walk in M2.**
Replace `sort_unstable()` with a Kahn-style topological walk over
`ReactiveGraph::forward` / `back` restricted to the dirty set, extracted
as a free function so it is exercisable by pure-logic unit tests on
synthetic dependency graphs.

- Pro: spec-faithful in the shipped binary; A5 is discharged by
  implementation rather than by documentation. M3 multi-binding
  inherits a verified primitive whose properties have been characterised
  before its production consumer exists — a Phase 6 pattern (introducing
  a primitive ahead of its single consumer was already accepted in
  DD-M2-P6-009 with the `IrLoadError::is_malformed()` helper).
- Pro: the extraction-into-free-function form is reachable from pure
  Rust tests under the project's testing rules; this is the same
  test-seam discipline already established in Phase 6 (DD-M2-P6-005's
  `__install_owning_thread_for_test`). Pure-logic tests are the only
  available stimulus for ordering correctness, since no M2 GUI surface
  exercises multi-Effect ordering.
- Con: the walk ships before any production caller exercises it on a
  non-trivial graph. M3 may discover constraints (cycle handling,
  ordering ties, fan-out interaction with `MUTATION_CAP`) that force
  redesign. The "implementation untested at GUI level" risk class is
  not abstract — Phase 6 surfaced at least one realised instance
  (DD-M2-P6-006: a runtime difference between `add_widget` and
  `set_root` invisible to source review, caught only at GUI execution).
  Mitigation is constrained to what synthetic unit tests can express;
  the residual exposure is real but bounded by the algorithm's small
  surface and the named M3 stimulus.

**Option B — Defer to M3 pre-doc; revise A5 to design-level discharge.**
Keep `sort_unstable()` in M2. Record the constraint here. Make
"replace with true topological walk" a mandatory pre-condition for
M3 multi-binding. **Revise A5** in `m2-plan.md` to drop the
shipped-binary clause: A5 becomes "DD-010 Accepted with the
constraint recorded; M3 is gated on its discharge".

- Pro: smallest M2 code change. No new code path ships without a
  production exercise.
- Con: requires an A5 wording revision that visibly weakens
  Phase 7's "Foundation Hardening" framing. The revision is a
  legitimate design move, but it changes what A5 promises.
- **A5 discharge under the literal reading: not satisfied.** The
  shipped binary still relies on the counter case happening to converge
  (i.e. on the IR loader emitting `state` before bindings). Recorded
  here for comparison; the comparison rests on whether the criterion
  itself is up for revision.

**Option C — Verified approximation (debug-mode walk + structural
invariant).** Keep the `sort_unstable()` fast path. Add a
`debug_assert!`-gated walk of `forward` / `back` that verifies the
ID-sorted order is in fact topological for the current dirty set.
Optionally add a structural invariant at Effect creation: the runtime
asserts that every newly-tracked Signal's existing dependents have IDs
less than the new Effect's ID, making the "ID order ≡ topological
order" precondition checked at the point it could be violated.

- Pro: smaller code surface than Option A; debug-mode evidence is
  stronger than Option B's documentation-only record.
- Con: **A5 discharge is contestable.** The release binary still runs
  the EffectId-numeric `sort_unstable()` path; the verifier is compiled
  out. Under A5's literal reading, what ships still relies on the
  counter case happening to converge — only the debug build has stronger
  evidence.
- Con: the structural-invariant sub-variant changes Effect creation
  semantics in ways that interact with M3 features not yet designed
  (lazy computed values, host-driven creation order in cross-widget
  bindings). This is the "implementation locality ≠ design locality"
  failure mode Phase 6 explicitly recorded (DD-M2-P6-007: a single
  method addition that constituted an unrecognised design commitment).
  The sub-variant commits the runtime to an Effect-creation-order
  discipline before M3 has chosen its constraints.
- Con: two code paths (cheap sort + verifier) where Option A has one;
  the verifier is itself a topo walk, so implementation cost overlaps
  with Option A without delivering Option A's release-mode correctness.

**Option C-lite — Assertion only, no structural invariant.**
A narrower form of C that adds only the `debug_assert!` walk in
`drain_dirty_effects()` and does not touch Effect creation.

- Pro: minimal code change.
- Con: **A5 discharge weaker than C.** The release binary still runs
  `sort_unstable()`; the debug assertion only proves the precondition
  holds for the cases the test/run exercises, which for M2 is exactly
  the counter case "happening to" converge. Under A5's literal wording
  this fails the same way Option B does, with extra debug-mode evidence
  as a fig leaf.

#### Recommendation

**Option A.** A5's literal reading restricts the recommendation space
to options that ship a structural correctness guarantee in the release
binary; Option A is the only entry that delivers one without spawning
a release/debug correctness asymmetry. The two natural objections to
Option A — "ships unexercised" and "no GUI confirmation" — are not
unique to this DD; they are a known Phase 6 risk class with at least
one realised instance, and the project's response to that class has
been the pure-logic test-seam pattern, already established and
accepted. Option A's extraction-into-free-function form lets the topo
walk inherit that pattern directly. The residual risk (algorithm
constraints discovered only at M3 stimulus) is real but bounded;
Options B / C / C-lite do not eliminate it, they only relocate it
into the M3 pre-doc cycle while either weakening A5 (B) or accepting
a release/debug asymmetry (C, C-lite).

Options B / C / C-lite are recorded above as the considered
alternatives, with their A5-discharge analysis worked out.

##### A5 interpretation grounding the recommendation

A5's operative clause — "the implementation no longer relies on the
counter case happening to converge" — is read here as a property of
the **shipped release binary**, not of the design record or the debug
build. Concretely:

- An implementation discharges A5 only if its release-mode behaviour
  is correct on dependency-graph shapes the M2 counter case does not
  cover. Documentation that *would* be correct, or assertions that
  *would* fire in debug, do not satisfy the clause.
- Coincidence between numeric ID order and topological order — the
  basis on which `sort_unstable()` happens to work in M2 — is exactly
  the form of "happening to converge" the clause names. An option
  that ships that coincidence as the production path does not
  discharge A5 by its own wording, regardless of how the design is
  documented.

This literal reading is what flips DD-010's recommendation from the
Phase 6 draft's Option B to Option A. It is recorded explicitly so
that future readers can evaluate the recommendation against the
criterion that produced it, and so that any future relaxation of A5
makes the dependency on this reading visible.

##### Required form of the implementation

Adoption of Option A is conditioned on the implementation taking the
following shape; deviation from any of these requires a new pre-doc
cycle, not an in-step adjustment.

1. **Free-function extraction.** The topological walk is implemented
   as a free function whose inputs are `&forward`, `&back`, the
   write-edge map, and the dirty set (or equivalent graph borrows).
   It must not require a `Compositor`, a Win32 / WinRT handle, or any
   state owned by the ABI layer. The function is the unit of
   verification.
2. **Mandatory synthetic-graph unit tests.** Coverage of the free
   function by pure-logic unit tests on synthetic dependency graphs
   is a precondition of step acceptance, not a follow-up task. The
   test set must include, at minimum: a chain (`a → b → c`), a
   diamond (`a → {b, c} → d`), a fan-out shape exercising
   `MUTATION_CAP` interaction, and an out-of-ID-order shape (an
   Effect with a smaller ID that depends on one with a larger ID —
   the case the M2 counter never produces). Cycle handling is named
   in Forward-compat exposure below.
3. **Single drain code path.** `drain_dirty_effects()` calls the
   extracted walk; the existing `sort_unstable()` is removed, not
   retained as a fast path. There is no release/debug behavioural
   asymmetry. (This is what distinguishes Option A from Option C and
   C-lite at the implementation level.)

#### Forward-compat exposure

**Phase 6 pre-condition: discharged by adoption.** The Phase 6 ADR
carried a Forward-compat paragraph naming "replace `sort_unstable()`
with a true topological walk" as a mandatory pre-condition for M3
multi-binding. With Option A adopted in M2, that pre-condition is
**satisfied at acceptance time**: the walk exists; M3 inherits it.
The reconciled text replaces the Phase 6 paragraph in the same commit
that flips DD-010 to `Accepted`.

**Residual M3 obligations created by Option A.** Adopting the walk in
M2 settles the *ordering primitive*, but does not by itself settle
every property M3 multi-binding will require of it. The following
items are explicitly handed to the M3 pre-doc cycle and must be
recorded against the M3 roadmap, not absorbed silently into M3
implementation:

1. **Cycle detection policy.** A Kahn-style topological walk is
   well-defined only on a DAG. The M2 counter case has no cycles by
   construction; the M2 free-function unit tests assert acyclic
   shapes. M3 multi-binding can in principle introduce cycles
   (e.g. two Signals that bind through each other's expressions).
   The M3 pre-doc must decide whether cycles are (a) prevented at
   IR-load time by a structural rule, (b) detected at runtime and
   surfaced as `WASAMO_ERR_REACTIVE_DIVERGED` (or a new error
   code), or (c) rejected at `wasamoc` lowering time. Until M3
   chooses, the M2 walk's behaviour on a cyclic input is
   **undefined-but-bounded**: the unit tests cover acyclic inputs;
   if a cycle reaches the walk in production, the runtime is in a
   state DD-010 did not specify.
2. **Ordering ties.** Multiple Effects with no dependency
   relationship between them have no topologically-required order;
   the walk currently picks one. M3 must decide whether the chosen
   order is observable contract (e.g. by Signal-creation order, or
   ABI-explicit) or remains implementation-defined.
3. **Fan-out interaction with `MUTATION_CAP`.** The M2 walk runs
   inside a drain loop bounded by `MUTATION_CAP = 16`. M3 multi-
   binding may legitimately produce dirty sets large enough to
   probe this interaction; the cap may need to grow, become
   per-shape, or be replaced by a different convergence guarantee.
   This was already named as an open question in DD-M2-P6-001's
   divergence semantics; M3 inherits it.

These items are recorded as a new section in
[docs/notes/m2-to-m3-handover.md](../notes/m2-to-m3-handover.md) at
DD-010 acceptance time, alongside the existing carry-forwards
(`wasamo-ir` crate split, `HandlerExpr` unification). The handover
note's role is exactly this: surface design premises M3 must inherit
that are not derivable from the codebase or from the Phase 6 ADR's
Accepted DDs. The handover update lands in the same commit that
flips DD-010 to `Accepted`. ROADMAP.md and
[vision-post-m2-roadmap.md](./vision-post-m2-roadmap.md) are not
edited here; M3's pre-doc cycle is responsible for translating the
handover note into specific acceptance criteria when M3 opens.

---

### DD-M2-P6-011 — String-typed property binding

**Status: Accepted (2026-05-10)**

#### Context

DD-M2-P6-007 added `strings: HashMap<String, Signal<String>>` to
`SignalRegistry`, but the binding evaluator path (`BindingEvalContext` /
`HandlerExpr::PropRead` / `evaluate_tracked`) reads only `i32s`.
To support a `.ui` property whose source Signal is `String`-typed,
three gaps must be closed:

1. `EvalContext` trait needs `get_string(&self, path) -> Result<String, EvalError>`
   and `read_string_tracked` (dependency-tracking variant).
2. `BindingEvalContext` must implement both, routing through
   `registry.strings`.
3. `HandlerExpr` / `evaluate_tracked` must dispatch to
   `read_string_tracked` when the expression is a string-typed PropRead.

Gap 3 requires a disambiguation strategy: the evaluator currently treats
every `PropRead` as i32. This gap was surfaced during the DD-M2-P6-007
implementation step (2026-05-07) and deferred because resolving it requires
an IR design decision that is independent of the `SignalRegistry` shape.

#### Options

**Option A — Type-tag `PropRead` at the IR level.**
Add a `ty` field: `PropRead { path: String, ty: PropType }` where
`PropType` is `I32 | Str`. The loader sets `ty` at name-resolution time;
`evaluate_tracked` dispatches on `ty`.

- Pro: single variant; evaluator dispatch is one match arm per type;
  IR stays compact.
- Con: all existing `PropRead` construction sites gain a required field;
  a test-only `PropType::I32` default must be added or all tests updated.

**Option B — Introduce a `StrPropRead` variant (accepted).**
Add `HandlerExpr::StrPropRead { path: String }` alongside the existing
`PropRead`. `evaluate_tracked` dispatches `StrPropRead` to
`ctx.read_string_tracked`; existing `PropRead` path is unchanged.

- Pro: no change to existing `PropRead` construction sites or tests;
  the two read paths are structurally separated in the IR.
- Con: minor IR variant proliferation; conceptually redundant with `PropRead`.

**Option C — Unified `read_typed(path) -> TypedValue` on `EvalContext`.**
Replace `get_i32` / `get_string` with a single polymorphic method returning
a `TypedValue` enum. The evaluator extracts the arm it needs.

- Pro: one method handles all future types.
- Con: replaces the existing `get_i32` / `set_i32` API surface, requiring
  changes to all `EvalContext` implementors (including test stubs);
  `TypedValue` enum adds a dependency between `handler.rs` and a new type.

#### Recommendation

**Accepted: Option B.** Phase 7 pre-doc framing resolved A6 as
demonstrative rather than fully generic: M2 must prove the binding path is
not silently `i32`-specialized by carrying a `.ui` String property bound to
`Signal<String>` through to runtime widget property state, but M2 does not
require full `TypedValue` unification.

Adding `StrPropRead` is the smallest change that closes the three DD-011
gaps while preserving the existing integer `PropRead` path. Option A is
viable but forces every `PropRead` construction site to supply a type tag
today. Option C remains the future-friendly abstraction, but its blast
radius across `EvalContext`, handler evaluation, binding evaluation, test
stubs, and IR tooling is broader than the M2 acceptance pressure.

#### Implementation requirements

Acceptance of Option B carries the following implementation requirements:

1. Add a String read path to `EvalContext` (`get_string` plus tracked read)
   and route `BindingEvalContext` String reads through `SignalRegistry.strings`
   with dependency tracking.
2. Add the accepted String property-read representation
   (`HandlerExpr::StrPropRead { path }` under Option B) and dispatch it to the
   tracked String read path.
3. Provide a real `.ui` / emitted-IR path into the String read form based on
   the declared state type. A hand-written `StrPropRead` unit test alone does
   not discharge A6.
4. Add an automated test that proves a `.ui` or emitted-IR String binding
   reaches runtime widget property state without requiring a visible window,
   pixel inspection, or a mock Visual Layer. Actual on-screen confirmation
   remains part of the existing phase-close GUI/manual regression.
5. Preserve existing integer behavior: `PropRead { path }` remains the i32
   read form; bare integer binding, integer interpolation, and counter-style
   handler mutation are regression-protected.
6. Cross-type reads must fail rather than silently coerce. The exact
   diagnostic (`UnknownProperty` vs `TypeMismatch`) may follow the existing
   registry/error shape unless the implementation can report `TypeMismatch`
   without broad churn.

If implementation evidence changes any of these assumptions, the DD-011
implementation retrospective must record the deviation and update the
appropriate higher-level document (this ADR, the phase progress file, or a
live note such as `docs/notes/typed-value-evaluator.md`) rather than leaving
the design record stale.

#### Forward-compat exposure

Low for M2. `StrPropRead` is additive and leaves the existing integer
binding path intact. The main forward-compat exposure is not a hidden M2
requirement: if later DSL or tooling work introduces a third scalar property
type, typed item/context binding, non-string binding result values, or a
normative expression type system, the project must revisit whether parallel
typed reads are still appropriate. That open question is tracked in
[docs/notes/typed-value-evaluator.md](../notes/typed-value-evaluator.md).

---

### DD-M2-P6-012 — Re-entrancy and safety-guard placement principle

**Status: Accepted (2026-05-10)**

#### Context

DD-M2-P6-001 (Option D) specifies the **observable** post-divergence
ABI contract: while the runtime is in `Diverged`, every `wasamo_*`
call except `wasamo_runtime_destroy` must behave as a no-op returning
`WASAMO_ERR_REACTIVE_DIVERGED`. The runtime additionally carries
several re-entrancy-sensitive states defined across DD-M2-P6-001 and
DD-M2-P6-005:

- `Diverged` — terminal absorbing state; rejects all but destroy.
- `IN_DRAIN` — Phase 1 mutation convergence loop is active;
  structure-changing ABI returns `WASAMO_ERR_REENTRANT_LOAD`.
- `IN_OBSERVER_CALLBACK` — Phase 3 post-commit observer drain is
  active; state-mutating ABI returns `WASAMO_ERR_OBSERVER_MUTATION`.
- UI-thread confinement — non-UI threads reach the runtime and
  receive `WASAMO_ERR_WRONG_THREAD`.

What none of these DDs specify is the **architectural rule for where
these guards must be enforced in the call stack**: at the ABI boundary
(every exported `wasamo_*` function checks the relevant states at
entry), at the internal state-machine layer (the runtime's mutation,
read, and structure-changing primitives check, with the ABI as a
thin pass-through), or at both layers as deliberate defense in depth.
The current M2 implementation places guards case-by-case —
`check_not_diverged` lives at the ABI layer (`abi.rs`); the `IN_DRAIN`
and `Diverged` checks in `drain_if_outermost` live at the internal
layer (`emit.rs`); the overall pattern is implicit, not documented as
a rule.

The Phase 5 retrospective surfaced a concrete instance of the cost
of leaving the rule implicit. A code path entered through the Win32
message loop reached internal runtime state without crossing a
`wasamo_*` exported function, and therefore bypassed the
`check_not_diverged` guard. The local fix (add the missing check) is
straightforward and orthogonal to this DD; **the architectural issue
surfaced by the bug is that the codebase had no stated invariant the
implementer could have consulted to know whether that entry path
required a guard, nor a convention that would have made the omission
visible at review time.**

This is not a single-bug retrospective item. The same omission shape
recurs whenever a new entry path to runtime state is added that is
not a `wasamo_*` ABI function:

- M3 timer callbacks dispatched from a Win32 timer message.
- M3 async-I/O completions delivered on the UI thread.
- M3+ window-procedure subclassing for additional message types.
- Any future re-entrancy state layered on top of the existing four,
  whether that state is introduced in M3 or beyond.

Without a placement principle, each new state and each new entry
path becomes an independent local decision, and a missed case is
silent. A runtime that should have returned an error instead
executes against state that violates an invariant — with no
guarantee the violation surfaces as a recoverable ABI error rather
than a panic, an assertion failure, or silent state corruption that
manifests later as an apparently unrelated bug.

Re-entrancy and guard placement therefore belong to the same
category of architectural rule as the drain-transaction commit
discipline (DD-M2-P6-001) and the UI-thread-confinement contract
(DD-M2-P6-005): a global runtime invariant whose enforcement
strategy must be stated, not left to per-call-site discretion.
Establishing the principle now — before M3 introduces both new
re-entrancy states and new non-ABI entry paths — converts a class
of implementation oversights into a structural invariant that can
be reviewed and verified uniformly.

#### Options

**Option A — ABI-boundary guards as the single source of
enforcement.** Every exported `wasamo_*` function checks every
relevant runtime state at entry; internal modules trust that no
caller reaches them in a disallowed state. Non-ABI entry paths
(Win32 callback thunks, message-loop reactions, future timer / I/O
completions) must invoke the same guard helpers explicitly before
touching runtime state.

- Pro: enforcement responsibility is concentrated at a small,
  finite, auditable set of entry points; new ABI functions inherit
  a copy-paste-guarded entry pattern; the rule "ABI = guard,
  internal = trust" is easy to state and to review.
- Con: non-ABI entry paths constitute a separate category that must
  remember to invoke the same guard helpers; the Phase 5
  retrospective bug landed in exactly that category. The principle
  reduces but does not eliminate the implementer's responsibility
  on those paths — it only makes the responsibility explicit and
  named.

**Option B — Internal-state-machine guards as the single source of
enforcement.** The runtime's own mutation, read, and structure-
changing primitives check state and refuse work. The ABI layer is a
thin pass-through that forwards arguments and translates internal
refusals to `WasamoStatus` codes. Non-ABI entry paths inherit
guards automatically because every path to state goes through the
same primitives.

- Pro: every path to runtime state is guarded regardless of how it
  was reached; a new entry path cannot bypass the guard because
  there is no guarded layer to bypass — the primitives themselves
  refuse.
- Con: error-reporting context (which `wasamo_*` function was
  called, which argument was bad, which name failed to resolve)
  must be threaded into the primitives or attached out of band;
  guard awareness spreads across `reactive`, `emit`, `registry`,
  and `window` rather than concentrating at one layer.

**Option C — Defense-in-depth at both layers.** ABI-boundary guards
provide diagnostic context and short-circuit obvious violations;
internal-state-machine guards catch any path that reached the
primitives without crossing the ABI. The two layers are
intentionally redundant; their per-state coverage is specified
explicitly so that neither layer assumes the other handled a state
it did not.

- Pro: structural protection against both shapes of failure — the
  Phase 5 retrospective bug shape (missed ABI guard) and any
  future internal refactor that moves work between layers;
  diagnostic context preserved at the ABI layer.
- Con: the same check is written twice; per-layer subset of
  states-to-check must be specified to avoid the "each side assumed
  the other handled it" failure mode; the duplication itself
  carries an audit cost.

**Option D — Compile-time-typed guard tokens.** Introduce a
zero-cost type (e.g. `LiveAccess`, `MutationAccess`) that is
constructible only via the guard helper, and require it as a
parameter on every primitive that touches the relevant state.
Code that reaches a primitive without first acquiring the token
fails to compile.

- Pro: omissions become compile errors rather than runtime bugs;
  the Phase 5 retrospective bug shape (call path reaches state
  without guard) is structurally impossible.
- Con: pervasive API change across `reactive`, `emit`, `registry`,
  and `window`; significant ergonomic cost on every internal
  caller; M2-late introduction collides with the M2-to-M3
  transition.

#### Recommendation

**Choose Option C — role-specified defense in depth.** The accepted
guard-placement principle is:

- The **ABI boundary is the diagnostic boundary**. Exported
  `wasamo_*` functions perform the relevant UI-thread, Diverged,
  `IN_DRAIN`, and `IN_OBSERVER_CALLBACK` checks before mutating or
  structurally changing runtime state. These checks own caller-facing
  `WasamoStatus` return values and last-error messages because that
  layer knows the public function name, argument context, and lifecycle
  exception being applied.
- The **internal runtime boundary is the invariant boundary**.
  Internal entry points that may be reached without crossing an
  exported ABI function must refuse or suppress work when the runtime
  state would make that work invalid. In M2 this is concretely
  represented by `emit::drain_if_outermost()` suppressing re-entrant
  drains while `IN_DRAIN` is set and suppressing all drain phases after
  `RuntimeHealth::Diverged`.
- **Runtime-owned non-ABI entry paths are first-class runtime entries**,
  not exceptions to the rule. The Win32 message-loop path in
  `lib.rs::run()` and future M3 timer / async-I/O / additional
  window-procedure paths must enter runtime state through an internal
  invariant boundary rather than relying on ABI-only guards they do not
  cross.
- **Cleanup / destroy paths remain explicit exceptions.** Any operation
  that is allowed after `Diverged` must be named at its entry boundary
  and documented as a lifecycle exception; the exception does not imply
  general permission to touch runtime state after divergence.

Option A is rejected because ABI-only enforcement still leaves the
Phase 5 omission shape as a per-entry-path obligation: every non-ABI
entry must remember to call an ABI-shaped guard helper even though it
does not cross the ABI. Option B is rejected because moving all guards
into internal primitives would either lose ABI diagnostic precision or
force public-call context through otherwise local runtime APIs. Option
D is not required for M2 acceptance: typed guard tokens are the
strongest structural answer, but their blast radius is disproportionate
to the Phase 7 acceptance need now that Option C gives both a
diagnostic boundary and an invariant boundary.

DD-012 acceptance therefore updates `docs/architecture.md` with this
principle as a global runtime invariant. Implementation alignment is
scoped to ensuring existing M2 paths match the accepted rule and adding
focused guard-placement tests; broader tokenisation or callback-surface
redesign is not part of this DD.

#### Forward-compat exposure

Low-medium after acceptance. The placement rule is now explicit before
M3 adds timer, async-I/O, and additional Win32 message handling, so
those new surfaces inherit the Option C responsibility split instead
of re-deciding guard placement locally.

Residual exposure remains in two places. First, each new non-ABI entry
path must name the internal invariant boundary it crosses; omission is
now review-visible but not compile-time impossible. Second, typed guard
tokens remain a M3+ revisit trigger if the number of internal entry
paths grows enough that runtime checks and review discipline no longer
provide sufficient structural confidence.

---

## Summary of accepted decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P6-010 *(Accepted 2026-05-09)* | `dirty_effects` topological sort fidelity | **Option A** — true topological walk in M2, extracted as a free function with pure-logic unit tests. A5 (literal reading) discharged by implementation; M3 inherits the verified primitive. Options B / C / C-lite recorded as considered, not recommended. | Low–medium (Option A) | Discharged at acceptance; M3 residuals (cycle / ties / fan-out) recorded in m2-to-m3-handover.md |
| DD-M2-P6-011 *(Accepted 2026-05-10)* | String-typed property binding | **Option B** — `StrPropRead` HandlerExpr variant. A6 is discharged demonstratively: M2 proves `.ui` String binding through runtime widget property state while preserving existing integer `PropRead` behavior. Option C `TypedValue` unification is deferred to a post-M2 open question. | Low (Option B) | Low for M2; later typed-expression pressure tracked in typed-value-evaluator.md |
| DD-M2-P6-012 *(Accepted 2026-05-10)* | Re-entrancy and safety-guard placement principle | **Option C** — role-specified defense in depth. ABI boundary owns caller-facing diagnostics; internal runtime boundary owns invariant enforcement for ABI-bypassing entries; cleanup exceptions are explicit. Option D typed tokens deferred as a M3+ revisit trigger. | Low-medium (focused implementation alignment and tests) | Low-medium; M3 timer / async-I/O / windowproc surfaces inherit the rule |

**Aggregate shipped picture.** The three DDs stayed narrowly scoped:
010 replaced the production dirty-Effect ordering path with a true graph
walk; 011 added an additive `HandlerExpr` variant plus String read methods
while preserving the existing integer path; 012 settled the guard-placement
principle and aligned the visible M2 gaps with focused tests. With these
implemented, M2's A5/A6 acceptance criteria are discharged. The broader
typed-value rewrite remains outside M2 unless later DSL/tooling evidence
reopens it.

**Aggregate forward-compat exposure.** All three DDs have explicit
successor work or revisit triggers — 010's M3 residuals after the
topological primitive is discharged, 011's post-M2 typed-value open
question, and 012's application across timer / async-I/O / windowproc
surfaces in M3.

## Out of scope

- **General topological-graph diagnostics tooling.** DD-010's true
  graph walk is in scope (under Option A) or scheduled (under Option
  B); a tool that visualises the dependency graph or its SCCs is
  post-M2.
- **`f32` / `bool` / aggregate-typed property binding.** DD-011 covers
  `i32` and `String` only. Additional scalar types are post-M2 work.
  They are one possible trigger for revisiting Option C, but they are
  not forced into M3 unless M3's DSL surface or public spec draft
  creates real type-system pressure.
- **`wasamo_post_event` and timer / async-I/O ABI.** DD-012's option
  set considers M3 timer and async-I/O entry paths as constraints on
  the principle; designing those entry paths themselves is M3.
- **Compile-time guard-token enforcement across the entire runtime.**
  DD-012 Option D is in scope as an option to evaluate; if rejected,
  it is post-M2. If accepted, its rollout sequencing across
  `reactive`, `emit`, `registry`, `window` is task-level detail
  recorded in `m2-plan.md`'s Progress section, not in this ADR.
- **Multi-Effect-per-Signal `.ui` constructs.** Out of scope for M2
  regardless of which DD-010 option is chosen; their introduction is
  M3's multi-binding work item.

## Provenance

This ADR was opened on 2026-05-08 to house the three DDs that the
Phase 6 ADR
([m2-phase-6-ui-lowering.md](./m2-phase-6-ui-lowering.md)) carried in
its draft slate but did not Accept. The Phase 6 ADR retains stub
entries at the DD section anchors that forward to this file; the
Phase 6 ADR itself remains `Accepted` for DD-M2-P6-001..009.

The acceptance-criteria revision that introduced A5/A6 and scoped them
to Phase 7 is recorded in the Progress section of
[m2-plan.md](../plans/m2-plan.md) under the 2026-05-08 entry; the
Phase 7 entry there names this ADR as the housing for DD-010/011/012.

### Minor implementation clarifications

- **2026-05-09 — DD-M2-P6-010 write-edge borrow.** During DD-010
  implementation, `ReactiveGraph::forward` / `back` were confirmed to
  encode read dependencies only. They are sufficient for invalidation
  but not for deriving the Effect-to-Effect ordering edge "writer runs
  before reader when both are dirty." The required implementation form
  was therefore clarified to name the write-edge map as an explicit
  graph borrow. This does not supersede DD-010 or change Option A; it
  records the concrete graph input needed to implement the accepted
  topological walk faithfully.
