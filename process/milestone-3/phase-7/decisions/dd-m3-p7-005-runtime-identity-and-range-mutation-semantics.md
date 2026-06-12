# DD-M3-P7-005 — Runtime identity baseline and range mutation semantics

**Status:** Proposed
**Phase:** M3-Phase 7
**AC:** A8 (runtime cardinality tracking); A12 (the identity baseline
is normative author-visible spec text)

## Context

Phase 6's conditional shipped **absent = fresh-on-return** as normative
semantics for the 0/1 case, and the handoff records that nothing beyond
fresh-on-return is solved (constraints §3). Iteration must now pick the
0..N identity baseline. The aligned boundary (FD-C): **un-keyed**,
append/truncate-only, with the *normative wording* — fresh vs
positional-stable — explicitly delegated to this DD (owner answers §3
note 1: in the Phase 7 scope the two are **observationally
equivalent**, because items are stateless scalars and only tail
mutations exist; but the dsl_spec sentence differs, and the future
keyed opt-in will cite that sentence as the baseline it must not
silently change — so the choice must be made consciously, not by
implementation accident).

Failure observability (constraints §5): Phase 6 left build / insert /
remove failures as post-validation log-only diagnostics — tolerable
for a single-child toggle, but a multi-child range edit can fail
**partway**, leaving partial state; this DD must decide the contract.

Drain contract (constraints §6 item 4): with `BATCH_DEPTH == 0`, a
write drains before returning — toggle-then-observe. Phase 6 preserved
it under structural mutation; Phase 7 must preserve it under range
mutation.

## Decision dependency summary

Consumes DD-M3-P7-002 (whole-value signal), DD-M3-P7-004 (the C1
expansion seam + `ForLoopSubtree`); the splice primitive executing the
plans decided here is DD-M3-P7-006's; failure diagnostics surface via
DD-M3-P7-007.

## Sub-issues

- **Normative identity wording** — fresh vs positional.
- **Mutation execution plan** — what a tail append / removal does mechanically.
- **Per-item binding reactivity** — frozen capture vs live positional
  read.
- **Partial-failure contract** — log-only vs stage-then-commit vs
  rollback / terminal.
- **Disposal & drain ordering** — teardown order; setter-return
  observability.

## Normative identity wording

### Options

- **W1 — fresh-on-change:** *"when a collection changes, its generated
  subtrees are rebuilt fresh."*
  - What you gain: one sentence, verbatim continuous with the Phase 6
    conditional wording; maximally permissive for implementations.
  - What you give up: it **licenses full rebuild on every append** — an
    author adding item 50 must accept items 0..49 being torn down (and,
    once items have any runtime-visible accumulation — animation
    state, scroll anchoring, M4 input focus — visibly disturbed). The
    future keyed opt-in would then be positioned against a baseline
    *worse* than what any reasonable implementation does, and
    tightening the sentence later (fresh → positional) is itself a
    semantic change keyed work would have to absorb.

- **W2 — positional, un-keyed:** *"a generated subtree's identity is
  its position in the collection; a tail append materialises only the
  new tail subtrees, a tail removal disposes only the removed tail
  subtrees; subtrees at retained positions are retained (their bound
  properties re-evaluate; they are not rebuilt)."*
  - What you gain: the contract authors actually need stated (append
    does not disturb the prefix — the property the gallery proof's
    before/after frames silently rely on); it is the honest description
    of the natural diff implementation; it is the standard un-keyed
    baseline of comparable systems, so the future keyed opt-in slots in
    as "identity follows the *element* instead of the *position*" — a
    clean delta; under a future whole-value replace it still has
    meaning (re-bind by position), so the host-boundary wave inherits a
    usable sentence.
  - What you give up: it constrains implementations (no
    full-rebuild-on-append shortcut) and promises prefix stability
    that must be *tested*, not assumed (a positive control in the
    verification closure: prefix subtree pointers unchanged across a
    tail append).

### Recommendation

**W2 — positional, un-keyed — stated normatively in dsl_spec.** The
explicit non-promise is stated beside it: *positions confer no
element-tracking identity; no state is preserved across removal; keyed
retention is a future opt-in surface* (FD-E: this also corrects the
Phase 6 forward-compat expectation — keyed identity is **not** what
Phase 7 ships — via live-doc sync, not retroactive ADR edits).

## Mutation execution plan

The `for` effect depends on the collection signal (whole-value, R1).
On change: read new length N, compare materialised count M (from the
C1 seam / live subtree list):

- **N > M (append):** instantiate body templates for positions M..N−1,
  splice them at materialised offsets `[offset(slot)+M, …)` in order.
- **N < M (pop / truncate):** dispose subtrees at positions N..M−1,
  tail-first.
- **N == M:** no *structural* edit (the idempotency mirror of Phase
  6's same-state-toggle test). Under DD-002's M3b a same-length
  static-literal reset can change values at retained positions; those
  updates ride the V2 positional reads (item-binding effects
  re-evaluate; equal values write idempotently) — the structural no-op
  branch is tested, not assumed.

Because element identity is positional and authored mutations are the
self-receiver tail-edit / static-literal assignments (DD-002), a
length diff fully determines the *structural* edit this phase — under
positional identity a whole-value replace's structural delta is
exactly its length delta, with retained-position value changes riding
the V2 reads. A future reorder (keyed diff) breaks that inference —
recorded as the explicit limit of this plan (the trigger lands with
the collection-UX wave; the W2 sentence, not this inference, is the
durable contract).

## Per-item binding reactivity

How does `label: thumb` get its value — captured at
materialisation, or read live?

- **V1 — frozen capture:** the element value is copied into the
  instantiation context at build time; the binding is a constant
  thereafter. Cheaper, but it bakes "elements never change in place"
  into *effect wiring* rather than into the mutation surface — a
  future replace would leave stale prefixes (a contract violation of
  W2's "bound properties re-evaluate") and force rewiring.
- **V2 — live positional read:** `ItemRead` lowers to an effect read
  of `collection[i]` on the whole-value signal (the position `i` fixed
  per instantiation; the *value* read live). Under tail-edit
  mutation the observable behaviour is identical to V1 (prefix values can't
  change); under a whole-value replace — present this phase as the
  DD-002 M3b static-literal reset — retained positions re-evaluate
  automatically — W2 holds with no rewiring. Cost: every
  per-item binding subscribes to the collection signal, so a tail
  append re-runs prefix item bindings (they recompute, produce equal
  values, write idempotently); breadth is bounded by N and consumes
  no cap depth (DD-007 accounting).

  `ItemRead` evaluation is guarded: if its fixed position is outside
  the collection's current length, the binding writes nothing. This is
  the defined same-batch removal case: a tail removal can dirty the
  removed tail item's binding before the `for` effect disposes that
  subtree, so
  the doomed effect must be a well-defined no-op rather than an
  out-of-range read. The guard is not an author-visible stale-value
  contract; at quiescence DD-006/this DD have disposed the removed
  subtree.

**Recommendation: V2.** The recompute breadth at gallery N is trivial,
and V2 is what makes W2 a *kept* promise rather than a coincidence of
the current mutation set. (The index binder reads its instantiation
position; positions of retained subtrees don't change under tail-only
mutation. Under future non-tail edits, position reassignment is part
of the deferred reorder design — recorded, not solved.)

## Partial-failure contract

A range insert builds N' new subtrees (widget construction, Visual
creation — fallible WinRT calls) and splices them. Phase 6's log-only
posture is re-examined for the range case:

### Options

- **PF1 — log-only throughout (status quo):** a mid-range failure
  leaves j-of-N' children inserted, with a log line. The tree no
  longer matches any collection state — silent partial UI.
- **PF2 — stage-then-commit:** all fallible work happens **before**
  the first tree mutation: build every new subtree fully (widgets,
  Visuals, not yet parented). Any failure ⇒ dispose staged work, log
  a range-scoped diagnostic, **abort the whole mutation with the tree
  observably unchanged**. The commit (vector splice, placement,
  Visual parenting, registry, effect attach — DD-006's primitive) then
  performs only operations that are infallible on the Rust side;
  WinRT parenting calls inside commit are not made fallibility-free by
  decree, but a failure there is an OS-level inconsistency logged with
  range context (which declared slot, which positions) rather than a
  designed state.
- **PF3 — rollback / transactional commit:** undo partial commits.
  Real rollback of WinRT Visual operations needs inverse-operation
  bookkeeping for failures that have never been observed in practice —
  machinery without a driver, and falsely comforting (the inverse ops
  can themselves fail).
- **PF4 — terminal error (divergence-style):** any structural failure
  poisons the runtime. Disproportionate: a failed *append* killing a
  healthy UI inverts severity, and it would make the (unobserved)
  failure class catastrophic instead of contained.

### Recommendation

**PF2.** It removes the *designed* partial-state window (the staging
phase is where construction failures actually live) at the cost of
building before splicing — which the natural implementation does
anyway — and upgrades observability with range-scoped diagnostics
(DD-007 surfaces them). The observable contract: **a collection
mutation either takes effect entirely or leaves the materialised tree
unchanged before the first insertion commit, in both cases with the
drain contract holding on return.** Removal is already a commit-stage
operation: if an OS-level failure is ever observed while disposing
effects, releasing registry entries, or removing Visuals, it is logged
with range context like a commit-stage WinRT parenting failure, not
promised as undoable. PF3 / PF4 are declined on proportionality grounds
with this recorded
re-trigger: if commit-stage WinRT failures are ever observed in
CI / the field, the contract is re-opened (constraints §5's "may need
a stronger story" honoured as a trigger, not pre-built).

## Disposal & drain ordering

- **Removal (pop / truncate):** per removed subtree, tail-first:
  effects disposed ahead of structural teardown (the §6.7.6
  dispose-ahead invariant, unchanged), widget-registry entries
  released via the existing destroy path, Visual removed — all inside
  DD-006's single splice primitive so order metadata cannot drift.
- **Insertion:** staged subtrees are spliced in declared order;
  their per-item binding effects (V2) are created at commit and run
  before the mutating call returns — the **M3-Phase 1 item-4
  synchronous drain contract is preserved, not revised**: after a
  tail-append assignment, the caller observes the new child with its
  bound properties written; after a tail-remove, the child is gone.
  This is the
  Windows-runtime integration assertion (verification closure item 4).
- **Quiescent order invariant (generalised):** at quiescence,
  materialised children = declared members expanded by live
  cardinality in document order — drain-order-independent, computed
  through the C1 seam. The Phase 6 sibling-conditional fixtures extend
  to `for`-flanked-by-`if` cases.

## Spec content seed

dsl_spec iteration chapter, normative: the W2 identity sentence + the
explicit keyed non-promise; mutation timing (on handler return the
tree reflects the mutation — toggle-then-observe generalised);
empty-collection behaviour (zero children, slot live); the
all-or-unchanged insertion-construction contract plus the logged
commit-stage removal/WinRT-failure posture. architecture.md: the range
mutation path (stage-then-commit, dispose-ahead, seam-computed
offsets) in the reactive / structural-mutation sections — accepted
contracts only, no option labels.

## Forward-compat exposure

- **Keyed identity / retained state** — the W2 positional sentence is
  the cited baseline; keyed arrives as opt-in element-tracking over
  the same declared-slot anchor (M4-input trigger; framing 正本).
- **Whole-value replace (host boundary)** — V2 + W2 already define its
  semantics at retained positions; only the *edit inference* (length
  diff ⇒ tail edit) must generalise to a diff against the new value.
- **Reorder** — excluded by construction (tail-only); its arrival
  reopens position reassignment + keyed diff together (M5 / collection
  UX).
- **Member-range bodies** — the plan's "one subtree per position"
  becomes "one range per position"; PF2 staging and the seam absorb it.

## Revision history

- Strategic owner-alignment review fold: scoped all-or-unchanged to
  staged insertion and documented removal/commit-stage failure posture;
  status remains Proposed.
- Recommendation-choice review fold: aligned the binder-read example
  with `.ui` property-binding surface notation; status remains Proposed.
- Implementation-readiness review fold: specified the V2 out-of-range
  positional-read guard for doomed tail bindings; status remains
  Proposed.
- Owner-direction fold (2026-06-12): authored-mutation wording synced
  to DD-002's assignment surface (self-receiver tail-edit assignments
  as the only writers); no semantic change — the execution plan, V2,
  PF2, and the drain contract are unchanged; status remains Proposed.
- Owner-review fold (2026-06-12, second pass): DD-002 M3b makes the
  static whole-value replace author-reachable — the length-diff
  inference is restated as structural-only (value changes at retained
  positions ride V2; reorder, not replace, is what breaks the
  inference); status remains Proposed.

## Technical risk re-evaluation

- **V2's prefix recompute** is the subtle behaviour: idempotent
  rewrites of unchanged values must not trigger spurious layout
  invalidation storms — the existing property-write path's
  equal-value short-circuit behaviour gets checked, and if absent, a
  bounded note (not a redesign) lands in the implementation plan.
- **PF2's staged-disposal path** (failure between staging and commit)
  is itself a branch that needs a directly-fired test (trap #4) —
  likely via a pure-logic staging planner test plus a fault-injected
  construction in the headless suite if feasible; if fault injection
  is not feasible mock-free, the disposition is recorded in the
  implementation log, not silently skipped.
- **Tail-first removal order** and registry/effect release get the
  Phase 6 conditional teardown tests generalised to ranges.
- **The no-op (N == M) branch** is tested explicitly (idempotency
  mirror).
- **The pop-doomed binding branch** is directly fired: after a tail
  `pop`, a removed item's binding may be dirty in the same drain batch;
  the test proves the guarded `ItemRead` skips instead of panicking.
