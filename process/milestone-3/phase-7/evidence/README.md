# T8 iteration evidence — capture invocations

Assistant-visible positive control for the `for`-generated gallery
thumbnail set (ADR evidence item 5). Capture mechanics: per-monitor-DPI
aware, title-enum `Gallery` HWND, `SetCursorPos` + `mouse_event`
window-relative clicks on the body-external Buttons, `CopyFromScreen`
over `GetWindowRect`. See [capture-iteration.ps1](./capture-iteration.ps1).

Build first: `cargo build -p gallery-rust --release`.

Both runs use a **1280×1316** window (wider than the script default so
the `for` set and the four Buttons share one frame, and the ≤ 7 appended
items stay on one fully-visible row). Window-relative Button centres at
this size: `Add (640,798)`, `Remove (640,856)`, `Clear (640,914)`,
`Reset (640,972)`. Each `;`-separated point is clicked then captured.

## Sequence A — append / remove (`t8-iteration-*.png`)

```
pwsh -NoProfile -File capture-iteration.ps1 `
  -OutputPrefix t8-iteration -Width 1280 -Height 1316 `
  -ClicksThenCapture "640,798;640,856" -Labels "init,add,remove"
```

`init` = 6 (`S01 #0`…`S06 #5`) → `add` = 7 (`+ NEW #6`, same row) →
`remove` = 6 (`NEW #6` gone). Held at ≤ 7 items so no item wraps below
the fold (the drop-last step is crisply legible).

## Sequence B — clear / reset (`t8-clearreset-*.png`)

```
pwsh -NoProfile -File capture-iteration.ps1 `
  -OutputPrefix t8-clearreset -Width 1280 -Height 1316 `
  -ClicksThenCapture "640,914;640,972" -Labels "init,clear,reset"
```

`init` = 6 → `clear` = 0 (empty `for` slot, member live) → `reset` = 6
(restored from the static literal).

> Button y-coordinates are layout-dependent: if `gallery.ui` changes the
> member order above the ScrollView, re-derive them from a recon frame
> (`-Labels recon` with no `-ClicksThenCapture`).
