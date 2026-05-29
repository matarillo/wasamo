### DD-M3-P4-005 — Measure-arrange algorithm (novel normative viewport / content / offset semantics)

**Status:** Accepted

**Context:** Introduces novel normative spec content into
`docs/dsl_spec.md` of a different *kind* than Phase 3 (no line-
formation algorithm) and lighter in *scope* than the upcoming
Phase 5 Grid star sizing which is the milestone's "second novel-
normative-spec phase" proper. The DD settles the content-measure
pass, viewport-vs-content size relationship, and offset clamping;
the ADR section is also the **seed** of the dsl_spec §4.11 chapter
(Moment 1 lands the spec chapter in design-spec-draft form;
Moment 2 re-syncs to implementation findings).

The algorithm is structurally simpler than Phase 3 WrapPanel's
two-stage measure-arrange — ScrollView has one child, no line
formation, no multi-pass measurement. The novel content is the
**unbounded-axis + bounded-axis asymmetric input** to the
content's measure pass, plus the **offset clamp** semantics that
have no analogue in Phase 1–3 surfaces.

**Sub-issues:**

- **Content measure pass.** Content is measured with a constraint
  of `(viewport_width, +∞)` — bounded cross axis (= viewport
  width per DD-001's vertical-only recommendation), unbounded
  scroll axis. Inverse of WrapPanel's measure input (WrapPanel:
  unbounded main + cross bound from Phase 3 DD-M3-P3-004
  `item-cross-size` / parent-cross passthrough). DD-001's vertical-only
  implies "scroll axis = vertical, unbounded direction =
  vertical, viewport-equals-cross-axis-bound = width".
- **Viewport vs content size relationship.**
  - `content_size_scroll_axis <= viewport_size_scroll_axis`:
    content fits within viewport. Offset is clamped to 0 (no
    scrolling possible).
  - `content_size_scroll_axis > viewport_size_scroll_axis`:
    content exceeds viewport. Offset is clamped to
    `[0, content_size - viewport_size]`. Visible content along
    the scroll axis is `[offset, offset + viewport_size)`.
- **Offset application.** After the content measure, the content's
  resolved rect is translated by `(0, -offset)` (in absolute
  layout-engine coordinates, before `sync_visuals()` converts to
  parent-relative). The content's outer rect within ScrollView's
  local space is then `(0, -offset, content_w, content_h)`;
  visible clipping is the rendering-side operation owned by
  DD-004's Composition clip.
- **ScrollView outer size.** Equals viewport size, regardless of
  content size. Cascading parent-bound violations are excluded —
  even if content size exceeds parent's slot, ScrollView's outer
  size stays at viewport. Phase 4 analogue of Phase 3 DD-005's
  "WrapPanel outer main-axis size does not grow to accommodate
  oversized children" rule.
- **Content-smaller-than-viewport behaviour.** Content paints at
  its measured size, anchored at the viewport's top-leading corner
  (`(0, 0)` in viewport-local coordinates). Remaining viewport
  area shows whatever visual content is behind the ScrollView
  (for example a Box fill supplied by surrounding composition);
  Phase 4 adds no ScrollView-level background attribute. Offset
  is forced to 0 by the clamp.
- **Unbounded scroll-axis parent.** Per DD-002 decision, fires
  `LayoutError::ScrollViewUnboundedAxis` at layout time. No
  degenerate Phase 4 ScrollView shape: unbounded scroll axis is
  structurally meaningless.
- **Rounding contract.** Inherits Phase 2 DD-005 / Phase 3 DD-005:
  `f32` for layout-engine internals, `i32` for attribute literals,
  promoted to `f32` at comparison. No pixel-snapping in Phase 4.
  `i32` offset is promoted to `f32` for the clamp arithmetic.
- **LayoutError surface.** New
  `LayoutError::ScrollViewUnboundedAxis` variant per DD-002.
  Internal-only; no `WASAMO_LAYOUT_ERROR_*` ABI tag is added in
  Phase 4 (no host can observe the new variant meaningfully).

**Options (overall algorithm):**

- **Option A — Asymmetric measure + clamp (as detailed above)
  (recommended).** Content measured with `(viewport_w, +∞)`;
  offset clamped to `[0, max(0, content_size - viewport_size)]`;
  outer size = viewport size; unbounded scroll-axis parent →
  `LayoutError::ScrollViewUnboundedAxis`.
  - What you gain: matches A5 verbatim ("inner unbounded measure
    + viewport clip + content offset binding"); composes cleanly
    with Phase 3 WrapPanel's pairing contract (per Phase 3 ADR
    DD-005 ScrollView pairing); narrowest spec surface.
  - What you give up: nothing relative to A5.
- Option B — Symmetric measure (content also measured with
  bounded scroll-axis = viewport size). Content cannot exceed
  viewport; scroll is impossible.
  - What you gain: simplest algorithm.
  - What you give up: contradicts A5's "inner unbounded
    measure"; ScrollView becomes a clipped Box, not a scrollable
    viewport.
- Option C — Lazy measure (only measure visible content based
  on current offset). Content extent unknown until scrolled to.
  - What you gain: theoretical performance gain for very large
    content.
  - What you give up: pre-1.0 over-engineering; Phase 4 sub-
    screen (30–40 thumbnails) does not pressure performance;
    introduces stateful measure cache; the offset clamp upper
    bound (`content_size - viewport_size`) becomes ill-defined
    if content_size is itself lazy.

**Decision:** Option A. The arrange pass re-applies the clamp on
every layout pass (window resize, content size change,
programmatic state mutation via the DD-003 binding). The
unbounded-scroll-axis check fires *before* the content measure;
the content measure pass is what runs in the recommended Option A
happy path.

**Spec content seed (Moment 1 §4.11 draft).** The DD-005
sub-issues map 1:1 to the §4.11 chapter outline:

1. ScrollView is a 1-child container with vertical-only scroll
   axis (anchors to DD-001).
2. Viewport size source = parent constraint passthrough; explicit
   `viewport-*` attribute deferred (anchors to DD-002).
3. `offset-y` attribute: `i32` pixels, bindable read-only, default
   0 at widget-catalog constructor (anchors to DD-003).
4. Content measure pass: bounded cross axis (= viewport width) +
   unbounded scroll axis (= vertical). Inverse of WrapPanel's
   measure input.
5. Offset clamp: `[0, max(0, content_size - viewport_size)]`.
   Silent clamp; over/under-scroll not admitted in Phase 4.
6. ScrollView outer size = viewport size; does not grow to
   accommodate content overflow.
7. Visible clip: ScrollView Visual installs `Visual.Clip =
   InsetClip{0,0,0,0}` (anchors to DD-004).
8. ScrollView-owned intermediate content Visual carries the
   offset: `Visual.Offset = (0, -offset_y, 0)`.
9. Unbounded scroll-axis parent fires
   `LayoutError::ScrollViewUnboundedAxis` (anchors to DD-002 /
   DD-005).
10. ScrollView mental model subsection (5 facts) — see Phase 4
    pre-doc framing decision I.

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
with a *bounded* scroll-axis constraint (Option B above)
contradicts A5's "inner unbounded measure" load-bearing
phrasing.
