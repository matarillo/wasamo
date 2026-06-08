# M3-Phase 6 — implementation handoff

Forward-carry material for the next phase's pre-doc framing, prepared for
the Phase 6 phase-end retrospective (retrospectives.md item 15 / §6.3).
The next planned implementation phase is **M3-Phase 7**; several entries
also target **M4** because Phase 6's gallery lightbox intentionally stopped
at visual proof rather than modal input / DPI / image behavior.

`doc-folded` dispositions are not transcribed as requirements here — the
next phase should read the synced specs directly. This file records only the
confirmed carry-forward constraints, out-of-phase residuals, and
next-phase-relevant learnings.

## Phase 7 handoff targets

- **Control-flow family extension starts from `IrMember` /
  `ControlFlowNode`, not from widgets.** Phase 6 shipped `if` as a structural
  member (`IrMember::ControlFlow(ControlFlowNode::If)`), with lowering,
  textual IR, validation, static load-time presence, and reactive mutation
  all dispatching explicitly between widget members and control-flow members.
  Phase 7 work on `else`, `switch`, or iteration should extend that family
  rather than materialising control flow as a widget. Re-trigger: any grammar,
  textual-IR, loader, validator, roundtrip, or traversal change that adds a
  control-flow form.

- **`BindingTarget::ConditionalSubtree` is the landing point for
  range-style structural targets.** The Phase 6 conditional runtime created
  the first structural binding target that inserts / removes a subtree,
  disposes the old subtree, keeps declared Visual order, and drains fresh
  subtree Effects before the setter returns. A future `ForLoopSubtree` (or
  equivalent range target) should reuse the same ownership questions:
  declared slot identity, insertion / removal atomicity, registry teardown,
  effect drain timing, and failure reporting. Re-trigger: any multi-child
  structural mutation target.

- **Declared-tree / entity-tree identity remains unresolved beyond
  fresh-on-return.** Phase 6 semantics are intentionally
  absent=fresh-on-return with no retention: a conditional subtree that is
  removed and later reinserted is rebuilt as a fresh entity. This is folded
  into `docs/dsl_spec.md` / `docs/architecture.md`. Phase 7 iteration should
  decide whether identity is positional, keyed, or deliberately fresh, and
  should define how that interacts with state, effects, disposal, and Visual
  order. Re-trigger: any `key:` syntax, list diffing, retained subtree state,
  or entity identity model.

- **Placement storage model is a Phase 7 decision before range mutation
  grows.** The current model stores parent-owned placement metadata in
  parallel vectors (notably ZStack placements), so structural mutation must
  update the materialised child list, placement metadata, and live Visual
  sibling order as one invariant. T5's ZStack conditional path needed
  `insert_child_with_zstack_placement` plus placement removal to avoid
  Visual/layout drift. Before implementing a range primitive, decide whether
  to keep the current SoA parallel-vector model, move placement onto child
  records (AoS), or use a keyed metadata map. Re-trigger: any `ForLoopSubtree`
  / range insertion, any new parent-owned per-child metadata kind, or any
  attempt to insert children through a widget-only path. When a placement-like
  surface changes, update the compiler gate, runtime validator, and default /
  alignment extraction together; treating one as "only a diagnostic" is how
  the T1/T2/T3 ZStack placement follow-up found drift.

- **Structural failure observability should be revisited for range
  mutation.** Phase 6 keeps build / insert / remove failures on the
  conditional mutation path as log-only diagnostics after validation. That
  was acceptable for a single-child conditional branch, but multi-child
  range edits can fail partially and may need a stronger runtime error or
  rollback story. Re-trigger: any structural edit that can insert/remove more
  than one child in one mutation.

- **Reactive-drain residuals 1-3 remain deferred.** Phase 6 implemented the
  accepted per-setter drain semantics for conditional subtrees, but the
  DD-M3-P6-005 SM-1 carry-forward remains: cycle detection, ordering ties,
  and fan-out × `MUTATION_CAP` behavior are still future reactive-engine
  work. Re-trigger: any reactive scheduler change, batch update semantics,
  multi-effect ordering guarantee, or structural mutation that can fan out
  to multiple dependent effects.

## M4 / later handoff targets

- **Dynamic Window title / host bindings remain deferred.** Phase 6 fixed the
  static title path and then moved Window host attributes onto
  `IrComponent.host_props` / `host_bindings`, but the Window host catalog
  admits no bindable host attributes this phase. Future work that opens
  dynamic title, dynamic host bindings, base-name validation, or an
  ABI-facing window descriptor must preserve the host-owned-attributes vs
  content-root separation. Do not put `title` / `backdrop` / `theme` back on
  `root.props` / `root.bindings`.

- **Lightbox input remains visual-only in Phase 6.** The owner smoke passed
  the visual proof, but also observed that clicking through gaps between the
  photo / caption / nav can activate the underlying ScrollView buttons while
  the lightbox is open. M4 input / modal design should decide whether an open
  lightbox blocks background hit-testing and focus, and if so implement event
  capture / dispatch suppression for the scrim or modal subtree. This is an
  observed behavior, not just an abstract out-of-scope line.

- **Caption row height depends on current text metrics / logical pixels.**
  T8 changed the lightbox caption Grid row from `32` to `64` because the
  two-line caption was too close to the nav row. The owner accepted the
  corrected geometry, but M4 DPI-awareness, font metrics, text layout, Grid row
  semantics, lightbox copy, or nav layout changes must re-check caption/nav
  separation.

- **DPI remains an M4 runtime-quality axis.** Phase 6 evidence again ran on a
  high-DPI box and observed the known DPI blur as non-failing background
  context. The core DPI handoff is still the Phase 5 handoff and the
  cross-milestone DPI decision; Phase 6 adds only the lightbox-specific
  caption metric dependency above.

- **Image / thumbnail-click lightbox behavior remains out of Phase 6.**
  The Phase 6 gallery uses `Box { aspect: 4:3 }` + `Text` placeholders and
  plain text Buttons (`Open lightbox`, `x`) to prove the structural path.
  Real image widgets, image-button hit-testing, thumbnail-click-to-open, and
  modal focus behavior remain M4 input.

## Closed items — do not carry as open residuals

- **R1 static Window title is closed.** The native title now flows from
  component `title: "..."` through `wasamo_load_ui` -> `window::create` and
  is proven by the Windows-runtime title fixture plus T7/T8 gallery evidence.
  Only dynamic Window-title behavior is deferred.

- **Observation 5 (`scroll_view_layout_integration` access violation) is
  root-caused and remediated.** The cause was cross-apartment reuse of a
  process-global Compositor across libtest's per-test threads. Both
  remediation steps are DONE / committed: the keep-alive apartment helper and
  the owning-thread executor (`run_on_owning_runtime_thread_or_skip`). There is
  no remaining Phase 6 remediation carry-forward. Future mock-free Windows
  integration binaries with two or more Compositor tests should use the helper,
  and M4+ interactive GUI tests may need the same owning thread plus a message
  pump; that is ordinary test-harness input, not an open Phase 6 defect.

## Main learnings carried forward

- **Final-step ownership must split local evidence from phase-branch CI.**
  Phase 6 T9 initially over-deferred local clean rebuild evidence together
  with the phase-branch `workflow_dispatch` run id after a small A12 code
  follow-up. The corrected split is: local clean rebuild is step-owned when
  the step changes code; the GitHub Actions run id / on-CI Windows evidence is
  phase-end-owned after the phase branch runs CI.

- **A T0-frozen task list can become stale inside a phase.** Phase 6 had two
  examples: DD-M3-P6-008 inserted T7b after T7 surfaced the component-root
  host-attribute boundary, and Observation 5's remediation status changed from
  "step 1 deferred" to "both steps done". The mutable phase plan must be
  revised against the current SSOT rather than working around stale frozen
  wording.

- **Semantic migrations now have enough samples to rule the audit pattern.**
  Phase 6 produced the second concrete semantic-migration sample the T4
  retrospective was waiting for: sample 1 was the `IrMember` migration, where
  silent widget-filtering helpers under-counted Box / ScrollView children until
  a traversal call-site audit fixed them; sample 2 was the T7b
  `IrComponent.host_props` / `host_bindings` migration, which recorded a
  call-site audit table and used compiler-enforced construction-site failures.
  Together with the earlier `kind_payload` no-`Default` precedent, the
  threshold for a process rule is met. Carry this to a vision decision record:
  codify semantic-migration audits as a forcing artifact (call-site
  classification table, `rg` queries, and compile-error-forcing construction
  mechanisms where feasible) and prefer those over silent-absorb helpers such
  as filtering iterators. Owning SSOT candidates are
  `process/procedures/retrospectives.md`, workflow / implementation-gate
  surfaces, or their successor process document as decided by the VDR.

## Pointers (doc-folded — not transcribed)

- **ZStack realised semantics** are folded into `docs/dsl_spec.md` §4.13 and
  `docs/architecture.md` §6.9: direct children, parent-owned `h-align` /
  `v-align`, union sizing, document-order z-order, outer-bounds clip, and no
  intermediate Visual.
- **Structural conditional semantics** are folded into `docs/dsl_spec.md`
  §4.14 / textual IR and `docs/architecture.md` §6.6 / §9: `if` is structural
  control flow, not a widget; absent subtrees are rebuilt fresh on return; and
  structural mutation marks layout dirty.
- **Direct conditional members under ScrollView are rejected in Phase 6.** The
  reader-facing rule is folded into `docs/dsl_spec.md` §4.11 / §4.14 and the
  diagnostics are pinned by compiler / loader tests. A future
  conditionally-empty ScrollView design must explicitly reopen this policy.
- **Component host surface separation** is folded into `docs/dsl_spec.md`
  textual IR and `docs/architecture.md`: `host_props` / `host_bindings` live on
  `IrComponent`, not the content root.
- **Observation 5 details** live in
  `docs/notes/verification-environments.md` §Observation 5, including the
  root cause, completed remediation status, evidence, and future helper-use
  note.
