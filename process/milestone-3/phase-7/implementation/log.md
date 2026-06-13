## Decisions log

- **2026-06-13 / T1 addendum 5 — compile-experiment: the trap-#1
  surface, compiler-verified (premise test).** To test the premise that
  the T2 migration surface is "enumerable by grep/reasoning" (my F-3),
  a throwaway `ControlFlowNode::For { binder, index_binder, collection,
  body }` variant was added to `wasamo-ir/lib.rs`, `cargo build
  --workspace` run, the compiler-forced breakage captured, then the
  variant **reverted** (no production code lands — a spike experiment,
  not a change). The premise **partly failed**, in three instructive
  ways:

  - **FE-1 — empirical ≠ grep.** The compiler forced exactly **9
    production sites**: `wasamoc/src/emit.rs:81`, and
    `wasamo-runtime/src/ir_loader.rs` 334 / 365 / 480 / 520 / 574 / 670 /
    971 / 1913. **`wasamoc/emit.rs:81` was *not* in my F-3 grep list**
    (I over-focused on `ir_loader.rs`) — a false negative. Conversely,
    several F-3-listed lines are *not* compiler-forced (below) — false
    positives. So grep over-and-under-counted; the compiler is the
    ground truth.
  - **FE-2 — `cargo build` does not compile `#[cfg(test)]`.** The
    test-module `ControlFlowNode::If` matches (e.g. `ir_loader.rs` ~2254,
    ~3179, the `materialized_index` tests ~2448–2517) did **not** break
    the build — they only break under `cargo test`. So the T2 trap-#1
    audit must run **both** `cargo build` *and* `cargo test` to surface
    the full match set; "release build green" is necessary-not-sufficient
    (the gate's core principle, here concrete).
  - **FE-3 — the dangerous sites are compiler-*invisible*.** The
    `_`-wildcard `ControlFlow` arms silently absorb `For` and the
    compiler says **nothing** — they were absent from the error list.
    Grep found **5**: `wasamo-ir/lib.rs:186` (`widget_children`,
    `IrMember::ControlFlow(_) => None` — the known Phase-6 hotspot,
    *confirmed compiler-silent*), and `ir_loader.rs` 352 / 788
    (`IrMember::ControlFlow(_) => { … }` handler arms — need
    classification) and 459 / 837 (`matches!(m,
    IrMember::ControlFlow(_))` boolean "is control-flow?" tests —
    likely correct under `For`, since `For` *is* control-flow, but must
    be confirmed). **This is the proof that compile-error-forcing does
    not protect the trap-#1 hotspots** — the audit cannot rely on "the
    compiler will enumerate the surface"; it must grep `ControlFlow(_)`
    separately. Validates DD-004 / preamble's `widget_children` emphasis
    with ground truth.

  **Compiler-verified trap-#1 map for `ControlFlowNode::For` (hand to
  T2):**
  | Class | Sites | Audit note |
  |---|---|---|
  | Compiler-forced (production) | `emit.rs:81`; `ir_loader.rs` 334/365/480/520/574/670/971/1913 | each gets a real `For` arm or a deliberate reject |
  | Compiler-silent wildcard (`ControlFlow(_)`) | `lib.rs:186` `widget_children`; `ir_loader.rs` 352/788 (arms), 459/837 (`matches!`) | **the dangerous half** — classify each *correct-filter* vs *bug-under-For*; grep-found, not compiler-found |
  | Parser string-dispatch (additive) | `ir_loader.rs` 1460/1544 (`Token::Ident == "if"`) | add a new `"for"` arm (additive, not a break) |
  | Test-only (need `cargo test`) | `ir_loader.rs` ~2254/~3179; `materialized_index` tests ~2448–2517 | FE-2: surfaced only under `cargo test` |

  **Scope honesty:** the experiment exercised the `ControlFlowNode`
  surface only (one variant). The `IrStateType` (R-A) and `HandlerExpr`
  collection/loop-local/assignment surfaces would each need their own
  scratch variant to be compiler-verified the same way; they remain
  **grep-predicted** until a T2-start experiment (recommended: repeat
  this technique per added variant before writing the audit table —
  cheaper and more accurate than grep, and it is *the* way to find the
  emit.rs:81-class false negatives).

  **Premise / framing conclusions (widening the frame):**
  - **"T1 lands no production code" did not bar the most rigorous
    check.** A reverted scratch experiment is a spike's proper tool and
    out-rigored every grep pass. The plan's no-code framing was read too
    literally as "no compiler runs"; the corrective is "no code *lands*,"
    not "no code *runs*."
  - **"Done = no surprises" is the wrong exit test** — it failed 4×. The
    correct T1 exit is **"every remaining unknown is owned by a named
    task and bounded"** (the coverage table + this map + the CF rows
    achieve that). Restating the exit criterion is the durable fix; the
    perpetual addenda were the symptom of using the wrong one.
  - **Diminishing returns / when to stop.** Past this experiment, further
    T1 deepening (e.g. scratch-verifying the `IrStateType` / `HandlerExpr`
    surfaces) is better done *at T2 start*, where it is that task's own
    audit, run once against the real edit — doing it now would re-incur
    the cost and is the same artifact-compulsion in a new guise. **T1's
    cross-task / plan-structure risk surface is now covered; T1 should
    close.** This stopping judgment is itself part of the discipline.

- **2026-06-13 / T1 addendum 4 — owner-agreement surface review across
  T2–T10.** Critical sweep of every task for decisions needing owner
  agreement (not delegated to the Accepted DDs). Conclusion: **one
  mandatory new consult (T8, re-framed below); no others** — the DD
  slate is owner-settled and the real owner gates already exist;
  manufacturing more consults would be the over-consultation the
  owner-confirm gate warns against. Three *conditional* escalation
  triggers are named (armed before, not after).

  - **Re-framed mandatory consult (T8): G-2 is the structured-item /
    `TypedValue` deferral trigger, not a demo-look choice.** The gallery
    thumbnail's per-item {colour, label} is **record-like data**;
    DD-M3-P7-002 §`TypedValue` pressure names exactly this ("a concrete
    app case where scalar items cannot express the data") as a
    trigger-backed defer that **cannot be smuggled**. Silently picking
    one attribute would consume the trigger without its named
    acceptance-revision path. So the T8 first subtask is upgraded: the
    owner is told the trigger fired, the recommendation is
    **reduce-to-single-attribute for Phase 7** (the trigger routes to
    M4/M5; reopening structured items is against FD-C thesis-sequencing
    and revises M3 acceptance), and the trigger observation lands in the
    **T10 handoff**. Plan T8 + T10 handoff updated. My earlier G-2 plan
    note treated this as composition aesthetics — under-framed; this is
    the correction.
  - **No other mandatory consult.** Confirmed owner-settled by Accepted
    DDs (no consult, implement + record in log): T2 schema shapes
    (DD-002/004, spellings adjustable), T3 author surface
    (`append`/`drop-last` + `in` owner-confirmed at the Accepted flip),
    T5 placement value-space (DD-006 "open for implementation"), T7
    PF2/cap (DD-005/007). Confirmed already-present owner gates (no new
    subtask needed): **T9** owner-manual smoke (an owner gate by
    construction), **T10** phase-end merge approval + the A12 spec
    review-before-commit + the conditional ABI-escalation bullet, and
    the Moment-1 spec drafts already owner-reviewed.
  - **Conditional escalation triggers (fire only if a delegated detail
    turns load-bearing — named here to arm the gate before the design
    decision, not at completion):**
    1. **T4 / T5 / T7 — observable change to the shipped Phase 6
       conditional behaviour.** `if` is a shipped, owner-smoked feature
       that the C1 migration (T4), the placement migration (T5), and the
       splice-seam routing (T7) all touch. Intent is zero observable
       change (regression fixtures are the guard); **if a regression
       fixture must *change* (real behaviour change), escalate**
       (owner-confirm criterion a/b — it alters shipped/accepted
       behaviour). Conditional, not a mandatory subtask.
    2. **T3 — a novel diagnostic-*philosophy* question** beyond
       DD-007's "name the deferral" (criterion b / author-visible) ⇒
       surface. Plain wording stays owner-reviewed at the T10 A12 gate
       (existing), not a T3 consult.
    3. **T7 — PF2 fault-injection infeasibility.** Already DD-005-
       mandated to be *recorded in this log, not silently skipped*;
       surfaced to the owner at the T7 retro (a disposition record, not
       a mid-task consult).
  - **Discipline recorded:** the bar for a *mandatory* owner-consult
    subtask is (product/author-visible **or** AC/phase/cross-task-
    constraint) **and** not settled by a DD **and** not covered by an
    existing gate (T9 smoke / T10 merge / review-before-commit). G-2
    meets all three; the conditional triggers meet only the first, so
    naming them as triggers — not subtasks — is the correct weight
    (over-consulting delegated detail is itself a gate violation).

- **2026-06-13 / T1 addendum 3 — applying the spike's own corrective to
  T1 (load-bearing verification + coverage table + break-first).** The
  retrospective's "目標・前提・計画仮説の再点検" prescribes two artifacts
  before declaring a spike done — (a) an evidence-set coverage table and
  (b) a "where does the plan-hypothesis break first" statement — and
  flags the failure mode of *proposing* a forcing artifact while
  exempting the current task. This entry applies (a)/(b) to T1 itself
  and verifies the single load-bearing assumption of §1.

  - **G-1 (load-bearing design correction — verified in code): the
    binding-eval context is constructed *internally*, so per-item reads
    need new registration entry points, not a reused one.** §1 assumed
    loop-local reads resolve "through a per-item `EvalContext`." But
    `register_binding_with_writer`
    ([reactive.rs:626](../../../../wasamo-runtime/src/reactive.rs#L626))
    and its bool sibling
    ([reactive.rs:666](../../../../wasamo-runtime/src/reactive.rs#L666))
    **construct `BindingEvalContext::new(&registry)` inside the effect
    closure** — the context is not a caller-supplied parameter, and the
    closure writes unconditionally (`Ok(value) => writer(value)`). So
    the instantiation context cannot be *injected* into the existing
    path. T6/T7 must add **new registration entry points** (one per
    element type — i32 / string / bool, mirroring the existing
    string/bool writer seams) whose closure (i) builds a
    `ForItemEvalContext { registry, collection, elem, position }` instead
    of `BindingEvalContext`, and (ii) is **guarded**:
    `Some(v) => writer(v)`, `None => skip` (the out-of-range "write
    nothing" — expressible only in a *new* closure, since the existing
    one always writes). This sharpens §1 and CF-4: the work is "a new
    per-item binding registration API ×3 element types with a guarded
    closure," materially more than §1's "extend the trait + impl."
    Verified directly, because §1 stood entirely on this assumption —
    the exact "load-bearing assumption left unverified" the reflection
    warns about. (Good news: the design holds — the seam is real and the
    guard is expressible; only the implementation shape was understated.)
    Plan T6/T7 updated.
  - **G-2 (T8 composition fact): the current gallery thumbnail varies
    two attributes; a scalar-item `for` binds one.** `gallery.ui`
    already has the `ScrollView { offset-y: scroll_y; WrapPanel { … } }`
    shape with `Box { aspect: 1:1; fill: #RRGGBBAA; Text { text: "S0N" } }`
    children
    ([gallery.ui:133](../../../../examples/gallery/gallery.ui#L133)) — so
    T8's additive `for`-growth into that WrapPanel is feasible
    (DD-007's assumed gallery shape is real). **But** each current
    thumbnail varies **two** per-item attributes — a distinct `fill`
    colour *and* a distinct label (`S01`..) — whereas a scalar
    collection item (`i32[]` / `string[]` / `bool[]`, single bound value
    per item, DD-002) can drive **one**. T1's remit is to **surface this
    constraint, not to pick the demo composition** (deciding the
    gallery's look here would be T1 overreaching into owner-visible demo
    aesthetics — the exact "freeze a complete-looking decision" failure
    the retrospective diagnoses). So the *resolution* (which single
    attribute the item drives — label/id with static fill, or a per-item
    colour with static label, or another) is **deferred to a first T8
    subtask surfaced to the owner** with an options-plus-recommendation
    (default: label/id + static fill), because the gallery is the A8
    positive-control vehicle the owner smokes at T9. Recorded now so the
    constraint is a T8 input, not a T8-time surprise; the choice is the
    owner's at T8 start. Plan T8 updated (the prior draft's prescriptive
    "reduce to label + static fill" is withdrawn as a T1 over-decision).

  **T1 evidence-set coverage table (corrective (a), applied to T1).**
  "Primary landing file" = where each downstream task's main change
  lands; status as of T1 close.

  | Task | Primary landing file(s) | T1 status |
  |---|---|---|
  | T2 IR schema | `wasamo-ir/lib.rs` ✓; `reactive.rs` registry/`Signal` ✓; `ir_loader.rs` `If`-match cluster ✓ (enumerated, addendum 2 F-3); `wasamoc/lower.rs` + `emit.rs` construction sites **✗ deferred** | read except lower/emit |
  | T3 wasamoc surface | `lexer.rs` ✓ (F-5a); `check.rs` namespace/threading ✓ (F-4); `parser.rs` **✗ deferred**; `ast.rs` **✗ deferred**; `lower.rs`/`emit.rs` **✗ deferred** | partial |
  | T4 C1 seam | `ir_loader.rs` `materialized_index*` / `DeclaredMemberSlot` ✓ | read |
  | T5 ST2 placement | `widget.rs` ZStack/insert/remove ✓; `layout.rs` zstack arrange **~ grep-only**; `ir_loader.rs` placement extraction **~ grep-only** | core read; arrange body skimmed |
  | T6 loader static | `ir_loader.rs` build + textual-IR parse ✓; binding-registration path ✓ (G-1) | read |
  | T7 splice + effect | `reactive.rs` effect/binding ✓; `widget.rs` destroy ✓; `handler.rs` eval contexts ✓; registration path ✓ (G-1) | read |
  | T8 gallery | `examples/gallery/gallery.ui` ✓ (G-2) | read |
  | T9 / T10 | docs/spec sync — N/A at T1 | n/a |

  **Deferral judgments (why ✗/~ is a judgment, not a gap):**
  - `lower.rs` / `emit.rs` (T2/T3): the exhaustive `IrState` /
    `HandlerExpr` construction-site enumeration **is the T2/T3 trap-#1
    close artifact** — reading them now to enumerate would duplicate that
    task's own audit. Boundary defensible *because* addendum 2 already
    pinned the highest-density cluster (`ir_loader.rs` `If`-matches).
  - `parser.rs` (T3): the plan's "LL(1) after the first `IDENT`" claim is
    sound **by construction** — `for` becomes a reserved keyword token
    (lexer `Keyword` enum, F-5a), so the member parser dispatches on it
    with no backtracking, exactly like the existing control-flow member.
    Reading 1279 lines to re-confirm a keyword-led dispatch is not
    warranted at T1; residual risk noted, owned by T3.
  - `layout.rs` zstack arrange (T5): read via grep to confirm it reads
    `zstack_placements` parallel-vector (the migration target); the full
    arrange body is T5's own regression surface (Phase 6 ZStack fixtures).

  **Break-first statement (corrective (b)).** If the T1 design/order is
  wrong, the first break was the **per-item binding registration path**
  (G-1) — now *verified* to support new guarded entry points, so that
  risk is closed. The top *residual* therefore shifts to the
  **deferred-unread** set: T3's `for`-header parse (mitigated:
  keyword-led ⇒ LL(1) by construction) and the T2/T3 exhaustive
  evaluator + `If`-match migration (mitigated: cluster pinned, audit is
  those tasks' close artifact). No residual rises to a T1-blocking
  unknown; each is owned by a named task with a recorded mitigation.

- **2026-06-13 / T1 addendum 2 — deeper code pass (wasamoc, disposal,
  loader parser).** The first spike read only the runtime *structural*
  side; a second critical pass read the previously-unread areas each
  later task actually lands in — `wasamoc` (`lexer.rs` / `check.rs`),
  the disposal/teardown path
  ([`widget.rs`](../../../../wasamo-runtime/src/widget.rs)
  `widget_destroy`), the textual-IR **parser** half of `ir_loader.rs`,
  and the two `EvalContext` impls. Five findings; F-1/F-3/F-4 are
  scope-relevant to T2/T3/T7 (no reorder), F-5 is confirmation.

  - **F-1 (T7 hard constraint): per-item binding effects must be owned
    by the generated *child* subtree, not the parent.** Teardown is
    `widget_destroy`
    ([widget.rs:1786](../../../../wasamo-runtime/src/widget.rs#L1786))
    → `dispose_subtree_bindings`
    ([widget.rs:1792](../../../../wasamo-runtime/src/widget.rs#L1792)),
    which clears `WidgetNode.bindings`
    ([widget.rs:327](../../../../wasamo-runtime/src/widget.rs#L327))
    recursively over the subtree, then severs the registry, then drops.
    The Phase 6 conditional stores its effect on the **parent**
    (`parent.bindings.push(handle)`,
    [ir_loader.rs:1969](../../../../wasamo-runtime/src/ir_loader.rs#L1969)).
    That pattern is correct for the `ForLoopSubtree` **structural**
    effect (it outlives individual items, like the conditional) but
    **wrong for the per-item value/index effects**: on a tail-removal
    T7 calls `widget_destroy(removed_child)`, which disposes only the
    *child subtree's* bindings — so a per-item effect parked on the
    parent would **leak** (and keep reading a freed position). So the
    ownership rule is split: `ForLoopSubtree` effect → parent.bindings;
    per-item value/index effects → the generated child subtree root's
    bindings. My §1 record (and the conditional analogy it leaned on)
    did not pin this; it is a correctness constraint, not a style note.
  - **F-2 (T7 trap-#2 sharpening): two of the six side-effects already
    exist as infrastructure.** `widget_destroy` already performs
    DD-006 side-effect #5 (effects disposed ahead of teardown) and #4
    (registry release) in the order bindings → registry → drop,
    recursively. So T7's removal path **reuses** `widget_destroy` per
    removed subtree (tail-first) rather than rebuilding #4/#5; the
    splice seam's *new* work is the children-vector splice (#1), Visual
    sibling order (#2), layout invalidation (#3), and staged-insert
    effect attach (#5 insert side). The trap-#2 close artifact should
    mark #4/#5-removal as **reused**, not re-implemented — my §2/§3
    treated all six as freshly enumerated.
  - **F-3 (T2 audit scope, materially larger): `ir_loader.rs` has ~12
    non-test `ControlFlowNode::If`-only match sites** that go
    non-exhaustive the moment T2 adds `For`: lines 336, 367, 482, 522,
    576, 672, 973, 1460 (parse dispatch), 1544 (nested-`if` in
    `parse_if_member`), 1931 (`append_static_member`), 2254, plus the
    emit site 3179. My §3 R-A list named only `append_static_member` +
    `materialized_index_for_declared_member`. **This is exactly the
    Phase-6 `widget_children` failure mode multiplied** — each of these
    must be classified (real `For` arm vs deliberate reject) in the T2
    trap-#1 audit table; several are in validation / counting /
    placement-collection helpers where a silently-missing `For` arm
    would mis-validate or mis-count. The §3 R-A site list is extended
    accordingly (below).
  - **F-4 (T3 scope, wider than "reject rows"): binder scope is new
    `check` machinery.** `check.rs` carries a **flat, state-only**
    `Namespace` (name→type,
    [check.rs:55](../../../../wasamoc/src/check.rs#L55)) built by
    `collect_state_namespace` and threaded immutably through
    `check_members_inner`
    ([check.rs:1335](../../../../wasamoc/src/check.rs#L1335) — which
    already gained a `parent_widget` param in Phase 6). DD-003's binder
    scope (binders added entering a `for` body, removed leaving it;
    binder-vs-state collision; index-vs-value collision; reads only
    inside the body) is a **new scoped dimension threaded alongside
    `ns`**, the direct analogue of the Phase-6 `parent_widget`
    threading — not just additional reject arms. The qualified-name
    resolver (`check.rs:1686`) already covers the DD-001/002
    qualified-reference rejects. T3 is "parser + binder-scope threading
    + reject rows," and the scope threading is the load-bearing part.
  - **F-5 (confirmations).** (a) The `.ui` lexer already lexes
    **kebab-case identifiers as single tokens** (`scan_ident`
    [lexer.rs:347](../../../../wasamoc/src/lexer.rs#L347), continuing on
    `-` + alpha, [lexer.rs:366](../../../../wasamoc/src/lexer.rs#L366)),
    with `in-out` pinned by an existing test
    (`in_outx_lexes_as_kebab_ident`) and a `Keyword` enum + reserved
    control-flow family already present — so T3's `in` reservation is
    mechanical and DD-002's contextual `append` / `drop-last` (and a
    *state* named `drop-last`) are lexically sound as single tokens.
    (b) The textual-IR parser is hand-rolled recursive descent:
    `parse_for_member` mirrors `parse_if_member`
    ([ir_loader.rs:1530](../../../../wasamo-runtime/src/ir_loader.rs#L1530),
    dispatched at the member loop's `Token::Ident == "if"` site
    [ir_loader.rs:1460](../../../../wasamo-runtime/src/ir_loader.rs#L1460)),
    and the collection atoms (`list-prop-read` / `list-append` /
    `list-drop-last` / `list`) slot into `parse_expr`'s atom-head
    dispatch ([ir_loader.rs:1645](../../../../wasamo-runtime/src/ir_loader.rs#L1645),
    beside `str-prop-read` / `bool-prop-read` at 1680/1684) — T2/T6
    additions are mechanical. (c) The instantiation-context impl is
    concrete: `BindingEvalContext<'a> { registry }`
    ([reactive.rs:416](../../../../wasamo-runtime/src/reactive.rs#L416))
    is a thin registry wrapper implementing `EvalContext`, so
    `ForItemContext` is the same wrapper plus `{ collection, elem,
    position }`; CF-6 extends `HandlerEvalContext`
    ([reactive.rs:493](../../../../wasamo-runtime/src/reactive.rs#L493)).

  **§3 R-A site list extended (F-3):** add the ~12 `ir_loader.rs`
  `ControlFlowNode::If` match sites above and the `wasamoc` lower/emit
  `IrState` / `HandlerExpr` construction sites
  ([`lower.rs`](../../../../wasamoc/src/lower.rs),
  [`emit.rs`](../../../../wasamoc/src/emit.rs)) — still to be read
  exhaustively at T2 start, where the trap-#1 audit table is the close
  artifact — to the previously-named lib.rs / reactive.rs / registry
  sites. The audit is **T2's** close artifact; T1's contribution is
  pinning that the `If`-match-site sweep over `ir_loader.rs` is the
  highest-density trap-#1 cluster.

  **Method learning (for the gate template / memory):** the original
  spike declared completion having read only the *runtime structural*
  files; the two corrections (addendum 1 evaluator seam, this pass's
  F-1/F-3/F-4) both came from the files a spike that audits "where does
  each later task land" would have read first. A pre-implementation
  spike's source set must cover **every task's primary landing file**,
  not just the phase's headline structural refactor.

- **2026-06-13 / T1 addendum — critical re-examination of the spike
  (evaluator seam + ownership gap).** A critical second pass over
  [`handler.rs`](../../../../wasamo-runtime/src/handler.rs) (the
  `EvalContext` trait + the five `evaluate_*` / `invoke_handler` match
  sites) and `Signal::set`
  ([reactive.rs:230](../../../../wasamo-runtime/src/reactive.rs#L230))
  found that the original spike entry below **overstated "no plan
  change"**. Four corrections, one of which **requires a plan revision**:

  1. **The instantiation-context seam is the `EvalContext` trait, not a
     bespoke closure (corrects §1).** Runtime expression evaluation goes
     through `trait EvalContext`
     ([handler.rs:12](../../../../wasamo-runtime/src/handler.rs#L12) —
     `get_i32` / `set_i32` / `read_i32_tracked` / `read_string_tracked` /
     `get_bool` / `read_bool_tracked` / `set_bool`, default-impl methods
     for additive back-compat) implemented by `BindingEvalContext`
     (reactive reads) and `HandlerEvalContext` (live writes). The
     faithful shape of `ForItemContext` is therefore **an `EvalContext`
     implementation that carries `position`**, resolving the loop-local
     reads via **new tracked trait methods** (e.g. `read_item_i32` /
     `read_item_string` / `read_item_bool`) that read
     `collection_signal.get()[position]` with the out-of-range guard
     returning the "write nothing" path — *not* a closure capturing a
     plain struct as §1 phrased it. The §1 datum (`collection`, `elem`,
     `position`) is unchanged; what changes is that the read resolves
     through the trait, so T6/T7's surface is "extend the `EvalContext`
     trait + its binding-context impl," wider than §1's "add a guarded
     read." DD spellings stay adjustable (no DD reopen).
  2. **Plan-ownership gap (requires revision): the handler-side
     collection-assignment evaluation is unowned.** The authored
     `thumbs = thumbs.append(x)` runs **inside a handler**
     (`invoke_handler` / `evaluate` → `HandlerEvalContext` →
     `Signal::set`). It is a whole-`Vec` read-modify-write needing a new
     `HandlerExpr` arm in the handler evaluator **and** a new
     `EvalContext` collection-write method. This is the *writer* that
     drives the signal T7's `for` effect (the *reader*) reacts to — so
     T7's mutation fixtures depend on it — yet the plan's T7 bullets name
     only the splice seam, `ForLoopSubtree` + effect, per-item bindings,
     Windows fixtures, and cap fixtures; **none names the handler-side
     assignment evaluation**, and T6 (loader static path) does not cover
     it. **Resolution: T7 gains an explicit bullet** for the handler-side
     collection-assignment evaluation (read-modify-write on the
     whole-value signal via an extended `HandlerEvalContext`; the
     equal-value no-dirty rule). Plan revised in this commit batch
     (retrospectives.md §11 ownership-on-Task-list mandate). New
     carry row **CF-6**.
  3. **§3 R-A omitted the handler.rs evaluator match sites (corrects
     §3).** When T2 widens `HandlerExpr`, the **five exhaustive matches**
     in `handler.rs` — `evaluate`
     ([handler.rs:95](../../../../wasamo-runtime/src/handler.rs#L95)),
     `evaluate_tracked`
     ([handler.rs:344](../../../../wasamo-runtime/src/handler.rs#L344)),
     `evaluate_binding`
     ([handler.rs:277](../../../../wasamo-runtime/src/handler.rs#L277)),
     `evaluate_binding_part`
     ([handler.rs:301](../../../../wasamo-runtime/src/handler.rs#L301)),
     `evaluate_bool_binding`
     ([handler.rs:328](../../../../wasamo-runtime/src/handler.rs#L328)),
     plus `invoke_handler`
     ([handler.rs:235](../../../../wasamo-runtime/src/handler.rs#L235)) —
     all break the build and are **among the most-affected trap-#1
     sites**. The T2 call-site audit table **must enumerate them**; §3's
     R-A site list (which named lower/emit/check/ir_loader/registry/
     `BindingTarget` only) is extended here.
  4. **`Signal::set` has no equal-value short-circuit today (sharpens
     CF-5).** `set`
     ([reactive.rs:230](../../../../wasamo-runtime/src/reactive.rs#L230))
     assigns unconditionally and always marks dependents dirty — so
     DD-002's "equal-value writes mark nothing dirty" is a **new**
     `PartialEq`-gated behaviour T2 adds for the collection signals, not
     a configuration of existing behaviour (CF-5 said "ship with," which
     understated it). This is exactly the DD-005 §Technical-risk "if
     absent, a bounded note lands in the plan" branch — confirmed
     **absent**. `Signal<T>` is `T: Clone + 'static`
     ([reactive.rs:213](../../../../wasamo-runtime/src/reactive.rs#L213)),
     so `Signal<Vec<_>>` is fine and `Vec: PartialEq` satisfies the gate
     (no blocker — recorded as the one *confirmed-clear* item).
  5. **Self-inflicted (corrects §2 Seam C):** §2 said
     `BindingTarget::ForLoopSubtree` "lands in T2's `BindingTarget`
     migration." Wrong mechanism — `register_binding`
     ([reactive.rs:607](../../../../wasamo-runtime/src/reactive.rs#L607))
     and `register_conditional_binding` destructure with **let-else, not
     exhaustive match**, so adding a `BindingTarget` variant does not
     force T2 changes; the plan's placement of `ForLoopSubtree` in **T7**
     (where first used) is correct and avoids a dead variant. Seam C is
     corrected inline below.

  **Net:** order T2→T7 unchanged; **one plan revision** (T7 gains the
  handler-side assignment-evaluation bullet, CF-6); design record
  corrected on the evaluator seam (1/3/5) and the equal-value rule (4).
  The original "no plan change" claim is withdrawn.

- **2026-06-13 / T1 — Pre-implementation spike: instantiation context,
  bisectable sequencing, risk sharpening, and the T2 gate selection.**
  T1 lands **no production code** (per
  [plan.md §T1](./plan.md)); its deliverables are the recorded design
  decisions below plus the plan revisions they imply. Task branch
  `feat/m3-phase-7-t1`. The three plan bullets are discharged in
  §1 (instantiation context), §2 (sequencing), §3 (risk sharpening +
  T2 gate selection). Sources read against current `HEAD` (`e97361c`):
  [`wasamo-ir/src/lib.rs`](../../../../wasamo-ir/src/lib.rs),
  [`wasamo-runtime/src/reactive.rs`](../../../../wasamo-runtime/src/reactive.rs),
  [`wasamo-runtime/src/ir_loader.rs`](../../../../wasamo-runtime/src/ir_loader.rs),
  [`wasamo-runtime/src/widget.rs`](../../../../wasamo-runtime/src/widget.rs),
  [`wasamo-runtime/src/layout.rs`](../../../../wasamo-runtime/src/layout.rs),
  [`wasamo-runtime/src/registry.rs`](../../../../wasamo-runtime/src/registry.rs).

  ### T1 start gate (implementation-gates selection for T1 itself)

  T1 produces no schema change, no branch, no tree mutation, and no GUI
  deliverable, so the structural traps do not apply to T1's own work;
  the one trap that does apply is the one T1 *is*:

  - **#1 semantic migration** — *not applicable to T1*: T1 adds no
    enum/schema variant (it only *designs* the T2 ones). The audit
    obligation it produces is recorded as the T2 gate selection (§3).
  - **#2 side effects / #3 parallel data / #7 GUI** — *not applicable*:
    no runtime mutation, no parallel vector touched, no GUI render in
    T1.
  - **#4 untested branch** — *not applicable*: no code branch is added.
  - **#5 carry-forward** — **applies; T1's entire output is
    carry-forward.** The instantiation-context shape, the sequencing
    seams, and the T2 gate selection are recorded here (log.md) and in
    [plan.md](./plan.md) with re-trigger criteria, so the downstream
    tasks consume a written record, not memory.
  - **#6 root cause** — standing; nothing recurring to disposition at
    T1.
  - **Review lane:** T1 is a design spike with no executable change; the
    task-end gate is owner review of the recorded design (no full code
    review, because there is no code). The high-risk review lanes it
    *assigns* (T2 schema, T5/T7 runtime structural) are recorded in §3.

  ### 1. Instantiation context type (plan T1 bullet 1)

  **Problem.** A `for` body template is one IR subtree shared by all N
  generated positions (DD-M3-P7-004 S1: `ControlFlowNode::For.body` is a
  single `Widget` member). A per-item binding such as `label: thumb`
  must therefore resolve its **value** and **index** from data supplied
  *at materialisation*, not baked into the shared template. The shipped
  binding path has no such per-instance datum: `register_binding`
  ([reactive.rs:601](../../../../wasamo-runtime/src/reactive.rs#L601))
  evaluates a `HandlerExpr` against the whole `SignalRegistry` and always
  writes the evaluated `String`; reads are by state name
  (`HandlerExpr::PropRead { path }`,
  [lib.rs:48](../../../../wasamo-ir/src/lib.rs#L48)). There is no concept
  of "the current item at position *i*".

  **Existing shape this generalises.** The Phase 6 conditional path is
  the structural precedent. `DeclaredMemberSlot`
  ([ir_loader.rs:92](../../../../wasamo-runtime/src/ir_loader.rs#L92)) is
  `Widget | Conditional(Rc<RefCell<ConditionalRuntimeState>>)` with
  `ConditionalRuntimeState { live_child: bool }`
  ([ir_loader.rs:97](../../../../wasamo-runtime/src/ir_loader.rs#L97));
  `materialized_index_for_declared_member`
  ([ir_loader.rs:1975](../../../../wasamo-runtime/src/ir_loader.rs#L1975))
  is the prefix-sum (Widget→1, Conditional→0/1) that the C1 seam
  generalises (T4). The conditional subtree is built once by
  `build_node` and re-built fresh on toggle; iteration needs **per-item**
  state because each position carries its own live read.

  **Decision — recommended shape.** The instantiation context is a
  *runtime* construct (not an IR construct): the body template stays
  position-agnostic, and the loader (T6) / `for` effect (T7) supply the
  per-instance context when materialising each position.

  ```rust
  /// Supplied once per generated subtree, at materialisation time.
  /// Fixes the subtree's position in the collection; the value-binder's
  /// reads resolve `collection[position]` *live* on the whole-value
  /// signal (DD-005 V2), the index-binder resolves to `position`
  /// (constant under tail-only mutation). DD/field spellings adjustable.
  struct ForItemContext {
      /// Registry key of the iterated collection state (DD-002 R1
      /// whole-value signal). Fixed for the whole body — lives here, not
      /// duplicated into every loop-local read.
      collection: String,
      /// Element scalar type. Selects the collection signal map and the
      /// evaluator/writer pair (mirrors the existing per-type writer
      /// seam, architecture.md §6.7.x). Fixed per `for`.
      elem: IrType,
      /// This item's fixed position = the materialised offset within the
      /// `for` slot's range. Read live; out-of-range → write nothing.
      position: usize,
  }
  ```

  Two IR-side companions (their exact spelling is T2/DD-002's, recorded
  here only as the shape the context serves):

  - **Loop-local reads are bare markers** in the body template —
    `ItemValueRead` (the current element) and `ItemIndexRead` (the
    current position) `HandlerExpr` variants carrying **no** collection
    name and **no** concrete index. Everything fixed-per-`for`
    (`collection`, `elem`) and everything fixed-per-instance (`position`)
    comes from the `ForItemContext` the binding effect closes over.
    Rationale: under flat scope with no nesting (DD-003), a body is
    instantiated under exactly one `for`, so the collection/elem are
    invariant across the body; keeping them out of the read markers keeps
    the *shared template* free of per-`for` data and lets the loader
    validate "a loop-local read appears only inside a `for` body"
    structurally.
  - **Considered alternative — reads carry the tag**
    (`ItemValueRead { collection, elem }`, context carries only
    `position`). Closer to DD-005's literal "`collection[i]`" phrasing
    and makes the IR self-describing for loader cross-checks, but
    duplicates the `for` header's collection/elem into every body read
    and splits the per-instance datum (`position`) from the
    per-`for` data across two carriers. **Rejected on merit:** the
    context already exists to carry `position`, so folding the two
    invariants into it costs nothing and keeps the template minimal.
    (DD variant spellings stay adjustable — if T6/T7 finds the loader
    validation wants the tag on the read, it may move without reopening
    a DD.)

  **Live / out-of-range guard (DD-005 V2).** The value read is
  `registry.collection_signal(elem)[collection].get().get(position)` →
  `Option<scalar>`. `None` (position ≥ current length) ⇒ the binding
  **writes nothing**. This is a real extension to the binding-evaluation
  path: today `register_binding_with_writer`
  ([reactive.rs:620](../../../../wasamo-runtime/src/reactive.rs#L620))
  unconditionally writes the evaluated `String`. The loop-local path
  needs a *guarded* evaluation that can yield "skip" (e.g. evaluate to
  `Option<String>`, write only `Some`). Recorded as the binding-path
  shape T6 (static reads) and T7 (the same-batch doomed-binding read,
  DD-005 / DD-007 cap row) must implement and directly test; it is the
  positive control for the doomed-binding no-panic fixture.

  **Per-`for` runtime slot.** Mirroring `DeclaredMemberSlot::Conditional`,
  iteration adds `DeclaredMemberSlot::ForLoop { … }` carrying the slot's
  live cardinality / generated-subtree state (so the C1 seam can sum it
  and the splice can address its range). Whether it stores a count plus
  external per-item effects, or a `Vec` of per-item records, is a
  T4/T6/T7 implementation choice; T1 fixes only that the **stable
  identity is the declared slot** and the materialised range
  `[offset, offset+cardinality)` is recomputed via the seam, never cached
  (DD-004 / DD-005). The `ForItemContext.position` is an index into that
  range.

  **Why a runtime context, not an IR field (thesis check).** Cardinality
  is runtime data (FD-A); the position cannot be lowered into the shared
  template without static expansion, which the thesis rejects (DD-004
  S3). The context is the minimal per-instance carrier that keeps the
  template shared and the reads live — it is the iteration analogue of
  the conditional's `ConditionalRuntimeState`, widened from a `bool` to a
  position.

  ### 2. Bisectable sequencing (plan T1 bullet 2)

  **Decision: keep the plan's default order**
  T2 (I2 schema) → T3 (`wasamoc` surface) → T4 (C1 seam) →
  T5 (ST2 placement) → T6 (loader static path) →
  T7 (splice primitive + `for` effect). It is dependency-correct and
  bisectable; T1 does **not** reorder. What T1 adds is the explicit
  record of the **three inter-task seams** that keep each intermediate
  commit building and each task reviewable in isolation:

  - **Seam A — T2's loader `For` arm is a deferred-load reject.** T2 is
    the compile-error-forcing schema bundle (R-A): adding
    `ControlFlowNode::For` makes the `IrMember::ControlFlow` match in
    `append_static_member`
    ([ir_loader.rs:1931](../../../../wasamo-runtime/src/ir_loader.rs#L1931))
    non-exhaustive. T2 keeps the build green by adding a `For` arm that
    returns an `IrLoadError` ("`for` not yet materialised") — a *real,
    directly-tested* reject branch (trap #4) that T6 replaces with static
    materialisation. The three registry collection signal maps land in T2
    (`SignalRegistry`,
    [reactive.rs:391](../../../../wasamo-runtime/src/reactive.rs#L391))
    but are only *read* by T6+ and *written* by T7; T2 ships them with
    the value-equality-on-set rule (DD-002: equal-value writes mark
    nothing dirty) and its unit test.
  - **Seam B — T4 introduces `DeclaredMemberSlot::ForLoop` ahead of its
    first construction.** The plan's T4 unit suite must cover
    interleaved `if`/`for`/static siblings and tail insert/remove plan
    derivation (plan T4 bullet 3), which requires the seam to handle a
    `For` cardinality arm. So T4 lands the `ForLoop` slot variant + the
    seam's cardinality arm and unit-tests it, even though the loader does
    not *construct* a `ForLoop` slot until T6. The variant is therefore
    **dead (unconstructed) between T4 and T6** — a recorded, bounded
    `dead_code` allowance whose closure is T6, not a smell (it exists to
    make the pure-logic seam testable before the WinRT loader path
    lands). Carry-forward row below.
  - **Seam C — T6 registers the `ForLoopSubtree` effect with a no-op
    initial reconcile; T7 fills its tail-edit body.** T6 owns static
    materialisation (walk the seam at load) **and** registering the
    `BindingTarget::ForLoopSubtree`
    ([reactive.rs:585](../../../../wasamo-runtime/src/reactive.rs#L585) —
    the `ForLoopSubtree` variant is added in **T7**, where first used:
    `register_binding` and `register_conditional_binding` destructure
    `BindingTarget` with let-else, not an exhaustive match, so the new
    variant forces no T2 change and needs no dead variant; see the T1
    addendum correction 5)
    effect, whose **initial run is reconciled to a no-op** so static load
    + first effect run do not double-create children (plan obligation 3 /
    plan T6 bullet 3, the explicit test). T7 fills the effect body with
    the stage-then-commit tail insert/remove through the splice seam. So
    between T6 and T7 the initial render is correct and no shipped
    example issues a collection mutation (the gallery `Add`/`Remove`
    arrives in T8; the mutation headless fixtures are T7's). The T6→T7
    effect-body stub is a carry-forward row below.

  **Cross-task dependency facts recorded for the reviewer:**

  - **Both T4 (offsets) and T5 (carried placement) precede T7**, because
    the T7 splice seam consumes both. T4 and T5 are mutually independent
    (index math vs placement storage) and build in either order; the plan
    keeps T4→T5.
  - **T5 must keep the conditional mutation path green under
    child-carried placement before the unified splice seam exists.** The
    Phase 6 conditional path calls `insert_child` / `remove_child` /
    `insert_child_with_zstack_placement` directly
    ([ir_loader.rs:2021-2054](../../../../wasamo-runtime/src/ir_loader.rs#L2021));
    T5 changes how those carry placement (child-slot, not parallel
    vector), so T5 updates that path and re-greens the Phase 6 ZStack
    fixtures *before* T7 wraps the six-effect bundle into one seam and
    routes both conditional and `for` through it (DD-006).
  - **T1's instantiation context is consumed by T6** (static per-item
    reads at load) **and T7** (live per-item reads under mutation).

  No plan reorder results; the plan's T1 bullets are checked and these
  seams are cited from the plan tasks (see plan edits in this commit
  batch).

  ### 3. Risk-table sharpening + T2 gate selection (plan T1 bullet 3)

  **Sharpened R-A / R-B / R-C hotspots (pinned to current source).**

  - **R-A (I2 compile-error-forcing schema migration).** Schema sites in
    [`wasamo-ir/src/lib.rs`](../../../../wasamo-ir/src/lib.rs):
    `IrState.ty: IrType` → `IrStateType`
    ([lib.rs:86](../../../../wasamo-ir/src/lib.rs#L86));
    `IrLiteral` gains `List(Vec<IrLiteral>)`
    ([lib.rs:14](../../../../wasamo-ir/src/lib.rs#L14));
    `HandlerExpr` gains the collection read + loop-local reads +
    assignment forms
    ([lib.rs:44](../../../../wasamo-ir/src/lib.rs#L44));
    `ControlFlowNode` gains `For`
    ([lib.rs:149](../../../../wasamo-ir/src/lib.rs#L149)). **Trap-#1
    hotspot:** `IrNode::widget_children()`
    ([lib.rs:176](../../../../wasamo-ir/src/lib.rs#L176)) — a widget-only
    filter that already drops `ControlFlow` members; every use must be
    classified *correct* (layout-time over materialised children) or *a
    bug under `For`* (traversal over declared members). Construction /
    match sites to migrate: `wasamoc` `lower.rs` / `emit.rs` / `check.rs`;
    runtime `ir_loader.rs` emit + parse + `append_static_member`
    ([ir_loader.rs:1904](../../../../wasamo-runtime/src/ir_loader.rs#L1904))
    + `materialized_index_for_declared_member`
    ([ir_loader.rs:1975](../../../../wasamo-runtime/src/ir_loader.rs#L1975));
    `SignalRegistry` + `new()`
    ([reactive.rs:391-399](../../../../wasamo-runtime/src/reactive.rs#L391));
    `BindingTarget`
    ([reactive.rs:585](../../../../wasamo-runtime/src/reactive.rs#L585));
    every `IrState { ty: … }` literal across `wasamoc` + runtime + tests.
  - **R-B (C1 seam touches the shipped conditional path).** Extract /
    generalise `materialized_index_for_declared_member`
    ([ir_loader.rs:1975](../../../../wasamo-runtime/src/ir_loader.rs#L1975));
    migrate `mutate_conditional_subtree`
    ([ir_loader.rs:1989](../../../../wasamo-runtime/src/ir_loader.rs#L1989))
    onto the seam as the 0/1 case; `DeclaredMemberSlot` /
    `ConditionalRuntimeState`
    ([ir_loader.rs:92-99](../../../../wasamo-runtime/src/ir_loader.rs#L92))
    gain `ForLoop`; `register_conditional_binding`
    ([reactive.rs:674](../../../../wasamo-runtime/src/reactive.rs#L674))
    is the `for`-effect analogue (T7). Regression gate: the
    materialised-index unit tests
    ([ir_loader.rs:2448-2517](../../../../wasamo-runtime/src/ir_loader.rs#L2448))
    and `wasamo-runtime/tests/conditional_toggle_integration.rs`.
  - **R-C (ST2 touches shipped arrange / loader).** Placement storage:
    `WidgetData::ZStack { zstack_placements }`
    ([widget.rs:181](../../../../wasamo-runtime/src/widget.rs#L181)),
    `WidgetNode::zstack`
    ([widget.rs:648](../../../../wasamo-runtime/src/widget.rs#L648)),
    the placement insert/remove inside `insert_child_inner`
    ([widget.rs:1373](../../../../wasamo-runtime/src/widget.rs#L1373)) /
    `remove_child`
    ([widget.rs:1400](../../../../wasamo-runtime/src/widget.rs#L1400)),
    and `insert_child_with_zstack_placement`
    ([widget.rs:1325](../../../../wasamo-runtime/src/widget.rs#L1325)).
    Arrange read: `LayoutNode.zstack_placements`
    ([layout.rs:252](../../../../wasamo-runtime/src/layout.rs#L252)),
    `LayoutNode::zstack`
    ([layout.rs:479](../../../../wasamo-runtime/src/layout.rs#L479)),
    the zstack arrange loop
    ([layout.rs:1382-1405](../../../../wasamo-runtime/src/layout.rs#L1382)),
    and the `WidgetData::ZStack` → `LayoutNode::zstack` bridge
    ([widget.rs:1634](../../../../wasamo-runtime/src/widget.rs#L1634)).
    Loader extraction re-targets: `collect_static_zstack_placements`
    ([ir_loader.rs:2246](../../../../wasamo-runtime/src/ir_loader.rs#L2246)),
    `zstack_placement_for_parent`
    ([ir_loader.rs:2057](../../../../wasamo-runtime/src/ir_loader.rs#L2057)),
    `extract_zstack_placement` (used at
    [ir_loader.rs:1922](../../../../wasamo-runtime/src/ir_loader.rs#L1922)).
    Grid stays parallel + static-only: `WidgetData::Grid { cell_placements }`
    ([widget.rs:170-173](../../../../wasamo-runtime/src/widget.rs#L170))
    SoA comment gains the DD-M3-P7-006 trigger pointer; the trap-#3 close
    artifact is "`zstack_placements` deleted (greppable); `cell_placements`
    static-only with DD pointer".
  - **R-E / R-F (T7).** The guarded `ItemRead` is the
    `ForItemContext.position` out-of-range branch (§1); the cap fixtures
    exercise `MUTATION_CAP = 16`
    ([reactive.rs:10](../../../../wasamo-runtime/src/reactive.rs#L10)) —
    DD-007 confirms the cap charges drain **depth**, so a ≫N
    tail-append (e.g. 64) converges in one non-empty drain iteration.
    No sharpening beyond DD-005/DD-007; recorded for completeness.

  **T2 implementation-gates selection (recorded before T2 opens —
  plan T1 bullet 3 / preamble obligation 2).** T2 = the schema /
  IR-migration full-review-lane task.

  - **#1 semantic migration — APPLIES (the task's core).** Close
    artifact: the `rg`-enumerated call-site audit table over `IrState` /
    `IrMember` / `ControlFlowNode` / `HandlerExpr` (+ a `BindingTarget`
    pre-audit for T7), each site classified
    extended / correctly-unaffected / deliberately-rejects, with
    `IrNode::widget_children()` and every widget-only filter explicitly
    classified (the exact Phase 6 failure mode). Recorded in this log at
    T2 close.
  - **#2 side effects — not applicable to T2.** T2 makes no materialised-
    tree mutation; the registry collection maps are a state-store
    addition, not a structural edit with derived layout/Visual effects
    (those live in T5/T7).
  - **#3 parallel data drift — not applicable to T2.** T2 touches no
    placement vector (T5 owns that); the new registry maps are keyed by
    state name, not parallel to a child list.
  - **#4 untested branch — APPLIES (narrowly).** T2's own new reject
    branches — the deferred-load `For` arm (Seam A), and any
    `IrLiteral::List` / `IrStateType` loader element-type / nesting /
    list-on-scalar rejects that land in T2 rather than T3/T6 — each ship
    with a directly-firing test (the full DD-007 matrix is T3/T6).
  - **#5 carry-forward — APPLIES.** Seams A/B/C and the registry
    value-equality-on-set contract are invariants T4/T6/T7 depend on;
    recorded as the carry rows below with re-triggers.
  - **#6 root cause — standing**, not pre-selected.
  - **#7 GUI evidence — not applicable** (T2 has no GUI deliverable).
  - **Review lane:** **full independent review** (schema / IR migration
    high-risk class), composing in the trap-#4 branch/test check.

  ### Carry-forward rows (re-trigger criteria)

  | # | Carry | Owner / re-trigger |
  |---|---|---|
  | CF-1 | Seam A: T2's loader `For` arm is a deferred-load reject with a direct test | **T6** replaces it with static materialisation; re-trigger = T6 opening |
  | CF-2 | Seam B: `DeclaredMemberSlot::ForLoop` is dead (unconstructed) between T4 and T6 | **T6** first constructs it; re-trigger = T6 opening (the `dead_code` allowance closes there) |
  | CF-3 | Seam C: T6 registers `ForLoopSubtree` with a no-op initial reconcile; effect body stubbed | **T7** fills the stage-then-commit tail-edit body; re-trigger = T7 opening |
  | CF-4 | Guarded loop-local read ("write nothing" on out-of-range position) is a new binding-eval branch | **T6** (static) + **T7** (same-batch doomed binding) implement + directly test it; re-trigger = first loop-local read lowering |
  | CF-5 | T2 **adds** a `PartialEq`-gated equal-value-no-dirty set for the collection signals (currently `Signal::set` has no short-circuit — confirmed absent, the DD-005 "if absent" branch) | **T7** cap accounting + the empty-`drop-last` no-dirty fixture rely on it; re-trigger = first collection write path |
  | CF-6 | Handler-side collection-assignment evaluation (whole-`Vec` read-modify-write via an extended `HandlerEvalContext` + a new `HandlerExpr` evaluator arm) is the *writer* the `for` effect reacts to | **T7** (new explicit plan bullet); re-trigger = T7 mutation fixtures need an authored `append` / `drop-last` to drive a signal change |

  ### T1 close gate (artifacts)

  - **#5 carry-forward (the only applying trap):** recorded above as the
    CF-1..CF-5 table with owners and re-triggers, plus the design record
    in §1 and the sequencing seams in §2. The downstream consumers (T2
    gate selection, T6/T7 seams) read this log, not memory.
  - **Build/test sanity (no production Rust changed by T1):** the
    workspace `cargo build --workspace` is green and `cargo test
    --workspace` was run as a baseline proxy (T1 adds no Rust, so the
    fmt / clean-rebuild gate is the merge-base state; per the preamble
    the local clean-rebuild gate is owned by a task only when it changes
    production Rust). Recorded in the T1 retrospective
    ([../retrospectives/t1.md](../retrospectives/t1.md)).
