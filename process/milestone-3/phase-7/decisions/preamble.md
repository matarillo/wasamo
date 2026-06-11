# M3-Phase 7 — Iteration grammar: Architecture Decisions

**Phase:** M3-Phase 7 (iteration grammar / collection-driven widget-tree generation)
**Date:** 2026-06-11
**Status:** Proposed

## Context

M3 acceptance criterion **A8** (see
[../../../_roadmap.md M3](../../../_roadmap.md),
[../../plan.md §Acceptance criteria](../../plan.md)):

> **A8** — Iteration grammar: collection binding drives widget-tree
> generation.

Phase 6's conditional rendering made a binding drive a subtree's
**present / absent** state (0/1). Phase 7 extends the same structural
control-flow family to **0..N cardinality**: a **runtime-mutable
collection binding drives the number of generated widget subtrees**.
The load-bearing distinction (framing FD-A, owner-aligned 2026-06-11)
is that this is **not static template expansion** — a fixed-N expansion
would be indistinguishable from a hardcoded tree and does not discharge
A8. The proof obligation is that the collection *changes at runtime*
and the generated subtree count changes with it (FD-B: a 2+ frame
append / truncate proof, never a single static frame).

Three further milestone obligations apply:

- **A11 (operational obligation).** `.ui` parser, `wasamo-ir`,
  `wasamoc`, `wasamo-runtime`, `docs/dsl_spec.md`, and the
  `examples/gallery/` sub-screen all advance within Phase 7. Iteration
  is a grammar surface; no side runs ahead alone.
- **A12 (DSL public-draft obligation).** Phase 7 adds the iteration
  grammar as a normative `docs/dsl_spec.md` section at the
  external-reader bar: `for` syntax, collection types / initial values /
  mutation surface, item / index scope, body shape, identity baseline,
  validation / diagnostics, and runtime mutation timing must be
  reproducible from the spec alone.
- **A1 (incremental gallery proof).** Phase 7 ships the WrapPanel +
  ScrollView-backed thumbnail sub-screen generated from a collection;
  Phase 8 assembles the full gallery.

The pre-doc framing for this phase was aligned with the owner on
2026-06-11 and is recorded in
[../requirements/framing.md](../requirements/framing.md) ("Owner
alignment outcome"). That alignment settled the framing decisions
FD-P / FD-Q / FD-A / FD-B / FD-C / FD-D / FD-E / FD-G / FD-F; the
remaining sub-decisions are recorded below as ADR `Recommendation`
directions for the `Status: Proposed` → `Accepted` review pass.

### The owner prior governing comparisons (FD-P)

Phase 7 DD comparisons take **product merit / thesis fit as the primary
axis**; implementation / revision cost is a tie-breaker only, never the
ground for rejecting an option. This is **not** maximalism: over-design
justified by future extensibility remains a named failure mode, and
schedule risk to Phase 8 / the public draft is handled in framing
§Risks, not in DD comparison tables. Each DD below rejects options on
merit and says so.

### Architectural-family confirmation (FD-Q)

Phase 7 fires `architectural-family.md` triggers 1 (M3 DSL spec
drafting) and 3 (a binding feature whose fit in `BindingTarget` must be
checked). The framing's reviewed judgment is **confirm**: iteration is
absorbed inside the tree-with-bindings family as a `ControlFlowNode`
/ `BindingTarget` internal extension; no pivot to a view-function
re-execution family and no new vision decision record is needed. This
is not a long-term ratification — the strain triggers remain live. The
confirm is written back to `architectural-family.md` at Moment 2
(revise-in-place, per FD-Q).

### End-state shape this phase extends without breaking

The shapes Phase 7 builds on (verified against the workspace at
drafting time):

- **`wasamo-ir`** ([wasamo-ir/src/lib.rs](../../../../wasamo-ir/src/lib.rs)):
  `IrType` is `I32 | Str | Bool`; `IrState { name, ty: IrType, default:
  IrLiteral }`; `IrMember = Widget(IrNode) | ControlFlow(ControlFlowNode)`;
  `ControlFlowNode::If { branches: Vec<ControlFlowBranch> }` with
  `ControlFlowBranch { condition: HandlerExpr, body: Vec<IrMember> }`
  (body enforced to a single `Widget` child at lowering / loader).
  `HandlerExpr` is the **single unified expression enum** shared by
  compiler and runtime (typed `PropRead` / `StrPropRead` /
  `BoolPropRead` variants; `Assign` / `CompoundAssign` statements;
  `Interpolation`; `Block`). Phase 7 extends `IrType`-adjacent state
  typing, `IrLiteral`, `HandlerExpr`, and `ControlFlowNode` — all
  same-family extensions, no second enum (constraint: settled premise).
- **`wasamo-runtime` reactive engine**
  ([wasamo-runtime/src/reactive.rs](../../../../wasamo-runtime/src/reactive.rs)):
  Signal / Effect two layers; `BindingTarget = WidgetProperty |
  ConditionalSubtree { parent, declared_member_index }`; the drain loop
  `drain_dirty_effects()` iterates the dirty set up to **`MUTATION_CAP
  = 16` drain iterations** — the cap counts **cascade depth**
  (drain-loop iterations), not effect count and not structural edit
  count. This fact is load-bearing for DD-007's cap accounting.
- **Placement storage**
  ([wasamo-runtime/src/widget.rs](../../../../wasamo-runtime/src/widget.rs)):
  parent-owned placement metadata is SoA — `WidgetData::Grid {
  cell_placements }` and `WidgetData::ZStack { zstack_placements }`
  are vectors kept **parallel** to `children`. Phase 6's conditional
  path needed `insert_child_with_zstack_placement` + paired removal to
  keep the invariant; the drift hazard is implementation-gates trap #3.
- **Grammar** ([docs/dsl_spec.md](../../../../docs/dsl_spec.md) §3):
  `member ::= property_decl | property_bind | widget_decl |
  signal_handler | state_decl | grid_track_list_member |
  conditional_member`; handler `statement ::= assign_stmt ";"` with
  `assign_op ::= "+=" | "-=" | "*=" | "/=" | "="`; `expr` admits **no
  operators**. `if` / `else` / `switch` / `for` are reserved keywords
  since Phase 6; **`in` was deliberately left unreserved** until its
  production exists — Phase 7 specifies that production, so Phase 7
  reserves `in` (DD-M3-P7-001).
- **Runtime-owned state.** `.ui` `state` lives in the runtime
  `SignalRegistry`; there is **no host-supplied initial-state bag and
  no host write / replace API** — for single values either
  ([host-state-boundary.md](../../../../docs/notes/host-state-boundary.md)).
  Phase 7's collection state is runtime-owned on the same basis; the
  general host state boundary is a separate thesis deferred with
  triggers (FD-C), but DD-002 records the representation constraints
  that keep a future host replace unblocked.

### Acceptance lens for this phase

- **A8** is satisfied when `.ui` declares a collection `state` and a
  `for` member over it, the shared crates lower → load → render it,
  and **mutating the collection at runtime inserts / removes generated
  subtrees** (cardinality tracks the collection), with the identity,
  drain, and atomicity contracts of DD-005 / DD-006 holding.
- **A11** is satisfied when the thumbnail slice advances every side and
  grows `examples/gallery/gallery.ui` additively.
- **A12** is satisfied when the iteration chapter lands in
  `docs/dsl_spec.md` at the external-reader bar by phase close,
  including invalid examples.
- The **plan-level obligations** are discharged in-ADR: `TypedValue`
  pressure is judged explicitly in DD-002; the reactive-drain residual
  fix-or-carry (with the cap accounting model) is judged in DD-007.

## Decisions

The Phase 7 ADR carries the seven framing-slate DDs (framing DD slate →
ADR numbering 1:1, FD-G):

| DD | Title | Decision summary (Proposed) |
|---|---|---|
| [DD-M3-P7-001](./dd-m3-p7-001-iteration-author-facing-grammar.md) | Iteration author-facing grammar surface | **`for`-block member** `for <binder> ("," <index-binder>)? in <collection> { <one widget child> }` — the same family shape as the Phase 6 `if` block. `in` becomes a reserved keyword (its production now exists). Body = **exactly one widget child per iteration** (mirror of B1; multi-member range deferred). Direct `for` admitted under **VStack / HStack / WrapPanel / ZStack**; rejected under **ScrollView** (one-content contract, symmetric with DD-M3-P6-007), **Box** (at-most-one), **Grid** (Cell-mediated children; `for`-of-`Cell` deferred), and at **component level** (no parent slot). |
| [DD-M3-P7-002](./dd-m3-p7-002-collection-value-surface-and-typedvalue-pressure.md) | Collection value surface, mutation statements, `TypedValue` pressure | Collection state types **`i32[]` / `string[]` / `bool[]`** with literal `[a, b, …]` / `[]`; IR carries `IrStateType::Scalar(IrType) \| Collection(IrType)` (a compile-error-forcing schema change) + `IrLiteral::List`. **Runtime-owned whole-value collection signals** (per-element-type seam; no `TypedValue`). Mutation = **handler statements** `xs.append(expr)` / `xs.pop()` — explicitly *statements*, not expressions, so the Q5 operator-uniformity rule is untouched. `TypedValue` **judged and not adopted** (trigger-backed defer); `f64[]` deferred; host-replace future-compat constraints recorded (whole-value set; positional element identity; value-semantic copy). |
| [DD-M3-P7-003](./dd-m3-p7-003-loop-local-context-scope-and-handler-admission.md) | Loop-local context, scope, handler admission | **Author-named binders** (no fixed magic `item` / `index` names); optional second binder = index (`i32`, read-only). Loop locals are **loop-local read-only bindings readable in binding-expression positions only** (FD-D — the first codified exception to all-references-via-state); they are **not** readable in `if` conditions, whose identifiers remain state-only this phase. Flat scope; binder ↔ state and binder ↔ binder collisions are errors; nested `for` anywhere inside a `for` body template is rejected (nested template scope deferred). **Per-item handlers rejected this phase** — a `signal_handler` member inside a `for` body template is a check error; admission deferred with the M4-input trigger. |
| [DD-M3-P7-004](./dd-m3-p7-004-ir-textual-ir-and-structural-traversal.md) | IR / textual IR representation + structural traversal | **`ControlFlowNode::For { binder, index_binder, collection, body }`** — a same-family variant beside `If`, body single-`Widget`-child (length-1) like Phase 6. Textual IR gains a `(for …)` member production; roundtrip preserves binders / collection ref / body. **Member-expansion is canonized**: one shared "declared members → materialised children" seam (prefix-sum index math over per-member live cardinality 0/1 for `If`, 0..N for `For`) used by both static load and reactive mutation — resolving the Phase 6 DD-007 reservation in the canonize direction. Runtime fills **`BindingTarget::ForLoopSubtree { parent, declared_member_index }`**. Semantic-migration call-site audit (gates trap #1) on every `IrMember` / `ControlFlowNode` match site. |
| [DD-M3-P7-005](./dd-m3-p7-005-runtime-identity-and-range-mutation-semantics.md) | Runtime identity baseline + range mutation semantics | Normative wording = **positional, un-keyed**: a generated subtree's identity is its position; **append materialises only the new tail items; `pop` disposes only the removed tail item; prefix subtrees are retained, not rebuilt** (per-item bindings are reactive positional reads, so the contract survives a future whole-value host replace). Keyed retention stays opt-in-future (never a silent default change). Range mutation is **stage-then-commit**: all fallible construction happens before any tree splice; a staging failure aborts the whole mutation observably unchanged (+ diagnostic). Disposal order: effects disposed ahead of teardown, tail-first. The setter-return drain contract (M3-Phase 1 item 4) is **preserved**: on handler return the new subtrees' effects have run. |
| [DD-M3-P7-006](./dd-m3-p7-006-placement-storage-and-structural-side-effects.md) | Placement storage model + structural side-effect atomicity | **Child-carried placement**: ZStack per-child placement moves from the parallel `zstack_placements` vector onto the child slot, so a child and its placement cannot drift (the trap-#3 class is removed structurally for every `for` / `if`-touched path, not policed by helper discipline). Grid `cell_placements` migration is **deferred with a trigger** (Grid rejects direct `for` this phase). One **range-splice primitive** owns the full side-effect set: child list, placement, layout dirty, Visual sibling order, registry, effects (gates traps #2 / #3 close artifacts mandatory). |
| [DD-M3-P7-007](./dd-m3-p7-007-validation-diagnostics-cap-and-reactive-drain.md) | Validation, diagnostics, cap accounting, reactive-drain disposition | Full reject matrix at `wasamoc check`, re-checked by the loader (`WASAMO_ERR_IR_MALFORMED`): non-collection `for` target, binder collisions, disallowed containers, component-level `for`, nested `for`, handler-in-body, binder-in-`if` condition, bad body shape, heterogeneous / non-scalar literals, non-literal collection elements, mutation statements on non-collections, element-type mismatches, qualified collection mutation LHS, whole-collection assignment. Empty collection ⇒ 0 generated children is **legal** in admitted containers. **Cap accounting fixed: `MUTATION_CAP` counts drain-loop iterations (cascade depth), so N-item breadth does not consume cap** — evidence required that the gallery proof stays ≪ 16. Reactive-drain residual items 1–3 **carried** with explicit record (no new failure mode surfaced; breadth ≠ depth); item 4 preserved. Every new reject branch gets a direct failure-path test (trap #4). |

## Owner confirmation before Accepted

- The framing's `item` / `index` vocabulary is accepted as placeholder
  wording for author-named binders, not as fixed magic names; `item` and
  `index` remain valid conventional binder names.

## Recommendation-choice review disposition

- **Finding 2 folded.** The placeholder-vs-fixed-name interpretation is
  now an explicit owner confirmation item before the Accepted flip; no
  DD recommendation changes.

## Implementation-readiness review disposition

- **Finding 1 folded.** DD-005 defines the removed-item `ItemRead`
  guard and DD-007 adds the direct `pop` test row.
- **Finding 2 folded.** DD-007's cap-accounting mechanism now matches
  source behaviour while preserving the breadth-not-depth conclusion.
- **Finding 3 folded.** DD-002 specifies empty-`pop` as no signal write
  and the verification closure pins no dirty re-run.
- **Finding 4 deferred.** Implementation-plan first task: design the
  instantiation context type (element tag, signal reference, position,
  live/out-of-range guard); DD variant spelling remains intentionally
  adjustable.
- **Finding 5 deferred.** Implementation-plan sequencing concern:
  order C1, ST2, splice, and `for` work so intermediate commits remain
  bisectable.
- **Finding 6 deferred.** Implementation-plan test refinement: load
  path must prove static materialisation plus initial `for` effect does
  not double-create children.

## Cross-DD decision dependencies

Three couplings span DDs; the primary DD owns the choice, dependents
carry the consequence (index only — arguments live in the owning DDs):

| Coupling (bundle) | Primary DD | Dependent DDs | Proposed bundle |
|---|---|---|---|
| **Iteration body shape** | DD-M3-P7-001 (one widget child per iteration) | DD-M3-P7-004 (body template length-1), DD-M3-P7-005 (per-item subtree grain), DD-M3-P7-007 (body-shape rejects) | one `widget_decl` per iteration → each item materialises exactly one child → tail splice adds/removes whole single-child subtrees. The member-range alternative shifts all three and is deferred, not chosen. |
| **Collection value representation** | DD-M3-P7-002 (whole-value per-type signals; statements not expressions) | DD-M3-P7-005 (positional reactive item reads), DD-M3-P7-003 (binder read lowering), DD-M3-P7-007 (mutation-statement rejects) | whole-value signal → for-effect computes cardinality diff → item binders lower to positional collection reads → append/pop are the only authored mutations. |
| **Member-expansion canonization** | DD-M3-P7-004 (shared declared→materialised seam) | DD-M3-P7-005 (insertion index math), DD-M3-P7-006 (splice primitive call shape) | one prefix-sum seam shared by static load and reactive mutation; `If` becomes the 0/1 special case of the same math. |

## Phase 7 verification closure (what counts as A8 evidence)

Per the framing verification strategy and the positive-control
constraint (a fixed-N single frame is **not** evidence — it cannot be
distinguished from a hardcoded tree), Phase 7 closes only when all
seven are observed:

1. **`wasamoc check` evidence (pure logic).** Positive controls: the
   gallery `.ui` and representative `for` fixtures compile and lower to
   `ControlFlowNode::For` with the declared binders / collection /
   single-child body. Negative controls: every reject in the DD-007
   matrix fires its own diagnostic (non-collection target, binder
   collision, disallowed container, component-level `for`, nested
   `for`, handler-in-body, binder-in-`if` condition, multi-child /
   non-widget body, bad literal, non-literal collection element, bad
   mutation statement, element-type mismatch, `in` / `for` used as
   identifiers).
2. **Pure-logic reducer / planner evidence.** The cardinality diff
   planner (old length → new length ⇒ tail insert / remove plan,
   declared-slot insertion index via the canonized prefix-sum seam) is
   exercised as free functions without WinRT, including interleaved
   `if` / `for` siblings, 0-length cases, and load-time
   materialisation where the `for` effect's initial run does not create
   duplicate children.
3. **Lowering / textual-IR roundtrip / loader evidence.** Emit → load
   preserves the `for` member (binders, collection ref, body); loader
   rejects malformed `for` shapes (`WASAMO_ERR_IR_MALFORMED` dual
   gate); collection state declarations and list literals roundtrip.
4. **Windows-runtime integration evidence (mock-free, CI-gated,
   fail-not-skip).** Live `WidgetNode` / Visual assertions: after
   `append` the child count and Visual sibling order reflect the new
   cardinality, in declared order with static siblings and `if`
   members flanking the `for` slot; after `pop` likewise; prefix
   subtree pointers are **unchanged** across a tail append (positional
   retention positive control); disposed tail subtrees release effects
   and registry entries; handler-return drain observability (item 4)
   holds — immediately after the mutating call the new subtrees'
   bound properties are written; empty-`pop` produces no dirty re-run;
   a same-batch dirty removed-item binding skips its out-of-range read;
   ZStack-path range mutation updates child-carried placement and
   Visual order in one splice. Fixtures
   fail (not skip) where the Compositor is unavailable; multi-test
   binaries reuse the Phase 6 keep-alive apartment helper.
5. **Assistant-visible GUI evidence.** Launch + DPI-aware screenshot +
   assistant analysis, **2+ frames**: initial N, after append N+1,
   after pop back to N (or N−1) — item count visibly tracks the
   mutation driven by body-external text Buttons. DPI blur is noted as
   the known M4 residual, not a Phase 7 failure.
6. **End-to-end host evidence (visible smoke).**
   `examples/gallery/gallery.ui` grows additively: the thumbnail set is
   generated by `for` over a collection `state`, with `Add` / `Remove`
   text Buttons outside the body driving `append` / `pop`. Owner
   human-visible smoke is a separate gate from item 5.
7. **A12 spec-closure gate (non-test).** `docs/dsl_spec.md` carries the
   iteration chapter at the external-reader bar — grammar, collection
   types / literals / mutation statements, binder scope rules, the
   positional un-keyed identity baseline stated normatively, runtime
   mutation timing, validation / invalid examples matching items 1 / 3
   — with the Moment 1 → Moment 2 marker flip completed.

## Forward-compat exposure (anticipated future surfaces)

All additive; none requires reshaping what Phase 7 ships. The
**activation-trigger 正本 is the framing scope table**
([../requirements/framing.md §Out of scope](../requirements/framing.md));
this list names the ADR-side landing points only:

1. **Keyed identity / retained state** — re-evaluated at the M4 input /
   focus / TextField pre-doc. The declared `for` member is the stable
   anchor; keyed retention arrives as an opt-in (`key:`-like) surface
   over the same `ForLoopSubtree` seam, never as a silent change to the
   positional baseline (DD-005).
2. **Data-driven reorder** — M5 / collection-UX; requires an ordering
   contract + keyed diff; the positional baseline deliberately excludes
   it (append / pop only).
3. **Structured item fields / `TypedValue`** — judged not-adopted in
   DD-002; reopens via the recorded triggers (record-like item state,
   `item.field` access, scalar insufficiency in a concrete app case).
4. **`f64[]`** — fourth scalar collection element; DD-002 defers with
   trigger (f64-needing concrete case).
5. **Host state boundary** (host-supplied initial collection, host
   replace, write-back) — M4 host bindings / M6 ABI-freeze wave;
   DD-002 records the representation constraints (whole-value set,
   positional identity, value-semantic copy) that keep it unblocked.
6. **Loop-external collection reads** (`length`, empty checks, element
   index reads) — Q5 uniform expression / reference extension; DD-002
   records the deferral, DD-007 owns the reject diagnostic until then.
7. **Per-item handlers / `item` in handler position** — rejected this
   phase (DD-003); lands with M4 per-item interaction.
8. **Per-item conditional presence** (loop-local binder in an `if`
   condition) — rejected this phase (DD-003); reopens on the framing
   FD-F trigger: the first concrete UI case needing per-item display /
   state branching from `bool` elements, naturally at M4 input per-item
   interaction or the next structural control-flow extension, whichever
   comes first.
9. **Nested `for` / template scope & shadowing** — with the next
   structural control-flow extension (`else` / `switch` / bare nesting).
10. **Member-range `for` body / multiple members per iteration** — the
   deferred body generalisation (DD-001); lands on the canonized
   expansion seam without IR reshaping.
11. **Grid placement migration to child-carried storage** — DD-006
   trigger: Grid admitting structural mutation (direct `for` /
   conditional Cells).
12. **LazyList / large-N performance** — M5+; small-N machinery proof
    is deliberate (FD-C).

## Out of scope

The deferred-items **正本** (with activation triggers and
responsibility landings) is the framing scope table
([../requirements/framing.md](../requirements/framing.md)); per FD-F
this ADR does not duplicate it. Out of A8 scope this phase, by
decision: keyed identity / retained state; data-driven reorder;
structured item fields / `TypedValue`; `f64[]`; host state boundary
(initial state / replace / write-back); loop-external collection reads;
per-item handlers and handler position `item` reads; loop-local binder
reads in `if` conditions / per-item conditional presence; nested `for`
/ template scope; member-range bodies; whole-collection assignment in
handlers; Grid / Box / ScrollView direct-`for`; large-N performance;
Image widget (thumbnails remain Box + Text placeholders); per-monitor
DPI (M4).

## Upstream document revisions (Moment 1 / Moment 2)

Per-review-concern commit rule applies
([AGENTS.md §Commit rules](../../../../AGENTS.md)). Touch / no-touch
judgments are explicit:

**Moment 1 — ADR Accepted commit set (design-spec draft):**

- This directory — ADR `Status: Accepted` flip.
- [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) — **touch.** New
  iteration chapter beside §4.14 (conditional rendering), continuing
  the structural-rendering-model family story; §3 grammar additions
  (`iteration_member`, collection `state_type`, list literal,
  `collection_stmt`); §2.1 keyword table adds **`in`** (and notes
  `for` now has a production); §state-decl section gains collection
  types; textual-IR chapter gains the `(for …)` member, list-literal,
  collection-read and mutation-statement productions plus loader
  validation policy; invalid examples per DD-007. Existing `for`
  forward references are **swept for staleness** in the same touch
  (FD-E / answers §5-3 — the dsl_spec side of the same live-doc-sync
  lane as the architecture §9 revision below). Two are known stale at
  drafting time, both in §4.14: the single-widget-body note's "the
  multi-child range form is the Phase 7 `for` driver" (DD-001 ships
  the single-widget body; the member-range form is deferred to the
  family extension, not chosen) and the structural-control-flow family
  list's "`for` … is the first construct to need keyed identity /
  state retention" (DD-005 ships the un-keyed positional baseline;
  keyed is opt-in future, M4-input-triggered). No DD option labels
  in spec prose. Marker: `M3-Phase 7 design accepted; implementation
  pending`.
- [`docs/architecture.md`](../../../../docs/architecture.md) —
  **touch.** §6.7.7/§6.7.8 `BindingTarget` gains `ForLoopSubtree`;
  the canonized member-expansion seam and the stage-then-commit range
  mutation contract are described; §6.9 ZStack subsection updated for
  child-carried placement; §9 three-layer-tree note updated — the
  stale "keyed item identity … the Phase 7 `for` driver" sentence is
  **revised** per FD-E (Phase 7 generalises the un-keyed base;
  keyed is opt-in future, M4-input-triggered).
- [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no touch
  (judged).** Collection state is runtime-owned; no host API is added
  (FD-C). If implementation surfaces an unavoidable ABI need it is
  recorded at Moment 2 with owner confirmation.
- [`docs/notes/architectural-family.md`](../../../../docs/notes/architectural-family.md)
  — at **Moment 2** (per FD-Q): alignment table + re-evaluation
  triggers gain the Phase 7 trigger-1/-3 confirm entry,
  revise-in-place.
- [`../../plan.md`](../../plan.md) — Phase 7 row populated.
- `implementation/preamble.md` / `plan.md` — opened after acceptance,
  with the final-task ownership split represented from the start.

**FD-E live-doc-sync timing (judged):** answers §5-3 places the
stale-reference sync at Moment 2. It is front-loaded to Moment 1
because the staleness is created by the **Accepted flip itself** —
from that commit until phase close, architecture §9 and the two
dsl_spec §4.14 forward references would contradict accepted DDs
(DD-001 / DD-005) in normative docs — and because the affected
sections are already rewritten in the Moment 1 design-sync commit, so
the revisions share its review concern. The revision is not duplicated
at Moment 2: the Moment 2 divergence-correction pass re-verifies the
revised sentences against the implementation like every other synced
statement.

**Moment 2 — Phase close commit set (impl re-sync):** dsl_spec /
architecture markers flip to `closed; implementation-synced` with
divergence corrections; `architectural-family.md` confirm entry lands;
plan row flips complete; phase-end retrospective + CI run-id ownership
per the split.

## Inputs absorbed

| Source | Disposition | Consumed at |
|---|---|---|
| FD-P — product-merit comparison prior | Discipline | every DD §Comparison; schedule risk stays in framing §Risks |
| FD-Q — architectural-family confirm | Settled framing | §Context; Moment 2 write-back |
| FD-A — cardinality thesis, A8 unrevised | Constraint | §Context; DD-001 / 002 / 005 |
| FD-B — 2+ frame mutation proof, runtime-owned state, body-external Button | Constraint | §Verification closure items 4–6 |
| FD-C — un-keyed / append-truncate / scalar / flat boundary; host boundary deferred with future-compat record | Settled framing | DD-002 / 003 / 005; §Out of scope |
| FD-D — loop-locals = expression-position read-only exception | Settled framing | DD-003 |
| FD-E — keyed expectation revised via live-doc sync, no retroactive ADR edits | Discipline | §Upstream revisions (architecture §9 + dsl_spec §4.14 `for` forward references) |
| FD-G — 7-DD slate | Structure | §Decisions |
| FD-F — deferred-items 正本 in framing | Discipline | §Out of scope / §Forward-compat |
| constraints.md §1–§10 | Constraint set | DD-004 (§1, §8), DD-005 (§2, §3, §5), DD-006 (§4), DD-007 (§6), DD-002 (§7), verification (§9), process (§10) |
| owner-intent-answers §3 notes 1–3 | Obligations | DD-005 (normative wording explicit), DD-002 (statement-vs-expression line), DD-007 (cap accounting before carry) |
| host-state-boundary.md | Boundary input | DD-002 future-compat record |
| dsl-grammar.md Q1 / Q5 / Q6 / Q8 | Thesis inputs | DD-001 (family form), DD-002 (uniformity line), DD-003 (id ≠ key) |
| Phase 6 DD-M3-P6-003/004/005/007 | Pattern reuse + family base | DD-001 / 004 / 005 / 007 |

## Revision history

| Date | Change |
|---|---|
| 2026-06-11 | Initial draft (Status: Proposed). All 7 DDs at Proposed pending owner review. Framing-level owner alignment confirmed 2026-06-11 ([../requirements/framing.md](../requirements/framing.md) §Owner alignment outcome). |
| 2026-06-11 | Recommendation-choice review fold: recorded owner confirmation for placeholder `item` / `index`, reflected binder-in-`if` and non-literal collection-element rejects, synced the framing FD-F trigger, and kept status Proposed. |
| 2026-06-11 | Implementation-readiness review fold: closed the removed-item read guard, corrected cap accounting, specified empty-`pop` no-dirty behaviour, and deferred plan-only sequencing/context/load-test findings; status remains Proposed. |
| 2026-06-11 | Doc-sync completeness review fold: added the dsl_spec §4.14 `for` forward-reference staleness sweep (answers §5-3) to the Moment 1 dsl_spec touch, and recorded the judged Moment 1 front-load of the FD-E live-doc sync with Moment 2 re-verification; status remains Proposed. |