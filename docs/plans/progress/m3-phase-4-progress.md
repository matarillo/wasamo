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

### T5 — End-to-end gallery visible smoke

Discharges ADR verification closure **evidence item 5** and the
phase-close / A11 gallery proof.

- [x] Grow `examples/gallery/gallery.ui` additively with a sibling
      `ScrollView { WrapPanel { Box × 30–40 } }` slice; the Phase 3
      standalone WrapPanel slice stays untouched.
- [x] Add programmatic scroll controls that mutate `state.scroll_y` by
      fixed increments.
- [x] Build and run `examples/gallery-rust/`.
- [x] Record `Start-Process` launch success by the assistant.
- [ ] Leave visual correctness as owner-manual GUI smoke: viewport
      clips sharply, buttons move content, clipped content is hidden,
      and off-viewport thumbnails enter view as scroll progresses.
- [x] C / Zig gallery hosts remain out of Phase 4 scope.

### T6 — Phase-end gates and Moment 2 re-sync

Discharges the m3-plan phase-end criteria for Phase 4.

- [ ] `cargo fmt --all -- --check` green.
- [ ] `cargo build --release --workspace` green locally and on CI.
- [ ] `cargo test --workspace` green locally and on CI.
- [ ] Windows-only integration evidence green on CI.
- [ ] `docs/dsl_spec.md` §4.11 Phase status marker flips to
      `M3-Phase 4 closed; implementation-synced`.
- [ ] `docs/architecture.md` top Status flips to include M3-Phase 4
      complete, with implementation divergences reconciled.
- [ ] `docs/plans/m3-plan.md` Phase 4 row flips to complete.
- [ ] This progress file records the phase-close evidence, CI pointer,
      implementation summary, and lifecycle transition.
- [ ] Phase-end retrospective recorded under
      `docs/notes/m3-phase-4/`.

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

## CI / verification log

(empty — populated as T1 onward lands)

## Out-of-phase residuals

(empty — populated only if Phase 4 closes with explicit carry-forward
items)
