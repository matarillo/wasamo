---
phase: M3-Phase 4
title: ScrollView primitive (minimal)
status: active
adr: docs/decisions/m3-phase-4-scroll-view.md
plan: docs/plans/m3-plan.md
opened: 2026-05-25
---

# M3-Phase 4 — ScrollView primitive (minimal): Progress

This is the live task list and execution log for M3-Phase 4. The
design decisions are frozen in
[m3-phase-4-scroll-view.md](../../decisions/m3-phase-4-scroll-view.md);
this file is mutable per
[plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle).

Phase 4 satisfies A5 by adding a vertical-only `ScrollView` with
inner unbounded measure, viewport clip, and `offset-y` content offset
binding. Scrollbar widget, wheel handler, drag, in-out offset
write-back, imperative scroll commands, horizontal/bidirectional
scrolling, and the general typed-`i32` writer pair are M4 or later.

The automated A5 evidence set is verification items 1 through 4 in
the ADR's
[Phase 4 verification closure](../../decisions/m3-phase-4-scroll-view.md#phase-4-verification-closure-what-counts-as-a5-evidence).
The phase-close / A11 gallery proof is item 5, including owner-manual
GUI smoke.

## Task list

### T0 — Moment 1 document sync

Opens execution after ADR acceptance and records the design draft in
the upstream documents named by the ADR's Moment 1 queue.

- [x] `docs/dsl_spec.md` adds the §4.11 ScrollView design draft and
      widget registry row.
- [x] `docs/architecture.md` records the ScrollView IR / binding /
      layout / Visual-layer architecture draft without flipping the
      top-level Status to Phase 4 complete.
- [x] `docs/plans/m3-plan.md` marks M3-Phase 4 in progress with
      links to this progress file and the accepted ADR.
- [x] `docs/plans/progress/m3-phase-4-progress.md` opens with
      `status: active`.
- [x] `docs/abi_spec.md` remains untouched at Moment 1; the new
      `LayoutError::ScrollViewUnboundedAxis` is internal and no ABI
      tag is added.

### T1 — `wasamoc check`: ScrollView surface and diagnostics

Discharges ADR verification closure **evidence item 1**.

- [x] Register `ScrollView` as a known widget type while preserving
      the generic parser shape.
- [x] Enforce exactly one child at compile time; reject zero-child and
      multi-child ScrollView declarations.
- [x] Accept `offset-y: <i32>` literals and `offset-y: scroll_y`
      (bare state identifier per dsl_spec §4.3) when `scroll_y` is
      declared as `i32` in `state`.
- [x] Reject non-integer literals and bindings to undeclared, `bool`,
      or `string` state.
- [x] Reject `bind offset-y:` / `in-out offset-y:`-style writable
      surfaces; Phase 4 binding is read-only.
- [x] Reject unknown ScrollView attributes, including `viewport-*`,
      `scroll-axis`, and `padding`.
- [x] Keep the gallery sub-screen `.ui` compiling cleanly as the
      positive control.

### T2 — Layout engine: ScrollView measure-arrange

Discharges ADR verification closure **evidence item 2**.

- [x] Implement pure-data ScrollView measure-arrange in
      `wasamo-runtime/src/layout.rs`.
- [x] Measure content with the viewport width and an unbounded
      vertical constraint.
- [x] Return ScrollView outer size equal to the viewport size,
      independent of content size.
- [x] Clamp `offset-y` across negative, zero, in-range, max, and
      larger-than-max values.
- [x] Cover content smaller than viewport, equal to viewport, and
      larger than viewport.
- [x] Raise `LayoutError::ScrollViewUnboundedAxis` for an unbounded
      vertical parent.
- [x] Preserve the `i32` to `f32` rounding contract with no
      pixel-snapping.

### T3 — IR loader / `validate()` invariant evidence

Discharges ADR verification closure **evidence item 3**.

- [x] Materialize `ScrollView` as a runtime widget kind with exactly
      one content child and an `offset-y` field.
- [x] Runtime `validate()` rejects 0-child and >1-child ScrollView IR
      with `WASAMO_ERR_IR_MALFORMED`.
- [x] Runtime `validate()` accepts negative and very large `offset-y`
      values; clamping remains a layout responsibility.
- [x] Keep `LayoutError::ScrollViewUnboundedAxis` internal; no
      `docs/abi_spec.md` change and no new ABI tag.

### T4 — Windows-runtime layout and Visual evidence, including R2 closure

Discharges ADR verification closure **evidence item 4** and closes the
Phase 3 R2 test-coverage residual inside Phase 4.

- [x] Add a mock-free Windows-runtime integration fixture with a
      bounded ScrollView viewport and overflowing content.
- [x] Assert ScrollView's resolved rectangle equals the expected
      viewport dimensions.
- [x] Assert the ScrollView-owned intermediate content Visual offset
      is `(0, 0, 0)` at `scroll_y = 0`.
- [x] Assert bound-state updates move the intermediate content Visual
      to `(0, -offset_y, 0)` after clamp.
- [x] Assert negative and larger-than-max states clamp to 0 and max.
- [x] Assert the outer ScrollView Visual has a non-null clip.
- [x] Assert the intermediate content Visual and child widget Visual
      have no clip.
- [x] Assert three-level nested root-relative position math for
      parent -> ScrollView Visual -> ScrollView-owned intermediate
      content Visual -> thumbnail Visual, closing R2.
- [ ] Exercise the unbounded scroll-axis runtime fixture when an
      ergonomic `.ui` / IR-level fixture exists; assert layout returns
      `Err(LayoutError::ScrollViewUnboundedAxis)`.
- [x] If no ergonomic integration fixture exists, explicitly downgrade
      the unbounded-parent case to pure-logic coverage in T2 and record
      that decision here.
- [x] Preserve the CI-gated Compositor skip/fail discipline inherited
      from Phase 2 / Phase 3.

### T5 — End-to-end gallery `.ui` and assistant-side build / launch

Discharges the **assistant-automated portion** of ADR verification
closure **evidence item 5**. The visible-correctness portion of
item 5 (owner-manual GUI smoke) is split into a dedicated step at
T6 — see the rationale in the Decisions log entry **"T5/T6 split for
owner-manual GUI smoke (2026-05-25)"**.

- [x] Grow `examples/gallery/gallery.ui` additively with a sibling
      `ScrollView { WrapPanel { Box × 30–40 } }` slice; the Phase 3
      standalone WrapPanel slice stays untouched.
- [x] Add programmatic scroll controls that mutate `state.scroll_y` by
      fixed increments.
- [x] Build and run `examples/gallery-rust/`.
- [x] Record `Start-Process` launch success by the assistant.
- [x] C / Zig gallery hosts remain out of Phase 4 scope.

(The original T5 bullet "Leave visual correctness as owner-manual GUI
smoke" has moved to T6 — see Decisions log "T5/T6 split for
owner-manual GUI smoke (2026-05-25)".)

### T6 — Owner-manual GUI smoke and any visible-correctness fix

Discharges the **visible-correctness portion** of ADR verification
closure **evidence item 5** and the A11 gallery proof's owner-acceptance
half. This step exists so that visible smoke is verified — and fixed
if it fails — **before** any phase-close mechanical work (spec / plan
status flips) lands in T7. See the Decisions log entry **"T5/T6 split
for owner-manual GUI smoke (2026-05-25)"** for the rationale.

- [x] Owner runs (or builds-and-runs) `target/release/gallery-rust.exe`
      (or the `debug/` variant); see `examples/gallery-rust/` README
      / `cargo run -p gallery-rust` if the T5 binary is no longer on
      disk (clean / fresh checkout). Owner observes: viewport clips
      sharply; +100 / −100 Buttons move content; clipped content is
      hidden; off-viewport thumbnails enter view as `scroll_y`
      progresses.
- [x] Owner explicitly accepts the smoke result, or records a fail
      observation note (per [human-visible GUI smoke](../../notes/human-visible-smoke.md)).
      Owner accepted on 2026-05-25 after the re-smoke pass on the
      rebuilt binary discharged all four observation points; smoke
      evidence at
      [docs/references/m3-phase-4/](../../references/m3-phase-4/)
      (`t6-gallery-smoke-scroll-y-0.png`,
      `t6-gallery-smoke-scroll-y-100.png`,
      `t6-gallery-smoke-scroll-y-800.png`,
      `t6-gallery-smoke-scroll-y-back-to-0.png`).
- [x] **If smoke fails:** implementation fix lands additively on the
      T6 branch (new commits); the smoke checklist above is re-run to
      green before T6 closes. Fix scope stays inside the Phase 4 ADR
      (`docs/decisions/m3-phase-4-scroll-view.md`) / dsl_spec §4.11 /
      architecture.md §6.5; any fix requiring a normative spec change
      escalates to T7 Moment 2 (or, if unsuitable for Moment 2, a
      mid-phase ADR addendum). Fix iterations stay inside T6 until the
      smoke checklist is green. Fix landed in commit `ed78d6c
      fix(wasamo-runtime): force window-root WidgetNode to Fill/Fill
      (M3-Phase 4 T6)`; no normative spec touch was required.
- [x] First owner-manual smoke pass (2026-05-25) recorded as **failure
      mode A** — see Decisions log "T6 smoke failure mode A disposition
      (2026-05-25)". Fix bundle selected: (a) new
      `WidgetNode::run_layout_as_window_root` that forces the root
      LayoutNode's sizing constraints to `Fill/Fill` before delegating
      to `layout::run_layout`, with `window.rs`'s `WM_SIZE` handler and
      `set_root` initial layout switched to call it; the plain
      `WidgetNode::run_layout` retains its previous semantics so
      existing integration tests that drive `WidgetNode`s as non-window
      roots stay green; (b) `examples/gallery/gallery.ui` ScrollView
      inner WrapPanel `item-cross-size: 64 → 128` so content_h exceeds
      viewport_h across the realistic window-size range and `+100/-100`
      motion is visible; (c) pure-logic pinning unit test in
      `wasamo-runtime/src/layout.rs::tests` documenting the
      Shrink-VStack-root + Fill-child collapse alongside the override
      behaviour; (d) mock-free runtime integration test in
      `wasamo-runtime/tests/scroll_view_layout_integration.rs` rooted
      at a VStack (gallery-shaped fixture) that drives
      `run_layout_as_window_root` and asserts ScrollView outer Visual
      height > 0 and intermediate Visual Y offset is negative at
      non-zero `scroll_y`. (c) and (d) together pin both the
      layout-engine invariant and the runtime-boundary override so
      future contributors do not regress either layer.
- [x] Re-run owner-manual smoke on the rebuilt `gallery-rust.exe`;
      owner accepts (all 4 observation points green) or records a
      further fail observation. Iterate until green. Re-run on
      2026-05-25 returned all observation points green; window close
      (Alt+F4 / ×) crash-free.
- [x] T6 step-end retrospective recorded under
      `docs/notes/m3-phase-4/`
      ([t6-step-end-retrospective.md](../../notes/m3-phase-4/t6-step-end-retrospective.md)).

### T7 — Phase-end gates and Moment 2 re-sync

Discharges the m3-plan phase-end criteria for Phase 4. Renumbered
from the original T6 when the T5/T6 split landed; checklist content
extends the original list with Moment 2 sync items surfaced by T4
/ T5 / T6 retrospectives' Item 10 (phase-sync classifications) and
T6 retrospective Follow-Up #1.

- [ ] `cargo fmt --all -- --check` green.
- [ ] `cargo build --release --workspace` green locally and on CI.
- [ ] `cargo test --workspace` green locally and on CI.
- [ ] Windows-only integration evidence green on CI.
- [ ] `docs/dsl_spec.md` §4.11 Phase status marker flips to
      `M3-Phase 4 closed; implementation-synced`.
- [ ] `docs/architecture.md` top Status flips to include M3-Phase 4
      complete, with implementation divergences reconciled.
- [ ] `docs/architecture.md` §6 (general layout) gains a single-sentence
      runtime-boundary invariant covering "window-root `WidgetNode` is
      sized to the client rect regardless of its declared width/height
      constraints, enacted by `WidgetNode::run_layout_as_window_root`
      and called by `window.rs`'s `WM_SIZE` handler and `set_root`
      initial layout" (T6 retrospective Item 10 (1) `phase-sync`; T6
      Follow-Up #1).
- [ ] `docs/architecture.md` §6.5 (WidgetNode / Visual Layer sync)
      gains explicit prose that the ScrollView intermediate Visual's
      scroll translation `(0, -offset_y, 0)` also shifts each content
      child's `sync_visuals()` `parent_abs_offset` by `(0, -offset_y)`
      so root-relative LayoutNode offsets convert correctly under the
      intermediate Visual (T4 retrospective Item 10, configured-class
      corrected to `phase-sync` in T5 retrospective Follow-Up; T5
      Follow-Up #2 / T6 Follow-Up #2).
- [ ] `docs/dsl_spec.md` §4.9 Box examples switch from the
      `;`-separated single-line form (`Box { aspect: <r>; fill: <c>;
      Text { ... } }` at the four extant sites) to the parser-
      accepted multi-line member-per-line form, plus a short adjacent
      note clarifying that `;` is currently a statement terminator
      inside handler blocks (§4.5 / §3 grammar) and that **accepting
      `;` as an optional member separator remains a post-Phase-4
      grammar open question** so this formatting does not foreclose
      that extension. Document version bumps to `1.2`; revision
      history wording is "parser-accepted examples; semicolon member
      separator left as post-Phase-4 open question" rather than
      "member separator confirmed as newline" (owner-confirmed
      framing, T5 副次学び #3 / T5 Follow-Up #2; T6 Follow-Up
      carry-over).
- [ ] `docs/plans/m3-plan.md` Phase 4 row flips to complete.
- [ ] **Out-of-phase residual R1 registration: gallery host Window
      title wiring.** Phase 4 smoke recorded
      `MainWindowTitle = "Wasamo"` (framework default) while
      `examples/gallery/gallery.ui` declares `title: "Gallery"`.
      Owner-confirmed framing: **owner intent is that `.ui` `title:`
      must drive the actual native window title**; this is an
      **M3 residual, not an M4 theming/chrome handoff**. Resolution
      condition is "the runtime/ABI host path applies the
      component-level `title` to the native window", **not** "title
      attribute declared unsupported". Register under
      §Out-of-phase residuals with the resolution gates: owning M3
      phase to be assigned during **M3-Phase 5 pre-doc input
      distillation**; implementation must land **no later than
      M3-Phase 8 Gallery E2E close**; Phase 6 (ZStack + conditional
      rendering) is a natural candidate for absorbing the small
      host/window-metadata wiring task (T5 Follow-Up; T6 Follow-Up
      #5).
- [ ] Carry-forward inputs to the next phase's pre-doc are recorded
      under `docs/notes/m3-phase-5/predoc-inputs.md` covering: (a)
      the "integration test fixture parent shape must cover at least
      one production root shape" rule (T6 retrospective Item 10 (2)
      `carry-forward`); (b) the "non-root Shrink container with Fill
      child" design-space note (T6 retrospective Item 10 (3)
      `carry-forward`); (c) the M4 handoff item "`scroll_y` Signal
      drift resolves via `in-out offset-y` write-back" (T6
      retrospective Follow-Up #4; this one is genuinely M4 because
      the writer-direction `in-out` surface is M4 scope); (d) the
      **R1 Window-title wiring owning-phase assignment** that
      M3-Phase 5 pre-doc must complete (Q2 disposition; owner
      intent: M3 residual, not M4 handoff). Per retrospectives.md
      §Main Learning / 設計制約の前送り, this file must be written
      before T7's merge gate.
- [ ] This progress file records the phase-close evidence, CI pointer,
      implementation summary, and lifecycle transition.
- [ ] T7 step-end retrospective recorded at
      `docs/notes/m3-phase-4/t7-step-end-retrospective.md`
      (retrospectives.md checklist items 1-11; step → phase merge
      gate; **owned by T7**).
- [ ] Phase-end retrospective recorded at
      `docs/notes/m3-phase-4/phase-end-retrospective.md`
      (retrospectives.md checklist items 12-18; **phase → main merge
      gate, performed on `feat/m3-phase-4` after T7 merges in**).
      This bullet stays `[ ]` at T7 step-close and is flipped by the
      separate phase-end retro commit; per Q3 disposition (2026-05-25)
      a reviewer reading T7 close as "phase-end retro outstanding"
      should resolve the apparent gap against this explicit ownership
      split.

## Decisions log

- **T4 — unbounded-scroll-axis fixture disposition (2026-05-25).** ADR
  Phase 4 verification closure item 4's last sub-bullet permits
  downgrading the unbounded-scroll-axis runtime fixture to pure-logic
  coverage when no ergonomic `.ui` / IR-level fixture can synthesise
  the case. Every Phase 4 widget catalog parent (VStack / HStack /
  Box / WrapPanel / window root) passes a finite scroll-axis cell to
  its ScrollView child at arrange time — VStack / HStack arrange-time
  `h` derives from finite parent allocation (Fixed / Fill against
  finite window, Shrink against finite content); Box / WrapPanel
  arrange children with their resolved finite rect; the window root
  passes `window_h`. There is therefore no `.ui` that reaches
  `arrange_scroll_view` with `h = f32::INFINITY`. The pure-logic
  `scroll_view_unbounded_scroll_axis_parent_is_runtime_error` test in
  `wasamo-runtime::layout::tests` (T2) pins the
  `LayoutError::ScrollViewUnboundedAxis` branch; the integration-test
  bullet is intentionally left unchecked above so the disposition is
  visible at-a-glance. Revisit if a future phase introduces a parent
  layout that legitimately passes unbounded scroll-axis input
  downstream (none anticipated through Phase 8).

- **T5/T6 split for owner-manual GUI smoke (2026-05-25).** The
  original Task list bundled owner-manual GUI smoke as the last
  `[ ]` of T5 and treated phase-end mechanical close as T6. T5 was
  closed by the assistant after the `.ui` + Build + Start-Process
  bullets had landed, which left the smoke bullet with **no step
  that actually owns its execution** — the smoke could only happen
  after owner review of T5, but T5 was being merged. The original
  T6 (phase-end / Moment 2 re-sync) made the smoke verification
  implicit via [retrospectives.md item 17](../../notes/retrospectives.md#phase-end-固有-merge--main),
  which would have meant interleaving spec / plan marker flips with
  smoke verification in a single step. If smoke had failed mid-T6,
  the phase branch would carry half-flipped spec markers while a
  fix step is opened — an avoidable rollback surface.

  Resolution: split the original Task list into three steps. T5
  retains the assistant-automated bullets; **new T6** is a dedicated
  owner-manual GUI smoke step that absorbs any fix iterations
  inside its own branch; **T7 (= former T6)** runs the mechanical
  phase-close gates only once visible correctness is owner-accepted.
  Smoke gate is now explicit in the progress file rather than
  inherited from `retrospectives.md` item 17. ADR (`docs/decisions/
  m3-phase-4-scroll-view.md`) numbering is not affected — the ADR
  refers to verification closure **evidence item 5**, which now
  maps onto T5 + T6 jointly; m3-plan.md Phase 4 row does not list
  T-numbers and needs no edit.

  Phase 2 / Phase 3 precedent was to bundle smoke into the
  phase-end step. Phase 4 deviates because the intermediate Visual
  + `InsetClip{0,0,0,0}` + `Visual.Offset = (0, -applied_y, 0)`
  shift + `sync_visuals` `parent_abs_offset` shift compose the
  largest Visual-layer change of M3, raising the prior on
  pixel-level regressions that integration tests cannot catch (e.g.
  clip sharpness, repaint ordering, peripheral wiring). The split
  is local to Phase 4; not a project-wide convention change.

- **T6 smoke failure mode A disposition (2026-05-25).** First owner-
  manual smoke pass on the T5 release artifacts (`target/release/
  gallery-rust.exe`, built 2026-05-25 19:49) reported (1) the ScrollView
  region under the Button row drew completely empty at `scroll_y = 0`,
  and (2) pressing "Scroll down (+100)" five times produced **no
  visible change** in the rendered window. Root cause: `gallery.ui`'s
  component root is a VStack with default `width: Fill, height:
  Shrink`; `layout::measure_vstack` for `Shrink` height excludes Fill
  children's desired_h (the standard Fill-collapses-under-Shrink-parent
  convention, intentionally pinned by
  `layout::tests::degenerate_fill_in_shrink_parent_clamps_to_zero`),
  so the root VStack resolved to `desired_h ≈ 312`
  (= Phase 3 WrapPanel + Button×2 + spacing/padding) regardless of
  window height. `arrange_vstack`'s `remaining` then clamped to `0`
  and the Fill ScrollView child received `child_h = 0`, producing an
  outer Visual sized `(w, 0)` whose `InsetClip{0,0,0,0}` auto-tracked
  to a zero-height clip rect → content never visible. The writer chain
  was equally inert by composition: `max_offset = max(0, content_h −
  0) = content_h` so `applied_offset_y = clamp(scroll_y, 0, content_h)`
  did update, but with viewport height 0 the resulting
  `Visual.Offset = (0, -applied_y, 0)` had nothing on-screen to move.

  T4's `scroll_view_layout_integration.rs` integration fixture
  (`FIXTURE_SRC`) roots a ScrollView directly under the `inherits
  Window` component (`width: Fill, height: Fill` from
  `WidgetNode::scroll_view`), bypassing the VStack-root path that
  production `.ui` (counter, bool-demo, gallery) all use. The T4
  Decisions log note "T4 fixture WrapPanel substitution vs ADR primary
  VStack (2026-05-25, commit `57f2366`)" already flagged the layout
  divergence between fixture parent and ADR's primary VStack-rooted
  envelope; this T6 finding is the visible-correctness consequence of
  that uncovered path.

  Disposition: in-scope T6 fix, **no normative spec change**.

  - **Implementation fix (a):** introduce a dedicated
    `WidgetNode::run_layout_as_window_root` that overrides the root
    LayoutNode's `width`/`height` to `SizeConstraint::Fill` before
    delegating to `layout::run_layout`; route `window.rs`'s `WM_SIZE`
    handler and `set_root` initial layout to call it. The plain
    `WidgetNode::run_layout` retains its current semantics (so
    existing mock-free integration tests like
    `wrap_panel_layout_integration.rs`, which drive `WidgetNode`s
    directly to exercise declared sizing constraints, continue to
    pass), and the pure-logic `layout::run_layout` —
    `degenerate_fill_in_shrink_parent_clamps_to_zero` and the broader
    Shrink/Fill convention — also stays untouched. The override
    formalises the implicit "Window client rect determines the root
    viewport" contract that Phase 2 / Phase 3 implicitly relied on
    but that no `.ui` had previously exercised with a Fill child at
    the root container.
  - **Fixture-visibility adjustment (b):** `examples/gallery/gallery.
    ui` bumps the ScrollView inner WrapPanel's `item-cross-size` from
    `64` to `128`. With viewport height ≈ `window_h − 312` and Box ×
    32, `item-cross-size: 64` produces `content_h` smaller than the
    viewport across 800×600 – 1280×900 windows (max_offset = 0,
    +100/-100 motion invisible even after fix (a)). Bumping to 128
    yields `content_h > viewport_h` across the same range so scroll
    motion is visually observable. ADR's "Box × 30–40" range and
    Phase 3 standalone WrapPanel slice both stay untouched.
  - **Pure-logic pinning test (c):** new `#[test]` in
    `wasamo-runtime/src/layout.rs::tests` re-states
    `degenerate_fill_in_shrink_parent_clamps_to_zero`'s outcome for a
    gallery-shaped VStack root (mixed Shrink-height + Fixed-height +
    Fill-height children including a ScrollView) and asserts that
    pre-setting the same root to `Fill` height before `run_layout`
    flips the Fill child's allocated height to non-zero. Captures
    both the basal trap and the override behaviour at the layer where
    `WidgetNode::run_layout`'s policy is enacted.
  - **Runtime integration test (d):** new `#[test]` in
    `wasamo-runtime/tests/scroll_view_layout_integration.rs` that
    lowers a VStack-rooted gallery-shaped `.ui` (Button +
    ScrollView) through `wasamoc` and `build_widget_tree`, then
    drives `WidgetNode::run_layout_as_window_root(200.0, 200.0)`
    (the same WinRT-bound entry point `window.rs` uses on `WM_SIZE`).
    Asserts the ScrollView outer Visual height > 0 at
    `scroll_y = 0` (the regression gate for fix (a)) and that the
    intermediate content Visual's `Y` offset is negative at
    `scroll_y = 100` (writer chain end-to-end on the production path,
    not just the ScrollView-root fixture path). Reuses the existing
    skip-guard discipline (fail on GitHub Actions, skip on
    `0x80070005` locally).

  T7 carry-over: `architecture.md §6` (general layout, not §6.5
  ScrollView) candidate addition documenting **"the window-root
  WidgetNode is sized to the client rect regardless of its declared
  width/height constraints"** as a single-sentence runtime-boundary
  invariant. The latent layout question "non-root VStack with mixed
  Shrink + Fill children" stays out of Phase 4 scope: it is the same
  collapse convention `degenerate_fill_in_shrink_parent_clamps_to_zero`
  pins, surfaces only when a future widget catalog ships a
  non-window-root container that wants Fill-child sizing, and is
  classified `carry-forward` to the next phase pre-doc input as a
  design-topic candidate rather than a Phase 4 bug.

  No screenshot automation is required for T6 close; owner-side
  visible smoke + the new integration test together discharge
  evidence item 5's visible-correctness half.

- **T7 Moment 2 dispositions for follow-up bullets (2026-05-25).**
  Three owner judgments were requested at T7 open and resolved before
  any spec or carry-forward editing:

  - **Q1 — dsl_spec.md §4.9 Box example notation.** The
    `;`-separated single-line `Box { aspect: <r>; fill: <c>; ... }`
    examples (lines 501 / 507 / 508 / 752) are switched to the
    parser-accepted multi-line form, but the revision history wording
    is **"parser-accepted examples; semicolon member separator left
    as post-Phase-4 open question"** and a short adjacent note
    explicitly preserves the post-Phase-4 grammar option of
    admitting `;` as an optional member separator. The owner-confirmed
    framing rejects a strict-A (purely "newline confirmed as the
    member separator") reading because it would foreclose a future
    grammar extension the owner wants to keep open. Document version
    bumps to `1.2`.
  - **Q2 — Window title wiring disposition.** The
    `MainWindowTitle = "Wasamo"` vs `.ui` `title: "Gallery"`
    divergence is registered under §Out-of-phase residuals as **R1**
    with a fixed resolution condition: **"the runtime/ABI host path
    applies the component-level `title` to the native window"**, not
    "title attribute declared unsupported". The owner-confirmed
    framing classifies this as an **M3 residual, not an M4
    theming/chrome handoff**. Gate structure: M3-Phase 5 pre-doc
    input distillation assigns the owning M3 phase; implementation
    must land **no later than M3-Phase 8 Gallery E2E close**; Phase
    6 (ZStack + conditional rendering) is a natural candidate for
    absorbing the small host/window-metadata wiring task because
    the lightbox UX naturally exercises window-level metadata. The
    pre-doc carry-forward bullet records (d) this M3-Phase 5 pre-doc
    obligation.
  - **Q3 — T7 retrospective scope.** T7's original last bullet
    "Phase-end retrospective recorded under `docs/notes/m3-phase-4/`"
    conflated two distinct retrospectives in
    [retrospectives.md](../../notes/retrospectives.md) §進行手順:
    step-end retro (items 1-11; step → phase merge gate) and
    phase-end retro (items 12-18; phase → main merge gate). The
    bullet is split into two: T7 owns the **step-end** retrospective
    only (`t7-step-end-retrospective.md`); the phase-end
    retrospective (`phase-end-retrospective.md`) is owned by a
    separate commit on `feat/m3-phase-4` after T7 merges in and is
    the precondition for the phase → main merge gate. This avoids
    the reviewer hazard "T7 close has an unflipped phase-end
    retrospective bullet" — the two bullets now name distinct
    retrospectives at distinct gates, so the `[ ]` on the phase-end
    line is the correct state at T7 step-close.

## CI / verification log

- **T6 close (2026-05-25).** Local clean rebuild proxy for the T6 fix bundle (commit `ed78d6c`) on the
  `feat/m3-phase-4-t6` branch: `cargo fmt --all -- --check` zero
  exit; `cargo build --release --workspace` green; `cargo build
  --workspace` green; `cargo test --workspace` green — `wasamo-runtime`
  lib **258 passed** (T5 baseline 257 + the new pure-logic pinning
  test `shrink_vstack_root_with_fill_scroll_view_child_collapses`);
  `scroll_view_layout_integration` **3 passed** (T5 baseline 2 + the
  new mock-free runtime integration test
  `scroll_path_vstack_root_fixture_pins_window_root_fill_override`
  driving `run_layout_as_window_root`); `wrap_panel_layout_integration`
  back to **2 passed** after the `run_layout` /
  `run_layout_as_window_root` split insulated it from the window-root
  override. Assistant-side `Start-Process` on rebuilt
  `target/release/gallery-rust.exe` produced PID 3916, MainWindowTitle
  `"Wasamo"`, HasExited = `$false` after 3 s, `Stop-Process -Force`
  clean exit. Owner-manual GUI smoke (2026-05-25, re-run on the post-
  `ed78d6c` binary) returned green on all four observation points
  (viewport clip sharp, +100/-100 Buttons move content by ±100 px,
  clipped regions stay within the ScrollView outer rect, off-viewport
  thumbnails enter as `scroll_y` progresses) plus the +1 reference
  observation (scrollbar non-display, expected under Phase 4 scope);
  window close (Alt+F4 / ×) crash-free; smoke evidence committed at
  `docs/references/m3-phase-4/t6-gallery-smoke-scroll-y-{0,100,800,
  back-to-0}.png`. GitHub Actions CI green confirmation is **T7**
  phase-end gate (`workflow_dispatch`); local clean rebuild is the
  T6-step-level proxy.

## Out-of-phase residuals

(empty — populated only if Phase 4 closes with explicit carry-forward
items)
