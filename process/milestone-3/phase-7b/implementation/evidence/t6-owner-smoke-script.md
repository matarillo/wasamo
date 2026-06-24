# M3-Phase 7b T6 — owner-manual GUI smoke: observation script

This is the **owner-performed** GUI smoke for ADR evidence item (4) — a
separate gate from T5's assistant baseline (which is launch + DPI-aware
screenshot + assistant analysis under [README.md](./README.md)). The
assistant prepares this runnable host + script; the smoke itself is
owner-performed and cannot be discharged by the assistant baseline.

The positive control is **placement varied**: an explicit alignment lands
where expected, a contrasting one lands elsewhere, and omitted placement
falls to the per-container default. A single static frame a hardcoded
widget tree could equally produce is **not** evidence — what the owner is
confirming is that *changing the placement keyword changes the rendered
position*.

> **Same-position note.** The pre-migration bare ZStack syntax is rejected
> on this branch (T4), so there is **no live old-vs-new comparison** to
> perform. The same-position invariant (the `slot.*` migration renders at
> the pre-migration positions) was **closed in T5** as the assistant
> portion — `t5-lightbox-slot-current.png` vs `t5-lightbox-bare-baseline.png`
> (T4-pre `3134287`, same gallery content) are pixel-position identical
> (bbox `x=150..648 y=60..544`). The owner may corroborate visually against
> that recorded pair when opening the lightbox, but does **not** re-derive
> the old surface.

## Environment

- Visible Windows desktop session (local physical machine or a screen-backed
  RDP/VNC). A plain SSH session is not a valid basis for a GUI smoke.
- The owner clicks the on-screen buttons **with a real mouse**, so the
  synthetic-input limitation that affects the assistant capture driver
  (dropped in a sandboxed session — see [README.md](./README.md)
  "Environment requirement") does not apply here.

## Build and launch

From the repo root:

```powershell
cargo build --release -p gallery-rust
.\target\release\gallery-rust.exe
```

The window opens with title **"Gallery"**. `is_placement_demo_open` and
`is_lightbox_open` both default to `false`, so the home screen shows the
"Open placement demo" and "Open lightbox" buttons and neither overlay.

## Observation 1 — ZStack `slot.h-align` (three keys → three positions)

1. On the home screen, click **"Open placement demo"**.
2. The placement-demo overlay appears (dark background), titled
   "Placement demo — Phase 7b verification (removed at Phase 8)".
3. Under "ZStack slot.h-align (same box, three keys -> three x positions):",
   three identically-sized labelled boxes overlay a dark panel:
   - **blue** box labelled `slot.h-align: start`,
   - **gray** box labelled `omitted -> center`,
   - **orange** box labelled `slot.h-align: end`.

**PASS:** the blue box sits at the **left**, the gray box at the **center**
(the per-container default for the omitted axis), and the orange box at the
**right** — three distinct horizontal positions in keyword order.

**FAIL:** the three boxes collapse to one position, appear in the wrong
order, or the omitted box is not centered. (A placement-ignoring
implementation could only produce a single stacked position — that is the
look-alike this control rules out.)

Reference frame: `t5-placement-demo.png` (top panel).

## Observation 2 — Grid cell placement (row / column / span + alignment)

In the same overlay, under "Grid placement: distinct row/column/span;
r0c0 stretch-fills vs r0c2 centered:", a 3-column × 2-row grid shows:

- **r0c0** (blue) — top-left cell, **stretch-fills** its cell (default
  alignment).
- **r0c2** (pink) — top cell of the **third column**, a **small box
  centered** in its cell (`h-align: center v-align: center`).
- **r1 span 3** (green) — the **second row**, **spanning all three
  columns**.

**PASS:** the three cells render at their distinct row/column/span, and the
stretch-fill (r0c0) vs centered (r0c2) contrast is visible — same Grid,
different per-cell alignment → different in-cell rendering.

**FAIL:** cells overlap at one origin, the span cell does not span, or
r0c0 and r0c2 look identical (no stretch-vs-centered contrast).

Reference frame: `t5-placement-demo.png` (lower panel).

4. Click **"Close demo"** to return to the home screen.

## Observation 3 — lightbox (same-position corroboration + surrounding behaviour)

1. On the home screen, click **"Open lightbox"**.
2. A centered photo/caption card renders over a scrim.

**PASS:** the lightbox card renders centered and legible, matching the
recorded `t5-lightbox-slot-current.png`. (This visually corroborates the
T5 same-position proof; it is not a live old-vs-new comparison.)

3. Close the lightbox; scroll the main gallery and confirm WrapPanel
   reflow / ScrollView behaviour stay correct around the placed children.

## Observation 4 — window close

Close the window (Alt+F4 / ×). **PASS:** no crash dialog.

## Recording the result

The owner reports each observation as pass / fail in chat (Japanese). On
**accept**, the T6 checklist flips and the T6 step-end retrospective is
recorded at [../../retrospectives/t6.md](../../retrospectives/) with the
owner result. On **fail**, the fix lands additively on the
`feat/m3-phase-7b-t6` branch (the fix container), and the smoke re-runs to
green before T6 closes — following the M3-Phase 4 T6 smoke-fail → fix → re-smoke
precedent.
