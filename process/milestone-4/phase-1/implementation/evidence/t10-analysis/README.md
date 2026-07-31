# T10 assistant GUI evidence — captures, comparisons and analysis

Frames were captured by
[`capture-t10-controls.ps1`](../capture-t10-controls.ps1), except the three
control-C legs, which came from
[`capture-t10-control-c.ps1`](../capture-t10-control-c.ps1). All on
2026-08-01, on the 125% development desktop (physical 2452 × 1291, logical 1962 × 1033,
monitor DPI 120), against a build produced by `cargo build -p wasamo-runtime
--release` followed by `cargo build --release --workspace` (T1 finding F-5,
T3 finding F-21).

**The capture harness declared Per-Monitor-Aware V2 and read the level back**
before touching a rectangle; `harness awareness : PER_MONITOR_AWARE_V2 (READ
BACK, not the call's return value)` appears in every `measurements.txt` in
this directory's sibling sets. A DPI-unaware observer would have been answered
in virtualized coordinates and every number below would have been wrong by
exactly 1.25, internally consistent and plausible (T9 finding F-48); and the
call that sets the awareness can itself fail against state established before
the script ran, which is why the readback rather than the return value is what
is recorded (T9 finding F-49). The harness aborts if the readback is not PMv2.

## What was captured

| Set | Build | Posture | Client (physical) | Purpose |
|---|---|---|---|---|
| `t10-shipped-created/` | branch tip | aware | 982 × 703 (created) | The measurement check — the created window, never moved or resized |
| `t10-unaware-created/` | branch tip | `__COMPAT_LAYER=DPIUNAWARE` | 980 × 701 (created) | The unaware half of the measurement check, so its row is measured here rather than carried from T4 |
| `t10-aware-a/`, `t10-aware-b/` | branch tip | aware | 1200 × 720 | Control A / B, `s = 1.25` side |
| `t10-unaware-a/`, `t10-unaware-b/` | branch tip | `__COMPAT_LAYER=DPIUNAWARE` | 1200 × 720 | Control A2 / B, `s = 1` side, **same bytes** |
| `t10-base-a/`, `t10-base-b/` | `80d79c4` (phase base), separate `CARGO_TARGET_DIR` | inherently unaware | 1200 × 720 | Control A1, "before" |
| `t10-mutation-inbound/` | branch tip, T5's inbound seam removed at all three sites | aware | 1200 × 720 | The run that shows control B can fail |
| `t10-aware-restored/` | accepted source after `cargo clean -p wasamo-runtime --release` + workspace rebuild | aware | 1200 × 720 | F-40 restoration |
| `t10-delivery-check/` | the **staged** `gallery-zig.exe` + `wasamo.dll` handed to T11 | aware | 982 × 703 (created) | The delivered copy runs, not just the build tree |

**Occlusion.** `CopyFromScreen` photographs a screen rectangle, not a window,
so anything in front of the host at capture time lands in the frame and the
artifact does not say so. Both harnesses now check four interior client points
with `WindowFromPoint` and refuse to record a frame if any of them belongs to
another window; `occlusion check : 4 interior points, all owned by the host
window` appears in each `measurements.txt` taken after that.
**The frames committed before the check existed do not carry that line**, and
their freedom from occlusion rests on inspection — every one was looked at, by
the author and by the independent review — not on the guard. The guard was
verified not to false-positive, and a capture taken with it is byte-identical
to `t10-aware-a` over all 864,000 pixels, so it is capture-neutral.

### Reproducing the base-commit side

```
git worktree add --detach <tmp>/base-80d79c4 80d79c4
cd <tmp>/base-80d79c4
$env:CARGO_TARGET_DIR = "<tmp>/base-target"     # never the repo's target/
cargo build -p wasamo-runtime --release
cargo build --release --workspace
```

then `capture-t10-controls.ps1 -Exe <tmp>/base-target/release/gallery-rust.exe`.
The separate `CARGO_TARGET_DIR` is F-40's requirement and the worktree is
removed afterwards, so `git worktree list` in a fresh clone shows nothing.

## The measurement check — the gallery's full signature at the landed declaration

Risk R-9 was closed at T9, whose three-host probe ran `counter-c` /
`counter-rust` / `counter-zig`. Tiles per row is a **gallery** property, and
the gallery's numbers had only ever been taken under a *throwaway*
declaration (T4, T5). Measured here against the landed one:

| Set | Posture | outer | client | layout extent | tiles/row |
|---|---|---|---|---|---|
| `t10-shipped-created` | declared PMv2 | 1000 × 750 | 982 × 703 | 785.6 × 562.4 DIP | **7** |
| `t10-unaware-created` | `__COMPAT_LAYER=DPIUNAWARE` | 1000 × 750 | 980 × 701 | 784 × 560.8 | **7** |

The rectangle agrees with T4's throwaway-declaration probe and with T9's
three counter hosts. The tile count is the number plan.md §T10 predicted the
landed state must show: **9 was the pre-T5 signature**, and a T10 that had
inherited 9 would have been pinning a half-finished phase.

**Both rows are measured here.** The unaware row was first written by citing
T4's three-state table, which is a number taken before T5, T6 and T7 landed —
at the one cell the responsibility re-audit says needed re-taking against the
landed declaration. Tile counts are read off the frames as tile-fill runs at
`y = 100`; they are not a field the harness emits.

**The rectangle alone does not separate anything** — T1 measured an unaware
process at 1000 × 750 too, because DWM stretches the logical 800 × 600 by the
same factor, and `t10-base-a`'s own record shows the created client at
980 × 701 for exactly that reason. **Nor does the tile count, now that both
rows read 7**: T4's three-state table could use the pair (rectangle, tiles)
because its third row read 9, which was the pre-T5 signature. With T5's seam
landed the two postures agree on both, so what separates them at this size is
the awareness readback and the two-pixel client difference. The pair that used
to do the work no longer does.

## Control A — crispness, before and after

`compare-frames.ps1` over the full 1200 × 720 client (no inset — these are
client captures, so there is no non-client frame to exclude):

| Comparison | Differing px of 864,000 | Max per-channel delta |
|---|---:|---:|
| `t10-aware-a` vs `t10-aware-b` (repeatability) | **0** | — |
| `t10-unaware-a` vs `t10-unaware-b` (repeatability) | **0** | — |
| `t10-base-a` vs `t10-base-b` (repeatability) | **0** | — |
| `t10-base-a` vs `t10-unaware-a` (base vs shipped, both `s = 1`) | 23,104 | 222 |
| `t10-base-a` vs `t10-aware-a` (**A1**, phase pair) | 23,385 | 223 |
| `t10-unaware-a` vs `t10-aware-a` (**A2**, posture pair) | 22,725 | 221 |
| `t10-aware-a` vs `t10-mutation-inbound` | 92,805 | 252 |
| `t10-aware-a` vs `t10-aware-restored` | **0** | — |

**Three byte-identical repeatability pairs is a stronger result than F-33
measured, and it does not retire F-33.** T5 measured 25 differing pixels a day
apart and 149 on a session's first launch, over *window-rect* captures whose
border is alpha-blended against the desktop. These are client-only captures of
a topmost window. That is a different measurement, not a refutation of the
earlier one, and the baseline discipline (re-capture, agree two captures per
side) is what produced these numbers rather than something they license
skipping.
**And the strongest repeatability datum here is one this document originally
missed** (independent review): `t10-control-c/1-before.png` is byte-identical
to `t10-aware-a/gallery-client.png` — captured **2.5 hours earlier, at a
different screen position, from a separate process launch, by a different
script**. The first version of this paragraph explained the zeros as "taken
minutes apart in one session", which that pair falsifies.

**The pixel counts above do not establish crispness in either direction.** A
large max delta proves only that the difference is outside the drift bound
this phase measured; a small one proves nothing; and neither says what moved
(T5 finding F-33, sharpened at the T5 round-3 review). Crispness is a
glyph-shape judgement and is made below, on the magnified pairs.

### The magnified pairs

Crops are nearest-neighbour ×5 via [`magnify-crop.ps1`](../magnify-crop.ps1),
which pins the interpolation mode so the script cannot make either side look
sharper or softer than it was captured.

Status-bar run. Crop rectangle `(8, 692) 210 × 22`, identical for all three
postures, so the comparison is over the same region at the same magnification.
The full string is `18 placeholders - Image and hit-testing are M4` and the
crop holds its first 26 characters, `18 placeholders - Image and`.

- [`status-base-80d79c4-5x.png`](./status-base-80d79c4-5x.png) — the phase
  base. Stems are two to three pixels wide and unevenly so, with a broad grey
  halo on both sides of every vertical; the `8`, `e`, `a` and `o` counters are
  filled to a muddy mid-grey rather than left open; the hyphen is a **two-row
  bar with no fully saturated pixel in it**. This is a 96-DPI rasterization
  enlarged by DWM, and the softness is the bitmap stretch.
- [`status-unaware-posture-5x.png`](./status-unaware-posture-5x.png) — the
  branch-tip binary with its declaration refused. Soft in the same way, which
  is the point of having it: the softness is the posture, not the vintage of
  the code.
- [`status-aware-5x.png`](./status-aware-5x.png) — the branch tip. Verticals
  are narrow and consistent stem to stem, edges fall off within one pixel
  instead of two or three, the same counters stay open, and the hyphen is a
  **single row with saturated pixels through its middle**.

Tile label, `IMG 001 #0`, crop rectangle `(22, 128) 110 × 22` — over the tile
fill rather than the window background:

- [`tile-base-80d79c4-5x.png`](./tile-base-80d79c4-5x.png) and
  [`tile-unaware-posture-5x.png`](./tile-unaware-posture-5x.png) — both soft.
  The `M` diagonals are stepped and grey-fringed, the `G` counter is closed
  up, the `#` crossbars merge.
- [`tile-aware-5x.png`](./tile-aware-5x.png) — the same run at 120 DPI. The
  `M` diagonals resolve, the `G` counter opens, and the `#` shows four
  separate strokes.

**A1 and A2 answer different questions and neither is described as doing the
other's work.** A1 (`t10-base-*` vs `t10-aware-*`) is the phase-level pair
risk R-1 is about: the base build has no declaration, no conversion seams and
no rasterization work, so its blur is the blur the phase exists to remove. A2
(`t10-unaware-*` vs `t10-aware-*`) comes from the *same executable* — the only
difference is that the AppCompat shim set the process awareness before Wasamo
code ran, so `runtime::init`'s declaration took `ERROR_ACCESS_DENIED` and
DD-M4-P1-001's failure handling tolerated it — so its difference cannot be
attributed to any other change on the branch. **What separates T3 / T5 / T6's
rendering work from the declaration is neither of these but the third row of
the table** (`t10-base-a` vs `t10-unaware-a`, both at `s = 1`): 23,104 pixels,
which is the same class T6 measured at 100% (9,360–24,868, max delta 220–249)
and is the `ceil` allocation plus DD-M4-P1-006's brush mapping, not the scale.

**A2 is also the first rendered evidence that the tolerated-declaration-failure
path ships working.** Every prior artifact for it — DD-M4-P1-001's unit tests,
`dpi_awareness_tolerated_failure_integration.rs` — is headless. Here a real
host whose declaration was denied comes up, lays out and renders.

## Control B — logical layout invariance across the scale factor

Both sides were driven to the **same physical client rectangle** rather than
the same outer rectangle, so F-28's tolerance drops out instead of being
stated. Reached exactly, in one iteration, on every run:

| Set | Window awareness | `GetDpiForWindow` | Outer | Client | Non-client frame | Layout extent |
|---|---|---:|---|---|---|---|
| `t10-aware-a` | PER_MONITOR_AWARE_V2 | 120 | 1218 × 767 | **1200 × 720** | 18 × 47 | **960 × 576** |
| `t10-unaware-a` | UNAWARE | 96 | 1220 × 769 | **1200 × 720** | 20 × 49 | **960 × 576** |

The non-client frames differ — 18 × 47 against 20 × 49 — which is exactly the
DPI-indexed-metrics behaviour T4 decomposed, and it is why controlling the
outer rectangle would have left the two layouts about 1.6 DIP apart. Both
sides receive **960 × 576**, a multiple of 24 in each axis (T8's
normalisation rule), and the residual against the target is `0x0` in both.

The two extents are reached by different routes and this is written out rather
than assumed: the aware host divides the physical client by its own window DPI
(120), and the unaware host is handed a client the OS already divided by the
monitor's scale. `measurements.txt` names the denominator it used in each run.

**Result: the layouts are identical, and bit-identical in position.** Measured
at the independent review rather than inferred from the frames looking alike:
the six toolbar button runs sit at exactly `10..69 | 80..177 | 188..295 |
782..911 | 922..1029 | 1039..1189` in the aware, unaware **and** base frames;
the tile runs match; the status bar's top edge is `y = 685` in all three.
Element order and wrap positions do not merely agree to a tolerance — they
agree exactly. The only differences are the glyph rasterization described
above.

### The run that shows control B can fail

Invariance is evidence only if a wrong build breaks it.
[`t10-mutation-inbound/`](../t10-mutation-inbound/) is the branch tip with
T5's inbound `GetClientRect → layout` division removed at **all three**
production sites — `window::set_root`, the `WM_SIZE` arm, and
`emit::flush_layout` — so layout receives physical pixels as though they were
DIP. Same controlled client, same aware posture, and the frame is not subtly
different:

- **11 tiles per row instead of 9.** Measured as tile-fill runs at `y = 100`:
  nine complete tiles from `x 15..124` to `1015..1124`, then `1140..1199` —
  **the tenth, 60 of its 110 px, clipped by the right edge**. The eleventh
  begins at `x 1265`, entirely outside the 1200-px client, and row 2 starts at
  `IMG 012 #11`, which is what establishes eleven per row rather than ten.
- **The toolbar's right-hand group is pushed off the window** — `Scroll down`
  is cut off at the right edge (118 of its 130 px, label still legible) and
  `Scroll up` / `Open lightbox` are gone entirely.
- **The status bar is gone**, pushed below the client.
- 92,805 differing pixels against the accepted frame, max per-channel delta
  252.

Element order survives; wrap positions and the reachable extent do not. This
is the same defect class T5 measured as 9 tiles against the accepted 7 at the
smaller client, seen at 960 DIP.

**Restoration, per T6 finding F-40.** The mutation was reverted with `git
checkout`, followed by `cargo clean -p wasamo-runtime --release` (9 files,
6.9 MiB removed) and a full `-p wasamo-runtime` then workspace release
rebuild; `t10-aware-restored/` is byte-identical to `t10-aware-a/` over all
864,000 client pixels. Byte identity distinguishes the accepted build from
*this* render-changing mutation and is not general source-identity evidence;
the clean-and-rebuild record is what covers a render-neutral one.

**The base tree's own posture readback is a second identity check.**
`t10-base-a/measurements.txt` records `window awareness : UNAWARE`. The
branch-tip `wasamo.dll` declares PMv2, so had the base host loaded the wrong
DLL — the F-40 failure mode — this line would have read
`PER_MONITOR_AWARE_V2`. The separate `CARGO_TARGET_DIR` is therefore
confirmed by behaviour and not only by configuration.

## Control C — captured, 125% → 150% → 125%

The owner changed **Settings > System > Display > Scale** while the window was
up; [`capture-t10-control-c.ps1`](../capture-t10-control-c.ps1) polled
`GetDpiForWindow` and took all three legs.
[`t10-control-c/`](../t10-control-c/), 2026-08-01 03:16.

| Leg | `GetDpiForWindow` | Outer | Client | Non-client | Layout extent |
|---|---:|---|---|---|---|
| `1-before` | 120 | 1218 × 767 | 1200 × 720 | 18 wide, 47 tall | **960 × 576 DIP** |
| `2-changed` | 144 | 1462 × 920 | 1440 × 864 | 22 wide, 56 tall | **960 × 576 DIP** |
| `3-restored` | 120 | 1218 × 767 | 1200 × 720 | 18 wide, 47 tall | **960 × 576 DIP** |

This is the **real OS path**: the window was not touched after the "before"
frame, so the rectangle at 144 DPI is the one Windows supplied through
`WM_DPICHANGED` and applied through T7's handler. The window stayed
`PER_MONITOR_AWARE_V2` throughout.

**What the frames show.** `2-changed.png` places the same **9 tiles per row in
2 rows**, the same three tab buttons left and three action buttons right, and
the status bar on the same baseline as `1-before.png`. Element order and wrap
structure are preserved, which is what this control asserts.

**Crispness survived the change**, which no other artifact in this phase
covers: every earlier crispness frame is at the window's *creation* scale.
[`status-changed-144dpi-5x.png`](./status-changed-144dpi-5x.png) is the status
run rasterized at 144 DPI — crop `(10, 830) 252 × 27`, the same region in DIP
as the 120-DPI crops, so at ×5 of a 1.2× denser raster it **prints 1.2× larger
than they do**; the comparison is of stroke quality, not of size — narrow even stems, counters open in `8`, `e`, `a`
and `o`, a clean one-pixel hyphen. The failure this rules out is T6's
`t6-scaled-surface-identity` signature: geometry that follows the new scale
while the text surface stays at the old resolution and is scaled up by the
Visual, which would look like
[`status-base-80d79c4-5x.png`](./status-base-80d79c4-5x.png) does.

**The round trip is byte-identical.** `3-restored.png` matches
`1-before.png` over all 864,000 client pixels — 0 differing. Returning to the
original scale reproduces the original frame exactly, so nothing in the change
path leaves residue in the rendered result.

**The DIP extent came back exactly, and that is not claimed as a property of
the implementation.** plan.md §T10 asserts element order and wrap structure,
explicitly **not** bit-exact wrap positions, because on this path the OS
chooses the rectangle and the non-client frame moves by its own DPI-indexed
metrics (18 × 47 → 22 × 56 here). The outer rectangle went 1218 → 1462 where
`1218 × 1.2 = 1461.6`, and 767 → 920 where `767 × 1.2 = 920.4` — rounded up in
width and down in height — and the client that fell out of those two
independent movements happened to divide by 1.5 exactly. **Whether Windows
computed the suggested rectangle to preserve the client extent, or the
rounding simply landed well, this run does not distinguish**: one step the
other way in width (1461) would have given 959.33 DIP. The controlled client
was a multiple of 24 in DIP (T8's normalisation rule), which is what made an
exact result *possible*; it is not what makes it *guaranteed*.

**What control C does not establish.** No intermediate frame was captured —
the harness waits about 1.5 s after detecting the DPI change before shooting —
so this says nothing about whether a stale intermediate projection is
presented during the change, which T7's retrospective left open and assigned
to T11's capture. And a display-setting change is not a monitor crossing: it
is a second path through the same handler, not the same path. T11 owns the
literal form.

## Control C — how it came to be captured

**This section first said "two frames across a live display-scale change are
not obtainable on this machine". That claim was wider than its evidence and
is withdrawn.** What was measured is that three *programmatic* routes are
unavailable in this session: one monitor, an RDP session on `Microsoft Remote
Display Adapter`,
`DisplayConfigGetDeviceInfo(DISPLAYCONFIG_DEVICE_INFO_GET_DPI_SCALE)`
returning `ERROR_GEN_FAILURE` for the only active source, and no
`PerMonitorSettings` registry key because the session scale is negotiated from
the RDP client. **The Settings UI was never tried**, and nothing measured here
says anything about it.

The wider claim also carried an assumption that was never in the plan: that
the assistant had to *cause* the change. The ADR set's verification-closure
item (5) says the assistant **captures** the path. A human changing Scale in
Settings while
[`capture-t10-control-c.ps1`](../capture-t10-control-c.ps1) polls
`GetDpiForWindow` is control C as written.

The harness captures three legs and never takes the keyboard or the
foreground; it raises the host with `HWND_TOPMOST` immediately before each
capture so the Settings window can stay in front while it waits. Before the
run above, its capture step had been exercised by the before leg and **the
DPI-change comparison had not** — that gap is now closed by the run itself,
which took the changed and restored legs through the same branches.

A cross-process synthesised `WM_DPICHANGED` was never available as a
substitute — its `LPARAM` is a pointer to the suggested `RECT` and Windows
does not marshal it across a process boundary — and the instrumented-host
fallback was not needed.

## What these captures do not establish

- **Nothing about real monitor-to-monitor delivery.** Neither side of control
  B is a real 100% monitor; the `s = 1` side is an unaware process on the same
  125% desktop. The comparison is the `s = 1` vs `s = 1.25` one risk R-2 needs
  and it is exact rather than approximate, which is why it was preferred — but
  it says nothing about `WM_DPICHANGED` arriving from a monitor crossing. That
  is T11's, and T8's synthesised message is the automated half.
- **Nothing about states other than the default sub-screen.** These frames are
  `gallery-default` only. The click-driven states (tab switch, lightbox
  insertion, bound-label update) are covered by T3's and T6's six-frame sets
  at their own postures, not re-photographed here.
- **Crispness is judged, not measured.** The magnified comparison is a human
  (here, assistant) reading of glyph shape. It is the assistant baseline that
  precedes the owner's human-visible smoke and does not replace it.
