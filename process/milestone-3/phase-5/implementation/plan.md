## Task list

### T0 — Moment 1 document sync

Opens execution after ADR acceptance and records the design draft in
the upstream documents named by the ADR's
[Moment 1 commit set](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2).
T0 closes when this implementation plan lands and all preceding
Moment 1 commits are on the pre-doc branch.

- [x] `process/milestone-3/phase-5/decisions/preamble.md` and
      `dd-m3-p5-001` through `dd-m3-p5-006` flipped to
      `Status: Accepted` (commit `f4f741f`, 2026-05-28).
- [x] `docs/dsl_spec.md` §4.12 Grid layout primitive chapter added
      as design-spec draft with Phase status marker `M3-Phase 5
      design accepted; implementation pending`; §4.4 widget registry
      row added for `Grid` only with `Cell` pointer to §4.12; top
      Status header advanced to include `M3-Phase 5 design accepted
      (implementation pending)`; revision history v1.3 entry
      (commit `5ea39a6`, 2026-05-29).
- [x] `docs/dsl_spec.md` §8.11 Loader validation policy retroactive
      spec-gap fold: extended from Phase 2-only to cover Phase 3
      WrapPanel non-negative attributes (DD-M3-P3-006), Phase 4
      ScrollView single-content-child (DD-M3-P4-006), and Phase 5
      Grid / Cell invariants (DD-M3-P5-006), per owner review on
      2026-05-29 (commit `5ea39a6`).
- [x] `docs/architecture.md` §6.8.7 Phase 5 Grid paragraph added
      (IrNode kind-payload carrier c1; `IrProp.value` strictly
      `IrLiteral`; Cell IR-only; constant-only attributes preserving
      the per-type writer seam; no §6.5 sync-visuals change); top
      Status header advanced to include `M3-Phase 5 design accepted
      (implementation pending)` (commit `5ea39a6`).
- [x] `process/milestone-3/plan.md` Phase 5 row populated
      (Status `in progress`; Progress file and ADR links wired
      to this directory and the ADR set); Phase 6 row Notes record
      the M3-Phase 4 R1 (Window title wiring) owning-phase cross-
      reference per M3-Phase 5 FD-E; Phase 1-4 Tracking-table stale
      links retroactively fixed in the same commit per owner review
      (commit `f2998f2`, 2026-05-29).
- [x] `process/milestone-3/phase-5/implementation/preamble.md`
      and this `plan.md` opened with `status: active` and the FD-C
      / FD-G / constraints §5 step-end / phase-end retrospective
      split represented in T7 from the start (THIS commit).
- [x] `docs/abi_spec.md` deliberately untouched at Moment 1 per
      DD-M3-P5-001 / DD-M3-P5-006: Grid adds no host-facing ABI
      surface; `LayoutError::GridUnboundedStarAxis` is
      runtime-internal and no `WASAMO_LAYOUT_ERROR_*` extension is
      introduced; no `PropertyValue` / `IrType` / `IrLiteral`
      variant added.

### T1 — `wasamoc check`: Grid surface and diagnostics

Discharges the **representative-fixture portion** of ADR
[verification closure evidence item (1)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
(representative fixed + weighted-star + spanning fixtures in unit
tests; all reject / diagnostic cases for the surface). The **gallery
Grid slice positive control** half of item (1) closes at T5 (the
slice's `.ui` compiles cleanly); the **evidence aggregation** for
item (1) closes at T7 alongside the CI gates.

- [x] **Pre-implementation spike** for risk
      [R-A](./preamble.md#technical-risks-planning-time-recon):
      settle the grafting shape of the narrow Grid-specific
      track-list parser path against `parse_widget_decl` /
      `parse_property_bind` (widget-type context routing) and
      decide whether `n*` admits a new lexer token, before opening
      the bullets below. Record the chosen shape in
      [log.md](./log.md). **Settled 2026-05-29** (see log.md
      T1 R-A entry): widget-type routing in `parse_widget_decl`;
      payload-less `Token::Star` (no fused `n*` literal); `1*` vs
      `1 *` distinguished by span adjacency; new `Member::GridTracks`
      AST carrier.
- [ ] Register `Grid` in `wasamoc`'s known widget registry / check
      surface; recognise `Cell` as a Grid-internal IR node kind
      that is rejected outside a `Grid` parent per DD-M3-P5-001 /
      DD-M3-P5-006. (Runtime widget-kind materialisation is T3.)
- [ ] Add the narrow Grid-specific track-list parser path for
      `columns:` / `rows:` per DD-M3-P5-002 (does not open a general
      list / collection grammar).
- [ ] Implement Surface A2 Grid / Cell check-side diagnostics per
      DD-M3-P5-001 / DD-M3-P5-002 / DD-M3-P5-003 / DD-M3-P5-005 /
      DD-M3-P5-006 (track-list shape including reserved-future
      `auto` diagnostic; placement-attribute presence with the
      single-Cell escape clause; Cell single-child; placement / span
      value range; same-cell / overlapping-rectangle conflict;
      unknown-attribute rejection on Grid and Cell; alignment-value
      vocabulary).
- [ ] Add `wasamoc` positive / negative tests covering the
      representative-fixture half of ADR evidence item (1).
- [ ] `wasamoc` emits the Grid carrier c1 (per DD-M3-P5-001) to
      textual IR in a Phase-5 implementation shape so that
      `IrProp.value` stays strictly `IrLiteral` and Grid's
      `columns:` / `rows:` track lists live in the Grid-specific
      kind payload, not in `IrProp` entries. The final textual
      shape feeds the T7 `dsl_spec.md` §8 fold.

### T2 — Layout engine: Grid track-resolution and arrange

Discharges ADR
[verification closure evidence item (2)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence).
Per
[CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules),
the layout engine is pure logic; tests are pure-logic unit tests on
the algorithm's `(input → output)` shape.

- [ ] **Mitigation for risk
      [R-D](./preamble.md#technical-risks-planning-time-recon):**
      settle and record the Grid data shape across `WidgetData::Grid`
      → `LayoutNode` (`Vec<TrackSize>` × 2, `Vec<CellPlacement>`,
      arrange-result cache) before implementing arrange. Decide
      whether to extend the existing flat-struct `LayoutNode`
      pattern (Phase 5 default per ADR) or escalate; record the
      chosen field shape in [log.md](./log.md).
- [ ] Implement Grid measure / arrange in
      `wasamo-runtime/src/layout.rs` per DD-M3-P5-004 (per-axis
      fixed-first + weighted-star track resolution; `f32` prefix
      boundaries; spanning reconciliation; `LayoutError::GridUnboundedStarAxis`;
      reserved no-op slot before star distribution per
      DD-M3-P5-002's `auto` deferral) and DD-M3-P5-005 (per-Cell
      `h-align` / `v-align` with stretch default; layout-side
      outer-bounds-rect invariant; document-order z-order).
- [ ] Add pure-logic tests covering ADR evidence item (2). The
      Visual-side clip-install assertion is T4's responsibility,
      not T2's.
- [ ] All-zero star sum cannot arise after DD-M3-P5-002 /
      DD-M3-P5-006 validate-time rejection; the corresponding
      layout-time defensive panic per DD-M3-P5-004 is retained.
- [ ] Prefer pure free-function extraction. The test-module-only
      mirror struct pattern is reserved per
      [CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules)
      and used only if entanglement with a Win32 / WinRT-bound type
      prevents extraction.

### T3 — IR loader / `validate()` invariant evidence

Discharges ADR
[verification closure evidence item (3)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence).

- [ ] **Pre-implementation spike** for risk
      [R-B](./preamble.md#technical-risks-planning-time-recon):
      settle how the **Grid path in `build_node`** bypasses the
      generic child append loop so IR Cell subtrees flatten into
      `WidgetNode` children + per-Cell `Vec<CellPlacement>` on
      `WidgetData::Grid` (`construct_widget`'s Grid arm only creates
      the Grid widget shell). Settle before opening the bullets
      below; record the chosen shape in [log.md](./log.md).
- [ ] Materialise `Grid` as a runtime widget kind backed by
      `KindPayload::Grid { columns, rows }` per DD-M3-P5-001 carrier
      c1 (`IrProp.value` stays strictly `IrLiteral`). Runtime loader
      parses the Phase-5 textual IR shape produced by `wasamoc` in
      T1 into the kind payload; the shared textual shape feeds the
      T7 `dsl_spec.md` §8 fold.
- [ ] `Cell` IR-loader path reads placement / span / alignment from
      standard `IrProp` entries (Int + Ident literals) and arranges
      each Cell's single content child as Grid's effective layout
      child. `Cell` is **not** registered in the runtime widget
      catalog.
- [ ] Implement runtime `validate()` defense-in-depth per
      DD-M3-P5-006 invariant table (track value range; placement /
      span value range; Cell single-content-child; same-cell /
      overlapping-rectangle conflict; alignment-value vocabulary;
      `Cell`-outside-`Grid`). All violations surface
      `WASAMO_ERR_IR_MALFORMED`; all checks are
      **reject-at-validate**, not clamp-at-arrange (the only
      layout-time gate is T2's `LayoutError::GridUnboundedStarAxis`).
      No `docs/abi_spec.md` change and no new ABI tag.
- [ ] Add pure-logic tests covering ADR evidence item (3).

### T4 — Windows-runtime layout and Visual evidence

Discharges ADR
[verification closure evidence item (4)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
and applies the Phase 4 T6 production-root-shape carry-forward per
[../requirements/constraints.md §1](../requirements/constraints.md#1-integration-test-fixture-parent-shape-は-production-root-shape-を必ずカバーする).
Mock-free Windows-runtime integration tests per
[CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules);
skip-guard inherits the Phase 2 T11 / Phase 3 / Phase 4 pattern
(fires on `0x80070005` from `wasamo_init`) and **fails** rather than
silently skips on a runner that cannot create the Compositor.

- [ ] **Grid-rooted fixture (window-root Fill/Fill path).** `.ui`
      declares a Grid as the root widget with mixed fixed and
      weighted-star tracks containing `Cell { Box { ... } }`
      children in known cells. Drives
      `WidgetNode::run_layout_as_window_root` (Phase 4 T6 entry
      point). Asserts:
      - (a) Grid's resolved rectangle matches the parent allocation.
      - (b) Each Cell's content Visual offset matches the resolved
        cell rectangle origin (with parent-relative offsets per
        [`docs/architecture.md` §6.5](../../../../docs/architecture.md#65-widgetnode-and-visual-layer-sync)).
      - (c) Grid's outer Visual has a **non-null** clip (the
        `InsetClip` from DD-M3-P5-005) — clip presence assertion.
      - (d) Each Cell content Visual has `Visual.Clip = null` —
        clip absence regression guard, symmetric with the Phase 3
        T8 WrapPanel and Phase 4 ScrollView precedents.
- [ ] **`VStack { Grid { ... } }` fixture (production root shape).**
      Same set of assertions (a)-(d) on the inner Grid. Matches the
      current gallery / counter / bool-demo `.ui` production root
      family and guards against the Phase 4 T6 runtime-boundary
      collapse class.
- [ ] **Unbounded star-axis runtime fixture (preferred when
      ergonomic).** `.ui` places a Grid with at least one star track
      inside a parent whose corresponding axis is unbounded
      (synthesisable by embedding in an intrinsic-measure context).
      Assert layout pass returns
      `Err(LayoutError::GridUnboundedStarAxis)`. If no ergonomic
      IR-level fixture exists, **downgrade** this case to pure-logic
      coverage in T2 and record the decision under the Decisions
      log in [log.md](./log.md).
- [ ] Verify the skip-guard fires (i.e. test FAILS, not skips) on
      an environment where `wasamo_init` returns `0x80070005`,
      before landing T4 — per
      [CLAUDE.md §Testing rules](../../../../CLAUDE.md#testing-rules).
      Local "passed without skip" does not prove the guard.

### T5 — End-to-end gallery `.ui` and assistant-side build / launch

Discharges the **assistant-automated portion** of ADR
[verification closure evidence item (5)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
and the **gallery positive-control portion** of item (1) — the slice
`.ui` compiles cleanly through `wasamoc check` (T1 closed the
representative-fixture / diagnostic halves of item (1) earlier; the
gallery slice closes the positive-control half here). The visible-
correctness portion of item (5) (owner-manual GUI smoke) is owned by
T6 per FD-I; T5 stops at build / launch success.

- [ ] Grow `examples/gallery/gallery.ui` **additively** with a
      sibling slice containing a Grid composition matching the FD-H
      minimum visible-proof shape: a 3-row × 3-column Grid with at
      least five Cells —
      - one header `Cell` spanning all three columns,
      - three middle-row `Cell`s in separate columns,
      - one footer `Cell` spanning all three columns,
      - mixed fixed + at least one weighted-star track per axis,
      - exercising column-span in the real `.ui` (row-span discharged
        by T2-T4 per FD-C).
      The Phase 3 standalone WrapPanel slice and Phase 4 ScrollView
      slice stay untouched.
- [ ] Cell content uses `Box { fill: ... }` + `Text { text: ... }`
      per the Phase 2 DD-M3-P2-006 placeholder pattern. No Image
      widget (deferred to M4 per Phase 2).
- [ ] Include at least one Cell whose content intentionally overflows
      its resolved rectangle (e.g. fixed-track sum exceeding Grid's
      bounded extent, or a Cell with content larger than its cell)
      so the **Grid outer-bounds clip is visibly observable** in T6
      smoke. Without this, the T6 clip-visibility observation point
      cannot be exercised on the live binary.
- [ ] Build and run `examples/gallery-rust/`. Record `Start-Process`
      launch success by the assistant. C / Zig gallery hosts remain
      out of Phase 5 scope.

### T6 — Owner-manual GUI smoke and any visible-correctness fix

Discharges the **visible-correctness portion** of ADR
[verification closure evidence item (5)](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
and the A11 gallery-proof owner-acceptance half. This step exists so
visible smoke is verified — and fixed if it fails — **before** any
phase-close mechanical work (spec / plan status flips) lands in T7,
matching the Phase 4 T5 / T6 split rationale.

- [ ] Owner runs `target/release/gallery-rust.exe` (or builds-and-
      runs `cargo run -p gallery-rust` if the T5 binary is no longer
      on disk). Observation points:
      - column tracks render at the declared widths (fixed and
        proportional);
      - the spanning header `Cell` spans all three columns;
      - the three middle-row `Cell`s occupy separate columns;
      - the spanning footer `Cell` spans all three columns;
      - Grid outer-bounds clip is visible when a Cell's content
        intentionally overflows;
      - document-order paint order is observed when overlapping
        content occurs.
- [ ] Owner explicitly accepts the smoke result, or records a fail
      observation note. **If smoke fails:** the implementation fix
      lands additively on the T6 branch (new commits); the smoke
      checklist above is re-run to green before T6 closes. Fix scope
      stays inside the Phase 5 ADR (`process/milestone-3/phase-5/decisions/`)
      / `docs/dsl_spec.md` §4.12 / `docs/architecture.md` §6.8.7;
      any fix requiring a normative spec change escalates to T7
      Moment 2 (or, if unsuitable for Moment 2, a mid-ADR addendum).
      Fix iterations stay inside T6 until the smoke checklist is
      green.
- [ ] T6 step-end retrospective recorded at
      `process/milestone-3/phase-5/retrospectives/t6.md`
      (retrospectives.md checklist items 1-11).

### T7 — Phase-end gates and Moment 2 re-sync

Discharges the m3-plan phase-end criteria for Phase 5 and the
Moment 2 doc-set listed in the
[ADR preamble Moment 2 section](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2).
Per
[../requirements/constraints.md §5](../requirements/constraints.md#5-phase-最終-step-の-retrospective--progress-checklist-は-step-end-と-phase-end-を分割する)
(FD-G), the step-end retrospective is **owned by T7**; the phase-end
retrospective is **NOT owned by T7** and stays unchecked at T7 close
— see the two retrospective bullets at the end of this list.

- [ ] `cargo fmt --all -- --check` green locally and on CI.
- [ ] `cargo build --release --workspace` green locally and on CI.
- [ ] `cargo test --workspace` green locally and on CI.
- [ ] Windows-only integration evidence green on CI (skip-guard
      verified per T4).
- [ ] `docs/dsl_spec.md` §4.12 Phase status marker flips to
      `M3-Phase 5 closed; implementation-synced`; document-level
      Status header updated to reflect Phase 5 closed
      (implementation-synced); revision history v1.4 entry recording
      the Moment 2 close, any impl/spec divergence corrections, and
      any earlier-phase retroactive spec-gap fold surfaced during
      re-sync (owner-confirmation required per the retroactive
      spec-gap minimum-fold pattern).
- [ ] `docs/dsl_spec.md` §8 textual IR grammar / AST shape folded
      to match the Phase 5 implementation shape of Grid carrier c1
      (`KindPayload::Grid { columns, rows }`) as landed by T1
      (wasamoc emit) and T3 (runtime loader parse). The §8 fold was
      deferred from Moment 1 because the carrier's textual shape
      pins at implementation time; per the retroactive spec-gap
      minimum-fold pattern this lands in the same revision-history
      v1.4 entry as the §4.12 status flip.
- [ ] `docs/architecture.md` top Status flips to include
      `M3-Phase 5 complete`; any implementation-divergent paragraphs
      in §6.8.7 re-synced to actual landed shape.
- [ ] `process/milestone-3/plan.md` Phase 5 row Status flips to
      `complete`.
- [ ] `docs/abi_spec.md` re-confirmed untouched (Grid added no
      host-facing ABI surface; `LayoutError::GridUnboundedStarAxis`
      remained host-internal). Touch only if a Moment 2 surprise
      forced an ABI surface change, in which case escalate to ADR
      revision per the ADR preamble's three retrospectives.md
      §phase-sync ADR-touch cases.
- [ ] `process/milestone-3/phase-5/decisions/preamble.md` /
      `dd-m3-p5-*.md` touched **only** if one of the three
      retrospectives.md §phase-sync ADR-touch cases applies
      (AC discharged-vs-impl divergence; out-of-phase residual
      cross-reference; thesis-level finding). Otherwise the ADR
      set stays at its Moment 1 Accepted state.
- [ ] Step retro `phase-sync` items from T1-T7 close into
      `doc-folded` / `carry-forward` / `local-only` —
      **no open `phase-sync` items survive past phase close**.
- [ ] [log.md](./log.md) records the phase-close evidence pointer,
      CI run id, implementation summary distilled from T1-T6, and
      any final post-merge distillation.
- [ ] Carry-forward inputs to the next phase's pre-doc recorded
      under [handoff.md](./handoff.md) (covering at minimum: any
      residual surfaced during T2-T6; any out-of-phase R found
      during gallery smoke; Phase 6 inputs including the R1 Window-
      title wiring assignment already cross-referenced in
      `m3-plan.md` Phase 6 row Notes).
- [ ] Front-matter `status` flips from `active` to `closing` on this
      file in the same commit that flips T7's "all gates green"
      checkbox above. No further task checkboxes are added after
      this point.
- [ ] **T7 step-end retrospective recorded** at
      `process/milestone-3/phase-5/retrospectives/t7.md`
      (retrospectives.md checklist items 1-11; step → phase merge
      gate; **owned by T7**, this is a T7 deliverable).
- [ ] **Phase-end retrospective recorded** at
      `process/milestone-3/phase-5/retrospectives/phase-end.md`
      (retrospectives.md checklist items 12-18; phase → main merge
      gate; **NOT owned by T7**, performed on the phase branch
      after T7 merges in by a separate retro commit per
      [../requirements/constraints.md §5](../requirements/constraints.md#5-phase-最終-step-の-retrospective--progress-checklist-は-step-end-と-phase-end-を分割する)).
      **This bullet stays `[ ]` at T7 close**; the phase-end retro
      commit on the phase branch flips it. Phase 5 phase-end retro
      additionally decides whether to promote constraints §5 into
      `process/procedures/retrospectives.md` as a project-wide rule
      based on the lived experience of running this split from the
      start.
