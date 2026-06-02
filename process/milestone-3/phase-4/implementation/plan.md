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
      observation note (per [human-visible GUI smoke](../../../../docs/notes/human-visible-smoke.md)).
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
      (`process/milestone-3/phase-4/decisions/preamble.md`) / dsl_spec §4.11 /
      architecture.md §6.5; any fix requiring a normative spec change
      escalates to T7 Moment 2 (or, if unsuitable for Moment 2, a
      mid-ADR addendum). Fix iterations stay inside T6 until the
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
      ([t6-step-end-retrospective.md](../retrospectives/t6.md)).

### T7 — Phase-end gates and Moment 2 re-sync

Discharges the m3-plan phase-end criteria for Phase 4. Renumbered
from the original T6 when the T5/T6 split landed; checklist content
extends the original list with Moment 2 sync items surfaced by T4
/ T5 / T6 retrospectives' Item 10 (phase-sync classifications) and
T6 retrospective Follow-Up #1.

- [x] `cargo fmt --all -- --check` green.
- [x] `cargo build --release --workspace` green locally and on CI.
- [x] `cargo test --workspace` green locally and on CI.
- [x] Windows-only integration evidence green on CI.
- [x] `docs/dsl_spec.md` §4.11 Phase status marker flips to
      `M3-Phase 4 closed; implementation-synced`.
- [x] `docs/architecture.md` top Status flips to include M3-Phase 4
      complete, with implementation divergences reconciled.
- [x] `docs/architecture.md` §6 (general layout) gains a single-sentence
      runtime-boundary invariant covering "window-root `WidgetNode` is
      sized to the client rect regardless of its declared width/height
      constraints, enacted by `WidgetNode::run_layout_as_window_root`
      and called by `window.rs`'s `WM_SIZE` handler and `set_root`
      initial layout" (T6 retrospective Item 10 (1) `phase-sync`; T6
      Follow-Up #1). Landed as a paragraph after §6.3's default-
      constraints table (single sentence expanded into a four-sentence
      paragraph to cover the override mechanism, the unchanged plain
      `run_layout` entry point, and the unchanged pure-logic
      conventions including `degenerate_fill_in_shrink_parent_clamps_
      to_zero`).
- [x] `docs/architecture.md` §6.5 (WidgetNode / Visual Layer sync)
      gains explicit prose that the ScrollView intermediate Visual's
      scroll translation `(0, -offset_y, 0)` also shifts each content
      child's `sync_visuals()` `parent_abs_offset` by `(0, -offset_y)`
      so root-relative LayoutNode offsets convert correctly under the
      intermediate Visual (T4 retrospective Item 10, configured-class
      corrected to `phase-sync` in T5 retrospective Follow-Up; T5
      Follow-Up #2 / T6 Follow-Up #2). Landed as a follow-up paragraph
      to the existing ScrollView Visual-extension paragraph, generalised
      to "any Visual that translates its own subtree must shift
      `parent_abs_offset` by the inverse translation for its
      descendants" so future intermediate-Visual widgets can cite the
      rule.
- [x] `docs/dsl_spec.md` §4.9 Box examples switch from the
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
      carry-over). §4.10 common-pitfalls 1 's `; …` continuation also
      dropped to match the new convention.
- [x] `docs/plans/m3-plan.md` Phase 4 row flips to complete.
- [x] **Out-of-phase residual R1 registration: gallery host Window
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
      #5). See §Out-of-phase residuals below for the registered entry.
- [x] Carry-forward inputs to the next phase's pre-doc are recorded
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
- [x] This progress file records the phase-close evidence, CI pointer,
      implementation summary, and lifecycle transition.
- [x] T7 step-end retrospective recorded at
      `docs/notes/m3-phase-4/t7-step-end-retrospective.md`
      (retrospectives.md checklist items 1-11; step → phase merge
      gate; **owned by T7**).
- [x] Phase-end retrospective recorded at
      `docs/notes/m3-phase-4/phase-end-retrospective.md`
      (retrospectives.md checklist items 12-18; **phase → main merge
      gate, performed on `feat/m3-phase-4` after T7 merges in**).
      Flipped by the separate phase-end retro commit on
      `feat/m3-phase-4`, after T7 merged in; per Q3 disposition
      (2026-05-25), this confirms the T7 step-end / phase-end
      ownership split.
