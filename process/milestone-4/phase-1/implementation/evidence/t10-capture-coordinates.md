# Re-derived capture coordinates — the T10 artifact T12 consumes (risk R-7)

T12 revises
[verification-environments.md](../../../../docs/notes/verification-environments.md)
§Observation 4. This file is the measurement input for that revision, not the
revision: it records what was measured on 2026-08-01 against the landed
declaration, what in Observation 4's current text is falsified, and a proposed
replacement. **T12 owns the wording**; nothing here is normative.

Measured on the 125% development desktop (physical 2452 × 1291, logical
1962 × 1033, monitor DPI 120) with a PMv2 harness that read its own level
back. Numbers come from
[`t10-analysis/README.md`](./t10-analysis/README.md) and the
`measurements.txt` beside each frame set; the non-client decomposition is a
separate direct probe recorded in [../log.md](../log.md) §T10.

## 1. What Observation 4 currently says, and what happened to it

> The capture tooling must be **per-monitor-DPI-aware**. On the M3-Phase 5 T6
> box (125% scale) the host is DPI-unaware, so DWM bitmap-stretches a logical
> 800×600 window to physical 1000×750; a DPI-unaware capture would crop or
> mis-scale the readback.

| Clause | Status |
|---|---|
| "the capture tooling must be per-monitor-DPI-aware" | **Still true, and now true for a second, independent reason.** See §3. |
| "the host is DPI-unaware" | **Falsified.** The runtime declares Per-Monitor-Aware V2 in `runtime::init` (DD-M4-P1-001, landed at T9). Measured: `GetWindowDpiAwarenessContext` = `PER_MONITOR_AWARE_V2`, `GetDpiForWindow` = 120. |
| "DWM bitmap-stretches a logical 800×600 window to physical 1000×750" | **Falsified as a mechanism, and the number survives.** The outer rectangle really is 1000 × 750 — but because the runtime converts the requested DIP size itself (T4's `realize_dip_window_size`), not because DWM stretches anything. |
| "a DPI-unaware capture would crop or mis-scale the readback" | **True, with the mechanism replaced.** See §3. |
| "the host's own DPI-unawareness is a separate runtime gap deferred to M4" | **Discharged.** That is this phase. |

**The trap in the third row is the one worth carrying forward.** The same
number, 1000 × 750, is produced by the unaware build and by the aware one, and
a note that keeps the number while dropping the mechanism reads as though
nothing changed. T1 measured the unaware case and T4 measured all three
states; the rectangle does not separate them and never did.

## 2. The numbers, measured against the landed declaration

An 800 × 600 DIP window on a 120-DPI monitor, `gallery-rust` and the staged
`gallery-zig` alike:

| | Declared PMv2 | `__COMPAT_LAYER=DPIUNAWARE` |
|---|---|---|
| `GetWindowDpiAwarenessContext` | `PER_MONITOR_AWARE_V2` | `UNAWARE` |
| `GetDpiForWindow` | 120 | 96 |
| Outer rectangle | 1000 × 750 | 1000 × 750 |
| Client rectangle | 982 × 703 | 980 × 701 |
| Non-client left / top / right / bottom | 9 / **38** / 9 / 9 | 10 / **39** / 10 / 10 |
| Extent the layout engine receives | 785.6 × 562.4 DIP | 784 × 560.8 logical |
| Gallery tiles per row | 7 | 7 |

Every figure is read from a PMv2 process. The two right-hand columns come from
the *same executable*; the AppCompat shim sets the process awareness before
Wasamo code runs, so the declaration is refused and DD-M4-P1-001's failure
handling tolerates it.

**A consequence for the existing tooling, measured rather than predicted.**
[`compare-frames.ps1`](./compare-frames.ps1)'s default insets (`InsetX 12`,
`InsetTop 44`, `InsetBottom 12`) were chosen against the 96-DPI non-client
frame. The aware frame's top inset is 38 rather than 39 and its side and
bottom insets are 9 rather than 10, so **the defaults still clear the frame**
— by 6 rows at the top instead of 13, and by 2 columns instead of 3. They are
not broken; the margin shrank. A later phase capturing at a scale above 125%
should re-derive them rather than inherit them, because the caption grows with
DPI while the constant does not.

## 3. Why the tool must declare — the mechanism has changed

Observation 4's stated reason is that an unaware capture of a **DWM-stretched
unaware host** would crop or mis-scale. The host is no longer unaware, so that
reason has expired. The replacement reason is stronger and applies to every
future phase:

**A DPI-unaware process asking `GetWindowRect` / `GetClientRect` about an
*aware* window is answered in virtualized coordinates — the real rectangle
divided by the system scale** (T9 finding F-48). Measured at T9: an
undeclared probe reported `outer=800x600` for windows that are really
`1000x750`. The reading is internally consistent, plausible, and wrong by
exactly the scale factor, and it lands on a different row of T4's three-state
table — so it does not look like an error, it looks like a different finding.

Two corollaries a later phase needs:

- **`GetDpiForWindow` and `GetWindowDpiAwarenessContext` are not
  virtualized.** A level or DPI readback is trustworthy from an undeclared
  caller; it is the *coordinates* that move. So "the tool read a sensible DPI"
  is not evidence that its rectangles are real.
- **Declaring is not the same as having declared** (T9 finding F-49).
  `SetProcessDpiAwarenessContext` fails with `ERROR_ACCESS_DENIED` if the
  process awareness was already set — by an AppCompat shim, by a host, by a
  loader — before the tool's own code ran. The tool must **discard the return
  value and read `GetThreadDpiAwarenessContext` back**, and abort if the
  readback is not what it needs. `capture-t10-controls.ps1` does this and
  prints the readback into every `measurements.txt`.

## 4. Proposed replacement for Observation 4's second bullet

Offered as a draft for T12, which owns the wording:

> - The capture tooling must be **per-monitor-DPI-aware, and must verify that
>   it is** rather than assume the call succeeded. From M4-Phase 1 onward the
>   Wasamo runtime declares Per-Monitor-Aware V2 itself
>   ([DD-M4-P1-001](../../process/milestone-4/phase-1/decisions/dd-m4-p1-001-dpi-awareness-declaration.md)),
>   so on a 125% box an 800 × 600 DIP window is a real 1000 × 750 physical
>   window with a 982 × 703 client — not a stretched bitmap. An **undeclared**
>   capture process is answered about that window in *virtualized* coordinates,
>   the real rectangle divided by the system scale, so it would read
>   800 × 600 and mis-crop by exactly the scale factor while looking entirely
>   consistent. `GetDpiForWindow` and `GetWindowDpiAwarenessContext` are not
>   virtualized, so a plausible DPI readback is not evidence that the
>   rectangles are real. `SetProcessDpiAwarenessContext` can also fail against
>   awareness established before the tool ran, so discard its result, read
>   `GetThreadDpiAwarenessContext` back, and abort on a mismatch.
> - When two captures are to be compared **across different process
>   awareness postures**, capture the **client** rectangle via
>   `ClientToScreen`, not the window rectangle: the non-client frame is sized
>   by DPI-indexed metrics (measured: 9 / 38 / 9 / 9 physical for an aware
>   window at 120 DPI against 10 / 39 / 10 / 10 for an unaware one), so equal
>   outer rectangles do not give equal client rectangles and the two images
>   are not even the same size.

## 5. Re-trigger criterion

Re-derive these numbers whenever the capture happens at a monitor scale other
than 125%, whenever a phase changes the window's non-client treatment (a
custom title bar is already a
[handoff](../handoff.md) item for M5), or whenever a host is added whose
window is not created through `window::create`. The invariant is that the
non-client metrics are DPI-indexed and independent of the scale factor the
runtime applies; the specific numbers are this machine's theme metrics.
