# M3-Phase 7b T5 — assistant GUI evidence

Assistant-automated GUI evidence for the parent-interpreted placement
(`slot.*`) surface (ADR evidence item (4), assistant portion). Capture is
**launch + per-monitor-DPI-aware `CopyFromScreen` over `GetWindowRect` +
assistant pixel analysis** (`PrintWindow` reads back blank under
DirectComposition). The owner-visible smoke is T6's; this baseline does
not replace it.

## Capture driver

[`capture-placement-demo.ps1`](./capture-placement-demo.ps1) — re-tuned
for the **current** gallery layout. The proven Phase 6/7
[`capture-lightbox.ps1`](../../phase-6/implementation/evidence/capture-lightbox.ps1)
click coordinates are stale, and synthetic input (`SetCursorPos` +
`mouse_event`, and posted `WM_LBUTTON*`) does **not** drive the wasamo
Composition app's button handler in a non-interactive agent session — so
the placement-demo surface is authored **default-open**
(`state is_placement_demo_open: bool = true` in `gallery.ui`) and the
assistant captures it directly without a click. The owner verifies the
toggle / Close button interactively in T6.

```
pwsh -File capture-placement-demo.ps1 -CaptureHomeOnly -Width 820 -Height 720 -OutputPrefix t5-demo
```

## Frames

### `t5-placement-demo.png` — the positive controls (canonical frame)

Two on-screen positive controls, both read off the migrated `slot.*` /
`SlotData` model:

- **ZStack `slot.h-align`** (top panel): three identical boxes differing
  **only** in `slot.h-align` land at three distinct horizontal positions —
  `slot.h-align: start` (blue) at the **left**, omitted (gray) at the
  **center** (per-container default), `slot.h-align: end` (orange) at the
  **right**. A wrong implementation that ignored the alignment keyword
  could not produce the keyword-driven left/center/right spread; the
  omitted box defaulting to center is the default contrast.
- **Grid placement** (lower panel): `r0c0 stretch` (blue) at the
  top-left cell **stretch-fills** its cell (default alignment); `r0c2
  centered` (pink) sits in the **column-2** cell and, with
  `h-align: center v-align: center`, is a small box **centered** in its
  cell (alignment contrast against r0c0's stretch-fill); `r1 span 3
  columns` (green) spans the full three-column width on **row 1**
  (row/column/span all reflected). Distinct cell positions + the
  stretch-vs-centered contrast are the positive control.

### `t5-gallery-home-no-demo.png` — environment-capability + motivation record

A capture of the gallery home **without** the demo (the original
`is_placement_demo_open = false` state). It confirms (a) the environment
renders non-blank GUI (assistant capture is dischargeable here), and (b)
the shipped gallery's incidental layout does **not** surface the placement
positive controls — only `slot.h-align: stretch` ZStack children, and the
main-screen Grid does not appear at this size — which is why T5 adds the
deliberate placement-demo surface.

## Same-position (re-render) baseline

T4 removed the old ZStack bare syntax, so the "same positions as the old
surface" half cannot be regenerated on this branch. It is read against the
Phase 6/7 lightbox evidence
([../../phase-6/implementation/evidence/](../../phase-6/implementation/evidence/)):
the `slot.*` migration is a pure re-expression of placement, so the
lightbox scrim/photo land where they did pre-migration. The assistant
provides the **contrast** half (this frame); the same-position half is
owner-confirmed in T6 (the lightbox needs an interactive click the
assistant session cannot deliver).

## Phase-8 removal

The placement-demo surface (`state is_placement_demo_open`, the "Open
placement demo" button, and the `if is_placement_demo_open { … }` overlay
in `examples/gallery/gallery.ui`) is throwaway Phase-7b verification
scaffolding, recorded for removal at the M3 Phase 8 close alongside the
other per-phase verification surfaces. See the T5 carry-forward in
[../log.md](../log.md).
