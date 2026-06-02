# M3-Phase 4 pre-doc framing

**Status:** framing aligned with owner (2026-05-25); input artefact for ADR drafting
**Date:** 2026-05-25
**Targets phase:** M3-Phase 4 (ScrollView primitive — minimal)

Per the project's doc-driven workflow established at
[M2-Phase 6 pre-doc framing](../../../milestone-2/phase-6/requirements/framing.md)
and continued through
[M3-Phase 2 pre-doc framing](../../phase-2/requirements/framing.md)
and
[M3-Phase 3 pre-doc framing](../../phase-3/requirements/framing.md),
individual DDs are not negotiated one-by-one in chat — framing is
aligned first, then the full ADR is drafted in one pass as
`Status: Proposed`, reviewed, and flipped to `Status: Accepted`.
This note records the framing intended for owner alignment before
ADR drafting begins; it remains as an input artefact and is not
promoted into the ADR.

The three preceding M3 phases supply several things this framing
inherits rather than re-derives:

- **Two-moment spec-sync structure** (Moment 1 design-spec draft at
  ADR-Accepted commit; Moment 2 implementation re-sync at phase
  close), with section-level `**Phase status:**` markers in the
  affected `docs/dsl_spec.md` chapter. See
  [m3-phase-2 framing decision D](../../phase-2/requirements/framing.md#d-upstream-document-revision-timing-two-sync-moments).
  The doc set and commit shape are now living rule in
  [retrospectives.md §phase-sync (Moment 2) で触る doc セット](../../../procedures/retrospectives.md#phase-sync-moment-2-で触る-doc-セット).
- **Moment-is-not-a-commit-unit rule**, recorded in
  [CLAUDE.md §Commit rules](../../../CLAUDE.md#commit-rules): each
  constituent document lands as its own commit on the pre-doc
  branch, scoped by review concern, not by Moment.
- **No fast-track at step-end or phase-end** — every merge requires
  owner explicit approval (Phase 4 prep, commit `49b49fb`); the
  pre-doc-branch landing of this framing's downstream commits (ADR,
  spec sync, progress doc) is no exception.
- **Step-end item 10** — cross-step / cross-phase 設計制約 carry,
  classified into one of `doc-folded` / `phase-sync` /
  `carry-forward` / `local-only`. Phase 4 is the first phase to use
  this vocabulary from day 1; pre-doc framing flags the points where
  it is most likely to fire so step retros can apply the routing
  without re-deriving the discipline.

---

## Phase 4 acceptance criteria (restated)

- **A5** (see [process/_roadmap.md M3](../../../_roadmap.md#m3-dsl-surface),
  [m3-plan.md §Acceptance criteria](../../plan.md#acceptance-criteria)):

  > ScrollView primitive (minimal: inner unbounded measure +
  > viewport clip + content offset binding; scrollbar widget,
  > wheel handler, and drag are deferred to M4).

  The three minimal-scope components — **inner unbounded measure**,
  **viewport clip**, **content offset binding** — map directly to
  the load-bearing DDs below (DD-005 / DD-004 / DD-003 respectively).
  "Minimal" is normative scope, not a soft target: anything that
  reaches into input handling (wheel / drag / keyboard PgUp) or
  chrome rendering (scrollbar widget) is out of Phase 4 by A5's
  explicit deferral to M4.

- **A11 (operational obligation).** `.ui`, `wasamo-ir`, `wasamoc`,
  `wasamo-runtime`, `docs/dsl_spec.md`, and the
  `examples/gallery/` sub-screen all advance within Phase 4. Phase 3
  grew `examples/gallery/` from Phase 2's single Box into a
  WrapPanel of 10 thumbnails; Phase 4 grows the sub-screen further
  per framing decision E below.

- **Normative viewport / content / offset semantics required.**
  Phase 3 was the first novel-normative-spec phase
  ([m3-phase-3 framing — first novel normative measure-arrange spec](../../phase-3/requirements/framing.md#phase-3-acceptance-criteria-restated));
  Phase 5 Grid retains the "second novel-normative-spec phase"
  position per [m3-plan.md §Phase breakdown](../../plan.md#phase-breakdown)
  (star sizing is the heavier algorithmic content). Phase 4 does
  not displace that ordering; rather, it introduces a **smaller
  novel normative surface** of its own — the
  **viewport / content / offset triangle**: viewport bounds along
  the scroll axis, content measures unbounded along the same axis,
  and content offset is the binding-driven translation between
  them. The spec is novel in *kind* (clip + offset semantics are
  not present anywhere in the M2 / Phase 1–3 dsl_spec.md) but
  smaller in *scope* than Phase 3's two-stage measure-arrange or
  Phase 5's star sizing. Acceptance for the spec text is the
  [m3-plan.md §Milestone-end criteria item 5](../../plan.md#milestone-end-criteria)
  external-reader bar, applied at phase close.

- **First M3 phase to pressure the typed-`i32` writer seam.** M2
  built the **String** evaluator / writer pair
  (`register_binding_with_writer(Box<dyn FnMut(String)>, …)` +
  `widget_write_property`); Phase 1 built the **bool** sibling
  (`register_bool_binding_with_writer` + `widget_write_property_bool`).
  The typed-`i32` writer pair is the
  *explicitly-anticipated-but-not-yet-built third pair* called out in
[architecture.md §6.7 *Per-type seam* paragraph](../../../../docs/architecture.md#67-reactive-engine-m2-phase-5)
  ("When a typed-i32 binding writer becomes warranted (no current
  catalog row needs it…), it lands as a third pair with the same
  shape — additive, not by widening `write_fn` into a value union").
  ScrollView's content offset is the first surface where the runtime
  would natively want to **write** an `i32` value back into a bound
  state (the user scrolls → the offset state updates). Framing
  decision A below settles whether Phase 4 builds the typed-`i32`
  writer pair or stays read-only / programmatic-only (the existing
  M2 string-baked path that `IrType::I32` properties currently
  dispatch through is sufficient for read-only) for the minimal
  surface.

- **Downstream commitments grounded in Phase 4.** Phase 4 is the
  terminus of the Phase 2 → Phase 3 → Phase 4 thumbnail-strip
  chain ([m3-plan.md §Phase dependencies](../../plan.md#phase-dependencies));
  Phase 5 (Grid), Phase 6 (ZStack + conditional), and Phase 7
  (iteration) do not depend on ScrollView at the IR / evaluator
  level. The narrow downstream commitment is to Phase 7: per
  [m3-plan.md §Phase dependencies](../../plan.md#phase-dependencies)
  ("Phase 7 … iteration grammar … its E2E proof (thumbnails
  generated from a collection) reuses the WrapPanel + ScrollView
  combination from Phase 4. Sequencing after Phase 4 keeps the
  E2E proof a strict superset rather than a re-do."), Phase 7
  must be able to swap in a collection-generated thumbnail set
  for whatever fixed-thumbnail composition Phase 4 ships. Phase
  4 must therefore **already prove the `ScrollView { WrapPanel {
  … } }` composition with a fixed thumbnail set** so that Phase
  7's iteration grammar work is a strict superset (collection-
  driven generation of the same composition), not the first proof
  of the composition itself. ScrollView must also **compose with
  a child whose extent is computed dynamically** (the future
  iteration result), which is automatic once the
  inner-unbounded-measure contract from DD-005 is in place.

---

## Layering note (DD-001 ⇄ DD-002 ⇄ DD-003 ⇄ DD-004 ⇄ DD-005)

Phase 4 has a structurally simpler DD chain than Phase 3 because
ScrollView is a **single-child container with one bound attribute
(offset) and one clip surface**, not a multi-child algorithmic
primitive. The chain has one new wrinkle — the offset attribute is
**both** a layout input (content offset feeds into the arrange pass)
**and** the runtime's first writer-direction binding candidate. The
layering is:

- **DD-001 (IR shape).** Settles that ScrollView is a 1-child
  container whose child is the scroll content. The scroll-axis
  choice (`vertical` only vs `vertical | horizontal` enum vs
  per-axis booleans) is a sub-issue here, parallel to Phase 3's
  DD-002 orientation question.
- **DD-002 (viewport size source).** Settles where the viewport's
  outer extent comes from: parent constraint passthrough on both
  axes (the WPF / Compose default), explicit attribute, or hybrid.
  This is the Phase 4 analogue of Phase 3 DD-004's
  `item-cross-size` "where does this measure constraint come from"
  question. The downstream impact is on DD-005: viewport size is
  the *cross-axis* bound passed to the content (and, on the
  scroll axis, the bound the content measure result is compared
  against to decide whether scrolling is active).
- **DD-003 (content offset binding).** Settles the offset
  attribute's surface: bindable read-only `i32` (programmatic
  / state-driven only), bindable in-out `i32` (writer seam
  built), or constant-only (no binding — defer all binding
  questions to a later phase). This DD is the
  **load-bearing question for the writer seam** decision.
- **DD-004 (clip surface installation).** Settles which
  Composition primitive ScrollView installs as its clip
  (`InsetClip`, `RectangleClip` typed as `CompositionGeometricClip`,
  or `Visual.Clip` with a manual `Rect`) and how the content
  offset is applied (`Visual.Offset` on the child container vs
  `Visual.TransformMatrix`). This is the Composition-side answer
  Phase 3 T9 surfaced via the absolute-vs-parent-relative
convention now in [architecture.md §6.5](../../../../docs/architecture.md#65-widgetnode-and-visual-layer-sync).
- **DD-005 (measure-arrange algorithm).** Consumes DD-001 +
  DD-002 + DD-003 to produce the pure-data measure-arrange:
  content is measured against an unbounded scroll-axis and
  bounded cross-axis constraint; ScrollView's outer extent is
  the viewport (from DD-002); offset is clamped to a range
  derived from `content_size - viewport_size`. This is also
  the seed of the dsl_spec §4.11 chapter (Moment 1 draft, Moment 2
  re-sync).
- **DD-006 (IR-loader defense-in-depth).** Settles which
  invariants live in pure `validate()` vs WinRT-bound
  `build_node`, mirroring the Phase 2 T7 / Phase 3 T6
  precedent. ScrollView-specific invariants are conditional on
  what DD-001 / DD-002 / DD-003 admit.

The chain is **mostly linear** (001 → 002 → 003 ⇄ 004 → 005 → 006);
the one branch is at DD-003 / DD-004, where the binding direction
(DD-003 sub-issue) and the Composition primitive choice for offset
(DD-004 sub-issue) need to be co-consistent — an in-out binding
that writes through `Visual.TransformMatrix` is a different
runtime contract than one that writes through `Visual.Offset`.
The ADR's DD-003 and DD-004 Recommendation prose must
cross-reference each other on this point.

Concrete consequences for the ADR's Options tables: the following
combinations are **invalid** and should not appear as recommended
cells —

- DD-005 = "content measured with bounded scroll-axis" with any
  DD-001 ScrollView IR shape (contradicts A5's "inner unbounded
  measure" — Phase 4 ScrollView cannot bound its content's
  scroll-axis; that would be a viewport-without-scroll, i.e. a
  Box).
- DD-003 = "in-out i32 binding" with DD-002 = "no viewport size
  source defined" (contradicts the offset semantics — clamping
  range `[0, content_size - viewport_size]` needs
  `viewport_size` to be a well-defined runtime value).
- DD-004 = "Visual.TransformMatrix for offset" with DD-003 =
  "constant-only offset" (does not contradict structurally, but
  is over-engineered: TransformMatrix's value-add over
  Visual.Offset is animatable / fractional offsets, neither of
  which a constant-only binding pressures).

---

## Agreed DD slate (6 entries proposed)

The Phase 4 ADR (working title
`process/milestone-3/phase-4/decisions/preamble.md`) will carry the following
six DDs.

### DD-M3-P4-001 — ScrollView IR node form, 1-child contract, and scroll-axis exposure

ScrollView is a new widget in `wasamo-ir` and `wasamo-runtime`.
Phase 4 must commit to (i) the IR node shape, (ii) the
0-child / 1-child / N-child shapes, (iii) scroll-axis exposure.

Sub-issues:

- **IR node shape.** Per-kind tag parallel to `HStack` / `VStack` /
  `Rectangle` / `Box` / `WrapPanel`, vs a structural variant in
  `IrLayout`. Phase 2 / Phase 3 settled the per-kind-tag answer;
  Phase 4 inherits unless evidence forces re-opening. Default:
  per-kind tag.
- **0-child / 1-child / N-child shape.** ScrollView is a
  single-content container: 1 child is the canonical shape.
  Options: (a) require exactly 1 child (`validate()` rejects 0
  and >1); (b) accept 0 or 1 (0 = empty viewport, no content to
  scroll); (c) accept 0+ with implicit synthetic wrapper (the
  N>1 case wraps in an implicit VStack, mirroring some
  framework conventions). Framing recommendation: **(a)**.
  Implicit wrappers are a Grid / iteration concern (Phase 5 /
  Phase 7); ScrollView's spec stays narrow. Phase 3 admitted
  0-child WrapPanel because empty-line-set was meaningful;
  empty ScrollView has no analogous meaning beyond "Box-shaped
  viewport with nothing in it", which Box already provides.
- **Scroll-axis exposure.** Whether ScrollView exposes a
  `scroll-axis: <vertical|horizontal|both>` attribute, or
  hardcodes vertical-only with horizontal reserved for a later
  DD. The Phase 4 gallery sub-screen scrolls vertically only
  (per the wireframe overflow strip:
  [m3-gallery-wireframe.html](../../references/m3-gallery-wireframe.html)).
  Exposing scroll-axis admits configurations (horizontal-only,
  both-axes) with no acceptance criterion calling for them; the
  both-axes case in particular adds significant measure-arrange
  spec content (content measured unbounded on both axes,
  diagonal clamping, scrollbar geometry implications) that A5's
  "minimal" wording rejects. Framing recommendation:
  **hardcode vertical-only scroll axis in Phase 4** — scroll-axis
  attribute is **not** exposed. Mirrors Phase 3 DD-002's
  hardcode-horizontal-main-axis decision. A later phase that
  needs horizontal or bidirectional scroll opens its own DD and
  adds the attribute additively.

**Layering with DD-002 / DD-005.** The 1-child rule names *what* a
ScrollView contains; DD-002 settles the viewport bound; DD-005
names *how* the content is measured against that bound. An
Option in DD-001 that admits N>1 children does **not** contradict
DD-005 structurally — an implicit synthetic wrapper (e.g.
`ScrollView { N children }` lowered to `ScrollView { VStack { N
children } }` at IR-load time) would let DD-005's single-content
measure pass apply to the wrapper. But it **falls outside the
recommended minimal single-content shape** because the
implicit-wrapper semantics would require additional normative
spec content (which wrapper, how its attributes default, how it
composes with the author's explicit children) with no acceptance
criterion in A5 calling for it. Any Option that hides the
scroll-axis choice from the IR (e.g. infers from parent shape)
contradicts the spec's normative requirement that scroll axis is
a static IR property — that contradiction is structural, unlike
the N>1 case.

**Inputs consumed.** [pre-doc-inputs.md §1](constraints.md)
(no lexer-surface extension — `scroll-axis` would have been a
kebab-case `Ident` already accepted by the post-Phase-3 lexer; the
hardcode recommendation makes this moot);
[pre-doc-inputs.md §3](constraints.md) (no new `IrNode`
variant — generic IR shape with `WidgetKind::ScrollView` tag, per
Phase 3 reuse pattern).

### DD-M3-P4-002 — Viewport size source

ScrollView's outer extent (the "window" through which content is
viewed) must come from somewhere. The candidate sources are
parent constraint passthrough, an explicit attribute pair
(`viewport-width: <i32>` / `viewport-height: <i32>` or similar),
or a hybrid (parent on cross axis, attribute on scroll axis).

Sub-issues:

- **Default source.** Options for the ADR Options table:
  - (a) Parent constraint passthrough on both axes (WPF /
    Compose / CSS default for `overflow: scroll` block-level
    elements). ScrollView fills its parent slot; the gallery's
    sub-screen ScrollView gets its viewport size from whatever
    surrounding HStack / VStack hands it.
  - (b) Explicit attribute pair, no passthrough. Author must
    declare viewport dimensions; ScrollView ignores parent
    constraint. Phase 3's `item-cross-size` precedent (DD-004
    Option-conditional default).
  - (c) Hybrid: parent passthrough on cross axis, explicit
    attribute on scroll axis (or vice versa). Captures the
    "I want a fixed-height scroll region inside a fluid-width
    container" pattern.
- **Behaviour under unbounded parent.** If DD-002 picks (a)
  passthrough and the parent's scroll-axis constraint is itself
  unbounded (e.g. ScrollView inside an intrinsic-sizing measure
  pass — Phase 5 Grid star sizing's pre-resolution measure is
  the candidate context), what happens? Options: (i) layout-time
  runtime error analogous to Phase 2 `BoxNoExtent` /
  `BoxAspectUnboundedBoth`; (ii) degenerate to viewport-equals-
  content (no scrolling); (iii) reserved (defer to whichever
  phase introduces the unbounded-parent context). Framing
  recommendation: **(i) runtime error**, new
  `LayoutError::ScrollViewUnboundedAxis` variant. ScrollView's
  scroll axis being unbounded is structurally meaningless (no
  bound to scroll *to*); the no-silent-dropout virtue that
  Phase 2 chose for `BoxNoExtent` transfers here, unlike
  WrapPanel's one-line-flow which had a defensible reading.
- **Bindable surface (if attribute exposed).** Constant-only in
  Phase 4, mirroring Phase 3 DD-003 / DD-004 and the Phase 1 /
  Phase 2 seam-building discipline. A phase that needs animated
  viewport sizing opens the per-type writer seam at that point —
  but the writer seam built for DD-003 (content offset) would
  cover it, so this collapses to "no new seam work needed if
  DD-003 admits in-out i32".

**Recommendation direction (for framing alignment):** ship
**Option (a) — parent constraint passthrough on both axes** as the
default; no `viewport-*` attribute in Phase 4. The gallery
sub-screen wraps ScrollView in an HStack / VStack that sets the
slot the ScrollView fills, which matches the WPF / Compose
convention readers will arrive with. If a later phase needs the
explicit-viewport surface, it opens a follow-up DD and adds the
attribute additively (the IR loader and Composition primitive code
do not need to be re-architected — viewport extent is already
runtime-derivable). The runtime-error sub-issue stays in scope
even with Option (a) because the unbounded-parent case is still
reachable (Phase 5 Grid star sizing's intrinsic measure pass).

**Inputs consumed.** [pre-doc-inputs.md §2](constraints.md)
(viewport-sized minimal ScrollView, pin explicitly; default-bound
source as Phase 4 ADR question);
[pre-doc-inputs.md §3](constraints.md)
(`LayoutError::ScrollViewUnboundedAxis` candidate addition,
runtime-only — host-visible ABI stays internal until a host
observes it).

### DD-M3-P4-003 — Content offset surface and binding direction (load-bearing)

ScrollView's content offset (`offset-y` for the vertical-only
recommendation from DD-001) is the runtime's first attribute
where the **runtime would natively want to write back into the
bound state** (the user scrolls → offset updates). Whether
Phase 4 builds the writer seam, ships a read-only binding, or
defers binding entirely is the load-bearing Phase 4 DD.

This DD also settles the literal shape (`i32` pixels vs `f64`
ratio) and whether a new `PropertyValue` variant is required.

Sub-issues:

- **Literal shape.** `i32` pixels (reuses Phase 1 / Phase 2 /
  Phase 3 plumbing, no new `IrType`, no new `IrLiteral` variant,
  no new `PropertyValue` variant — the per-type writer seam, if
  built, is for `i32`); `f64` ratio in `[0.0, 1.0]` (introduces
  `f64` as a fourth scalar, with `IrType::F64`, `IrLiteral::F64`,
  evaluator surface, and at minimum a reader seam — and a writer
  seam if DD-003 admits in-out). Framing recommendation: **`i32`
  pixels**. `f64` ratio is conceptually cleaner for some
  scrollbar implementations but pressures `TypedValue` deferral
  (F5) far harder than necessary for a minimal Phase 4. The
  ratio shape can be added in a later phase as a sibling
  attribute (`offset-ratio: <f64>`) without breaking the `i32`
  surface; the reverse — starting with `f64` and adding `i32` —
  would leave the ratio shape as the canonical one with `i32`
  as an awkward second-class addition.
- **Binding direction.** Options:
  - (a) **Constant-only.** Phase 4 ships `offset-y: <i32>` as
    a static attribute. Programmatic scrolling and user-driven
    scrolling are both deferred (the gallery's overflow proof
    becomes purely visual: the viewport is positioned at a
    fixed offset that demonstrates clipping, but does not move).
    Maximum conservatism; defers all binding work.
  - (b) **Bindable read-only.** Phase 4 admits
    `offset-y: \{state.scroll_y}` (per Phase 1 / Phase 2 /
    Phase 3 binding syntax); the binding is one-directional
    (state → attribute). The runtime reads the state on each
    update and applies the offset; user-driven scroll is **not**
    yet implemented (no wheel handler, no drag — per A5's
    deferral to M4), so the offset's value only changes via
    programmatic state mutation. No writer seam built. The
    gallery sub-screen can demonstrate programmatic scrolling
    via a button that mutates `state.scroll_y`.
  - (c) **Bindable in-out.** Phase 4 admits the binding as
    bidirectional. The runtime writes back when the offset
    changes (which, in Phase 4, only happens via the
    layout-time clamp — `[0, content_size - viewport_size]` —
    since no input handler exists to scroll). The writer seam
    for `i32` is built. The gallery demo is identical to (b)
    in visible behaviour for Phase 4, but the seam is in place
    for M4's wheel / drag handlers to wire up.
- **Clamping semantics.** Regardless of binding direction, the
  applied offset is clamped to `[0, max(0, content_size -
  viewport_size)]` (zero when content fits within viewport).
  Over-scroll and under-scroll are **not** admitted in Phase 4
  (those are touch-flick / bounce behaviours and are M4 input
  territory). If DD-003 picks (c) in-out and the clamp differs
  from the bound state's value, the runtime writes back the
  clamped value — this is the writer seam's only Phase 4
  trigger.
- **Default when attribute absent.** `offset-y: 0` by default
  (top of content visible). Applied at the widget-catalog
  constructor layer, not the IR loader's `unwrap_or` —
  inheriting Phase 3 T5's discipline
  ([pre-doc-inputs.md §4](constraints.md)).

**Recommendation direction (for framing alignment):** ship
**Option (b) — bindable read-only `i32` offset** as the Phase 4
default. Rationale:

- (a) constant-only ships a visibly static ScrollView, which is
  technically A5-compliant (clip is the load-bearing visible
  proof, not motion) but produces a sub-screen that does not
  demonstrate the binding pathway A5's "content offset binding"
  phrasing names. Acceptable fallback if framing alignment
  determines the writer seam is too large for Phase 4 and the
  read-only seam alone introduces other complexity (it doesn't —
  the i32 reader seam already exists).
- (b) bindable read-only matches A5's wording exactly (binding
  is present; direction left unspecified), reuses the existing
  i32 reader, requires no new `PropertyValue` / `IrType` /
  `IrLiteral` variants, and leaves the writer-seam question for
  M4 where the input handlers actually need it. The Phase 1 /
  Phase 2 seam-building discipline ("build the seam in the phase
  that needs it") points here.
- (c) bindable in-out is the most architecturally complete answer
  and is the natural shape M4 will need, but Phase 4 has no input
  handler to *exercise* the writer direction (only the layout-
  time clamp triggers it, which is rare and not visually
  observable in the gallery). Building the seam ahead of need
  violates the seam-building discipline and Phase 4 close cannot
  produce evidence of the writer working in a visually meaningful
  way. Recommend deferring (c) to M4-Phase 1 (input handling,
  wheel / drag wiring); the framing records (c) as the explicit
  M4 hand-off rather than carrying it as a Phase 4 open question.

**Scoping intent — Phase 4 `offset-y` is not the future scroll model.**
Phase 4's `offset-y` binding is a **bindable control surface** for
proving that viewport offset traverses the DSL / IR / runtime path
(A5's "content offset binding" component). It does **not** make
state-bound offset the only — or even the primary — future
ScrollView model. M4 and beyond may additively add **input-driven
internal scrolling** (wheel / drag / keyboard PgUp gestures
mutating the offset without traversing author-bound state),
**optional state write-back / in-out binding** (the deferred (c)
shape from this DD), **scrollbar widget synchronization**, and
**imperative `scroll_to(x, y)` / `scroll_by(dx, dy)` command
surface** on the host-facing API. All four are additive on top of
the Phase 4 surface — they do not require Phase 4's `offset-y`
attribute to be removed, renamed, or re-semanticised. The Phase 4
ADR and §4.11 spec text should therefore present `offset-y` as
*one* control surface (the bindable one), not as the canonical
or definitive one. The ADR's M4 hand-off section enumerates the
four candidates above so the M4 input / scrollbar work has a
named landing point.

**Layering with DD-002 / DD-004 / DD-005.** Clamping bound
comes from DD-002 (viewport size) and DD-005 (content measured
size). Composition primitive that applies the offset
(`Visual.Offset` vs `Visual.TransformMatrix`) is DD-004; with
`i32` pixels and no animation, `Visual.Offset` is sufficient
and is the framing recommendation. DD-005's per-pass arithmetic
re-applies the clamp on every layout pass (window resize, content
size change, programmatic state mutation via the binding).

**Inputs consumed.** [pre-doc-inputs.md §3](constraints.md)
(per-type writer seam pressure; `i32` reuses existing plumbing,
`f64` ratio would not; binding direction read-only vs in-out
called out as a Phase 4 ADR question);
[pre-doc-inputs.md §4](constraints.md) (default at widget-
catalog constructor); [pre-doc-inputs.md §10](constraints.md)
(offset binding direction belongs in spec / ADR; clamping
semantics belong in spec).

### DD-M3-P4-004 — Clip surface installation and Composition primitive choice

A5 names "viewport clip" as a load-bearing component. Phase 3 T8
established that **WrapPanel installs no clip surface**
([m3-phase-3 ADR DD-005 oversized-line section](../../phase-3/decisions/preamble.md));
ScrollView is the **dual** — it must install a clip surface
because the gallery's overflow state
(`ScrollView { content … }`) is exactly where the
"parent clips" contract Phase 3 deferred to becomes active.

Sub-issues:

- **Clip primitive.** Options:
  - (a) `Visual.Clip = InsetClip { 0, 0, 0, 0 }` sized to the
    viewport (an InsetClip whose insets are all zero and whose
    extent matches the parent Visual is the canonical
    Windows.UI.Composition pattern for "clip to my own bounds").
  - (b) `Visual.Clip = RectangleClip { left: 0, top: 0, right:
    viewport_w, bottom: viewport_h }` (explicit rectangle —
    `CompositionGeometricClip` with a `CompositionRectangleGeometry`,
    or whatever the Windows.UI.Composition API exposes for
    rectangle clipping).
  - (c) `Visual.Clip = InsetClip` with non-zero insets
    derived from a future `padding` attribute (over-engineered
    for Phase 4; padding is out of scope per A5 minimal).
- **Offset application primitive.** Options:
  - (a) `Visual.Offset` on the child container Visual (the
    inner Visual that owns the content WidgetNode's
    SpriteVisual). Mutation = `SetOffset(0, -offset_y, 0)`
    (negative because moving the content up exposes lower
    content through the viewport).
  - (b) `Visual.TransformMatrix` on the same Visual. Mutation
    = `SetTransformMatrix(Matrix4x4.CreateTranslation(0,
    -offset_y, 0))`.
  Framing recommendation: **(a) `Visual.Offset`**. `i32` pixels
  + no animation makes TransformMatrix's value-add (fractional
  offsets, animation compositability) non-load-bearing for
  Phase 4. M4's animation / smooth-scroll work can switch to
  TransformMatrix without breaking the surface.
- **Where the clip lives.** ScrollView's own Visual (outer)
  carries the clip; the content Visual is a child of the outer
  Visual whose `Visual.Offset` carries the scroll position.
  This is the natural Composition tree shape: outer Visual ==
  viewport (clipped), child Visual == scrollable canvas
  (translated). Verified compatible with the existing
  parent-relative `Visual.Offset` convention in
[architecture.md §6.5](../../../../docs/architecture.md#65-widgetnode-and-visual-layer-sync).
- **Interaction with R2 (Phase 3 carry-over).** Phase 3 T9
  surfaced a `sync_visuals` bug whose root cause was the
  implicit absolute-vs-parent-relative offset convention.
  R2 (test-coverage half) was filed as open
  ([pre-doc-inputs.md §8](constraints.md), [§9](constraints.md)).
  Phase 4 touches the same boundary meaningfully: the content
  offset changes at runtime, so the relative-offset translation
  is exercised on every scroll. Framing decision F below settles
  whether R2 closes inside Phase 4 or carries forward.

**Recommendation direction (for framing alignment):** ship
**(a) `Visual.Clip = InsetClip { 0, 0, 0, 0 }`** for the clip and
**(a) `Visual.Offset`** for the content offset. Both match the
existing M2 visual-layer conventions and require no new
Composition primitives. The integration test
([framing decision C](#c-verification-strategy)) asserts clip
**presence** on the ScrollView Visual and clip **absence** on the
inner content's WrapPanel / Box Visual (symmetric inverse of
Phase 3 T8 — the assertion shape is the same code with `assert!`
inverted).

**Inputs consumed.** [pre-doc-inputs.md §7](constraints.md)
(clip presence as positive evidence — the inverse of Phase 3 T8);
[pre-doc-inputs.md §8](constraints.md) (re-read §6.5 before
deciding Visual.Offset vs Visual.TransformMatrix; recommendation
is Visual.Offset for Phase 4);
[pre-doc-inputs.md §9](constraints.md) (R2 in-or-out
disposition — settled by framing decision F).

### DD-M3-P4-005 — Measure-arrange algorithm (novel normative viewport / content / offset semantics)

Introduces novel normative spec content into `docs/dsl_spec.md`
of a different *kind* than Phase 3 (no line-formation algorithm)
and lighter in *scope* than the upcoming Phase 5 Grid star sizing
which is the milestone's "second novel-normative-spec phase" proper.
The DD settles the content-measure pass, viewport-vs-content size
relationship, and offset clamping; the ADR section is also the
**seed** of the dsl_spec chapter (Moment 1 lands the spec chapter
in design-spec-draft form; Moment 2 re-syncs to implementation
findings).

The algorithm is structurally simpler than Phase 3 WrapPanel's
two-stage measure-arrange — ScrollView has one child, no line
formation, no multi-pass measurement. The novel content is the
**unbounded-axis + bounded-axis asymmetric input** to the
content's measure pass, plus the **offset clamp** semantics that
have no analogue in Phase 1–3 surfaces.

Sub-issues:

- **Content measure pass.** Content is measured with a
  constraint of `(viewport_width, +∞)` — bounded cross axis
  (= viewport width per DD-001's vertical-only recommendation),
  unbounded scroll axis. This is the inverse of WrapPanel's
  measure input (WrapPanel: unbounded main + DD-004-derived
  cross). The DD names the constraint construction explicitly
  so DD-001's "vertical-only" implies "scroll axis = vertical,
  unbounded direction = vertical, viewport-equals-cross-axis-
  bound = width".
- **Viewport vs content size relationship.**
  - `content_size_scroll_axis <= viewport_size_scroll_axis`:
    content fits within viewport. Offset is clamped to 0
    (no scrolling possible).
  - `content_size_scroll_axis > viewport_size_scroll_axis`:
    content exceeds viewport. Offset is clamped to
    `[0, content_size - viewport_size]`. Visible content is
    `[offset, offset + viewport_size)` along the scroll axis.
- **Offset application.** After the content measure, the
  content's resolved rect is translated by `(0, -offset)`
  (in absolute layout-engine coordinates, before
  `sync_visuals()` converts to parent-relative). The content's
  outer rect within ScrollView's local space is then
  `(0, -offset, content_w, content_h)`; the clipping is the
  rendering-side operation owned by DD-004's Composition clip.
- **ScrollView outer size.** Equals viewport size, regardless
  of content size. Cascading parent-bound violations are
  excluded — even if content size exceeds parent's slot,
  ScrollView's outer size stays at viewport. This is the
  Phase 4 analogue of Phase 3 DD-005's "WrapPanel outer
  main-axis size does not grow to accommodate oversized
  children" rule.
- **Content-smaller-than-viewport behaviour.** Content paints
  at its measured size, anchored at the viewport's top-leading
  corner (`(0, 0)` in viewport-local coordinates). The
  remaining viewport area shows ScrollView's background (the
  parent's `fill` if no ScrollView-level background attribute is
  exposed — Phase 4 does not introduce a `fill` on ScrollView).
  Offset is forced to 0 by the clamp.
- **Unbounded scroll-axis parent.** Per DD-002 framing
  recommendation, this fires `LayoutError::ScrollViewUnboundedAxis`
  at layout time. No degenerate Phase 4 ScrollView shape:
  unbounded scroll axis is structurally meaningless.
- **Rounding contract.** Inherits Phase 2 DD-005 / Phase 3
  DD-005: `f32` for layout engine internals, `i32` for
  attribute literals, promoted to `f32` at comparison. No
  pixel-snapping in Phase 4. `i32` offset is promoted to
  `f32` for the clamp arithmetic.
- **LayoutError surface.** New
  `LayoutError::ScrollViewUnboundedAxis` variant per DD-002
  recommendation. ABI / host-visible surface stays internal
  per [pre-doc-inputs.md §3](constraints.md) — no
  `WASAMO_LAYOUT_ERROR_*` ABI tag added unless a host can
  meaningfully observe the new variant (it cannot in Phase 4;
  the host receives layout failure as opaque).

**Layering with DD-001 / DD-002 / DD-003 / DD-004.** The
algorithm assumes:
- A 1-child ScrollView (per DD-001).
- A viewport size from DD-002 (parent passthrough by default).
- An offset value from DD-003 (read-only `i32` binding by
  default, clamped per the rule above).
- A clip + offset application via Composition primitives in
  DD-004 (Visual.Clip + Visual.Offset).

Any Option in DD-005 that re-derives any of these contradicts
the chain. In particular, an Option that re-measures the content
with a *bounded* scroll-axis constraint (to "fit" content into
viewport) contradicts A5's "inner unbounded measure" load-bearing
phrasing.

**Inputs consumed.** [pre-doc-inputs.md §2](constraints.md)
(ScrollView pairing contract with WrapPanel is already in Phase 3
ADR DD-M3-P3-005; Phase 4 confirms ScrollView passes "unbounded"
through cleanly to its content's cross-axis input — for a
ScrollView { WrapPanel { … } } gallery use, ScrollView's
vertical-only direction means WrapPanel receives bounded main =
viewport width and unbounded cross axis, consistent with Phase 3
DD-004's parent-passthrough default plus the gallery's explicit
`item-cross-size: 88` settling the actual child bound);
[pre-doc-inputs.md §6](constraints.md) (bounded-vs-unbounded
fork in measure-arrange; pin both branches with reject tests —
the framing decision C verification mix below names this);
[pre-doc-inputs.md §10](constraints.md) (spec-drafting bar
applied to viewport / content / offset spec items).

### DD-M3-P4-006 — IR-loader defense-in-depth invariants

Phase 2 T7 surfaced the principle: IR-load → runtime-materialise
invariants belong in pure-logic `validate()`, not in WinRT-bound
`build_node`. Phase 3 T6 extended this with WrapPanel's value-
range invariants. Phase 4 extends it with ScrollView's invariants,
which are a **different shape** than either Phase 2 (structural
placement) or Phase 3 (value range) —
[pre-doc-inputs.md §5](constraints.md) names this explicitly.

Sub-issues:

- **Child count.** Per DD-001 recommendation, ScrollView admits
  exactly 1 child. `validate()` rejects 0 children and rejects
  >1 children with `WASAMO_ERR_IR_MALFORMED`. This is the
  **structural** half of the invariant shape (Phase 2's pattern);
  the value-range half is the next sub-issue.
- **Offset value range.** Per DD-003 recommendation, `offset-y`
  is `i32`. `wasamoc check` rejects values outside `i32` at
  compile time (already-existing `IntLit` handling). The
  Phase 3 DD-006 "negative literal rejection" pattern **does
  not apply** to ScrollView's offset — negative offsets are
  layout-time-clamped to 0 per DD-005, not IR-rejected; an
  author may bind a `state.scroll_y` that legitimately reaches
  negative values during state transitions. The two-gate
  defense-in-depth pattern still applies, but the runtime
  gate is the **clamp in DD-005's arrange pass**, not a
  validate-time rejection. This is the **value-range half**,
  shaped as runtime-clamp rather than runtime-reject —
  the per-pre-doc-inputs.md §5 distinction.
- **Bound-direction validation (conditional on DD-003).** If
  DD-003 picks (c) in-out binding, `validate()` must check the
  bound state is mutable (writable). This is currently outside
  the IR's vocabulary (M2 / Phase 1 bindings did not need a
  mutability discriminator); the validation can defer to
  `wasamoc check`. Conditional on DD-003 — collapses if the
  recommendation (b) read-only is chosen.
- **Error class.** All ScrollView invariant violations surface
  as `WASAMO_ERR_IR_MALFORMED`, consistent with Phase 2 / Phase
  3 precedent.

**Inputs consumed.** [pre-doc-inputs.md §5](constraints.md)
(runtime-gate scope follows the phase's invariant shape; not
inherited from Phase 3's value-range pattern by default —
ScrollView is *compound* in this sense: structural child-count
gate + runtime-clamp for offset, not a validate-time
value-range reject for offset).

---

### Out of scope (to be carried in the ADR's Out-of-scope section)

- **Scrollbar widget**, wheel handler, drag — A5 explicit
  deferral to M4. Phase 4 sub-screen demonstrates programmatic
  scroll (via a button or auto-set state), not user-input scroll.
- **Horizontal and bidirectional scroll axes** — DD-001 hardcodes
  vertical-only; later phase adds attribute additively.
- **`viewport-width` / `viewport-height` attributes** — DD-002
  defers; parent passthrough is the Phase 4 path.
- **`f64` / ratio offset surface** — DD-003 ships `i32` pixels;
  ratio is a sibling future addition.
- **In-out offset binding (writer seam)** — DD-003 ships
  read-only; writer seam is an M4 hand-off named in the ADR's
  M4 hand-off section.
- **Future scroll-model surfaces (M4+).** Per the DD-003 scoping
  intent paragraph, Phase 4's `offset-y` is one control surface;
  the ADR's M4 hand-off section enumerates the additive future
  surfaces that may land beyond Phase 4 without disturbing it:
  - **Input-driven internal scrolling** (wheel / drag / keyboard
    PgUp / Home / End handlers mutating the offset directly
    inside ScrollView, without traversing author-bound state).
  - **Optional state write-back / in-out binding** — the
    deferred DD-003 (c) shape; requires the typed-`i32` writer
    pair from framing decision A.
  - **Scrollbar widget synchronization** — a separate widget
    (likely `ScrollBar` as a sibling primitive, not built into
    ScrollView) whose position both reflects and drives the
    ScrollView offset.
  - **Imperative `scroll_to(x, y)` / `scroll_by(dx, dy)` command
    surface** on the host-facing API, for programmatic-without-
    state-binding scrolling (the analogue of WPF's
    `ScrollViewer.ScrollToVerticalOffset` or SwiftUI's
    `ScrollViewReader`).

  None of these require modifying Phase 4's `offset-y` attribute,
  IR shape, or default behaviour.
- **Over/under-scroll**, bounce, momentum — touch-flick / smooth-
  scroll territory, M4 input + animation.
- **Background `fill` on ScrollView** — Phase 4 does not
  introduce a ScrollView-level `fill` attribute; the visible
  background is whatever parent / sibling provides.
- **Nested ScrollViews** — structurally permitted (nothing in
  the IR or layout forbids it), but Phase 4 ships no test
  fixture or sub-screen exercising the case. The unbounded-
  parent runtime error from DD-002 covers the pathological
  inner ScrollView whose parent is itself an unbounded
  ScrollView.
- **Image widget as scroll content** — Image deferred to M4
  per Phase 2 DD-006 / M3 plan; Phase 4 sub-screen content is
  Box + Text placeholders.
- **TypedValue generic value union** (F5 maintained — Phase 4
  introduces no new scalar type per DD-003 recommendation).
- **Padding on ScrollView** — out of A5 minimal; defer to
  later phase if needed.

---

## Owner-agreed framing decisions

### A. Typed-`i32` writer pair — defer to M4

The Phase 4 framing recommends **deferring the typed-`i32`
evaluator / writer pair to M4** (DD-003 recommendation = read-only
`i32` binding, using the existing M2 string-baked dispatch path
for `IrType::I32` properties; no new
`register_i32_binding_with_writer` sibling built). Rationale
stated at DD-003; recorded as a separate framing decision because
it has cross-DD implications:

- DD-006 collapses the "bound state mutability" sub-issue
  (conditional on DD-003 in-out being chosen — not chosen, so
  no sub-issue).
- The ADR's M4 hand-off section explicitly names the typed-`i32`
  writer-pair build as M4-Phase 1 (input handling) work; the pair
  built then will also unblock any later phase that needs to
  write back into an `i32`-bound attribute, additively per the
  architecture.md §6.7 *Per-type seam* paragraph.
- Phase 4 close cannot produce evidence of the writer working in
  a visually meaningful way; the closure item set
  (framing decision C) reflects this by not asking for writer
  evidence.

The framing alignment may override this: if owner determines the
writer seam should be built in Phase 4 anyway (e.g. to make the
M4 wire-up trivial, or to surface ABI-side discovery early), the
ADR's DD-003 Recommendation reverses and the closure item set
gains a writer-seam evidence item.

### B. DD slate completeness check

Per [process/README.md §Pre-doc discipline](../../../README.md),
the framing must verify that the proposed DD slate serves A5,
not merely execute the m3-plan task description literally. Check:

- A5 enumerates three minimal-scope components: **inner
  unbounded measure** (DD-005), **viewport clip** (DD-004),
  **content offset binding** (DD-003). All three map directly to
  load-bearing DDs.
- DD-001 (IR shape + axis) and DD-002 (viewport size source) are
  prerequisite for DD-003 / DD-004 / DD-005 to be formulated
  without ambiguity.
- DD-006 covers IR-loader defense-in-depth, mirroring Phase 2 T7
  / Phase 3 T6 precedent.
- No surface beyond A5 is added: no scrollbar, no input handler,
  no padding, no horizontal / both axis, no f64 / ratio.
- **Bindable-surface DD folded.** Phase 4 follows the Phase 3
  pattern: the binding question is folded into DD-003 (the only
  DD where it is load-bearing) rather than carried as a separate
  DD; DD-002's viewport-size sub-issue collapses to constant-only
  by inheritance (no Phase 4 attribute admits binding except
  offset).

### C. Verification strategy

Per [m3-plan.md §Verification strategy](../../plan.md#verification-strategy),
Phase 4 chooses from the menu:

- **`wasamoc` check-side pure-logic tests** for compile-time
  diagnostics (DD-006 child-count rejection; `IntLit` parsing
  for `offset-y`). Already-existing infrastructure for `IntLit`;
  the new evidence is the structural rejection of 0-child /
  >1-child ScrollView.
- **Pure-logic unit tests** for the measure-arrange algorithm
  (DD-005). The arrange pass is the place where the
  content-fits-in-viewport branch, the content-exceeds-viewport
  branch, the clamp arithmetic, and the unbounded-scroll-axis
  `LayoutError::ScrollViewUnboundedAxis` are exercised.
  **Pin both bounded and unbounded branches with reject tests**
  per [pre-doc-inputs.md §6](constraints.md) — the bounded-
  vs-unbounded fork in ScrollView is the viewport-bounded
  cross-axis vs content-unbounded scroll-axis.
- **Pure-logic unit tests** for IR-loader invariants (DD-006
  child-count gate via `validate()`).
- **Mock-free Windows-only integration test** (CI-gated, fails
  rather than skips per
  [CLAUDE.md §Testing rules](../../../CLAUDE.md)) for live
  ScrollView materialisation through `.ui → IR → runtime`. The
  integration test asserts:
  - clip **presence** on the ScrollView Visual (the
    symmetric inverse of Phase 3 T8's clip-absence assertion);
  - clip **absence** on the inner content Visual (regression
    guard: ScrollView's clip presence must not have caused the
    content widget to also acquire a clip);
  - `Visual.Offset` on the content Visual changes to `(0,
    -offset_y, 0)` when the bound state is mutated (DD-003 (b)
    read-only path: programmatic mutation triggers the binding
    propagation and the sync_visuals re-application).
- **Visible smoke** via the ScrollView sub-screen in
  `examples/gallery/` + `examples/gallery-rust/` (framing
  decision E) for owner-manual GUI smoke (framing decision G).

Per [pre-doc-inputs.md §10](constraints.md), evidence items
do not collapse just because they share helper infrastructure —
the `wasamoc` check-side tests, in-crate measure-arrange tests,
IR-load `validate()` gate tests, and Windows integration tests
each have distinct evidence meanings.

### D. Upstream-document revision timing (two sync moments)

Phase 4 inherits the two-moment structure from
[m3-phase-2 framing decision D](../../phase-2/requirements/framing.md#d-upstream-document-revision-timing-two-sync-moments)
and Phase 3's same-shape inheritance, but follows the current
living-rule doc set and commit shape in
[retrospectives.md](../../../procedures/retrospectives.md) (per its operational
note: doc set and commit shape have been updated after Phase 2 /
Phase 3 実運用, and retrospectives.md is the living rule for
phase-end execution; framing decisions inherit the *structure*,
not the historical doc list verbatim).
The Phase 4 `dsl_spec.md` section marker mirrors the Phase 2 /
Phase 3 form:

```
**Phase status:** M3-Phase 4 ADR-accepted design draft; pending
implementation re-sync
```

flipping at phase close to:

```
**Phase status:** M3-Phase 4 closed; implementation-synced
```

placed as the first line under the ScrollView chapter heading
(new §4.11 alongside Phase 2's §4.9 Box and Phase 3's §4.10
WrapPanel chapters). The chapter appears as the **design-spec
draft** in Moment 1 (ADR-Accepted commit) and is re-synced in
Moment 2 (phase close).

**Moment 1 — ADR Accepted commit set (design-spec draft).**
Constituent commits, each landing as its own commit on the
pre-doc branch per the per-review-concern rule in
[CLAUDE.md §Commit rules](../../../CLAUDE.md#commit-rules) and
[retrospectives.md](../../../procedures/retrospectives.md). The draft-side doc
set Phase 4 commits to at Moment 1 is enumerated below
(retrospectives.md §phase-sync で触る doc セット規定は phase-end
Moment 2 を対象とした規範であり、Moment 1 の draft set はその
mirror として **直接同一視されるものではない**; Phase 4 が
Moment 1 で触れる文書はここで明示的に列挙する):

- `process/milestone-3/phase-4/decisions/preamble.md` — ADR
  `Status: Accepted` flip.
- `docs/dsl_spec.md` — new §4.11 ScrollView chapter as
  design-spec draft. No new tokens, grammar rules, or AST
  variants per DD-001 / DD-002 / DD-003 framing
  recommendations (`offset-y` reuses existing kebab-case
  `Ident` + `IntLit`).
- `docs/architecture.md` — ScrollView entry under the M2-revised
  IR section; layout engine section updated for the new
  pure-data ScrollView measure-arrange types; §6.5 may receive
  a one-paragraph addition naming Visual.Offset as the
  scroll-position primitive (cross-reference target for DD-004).
- `docs/abi_spec.md` — **no touch expected** in Phase 4 per
  DD-005 / DD-006 recommendations (LayoutError stays internal;
  no new ABI tag).
- `docs/plans/m3-plan.md` — Progress section's Phase 4 row
  populated (Status: in progress; Progress file link; ADR link).
- `docs/plans/progress/m3-phase-4-progress.md` — new file opened
  with task list mapped to ADR's verification closure items.

Implementation begins only after these commits land; the
constituent shape preserves review-concern separability under
[CLAUDE.md §Commit rules](../../../CLAUDE.md#commit-rules).

**Moment 2 — Phase close commit set (impl re-sync).**

- `docs/dsl_spec.md` §4.11 — section marker flips to "closed;
  implementation-synced", plus any corrections required if the
  design draft and implementation diverged (marker flip is
  required regardless of divergence; corrections are conditional
  on what re-sync surfaces).
- `docs/architecture.md` — top Status flips to
  `M3-Phase 4 complete`; impl-divergent paragraphs re-synced.
- `docs/plans/progress/m3-phase-4-progress.md` — phase-close
  retrospective link, CI evidence pointer, impl summary.
- `docs/plans/m3-plan.md` Progress row — Status flips to
  complete.
- `process/milestone-3/phase-4/decisions/preamble.md` — touch only if
  one of the three retrospectives.md §phase-sync ADR-touch
  cases applies (AC discharged-vs-impl divergence; out-of-phase
  residual cross-ref; thesis-level finding).
- Step retro `phase-sync` items (per
  [retrospectives.md item 10](../../../procedures/retrospectives.md#step-end-固有-merge--phase-ブランチ))
  must all close into `doc-folded` / `carry-forward` /
  `local-only` at Moment 2 — no open `phase-sync` items
  survive past phase close.

### E. Phase 4 visible proof — sibling `ScrollView { WrapPanel { … } }` slice with fixed thumbnails

The Phase 4 visible proof grows
[examples/gallery/gallery.ui](../../../../examples/gallery/gallery.ui)
by **adding a sibling section that wraps a fixed thumbnail set
inside the canonical `ScrollView { WrapPanel { Box × N } }`
composition**. The existing Phase 3 standalone WrapPanel slice
**stays in place unchanged** — per
[pre-doc-inputs.md §11](constraints.md) and Phase 3 framing
decision E's "sub-screen per phase" principle, modifying the
Phase 3 slice would obscure its standalone wrap evidence.

The sibling sub-screen composition:

- **Content shape.** The canonical `ScrollView { WrapPanel { Box
  × N } }` composition. Per
  [m3-plan.md §Phase dependencies](../../plan.md#phase-dependencies),
  the thumbnail-strip chain culminates at this composition, and
  Phase 7's iteration grammar proof is sequenced to *swap in*
  collection-driven generation of the same composition as a
  strict superset. Proving the composition with a fixed thumbnail
  set in Phase 4 is what makes Phase 7 a superset rather than the
  initial composition proof.
- **Item count.** Enough to require **both wrap and vertical
  scroll** on the default 800×600 window. Per
  [pre-doc-inputs.md §11](constraints.md), Phase 3's 88×88
  thumbnails with 12px spacing wrap 7-per-row at the default
  client width; a ~400px-tall ScrollView viewport fits ~4 rows.
  Total content needs > 4 rows for scroll to be visible.
  Recommend **30–40 items (5–6 rows)** to leave headroom for
  visual obviousness — the wireframe overflow strip is the
  reference target.
- **Programmatic scroll demo.** Per DD-003 recommendation (b)
  read-only binding, the sub-screen includes a control (e.g. a
  pair of Button widgets, "scroll up" / "scroll down") that
  mutate `state.scroll_y` by ±100 per click. This is the visible
  evidence that the binding pathway is alive in Phase 4.
- **Isolation from Phase 3 slice.** The Phase 3 standalone
  WrapPanel slice (10 thumbnails, no ScrollView) and the new
  Phase 4 `ScrollView { WrapPanel { … } }` slice (30–40
  thumbnails) appear as two sibling sub-sections in
  `gallery.ui`. The Phase 3 slice continues to prove that
  WrapPanel works standalone; the Phase 4 slice proves that
  the composition works. Phase 7 will eventually replace the
  fixed thumbnail children of the Phase 4 slice with an
  iteration grammar binding.
- **Rust host only.** Per
  [m3-plan.md §Phase-end criteria item 5](../../plan.md#phase-end-criteria),
  Phase 4 ships at least one host's gallery proof;
  `examples/gallery-rust/` is the canonical one. C and Zig host
  parity comes at Phase 8 with the full gallery.

**`examples/gallery/` is still a partial gallery, not the A1
proof.** A1 acceptance lives in Phase 8 per the
[acceptance ↔ phase mapping](../../plan.md#acceptance--phase-mapping);
Phase 4 grows the gallery from Phase 3's standalone WrapPanel
slice into [standalone WrapPanel slice] + [ScrollView { WrapPanel
{ … } } slice].

### F. Phase 3 carry-over residuals — disposition

Phase 3 left two open residuals
([pre-doc-inputs.md §9](constraints.md)):

- **R1 — `.gitignore` `*.uic` addition.** Cross-cutting hygiene
  unrelated to ScrollView. Phase 4 does not touch build hygiene
  in any DD or framing decision. Disposition: **defer**, remains
  open for whichever later phase touches build hygiene
  (`.gitignore`, file extension policy, etc.).
- **R2 — `sync_visuals` ↔ pure-layout boundary test coverage
  gap.** Phase 3 T9 surfaced the absolute-vs-parent-relative
  offset convention bug; the architecture fix landed
([architecture.md §6.5](../../../../docs/architecture.md#65-widgetnode-and-visual-layer-sync)),
  but the test-coverage half was not closed in Phase 3.
  Phase 4 touches the same boundary meaningfully (DD-004's
  Visual.Offset application on every scroll). Disposition:
  **close R2 inside Phase 4** as part of the Windows integration
  test (framing decision C). The test asserting `Visual.Offset`
  changes to `(0, -offset_y, 0)` on bound-state mutation is the
  natural place to add coverage for nested non-zero-offset
  visual trees (ScrollView Visual at parent offset X, content
  Visual at offset (0, -offset_y) relative to ScrollView,
  Box thumbnails inside content at their own offsets — the
  three-level nesting Phase 3 lacked).

  The framing alignment may override this if owner determines
  R2's test coverage is broader than what Phase 4's natural
  integration test exercises (in which case R2 carries forward
  to a dedicated test-coverage pass).

### G. Live-note re-evaluation triggers — handling

[pre-doc-inputs.md §13](constraints.md) flags the
`docs/notes/*` audit items. The framing settles their disposition
upfront so the ADR Inputs section can cite settled handling
rather than re-deciding:

- **[architectural-family.md](../../../../docs/notes/architectural-family.md) — stays
  consumed.** ScrollView is a built-in primitive in the
  tree-with-bindings family, no re-evaluation needed. Phase 1 /
  Phase 2 / Phase 3 framings already established this.
- **[layout-engine.md](../../../../docs/notes/layout-engine.md) — partial fire.**
  ScrollView's measure-arrange is the next M3 phase exercising
  the layout engine's pure-data surface non-trivially (Phase 2
  Box and Phase 3 WrapPanel each touched it in their own ways;
  the count is not load-bearing here). Specific dispositions:
  - 3.1 DPI scaling — defer. Phase 4 stays in logical-pixel
    `f32` coordinate space.
  - 3.2 AccessKit sync — not applicable (M4).
  - 3.3 async measure — not applicable (Image deferred to M4).
  - 3.4 cache invalidation — Phase 4's `offset-y` binding is
    bindable (DD-003 (b)), which means **offset changes are a
    runtime layout trigger**. The existing whole-window dirty
    path (per Phase 3 framing decision F) covers it; no
    sub-tree dirty work is in Phase 4 scope. The 1,000-node
    performance threshold is not pressured by Phase 4's
    sub-screen (tens of thumbnails — Phase 3 standalone slice's
    10 + Phase 4 sibling slice's 30–40, well under 1,000).
  - 3.5 user-defined layout — not applicable.
- **[dsl-grammar.md](../../../../docs/notes/dsl-grammar.md) — mostly unfired.**
  Phase 4 ships no template-local scope, no iteration, no
  qualified state reference beyond what Phase 1 / Phase 2 /
  Phase 3 already exercise. Q1 / Q3 / Q5 remain Phase 7+.
- **[component-extension-model.md](../../../../docs/notes/component-extension-model.md) — unfired.**
  ScrollView is a built-in component.
- **[typed-value-evaluator.md](../../../../docs/notes/typed-value-evaluator.md) —
  conditional fire deferred.** DD-003 recommendation (b) read-
  only `i32` binding reuses Phase 1 / Phase 2 / Phase 3 i32
  plumbing; no new `IrType` / `PropertyValue` variant. F5
  (`TypedValue` deferral) remains. If framing alignment reverses
  DD-003 to (c) in-out, the writer seam is built — still no new
  TypedValue; the seam is per-type for i32.
- **[workspace-layout.md](../../../../docs/notes/workspace-layout.md) — unfired.**
  Phase 4 adds no new crate.
- **[verification-environments.md](../../../../docs/notes/verification-environments.md) /
  [headless-verification.md](../../../../docs/notes/headless-verification.md).**
  Phase 4 inherits Phase 2 / Phase 3's skip-guard pattern
  verbatim. Framing decision C commits Phase 4 to the
  fail-rather-than-silently-skip discipline.
- **[process-rules-ssot.md](../process-rules-ssot.md) Q6 — relevant.**
  The 3-role boundary (execution log / step retrospective /
  phase acceptance evidence) inherited from Phase 2 / Phase 3.
  Phase 4 does not introduce a new evidence document type.
- **[release-distribution.md](../../../../docs/notes/release-distribution.md) —
  unfired.** Phase 4 introduces no release / packaging surface.

### H. GUI smoke responsibility separation

Inherits [m3-phase-2 framing decision G](../../phase-2/requirements/framing.md#g-gui-smoke-responsibility-separation-predoc-inputs-5)
and Phase 3's same-shape inheritance. Visual correctness of
ScrollView rendering (viewport clips correctly; content below
viewport bottom is invisible; programmatic scroll button moves
content; clipping edge is sharp) is **owner-manual GUI smoke**.
The assistant records `Start-Process` launch command success and
any captured headless integration output but does not assert on
visual rendering. The ADR's verification strategy section
distinguishes headless test gates from owner GUI smoke gates per
Phase 2 / Phase 3 precedent.

### I. ScrollView mental model — short anchor in dsl_spec §4.11

The facts a reader needs to internalise to use ScrollView
correctly are distributed across DD-001 (1-child shape +
vertical-only), DD-002 (viewport from parent), DD-003 (offset
binding), DD-004 (clip), and DD-005 (measure-arrange). The framing
direction is to consolidate them into a **single short subsection**
in the user-facing spec, mirroring Phase 3 framing decision H's
"sizing mental model" subsection:

1. **Viewport size comes from parent.** ScrollView fills its
   parent slot on both axes; there is no `viewport-*` attribute
   in Phase 4. To control viewport size, the parent's slot must
   be sized (via the parent's own attribute / layout role).
2. **Content measures with viewport-bounded cross axis + unbounded
   scroll axis.** Content along the scroll axis can be arbitrarily
   tall; along the cross axis it is bounded by the viewport
   width. Content that exceeds the viewport on the scroll axis
   is scrollable; content that is shorter than the viewport is
   anchored at the top and does not scroll.
3. **Content offset is clamped to `[0, max(0, content_size -
   viewport_size)]`.** Out-of-range bound values are silently
   clamped on every layout pass; the bound state's *written*
   value is read-only-bound per the Phase 4 default (DD-003 (b)),
   so the source state and the applied offset may diverge — the
   author observes the displayed scroll position, not the bound
   value, as ground truth.
4. **The clip is owned by ScrollView, not by the content.**
   Content widgets remain unclipped (consistent with Phase 3 T8
   discipline); only the ScrollView Visual installs a clip
   surface. Composing two ScrollViews around the same content
   stacks two clips; composing ScrollView around an HStack
   around content does not have an HStack-level clip.
5. **`offset-y` is the Phase 4 external control surface, not the
   only future scroll model.** The bindable `offset-y` attribute
   is how Phase 4 exposes scroll position to author code; it is
   not a commitment that state-driven offset is the canonical
   way to scroll. Input-driven scrolling (wheel / drag /
   keyboard) and scrollbar-driven scrolling are M4+ work and
   land additively without redefining `offset-y` (per the DD-003
   scoping intent paragraph and the M4 hand-off section).

**Placement.** The subsection lives in `docs/dsl_spec.md` §4.11
(the new ScrollView chapter), positioned before the formal
measure-arrange algorithm so the reader builds the model before
the rules. The ADR's DD-005 Recommendation prose cross-references
this subsection rather than restating the five facts.

**Ecosystem contrast (one bullet each).** ScrollView's surface
intersects multiple ecosystem conventions; readers will arrive
carrying analogues that do not transfer cleanly:

- **WPF `ScrollViewer`** — exposes `HorizontalScrollBarVisibility`
  / `VerticalScrollBarVisibility`, content-size-driven viewport,
  built-in scrollbar widgets. Wasamo's Phase 4 ScrollView is
  scrollbar-less (M4 hand-off), viewport-from-parent, and offset-
  bindable rather than scrollbar-driven. The conceptual primitive
  ("clip + offset + measure-arrange") matches; the surface is
  narrower.
- **CSS `overflow: scroll`** — block-level element, scrollbar
  always visible regardless of content fit, content-driven
  intrinsic sizing. Wasamo's Phase 4 ScrollView has no scrollbar
  in any state, and content size does not back-propagate to
  ScrollView's outer size (which stays at viewport).
- **SwiftUI `ScrollView`** — viewport-from-parent (default),
  scroll-axis attribute (`.horizontal` / `.vertical` /
  `[.horizontal, .vertical]`), gesture-driven offset (not
  state-bound). Wasamo's Phase 4 hardcodes vertical and binds
  offset to state; gesture / wheel input is M4. The
  `.scrollPosition($state)` SwiftUI surface is conceptually
  closest to DD-003 (c) in-out, which Phase 4 defers.

**This is a docs framing decision, not a design change.** The
design recommended by DD-001 through DD-005 stands; the subsection
exists only to provide a single short anchor for the model the
recommended design implies.

---

## Inputs absorbed

### From [pre-doc-inputs.md](constraints.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §1 No new parser grammar can hide lexer-surface changes | Premise / sub-issue input | DD-001 (no lexer extension — `scroll-axis` attribute is hardcoded-deferred; `offset-y` is post-Phase-3-`Ident`-compatible kebab-case) |
| §2 ScrollView pairing already in Phase 3 ADR | Premise | DD-005 (composition with WrapPanel cross-axis input is the Phase 3 DD-005 pairing, no re-derivation); framing decision E (Phase 4 does not modify or wrap the existing Phase 3 standalone WrapPanel slice; instead adds a sibling `ScrollView { WrapPanel { … } }` slice proving the canonical composition with fixed thumbnails, keeping Phase 3's standalone wrap evidence isolated) |
| §3 New variants? PropertyValue / IrLiteral / IrType / LayoutError / ABI | Direct input | DD-003 (no new `PropertyValue` / `IrLiteral` / `IrType` for the recommended `i32` + read-only path; defers writer seam to M4); DD-005 (new `LayoutError::ScrollViewUnboundedAxis`, internal only — no ABI tag) |
| §4 Defaults at widget catalog, not IR loader | Discipline reminder | DD-003 default offset (widget-catalog constructor `WidgetData::ScrollView`); DD-006 (validate gate is structural, not default-supplying) |
| §5 Runtime-gate scope follows invariant shape | Direct input | DD-006 (compound shape: structural child-count gate + runtime-clamp for offset; explicitly *not* a value-range reject) |
| §6 Bounded-vs-unbounded fork — pin both with reject tests | Direct input | Framing decision C (verification mix: bounded cross-axis branch and unbounded scroll-axis branch both pinned in pure-logic unit tests; `LayoutError::ScrollViewUnboundedAxis` is a reject-test variant) |
| §7 ScrollView is clip-installing widget — inverse of WrapPanel | Direct input | DD-004 (clip presence as the load-bearing assertion); framing decision C (Windows integration test asserts clip presence on ScrollView + clip absence on content, symmetric inverse of Phase 3 T8) |
| §8 §6.5 architecture context for Visual.Offset vs TransformMatrix | Direct input | DD-004 (Visual.Offset recommendation); framing decision F (R2 in-scope for closure inside Phase 4 via the integration test's three-level offset assertion) |
| §9 R2 in-scope candidate | Direct input | Framing decision F (close R2 inside Phase 4 as part of the Windows integration test); R1 deferred |
| §10 Spec drafting bar — viewport / measure / offset semantics / binding direction / clip / content < viewport | Constraint | DD-005 (algorithm sub-issues map 1:1 with the spec coverage list); DD-003 (binding direction = ADR DD); DD-004 (clip as normative requirement); §4.11 chapter outline in framing decision D |
| §11 Gallery sub-screen growth — sibling vs wrap | Direct input | Framing decision E (sibling sub-screen with the canonical `ScrollView { WrapPanel { Box × 30–40 } }` composition using fixed thumbnails; Phase 7 iteration grammar later swaps fixed children for collection-driven generation as a strict superset) |
| §12 AskUserQuestion paused; fast-track removed; phase-end gates; retrospectives wording fold; item 10 vocabulary new | Process / discipline reminders | This framing follows the inline-options-in-chat pattern; the ADR / spec sync / progress doc commits land per-owner-approval per [retrospectives.md](../../../procedures/retrospectives.md); step retros from T1 onward use the item 10 disposition vocabulary; framing decision D Moment 1 / Moment 2 doc sets follow the [retrospectives.md phase-sync doc set](../../../procedures/retrospectives.md#phase-sync-moment-2-で触る-doc-セット) |
| §13 docs/notes audit triggers | Direct input | Framing decision G (per-note disposition; layout-engine partial fire; typed-value conditional fire deferred; verification-environments fired with inherited skip-guard discipline) |

### From [m3-plan.md](../../plan.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §Acceptance criteria — A5 | Constraint | Framing decision B (pre-doc-discipline check); DD-003 / DD-004 / DD-005 (load-bearing component mapping) |
| §Acceptance criteria — A11 | Constraint | Phase 4 acceptance restatement (operational obligation); framing decision D (two-moment sync) |
| §Acceptance criteria — A12 | Constraint | DD-005 spec content depth (Phase 8 promotes per-phase chapters; Phase 4's chapter held to external-reader bar) |
| §Phase breakdown — Phase 4 description ("minimal: inner unbounded measure + viewport clip + content offset binding") | Constraint | DD-003 / DD-004 / DD-005 mapping; Out-of-scope section enforces A5's M4 deferrals |
| §Phase dependencies — Phase 2 → 3 → 4 thumbnail-strip chain | Constraint | DD-005 (ScrollView consumes WrapPanel's pairing contract from Phase 3 DD-005, not re-derive); framing decision E (sub-screen growth path) |
| §Verification strategy | Menu | Framing decision C |
| §Phase-end criteria item 5 (gallery sub-screen per phase) | Hard constraint | Framing decision E (Phase 4 grows with sibling sub-screen; Rust host only) |
| §Risks — Spec-drafting drift | Mitigation | Framing decision D (Moment 1 lands design-spec draft; phase does not close with TODO spec text) |

### From [m3-gallery-wireframe.html](../../references/m3-gallery-wireframe.html)

| Element | Disposition | Consumed at |
|---|---|---|
| Overflow proof strip (vertical scroll) | Visible-proof reference | Framing decision E (Phase 4 gallery sub-screen demonstrates vertical scroll); DD-001 (vertical-only hardcode matches wireframe) |
| Numbered callout for ScrollView (if present) | Premise | DD-001 (1-child container shape) |

### From [process/milestone-3/phase-3/decisions/preamble.md](../../phase-3/decisions/preamble.md)

| DD | Disposition | Consumed at |
|---|---|---|
| DD-M3-P3-001 (WrapPanel IR + N-child) | Pattern reuse | DD-001 (ScrollView IR shape mirrors WrapPanel's per-kind-tag approach; child-count gate differs — exactly-1 vs N) |
| DD-M3-P3-002 (orientation hardcoded horizontal) | Pattern reuse | DD-001 (scroll-axis hardcoded vertical, mirrors hardcode-and-defer-additive pattern) |
| DD-M3-P3-003 (spacing attributes, constant-only) | Pattern reuse | DD-002 / DD-003 (constant-only stance for non-binding attributes; binding sub-issue folded into DD-003 rather than standalone DD) |
| DD-M3-P3-004 (item-cross-size, parent-passthrough default) | Pattern reuse | DD-002 (viewport-from-parent-passthrough default matches the "no synthesised bound out of nowhere" principle) |
| DD-M3-P3-005 (WrapPanel measure-arrange + ScrollView pairing contract) | Direct input | DD-005 (Phase 4 ScrollView passes unbounded scroll-axis to content and bounded cross-axis = viewport width to content; the pairing's content side is already settled by Phase 3 DD-005) |
| DD-M3-P3-006 (IR-loader defense-in-depth) | Pattern reuse | DD-006 (validate-vs-build_node placement; compound invariant shape distinguished from Phase 3's value-range shape per pre-doc-inputs §5) |
| Phase 3 verification closure item 4 (WrapPanel no clip surface) | Direct input | DD-004 / framing decision C (ScrollView is the symmetric inverse — clip presence is the load-bearing assertion) |

### From [process/milestone-3/phase-2/decisions/preamble.md](../../phase-2/decisions/preamble.md)

| DD | Disposition | Consumed at |
|---|---|---|
| DD-M3-P2-005 (aspect measure-arrange + LayoutError) | Pattern reuse | DD-005 (rounding contract reused; LayoutError extension pattern reused for `ScrollViewUnboundedAxis`); DD-002 (unbounded-parent → runtime error parallels Phase 2's `BoxNoExtent`) |
| Phase 2 T7 IR-load `validate()` introduction (not a DD; surfaced during impl, later abstracted as Phase 3 DD-M3-P3-006) | Pattern reuse | DD-006 (IR-loader defense-in-depth) |

### From [m2-to-m3-handover.md](../../../milestone-2/handoff.md)

| Section | Disposition | Consumed at |
|---|---|---|
| §1 `wasamo-ir` shared IR crate | Premise | DD-001 (ScrollView as new variant in existing IR; no new IR crate) |
| §2 `HandlerExpr` unified | Premise | DD-003 (binding reuses existing `HandlerExpr` shape for `i32`; no per-attribute enum extension) |
| §3 reactive drain residuals | Out of scope unless DD-003 in-out is chosen | Out-of-scope (Phase 4 read-only binding does not pressure drain residuals); ADR's M4 hand-off section notes that in-out binding may pressure drain when wired up |
| §4 `TypedValue` deferral | Discipline reminder | DD-003 (i32 + read-only preserves F5; no `TypedValue` pressure) |

---

## Next session — handoff

Once framing is owner-aligned, the next session begins ADR drafting:

1. Create `process/milestone-3/phase-4/decisions/preamble.md` (working title)
   as `Status: Proposed`, carrying the 6 DDs above with full Option
   tables, Recommendation prose, and the two-axis risk / exposure
   evaluation per DD (per
   [process/README.md §Risk evaluation](../../../README.md)).
2. Owner review pass.
3. On `Status: Accepted` flip, the upstream document edits
   enumerated under **framing decision D Moment 1** land as
   **per-review-concern commits** on the pre-doc branch (not a
   single bundle), per
   [CLAUDE.md §Commit rules](../../../CLAUDE.md#commit-rules).
4. Phase progress file
   `docs/plans/progress/m3-phase-4-progress.md` opens with
   `Status: active`; the m3-plan.md Progress row flips from
   `not started` to `in progress`.
5. Implementation phase proceeds. From T1 onward, step retros
   apply [retrospectives.md item 10](../../../procedures/retrospectives.md#step-end-固有-merge--phase-ブランチ)
   vocabulary (Phase 4 is the first phase to use it from day 1).
   At phase close, **framing decision D Moment 2** lands
   per-review-concern: `docs/dsl_spec.md` §4.11 re-sync,
   `docs/architecture.md` re-sync, progress file retired per
   the standard lifecycle, phase-end retrospective recorded per
   the *retro forward distillation* discipline.
