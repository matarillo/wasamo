## Decisions log

- **2026-06-07 / T6 start gate — R1 Window-title host-wiring:** selected
  implementation-gate traps before coding. Applies: **#2 missed side
  effects** (the static component-level `title` must affect native window
  creation, while dynamic `bind title` remains deferred); **#4 untested
  authored branch** (the loader adds a non-`Str` `title` rejection branch and
  the absent / empty fallback branch must be pinned); **#7 positive-control
  discipline for visible state** (T6's CI-gated evidence is live HWND title
  state, while screenshot / assistant analysis of the title bar is owned by
  T7 and owner-visible corroboration by T8). Not applicable: **#1 semantic
  migration** (no enum / IR schema variant or field is added); **#3 parallel
  data drift** (no parallel vector / map / index is introduced or mutated);
  **#6 root cause** (no recurring failure observed at task start). **#5
  carry-forward was reclassified during review follow-up**: dynamic title was
  already ADR-deferred, but T6 did add a title-specific loader invariant that
  should inform the later Window-prop seam. Review lane: **full independent
  review** because the task includes Windows-runtime evidence, with the trap
  #4 branch/test check folded into that review.
- **2026-06-07 / T6 R1 static Window-title host-wiring:** T6 confirms the
  R-D extraction point: the static component-level `title:` has already been
  spliced onto `component.root.props`, while a dynamic `bind title = ...`
  remains in `root.bindings` and is still the DD-M3-P6-006 deferred
  window-property-binding seam. The runtime now validates that any root
  `title` prop is an `IrLiteral::Str`, resolves an absent or empty title to
  `DEFAULT_WINDOW_TITLE`, and passes the non-empty string literal to
  `window::create` in `wasamo_load_ui`. The malformed-title rejection is
  intentionally single-sourced in `validate_static_window_title`;
  `resolve_static_window_title` is a crate-local infallible projection over a
  validated component. No ABI signature, export, `PropertyValue` tag, or
  `docs/abi_spec.md` text changed. The stale counter example README notes
  that said DSL titles were still dropped were refreshed to match the
  implemented host path.
  - **Close-gate artifacts:** #2 side effects — static `title` now affects
    native window creation; dynamic title remains deliberately deferred and
    unwired; `backdrop` / `theme` remain untouched. #4 branch tests —
    `static_window_title_resolves_string_or_default` pins absent / empty /
    string resolution, `static_window_title_rejects_non_string_root_prop`
    pins the loader rejection branch, and `abi_load_ui` pins
    `WASAMO_ERR_IR_MALFORMED` at the ABI boundary. #5 carry-forward — later
    Window-derived props should reuse this validate-then-resolve split rather
    than adding silent fallback for wrong-typed direct IR. #7 Windows-runtime
    state evidence — `static_component_title_reaches_native_window` lowers a
    `.ui` declaring `title: "Gallery"`, loads it through `wasamo_load_ui`,
    then reads the live HWND title via `GetWindowTextW`; the positive control
    is `"Gallery"` rather than the prior `"Wasamo"` default. T7/T8 still own
    the screenshot / human-visible title-bar corroboration.
- **2026-06-05 / Observation 5 remediation step 1 — marshal onto owning
  thread + abbreviated retro (branch `test/obs5-step1-marshal-owning-thread`
  → `feat/m3-phase-6`):** step 1 — owner-scheduled at the step-2 close — is
  now done. The keep-alive `tests/common/mod.rs` park thread became a
  work-queue executor; `run_on_owning_runtime_thread_or_skip` replaces
  `init_runtime_or_skip` and runs each Compositor test body on the single
  owning thread (panic caught there + re-raised on the libtest thread so
  `#[test]` still fails correctly). The five ≥2-Compositor binaries wrap
  their bodies in the helper closure. This eliminates the cross-apartment
  residual step 2 only tolerated. Abbreviated retro (out-of-band step, no
  numbered task slot → folded here, not a `tN.md`), per retrospectives.md
  items 1–11:
  - **Main learning:** the "does this one helper hold too many
    responsibilities?" question resolved not to *one responsibility* but to
    **shared change/deletion locality + coupling avoidance** — init, skip
    policy, marshalling, and panic-relay have *different* change drivers yet
    are added and deleted together, and splitting the skip check back to the
    callers would re-introduce a two-calls-must-agree coupling on the same
    process-global init outcome. Recorded in the helper's own rationale
    comment so the design intent travels with the code.
  - **Items 2 / 6 / 7 / 8 = none:** no spec-doc (`abi_spec` / `architecture`
    / `dsl_spec`) change; no new or promoted DD / ADR; no milestone-AC or
    phase-structure change. Changes are limited to the test harness and
    `docs/notes/`.
  - **Item 9 (carry-forward):** none new — step 1 *was* the carry-forward
    from the step-2 close and is now discharged. The helper's only remaining
    lifecycle is its deletion condition (process-per-test runner, e.g.
    `cargo nextest`, or libtest ceasing per-test thread spawn), documented in
    [docs/notes/verification-environments.md Observation 5](../../../../docs/notes/verification-environments.md)
    §Remediation status and the helper module doc.
  - **Item 10 (cross-task constraint), `doc-folded`:** unchanged in substance
    — ≥2-Compositor binaries route through the shared helper; the helper now
    additionally *executes* bodies on the owning thread rather than only
    keeping the apartment alive. Folded into Observation 5 and the helper's
    module doc; pointer only here.
  - **Item 11 (ownership):** Observation 5 §Remediation status flips step 1
    to DONE in the same change; no open `[ ]` left implicit.
- **2026-06-05 / Observation 5 teardown-AV investigation — abbreviated
  retrospective (branch `investigate/obs5-scrollview-teardown-av` →
  `feat/m3-phase-6`):** an out-of-band investigation, not a numbered plan
  task, so it gets an abbreviated retro folded here rather than a
  `retrospectives/tN.md` file (no task slot to invent). The clean-rebuild
  gate is in the CI/verification log above (green, first run, no AV).
  Per the task-end checklist (retrospectives.md items 1–11):
  - **Main learning:** the original symptom framing can be wrong — the AV
    was filed for two phases as a "process-exit teardown" fault on the
    assumption that a printed `... ok` meant the crash was in teardown. A
    minidump (`procdump -e -ma` + `cdb`) showed it is in the *next* test's
    `build_widget_tree` → `CreateSpriteVisual`, dispatching through a vtable
    in an unloaded `dcomp.dll`. The diff-independent recurrence plus the
    "capture the dump, don't re-roll" standing rule is what eventually
    forced the correct diagnosis. Method (repro matrix → minidump → faulting
    module) generalises to future native-COM AVs.
  - **Items 2 / 6 / 7 / 8 = none:** no spec-doc (`abi_spec` / `architecture`
    / `dsl_spec`) change; no new or promoted DD / ADR; no milestone-AC or
    phase-structure change. Changes are limited to `docs/notes/` and the
    test harness.
  - **Item 9 (carry-forward):** remediation **step 1** (marshal Compositor
    work onto the owning thread) is deferred to a separate owner decision —
    *no hard deadline*, with revisit triggers — recorded in
    [docs/notes/verification-environments.md Observation 5](../../../../docs/notes/verification-environments.md)
    §Remediation status. The residual it addresses (test bodies calling
    non-agile Composition objects cross-apartment, benign only while
    `dcomp.dll` is held resident) is UB-adjacent but test-harness-only;
    production is unaffected (hypothesis A confirmed, B excluded).
  - **Item 10 (cross-task constraint), `doc-folded`:** Compositor
    integration binaries with two or more Compositor tests must initialize
    the runtime on a process-lifetime thread via the shared
    `wasamo-runtime/tests/common/mod.rs` keep-alive helper, so the
    Compositor's apartment is not torn down between tests. Folded into
    Observation 5 and the helper module's own doc comment (which states the
    rationale, when the helper is/ isn't required, and its deletion
    conditions); pointer only here.
  - **Item 11 (ownership):** the phase plan's Phase 7 handoff bullet for the
    teardown-AV investigation is revised in the same change to reflect
    root-cause-done + step 2 landed + step 1 owner-deferred (no hanging
    `[ ]` left implicit).
- **2026-06-05 / T5 conditional reactive runtime:** T5 fills the
  DD-M3-P6-004 / 005 structural binding seam without adding any IR / ABI /
  grammar or host-facing error surface. `BindingTarget::ConditionalSubtree { parent,
  declared_member_index }` is registered through a new
  `register_conditional_binding` wrapper over `EffectHandle::new`; property
  binding entry points now destructure `BindingTarget` refutably. The
  runtime tracks each parent's declared member slots as
  `DeclaredMemberSlot::{Widget, Conditional(state)}` while iterating
  `IrMember` in declared order; the materialised insertion/removal index is
  recomputed from preceding declared slots and each conditional's live
  presence bit on every mutation. This closes the T4 carry-forward
  constraint for T5's positional path: the traversal that materialises
  conditionals and computes positional metadata dispatches on `IrMember`,
  not `widget_children()`.
- **2026-06-05 / T5 positional Visual + ZStack placement update:** The
  `WidgetNode::insert_child` Visual operation is now index-aware: append
  still uses `InsertAtTop`, while mid-list insertion uses `InsertBelow`
  relative to the current child at the target index so live Visual sibling
  order matches `WidgetNode.children`. Because ZStack stores
  parent-owned child placement metadata parallel to materialised children,
  dynamic insert/remove also updates the ZStack placement vector at the same
  index (`insert_child_with_zstack_placement` / `remove_child`). This is the
  T5 R-F closure and preserves the T4 traversal-audit rule for positional
  metadata.
- **2026-06-05 / T5 ZStack placement construction refactor:** The T5
  positional-mutation fix also moved static ZStack placement construction
  from a precomputed `collect_static_zstack_placements` vector to the same
  per-child insertion path used for dynamic members:
  `append_static_member` calls `insert_child_with_zstack_placement` whenever
  the parent is ZStack. The old static reducer helpers
  (`evaluate_static_condition`, `collect_static_zstack_placements`) are now
  `#[cfg(test)]`; their unit tests still pin reducer logic and are now
  commented as such, but no longer guard a production call-site directly.
  The new load-bearing index reducer
  `materialized_index_for_declared_member` has headless unit coverage in
  `materialized_index_counts_preceding_widgets_and_live_conditionals`,
  including the preceding-conditional removal shift. Production placement
  evidence is covered by the ZStack Windows integration fixtures and T5's
  `conditional_zstack_reinsert_uses_declared_placement_metadata`.
- **2026-06-05 / T5 parent-owned metadata mutation constraint:** The
  ZStack placement-vector fix surfaced a future-structural constraint:
  under the current SoA model, any structural mutation primitive that changes
  a materialised child list under a container with parent-owned positional
  metadata must update that metadata atomically with `WidgetNode.children`
  and the live Visual sibling order. T5 implements the single-child case for
  conditional insert/remove, but this invariant is a cost of the current
  parallel-vector representation, not a law that Phase 7 must preserve.
  Phase 7 must decide the placement storage model before `ForLoopSubtree`:
  keep SoA parallel vectors (affirm DD-M3-P6-002's implementation shape),
  move placement onto child nodes / child records (AoS, superseding the
  current shape), or use a `WidgetId`-keyed metadata map. Children ↔ Visual
  order synchronisation is unavoidable in every model; the reducible
  parallel structure is the placement vector itself, and the value of
  removing it grows linearly with future parent-owned per-child metadata
  kinds. T5 is sample 1 for dynamic parallel-vector sync; `ForLoopSubtree`
  would be sample 2, so the ≥2-sample discipline makes Phase 7 the decision
  point. T5's `append_child` consolidation is a local guard and remains
  subordinate to that Phase 7 storage-model decision.
- **2026-06-05 / T5 structural-binding handover constraints:** Conditional
  initial presence is now established by `EffectHandle::new`'s eager initial
  run; a future reactive-engine change that delays initial Effects must
  preserve this loader materialisation contract or add an explicit
  initialisation path. ZStack-aligned structural insertion must use the
  placement-carrying API; T5 guards the former two-path footgun by making
  `append_child` delegate to `insert_child_inner(len, child, None)`, so the
  centered ZStack default is concentrated in one insertion primitive.
  Conditional mutation build / insert / remove / slot-missing failures
  remain log-only (`eprintln!`) and are not surfaced through runtime health;
  Phase 7 range mutation should re-check whether log-only structural failure
  remains sufficient for multi-child edits. The final API consolidation shape
  remains dependent on the Phase 7 placement-storage model decision.
- **2026-06-05 / T5 self-review layout invalidation fix:** The initial T5
  implementation inserted/removed conditional children synchronously but did
  not mark the owning window layout-dirty on structural success. Self-review
  classified that as T5-owned, because a conditionally-present subtree can
  affect parent measurement/allocation even when no size-affecting property
  write occurs. `mutate_conditional_subtree` now marks dirty via the parent
  widget after successful insert/remove. The same pass added
  `conditional_zstack_reinsert_uses_declared_placement_metadata`, which
  drives conditional insert/reinsert under `ZStack` through
  `run_layout_as_window_root` and asserts the dynamic child's declared
  `h-align` / `v-align` placement.
- **2026-06-05 / T5 follow-on classification for dirty-layout evidence:**
  This is **not** a Phase 7 carry-forward. T5 fixed the structural mutation
  primitive, but the full real-window path (`mark_layout_dirty_for` →
  `drain_if_outermost` → `flush_layout` under `WindowState`) must be pinned
  by Phase 6 GUI evidence. T7 now owns the assistant screenshot before/after
  pair captured immediately after the click-driven lightbox toggle, without
  relying on resize; T8 owns the same owner-visible smoke criterion and the
  Phase 6 fix slot if the path fails. T9 Moment 2 architecture sync must
  include `docs/architecture.md` §6.6 so layout invalidation is no longer
  documented as property-change-only.
- **2026-06-05 / T5 closes T4b DD-M3-P6-007 comment handoff:** T4b left a
  narrow source-comment follow-up for the next `ir_loader.rs` touch: refresh
  `validate_phase4_node_invariants` from the "interim / open DD-007" wording
  to the accepted-(a) ScrollView direct-conditional rejection. T5 performed
  that refresh; the handoff is closed without reopening DD-M3-P6-007.
- **2026-06-05 / T5 reactive-drain items 1–3 disposition:** T5 implements
  the DD-M3-P6-005 DB-1 item-4 proof and does **not** revise the inherited
  reactive-drain items 1–3. Cycle detection, ordering ties, and fan-out ×
  `MUTATION_CAP` remain the DD-M3-P6-005 SM-1 carry-forward exactly as the
  ADR records: the conditional insertion Effect writes the widget tree, not
  its own Signal; quiescent child order is fixed by declared member order;
  and large-subtree cap strategy is deferred until the structural family
  (`for` / larger repeated subtrees) reveals the real budget requirement.
  This entry covers only the reactive-drain items 1–3 disposition; the
  separate parent-owned metadata mutation constraint above is the T5-specific
  carry-forward candidate.
- **2026-06-05 / T5 surfaced known issue — ScrollView teardown AV (carry-forward):**
  T5's follow-up clean rebuild re-observed the `scroll_view_layout_integration`
  process-exit access violation (see the CI/verification entry below). It is
  diff-independent (same fault recorded in Phase 5 T1 with a `wasamoc`-only
  diff) and therefore **not** a T5 regression, so it does not gate the T5
  merge. It is **not** settled as benign either: the fault is in COM/Compositor
  teardown at process exit and a real runtime teardown defect (hypothesis B)
  is not excluded. Disposition recorded as
  [docs/notes/verification-environments.md Observation 5](../../../../docs/notes/verification-environments.md)
  (hypotheses A/B; "capture a minidump on the next occurrence rather than
  re-rolling to green"; the faulting module decides the fix). **Carry-forward:
  promote this into the phase-end `handoff.md` (T9) as a Phase 7 / runtime
  investigation item** — root-cause the teardown AV from a captured dump and
  decide the permanent fix (never-dropped global Compositor + no
  `RoUninitialize`, vs a `widget_destroy` teardown-order fix). This has now
  recurred ≥2 times, so by the project's ≥2-sample discipline it graduates
  from "transient" to a tracked known issue.
- **2026-06-03 / T4 IrMember schema migration:** T4 landed the accepted
  DD-M3-P6-004 O1 shape directly: `IrNode.children` is now
  `Vec<IrMember>`, with `IrMember::Widget(IrNode)` and
  `IrMember::ControlFlow(ControlFlowNode::If { branches })`.
  Construction-site migration used a narrow helper discipline rather than
  a broad abstraction: production walkers use `IrNode::widget_children()`
  when an invariant is widget-child-only, and explicit
  `IrMember` dispatch where control flow is semantically relevant
  (`wasamoc` lower / emit; runtime parse / validate / static member
  append). The schema change, `wasamoc` emit/lower, textual IR parser,
  validators, and static load-time presence reducer were bundled in one
  buildable implementation commit per the T4 R-A/R-B risk note.
- **2026-06-03 / T4 review follow-up traversal audit:** owner review
  found two places where the initial `widget_children()` split silently
  changed semantics: phase-specific runtime validators did not descend
  into `ControlFlow` bodies, and ZStack placement vectors were built from
  widget-only declared children while materialised children came from
  static `IrMember` expansion. The follow-up fixes both by dispatching
  phase validators through `IrMember`, building ZStack placements through
  `collect_static_zstack_placements` with the same load-time condition
  evaluation as `append_static_member`, and rejecting direct runtime
  `ControlFlow` under `Grid` / `Cell` because those IR-only wrappers are
  flattened by a Grid-specific build path. This amends the T4
  retrospective's original "no new constraint" statement: declared-member
  traversal that affects validation or positional metadata must dispatch
  `IrMember`; widget-only helpers are valid only when dropping
  `ControlFlow` is explicitly part of the invariant.
- **2026-06-03 / T4 review follow-up #2 — `Vec<IrMember>` traversal
  call-site audit (semantic-migration audit):** a second review pass
  produced the explicit traversal-contract audit the original migration
  should have carried. Each production `IrMember`-bearing traversal was
  classified `must-dispatch` / `ignore-OK (+ proof)` /
  `defer-with-approval`; every `ignore-OK` carries a reject test or an
  impossibility note (the bar that makes the no-constraint claim
  falsifiable).
  - **`must-dispatch ✓` (already correct):** control-flow shape
    (`validate_phase6_control_flow_invariants`); the Phase 2/3/4/5/ZStack
    validator `*_member_invariants` body recursions; `validate_node_references`
    → `validate_member_references` (condition validation); the non-Grid
    build append (`append_static_member`); ZStack placements
    (`collect_static_zstack_placements`); `wasamoc` `lower` / `emit`
    member dispatch. Evidence: `validate_rejects_*`,
    `zstack_static_placements_follow_materialized_member_order`,
    `conditional_lowers/emitted_*`.
  - **`ignore-OK` (ControlFlow legitimately dropped):** Grid/Cell build
    + validate widget-only iterations — proof: `Grid`/`Cell` reject all
    direct `ControlFlow` upstream (`validate_rejects_direct_conditional_{grid,cell}_member`),
    so no `ControlFlow` reaches those sites. `WidgetNode.children` walks
    in `widget.rs` — impossibility note: the materialised widget tree has
    no `ControlFlow` variant (it is expanded at build).
  - **FINDING (was mis-classified `ignore-OK`, corrected to
    `must-dispatch`):** the **Box at-most-one** and **ScrollView
    exactly-one** child-count gates counted `widget_children()` only, so a
    conditional sibling (`Box { Content  if c }` / `ScrollView { Content
    if c }`) under-counted and slipped past **both** `wasamoc check` and
    runtime `validate()`, materialising two children. This is the same
    widget-only-vs-materialised root as the review-#1 findings; review #1
    fixed validator *descent* but left the *count basis* widget-only.
    Fixed: Box counts every child member (`node.children.len()`;
    `WidgetDecl | Conditional` at check); ScrollView rejects any direct
    conditional member (interim, symmetric with Cell). The
    conditional-only ScrollView case (`ScrollView { if c { … } }`) is left
    rejected pending **DD-M3-P6-007** (the conditionally-empty-container
    relaxation is a Phase 6 design decision, owner-gated).
  - **Rule candidate (carry-forward, not yet ruled):** "any traversal
    that validates declared structure, computes positional metadata, or
    materialises declared members must dispatch on `IrMember` unless it
    has a documented, tested widget-only invariant; prefer compile-error-
    forcing mechanisms (exhaustive `match`, no `Default`) over
    silent-absorb helpers (filtering iterators)." Precedent: the
    `kind_payload` migration (DD-M3-P5-001) used no-`Default` to force
    construction-site compile errors (success); the `IrMember` filtering
    helper bypassed that discipline (this failure). Recorded as a
    handoff carry-forward; rule-ification (workflow.md / retrospectives.md
    + a vision decision record) deferred to the next semantic migration so
    the rule is designed against ≥ 2 samples, not over-fit to one.
- **2026-06-03 / T3 skip-guard disposition:** ZStack live Visual
  integration introduces no new runtime capability path beyond the
  existing `wasamo_init` → Compositor creation surface. The
  `init_runtime_or_skip` guard in
  `wasamo-runtime/tests/zstack_layout_integration.rs` therefore reuses
  the Phase 5 Grid pattern byte-for-byte in behavior: local
  `0x80070005` returns `None` (developer-laptop skip), while GitHub
  Actions fails rather than silently skipping. This records the
  inheritance disposition requested by T3 instead of re-proving the
  already inherited missing-Compositor path.
- **2026-06-03 / T3 VisualCollection evidence seam:** The ZStack live
  Visual-order fixture needs to enumerate `VisualCollection`; the
  runtime crate's existing `windows` dependency now enables the
  `Foundation_Collections` feature so the test can read the live child
  collection directly. This is an API-feature enablement for the
  existing dependency, not a new build system / CI surface.
- **2026-06-04 / T4b DD-M3-P6-007 accepted (a):** the open ScrollView
  conditional-content question closed **(a) — reject a direct conditional
  member; defer conditionally-empty content**, after a multi-pass
  design-decision review (strategic / recommendation-choice /
  implementation-readiness). Doc/process only: DD-007 `Proposed → Accepted`,
  preamble §Decisions index (+ Revisions), `docs/dsl_spec.md` §4.11 sentence
  + §4.14 diagnostics row, plan.md T4b. **No code change** — the T4
  review-follow-up interim is the final rule, so the existing dual-gate
  tests (`scrollview_conditional_member_rejected` /
  `scrollview_conditional_only_member_rejected`;
  `validate_rejects_scrollview_with_conditional_member` /
  `validate_rejects_scrollview_with_conditional_only_member`,
  `IrLoadError::Validate` → `WASAMO_ERR_IR_MALFORMED`) are the final
  evidence. The review found and corrected a stub citation error
  (ScrollView exact-one = DD-M3-P4-001, not DD-M3-P4-003).
  - *Deferred (low harm, not a checklist item):* `ir_loader.rs`
    `validate_phase4_node_invariants` still narrates the rejection as the
    "interim / open DD-M3-P6-007 ... until that is decided" state. Refresh
    that provenance comment to "accepted (a); conditionally-empty direction
    deferred" at the **next `ir_loader.rs` touch (T5)** or phase-end — left
    now to keep T4b a code-no-touch close; harm is low (the comment still
    links DD-M3-P6-007, and behaviour / diagnostic / public spec are
    correct).

---

## CI / verification log

- **2026-06-07 / T6 local verification:** scoped checks green —
  `cargo test -p wasamo-runtime --lib static_window_title` (2 tests),
  `cargo test -p wasamo-runtime --test abi_load_ui` (1 test), and
  `cargo test -p wasamo-runtime --test window_title_integration` (1 test).
  Final clean-rebuild gate green: `cargo fmt --all -- --check`;
  `cargo clean` (`4329 files, 1.4GiB` removed);
  `cargo build --release --workspace` (37.12s);
  `cargo build --workspace` (35.67s); `cargo test --workspace` (included the
  new `window_title_integration` fixture). Existing Cargo warnings about the
  `wasamo` linkable target / `wasamo-sys` import-library ordering were
  observed.
- **2026-06-05 / Observation 5 remediation step 1 — local gate + GitHub
  Actions CI (branch `test/obs5-step1-marshal-owning-thread`, commit
  `4d2cb3e`):** local clean-rebuild gate green — `cargo fmt --all -- --check`
  green; `cargo clean` (3764 files, 1.2GiB removed); `cargo build --release
  --workspace` green (40.6s); `cargo build --workspace` green;
  `cargo test --workspace` green. (A direct `cargo test --workspace` straight
  from `cargo clean` first hit the known LNK1356 `wasamo-sys → wasamo-dll`
  `/WHOLEARCHIVE` ordering race (DD-M2-P1-006); building the workspace first,
  as CI does, then testing was green — not a regression from this change.)
  Targeted: full `wasamo-runtime` suite (333 unit + all integration) green
  under `--test-threads=1`, the form that previously crashed deterministically
  with `0xC0000005`. **Positive control:** a temporary thread-identity probe
  showed the marshalled test body running on `wasamo-test-runtime-owner` while
  its caller ran on the libtest thread named after the test — distinguishing
  real owning-thread execution from a no-op wrapper (which would have printed
  the same name twice); the probe was reverted before commit. **GitHub Actions
  CI:** run
  [27014203528](https://github.com/matarillo/wasamo/actions/runs/27014203528)
  (`workflow_dispatch` on the branch, headSha `4d2cb3e`) — conclusion
  **success** (~3m8s); `Test (workspace)` and all binding / example smoke
  steps green on the windows-latest runner, confirming the executor-thread
  marshalling works on the actual CI runner (default multi-threaded
  `cargo test --workspace`). Existing Cargo warnings about the `wasamo`
  linkable target / `wasamo-sys` import-library ordering were observed.
- **2026-06-05 / Observation 5 remediation step 2 — task-end clean rebuild
  (branch `investigate/obs5-scrollview-teardown-av`, post-commits
  `02ff614`, `a304dc5`, `83aadb7`):** `cargo fmt --all -- --check` — green
  (post-commit state); `cargo clean` completed (`5067 files, 1.4GiB`
  removed); `cargo build --release --workspace` — green (44.4s);
  `cargo build --workspace` — green (41.3s); `cargo test --workspace` —
  green on the **first** run (23s; `wasamo-runtime` lib 333, `wasamoc` 316,
  `wasamo-ir` 17, all integration suites, 0 failed). The process-exit
  access violation that forced a `--workspace` rerun at the T5 follow-up
  clean rebuild did **not** recur: the keep-alive apartment helper
  (`wasamo-runtime/tests/common/mod.rs`) keeps `dcomp.dll` resident for the
  whole test binary. Positive control: before the fix, `scroll_view` /
  `wrap_panel` / `grid` crashed 5/5 · 3/3 · 3/3 under `--test-threads=1`;
  after, the full `wasamo-runtime` suite is green under `--test-threads=1`
  as well. Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-05 / T5 follow-up clean rebuild (post-commits `cc5d130`,
  `35c2d88`, `f7a2281`):** `cargo clean` completed (`5311 files,
  1.4GiB` removed); `cargo fmt --all -- --check` — green;
  `cargo build --release --workspace` — green (57.88s);
  `cargo build --workspace` — green (47.89s). First
  `cargo test --workspace` run hit a `scroll_view_layout_integration`
  process-exit access violation **after individual assertions had passed**
  (the fault is in COM/Compositor teardown at process exit, not in the
  asserted ScrollView behaviour); the three ScrollView integration tests
  were rerun individually and were green, and the subsequent
  `cargo test --workspace` rerun was green (`wasamo-runtime` lib 333,
  `wasamoc` 316, `wasamo-ir` 17, integration suites all green, 0 failed).
  This matches the **same teardown AV recorded in Phase 5 T1**
  ([phase-5/t1.md](../../phase-5/retrospectives/t1.md)), where the diff was
  `wasamoc`-only and never touched the insertion path — so it is
  diff-independent and not a T5 regression (T5's `append_child` delegation
  is behaviour-identical for ScrollView). It is **not** dismissed as a mere
  flake: the known-issue disposition (hypotheses + "capture a minidump on
  next occurrence rather than re-rolling"; production teardown defect not
  yet excluded) is recorded as
  [docs/notes/verification-environments.md Observation 5](../../../../docs/notes/verification-environments.md)
  and carried forward below. Existing Cargo warnings about the `wasamo`
  linkable target / `wasamo-sys` import-library ordering were observed.
- **2026-06-05 / T5 local scoped:** `cargo test -p wasamo-runtime --test
  conditional_toggle_integration` — green (2 tests). Added
  `conditional_toggle_preserves_declared_visual_order_and_disposes_registry`
  for declared sibling order, two sibling conditionals, preceding removal
  index shift, true→true / false→false no-op, live VisualCollection order,
  and registry teardown through `widget_destroy`; added
  `conditional_toggle_drains_fresh_subtree_effects_before_return` for
  same-drain present/absent observation and freshly-created subtree Effects
  observing the latest state before the toggling setter returns.
- **2026-06-05 / T5 local scoped runtime:** `cargo test -p wasamo-runtime
  --lib ir_loader::tests` — green (127 tests);
  `cargo test -p wasamo-runtime --lib reactive::tests` — green (39 tests);
  `cargo test -p wasamo-runtime` — green (runtime lib 332 plus all
  integration suites, including the new conditional toggle fixture).
- **2026-06-05 / T5 local pre-retro:** `cargo fmt --all -- --check` —
  green; `cargo build --release --workspace` — green; `cargo build
  --workspace` — green; `cargo test --workspace` — green; `cargo test -p
  wasamo-runtime` — green. Existing Cargo warnings about the `wasamo`
  linkable target / `wasamo-sys` import-library ordering were observed.
- **2026-06-03 / T4 local scoped:** `cargo fmt --all -- --check`
  — green; `cargo test -p wasamo-ir` — green (17 tests);
  `cargo test -p wasamoc --lib` — green (308 tests);
  `cargo test -p wasamo-runtime --lib` — green (322 tests).
  Covered `IrMember` schema encoding, control-flow keyword / parser /
  check / lower / emit diagnostics, runtime textual IR parsing /
  roundtrip, and static conditional presence / validator rejection
  evidence.
- **2026-06-03 / T4 local pre-commit:** `cargo build --release
  --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green; `cargo test -p wasamo-runtime`
  — green (runtime lib 322 plus integration tests). Existing Cargo
  warnings about the `wasamo` linkable target were observed.
- **2026-06-03 / T4 task-end clean rebuild (post-commit
  `774b567`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`4862 files, 1.3GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green; `cargo test -p wasamo-runtime`
  — green (runtime lib 322 plus integration tests). Existing Cargo
  warnings about the `wasamo` linkable target / `wasamo-sys`
  import-library ordering were observed.
- **2026-06-03 / T4 review follow-up local:** `cargo test -p
  wasamo-runtime --lib ir_loader::tests` — green (122 tests);
  `cargo test -p wasamoc --lib check::tests::conditional` — green
  (11 tests); `cargo test -p wasamoc --lib` — green (310 tests);
  `cargo test -p wasamo-runtime --lib` — green (327 tests). Covered
  Grid direct conditional diagnostics, literal-condition diagnostics,
  runtime bool-read/non-bool condition rejection, ControlFlow-body
  validator descent, Grid/Cell direct-ControlFlow runtime rejection, and
  ZStack static placement order matching materialised member order.
  Final follow-up verification: `cargo fmt --all -- --check` — green;
  `cargo build --workspace` — green; `cargo test --workspace` —
  green; `cargo test -p wasamo-runtime` — green (runtime lib 327 plus
  integration tests). Existing Cargo warnings about the `wasamo`
  linkable target were observed.
- **2026-06-03 / T4 Cell conditional check follow-up local:**
  added the source-level dual gate for `Cell { <widget> if ... }`,
  matching the runtime `validate_rejects_direct_conditional_cell_member`
  defense-in-depth rejection. `cargo test -p wasamoc --lib
  check::tests::conditional_cell_sibling_rejected` — green; `cargo test
  -p wasamoc --lib check::tests::conditional` — green (12 tests);
  `cargo test -p wasamoc --lib` — green (311 tests); `cargo fmt --all
  -- --check` — green.
- **2026-06-03 / T4 review follow-up #2 — Box/ScrollView count fix
  (clean rebuild):** the `Vec<IrMember>` traversal call-site audit fix.
  Box (`node.children.len()`; `WidgetDecl | Conditional` at check) and
  ScrollView (reject direct conditional member, interim) single-child
  gates now count a conditional sibling at both `wasamoc check` and
  runtime `validate()`. Added tests
  `box_widget_and_conditional_sibling_rejected`,
  `box_conditional_only_child_accepted`,
  `scrollview_conditional_member_rejected` (`wasamoc`),
  `validate_rejects_box_with_widget_and_conditional_sibling`,
  `validate_accepts_box_with_conditional_only_child`, and
  `validate_rejects_scrollview_with_conditional_member` (runtime).
  `cargo fmt --all -- --check` — green; `cargo clean` completed
  (`5038 files, 1.3GiB` removed); `cargo build --release --workspace`
  — green (46.84s); `cargo build --workspace` — green (41.02s);
  `cargo test --workspace` — green (`wasamoc` 314, `wasamo-runtime` lib
  330, `wasamo-ir` 17, integration suites all green, 0 failed). Existing
  Cargo warnings about the `wasamo` linkable target / `wasamo-sys`
  import-library ordering were observed.
- **2026-06-03 / T4 review follow-up #2 — Codex review additions (clean
  rebuild):** Codex re-review returned no blocker; one should-fix (pin the
  DD-M3-P6-007 centre case `ScrollView { if c { … } }` directly) and one
  nit (a multiple-conditional-sibling Box reject as the shortest
  `children.len()` count-basis proof). Added
  `scrollview_conditional_only_member_rejected`,
  `box_multiple_conditional_siblings_rejected` (`wasamoc`),
  `validate_rejects_scrollview_with_conditional_only_member`, and
  `validate_rejects_box_with_multiple_conditional_siblings` (runtime).
  `cargo fmt --all -- --check` — green; `cargo clean` completed
  (`3150 files, 1.1GiB` removed); `cargo build --release --workspace`
  — green (44.08s); `cargo build --workspace` — green (35.29s);
  `cargo test --workspace` — green (`wasamoc` 316, `wasamo-runtime` lib
  332, `wasamo-ir` 17, integration suites all green, 0 failed). Existing
  Cargo warnings about the `wasamo` linkable target / `wasamo-sys`
  import-library ordering were observed.
- **2026-06-02 / T1 local:** `cargo fmt --all -- --check` — green.
- **2026-06-02 / T1 local:** `cargo test -p wasamoc` — green;
  covered the ZStack check / lower / emit evidence with tests including
  `zstack_known_widget_no_warning`,
  `zstack_direct_child_alignment_accepted`,
  `zstack_unknown_attribute_rejected`,
  `zstack_reserved_layering_attribute_rejected`,
  `zstack_grid_track_attribute_rejected`,
  `zstack_child_bad_alignment_value_rejected`,
  `placement_attr_outside_zstack_child_or_cell_rejected`,
  `placement_attr_on_zstack_itself_rejected_with_container_position`,
  `zstack_lowers_as_direct_children_without_kind_payload`, and
  `zstack_emitted_as_node_with_direct_children_in_order`.
- **2026-06-02 / T1 local:** `cargo clippy -p wasamoc` — green.
- **2026-06-02 / T1 task-end clean rebuild:** `cargo clean`
  completed (`2993 files, 1012.3MiB` removed);
  `cargo build --release --workspace` green; `cargo build --workspace`
  green; `cargo test --workspace` green. Existing Cargo warnings about
  the `wasamo` linkable target / `wasamo-sys` import-library ordering
  were observed.
- **2026-06-02 / T2 local:** `cargo fmt --all -- --check` — green.
- **2026-06-02 / T2 local:** `cargo test -p wasamoc` — green.
- **2026-06-02 / T2 local:** `cargo test -p wasamo-runtime zstack` —
  green; added pure-logic ZStack layout tests
  `zstack_defaults_to_fill_fill_and_centers_children`,
  `zstack_shrink_measure_uses_child_union_with_fill_child_zero`,
  `zstack_arrange_alignment_overrides`, and
  `zstack_arrange_preserves_document_order_substrate`.
- **2026-06-02 / T2 local:** `cargo build --release --workspace` —
  green. Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-02 / T2 task-end clean rebuild:** `cargo clean` completed
  (`4163 files, 1.1GiB` removed); `cargo build --release --workspace`
  green; `cargo build --workspace` green; `cargo test --workspace`
  green. Existing Cargo warnings about the `wasamo` linkable target /
  `wasamo-sys` import-library ordering were observed.
- **2026-06-02 / T2 review follow-up local:** tightened the
  `zstack_arrange_preserves_document_order_substrate` evidence so the two
  children have distinguishable overlapping geometry, corrected the T2
  retrospective's limited helper-rename classification, and renamed
  `align_in_rect` parameters from cell-specific to rect-specific names.
  `cargo fmt --all -- --check` green; `cargo test -p wasamo-runtime
  zstack` green (4 passed); `cargo build` green. Clean follow-up
  verification: `cargo clean` completed (`3707 files, 1.1GiB` removed);
  `cargo build --release --workspace` green; `cargo build --workspace`
  green; `cargo test --workspace` green. Existing Cargo warnings about
  the `wasamo` linkable target / `wasamo-sys` import-library ordering
  were observed.
- **2026-06-03 / T3 local scoped:** `cargo fmt --all -- --check` —
  green after formatting; `cargo test -p wasamo-runtime zstack` —
  green. Covered runtime validate tests
  `zstack_positive_control_validates_direct_children`,
  `zstack_attribute_rejected_at_validate`,
  `zstack_binding_rejected_at_validate`,
  `zstack_child_unknown_alignment_rejected_at_validate`,
  `placement_prop_outside_zstack_child_or_grid_cell_rejected_at_validate`,
  and `validate_rejects_zstack_with_kind_payload`; roundtrip test
  `zstack_emit_then_parse_preserves_direct_children_and_order`; live
  Visual fixtures
  `zstack_rooted_fixture_preserves_live_visual_order_and_clip` and
  `zstack_vstack_root_fixture_pins_production_root_shape`.
- **2026-06-03 / T3 local pre-commit:** `cargo test -p wasamo-runtime`
  — green (included the new ZStack live Visual fixtures, plus existing
  Grid / ScrollView / WrapPanel integration coverage); `cargo build
  --release --workspace` — green; `cargo build --workspace` — green;
  `cargo test --workspace` — green. Existing Cargo warnings about the
  `wasamo` linkable target / `wasamo-sys` import-library ordering were
  observed.
- **2026-06-03 / T3 task-end clean rebuild (post-commit
  `63d6262`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`7195 files, 2.2GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green; `cargo test
  --workspace` — green. Existing Cargo warnings about the `wasamo`
  linkable target and `wasamo-sys` import-library ordering were observed.
- **2026-06-03 / T3 review follow-up local:** pinned empty ZStack as a
  valid runtime shape (`zstack_zero_children_validates`) and strengthened
  the live Visual fixture so the aligned child's `Visual.Offset` proves
  `h-align: end` / `v-align: start` through the runtime
  `WidgetData::ZStack` → `LayoutNode::zstack` boundary. `cargo fmt
  --all -- --check` passed after formatting; `cargo test -p
  wasamo-runtime zstack` — green.
- **2026-06-03 / T3 review follow-up clean rebuild (post-commit
  `395da0f`):** `cargo fmt --all -- --check` — green; `cargo clean`
  completed (`3755 files, 1.2GiB` removed); `cargo build --release
  --workspace` — green; `cargo build --workspace` — green; `cargo test
  --workspace` — green. Existing Cargo warnings about the `wasamo`
  linkable target and `wasamo-sys` import-library ordering were observed.
- **2026-06-03 / T1+T2 cross-task review follow-up clean rebuild
  (post-commit `4616e48`):** a T1/T2 re-review on the test-breadth and
  cross-phase-constraint lenses pinned three deliberate diagnostic/size
  branches that had no test —
  `zstack_child_non_keyword_alignment_value_rejected` (T1 `wasamoc`
  `check_zstack_child_align` non-identifier arm),
  `zstack_handler_rejected_at_validate` (T3 runtime `validate` ZStack
  handler arm), and
  `zstack_fixed_size_measure_reports_declared_extent_not_child_union`
  (T2 `measure_zstack` `Fixed` size arm) — and corrected t1.md item 5 /
  10 and t2.md item 10 to record the placement/alignment constraint as a
  single `carry-forward` with three implementation sites. `cargo fmt
  --all -- --check` — green; `cargo clean` completed (`4935 files,
  1.3GiB` removed); `cargo build --release --workspace` — green
  (39.27s); `cargo build --workspace` — green (35.80s); `cargo test
  --workspace` — green (`wasamoc` 293, `wasamo-runtime` lib 314).
  Existing Cargo warnings about the `wasamo` linkable target and
  `wasamo-sys` import-library ordering were observed. This is the single
  SSOT record for the follow-up verification; the t1 / t2 / t3 retro
  item-3 sections stay scoped to their own original commits.
