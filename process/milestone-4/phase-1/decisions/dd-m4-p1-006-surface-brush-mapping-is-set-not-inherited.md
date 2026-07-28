# DD-M4-P1-006 — The surface brush's texel mapping would be set, not inherited

**Status:** Proposed
**Phase:** M4-Phase 1
**Proposes to supersede:** [DD-M4-P1-002](./dd-m4-p1-002-coordinate-space-and-conversion-boundary.md)
§The rasterization surface step 4's mechanism clause **and** §The
rounding contract for surfaces' "transparent padding" mechanism
sentence, on acceptance of this record. The one-to-one scale requirement,
the `ceil` allocation rule, the exact-`f32` Visual extent, and every other
part of DD-M4-P1-002 would stand.

## Context

Step 4 of DD-M4-P1-002 §The rasterization surface reads:

> **Keep the brush mapping one-to-one.** The visual's `Size` is the
> *physical* size (`dip × s`, from §The conversion sites), and the surface
> is `dip × s` pixels, so **the default surface-brush stretch maps one
> texel to one device pixel**. Crispness follows from the two numbers
> agreeing, not from a filtering mode.

The requirement is right. The mechanism contradicts the same section:
§The rounding contract for surfaces allocates **`ceil(dip × s)`** pixels
and calls the at-most-one-pixel excess transparent padding, while the
Visual keeps the exact `f32` extent. `ceil` is what stops the two numbers
agreeing, and it does so for every surface the runtime allocates that is
not already an integer number of device pixels wide — measured on the
gallery, every text surface (`15.81`, `46.57`, `72.03` … DIP).

`CompositionSurfaceBrush` defaults to `CompositionStretch::Uniform` with
horizontal and vertical alignment ratios of `0.5`, so a surface larger
than its Visual is **scaled down and centred**, not padded. Implementing
step 4 as written therefore loses the property the step is named for.

## Proposed decision

**The runtime would set the surface brush's stretch and alignment
explicitly:
`CompositionStretch::None`, with `HorizontalAlignmentRatio` and
`VerticalAlignmentRatio` both `0.0`.**

- `None` performs no scaling, so the surface keeps unit scale relative to
  the Visual. Where the surface is larger than the Visual — which `ceil`
  makes it whenever the physical extent is not an integer — `None`
  **clips at the Visual's bounds**; it does not pad.
- Alignment `0.0` puts the surface's origin at the Visual's origin, so
  the sub-pixel band that gets clipped is the one on the **right and
  bottom**. The default `0.5` would instead centre the surface and
  displace every glyph by up to half a pixel in each axis.

This is a claim about **scale and brush-relative alignment**, not about
absolute screen-pixel phase. A Visual may still start at a fractional
device coordinate under the phase's no-pixel-snapping contract. This
record neither introduces `SnapToPixels` nor claims that `None` alone
eliminates interpolation caused by any other transform.

**What T6 would confirm, if this record is accepted, is clipping
behaviour rather than padding.** The expected mapping is: no stretch,
no centring displacement, and the surface storage beyond the exact
Visual extent clipped on the right and bottom. The control must exercise
a non-proportional source/destination pair; centred `Uniform` can have a
zero alignment displacement when their aspect ratios happen to agree.

### Separate residual: visible glyph overhang

`ceil` converts a fractional physical extent into enough whole texels to
store that extent. It does **not** define or reserve space for visible
glyph overhang outside the `DWRITE_TEXT_METRICS` extent used by
`TextRenderer::measure`; DirectWrite reports such overhang separately.
The current exact-size Visual contract therefore clips painting outside
that measured extent regardless of whether the backing surface happens
to contain some of it.

T6 may measure current gallery strings for a visible regression, but
that observation cannot settle the general overhang policy. If clipping
is observed, T6 records and escalates it rather than silently growing the
Visual: admitting overhang changes DD-M4-P1-002 §The rounding contract
for surfaces and needs a separate accepted revision.

Under this proposal, nothing else would change: allocate at
`ceil(dip × s)` pixels, set the D2D context DPI to the window's DPI,
divide the atlas origin, and keep the Visual at the exact `f32` physical
extent.

### Options rejected

- **Rely on the default** — what step 4's mechanism clause states.
  Rejected: the default is `Uniform`, not one-to-one, once `ceil` makes
  the two sizes differ.
- **Allocate the surface at the exact `f32` size**, so the two numbers do
  agree. Rejected: it re-opens a decision this record does not touch.
  `ceil` exists because a truncated surface clips the final column of
  the requested physical extent whenever `dip × s` is non-integer; that
  can happen at scale 1 because measured DIP extents are fractional.
- **Set the stretch and leave alignment at `0.5`.** Rejected: it
  satisfies the texel mapping and still displaces every glyph by up to
  half a pixel — the same failure as an unconverted atlas origin, reached
  through a different door.
- **Grow the Visual to the `ceil`-sized surface.** Rejected for this
  proposal: it would show all surface storage, but changes the accepted
  exact-`f32` projection and the phase's no-pixel-snapping contract. A
  measured overhang defect is the trigger to reconsider that contract,
  not authority to change it inside T6.

## Verification

The candidate values are read from Microsoft's `CompositionStretch`,
`CompositionSurfaceBrush.HorizontalAlignmentRatio` and
`VerticalAlignmentRatio` documentation. If this record is accepted, T6
would confirm them by measurement — the first task at which the
difference is observable. The positive control compares the candidate
against the default on a non-proportional size pair, and observes both an
integer and a fractional device-space Visual origin so unit scale is not
over-read as screen-pixel alignment. The phase's standing rule is that a
mechanism written into a record is measured rather than cited.

## Consequences

- If accepted before **T6**, T6 would set both properties at every
  `CreateSurfaceBrushWithSurface` site and measure the result. If revised
  or still Proposed, T6 must implement no unaccepted substitute. The
  runtime calls `SetStretch` nowhere today, so the candidate would be an
  addition rather than a change.
- **[architecture.md §12.4](../../../../docs/architecture.md#coordinate-spaces)**
  no longer claims that the default brush provides one-to-one mapping;
  while this record is Proposed it identifies `None` / `0.0` as the
  candidate rather than as an accepted contract.
- No DD besides the two named DD-M4-P1-002 mechanism sentences and no
  shipped code would be affected. The runtime has no `ceil`-sized
  surface yet; T6 is the task planned to introduce the first.

## Revision history

- 2026-07-29: Initial draft (Status: Proposed). Evidence:
  [implementation/log.md](../implementation/log.md) §T5.
- 2026-07-29: Revised while still Proposed after review: made acceptance
  conditional throughout, expanded the proposed supersede to the
  "transparent padding" sentence, separated integer allocation from
  glyph overhang, and bounded the unit-scale claim away from absolute
  screen-pixel alignment.
