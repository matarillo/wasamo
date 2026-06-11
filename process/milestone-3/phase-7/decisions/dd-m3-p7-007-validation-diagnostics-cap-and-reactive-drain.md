# DD-M3-P7-007 — Validation, diagnostics, cap accounting, and reactive-drain residual disposition

**Status:** Proposed
**Phase:** M3-Phase 7
**AC:** A8 / A12 (diagnostics are public-draft content); plan-level
obligation: reactive-drain fix-or-carry with **cap accounting settled
first** (owner answers §3 note 3; silent carry-forward prohibited,
constraints §6)

## Context

Iteration is the first grammar where one state change fans out to N
generated subtrees and their dependent effects. This DD owns three
things: (1) the **reject matrix** — which invalid shapes `wasamoc
check`, the textual-IR loader, and runtime validation refuse, each
with a named diagnostic and a directly-firing test (trap #4); (2) the
**`MUTATION_CAP` accounting model** — what the cap counts and how
N-item materialisation is charged, which the fix-or-carry judgment is
conditional on; (3) the **reactive-drain residual disposition** — M2
handoff §3 items 1–4, re-judged for the range-mutation path.

Ground truth for (2), verified in source
([reactive.rs](../../../../wasamo-runtime/src/reactive.rs)):
`MUTATION_CAP = 16` bounds the **iterations of the drain loop** —
each iteration drains the *entire* current dirty set as one batch;
writes made during an iteration enqueue for the next. The cap
therefore bounds **cascade depth** (chains of effects re-dirtying
signals), not effect count, not structural edit count.

## Decision dependency summary

Consumes every other DD's reject branches (DD-001 sweep & body,
DD-002 type / literal / statement, DD-003 binder / handler, DD-004
loader, DD-005 failure diagnostics) and fixes their diagnostic /
test discipline. The cap judgment consumes DD-005's V2 (per-item
bindings re-run as breadth) and the E2E proof scale (FD-B).

## Sub-issues

- **Reject matrix** — compile-time / load-time / runtime.
- **Empty-collection shape** — the 0-child case and the Phase 6
  ScrollView interaction.
- **Cap accounting model** — what `MUTATION_CAP` counts for iteration.
- **Reactive-drain residual** — fix-or-carry per item.
- **Diagnostic & test discipline** — trap #4 closure.

## Reject matrix

Dual gate throughout (the Phase 6 pattern): `wasamoc check` rejects
authored `.ui` with a named diagnostic; the loader independently
re-rejects equivalent textual-IR shapes as `WASAMO_ERR_IR_MALFORMED`.
Owning DD in parentheses; every row is a test.

**Header / target:**
- `for` over a non-collection (scalar state, undeclared name) —
  type / name-resolution error (DD-001).
- `for` over anything but an `IDENT` (literal, operator expr) —
  "collection expressions not yet supported" recorded deferral
  (DD-001 / Q5).
- qualified collection reference after `in` (`for x in root.xs`) —
  "loop collection must be a local state name" diagnostic, naming the
  DD-001 reference-shape deferral.
- binder = declared state name; element binder = index binder;
  `in` / `for` / family keywords in identifier position (DD-003 /
  DD-001).

**Placement:**
- direct `for` under ScrollView / Box / Grid; component-level `for` —
  each a distinct diagnostic naming the container contract (DD-001
  sweep).

**Body:**
- non-widget member directly in the body (property / bind / handler /
  `state` / track-list); **handler anywhere inside the body
  template** (the HA1 admission reject, deferral named — DD-003);
  multiple children; bare control-flow as immediate body; a `for`
  member at any depth inside a `for` body template (nested-scope
  deferral named — DD-001 / DD-003).

**Binder reads:**
- binder read outside its `for` body; undeclared binder; binder read
  in handler position; binder read in an `if` condition — `cond_expr`
  identifiers resolve to `bool` state only, and per-item conditional
  presence is a recorded deferral (DD-003).

**Collection declarations / literals (DD-002):**
- nested collection types (`i32[][]`); heterogeneous or
  element-type-mismatched list literal; list literal as a scalar
  state's default and vice versa.
- non-literal collection element (`state xs: i32[] = [a, b]`) —
  "collection literal elements must be scalar literals; collection
  expressions are not yet supported" recorded deferral (DD-002 / Q5).
- loop-external collection reads (`xs.length`, `xs[0]`, empty checks
  outside the `for` header / loop-local binder path) — "collection
  reads outside iteration not yet supported" recorded deferral
  (DD-002 / Q5).

**Mutation statements (DD-002):**
- `append` / `pop` on a scalar or undeclared LHS; `append` element
  type mismatch; `append()` arity; `pop(expr)`; qualified collection
  LHS (`root.xs.append(...)`) — "collection mutation requires a local
  state name" diagnostic; whole-collection assignment (`xs = [..]` /
  `xs = ys`) — "collection assignment not yet supported" recorded
  deferral.

**Runtime validation (load + mutation time):**
- loader re-checks of all structural rows above (DD-004);
- DD-005 PF2 staging-failure diagnostics: range-scoped (declared
  slot, positions, failed stage) — **log-surface upgraded from Phase
  6's single-child line to range context**, the §5 observability
  answer this phase commits to.

## Empty-collection shape

A live `for` slot with cardinality 0 is **legal** — zero materialised
children in an admitted container (VStack / HStack / WrapPanel /
ZStack all tolerate zero children today; the sweep test pins each).
The Phase 6 DD-M3-P6-007 interaction resolves cleanly: ScrollView's
exactly-one-content contract is protected by the *placement* reject
(no direct `for` under ScrollView), so the conditionally-empty
question does not reopen — the gallery shape is `ScrollView {
WrapPanel { for … } }`, where an empty collection yields an empty
WrapPanel, which is valid. Recorded explicitly so the DD-M3-P6-007(b)
deferral (the conditional-content model) is *not* silently consumed
by this phase.

## Cap accounting model (settled before the carry judgment)

**Model:** `MUTATION_CAP` counts **drain-loop iterations = cascade
depth**. Charging for one authored collection mutation under the
Phase 7 design:

- the handler's `append` / `pop` is one signal write → drain
  iteration 1 runs the dirty set, including the `for` effect, which
  executes the whole tail edit (stage + commit, all N' subtrees) as
  **one effect run**;
- effects created for the staged subtrees (per-item bindings, V2) and
  any prefix re-runs (V2 breadth) are enqueued and run in iteration 2
  as **one batch regardless of N**;
- a further iteration occurs only if those bindings *write signals* —
  which Phase 7 item bindings never do (they write widget
  properties).

So an authored collection mutation consumes **≈ 2 of 16 iterations,
independent of collection size** — breadth (N) is charged zero depth.
What *does* consume depth is effect→signal→effect chaining, which
iteration does not introduce (item bindings are signal-to-property).
**N-item materialisation is counted as: 1 structural effect run + 1
batched binding iteration — not N anything.**

**Evidence obligation (verification closure):** a Windows-headless
fixture asserts a representative `append` at gallery scale (and a
deliberately larger N, e.g. 64 > `MUTATION_CAP`) converges without
divergence — positively demonstrating that breadth does not approach
the cap, so the proof's passing is by design, not by small-N luck
(the framing R3 risk discharged). Because the authored mutation surface
is only `append(expr)` / `pop()`, the >N fixture reaches the large
cardinality either by issuing many appends in one handler batch or by a
headless direct signal setup before the observed drain; the fixture
must state which path it uses.

## Reactive-drain residual disposition (fix-or-carry, explicit)

M2 handoff §3, re-judged on the range path — accounting settled
above, so the carry is now *permitted* to be judged:

| Item | Disposition | Grounds / re-trigger |
|---|---|---|
| 1. cycle detection policy | **Carry** | iteration adds no effect→signal edge (item bindings write properties, not signals; mutation statements run in handlers, not effects) — no new cycle shape exists to detect. Re-trigger: any surface letting a generated subtree's effect write state. |
| 2. ordering ties | **Carry** | inter-effect tie order stays implementation-defined; observability is protected by the **quiescent order invariant** (DD-005: declared order × live cardinality, drain-order-independent), generalised from Phase 6 and tested with `for`/`if` interleavings. Re-trigger: an observable contract requiring inter-effect order. |
| 3. fan-out × `MUTATION_CAP` | **Carry, with the accounting fixed above as the recorded ground** | fan-out is breadth; the cap charges depth; the ≫N fixture is the standing evidence. Re-trigger: any charging change to the drain loop; effect-to-signal writes; acceptance demanding N where *per-iteration batch cost* (time, not cap) matters — that is the M5+ LazyList/performance thesis. |
| 4. synchronous non-batched drain contract | **Preserved (not carried — held)** | DD-005 keeps toggle-then-observe under range mutation; the handler-return assertion is verification-closure item 4. |

No item is silently carried: each row above is the explicit record
constraints §6 demands, and the carry rows land verbatim in the
implementation handoff.

## Diagnostic & test discipline (trap #4 closure)

- Every reject row ⇒ at least one test that **directly fires that
  branch** (pure-logic for `wasamoc check` rows; loader tests for
  `WASAMO_ERR_IR_MALFORMED` rows; headless runtime tests for
  mutation-time diagnostics). No reject ships untested.
- Diagnostics for *deferrals* (collection expressions, collection
  assignment, per-item handlers, nested `for`) name the deferred
  surface — recorded deferrals, the Phase 6 operator-condition
  pattern.
- The A12 chapter's invalid-examples section is generated against
  this matrix (one example per author-reachable row), so spec and
  diagnostics cannot drift (the items-1/3 ↔ item-7 cross-check in
  the verification closure).

## Spec content seed

dsl_spec iteration chapter: diagnostics table (author-reachable rows
+ messages' normative content), empty-collection semantics, the
mutation-timing/all-or-unchanged contract sentences (with DD-005),
invalid examples. architecture.md: one paragraph stating the cap's
charging model for structural mutation (depth, not breadth) as the
accepted contract.

## Forward-compat exposure

- Each *recorded deferral* reject converts to an admission when its
  surface lands (collection expressions — Q5; collection assignment —
  host replace / Q5; per-item handlers — M4 input; nested `for` —
  family extension), lifting a reject rather than reshaping grammar.
- The carry rows' re-triggers above are the standing reactive-engine
  reopening conditions; they ride the implementation handoff into the
  next phase's constraints sweep.

## Strategic review disposition

- **Review F13 folded.** Added the qualified collection-reference row on
  the `for` header side so the matrix names the same DD-001
  reference-shape deferral that mutation LHS diagnostics name for
  `root.xs.append(...)`.
- **Review confirmation recorded.** No strategic change was requested
  for cap accounting, breadth-vs-depth charging, empty-collection
  legality, or the reactive-drain carry table.

## Recommendation-choice review disposition

- **Finding 1 folded.** Added the binder-in-`if`-condition reject row
  and tied it to DD-M3-P7-003's per-item conditional-presence deferral.
- **Finding 3 folded.** Added the non-literal collection element reject
  row so DD-M3-P7-002's scalar-literal grammar seed has a matrix-backed
  diagnostic and invalid example.
- **Minor note folded.** The >N cap fixture now records how the large
  cardinality is reached despite append/pop being the only authored
  mutations.

## Revision history

- Strategic owner-alignment review fold: added the qualified
  post-`in` collection-reference reject row and recorded that cap /
  empty-collection / reactive-drain judgments remain unchanged; status
  remains Proposed.
- Recommendation-choice review fold: added binder-in-`if` and
  non-literal collection-element reject rows, plus the >N fixture setup
  note; status remains Proposed.

## Technical risk re-evaluation

- **Matrix breadth is the main execution risk** (~20 reject branches
  × dual gate): mitigated by table-driven test fixtures and by the
  matrix living in one place (this DD) that the implementation plan
  checklists against — not rediscovered per-crate.
- **The ≫N convergence fixture** is cheap insurance against the only
  plausible cap surprise (an implementation accidentally running the
  tail edit as N separate effect runs — which *would* still be
  breadth-in-one-iteration, but the fixture would catch a
  depth-charging regression too).
- **Log-surface upgrade** (range-scoped diagnostics) touches the
  existing log-only path; it is additive context, not a contract
  change — PF2 (DD-005) owns the contract.
