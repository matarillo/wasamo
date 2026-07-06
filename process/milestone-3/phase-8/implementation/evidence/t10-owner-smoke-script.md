# T10 — FD-8-G(5) owner human-visible smoke script

Owner-performed final smoke over the agreed Phase 8 state set. This is
the G(5) gate: the evidence is the owner's live observation and explicit
acceptance recorded in [../log.md](../log.md) — the saved T7 screenshots
are prep material only and do not substitute for this run.

This script **cites** the agreed state set; it does not redefine it. The
state-set definition lives in the plan T10 item and the ADR verification
closure; the surface agreement lives in the T2 G(1) / A1 table
([../log.md](../log.md)); the assistant-visible baseline is the T7
package ([README.md](./README.md)).

## Environment

Per [docs/notes/human-visible-smoke.md](../../../../../docs/notes/human-visible-smoke.md):

- A **visible Windows desktop session** is required (local machine, or an
  RDP/VNC session where you can see the screen). A plain SSH session is
  not valid evidence.
- Shell: `pwsh`, from the repo root.

## Build (rehearsed by T10 assistant prep; skip if binaries are current)

The three host binaries below were built and launch-rehearsed during T10
prep. Rebuild only if you want to reproduce from scratch. Build order per
AGENTS.md: the release workspace build must precede the C / Zig hosts.

```powershell
# 1. Release workspace (wasamoc.exe, wasamo.dll, gallery-rust.exe)
cargo build --release --workspace

# 2. C host (use the VS-bundled cmake if plain cmake is not on PATH)
$cmake = "C:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
& $cmake -S examples/gallery-c -B build/gallery-c
& $cmake --build build/gallery-c --config Release

# 3. Zig host (defaults point at target/release)
Push-Location .\examples\gallery-zig
zig build -p ..\..\build\gallery-zig -Doptimize=ReleaseSafe
Pop-Location
```

Make `wasamo.dll` resolvable for the C / Zig hosts before launching them:

```powershell
$env:PATH = "$PWD\target\release;$env:PATH"
```

## Known M3 placeholders and M4 residuals — observe, do not fail

These are agreed dispositions, not defects. Seeing them is expected:

- Lightbox `<` / `>` buttons are **inert placeholders** (index navigation
  needs M-expr2a element access; out of M3). Clicking them changes
  nothing — that is the agreed behavior.
- Thumbnails are `Box` + `Text` stand-ins; clicking a thumbnail does
  nothing (real images and hit-testing are M4).
- Scrolling is via the `Scroll down` / `Scroll up` buttons only; no
  scrollbar, mouse wheel, or drag (M4).
- The status strip text is static; no live collection count (M3).
- The initial window size at launch may be small; resizing by hand is the
  expected operation (Problem B disposition — explicit sizing is
  scheduled post-M3). Window title stays "Gallery"; dynamic title is M4.
- Minor DPI blur on high-DPI monitors is a known M4 residual.

If you observe something outside both the pass criteria and this list,
record it as a fail observation.

## Part 1 — Rust host (representative; full state set)

Launch:

```powershell
.\target\release\gallery-rust.exe
```

If the window opens small, drag-resize it to a comfortable wide size
(roughly 1200 x 760).

### 1. Default view

Observe:

- A window titled **Gallery** opens and stays open (no crash).
- Header row: **All / Albums / Favorites** tab buttons on the left;
  **Scroll down / Scroll up / Open lightbox** on the right.
- **All** shows the selected (highlighted) background; **Albums** and
  **Favorites** do not.
- Darker content area with square thumbnail placeholders labelled
  `IMG 001 #0` … onward, wrapping into multiple rows.
- Status strip at the bottom:
  `18 placeholders - Image and hit-testing are M4`.

**Pass:** all of the above. **Fail:** blank window, missing region,
clipped/unreadable header labels, or no selected tab.

### 2. Tab selection with exclusion (positive control)

This is the A10 two-frame positive control performed live: the point is
that the single selected highlight **moves** — a static look-alike
cannot do that.

- Click **Albums** → Albums gains the selected background **and All
  loses it** in the same moment. Exactly one tab is selected.
- Click **Favorites** → the highlight moves again; Albums clears.
- Click **All** → back to the initial state.

**Pass:** after every click, exactly one tab is selected, and it is the
one you clicked. **Fail:** two tabs selected at once, zero selected, the
highlight not moving, or the highlight indistinguishable from the
unselected background.

### 3. Lightbox open / close (conditional subtree present → absent)

- Click **Open lightbox** → a dimmed scrim covers the whole window; a
  centered 4:3 `[photo]` placeholder appears with a caption below,
  `<` / `>` at the photo sides, and **x** at the upper right.
- (Optional) Click `<` or `>` — nothing changes; that is the agreed
  inert placeholder.
- Click **x** → the overlay disappears completely; the gallery beneath
  is fully visible and interactive again.

**Pass:** overlay fully appears and fully disappears. **Fail:** overlay
does not open, does not close, or leaves visual remnants.

### 4. Wrap / overflow (reflow + scroll movement)

- Drag-resize the window narrower (to roughly two-thirds of the wide
  width). The thumbnail grid **reflows to fewer columns**; nothing
  overlaps or disappears.
- Click **Scroll down** once or twice → the thumbnail rows move up and
  later `IMG` numbers scroll into view. Click **Scroll up** → content
  returns.

**Pass:** reflow happens on resize, and the visible thumbnail range
changes with the scroll buttons. **Fail:** no reflow, clipped/overlapping
content, or the scroll buttons not moving the content.

### 5. Close

- Close the window with the title-bar **X**.

**Pass:** the process exits without a crash or error dialog.

## Part 2 — C host (launch + default view)

```powershell
.\build\gallery-c\Release\gallery-c.exe
```

Observe the same default view as Part 1 step 1 (All selected, header
controls, 18 placeholders, status strip), then close the window.

**Pass:** identical default render and clean close.

## Part 3 — Zig host (launch + default view)

```powershell
.\build\gallery-zig\bin\gallery-zig.exe
```

Same check as Part 2, then close.

**Pass:** identical default render and clean close.

## Recording the result

Reply in chat with the per-part result. On success an explicit
acceptance ("G(5) OK" or equivalent) is recorded in
[../log.md](../log.md) as the T10 gate evidence. A fail observation is
recorded in log.md with what you saw and in which step; fixes land
additively on the `feat/m3-phase-8-t10` branch and the affected steps
re-run to green (Phase 4 / 7b precedent).
