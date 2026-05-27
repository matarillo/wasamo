---
title: M3-Phase 4 pre-doc inputs — forward distillation from M3-Phase 3 close
status: live
created: 2026-05-22
source-phase: M3-Phase 3
target-phase: M3-Phase 4
---

# M3-Phase 4 pre-doc inputs

Forward-distillation of M3-Phase 3 (WrapPanel) learnings into
actionable inputs for M3-Phase 4 (ScrollView primitive — minimal).
Same shape as
[m3-phase-3/predoc-inputs.md](../m3-phase-3/predoc-inputs.md): each
section names a Phase 3 finding, frames it as a constraint or
question for the Phase 4 ADR / progress doc, and points to the
underlying step retrospective(s) so the chain is auditable.

Per [retrospectives.md forward-carry rule](../retrospectives.md), the
Phase 3 Main Learnings must be present in this file before the
Phase 3 phase-end merge gate.

## 1. "No new parser grammar" can still hide lexer-surface changes

Phase 3 T1 found that the progress-doc preamble "Phase 3 introduces
no new parser grammar" was literally true at the parser level but
silently required two lexer-surface extensions to reach the parser:
kebab-case `Token::Ident` and an optional leading `-` on `IntLit`.
The fix landed inline in T1 and was folded into `dsl_spec.md` §2.2 at
Phase 3 close.

Pre-doc question for Phase 4:

- Does any ScrollView attribute name require a token-shape that the
  M3-Phase 3 lexer doesn't already accept? Specifically: does any
  attribute or binding surface use characters outside the post-Phase-3
  `Ident` pattern (`[A-Za-z_][A-Za-z0-9_]*(?:-[A-Za-z][A-Za-z0-9_]*)*`),
  or does any literal need a sign / unit / quoting form that is not
  already in §2.2?

See [t1-step-end-retrospective.md](../m3-phase-3/t1-step-end-retrospective.md).

## 2. ScrollView is the next novel-normative measure-arrange — but the structural framing already exists

Phase 3 T7 was the first M3 measure-arrange whose outer cross-axis
extent depended on its children. Phase 4 ScrollView is the second
novel measure-arrange, and the Phase 3 ADR (DD-M3-P3-005) and
WrapPanel chapter explicitly anticipate its pairing:
ScrollView bounds the main axis (viewport width) and unbounds the
cross axis — exactly the inverse of WrapPanel's bounded-main /
bounded-cross input.

This means Phase 4 should *not* re-derive the wrap-panel pairing
contract; it should consume it.

Pre-doc questions for Phase 4:

- Does ScrollView's outer measure depend on its content's intrinsic
  size, or does it report viewport-sized regardless? (The minimal
  ScrollView is almost certainly viewport-sized; pin this explicitly.)
- How does ScrollView's *unbounded* cross-axis constraint compose with
  WrapPanel's "if cross-axis is unbounded and a child is aspect-only,
  fire `BoxAspectUnboundedBoth`" rule? The composition is already
  spec'd by DD-M3-P3-005 — Phase 4 only needs to confirm ScrollView
  passes "unbounded" through cleanly.
- Is the viewport's main-axis bound a fixed attribute, or does it
  inherit from parent? The default-bound source decision belongs in
  the Phase 4 ADR's Spec impact preview.

See [t7-step-end-retrospective.md](../m3-phase-3/t7-step-end-retrospective.md)
and ADR DD-M3-P3-005.

## 3. "No new IR variant" must be a test fixture, not a promise

T3 / T4 verified the Phase 3 framing ("WrapPanel reuses generic
`IrNode` + `IrProp` + `IrLiteral::Int`") by adding tests that
exercise the existing generic shapes against new fixture inputs —
and by *not adding production code*. The "no production diff" was
itself evidence.

Phase 4 ScrollView is structurally heavier than WrapPanel because
content offset is the obvious bindable surface (the scroll position
is the central reactive value). Pre-doc questions:

- Does Phase 4 introduce a new `PropertyValue` variant for content
  offset? If yes, it crosses the Phase 2 / Phase 3 boundary that has
  held since Phase 1 (layout primitives have not pressured
  `PropertyValue`); enumerate the per-type evaluator/writer pair that
  must land in the same step (see ADR §6.8 "Per-type seam" paragraph
  in `docs/architecture.md`).
- Does Phase 4 introduce a new `IrLiteral` variant or reuse
  `IrLiteral::Int`? Scroll offset as `i32` pixels reuses; scroll
  offset as ratio (0.0–1.0) does not.
- Does Phase 4 introduce a new `IrType` for the offset binding? `i32`
  reuses Phase 3's surface; `f64` would need a new entry.
- Does Phase 4 introduce a new `LayoutError` variant? Likely yes for
  some pathological ScrollView shape (e.g. ScrollView inside an
  unbounded parent on the scroll axis — analogue of Phase 2's
  `BoxNoExtent` for ScrollView). Decide whether to extend
  `LayoutError` or to fold into existing variants.
- Does Phase 4 introduce a new `WASAMO_VALUE_*` or
  `WASAMO_LAYOUT_ERROR_*` ABI tag? Follow the same rule Phase 3
  used: extend ABI only when host code can observe the new value /
  error; otherwise hold the surface internal.

See [t3-step-end-retrospective.md](../m3-phase-3/t3-step-end-retrospective.md)
and [t4-step-end-retrospective.md](../m3-phase-3/t4-step-end-retrospective.md).

## 4. Defaults belong at the runtime widget catalog, not the IR layer

T5 surfaced the rule that "absent attribute → default value" is
applied by the widget catalog constructor (`WidgetData::WrapPanel`
in Phase 3), not by the IR loader's `unwrap_or(0)`. The progress doc
distinguished "defaults are applied at the runtime layer in T5, not
at the IR layer" and the implementation had to match that distinction
exactly.

Pre-doc question for Phase 4:

- For each ScrollView attribute the Phase 4 ADR introduces (viewport
  size source, scroll offset, scroll axis), specify which layer is
  responsible for materialising the default when the attribute is
  absent in the IR. Default to the widget-catalog constructor (the
  `WidgetData::ScrollView` variant the Phase 4 progress doc T5
  analogue will add); use the IR loader's `unwrap_or` shape only when
  the attribute has no widget-catalog field at all.

See [t5-step-end-retrospective.md](../m3-phase-3/t5-step-end-retrospective.md).

## 5. Runtime-gate scope follows the current phase's progress doc, not Phase N-1's pattern

T6 found that Phase 2 T7's `validate_phase2_node_invariants` was a
placement gate (reject Box-internal attributes appearing on non-Box
widgets), whereas Phase 3 T6's `validate_phase3_node_invariants` was
a value-range gate (reject negative `i32` on WrapPanel attributes).
The scopes are different because the underlying invariants are
different — Phase 2's invariants are structural; Phase 3's are
numeric.

Pre-doc question for Phase 4:

- What is the *shape* of ScrollView's runtime-gate invariants?
  Value-range (negative offset), structural (ScrollView inside a
  non-bounded parent), or compound (offset within content size)?
  Choose the validate-path shape from the invariant shape; do not
  inherit Phase 3's value-range pattern by default.

See [t6-step-end-retrospective.md](../m3-phase-3/t6-step-end-retrospective.md).

## 6. Free-function extraction does not auto-prove call-site correctness

T7's central learning: even after extracting `compute_wrap_lines`
into a free function, the bounded vs. unbounded code paths bound the
helper's `cross_extent` argument to two semantically different
values, and the resulting drift was caught only by a reject-style
test (a test that fails when the drift returns, not one that
incidentally passes today).

Pre-doc question for Phase 4:

- Identify the bounded-vs-unbounded fork in ScrollView's
  measure-arrange (viewport-bounded main-axis vs. content-unbounded
  scroll-axis). For each call site of any shared helper, enumerate
  what argument is passed and assert it in a test that *would fail*
  if the call-site arg drifted. Shared helper + entry flag = pin
  both branches with reject tests; do not rely on incidental
  shared-helper coverage.

See [t7-step-end-retrospective.md](../m3-phase-3/t7-step-end-retrospective.md).

## 7. ScrollView is a clip-installing widget — the inverse of WrapPanel

Phase 3 T8 closed the visible-overflow regulation by asserting that
WrapPanel installs *no clip surface* on its Composition visual. The
Phase 3 ADR's verification closure item 4 spelled out "WrapPanel
installs no clip surface" as positive evidence.

Phase 4 ScrollView is the dual: ScrollView *must* install a clip
surface, because the gallery's overflow state (`ScrollView {
WrapPanel { … } }`) is exactly where WrapPanel's "parent clips"
contract becomes active.

Pre-doc question for Phase 4:

- Specify the Composition clip surface ScrollView installs (likely
  `Visual.Clip = InsetClip(0,0,0,0)` sized to the viewport, or a
  `RectangleClip` of the viewport rect). The integration test should
  assert *clip presence* on the ScrollView visual and *clip absence*
  on the inner WrapPanel — the symmetric inverse of Phase 3 T8's
  assertion.

See [t8-step-end-retrospective.md](../m3-phase-3/t8-step-end-retrospective.md)
and ADR §Phase 3 verification closure item 4.

## 8. Pure-layout absolute offset vs. Composition parent-relative offset is now stated in architecture

Phase 3 T9 surfaced a `sync_visuals` bug whose root cause was the
implicit absolute-vs-parent-relative offset convention at the
layout-engine / Visual-Layer boundary. The architecture-side
clarification was folded into [architecture.md §6.5](../../architecture.md)
at Phase 3 close (R3-A). The test-coverage half remains open as
out-of-phase residual R2.

Pre-doc questions for Phase 4:

- ScrollView's content offset binding produces non-zero parent-
  relative offsets via `Visual.Offset` or
  `Visual.TransformMatrix` (depending on how scroll is implemented).
  Re-read §6.5 before deciding which Composition primitive carries
  the scroll offset.
- Phase 4's measure-arrange touches the `sync_visuals` boundary
  meaningfully (the content offset changes on every scroll
  interaction). This is the natural place to close out-of-phase
  residual R2 (the test-coverage half — a pure-or-Compositor-backed
  test that asserts the relative-offset computation for nested
  non-zero-offset visual trees). Decide in the Phase 4 pre-doc
  whether to land R2 inside Phase 4 or to forward it to a later
  test-coverage pass.

See [t9-step-end-retrospective.md](../m3-phase-3/t9-step-end-retrospective.md)
and Phase 3 progress doc §Out-of-phase residuals R2.

## 9. Outstanding Phase 3 residual that pertains to Phase 4's scope

R2 (above) is the only Phase 3 residual that Phase 4 may want to
discharge in-scope. R1 (`.gitignore` `*.uic`) is cross-cutting
hygiene; defer unless Phase 4 separately touches build hygiene.

## 10. Spec drafting bar stays high — ScrollView is the third M3 layout primitive with novel normative content

Phase 3 raised the spec-drafting bar (novel normative measure-arrange
algorithm). Phase 4 inherits that bar: the ScrollView chapter in
`dsl_spec.md` should land before implementation begins, covering at
minimum:

- viewport bound source (attribute vs. parent);
- content measure pass (unbounded along scroll axis, bounded along
  cross axis — confirms the pairing the WrapPanel chapter already
  spec'd from the other side);
- content offset semantics (clamp to `[0, content_size -
  viewport_size]`, or allow over/under-scroll?);
- offset binding direction (read-only vs. in-out — affects whether
  `PropertyValue` extension is needed);
- clip surface installation as a normative requirement;
- behaviour when content size is smaller than viewport.

Pre-doc question:

- Which of the above belong in `dsl_spec.md §4.11` (the next
  available section) vs. `architecture.md §6.X` vs. a Phase 4 ADR
  DD? Default per Phase 3: normative semantics in `dsl_spec.md`;
  implementation pipeline in `architecture.md`; design decisions
  with multiple defensible answers in the ADR.

## 11. Gallery sub-screen growth path

Phase 3 grew `examples/gallery/` from a single Box sub-screen to a
WrapPanel of 10 thumbnails (88×88 canonical example, 7+3 wrap on the
default 800×600 window). Phase 4's ScrollView sub-screen wraps the
existing WrapPanel: `ScrollView { WrapPanel { … } }` with enough
thumbnails to demonstrate vertical scroll on the same default window.

Pre-doc questions:

- Does Phase 4 grow the existing Phase 3 sub-screen (wrap the
  WrapPanel) or add a sibling sub-screen? Per framing decision E
  (sub-screen per phase), a sibling is the default; wrapping the
  Phase 3 sub-screen would mean Phase 3's wrap evidence is no longer
  visually isolated.
- How many thumbnails are needed to make vertical scroll visible on
  the 800×600 default window? With 88×88 thumbnails, 12 px item /
  line spacing, and Phase 3's observed 7+3 wrap (7 thumbs per row
  at ≈784 px client width), each wrap row contributes ~100 px
  (88 thumb + 12 line-spacing). A ScrollView viewport occupying
  ~400 px of the window's vertical space fits ~4 rows, so total
  content needs >4 rows for scroll to be visible — roughly 30–40
  thumbnails (≈5–6 rows). The exact count is a Phase 4 framing
  decision; the Phase 3 5–10 ceiling (framing decision E) does not
  apply to Phase 4 since wrap is no longer the visible proof on
  its own.
- C / Zig host parity: Phase 4 still ships gallery-rust only (per
  framing decision E); full gallery parity stays at Phase 8.

## 12. Process-level continuity items

- **AskUserQuestion is paused for design choices.** Phase 3 T10
  established that owner prefers inline-in-chat option presentation
  over AskUserQuestion for design choices. Phase 4 pre-doc should
  follow the same pattern — present options inline with pros/cons
  and a recommendation rather than spinning up a structured question.
- **Step-end fast-track gate is removed.** Phase 3 closed multiple
  steps via fast-track (T2 / T3 / T4 / T6), but the gate was
  retired as a Phase 4 prep step alongside the no-ff wording fold
  (see next bullet). Every step-end and phase-end merge now requires
  owner explicit approval; the "report-then-merge with after-the-fact
  notification" path is no longer available. Rationale: step-level
  decision visibility on the owner side outweighs the cost of
  rubber-stamping items 2–8 = "なし" judgments inline.
- **Phase-end merge is owner-gated.** Phase 3 close re-confirmed
  that phase-end merge to main and the subsequent push are separate
  owner gates (per `retrospectives.md` and
  `feedback_phase_end_merge.md`). Phase 4 pre-doc should not assume
  any phase-end gate is auto-discharged.
- **`retrospectives.md` step → phase merge wording fold (resolved).**
  Up through M3-Phase 3 the procedure doc described step → phase
  merge as "ff merge", but Phase 2 / Phase 3 実運用 had been
  **no-ff** consistently (see `git log --merges feat/m3-phase-2`
  and `feat/m3-phase-3` — every task merge is a
  `Merge branch 'feat/m3-phase-N-tM'` commit). The drift was
  surfaced during M3-Phase 3 T10 phase-end review and folded as a
  Phase 4 prep commit (alongside the fast-track removal above);
  `retrospectives.md` now describes both step→phase and phase→main
  as no-ff merge. Past step retrospectives' "ff merge" mentions are
  preserved as historical record of the pre-fold wording.
- **Step-end item 10 (cross-step / cross-phase 設計制約 carry) is
  new.** Phase 4 prep commit added a new step-end checklist item to
  `retrospectives.md` for surfacing implicit design constraints that
  the pre-doc / ADR / `architecture.md` / `dsl_spec.md` / `abi_spec.md`
  did not anticipate (worked-example precedent: M3-Phase 3 T9
  pure-layout absolute vs. `Visual.Offset` parent-relative
  convention). When "あり", the step retro classifies the constraint
  by one of four dispositions: `doc-folded` (folded into spec / ADR
  in-step), `phase-sync` (deferred to phase-end Moment 2), `carry-forward`
  (forwarded to next phase pre-doc input), `local-only` (not a future
  constraint, with one-sentence rationale). Phase 4 will be the first
  phase to use this vocabulary; step retros and the phase-end forward
  distillation should follow it from day 1. `phase-sync` items must
  close at phase-end into one of `doc-folded` / `carry-forward` /
  `local-only` (no open phase-sync items survive past phase close).
  If the vocabulary or the disposition routing turns out to need
  adjustment during Phase 4, fold the change back into `retrospectives.md`
  rather than carrying drift forward.

## 13. `docs/notes` audit triggers for Phase 4

Re-read the same live-note set Phase 3 audited
([m3-phase-3/predoc-inputs.md §13](../m3-phase-3/predoc-inputs.md)),
re-classifying each as fired / partially fired / not fired for
ScrollView. Notably:

- `layout-engine.md`: **fired** (ScrollView is the next novel
  measure-arrange and introduces the first M3 widget whose content
  is observably *clipped* against its outer extent).
- `typed-value-evaluator.md`: **likely fired** if Phase 4 introduces
  a bindable offset surface (the trigger Phase 1 originally fired
  on). Decide in pre-doc whether ScrollView's offset binding
  pressures `TypedValue` deferral or stays on the existing per-type
  writer seam.
- `architectural-family.md`: **stays consumed** — ScrollView is a
  built-in primitive in the tree-with-bindings family, no
  re-evaluation needed.
- `verification-environments.md`: **fired** — Phase 4 will need a
  Windows-runtime integration test (clip presence + content offset
  application) following the Phase 3 T8 pattern.
