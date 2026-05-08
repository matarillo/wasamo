# M2-Phase 7 — Reactive Foundation Hardening & Contract Finalization: Architecture Decisions

**Phase:** M2-Phase 7 (Reactive Foundation Hardening & Contract Finalization)
**Date:** 2026-05-08 (ADR opened; DDs remain Proposed pending per-DD pre-doc cycles)
**Status:** Proposed

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
  010 acceptance: the constraint is either discharged (if 010 = Option
  A is adopted now) or restated against M3 (if 010 = Option B is
  adopted with a tighter trigger).

---

### DD-M2-P6-010 — `dirty_effects` topological sort fidelity

**Status: Proposed**

#### Context

DD-M2-P6-001 = Option D specifies Phase 1 ordering as
"topological-by-dependency-graph" for `dirty_effects`. The M2
implementation in `drain_dirty_effects()` uses `sort_unstable()` on
`EffectId` values, which are monotonically increasing integers assigned
at Effect creation time. This approximates topological order only
because, in the M2 counter shape, a binding Effect is always created
after any Effect it depends on, so its ID is always larger.

The approximation holds for the M2 acceptance set (single binding, one
handler, one reactive value) because there is only one Effect per
Signal. It breaks silently when two Effects depend on the same Signal
in a non-trivial order, or when an Effect created earlier in time
happens to be a downstream consumer of one created later.

This gap was discovered during the DD-M2-P6-001 implementation
retrospective (2026-05-07).

#### Options

**Option A — Fix now.** Replace `sort_unstable()` with a true
topological walk of `ReactiveGraph::forward` / `back` before M2 ships.

- Pro: spec-faithful immediately.
- Con: adds graph-traversal code with no M2 test stimulus. The counter
  shape never exercises non-trivial ordering; correctness of the walk
  would be asserted by tests alone, with no GUI confirmation.

**Option B — Defer to M3 pre-doc with an explicit constraint record.**
Accept the EffectId-numeric approximation for M2. Record the
constraint here. Make "replace with true topological walk" a mandatory
pre-condition for the M3 multi-binding implementation step.

- Pro: no code risk in M2; the approximation is not observable by any
  M2 acceptance criterion.
- Con: the spec-vs-impl gap exists for the M2 lifetime. A reader of
  `drain_dirty_effects()` without this note would not know the sort is
  an approximation.

#### Recommendation

**Option B.** The M2 acceptance criteria (counter with one binding and
one handler) do not exercise multi-Effect ordering; the approximation
is not observable. Shipping a topological walk implementation without
a stimulus that would catch a bug in that walk creates more risk than
it removes.

**Mandatory constraint for M3:** Before any M3 work that introduces
more than one Effect per reactive Signal (multi-binding, computed
values, cross-widget bindings), replace `sort_unstable()` in
`drain_dirty_effects()` with a true topological sort over
`ReactiveGraph::forward` / `back`. The M3 multi-binding test cases
will provide the exercise stimulus that makes the walk verifiable.

#### Forward-compat exposure

Medium. The graph-walk implementation is not complex, but it is
load-bearing for M3 correctness. The constraint must be discharged
in the M3 pre-doc phase before coding begins.

> Note: the recommendation text is inherited verbatim from the Phase 6
> draft. Phase 7 pre-doc may revise it in light of (a) the now-stable
> A5 framing, which raises the bar from "M2-acceptance-observable" to
> "Foundation-grade"; (b) the implementation evidence accumulated
> across DD-M2-P6-001..009. The DD-010 working branch is where that
> revision lands.

---

### DD-M2-P6-011 — String-typed property binding

**Status: Proposed**

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

**Option B — Introduce a `StrPropRead` variant (recommended).**
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

**Option B.** Adding `StrPropRead` is the smallest change that closes all
three gaps without touching existing `PropRead` paths or `EvalContext`
method signatures beyond the two new `get_string` / `read_string_tracked`
additions. Option A is equally low-risk but forces every `PropRead`
construction site to supply a type tag today. Option C is premature
generalisation — M2 has only two types, and the current two-method
`EvalContext` shape already encodes that.

#### Forward-compat exposure

Low. `StrPropRead` is additive. If M3 introduces additional scalar types
(e.g. `f32`, `bool`), each adds a parallel `<T>PropRead` variant and
`EvalContext::get_<T>` pair, or the `TypedValue` unification (Option C)
is revisited at that point with concrete M3 stimulus.

> Note: the recommendation text is inherited verbatim from the Phase 6
> draft. Phase 7 pre-doc revisits it under A6 framing
> ("Type-Agnostic Reactive Binding") — A6 may favour Option C
> (`TypedValue` unification) as the more "type-agnostic" surface, even
> at the cost of broader churn. The DD-011 working branch is where
> that re-evaluation lands.

---

### DD-M2-P6-012 — Re-entrancy and safety-guard placement principle

**Status: Proposed**

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

**To be settled by the DD-012 pre-doc cycle.** The Phase 6 draft
recorded this DD as `Deferred` with no recommendation, on the explicit
ground that the decision should not be made mid-Phase-6 implementation.
With Phase 6 closed, the DD-012 working branch evaluates A / B / C / D
against full Phase 6 implementation evidence — including which runtime
states each option enforces, which non-ABI entry paths each option
leaves exposed, how each option interacts with the M3 timer and
async-I/O extensions, and how the Win32 / WinRT FFI surface constrains
the choice — and the agreed option is recorded here.

Until that cycle settles the rule, **no new ABI function and no new
non-ABI entry path introduced before this DD's Acceptance should be
treated as setting precedent**: any guard placement chosen for a new
entry point is provisional and may be reorganised once the principle
is decided.

#### Forward-compat exposure

High if left unspecified through M2. The cost of choosing incorrectly
grows with each new re-entrancy state and each new non-ABI entry
path; M3 is expected to add at least timer, async-I/O, and
additional Win32 message handling, multiplying combinations through
the four existing states. Settling the principle in Phase 7
keeps the M3 pre-doc free to apply it uniformly across its
new surface.

Low specifically for the Phase 5 retrospective bug. That bug is a
single missed call site and is repaired by adding the missing guard,
independently of which option is eventually chosen.

---

## Summary of proposed decisions

| ID | Topic | Recommendation | Impl risk | Forward-compat exposure |
|---|---|---|---|---|
| DD-M2-P6-010 *(Proposed)* | `dirty_effects` topological sort fidelity | Inherited Phase 6 draft: **Option B** — accept EffectId-numeric approximation for M2; replace with true graph walk before M3 multi-binding. Subject to Phase 7 pre-doc revision under A5 framing. | Low (Option B) / Low–medium (Option A) | Medium |
| DD-M2-P6-011 *(Proposed)* | String-typed property binding | Inherited Phase 6 draft: **Option B** — `StrPropRead` HandlerExpr variant. Subject to Phase 7 pre-doc revision under A6 framing (which may favour Option C). | Low (Option B) / Low–medium (Option A or C) | Low |
| DD-M2-P6-012 *(Proposed)* | Re-entrancy and safety-guard placement principle | **To be settled by DD-012 pre-doc cycle.** A (ABI-boundary) / B (internal-state-machine) / C (defense-in-depth) / D (typed guard tokens). | n/a (open) | High if unspecified through M2 |

**Aggregate impl-risk picture.** The three DDs are scoped narrowly:
010 changes `drain_dirty_effects()` only; 011 adds an additive
`HandlerExpr` variant and two `EvalContext` methods (under the inherited
recommendation); 012 settles a *principle* whose enforcement may or may
not require code change beyond the local Phase 5 retrospective fix
(already landed in Phase 6). The Phase 7 closing risk is therefore
concentrated in 012 — both because its option set ranges from
documentation-only to pervasive type-system change, and because A5
acceptance hinges on its quality, not its presence.

**Aggregate forward-compat exposure.** All three DDs have explicit
M3-or-later successor work — 010's mandatory pre-condition (or its
discharge), 011's M3 type-system extension trigger, and 012's
application across timer / async-I/O / windowproc surfaces in M3.

## Out of scope

- **General topological-graph diagnostics tooling.** DD-010's true
  graph walk is in scope (under Option A) or scheduled (under Option
  B); a tool that visualises the dependency graph or its SCCs is
  post-M2.
- **`f32` / `bool` / aggregate-typed property binding.** DD-011 covers
  `i32` and `String` only. Additional scalar types are M3, paired with
  the DSL spec finalisation; the DD's Forward-compat exposure
  paragraph names them as the trigger for revisiting Option C.
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
