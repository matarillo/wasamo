# T11 owner human-visible smoke — captures, comparisons and analysis

The literal cross-monitor form of positive control C: a window dragged between
two panels at different scale factors, on a machine that is **not** the
development machine.

**Captured by the owner on 2026-08-03**, on a laptop with the internal panel at
**150%** and an external display at **100%**, both owner-set before the host was
launched. The subject is the T10 delivery set — `gallery-zig.exe` +
`wasamo.dll` — transferred by `scp`, extracted, and **launched from the
delivery directory**. No repository, no toolchain and no build exist on that
machine; nothing was rebuilt for this task.

**The instrument is a human plus Windows' built-in window capture.** No script
ran on the observing machine. The frames were returned by `scp` and analysed
here with [`t11-edge-stats.ps1`](../t11-edge-stats.ps1) and
[`t11-frame-diff.ps1`](../t11-frame-diff.ps1); the magnified crops come from
[`magnify-crop.ps1`](../magnify-crop.ps1) with its interpolation pinned to
nearest-neighbour.

**No number in this file is compared against the development machine's.** The
non-client metrics, the theme and the panel are all this machine's. Where a
number corroborates one recorded at §T4 or §T10, that is said explicitly and
the comparison is of *shape*, not of value.

## What was captured

| Frame | How the window got there | Monitor | What it is |
|---|---|---|---|
| `1-internal-150-aware.png` | drag | internal 150% | Before the crossing |
| `2-external-100-aware.png` | drag | external 100% | After the crossing, settled |
| `7-external-100-clicked.png` | drag | external 100% | After clicking the Favorites tab on the destination monitor |
| `3-internal-150-restored.png` | drag | internal 150% | After dragging back |
| `9-snapmove-internal-150.png` | `Win+Shift+→` | internal 150% | Before, on the non-modal path |
| `10-snapmove-external-100.png` | `Win+Shift+→` | external 100% | After, on the non-modal path |
| `11-snapmove-internal-150-restored.png` | `Win+Shift+←` | internal 150% | After returning, on the non-modal path |
| `4-internal-150-side-by-side.png` | — | internal 150% | Positive control: the aware run (left) and the same executable under `__COMPAT_LAYER=DPIUNAWARE` (right), in one frame |
| `5-external-100-side-by-side.png` | — | external 100% | The same pair on the identity monitor — the leg that must **agree** |
| `8-internal-175-side-by-side.png` | — | internal 175% | The same pair at a third scale; an extra leg the owner added |
| `6-taskmgr-dpi.png` | — | — | The posture readback: Task Manager's DPI Awareness column |
| `crop-{100,150,175}-{aware,unaware}-x6.png` | — | — | The same tile label from each side, magnified ×6 nearest-neighbour |

## The settled state across a real crossing

**Logical layout is preserved.** 7 tiles per row on both monitors, wrap
structure 7 / 7 / 4, element order identical. The toolbar's accent-coloured
band — from the `All` tab through `Open lightbox`, i.e. the full content width —
measures **1154 px at 150%** and **768 px at 100%**, a ratio of **1.5026**.

In DIP that is **769.3 against 768.0 — a 1.3 DIP difference**, and it is not a
defect.

**What F-28 does and does not say about that number** — narrowed after the
independent review, which was right that the first wording claimed a prediction
F-28 never made. F-28 is a measurement of the **client extent** on the
development machine at 125%: 784 DIP against 785.6, a 1.6 DIP residual, because
layout receives the client extent while the non-client frame scales by its own
DPI-indexed system metrics rather than by `s`. The 1.3 above is the **content
band**, a different quantity — the client residual less whatever the toolbar's
own padding contributes. F-28's *mechanism* is what says a residual must exist
and must not be assumed away; it predicts neither this value nor this quantity,
and the phase's rule that a comparison across machines is of shape and not of
value applies to it. The same non-invariance shows on the captured window
bounds — 1182 / 1.5 = 788.0 against 786.0 — which is a third quantity again.
**What matters for §T11 is the consequence, and that is measured directly: it
did not move a wrap position.** Element order and wrap structure are what the
task asserts, never a bit-exact position.

**The non-client scales with the window, and visibly not by `s`.** The same
band's top edge sits at **y = 61 at 150%** and **y = 42 at 100%**, a factor of
**1.452** against the width's 1.5026.

**Attributing that gap to the caption takes one step of argument, and the first
version of this paragraph asserted it instead** (independent review). What is
measured is a distance from the top of the captured frame to the first accent
pixel, and that distance is the caption height *plus* the toolbar's own top
padding. The two cannot be separated from these frames: scanning a column
through both, the caption background and the toolbar background are the same
colour, so there is no detectable boundary between them. The step is that the
padding is a **DIP** quantity the app lays out, so it scales by `s` up to
rounding — and 61 = c₁₄₄ + 1.5p against 42 = c₉₆ + p puts the caption ratio near
1.44 for any plausible p (p = 8 gives 49 / 34; p = 6 gives 52 / 36), comfortably
below 1.5 and well outside the ±1 px of layout rounding these frames show
elsewhere. So the conclusion stands — **the V2 automatic non-client scaling is
DPI-indexed, not proportional to `s`** — and it stands on that assumption, which
is now stated. DD-M4-P1-004's claim is about the **outer** rectangle, and this
is why it must stay there.

**The window's logical size survives the crossing.** Captured bounds are
1182 × 891 at 150% and 786 × 592 at 100%. Windows' window capture returns the
DWM extended frame bounds, which exclude the invisible resize border, so these
are consistent with an outer 1200 × 900 / 800 × 600 rather than a measurement
of it — recorded as corroboration of DD-M4-P1-004's outer-rectangle claim, not
as a second measurement of it.

**The round trip returns the same frame.** Inside a 4-pixel inset, frames 1 and
3 differ in **12 pixels of 1,036,642**, and all twelve are at the four corner
radii, where the window's rounded corner blends with whatever is behind it — the
window sat at a different desktop position on the way back. Outside the inset
every difference is on the outermost one or two pixels of the frame, i.e. the
same border. Client *and* caption came back identically.

**The pointer path follows the new scale.** In `7-external-100-clicked.png` the
Favorites tab is selected and All is released, on the destination monitor. T8
drives a click through a real `WM_LBUTTONUP` at a synthesised scale; this is the
only observation in the phase whose coordinate comes from a real device across a
real crossing.

## The non-modal delivery path agrees with the modal one

The drag delivers `WM_DPICHANGED` while a modal window-move loop is running.
`Win+Shift+→` delivers it with no modal loop, and T7's handler re-enters through
a nested `WM_SIZE` — which is why T7's review lane names re-entrancy through the
message loop. Only the modal path had been observed, so the second one was run.

| Comparison | Result |
|---|---|
| `10` vs `2` (destination, 100%) | 3,885 of 454,352 differ, **max per-channel delta 1**, confined to the tile-label and status-bar rows |
| `9` vs `1` (origin, 150%) | 16 of 1,036,642 differ: 12 at the corner radii, and **4 text pixels in the `All` / `Albums` tab labels** at ≤ 8 per channel |
| `11` vs `9` (its own round trip) | 4 pixels, all corner radii |

Every non-corner difference is an intensity change on an already-covered text
pixel — F-33's signature, here an order of magnitude below the 13 per channel
that finding measured, across two separate launches. The edge statistics are
identical to the drag path's (below).

## The positive control

### The posture, read back rather than assumed

`6-taskmgr-dpi.png`, Task Manager's Details tab with the **DPI Awareness**
column added:

| Name | PID | Description | DPI Awareness |
|---|---|---|---|
| `gallery-zig.exe` | 15184 | gallery-zig.exe | モニターごと (v2) — Per-Monitor V2 |
| `gallery-zig.exe` | 18408 | gallery-zig.exe | 非対応 — Unaware |

This is the level actually in force, not the declaration call's return value
(T9 finding F-49).

**Split into what this frame shows and what other frames show** (independent
review, which was right that one sentence collected too much). The frame shows
the resulting **posture** of two processes, and nothing else. That the runtime
*tolerates* a refused declaration is shown by frames 4 / 5 / 8, where the unaware
process comes up and renders. The third thing DD-M4-P1-001 §Failure handling
specifies — that the outcome of the attempt is recorded as a **diagnostic** —
was **not** read back on this machine and is not claimed here; T9 has it
headlessly. A section that cites F-49 for the difference between arranging OS
state and succeeding at it owes the diagnostic the same treatment.

**What this frame does not establish.** The list is sorted by Description and
is **scrolled** — the thumb sits about a fifth of the way down its track — so
the row immediately above the topmost visible one is hidden, and rows sharing a
Description are contiguous. A third `gallery-zig.exe` cannot be excluded *from
the image*. That only two were running is the **owner's attestation**, recorded
as such. Nothing rests on it: the mapping from process to on-screen window
comes from the owner having launched the left one normally and the right one
from a shell carrying the variable, corroborated below.

### The measurement

The same tile label (`IMG 001 #0`) in each window.
[`t11-edge-stats.ps1`](../t11-edge-stats.ps1) reports the largest luminance step
between neighbouring pixels and the share of pixels at intermediate intensity;
a natively rasterized glyph turns over inside about one pixel, a glyph
rasterized at 96 DPI and resampled up by the compositor spreads that turn over
two or more.

| Monitor scale | aware (left) | unaware (right) |
|---|---|---|
| 100% | max\|dx\| 158.6, mid-band 13.2% | max\|dx\| **158.6**, mid-band **13.2%** |
| 150% | max\|dx\| 157.0, mid-band 8.2% | max\|dx\| 106.6, mid-band 19.7% |
| 175% | max\|dx\| 156.6, mid-band 10.6% | max\|dx\| 92.3, mid-band 21.1% |

**The agreement leg fired.** At 100% every conversion in this phase is the
identity, and the two runs are indistinguishable to the last digit reported. A
control that separated them there would have been measuring window identity,
position or capture rather than rasterization. This is what makes the 150% and
175% separations attributable to the posture.

**The identification is not an appearance judgement.** `1-internal-150-aware.png`
is a window known to be the aware one — launched normally, with no variable set —
and reads 157.0 / 8.2% at 150%, which is the left window's value; the right reads
106.6 / 19.7%. Its content band is 1154 px against the left window's 774, so what
the metric tracks is not window width. The owner's account of which shell
launched which agrees.

**The aware side's `max|dx|` is flat across scales and the unaware side's is
not**: 158.6 → 157.0 → 156.6 against 158.6 → 106.6 → 92.3. Crispness independent
of the scale factor is risk R-1's claim, and this is it as a number rather than
as an argument. **The claim is confined to that statistic** (independent review):
the aware `mid-band` column reads 13.2% → 8.2% → 10.6%, which is neither flat nor
monotonic, because the share of intermediate pixels depends on how much glyph and
how much background a crop contains and the three crops are different sizes. The
unaware side is monotonic in both columns.

**The crossing frames carry this directly, and the first version of this file
left it on the control pairs and owner attestation.** The same statistic on the
frames that were never resized: 158.6 / 13.2% on the drag destination, 158.6 /
13.2% after the click there, 158.6 / 13.1% on the snap destination — identical to
the 100% aware control — and 157.0 / 8.2% on all three 150% frames, identical to
leg 0. So "text is crisp on the destination monitor" is measured on the crossing
itself, not inferred from a separate pair. The check was pointed out by the
independent review and is recorded as its finding, not as something the task
did.

**Identical statistics are not identical pixels, and the difference is
explained.** The two windows in each pair were sized by hand, so their content
bands differ — 515 px against 522 px at 100%, 774 against 761 at 150% — the tile
grid lands on different origins, and the glyph pixels differ while the character
of the rasterization does not. Measured:
`t11-frame-diff.ps1 -A crop-100-aware-x6.png -B crop-100-unaware-x6.png -Inset 0`
gives 17,712 of 61,776 at max per-channel delta 121. Recorded because
"identical" was written once here before it was checked, and it was true of the
statistics only.

Look at `crop-150-aware-x6.png` against `crop-150-unaware-x6.png`: the aware
stems carry a one-pixel fringe and the counters of the `0`s stay open; the
unaware ones have a two-pixel grey ramp on both sides of every stroke and the
`#` fills in. That is the judgement; the table is its corroboration.

## Deviations from the protocol, and one new observation

1. **The side-by-side windows were narrowed.** Two 1200-pixel windows do not fit
   on one 150% panel, so the owner resized them; the pairs show 5 tiles per row
   instead of 7. [`protocol.md`](./protocol.md) said not to resize. The
   consequence is scoped rather than waved away — and the scoping needed one
   correction of its own, because the first version of it read as an absolute
   and the very next paragraph broke it (independent review). **What the resized
   frames cannot carry is a claim about the layout the crossing produced**, since
   their width was set by hand rather than by the OS: every such claim above
   rests on 1 / 2 / 3 and 9 / 10 / 11. What they *can* carry is an observation
   about what the layout engine does **at the width they themselves establish**,
   which is a different kind of statement and is what the toolbar note below is.
   They also carry the crispness claims, which do not depend on window width at
   all. The owner notes that internal 100% +
   external 150% would have avoided the resize; re-shooting was judged not worth
   it, because a crispness comparison does not depend on window width and both
   scaling directions are already covered by the round trip.
2. **The 175% pair is an extra leg** the owner added, not in the protocol. It is
   the third point that makes the aware side's flatness a trend rather than two
   values.
3. **The intermediate-projection question was not attempted**, having been
   removed at the start gate as unanswerable by a human instrument (log.md §T11
   re-audit 1). Screen recording was offered with its asymmetry stated and
   declined. Not captured — not "captured and saw nothing".

**New observation, and it is not a DPI defect.** In every narrowed frame the
toolbar's `Favorites` and `Scroll down` overlap: the row neither wraps nor
clips when the client is too narrow for it. It appears identically at 100%, at
150% and at 175% — and at 100% every conversion in this phase is the identity —
so it is width-driven, not scale-driven, and it is outside this phase. Carried
to [handoff.md](../../handoff.md).

## Reproducing the numbers

```powershell
$d = "process/milestone-4/phase-1/implementation/evidence"
& $d/t11-edge-stats.ps1 -In $d/t11-owner-smoke/5-external-100-side-by-side.png -X 22  -Y 140 -W 76  -H 20 -Label "100% aware"
& $d/t11-edge-stats.ps1 -In $d/t11-owner-smoke/5-external-100-side-by-side.png -X 558 -Y 140 -W 76  -H 20 -Label "100% unaware"
& $d/t11-edge-stats.ps1 -In $d/t11-owner-smoke/4-internal-150-side-by-side.png -X 30  -Y 216 -W 120 -H 24 -Label "150% aware"
& $d/t11-edge-stats.ps1 -In $d/t11-owner-smoke/4-internal-150-side-by-side.png -X 839 -Y 216 -W 120 -H 24 -Label "150% unaware"
& $d/t11-edge-stats.ps1 -In $d/t11-owner-smoke/8-internal-175-side-by-side.png -X 38  -Y 234 -W 134 -H 26 -Label "175% aware"
& $d/t11-edge-stats.ps1 -In $d/t11-owner-smoke/8-internal-175-side-by-side.png -X 977 -Y 239 -W 134 -H 26 -Label "175% unaware"
& $d/t11-edge-stats.ps1 -In $d/t11-owner-smoke/1-internal-150-aware.png        -X 30  -Y 202 -W 120 -H 24 -Label "leg 0 reference"

& $d/t11-frame-diff.ps1 -A $d/t11-owner-smoke/1-internal-150-aware.png -B $d/t11-owner-smoke/3-internal-150-restored.png -Inset 4 -Map
& $d/t11-frame-diff.ps1 -A $d/t11-owner-smoke/9-snapmove-internal-150.png -B $d/t11-owner-smoke/1-internal-150-aware.png -Inset 4 -Map
& $d/t11-frame-diff.ps1 -A $d/t11-owner-smoke/10-snapmove-external-100.png -B $d/t11-owner-smoke/2-external-100-aware.png -Inset 4 -Map
```

`t11-frame-diff.ps1` exits 1 on any difference, 2 on a size mismatch and 0 when
the frames are identical, so a difference has to be read and classified rather
than passed over. (The identical arm did not exit at all until the independent
review; see log.md §T11 §Trap #4.)

**The crops, whose provenance this section omitted.** They are not the same
rectangles as the statistics above — a crop wants a little margin around the
glyphs so the shapes are legible, a statistic wants the tightest box that is the
same on both sides — so the two are recorded separately rather than left for a
reader to recover:

```powershell
& $d/magnify-crop.ps1 -In $d/t11-owner-smoke/5-external-100-side-by-side.png -Out ...\crop-100-aware-x6.png   -X 22  -Y 138 -W 78  -H 22 -Factor 6
& $d/magnify-crop.ps1 -In $d/t11-owner-smoke/5-external-100-side-by-side.png -Out ...\crop-100-unaware-x6.png -X 558 -Y 138 -W 78  -H 22 -Factor 6
& $d/magnify-crop.ps1 -In $d/t11-owner-smoke/4-internal-150-side-by-side.png -Out ...\crop-150-aware-x6.png   -X 28  -Y 212 -W 128 -H 32 -Factor 6
& $d/magnify-crop.ps1 -In $d/t11-owner-smoke/4-internal-150-side-by-side.png -Out ...\crop-150-unaware-x6.png -X 837 -Y 212 -W 128 -H 32 -Factor 6
& $d/magnify-crop.ps1 -In $d/t11-owner-smoke/8-internal-175-side-by-side.png -Out ...\crop-175-aware-x6.png   -X 34  -Y 230 -W 145 -H 34 -Factor 6
& $d/magnify-crop.ps1 -In $d/t11-owner-smoke/8-internal-175-side-by-side.png -Out ...\crop-175-unaware-x6.png -X 973 -Y 235 -W 145 -H 34 -Factor 6
```

Each crop strictly contains its own statistics rectangle, at the same offset on
both sides of a pair, and each comes from the window its name says.

## What this closes, and what it does not

**Closes, together with T8 and not alone**, AC7's third requirement. T8 proves
the handling path with a synthesised message and states that it cannot prove
that crossing a real monitor boundary delivers the same message with a usable
suggested rectangle; this is that half, on two delivery paths.

**Does not close**: whether a stale intermediate projection is ever presented as
a frame during the change (removed at the start gate, carried to
[handoff.md](../../handoff.md)); and anything about **per-window differing
scale** — both windows in every pair above were on one monitor at one scale, and
M4-Phase 8 is where two windows at two scales become a question.
