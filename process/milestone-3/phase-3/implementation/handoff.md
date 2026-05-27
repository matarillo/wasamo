## Out-of-phase residuals

- **(R1) `.gitignore` `*.uic` pattern.** During T9, an ad-hoc debug
  invocation `wasamoc build examples\gallery\gallery.ui
  examples\gallery\gallery.uic` produced an in-tree `.uic` artefact
  (removed manually). The production build paths route `.uic`
  through `OUT_DIR` via `build.rs` (`examples/*/build.rs`), so the
  in-tree artefact is never produced by a normal workspace build —
  but the temptation to write `.uic` in-tree for debugging recurs.
  A `.gitignore` rule for `*.uic` would prevent accidental commits.
  Phase 3 scope did not include build-hygiene changes, so this is
  not folded here; tracked for any future cross-cutting hygiene
  pass. Surfaced in
  [t9-step-end-retrospective.md](../../notes/m3-phase-3/t9-step-end-retrospective.md)
  Follow-Up R1.

- **(R2) `sync_visuals` ↔ pure-layout boundary test gap.** The
  Phase 2 test suite pins `LayoutNode.offset` to the absolute
  (root-relative) convention but does not exercise the conversion
  to parent-relative `Visual.Offset` performed by `sync_visuals()`.
  The T9 visible-smoke bug whose fix landed at commit `570d08a` was
  detected only by owner-manual GUI smoke (framing decision G); a
  regression of the same class would again rely on visible-smoke
  detection. A pure-or-Compositor-backed test that asserts the
  relative-offset computation for a nested non-zero-offset visual
  tree would close the detection gap independently of visible
  smoke. Belongs to whichever later phase first revisits the
  `WidgetNode` / Visual-Layer sync seam (likely Phase 4 ScrollView
  or a focused test-coverage pass). Surfaced in
  [t9-step-end-retrospective.md](../../notes/m3-phase-3/t9-step-end-retrospective.md)
  Follow-Up R2. Architecture-level offset convention is now stated
  in [docs/architecture.md §6.5](../../architecture.md) (folded in
  T10 as R3-A); this residual is the test-coverage half that is
  not folded.
