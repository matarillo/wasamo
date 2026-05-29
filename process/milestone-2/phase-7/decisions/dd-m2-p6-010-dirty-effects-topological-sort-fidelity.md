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
