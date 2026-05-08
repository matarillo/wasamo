# M2-Phase 7 / DD-M2-P6-010 pre-doc framing

**Status:** framing aligned with owner (2026-05-08); input artefact for
ADR drafting
**Date:** 2026-05-08
**Targets DD:** DD-M2-P6-010 — `dirty_effects` topological sort fidelity
**Targets phase:** M2-Phase 7 (Reactive Foundation Hardening)
**ADR housing:** [docs/decisions/m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md)

Per the project's doc-driven workflow, framing is aligned in chat first
and recorded here as the input artefact for ADR drafting. Phase 7's
three DDs (010 / 012 / 011) are processed as **independent** pre-doc
cycles; this note covers DD-010 only. The Phase 6 framing precedent is
[m2-phase-6-pre-doc-framing.md](./m2-phase-6-pre-doc-framing.md).

---

## DD-010 question (restated)

`drain_dirty_effects()` currently sorts the dirty Effect set by
numeric `EffectId` (`v.sort_unstable()` in
[wasamo-runtime/src/reactive.rs:126](../../wasamo-runtime/src/reactive.rs#L126)).
DD-M2-P6-001 = Option D specifies that ordering as
"topological-by-dependency-graph". The numeric-ID sort approximates
topological order only because, in the M2 counter shape, every
binding Effect is created strictly after the Effects whose Signals
it reads — so its ID is always larger.

The approximation breaks silently as soon as two Effects depend on
the same Signal in a non-trivial order, or whenever an Effect created
earlier in time is a downstream consumer of one created later. M2's
acceptance set never exercises that shape; M3 multi-binding will.

The Phase 6 ADR draft recommended **Option B** (accept the
approximation for M2; mandate replacement before M3 multi-binding).
Phase 7 reopens the recommendation under **A5 framing** —
"Foundation-grade" rather than "M2-acceptance-observable".

---

## A5 framing — what changed since the Phase 6 draft

The 2026-05-08 acceptance-criteria revision added A5 to
[m2-plan.md](../plans/m2-plan.md). Its operative clause for DD-010:

> the implementation no longer relies on the counter case happening
> to converge.

Read literally, A5 is a property of the **shipped implementation**,
not just of the design record. Option B as drafted in Phase 6 ships
M2 with the EffectId-numeric approximation in place; the counter case
converges precisely because the approximation happens to coincide
with the topological order on that shape. Therefore Option B as
drafted **does not** discharge A5 by its own wording.

**Settled in framing (2026-05-08): A5 stays as written.** DD-010 must
adopt an option that **structurally guarantees** topological
correctness in the shipped binary, not merely "would be correct if
the graph were walked". Option B (defer + tone down A5) is therefore
off the table as a recommendation; it is retained in the Options
table as a documented record of the alternative considered, with the
A5-non-discharge consequence stated in its Con. Variants whose
production-side correctness rests on `debug_assert!`-only checks
(Option C, C-lite) face the same scrutiny: a release-build assertion
that is compiled out is not "the implementation no longer relies on
the counter case happening to converge" by A5's wording.

---

## Implementation evidence accumulated across DD-001..009

Carrying these forward into the option re-evaluation:

1. **`ReactiveGraph::forward` / `back` are already maintained.**
   `reactive.rs` builds both directions during dependency tracking
   ([reactive.rs:55-56](../../wasamo-runtime/src/reactive.rs#L55-L56)).
   A topological walk has its inputs structurally available; the
   added cost is the walk itself, not graph instrumentation.

2. **`EffectId` monotonicity is a real property, not folklore.**
   IDs are assigned at Effect creation time from a counter; tracking
   stack pushes only occur during evaluation. So "ID order"
   coincides with "creation order". The narrow correctness condition
   is: "for every edge `u → v` in the dependency graph,
   `id(u) < id(v)`" — which holds iff every Effect is created after
   the Effects whose Signals it reads. The counter case satisfies
   this *by construction* of the IR loader's emit order; M3 cannot
   guarantee it without a structural rule the loader does not yet
   enforce.

3. **`DIRTY_EFFECTS` is a `HashSet`, drained per iteration.** Drain
   loop caps at `MUTATION_CAP = 16` iterations
   ([reactive.rs:121-128](../../wasamo-runtime/src/reactive.rs#L121-L128)).
   Per-iteration cost of a true topological walk is bounded by the
   dirty set size and the local out-degree in `forward`; M2 sizes
   are tiny.

4. **No M2 test stimulus stresses ordering.** The Phase 6 draft's
   Con on Option A — "correctness asserted by tests alone, with no
   GUI confirmation" — still holds. Pure-logic unit tests on a
   topo-sort routine are within the project's
   [testing rules](../../CLAUDE.md#testing-rules) (no Win32/WinRT
   FFI dependency); GUI confirmation requires multi-binding stimulus
   that does not exist before M3.

---

## Reframed option set

The Phase 6 draft enumerated A and B. Phase 7 evidence motivates a
middle option. Final framing carries four entries; A5 (kept as
written, see above) restricts the recommendation space to options
that ship a structural guarantee. Drafter recommendation: **Option A**.

### Option A — True topological walk in M2 *(drafter recommendation)*

Replace `sort_unstable()` with a Kahn-style topological walk over
`ReactiveGraph::forward` / `back` restricted to the dirty set.
Cover with pure-logic unit tests on synthetic dependency graphs
(extract the walk into a free function that takes `&forward`,
`&back`, and the dirty set; this falls cleanly within the testing
rules without needing the mirror-struct pattern).

- Pro: spec-faithful in M2; A5 discharged by implementation;
  M3 multi-binding inherits a verified primitive.
- Pro: pure-logic unit tests provide stimulus that the M2 GUI cannot.
- Con: code that ships before any production caller exercises it on
  a non-trivial graph; M3 may discover constraints (cycle handling,
  ordering ties, fan-out interaction with `MUTATION_CAP`) that force
  a redesign.

### Option B — Defer to M3, tone down A5 *(considered, not recommended — A5 framing precludes)*

Keep `sort_unstable()` for M2. Record the constraint in the ADR.
Make "replace with true topological walk" a mandatory pre-condition
for M3 multi-binding, as the Phase 6 draft already did. **Revise
A5** in `m2-plan.md` to drop the implementation-level clause:
A5 becomes "DD-010 Accepted with the constraint recorded; M3 is
gated on its discharge" rather than "implementation no longer
relies on the counter case happening to converge".

- Pro: smallest M2 code change; aligns with the Phase 6 draft
  intuition that shipping unexercised algorithms is itself a risk.
- Con: requires an A5 wording revision that visibly weakens
  Phase 7's "Foundation Hardening" framing — the change is
  honest, but it shifts what A5 promises.
- **A5 discharge:** does not discharge A5 as currently worded;
  recorded for comparison only.

### Option C — Verified approximation *(considered, unlikely)*

Keep the `sort_unstable()` fast path. Add, alongside it, a
debug-mode assertion that walks `forward` / `back` and verifies the
ID-sorted order is in fact topological for the current dirty set.
The assertion is `debug_assert!`-gated, so release builds carry no
overhead. Optionally, also add a structural invariant at Effect
creation: the runtime asserts that every newly-tracked Signal's
existing dependents have IDs less than the new Effect's ID, which
makes the "ID order ≡ topological order" precondition checked at
the point it could be violated.

- Pro: unit tests on the assertion logic are the same shape as
  Option A's tests, but exercise a smaller code surface.
- Pro: structural-invariant variant catches violations at Effect
  creation time, the point closest to the source of any future
  breakage.
- Con: **A5 discharge is contestable.** The release binary still
  runs the EffectId-numeric `sort_unstable()` path; the
  `debug_assert!` verifier is compiled out. Under A5's literal
  wording ("the implementation no longer relies on the counter case
  happening to converge"), what ships still relies on the counter
  case happening to converge — only the debug build has stronger
  evidence. The structural-invariant sub-variant is closer to a
  pass, but the running drain still uses the cheap sort.
- Con: two code paths (cheap sort + debug verifier) where Option A
  has one; the verifier is itself a topo walk, so the implementation
  cost overlaps with Option A without delivering Option A's
  release-mode correctness.
- Con: the structural invariant sub-variant changes Effect creation
  semantics in ways that may interact with M3 features not yet
  designed (computed values created lazily, cross-widget bindings
  whose creation order is host-driven).

### Option C-lite — Assertion only, no structural invariant *(considered, unlikely)*

A narrower form of C that adds only the `debug_assert!` walk in
`drain_dirty_effects()` and does not touch Effect creation.
Cheaper to implement; does not constrain M3's design space.

- Pro: minimal code change.
- Con: **A5 discharge weaker than C.** The release binary still
  runs `sort_unstable()`; the debug assertion only proves the
  precondition holds for the cases the test/run exercises, which
  for M2 is exactly the counter case "happening to" converge.
  Under A5's literal wording this fails the same way Option B does,
  with extra debug-mode evidence as a fig leaf.

---

## Owner-agreed framing decisions (2026-05-08)

- **A. A5 stands as written.** The acceptance-criteria clause
  "the implementation no longer relies on the counter case happening
  to converge" is treated literally. DD-010 is recommended toward an
  option that ships a structural correctness guarantee in the
  release binary. A5 is **not** revised down to design-level
  discharge.

- **B. Drafter recommendation: Option A** (true topological walk
  in M2). Rationale: A5 = literal forces a release-mode guarantee;
  Option A is the only entry that delivers one without spawning a
  release/debug correctness asymmetry. The walk extracts to a free
  function within the project's
  [testing rules](../../CLAUDE.md#testing-rules) (no Win32/WinRT
  coupling), so the "ships unexercised" Phase 6 objection is
  mitigated by pure-logic unit tests that cover shapes M3 will
  introduce.

- **C. Options B / C / C-lite remain in the ADR Options table.**
  They are recorded with their A5-discharge analysis as the
  "considered, not recommended" alternatives. Their inclusion
  preserves the audit trail of why the cheaper options were
  rejected; their exclusion from the recommendation is decided.

- **D. Forward-compat exposure paragraph reconciliation deferred to
  ADR drafting.** Not approving any option yet; if Option A is
  Accepted, the paragraph's mandatory-pre-condition for M3 is
  *discharged* (the walk exists; M3 inherits it). The reconciled
  text lands in the same commit that flips DD-010 to `Accepted`.

- **E. Retrospective-process observations are required at ADR
  drafting time, not at framing time.** The framing's four
  Implementation evidence items are sufficient for option
  evaluation; the ADR draft additionally folds in observations
  from the DD-M2-P6-001..009 retrospective process. Procedure:
  the drafter re-reads the retrospective record at ADR-drafting
  entry and proposes which observations to incorporate; owner
  reviews in the ADR review pass. (The owner has flagged this as
  a precondition for ADR agreement, not for framing alignment.)

---

## Next session — handoff

Inputs are complete. The next session begins ADR drafting:

1. **Retrospective sweep.** Re-read the DD-M2-P6-001..009
   retrospective record (decision E above) and prepare a list of
   observations to fold into DD-010's Context / Options Pro/Con
   prose. This is the precondition the owner flagged for ADR
   agreement; surface it at the start of the drafting session, not
   at review time.
2. **ADR DD-010 section revision.** In
   [m2-phase-7-reactive-foundation.md](../decisions/m2-phase-7-reactive-foundation.md):
   - Replace the inherited Phase 6 Recommendation prose with the
     Phase 7 conclusion (Option A, with A5-literal-reading rationale).
   - Carry forward Options B / C / C-lite per decision C, each with
     the A5-discharge analysis worked out above.
   - Reconcile the Forward-compat exposure paragraph per decision D
     (assuming Option A: discharge).
   - Fold in retrospective observations from step 1.
   - Update the "Summary of proposed decisions" table for DD-010.
3. **Owner ADR review pass.** Owner agrees / requests revisions on
   the Option A recommendation, the retrospective-observations
   incorporation, and the Forward-compat reconciliation.
4. **Acceptance commit.** Flip DD-010 to `Accepted`; the ADR edit
   lands as a single commit. No `m2-plan.md` A5 edit (decision A:
   A5 unchanged).
5. **Implementation step.** Implement Option A on the active phase
   step branch per the
   [step branch workflow](./retrospectives.md) — the topological
   walk extracted as a free function with pure-logic unit tests on
   synthetic dependency graphs.
6. **Framing note disposition.** This note remains as input
   artefact; not promoted into the ADR. Archive (or delete, with
   git history retention) once DD-010 is Accepted and
   implementation lands. The note may be referenced from DD-012 /
   DD-011 framing if their pre-doc cycles want to inherit
   structural moves (e.g. the A5-literal-reading discipline).
