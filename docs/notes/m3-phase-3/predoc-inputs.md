---
title: M3-Phase 3 pre-doc inputs — carried forward from M3-Phase 2 close
status: live
created: 2026-05-20
source-phase: M3-Phase 2
target-phase: M3-Phase 3
---

# M3-Phase 3 pre-doc inputs

This note carries forward the Phase 2 close learnings into the
M3-Phase 3 (WrapPanel layout primitive) pre-doc. It is intentionally
actionable rather than retrospective-only: Phase 3 should be able to
start from these constraints without re-reading every Phase 2 commit.

## 1. WrapPanel should consume Box intrinsic sizing, not redefine it

Phase 2 gave `Box { aspect: <ratio> }` a defined intrinsic size when
one parent axis is bounded: the bounded axis wins and the other axis is
derived from the ratio. Phase 3's WrapPanel ADR should cite
`docs/dsl_spec.md` §4.9 for that behavior and define how WrapPanel
offers main-axis / cross-axis constraints to children; it should avoid
restating the Box aspect algorithm.

Concrete pre-doc question:

- In a thumbnail strip, does WrapPanel measure each child with a fixed
  main-axis item slot, a max main-axis constraint, or an unbounded
  main-axis constraint plus later arrange? The answer determines how
  `Box { aspect: 1:1 }` obtains its thumbnail size.

## 2. Placeholder thumbnails are now the normative gallery asset shape

Phase 2 made `Box { aspect: <ratio>; fill: <color>; Text { ... } }`
the normative pre-Image placeholder pattern. Phase 3 should build its
gallery sub-screen from that shape rather than introducing an Image-like
surface, asset pipeline, or host-imperative fixture.

Concrete pre-doc question:

- What minimal thumbnail item shape should Phase 3 use for visible
  proof: square placeholders (`1:1`), mixed aspect placeholders, or a
  fixed set that includes both? Mixed aspect items are better evidence
  for wrapping only if the WrapPanel contract intentionally supports
  variable child extents in the main axis.

## 3. Multi-child overlap remains out of Box scope

Phase 2 deliberately rejected 2+ children in Box and pointed overlap to
ZStack. Phase 3 should not rely on Box for labels over thumbnails or
badges over images. If a WrapPanel item needs a composite thumbnail
before ZStack ships, the Phase 3 ADR should keep it as a plain child
tree that does not require overlap semantics.

Concrete pre-doc question:

- Does the Phase 3 gallery proof need labels inside the placeholder, or
  can the visible item be a single Box + centred Text placeholder until
  ZStack / Image later broaden the composition?

## 4. Spec-drafting bar rises in Phase 3

Phase 2's spec close was mostly a re-sync of an ADR-written Box chapter.
Phase 3 is the first M3 phase whose acceptance text calls out a novel
normative measure-arrange algorithm. The pre-doc should land a draft
WrapPanel spec outline before implementation starts, including:

- line formation inputs and outputs;
- main-axis overflow behavior;
- cross-axis line sizing;
- spacing / padding treatment, or an explicit statement that those
  attributes are not in Phase 3 scope;
- unbounded-parent behavior, especially inside the later ScrollView
  phase.

## 5. Keep the constant-only value boundary unless Phase 3 needs more

Phase 2 kept `Ratio` and `Color` Box-internal and avoided new
`PropertyValue` / ABI arms. Phase 3 should preserve that boundary unless
WrapPanel itself introduces a bindable value type. If a new property
needs binding, the per-type runtime writer, IR literal/type surface, and
ABI conversion story should be decided in the same step rather than
split across phases.

## 6. Verification shape to inherit

Phase 3 should keep the Phase 2 split:

- pure-logic measure-arrange tests for the WrapPanel line breaker;
- IR / loader tests only for the new widget and properties it actually
  adds;
- one Windows-only integration test if the visible behavior depends on
  real compositor-backed widget state;
- a gallery sub-screen grown through `.ui -> wasamoc -> IR text ->
  wasamo_load_ui`, not host-imperative construction.

The Phase 2 T11 skip guard pattern is still the right model: local
developer machines may skip when Compositor creation is unavailable, but
GitHub Actions must fail rather than silently skip.
