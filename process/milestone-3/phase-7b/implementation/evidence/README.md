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
# current slot.* lightbox (click "Open lightbox")
pwsh -File capture-placement-demo.ps1 -Width 800 -Height 600 -OutputPrefix t5 -OpenLightboxAt "400,341"
# T4-pre bare-syntax baseline (build gallery-rust at commit 3134287 in a
# worktree, then point -ExePath at it; produces t5-lightbox-bare-baseline.png):
#   git worktree add --detach ../wasamo-t4pre 3134287
#   (in the worktree) cargo build --release -p wasamo-runtime -p wasamo-dll -p gallery-rust
pwsh -File capture-placement-demo.ps1 -ExePath ..\..\..\..\..\..\wasamo-t4pre\target\release\gallery-rust.exe `
     -Width 800 -Height 600 -OutputPrefix t5-baseline -OpenLightboxAt "400,341"
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

### `t5-lightbox-slot-current.png` + `t5-lightbox-bare-baseline.png` — same-position re-render proof

The same-position proof isolates the **`slot.*` migration** from gallery
content evolution by comparing two frames built from the **same gallery
content**, differing only in placement syntax:

- `t5-lightbox-slot-current.png` — the current-branch lightbox
  (`slot.h-align: stretch` …), captured at 800×600 by clicking "Open
  lightbox".
- `t5-lightbox-bare-baseline.png` — the **T4-pre** lightbox (commit
  `3134287`, the last commit on the bare `h-align` / `v-align` surface,
  built in a throwaway worktree because the current `wasamoc` rejects the
  bare form), same window size. Its lightbox `Grid` is identical
  (`columns: 1* 400 1*  rows: 1* 300 64 44 1*`); only the placement
  *syntax* differs.

These two are **pixel-position-identical** — the light photo-placeholder /
caption region measures `x=150..648  y=60..544` in **both** (extracted by
a bbox scan). The bare and `slot.*` forms lower to the same `IrSlotData`
(T4 lower tests) and so render at the same coordinates: the migration is
position-preserving.

> Note on the Phase 6 evidence: the earlier
> [`../../phase-6/implementation/evidence/t7-lightbox-open.png`](../../phase-6/implementation/evidence/t7-lightbox-open.png)
> is **not** the right baseline — it is from an older gallery version, so
> its photo region (`y=80..524`) differs by ~20px from the current frame
> for reasons of gallery-`.ui` evolution, *not* the placement migration.
> The T4-pre baseline above controls for that by holding the gallery
> content fixed.

## "Gallery does not surface the contrast" (recorded in [../log.md](../log.md))

The shipped gallery's incidental layout does **not** exercise the
placement positive controls — its ZStack children are all
`slot.h-align: stretch`, and the main-screen Grid does not render visibly
at the capture size — which is why T5 adds the deliberate placement-demo
surface. (A pre-implementation probe confirmed both this and that the
environment renders non-blank GUI.)

## T6 — owner-manual GUI smoke frames

The **owner-performed** GUI smoke (ADR evidence item (4), owner portion;
the assistant baseline above does not replace it) was run on a visible
desktop with a real mouse per
[t6-owner-smoke-script.md](./t6-owner-smoke-script.md), and the owner
accepted all observations as pass (2026-06-23). The owner-captured frames:

- `t6-home.png` — gallery home, false-state (both "Open placement demo"
  and "Open lightbox" buttons present, no overlay).
- `t6-placement-demo.png` — the placement-demo overlay carrying both
  positive controls in one frame: **ZStack** `slot.h-align: start` (blue)
  at the left, `omitted -> center` (gray) at the center, `slot.h-align:
  end` (orange) at the right — three distinct x positions; **Grid**
  `r0c0 stretch` (blue) stretch-filling its top-left cell vs `r0c2
  centered` (pink) as a small centered box in the column-3 cell, and
  `r1 span 3 columns` (green) spanning the full width on row 1 (distinct
  row/column/span + the stretch-vs-centered alignment contrast).
- `t6-lightbox.png` — the lightbox card centered over the gallery,
  corroborating the T5 same-position frame; the main-screen header Grid
  (`Header spans 3 columns`, `C0 fixed 120` / `C1 star 1*` / `C2 star 2*`)
  also renders at this window size.

These owner frames mirror the T5 assistant frames on the same surface;
their added value is the **owner-confirmed, real-desktop provenance** of
the human-visible smoke (M3-Phase 4 T6 precedent), not new visual content.

> The demo Grid uses **fixed** tracks (`columns: 220 220 220`,
> `rows: 56 56`) and so does not resize with the window — a deliberate T5
> choice because star (`*`) tracks / `aspect` cells abort arrange in this
> nested overlay (T5 layout finding, T7 / phase-end triage carry-forward).
> Observation 2's positive control is the in-frame alignment contrast, not
> resize responsiveness, so the fixed Grid does not weaken the smoke. The
> ZStack panel is width-following (the `start`/`end` spread widens with the
> window).

## Phase-8 removal

The placement-demo surface (`state is_placement_demo_open`, the "Open
placement demo" button, and the `if is_placement_demo_open { … }` overlay
in `examples/gallery/gallery.ui`) is throwaway Phase-7b verification
scaffolding, recorded for removal at the M3 Phase 8 close alongside the
other per-phase verification surfaces. See the T5 carry-forward in
[../log.md](../log.md).
