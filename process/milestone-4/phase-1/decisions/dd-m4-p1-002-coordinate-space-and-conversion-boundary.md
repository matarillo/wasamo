# DD-M4-P1-002 — Coordinate-space definition and the conversion boundary

**Status:** Accepted
**Phase:** M4-Phase 1
**AC:** AC7, second requirement ("render crisply on high-DPI displays
without DWM bitmap scaling"). Also the phase's central contract:
[framing](../requirements/framing.md) §Phase 1 の受入れ基準 records that
the coordinate-space definition is consumed by five later phases.

## Context

This is the decision
[layout-engine.md §3.1](../../../../docs/notes/layout-engine.md) has been
waiting for since M1:

> 物理ピクセルをエンジンが意識すべきか。DirectWrite のヒンティング精度に
> 影響する。M1 では論理ピクセルのみで動作。Grid / ScrollView 導入時に
> 再考が必要。

The reconsider trigger (Grid / ScrollView) fired inside M3 and went
unprocessed. What
[constraints §2](../requirements/constraints.md) draws out of the note is
the part that is easy to miss: **the question's real subject is
DirectWrite hinting precision, not coordinate arithmetic.** Text is
rasterized in
[`text.rs`](../../../../wasamo-runtime/src/text.rs) onto a
`CompositionDrawingSurface` and applied to a Visual as a surface brush.
Multiplying coordinates by a scale factor does not add a single pixel to
that surface. An implementation that gets every coordinate right and
leaves the surface alone produces **exactly the blur it set out to
remove**, passes every integration test, and looks perfect on a 100%
monitor. That is framing risk R2, and it is why this DD is organised
around the surface rather than around the arithmetic.

### The material facts (verified against the workspace at drafting time)

- `CompositionGraphicsDevice::CreateDrawingSurface` takes a size the
  `windows` crate names `sizepixels`; the sibling
  `CreateDrawingSurface2` takes a `SizeInt32` of the same meaning. The
  surface is a pixel buffer, and Wasamo currently allocates it at the
  *logical* text size.
- `ICompositionDrawingSurfaceInterop::BeginDraw` hands back an
  `ID2D1DeviceContext` at its default **96 DPI**, and an atlas offset in
  **pixels**. `draw_text` uses that offset directly as the drawing
  origin, which is correct only while the context is at 96 DPI.
- `TextRenderer::measure` returns DirectWrite metrics computed at 96 DPI
  — i.e. **already in DIP**. Layout consumes them as lengths. This one
  fact is what makes "layout stays in DIP" nearly free
  ([constraints §4](../requirements/constraints.md)).
- The Composition visual tree over a DPI-aware HWND is in the client
  area's **physical pixels**. Composition applies no DPI scaling of its
  own.
- `visual_rect` reads `Visual.Offset` / `Visual.Size` back off the live
  Visual, and hit-testing compares those against raw `lparam` pointer
  coordinates. Both are physical today.
- The window root gets `SetRelativeSizeAdjustment(1, 1)`, tying its size
  to the client area.
- Two Visual writes happen at **widget construction time**, outside any
  layout pass: the Button label's `SetOffset(PAD_H, PAD_V)` and
  `SetSize(lw, lh)`. Construction happens in the IR loader, before the
  widget tree is attached to a window — so **no scale factor exists at
  that moment**.

## Decision dependency summary

Consumes [DD-M4-P1-001](./dd-m4-p1-001-dpi-awareness-declaration.md)
(the OS now reports real per-monitor DPI). Provides the space definition
that [DD-M4-P1-003](./dd-m4-p1-003-dpi-change-propagation.md) propagates
and that
[DD-M4-P1-004](./dd-m4-p1-004-unit-contract-and-spec-wording.md) writes
into the specs. Close artifact: the call-site audit table (§The
conversion sites).

## Sub-issues

- **Which space layout works in**, and where the conversion happens.
- **How the text rasterization surface reaches device resolution** —
  the phase's hard part.
- **When surfaces learn their scale**, given that construction precedes
  attachment.
- **Which space hit-testing runs in.**
- **The carrier of the arithmetic** — a type, or bare `f32`.
- **The rounding contract.**

## Which space layout works in

### Options

- **C1 — Layout in physical pixels.** The scale is applied on the way
  *in*: authored `.ui` dimensions, spacings, and track sizes are
  multiplied at load or at layout entry, and the engine computes in
  device pixels throughout.
  - What you gain: Visual writes need no conversion at all — layout's
    output is already the visual tree's space. Sub-pixel positioning
    decisions could be made with full knowledge of the device grid,
    which is the precondition for any future pixel snapping.
  - What you give up: `TextRenderer::measure` returns DIP, so it would
    need converting *into* the engine — the one place today that needs
    no conversion becomes one that does. Every authored constant
    becomes resolution-dependent inside the engine, so the engine's
    pure-logic tests (which encode expected extents as literals) would
    each need a scale parameter or a scale-1 assumption that stops
    being representative. And the invariant this phase most needs —
    logical layout is identical at every scale — becomes something the
    engine merely *tends to* produce rather than something it cannot
    violate: `f32` results at 1.25× do not round-trip exactly, so
    WrapPanel line breaks near a boundary can differ between scales.
    That is positive control B failing by construction. **Rejected on
    merit.**

- **C2 — Layout in DIP, with a scale transform on the root Visual.**
  The root `ContainerVisual` gets `Scale = (s, s)`; every
  `SetOffset` / `SetSize` keeps writing DIP; window client size and
  pointer coordinates are divided by `s` on the way in.
  - What you gain: the smallest coordinate diff of any option — the
    entire `sync_visuals` traversal and both Button-label writes are
    untouched. One property set once per window.
  - What you give up: **it does not buy crispness, and it does not even
    reduce the work needed to buy it.** A DIP-sized text surface under
    a scale transform is a stretched bitmap — the exact DWM failure,
    relocated. The fix is to allocate the surface at device resolution
    and set the D2D context's DPI, which is *the whole of §The
    rasterization surface below* — identical under C2 and C3. So C2
    saves only the coordinate multiplications, and pays for that saving
    twice: the surface then has more texels than its visual has DIP
    units, so whether the result is crisp depends on Composition
    sampling the surface under the **effective** world transform in one
    pass rather than materialising an intermediate at the visual's
    local size. That behaviour is not contractual, is not observable
    from the API, and — worst of the three — fails *silently and only
    at scale ≠ 1*.
  - The second cost is structural: the visual tree's coordinate space
    would be DIP while the pointer stream, the client rectangle,
    `ClientToScreen`, and clip rectangles remain physical. Every future
    consumer that reaches into the visual tree for a screen-space
    answer — the IME candidate-window rectangle (M4-Phase 5 / 6), the
    top layer's placement (M4-Phase 9) — has to remember to multiply.
    C2 does not remove the multiplication; it distributes it to every
    future call site instead of confining it to one seam. It also
    leaves the root Visual in a mixed state: its own `Size` comes from
    `SetRelativeSizeAdjustment(1, 1)` and is therefore physical (a
    visual's size is expressed in its *parent's* space), while its
    children are placed in a DIP local space. That invariant is
    correct but is the kind of thing a reader gets wrong once and
    debugs for a day.

- **C3 — Layout in DIP, conversion at the seams.** Layout, `.ui`
  values, `measure` results, and the typography ramp are DIP. The
  window's client extent and the pointer coordinates are converted to
  DIP at the message-loop entry. The Visual writes multiply by `s`. The
  text surface is allocated at device resolution with the D2D context's
  DPI set accordingly.
  - What you gain: one space per concern, each stated. The visual tree
    is physical — the same space as the client rectangle, the pointer
    stream, `ClientToScreen`, and clips — so a later phase asking "where
    is this widget on screen" gets its answer without a hidden
    multiplication. Crispness is bought explicitly rather than inferred
    from compositor behaviour. The layout engine is untouched, so
    framing agreement ① is honoured without argument, and layout
    results are scale-invariant *because the engine never sees a scale*
    — which is what makes positive control B an assertion rather than a
    hope, and what makes DD-003's change handling as small as it is.
  - What you give up: the conversion is spread across a handful of write
    sites rather than concentrated in one property, so the call-site
    audit is load-bearing — a missed site is wrong only at scale ≠ 1.
    That is the risk the implementation gate is armed for.

### Comparison

C1 loses on the invariant it would put at risk (logical layout identical
across scales) and on contaminating the one place — DirectWrite
measurement — that is already in the right unit.

C2 versus C3 is the comparison framing named as "the substance of the
decision," and it resolves cleanly once the surface work is separated
out: **the hard part of AC7 is identical in both.** C2's advantage
shrinks to "fewer multiplications in `sync_visuals`," and against that
it stakes crispness on unspecified compositor resampling and hands
every future screen-coordinate consumer a conversion to remember. Under
the product-merit prior, an explicit contract beats a smaller diff.

C3's cost — an auditable set of write sites — is a tie-breaker
consideration, and it is bounded and enumerable (below).

### Recommendation

**C3 — layout in DIP, conversion at the seams.** Stated normatively:

> **Layout coordinates are device-independent pixels (DIP), where 1 DIP
> is 1/96 inch. The Composition visual tree and the Win32 pointer
> message stream are in the physical pixels of the window's client area.
> Conversion between them occurs only at the sites enumerated below, and
> the layout engine never receives a scale factor.**

This answers
[layout-engine.md §3.1](../../../../docs/notes/layout-engine.md): **no,
the engine should not be aware of physical pixels** — and the hinting
precision the note was really asking about is bought at the
rasterization surface, not in the engine.

## The rasterization surface (the hard part)

Coordinates being right does not make text crisp. What does:

1. **Allocate the surface at device resolution.**
   `CreateDrawingSurface(Size { w: dip_w × s, h: dip_h × s })` — the
   parameter is pixels. `CreateDrawingSurface2`'s `SizeInt32` is
   available if an explicit integer allocation reads better; the
   contract below is the same either way.
2. **Tell D2D what resolution it is drawing at.** After `BeginDraw`,
   `dc.SetDpi(96 × s, 96 × s)`. The D2D coordinate space then remains
   DIP — so `create_text_layout`'s `max_w` / `max_h` stay the DIP
   values, and `TypographyStyle::size_sp` stays a DIP font size — while
   glyph rasterization and DirectWrite hinting happen at the device
   resolution. This is what
   [layout-engine.md §3.1](../../../../docs/notes/layout-engine.md)'s
   "ヒンティング精度" refers to.
3. **Convert the atlas offset.** `BeginDraw` returns the offset of the
   allocated region within the backing atlas, **in pixels**. `draw_text`
   currently uses it directly as a D2D drawing origin, which is only
   correct while the context is at 96 DPI. Once step 2 changes the DPI,
   the origin must become `(offset.x / s, offset.y / s)`. **This is a
   trap, not a detail:** the atlas offset is often `(0, 0)`, so an
   implementation that forgets the conversion works most of the time
   and shifts text inside its own surface intermittently. It is called
   out here so the implementation writes it deliberately.
4. **Keep the brush mapping one-to-one.** The visual's `Size` is the
   *physical* size (`dip × s`, from §The conversion sites), and the
   surface is `dip × s` pixels, so the default surface-brush stretch
   maps one texel to one device pixel. Crispness follows from the two
   numbers agreeing, not from a filtering mode.

**Alternative considered:** leave the D2D context at 96 DPI and apply a
scale transform to it instead (`SetTransform`). D2D composes the world
transform into glyph rasterization, so this is also correct, and it
avoids the step-3 offset conversion. It is rejected as the primary
because it consumes the context's transform — a resource later text work
(rotation, mirroring) may want — and because expressing "this surface is
at 150% resolution" as a DPI is the more legible statement of intent. If
implementation finds the offset conversion fragile in practice, swapping
to the transform form is a permitted implementation choice under the
same contract; the contract is "the surface has `ceil(dip × s)` pixels
and glyphs are rasterized at device resolution," not the specific API
pair.

### The rounding contract for surfaces

Surface allocation is the **one place** in this phase where a real
number must become an integer count. The contract:

- **Round up** (`ceil`) the pixel dimensions. Truncation clips the final
  column or row of glyph coverage — a defect that appears only at
  non-integer scales and reads as "the last letter is cut off."
- The **visual's** size stays the exact `f32` physical value. The
  at-most-one-pixel excess in the surface is transparent padding.
- No integer snapping is introduced anywhere else. The "no pixel
  snapping" policy already stated in
  [docs/dsl_spec.md](../../../../docs/dsl_spec.md) and
  [docs/architecture.md](../../../../docs/architecture.md) stands
  unchanged; introducing it is deferred with a recorded trigger
  ([framing](../requirements/framing.md) §含まないもの).

## When surfaces learn their scale

Widget construction happens in the IR loader, **before** the tree is
attached to a window — so at construction there is no window and no
scale. Both `WidgetNode::text` and the Button label path create their
surface at construction time.

### Options

- **R-a — Construct at scale 1; a re-rasterization walk brings the tree
  to the window's scale.** `set_root` runs the walk after attaching;
  `WM_DPICHANGED` runs the same walk.
  - What you gain: the widget tree stays **window-independent at
    construction**, which is the property M4-Phase 8 needs (a tree built
    before its window is chosen, or moved between windows at different
    scales). One mechanism with two callers, so the change path is
    exercised on every startup rather than only when someone drags a
    window between monitors.
  - What you give up: every attach re-rasterizes text once, even at
    100% where the result is identical. Bounded, one-time, at gallery N.
- **R-b — Thread the scale into every constructor.** Requires the window
  to exist and be known before the tree is built, which inverts the
  current load order and would have to be inverted again for
  M4-Phase 8. Rejected on merit: it makes the widget tree
  window-bound for no gain.

### Recommendation

**R-a.** The walk is specified once here and reused by DD-003. It
re-creates each text-bearing node's surface and brush from state the
node already holds — `WidgetData::Text { content, style }` and
`ButtonData { label_text, label_style }` — so no new retained state is
required.

Note the consequence that makes DD-003 simple: because `measure` is DIP
and unaffected by scale, re-rasterization **does not change any node's
`SizeConstraint::Fixed(w, h)`**, and therefore does not invalidate
layout.

## Which space hit-testing runs in

Today `hit_test_click_inner` / `update_hover_inner` compare raw pointer
`lparam` values against `visual_rect`'s readback. Under C3 both sides
are physical, so **the existing arithmetic stays correct with no change
at all.** That makes this a genuine choice rather than a forced one.

### Options

- **H1 — Leave it physical.** Zero change; internally consistent
  ("hit-testing compares device pointer coordinates to device
  rectangles"). What you give up: the contract handed to M4-Phase 2 is
  then a *physical* pointer, while everything M4-Phase 2 will want to
  express — a minimum touch-target size, hit-area padding — is
  authored in DIP. The phase would be handing its most important
  downstream consumer the wrong unit and calling it done.
- **H2 — Convert the pointer to DIP at the window entry, and convert the
  `visual_rect` readback alongside it.** Hit-testing arithmetic is then
  DIP, like the rest of the runtime.
  - Honest accounting: **the two conversions cancel today**, because
    hit-testing sources its geometry from the visual tree. They stop
    cancelling the moment M4-Phase 2 sources geometry from layout or
    introduces a DIP-denominated hit-area rule — which is the phase
    immediately after this one.
  - What you gain: the delivered contract is "the pointer arrives in
    DIP," which is what the coordinate-space definition should say, and
    the pointer-conversion seam exists in one place for touch
    (M4-Phase 2) to reuse.
- **H3 — Stop reading geometry back off the Visual; cache each node's
  arranged DIP rectangle during layout and hit-test against that.**
  The right long-term shape, and arguably the honest fix for the
  `visual_rect` readback (whose own comment admits it exists "to avoid
  tracking a separate state"). Deferred, not rejected: it adds retained
  per-node state whose lifetime and invalidation rules belong with the
  event-routing model, which is M4-Phase 2's decision, and taking it
  here would put this phase into the hit-testing design it was scoped
  out of ([framing](../requirements/framing.md): "新しい当たり判定の面は
  作らない").

### Recommendation

**H2**, with **H3 recorded as M4-Phase 2's natural next step**
(trigger: the event-routing model needing layout-derived hit rectangles
or a DIP-denominated minimum hit target — both expected in that phase).

## The conversion sites (call-site audit table)

This table **is** the implementation gate's audit artifact. It is
derived from [constraints §4](../requirements/constraints.md)'s seven
paths, expanded with the two construction-time writes found while
drafting. The implementation audits against this table, not from
memory; "no coordinate enters or leaves outside these rows" is the
claim being checked.

| # | Site | Direction | Today | After |
|---|---|---|---|---|
| 1 | `wnd_proc` `WM_SIZE` (`lparam` client extent) → `run_layout_as_window_root` | in | physical used as DIP | ÷ s at the seam |
| 2 | `set_root` `GetClientRect` → first layout | in | physical used as DIP | ÷ s at the seam |
| 3 | `wnd_proc` `WM_MOUSEMOVE` / `WM_LBUTTONDOWN` / `WM_LBUTTONUP` (`lparam` x, y) | in | physical | ÷ s at the seam (H2) |
| 4 | `widget.rs` `sync_visuals` node `SetOffset` / `SetSize` | out | DIP written raw | × s |
| 5 | `widget.rs` `sync_visuals` ScrollView intermediate `SetOffset` / `SetSize` | out | DIP written raw | × s |
| 6 | `widget.rs` Button label `SetOffset` / `SetSize` — **at construction** and in the label-update path | out | DIP written raw, no scale available | **moves into the sync pass** (below) |
| 7 | `text.rs` `draw_text` surface allocation + D2D context DPI + atlas origin | out | 96 DPI, DIP-sized surface | `ceil(dip × s)` pixels, DPI `96 × s`, origin ÷ s |
| 8 | `window.rs` root `SetRelativeSizeAdjustment(1, 1)` | out | physical ↔ physical | **unchanged** — a relative relation between two physical quantities; asserted, not modified |
| 9 | `widget.rs` `visual_rect` readback → hit-test / hover | in | physical compared to physical | ÷ s (H2) |
| 10 | `text.rs` `TextRenderer::measure` → layout | — | DIP | **unchanged** — the fact that carries "layout stays DIP" |
| 11 | `text.rs` `TypographyStyle::size_sp` (12 / 14 / 20 / 28) | — | logical | **unchanged**, defined as DIP by DD-004 |
| 12 | `widget.rs` ScrollView / Grid / Box `InsetClip` insets | out | all zero | **unchanged** — zero is scale-invariant; asserted, and re-checked if a non-zero inset is ever introduced |
| 13 | `window.rs` `create_hwnd` `CreateWindowExW` width / height | in | host value used as physical | DIP → physical, per [DD-M4-P1-003](./dd-m4-p1-003-dpi-change-propagation.md) |

**Rows 4 / 5 / 6 detail — convert once, on the difference.**
`sync_visuals` computes `computed.offset − parent_abs_offset` in DIP and
multiplies the *result*, rather than multiplying both operands and
subtracting. The two are equal in exact arithmetic and differ in `f32`;
the single multiplication has one rounding instead of two. The
ScrollView recursion likewise stays entirely in DIP — `child_parent_abs`
is `(offset.0, offset.1 − applied_y)` in DIP, and only the two
Composition writes multiply.

**Row 6 detail — the construction-time writes move.** The Button label's
offset and size are written at construction, where no scale exists.
Rather than threading a scale into construction (rejected as R-b), the
label's placement moves into the scale-aware sync pass, alongside every
other Visual write. Two consequences worth stating: the label follows a
DPI change for free, and every Composition geometry write in the runtime
then happens in exactly one pass, which is what makes the audit above
complete rather than approximately complete.

## The carrier of the arithmetic

### Options

- **U1 — bare `f32`, with a `scale` threaded through.** Nothing to
  learn; nothing to test either. Every site re-derives the rounding
  behaviour, and the framing's reserved unit-test target ("the
  operations of the type, if a type is introduced") has nothing to
  point at.
- **U2 — a `DipScale` value type** owning `to_physical(dip) -> f32`,
  `to_dip(px) -> f32`, the rectangle form, and the `ceil` surface
  contract. Pure logic, no Win32 or WinRT dependency, so it is unit-
  testable under
  [AGENTS.md §Testing rules](../../../../AGENTS.md) without any mocking
  question. One place for the rounding contract to live and be checked.
- **U3 — phantom-typed length newtypes (`Dip<T>` / `Px<T>`) across the
  runtime.** The strongest guarantee: a mixed-unit expression stops
  compiling. Rejected on proportionality — it would touch the layout
  engine's signatures, which framing agreement ① keeps out of scope,
  and it would convert a bounded 13-row audit into a codebase-wide
  refactor. Recorded as available if a unit-mixing defect actually
  recurs; not adopted on a prediction that it might.

### Recommendation

**U2.** It gives the phase's pure-logic tests a real target — conversion
at 125% / 150% / 200%, position-and-extent consistency, round-trip
error, rounding direction, the `ceil` allocation rule — and keeps the
rounding contract in one auditable place.

## Spec content seed

`docs/architecture.md`: a normative coordinate-space section stating the
two spaces, the conversion seams as a class (not as a source-file list),
the text-surface resolution contract, and the invariant that layout
results do not depend on the scale factor; §12's DPI open-questions row
resolved. `docs/dsl_spec.md`: dimension values defined as DIP.
`docs/abi_spec.md`: handled by DD-004. Provenance in all three is a
hyperlink to this ADR — no DD labels or option vocabulary in spec prose.

## Forward-compat exposure

All additive; none requires reshaping what this DD ships.

1. **Layout-derived hit rectangles (H3)** — M4-Phase 2, trigger
   recorded above.
2. **Screen-coordinate mapping** (IME caret and composition rectangles,
   M4-Phase 5 / 6; top-layer placement, M4-Phase 9) — lands as
   `visual absolute physical → ClientToScreen`, with no scale
   multiplication, because C3 keeps the visual tree in the same space
   the Win32 call expects. This is the concrete downstream payoff of
   choosing C3 over C2.
3. **Per-window scale** — M4-Phase 8; the `DipScale` value is per window
   from the start (DD-003), so no structural change is needed.
4. **Resolution-dependent image assets** — M4-Phase 4; the second
   rasterized asset kind arrives on the same surface-resolution
   contract stated here.
5. **Integer pixel snapping** — deferred; would extend the rounding
   contract inside `DipScale` rather than change the space definition.
6. **Text rendering-quality tuning** (rendering mode, gamma, explicit
   hinting) — M5 theming wave; this phase's obligation ends at "drawn at
   the correct resolution."
7. **Phantom-typed lengths (U3)** — available if unit mixing recurs.

## Technical risk re-evaluation

- **The R2 failure mode — coordinates right, crispness not bought.** The
  most likely way this phase ships broken, and the reason the surface
  work is specified before the arithmetic. Positive control A (the
  before/after magnified comparison) is the only evidence that
  discharges it; integration tests cannot, because a stretched bitmap
  reports the same numbers.
- **A missed conversion site** is wrong only at scale ≠ 1, so the
  development machine at 125% is the right place to catch it and CI at
  100% is not. Mitigated by the audit table above plus positive control
  B, which fails visibly if a size path is missed and a wrap position
  moves.
- **The atlas-offset trap** (§The rasterization surface step 3) is
  intermittent by nature. The mitigation is that it is named here, so
  the implementation writes the conversion deliberately rather than
  discovering it.
- **Non-integer scales accumulate `f32` error.** Bounded by construction
  under C3: layout never sees the scale, so error cannot compound
  *through* layout — it enters only at the final write. The
  convert-once-on-the-difference rule keeps that to one rounding per
  write. `DipScale`'s round-trip test pins the magnitude.
- **Moving the Button label write into the sync pass touches shipped
  rendering code.** The existing Button integration and gallery
  fixtures are the regression gate; the move is its own commit ahead of
  the scale work, so a regression there is bisectable independently of
  the DPI change.
- **Performance.** Re-rasterizing every text surface on attach and on
  scale change is O(text nodes) at gallery N, on an event that is either
  once-per-window or human-initiated. Not an axis this phase optimises.

## Revision history

- 2026-07-28: Initial draft (Status: Proposed).
- 2026-07-28: Accepted flip following owner approval of the phase slate; no
  change requested to the recommendations or their comparisons.
