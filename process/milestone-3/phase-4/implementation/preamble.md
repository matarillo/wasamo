---
phase: M3-Phase 4
title: ScrollView primitive (minimal)
status: closing
adr: process/milestone-3/phase-4/2.designs/_index.md
plan: process/milestone-3/_plan.md
opened: 2026-05-25
---

# M3-Phase 4 — ScrollView primitive (minimal): Progress

This is the live task list and execution log for M3-Phase 4. The
design decisions are frozen in
[m3-phase-4-scroll-view.md](../decisions/preamble.md);
this file is mutable per
[plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle).

Phase 4 satisfies A5 by adding a vertical-only `ScrollView` with
inner unbounded measure, viewport clip, and `offset-y` content offset
binding. Scrollbar widget, wheel handler, drag, in-out offset
write-back, imperative scroll commands, horizontal/bidirectional
scrolling, and the general typed-`i32` writer pair are M4 or later.

The automated A5 evidence set is verification items 1 through 4 in
the ADR's
[Phase 4 verification closure](../decisions/preamble.md#phase-4-verification-closure-what-counts-as-a5-evidence).
The phase-close / A11 gallery proof is item 5, including owner-manual
GUI smoke.

## Implementation summary (Phase 4 close)

Phase 4 added a vertical-only ScrollView primitive that exposes a
bounded viewport over one scrollable content child and translates
that content by a clamped `offset-y` value. Wire-level shape:

- **wasamoc surface (T1):** `ScrollView` registered as a known widget
  type with the generic parser shape preserved; exactly one child
  enforced at compile time; `offset-y` accepts an `i32` literal or a
  bare `state` identifier of declared type `i32`; `bind` / `in-out`
  modifiers, `viewport-*` / `scroll-axis` / `padding` attributes,
  and bool/string state binding all rejected by `wasamoc check`.
- **Layout engine (T2):** pure-data measure-arrange in
  `wasamo-runtime/src/layout.rs`: content measured with viewport
  width and unbounded vertical constraint; outer size equals viewport
  size; `offset-y` clamped to `[0, max(0, content_h - viewport_h)]`;
  unbounded vertical parent produces `LayoutError::ScrollViewUnbounded
  Axis`; rounding contract preserved.
- **IR loader (T3):** `ScrollView` materialised with exactly one
  content child and an `offset-y` field; 0-child / >1-child rejected
  at IR load with `WASAMO_ERR_IR_MALFORMED`; out-of-range `offset-y`
  accepted as input and clamped at layout time.
- **Visual layer + Windows runtime (T4):** ScrollView owns an outer
  widget Visual plus a ScrollView-owned intermediate content Visual;
  outer carries `Visual.Clip = InsetClip { 0, 0, 0, 0 }`; intermediate
  carries `Visual.Offset = (0, -applied_offset_y, 0)`; the content
  child's `sync_visuals()` `parent_abs_offset` is shifted by the
  inverse translation so root-relative LayoutNode offsets convert
  correctly underneath the intermediate Visual.
- **Gallery integration (T5):** `examples/gallery/gallery.ui` grew an
  additive `VStack { WrapPanel … ; Button … ; Button … ; ScrollView {
  WrapPanel { Box × 32 } } }` slice with Button-driven `scroll_y`
  controls; no runtime / IR loader / wasamoc code touched at T5.
- **Window-root sizing fix (T6):** `WidgetNode::run_layout_as_window_
  root` introduced to override the root LayoutNode's
  `width` / `height` to `SizeConstraint::Fill` before delegating to
  `layout::run_layout`, resolving the Shrink-VStack-root + Fill-
  ScrollView-child collapse that the Phase 3 WrapPanel test envelope
  had not exercised. Plain `run_layout` retained for non-window-root
  integration tests; pure-logic layout engine and the
  `degenerate_fill_in_shrink_parent_clamps_to_zero` convention
  untouched. Gallery `item-cross-size: 64 → 128` made content_h
  exceed viewport_h across realistic window sizes so `+100/-100`
  motion is visible. Pure-logic pinning test
  (`shrink_vstack_root_with_fill_scroll_view_child_collapses`) and
  mock-free runtime integration test
  (`scroll_path_vstack_root_fixture_pins_window_root_fill_override`)
  added together to pin the layout-engine invariant and the runtime-
  boundary override at independent layers.

A5 evidence discharge: items 1-4 (wasamoc surface; layout engine;
IR loader / validate; Windows-runtime layout + Visual + R2 closure
including three-level nested root-relative position math for
parent → ScrollView outer → intermediate → thumbnail) automated;
item 5 (end-to-end gallery + visible smoke) split into T5 (assistant-
automated) + T6 (owner-manual smoke with T6 fix iteration absorbed
in-step). A11 owner-acceptance recorded 2026-05-25 on the rebuilt
gallery host. CI evidence: workflow_dispatch run `26404665377`,
success.

## Lifecycle transition

Per [plans/README.md §Phase progress file lifecycle](../README.md#phase-progress-file-lifecycle),
this progress file transitions from `status: active` to
`status: closing` at T7 step-close. Front-matter `status` flips from
`active` to `closing` in this same commit. The progress file remains
mutable during the `closing` window only for the phase-end
retrospective evidence pointer and any final post-merge distillation;
no further task checkboxes are added. The progress file flips from
`closing` to `retired` (and is deleted by default per the lifecycle
table) on the phase → main merge commit, after the phase-end
retrospective lands. The phase-end retrospective
(`docs/notes/m3-phase-4/phase-end-retrospective.md`) is owned by a
separate commit on `feat/m3-phase-4` after T7 merges in and is the
precondition for the phase → main merge gate; it is **not** a T7
deliverable.
