# T6 rendering evidence

All captures use `CopyFromScreen` over the live gallery window and a release
workspace build. The 125% sets use one throwaway
`SetProcessDpiAwarenessContext(PMv2)` call in `runtime::init()`; the call is
absent from the final source and remains T9's responsibility.

## Sets

| Set | Code | Effective DPI | Purpose |
|---|---|---:|---|
| `t6-baseline-a/b/c` | parent `7c4ddc7` plus throwaway declaration | 120 | Fresh pre-T6 scaled-path baseline |
| `t6-after-a/b/c` | T6 implementation plus throwaway declaration | 120 | Device-resolution surface, DPI, origin, brush and scale-walk result |
| `t6-100-baseline-a/b` | parent `7c4ddc7` | 96 | Fresh identity-path baseline |
| `t6-100-after-a/b` | T6 implementation | 96 | `ceil` allocation plus accepted brush mapping at identity scale |
| `t6-brush-default` | T6 implementation with the three DD-M4-P1-006 brush setters removed | 96 | Mutation control isolating the accepted mapping from the other row-7 changes |
| `t6-final` | Accepted T6 source after a forced clean release rebuild | 96 | Branch-tip restoration check after the mutation build |
| `t6-scaled-surface-identity-a/b` | T6 geometry/cache plus accepted brush mapping, with text rasterization forced to 96 DPI | 120 | Direct negative control: correct scaled geometry with a stale-resolution surface |
| `t6-r1-final-a/b` | R1-remediated source at `fad59e2`, release workspace build | 96 | Success-path regression after separating authoritative geometry scale from raster freshness |

The capture repeats are not interchangeable historical references. They are
kept together because T5 measured session drift and established that a single
frame is not a baseline.

`Effective DPI` describes the runtime coordinate/raster path, not the final
desktop-frame sampling. The `t6-100-*`, brush-mutation and `t6-final` processes
were still DPI-unaware at runtime-effective 96 DPI; on this 125% desktop DWM
then enlarged their complete frames by approximately 1.25. Their pixel counts
and same-path mutation comparisons remain valid, but any bounding-box size
measured in those PNGs is a post-DWM physical measurement, not a 96-DPI frame
dimension.

## Repeatability and comparisons

`compare-frames.ps1` results over the client interior:

- `t6-baseline-a` vs `t6-baseline-b`: all six frames byte-identical.
- `t6-baseline-b` vs `t6-baseline-c`: all six frames byte-identical.
- `t6-after-b` vs `t6-after-c`: all six frames byte-identical.
- `t6-after-a` vs `t6-after-b`: the three gallery frames differ by 32–52
  text pixels, maximum per-channel delta 8; the three label-update frames are
  identical. This is inside the previously measured small-delta class and is
  recorded as inconclusive, not passed away.
- `t6-baseline-b` vs `t6-after-b`: all six scaled frames differ materially
  (74,830–563,500 pixels, maximum per-channel delta 225–252).
- `t6-100-baseline-b` vs `t6-100-after-b`: all six identity-scale frames
  differ materially (9,360–24,868 pixels, maximum per-channel delta 220–249).
  This falsifies the old plan phrase "rendering unchanged at 100%": the D2D
  DPI and atlas-origin operations are identities there, but `ceil` allocation
  and DD-M4-P1-006 are not.
- `t6-brush-default` vs `t6-100-after-b`: all six frames differ materially
  (9,327–25,197 pixels, maximum per-channel delta 217–234). This mutation
  isolates the accepted mapping as an observable part of the 100% change.
- `t6-100-after-b` vs `t6-final`: all six client interiors are byte-identical.
  This closes the named mutation-control loop: the final release DLL is not
  the setter-removed artifact. Byte identity does not by itself prove arbitrary
  source identity; a render-neutral mutation needs a structural/source
  artifact rather than an image comparison.
- `t6-scaled-surface-identity-a` vs `-b`: all six client interiors are
  byte-identical.
- `t6-scaled-surface-identity-b` vs `t6-after-b`: all six frames differ
  materially (9,183–25,977 pixels, maximum per-channel delta 223–252).
  Unlike the parent baseline, both sides use 120-DPI node caches and therefore
  occupy the same client with the same 9×2 tile layout. The mutation changes
  only `draw_text_at_dpi`'s effective DPI; its temporary process-awareness
  declaration merely exposes the already-existing 120-DPI path.
- `t6-r1-final-a` vs `-b`: all six client interiors are byte-identical.
- `t6-final` vs `t6-r1-final-a`: all six client interiors are byte-identical.
  R1's state split therefore leaves the accepted success-path frame unchanged;
  the new mock-free geometry/raster test is the positive control that fires the
  otherwise render-neutral state distinction.

## Assistant screenshot analysis

The scaled positive control is the same text on the same 120-DPI monitor:

- [`status-before-5x.png`](./status-before-5x.png) — the parent build keeps
  identity-sized surfaces and identity node caches. The gallery occupies only
  `1 / 1.25` of the client. Its 96-DPI glyph rasterization produces grey edges,
  uneven multi-pixel stems and partly filled counters; the default centred
  `Uniform` brush also applies a small whole-surface shrink rather than a DWM
  bitmap stretch.
- [`status-after-5x.png`](./status-after-5x.png) — T6 projects the tree across
  the client and rasterizes the same run at 120 DPI. Stems and horizontal
  strokes are narrower and more consistent, counters remain open, and the
  accepted one-to-one brush removes the old slight shrink/centring.

The review-added negative control removes the geometry confound from that
pair. `t6-scaled-surface-identity-b/gallery-default.png` and
`t6-after-b/gallery-default.png` have the same window, button and tile edges
and the same 9×2 arrangement. In the mutation, labels remain rasterized at
96 DPI inside 120-DPI Visuals and appear visibly smaller and less consistent;
the accepted frame rasterizes those same labels at 120 DPI. Because the
accepted DD-M4-P1-006 brush does not stretch, this wrong implementation shows
as under-sized glyphs rather than P2's old-default-brush blur. Either image is
compatible with correct geometry; only the pair distinguishes the required
surface-resolution path.

The brush mutation is the same label at effective 96 DPI:

- [`brush-default-5x.png`](./brush-default-5x.png) — the WinRT default
  (`Uniform`, alignment `0.5`) scales and centres the integer surface.
- [`brush-accepted-5x.png`](./brush-accepted-5x.png) — the accepted `None` /
  `0.0` mapping holds unit scale and the surface origin at the Visual origin;
  the visible displacement and glyph-shape change disappear only with these
  setters present.

`text_surface_mapping_integration.rs` supplies the structural positive
control behind the last pair using the real WinRT objects: it reads the
default as `Uniform` / `0.5`, the production brush as `None` / `0.0`, confirms
the `ceil` source and exact-`f32` Visual sizes are non-proportional, and lays
out text at both an integer and a fractional device-space origin.

The R1 final capture was also inspected directly. `gallery-default.png` is a
non-blank 9 × 2 tile gallery with all tab/toolbar/tile/status text present;
`gallery-lightbox.png` shows the click-created overlay, navigation and close
labels; `labelupdate-clicked.png` shows `Counted 10 times` plus the static and
Grid-stretched reference labels. Those three states exercise initial attach,
conditional insertion and property-driven re-layout. Their exact match to
`t6-final` confirms no success-path GUI regression, while
`authoritative_geometry_scale_preserves_a_stale_raster_retry` distinguishes
the new failure-state semantics from the old shared-cache implementation.

None of these captures proves real cross-monitor delivery; T11 owns that
literal path. The scaled sets prove the T6 rendering path after an effective
scale has reached the window.
