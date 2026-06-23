# M3-Phase 7b T5 — assistant GUI evidence

Assistant-automated GUI evidence for the parent-interpreted placement
(`slot.*`) surface (ADR evidence item (4), assistant portion). Capture is
**launch + per-monitor-DPI-aware `CopyFromScreen` over `GetWindowRect` +
assistant pixel analysis** (`PrintWindow` reads back blank under
DirectComposition). The owner-visible smoke is T6's; this baseline does
not replace it.

## Capture driver

[`capture-placement-demo.ps1`](./capture-placement-demo.ps1) — re-tuned
for the **current** gallery layout (the Phase 6/7
[`capture-lightbox.ps1`](../../phase-6/implementation/evidence/capture-lightbox.ps1)
click coordinates are stale after the placement-demo button was inserted).
It launches `gallery-rust`, finds the Gallery HWND by title, sizes the
window, and drives the home buttons by posting `WM_LBUTTON*` (and a
`SetCursorPos`+`mouse_event` fallback).

**Environment requirement.** Synthetic input drives the wasamo
Composition app's buttons only on a **real / elevated visible desktop**;
inside a restricted (sandboxed) session the injected input is dropped and
the button never fires (the toggle / lightbox stays closed). Run the
captures on a non-sandboxed desktop. The placement-demo state defaults to
`false` in `gallery.ui`, so every frame below is regenerable from the
committed code by re-running the driver:

```
# home + placement demo (click "Open placement demo")
pwsh -File capture-placement-demo.ps1 -Width 820 -Height 720 -OutputPrefix t5 -OpenDemoAt "410,399"
# same-position lightbox at the Phase 6/7 baseline size (click "Open lightbox")
pwsh -File capture-placement-demo.ps1 -Width 800 -Height 600 -OutputPrefix t5 -OpenLightboxAt "400,341"
```

## Frames

### `t5-home.png` — clean home (false-state, regenerable)

The gallery home with `is_placement_demo_open = false` (committed default).
The "Open placement demo" button is present; the demo overlay is absent.
This is the genuine false-state captured from the current commit (not a
probe), and the baseline the demo frame is reached from by one click.

### `t5-placement-demo.png` — the positive controls (canonical frame)

Reached by clicking "Open placement demo" on `t5-home.png`. Two on-screen
positive controls, both read off the migrated `slot.*` / `SlotData` model:

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

### `t5-lightbox-slot-current.png` — same-position re-render proof

The current-branch (`slot.*`-migrated) lightbox, captured at the Phase
6/7 baseline window size (800×600) by clicking "Open lightbox". Compared
to the pre-migration baseline
[`../../phase-6/implementation/evidence/t7-lightbox-open.png`](../../phase-6/implementation/evidence/t7-lightbox-open.png)
(also 800×600): the scrim fills the whole window (Fill/Fill), and the 4:3
"Lightbox photo placeholder" is centered, with the "Gallery image 01"
caption directly below it, the "Box 4:3 placeholder…" line under that, and
the "< > x" nav buttons centered at the bottom — the **same centered
overlay arrangement** as the baseline. The `slot.*` migration is a pure
re-expression of placement and lands the lightbox content in the same
positions. (Home content *behind* the semi-transparent scrim differs
between the two frames because the gallery `.ui` evolved across phases;
that is not the lightbox placement under test.)

## "Gallery does not surface the contrast" (recorded in [../log.md](../log.md))

The shipped gallery's incidental layout does **not** exercise the
placement positive controls — its ZStack children are all
`slot.h-align: stretch`, and the main-screen Grid does not render visibly
at the capture size — which is why T5 adds the deliberate placement-demo
surface. (A pre-implementation probe confirmed both this and that the
environment renders non-blank GUI.)

## Phase-8 removal

The placement-demo surface (`state is_placement_demo_open`, the "Open
placement demo" button, and the `if is_placement_demo_open { … }` overlay
in `examples/gallery/gallery.ui`) is throwaway Phase-7b verification
scaffolding, recorded for removal at the M3 Phase 8 close alongside the
other per-phase verification surfaces. See the T5 carry-forward in
[../log.md](../log.md).
