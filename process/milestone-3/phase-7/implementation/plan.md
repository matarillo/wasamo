## Task list

Phase 7 ships one grammar surface (iteration) whose work spans every
side (A11): the `wasamoc` author surface (T3), an IR-schema migration
(T2), two runtime structural refactors decided **before** the range
primitive (T4 seam canonization, T5 placement migration), the loader
static path (T6), the reactive range-mutation runtime (T7), and the
gallery slice + close gates (T8–T10). The final-task ownership split
([preamble.md §Step-end / phase-end retrospective split](./preamble.md#step-end--phase-end-retrospective-split-final-task-ownership))
is represented in T10 from the start.

Default to **one commit per task-list item** per
[AGENTS.md §Commit rules](../../../../AGENTS.md#commit-rules). The
known exception this phase is the **`IrStateType` schema change**
(preamble risk R-A): it breaks `wasamoc`, the textual-IR emit / load
path, and the runtime registry simultaneously, so it bundles into one
buildable commit (recorded in T2). If implementation reveals an item
should split or reorder, revise this list so it stays an accurate
record rather than a frozen prediction.

### T0 — Moment 1 document sync

Opens execution after ADR acceptance and records the design draft in
the upstream documents named by the ADR's
[Moment 1 commit set](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2).
T0 closes when this implementation plan lands and all preceding
Moment 1 commits are on the pre-doc branch. Implementation (T1) begins
only after T0 closes.

- [x] `process/milestone-3/phase-7/decisions/preamble.md` and
      `dd-m3-p7-001` through `dd-m3-p7-007` flipped to
      `Status: Accepted`, with both owner confirmations recorded
      (`item` / `index` as placeholder vocabulary; `append` /
      `drop-last` fixed) (commit `cad3891`, 2026-06-13).
- [x] `docs/dsl_spec.md` — §4.15 iteration chapter added as the second
      chapter of the structural rendering model (`for` block,
      author-named binders, collection state types / literals,
      whole-value collection assignment, positional un-keyed identity
      baseline + keyed non-promise, mutation timing, admission sweep,
      diagnostics matrix with invalid examples); §2.1 `in` reservation;
      §2.2 / §3 / §4.6 / §4.7 / §5 / §8.4 / §8.5 / §8.9 / §8.11
      supporting additions with a worked declared-slot-offsets example;
      stale §4.14 `for` forward references swept (single-widget body,
      not member-range; positional baseline, not keyed). Phase status
      marker `M3-Phase 7 design accepted; implementation pending`
      (commit `7dbd18e`, v1.8).
- [x] `docs/architecture.md` — §6.7.10 iteration runtime shape
      (`ControlFlowNode::For`, whole-value collection signals, the
      canonized member-expansion seam, `ForLoopSubtree`, live
      positional reads + guard, stage-then-commit, the splice seam's
      side-effect bundle, drain contract, cap charging model); §6.8.5
      child-carried placement contract (+ §6.8.4 Grid defer pointer);
      §6.7.7 / §6.7.8 seams filled; §9 stale keyed-expectation sentence
      revised per the live-doc-sync rule; top Status marker added
      (commit `cf05f90`).
- [x] `docs/abi_spec.md` deliberately untouched at Moment 1 per the
      ADR preamble (judged no-touch): collection state is
      runtime-owned; no host API is added; the `For` construct adds no
      host-facing ABI surface. If implementation re-sync surfaces an
      unavoidable ABI need, it is recorded at Moment 2 with owner
      confirmation.
- [x] `process/milestone-3/plan.md` Phase 7 row populated (Status
      `in progress`; Progress-file link → this directory; ADR link →
      the ADR set; `TypedValue` not-adopted judgment in the Notes
      column) — **this review batch (commit pending owner review).**
- [x] `process/milestone-3/phase-7/implementation/preamble.md` and
      this `plan.md` opened with `status: active`, the ADR
      §Obligations represented in T1 / T6 and the sequencing default,
      and the final-task ownership split represented in T10 from the
      start — **this review batch (commit pending owner review).**

### T1 — Pre-implementation spike: instantiation context + sequencing

Discharges ADR obligations 1 and 2
([preamble.md §Obligations](./preamble.md#obligations-carried-from-the-adr-represented-in-this-plan-from-the-start)).
No production code lands; outputs are recorded decisions in
[log.md](./log.md) plus any revision of this plan.

- [x] Design the **instantiation context type** — element type tag,
      collection signal reference, fixed position, live /
      out-of-range guard — against the current `reactive.rs` /
      `ir_loader.rs` source; record the chosen shape (DD variant
      spellings remain adjustable without reopening the DDs). Recorded
      in [log.md](./log.md) §1 (`ForItemContext { collection, elem,
      position }` runtime carrier + bare loop-local read markers +
      guarded out-of-range read).
- [x] Fix and record the **bisectable sequencing** of I2 (T2), the
      wasamoc surface (T3), C1 (T4), ST2 (T5), the loader static path
      (T6), and the splice primitive + `for` effect (T7); revise this
      plan if the default order changes. **Default order kept**; the
      three inter-task seams (Seam A: T2 deferred-load `For` reject;
      Seam B: T4 `ForLoop` slot dead until T6; Seam C: T6 no-op initial
      reconcile, T7 fills the effect body) are recorded in
      [log.md](./log.md) §2 with the CF-1..CF-5 carry table.
- [x] Sharpen the preamble §Technical risks table against the current
      source (pin file/line hotspots for R-A / R-B / R-C); record the
      implementation-gates selection for T2 before opening it. Pinned
      hotspots + T2 gate selection (full-review lane; traps #1/#4/#5
      apply, #2/#3/#7 non-applicable with reasons) in
      [log.md](./log.md) §3.

### T2 — IR schema migration: collection state typing + `For` variant

The **schema / IR-migration full-review-lane** task (gates trap #1).
Lands `wasamo-ir` + `wasamoc` emit/lower + textual-IR parse + runtime
loader/registry migration as one buildable commit bundle (risk R-A).

- [x] `wasamo-ir`: `IrState.ty` → `IrStateType::Scalar(IrType) |
      Collection(IrType)` (compile-error-forcing; `Collection` cannot
      nest); `IrLiteral::List(Vec<IrLiteral>)`
      (scalar-homogeneous, enforced at check / loader);
      `ControlFlowNode::For { binder, index_binder, collection, body }`
      (body `Vec<IrMember>`, length-1 / `Widget`-only enforced —
      shared enforcement helper with `If`, not duplicated);
      `HandlerExpr` gains the typed collection read (one variant
      carrying the element tag), the loop-local reads, and the
      collection-assignment forms (single unified enum; exact
      spellings per T1).
- [x] Migrate every construction / match site across `wasamoc`, the
      textual-IR emitter / loader, validators, and the runtime
      registry so the workspace builds; `SignalRegistry` gains the
      three whole-value collection signal maps with a value-equality
      check on set (equal-value writes mark nothing dirty).
- [x] IR-type unit tests cover the state-typing encoding, the list
      literal, and the `For` member encoding.
- [x] **Close artifact (trap #1):** the `rg`-enumerated call-site
      audit table over `IrState` / `IrMember` / `ControlFlowNode` /
      `HandlerExpr` (+ `BindingTarget` pre-audit for T7), each site
      classified (extended / correctly unaffected / deliberately
      rejects) — `IrNode::widget_children()` and every widget-only
      filter explicitly classified. Recorded in [log.md](./log.md).

### T3 — `wasamoc` author surface: parse, check matrix, lower, emit

Discharges ADR evidence item (1) (compile-time positive + negative
controls) and the emit half of item (3). Branch/test-focused review
tier for the reject branches; every matrix row fires a direct test
(trap #4). T3 owns the author-reachable grammar and diagnostics only:
loader dual-gate rejections stay T6-owned, and runtime guarded reads /
collection writes stay T7-owned.

- [x] Lexer: reserve **`in`** (keyword enum + `scan_ident`); reject it
      at identifier positions; regression test pins `in-out` as an
      unaffected single hyphenated lexeme. Bracket / paren / comma
      tokens as needed by the grammar below.
- [x] AST + parser: collection-aware `TypeName` / list-literal /
      collection-expression shapes; loop-local identifier expressions
      that can be resolved only under a `for` body; the `for` member
      (`for IDENT ("," IDENT)? in IDENT {
      iteration_body }`, LL(1) after the first IDENT); collection
      state types (`i32[]` / `string[]` / `bool[]`) + collection
      literal defaults; the collection-assignment statement
      (`IDENT "=" collection_expr` with `append` / `drop-last` /
      literal RHS; contextual method names — a state named `append` or
      `drop-last` still parses, positive test).
- [x] `wasamoc check`: the full author-reachable DD-M3-P7-007
      compile-time matrix —
      header/target rows (non-collection target, non-IDENT target,
      qualified reference, binder collisions, keywords as binders),
      placement rows (ScrollView / Box / Grid / component-level),
      body rows (non-widget member, handler-in-body, multi-child, bare
      control-flow body, nested `for` at any depth), binder-read rows
      (outside body, undeclared, handler position, `if` condition),
      collection declaration/literal rows (nested types,
      heterogeneous / mismatched / non-literal elements, list-on-scalar
      and vice versa), collection-assignment rows (scalar LHS,
      compound ops, arity, wrong receiver / chained / bare copy, bare
      statement, qualified LHS / receiver, `collection_expr` outside
      RHS), **loop-external collection-read rows (bare name / whole-value
      qualified read / member navigation / interpolation / scalar-RHS at
      check, indexed read at parse)**, and the **bool-element loop-binder
      interpolation reject**. Each diagnostic names its deferral where the
      row is a recorded deferral. Rows that only exist in textual IR or
      runtime evaluation are explicitly mapped to T6/T7 in the close
      branch map. (The two bolded rows were added in the in-task review
      remediation `fccd277`; the loader dual-gate for textual-IR
      `for`-external reads is carried to T6.)
- [x] Lower → `ControlFlowNode::For` + collection state / literal /
      assignment forms; textual-IR emit per dsl_spec §8.4 / §8.5 /
      §8.9; binder reads in body bindings lower to the typed
      loop-local reads.
- [x] Tests: positive controls (representative `for` fixtures + the
      gallery shape compile and lower with declared binders /
      collection / single-child body; index-binder form; empty
      initial value) + one reject test per matrix row; emit roundtrip
      shape tests.

### T4 — C1: canonize the member-expansion seam

Discharges ADR evidence item (2). The riskiest refactor of the phase
(risk R-B): touches the shipped Phase 6 conditional path. Own task,
own commit, full independent review (runtime structural change).

- [x] Introduce the shared **declared slot expansion** seam as pure
      logic over runtime slot cardinalities: widget = 1, `If` = 0/1,
      `ForLoop` = current collection length. The seam owns prefix-sum
      materialised offsets, total materialised child count, and tail
      range plan derivation from old length → new length. It does **not**
      build `for` children, evaluate collection reads, or cache offsets;
      those remain T6/T7-owned.
- [x] Add the bounded `DeclaredMemberSlot::ForLoop` representation now,
      ahead of first production construction (Seam B from T1). It is
      test-constructed only in T4 so the pure seam can prove interleaved
      `if` / `for` / static siblings; T6 closes the dead-production
      allowance by constructing it from loader `for` members.
- [x] Migrate the Phase 6 conditional path
      onto `materialized_offset_for_declared_slot` as the canonical
      seam's 0/1 special case, preserving the shipped conditional
      insertion / removal behavior.
- [x] Pure-logic unit suite: interleaved `if` / `for` / static
      siblings, zero-cardinality slots, boundary slots, tail
      insert/remove/no-op plan derivation (old length → new length), and
      load-time materialisation counts.
- [x] Phase 6 declared-order Windows fixtures run unchanged as the
      regression gate.

### T5 — ST2: ZStack child-carried placement migration

The placement-storage decision executed **before** the range primitive
(DD-M3-P7-006; risk R-C). Own commit preceding T7; full independent
review (runtime structural change).

- [x] Move ZStack per-child placement from the parent-owned parallel
      `zstack_placements` vector onto the child slot across all three
      runtime faces: `WidgetNode` mutation/storage, `LayoutNode`
      arrange/build-tree transfer, and the loader's static /
      conditional insertion paths. The parent still interprets the
      placement: ZStack children carry `Some(explicit-or-default
      placement)`, placement-free parent insertions normalize the
      child slot to `None`, and the existing conditional-under-ZStack
      path stays behaviorally green before T7 introduces the unified
      splice seam.
- [x] Grid `cell_placements` stays parallel-vector and static-only;
      the SoA comment in `widget.rs` gains the DD-M3-P7-006 trigger
      pointer.
- [x] **Close artifact (trap #3):** no parallel placement vectors
      remain on mutated paths (`zstack_placements` deleted —
      greppable); Phase 6 ZStack fixtures (union sizing, alignment
      defaults / overrides, conditional-under-ZStack placement) green
      as regressions.

### T6 — Runtime loader: `for` member load + static materialisation

Discharges the loader half of ADR evidence item (3) and ADR
obligation 3's static-load half (load-path test refinement). Because
this task changes `for` from a build-time reject into runtime tree
materialisation, it takes the full independent review; its loader
reject additions also receive the branch/test-focused check.

- [x] Loader parses the textual-IR `for` member (binders, collection
      read, single-widget body) and the collection state declarations
      / list-literal defaults; emit → load roundtrip preserves all of
      them.
- [x] Static load materialises the `for` slot's initial cardinality
      from the collection's initial value through the T4 seam —
      including the **empty-initial case (zero children, member
      live)** — and constructs the first production
      `DeclaredMemberSlot::ForLoop`.
- [x] Initial per-item bindings for statically materialised `for`
      children evaluate through `ForItemEvalContext`-style registration
      entry points: value and index loop-local reads are supplied from
      `{ collection, elem, position }`, guarded out-of-range reads write
      nothing, and the generated child's own `bindings` owns the
      EffectHandle.
- [x] **Load-path test (obligation 3 split):** T6 proves static
      materialisation is single-pass and does not double-create
      children before any structural `for` effect exists. T7 owns the
      complementary proof after `BindingTarget::ForLoopSubtree` and its
      initial effect run land.
- [x] Loader `validate()` dual-gate re-checks of the structural matrix
      rows (`WASAMO_ERR_IR_MALFORMED`): non-collection / unresolved
      collection read, bad body shape, bad binders, disallowed
      container / component level, nested `for`, handler-in-body,
      loop-local `item-read` / `index-read` position and scope
      violations exposed through textual IR,
      collection-declaration and collection-assignment violations,
      **`for`-external collection reads exposed through textual IR (a
      `list-prop-read` or member navigation outside a `for` body — the
      loader counterpart of the T3 loop-external read reject closed in
      `fccd277`)** —
      each with a direct test (trap #4). Preserve T2's stricter
      scalar-default gate: scalar/scalar default mismatches such as
      `state count: i32 = true` remain loader rejects.

### T7 — Reactive range mutation: splice seam + `for` effect

Discharges ADR evidence item (4) — the novel-runtime task. Full
independent review (runtime structural change + mock-free Windows
runtime evidence, not assistant-visible screenshot evidence); gates
traps #1 (BindingTarget/HandlerExpr sites), #2 (side-effect
enumeration), #3 (parallel/derived state sync for declared slots and
child-carried placement), #4 (mutation-time reject/diagnostic paths),
and #5 (carry-forward ownership). T7 does not own the gallery
composition or screenshot positive controls; those remain T8-owned.

- [x] **Handler-side collection-assignment evaluation (T1 addendum
      CF-6):** the authored `xs = xs.append(e)` / `xs = xs.drop-last()` /
      `xs = [..]` runs inside a handler — extend `HandlerEvalContext`
      with a whole-value collection read-modify-write method and add the
      collection-assignment `HandlerExpr` arm to the handler evaluator
      (`invoke_handler` / `evaluate`), driving `Signal::set` on the
      whole-value collection signal. This is the **writer** the `for`
      effect (below) reacts to; the equal-value no-dirty rule (CF-5)
      applies. Without it the mutation fixtures cannot drive a signal
      change. Collection `HandlerExpr.elem` is authoritative only after
      the loader's annotation pass; if T7 constructs collection handlers
      outside `parse_ir`, it must re-derive or otherwise prove the same
      element type before evaluation. Trap #1 (new evaluator arm) + trap
      #4 (the equal-value and bad-RHS runtime paths each fired).
- [x] **Splice seam (DD-M3-P7-006):** one placement-aware mutation
      seam owning the six-item side-effect bundle (children splice
      with carried placement, Visual sibling order at seam-computed
      positions, layout invalidation, registry release/registration,
      effect disposal-ahead-of-teardown / attach-at-commit, no other
      parent-owned metadata); the Phase 6 conditional mutation routes
      through it. **Reuse (T1 addendum 2 F-2):** side-effects #4
      (registry release) and #5-removal (effects disposed ahead of
      teardown) already exist as `widget_destroy` →
      `dispose_subtree_bindings` (bindings → registry → drop, recursive);
      the removal path reuses it per removed subtree, tail-first, rather
      than re-implementing #4/#5. **Close artifact (trap #2):** the
      bundle checked off per change, marking #4/#5-removal *reused*.
- [x] **`BindingTarget::ForLoopSubtree` + `for` effect:** reads the
      whole-value signal, computes the tail plan via the T4 seam,
      preserves T6's static-load result on its initial run (no
      double-create),
      executes **stage-then-commit** (DD-M3-P7-005 PF2): all fallible
      construction before any tree mutation; staging failure disposes
      staged work, logs a **range-scoped** diagnostic, leaves the tree
      observably unchanged. Staged-disposal branch directly fired
      (pure-logic staging planner test; fault-injected construction if
      feasible mock-free, else disposition recorded in log.md).
- [x] **Mutation-time per-item bindings:** reuse the T6
      `ForItemEvalContext` / guarded registration entry points for
      staged tail-inserted children, and directly prove the
      mutation-time branches T6 cannot observe: same-batch
      out-of-range read skips, same-length reset updates retained
      positions, and tail-removal disposes child-owned effects.
      **Effect ownership (T1 addendum 2 F-1):** per-item value/index
      effects are owned by the **generated child subtree's** `bindings`,
      **not** the parent (unlike the Phase 6 conditional effect) — so
      `widget_destroy` on tail-removal disposes them; the
      `ForLoopSubtree` structural effect stays on the parent. A
      parent-parked per-item effect would leak on removal.
- [x] **Windows-runtime fixtures (CI-gated, fail-not-skip):** after a
      tail-append assignment — child count + Visual sibling order
      reflect the new cardinality in declared order with static and
      `if` siblings flanking the `for` slot; **prefix subtree pointers
      unchanged** (positional retention positive control); after a
      tail-remove — disposed subtrees release effects + registry
      entries, tail-first; handler-return observability (drain item 4)
      holds; empty-collection `drop-last` writes an equal value and
      produces **no dirty re-run**; a **same-length static-literal
      reset** re-evaluates item bindings in place with no structural
      edit and prefix pointers unchanged; a **same-batch dirty
      removed-item binding** skips its out-of-range read (no panic);
      ZStack-path range mutation updates child-carried placement and
      Visual order in one splice.
- [x] **Cap fixtures (DD-M3-P7-007):** a representative tail-append at
      gallery scale and a deliberately larger N (e.g. 64 >
      `MUTATION_CAP`) converge without divergence — breadth consumes
      no cap depth; the fixture states which setup path it uses.
      Record the reactive-drain items 1–3 carry disposition (verbatim
      rows from DD-M3-P7-007) in [log.md](./log.md).

### T8 — Gallery thumbnail slice + assistant-side build / launch

Discharges the assistant-automated portion of ADR evidence items (5)
and (6) and the gallery positive-control portion of item (1). The
owner-visible portion of item (6) is T9's per the split. Assistant
evidence is **launch + DPI-aware screenshot capture + assistant
analysis**; `Start-Process` survival is a supporting signal only.

- [ ] **Structured-item trigger decision with the owner (T1 addendum 3
      G-2 / T1 addendum 4) — first T8 subtask, before authoring the
      `.ui`.** The current thumbnails vary **two** per-item attributes
      (distinct `fill` colour + label `S0N`) — i.e. **record-like
      per-item data** ({image/colour, id}). A scalar-item `for`
      (`i32[]` / `string[]` / `bool[]`) binds **one** value per item, so
      this is **the first concrete surfacing of the DD-M3-P7-002
      structured-item / `TypedValue` deferral trigger** ("a concrete app
      case where scalar items cannot express the data") — which
      DD-M3-P7-002 says **cannot be smuggled**: silently picking one
      attribute would consume the trigger without the named
      acceptance-revision path. So this is **not** a casual demo-look
      choice; surface it to the owner as the trigger firing, with the
      recommendation and record the decision in [log.md](./log.md)
      before authoring:
      - **Recommended — reduce to a single varying attribute for Phase 7**
        (the label/id from the collection, static `fill`; simplest, and
        keeps the append/remove prefix-undisturbed positive control
        legible). The trigger routes to **M4/M5** (reopening structured
        items now is against FD-C thesis-sequencing and would revise M3
        acceptance — DD-M3-P7-002); record the trigger observation in
        the **T10 handoff**.
      - Alternative the owner may pick: bind a per-item colour with a
        static label; or treat the gallery as the trigger to reopen
        structured items (scope expansion — explicitly against the
        recommendation).
      Lightweight options-plus-recommendation check, not a DD/ADR; the
      owner-confirm gate applies because it is owner-visible demo
      composition **and** a recorded-deferral-trigger event, not a
      delegated implementation detail.
- [ ] Grow `examples/gallery/gallery.ui` **additively** per the owner's
      composition decision above: the thumbnail set inside the existing
      `ScrollView { WrapPanel { … } }` becomes `for`-generated from a
      collection `state` (Box + Text placeholders per the §4.9
      image-placeholder pattern), with `Add` / `Remove` **text Buttons
      outside the `for` body** driving the tail-append / tail-remove
      assignments. Existing gallery slices stay byte-identical except
      where the slice composition requires otherwise (record the
      decided deviation).
- [ ] Build and run `examples/gallery-rust/`. Record assistant
      evidence as **2+ frames**: initial N → after `Add` (N+1
      thumbnails, prefix visually undisturbed) → after `Remove` —
      the item count visibly tracks the mutation driven by the
      body-external Buttons (the FD-B positive control; a single
      static frame is not evidence). DPI blur noted as the known M4
      residual, not a Phase 7 failure. Screenshots under
      [evidence/](./evidence/).

### T9 — Owner-manual GUI smoke

Discharges the owner-visible smoke for ADR evidence item (6); a
separate gate from T8's assistant baseline.

- [ ] Owner runs `examples/gallery-rust/` and observes, with the
      **positive control = the collection mutated**: `Add` appends a
      thumbnail (existing thumbnails undisturbed), `Remove` removes
      the last one, the empty case is well-behaved, and WrapPanel
      reflow / ScrollView behaviour stay correct around the generated
      set.
- [ ] Owner explicitly accepts the smoke result, or records a fail
      observation; fixes land additively on the T9 branch and the
      checklist re-runs to green before T9 closes.
- [ ] T9 step-end retrospective recorded at
      `process/milestone-3/phase-7/retrospectives/t9.md`.

### T10 — Phase-end gates and Moment 2 re-sync

Discharges the m3-plan phase-end criteria, the
[ADR Moment 2 commit set](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2),
and the **A12 spec-closure gate** (ADR verification closure item 7).
The step-end retrospective is **owned by T10**; the phase-end
retrospective, CI run id, handoff finalization, and the
implementation-preamble status flip are **NOT owned by T10** (see the
preamble split). Before closing, cross-check this T0-frozen task list
against mid-phase owner decisions and revise where they diverge.

- [ ] `cargo fmt --all -- --check` green locally (returns to T10
      ownership if T10 changed production Rust).
- [ ] `cargo build --release --workspace` and `cargo test --workspace`
      green locally; CI green is phase-end-owned.
- [ ] `docs/dsl_spec.md` §4.15 marker flips to `M3-Phase 7 closed;
      implementation-synced`; document Status header updated;
      divergence corrections folded (including the design-draft token
      spellings in §8.4 / §8.5 / §8.9 pinned to the landed shapes);
      the Moment-1-front-loaded §4.14 / §9 revisions **re-verified
      against the implementation** like every other synced statement.
- [ ] **A12 spec-closure gate (evidence item 7):** the iteration
      chapter at the external-reader bar — grammar, collection types /
      literals / assignment forms, binder scope rules, the positional
      un-keyed identity baseline stated normatively, runtime mutation
      timing, validation / invalid examples matching the shipped
      diagnostics.
- [ ] `docs/architecture.md` §6.7.10 / §6.8.5 / §9 re-synced to the
      landed shape; top Status flips to `M3-Phase 7 closed
      (implementation-synced)`.
- [ ] `docs/notes/architectural-family.md` — the FD-Q trigger-1/-3
      confirm entry lands (revise-in-place).
- [ ] `docs/abi_spec.md` re-confirmed untouched; any forced ABI
      surface escalates with owner confirmation.
- [ ] `process/milestone-3/plan.md` Phase 7 row Status flips to
      `complete`.
- [ ] ADR set touched **only** if a retrospectives.md §phase-sync
      ADR-touch case applies; otherwise it stays at its Moment 1
      Accepted state.
- [ ] [log.md](./log.md) records the phase-close evidence pointers and
      implementation summary distilled from T1–T9.
- [ ] Carry-forward inputs to the Phase 8 pre-doc recorded under
      [handoff.md](./handoff.md) (at minimum: the DD-M3-P7-007 carry
      rows + re-triggers; the keyed-identity / per-item-handler /
      per-item-condition / nested-`for` / member-range /
      loop-external-read deferrals with their framing-正本 triggers;
      the Grid placement-migration trigger; the host-state-boundary
      future-compat record; **the structured-item / `TypedValue`
      trigger observation the gallery surfaced at T8 (G-2) — the first
      concrete app case where scalar items cannot express the per-item
      data, routed to M4/M5 per DD-M3-P7-002**). **NOT owned by T10**;
      stays `[ ]` at T10 close.
- [ ] Front-matter `status` on [preamble.md](./preamble.md) flips
      `active` → `closing` at the **phase-end batch commit**, not at
      T10 step-close. **NOT owned by T10**; stays `[ ]` at T10 close.
- [ ] **T10 step-end retrospective recorded** at
      `process/milestone-3/phase-7/retrospectives/t10.md` (items 1–11;
      **owned by T10**).
- [ ] **Phase-end retrospective recorded** at
      `process/milestone-3/phase-7/retrospectives/phase-end.md` (items
      12–18; **NOT owned by T10**; separate retro commit on the phase
      branch after T10 merges in). Step retro `phase-sync` items from
      T1–T10 close into `doc-folded` / `carry-forward` / `local-only`
      at the phase-end retro — no open `phase-sync` items survive past
      phase close.
