# Phase 7 iteration evidence — capture invocations

Visible positive controls for the `for`-generated gallery thumbnail set.
The **T8** frames (`t8-*.png`) are the assistant-visible baseline (ADR
evidence item 5); the **T9** frames (`t9-owner-smoke-*.png`) are the
owner-manual human-visible smoke (ADR evidence item 6). The two are
separate gates — the assistant baseline does not replace the owner smoke.

## T8 — assistant-visible baseline (`t8-*.png`)

Capture mechanics: per-monitor-DPI aware, title-enum `Gallery` HWND,
`SetCursorPos` + `mouse_event` window-relative clicks on the body-external
Buttons, `CopyFromScreen` over `GetWindowRect`. See
[capture-iteration.ps1](./capture-iteration.ps1).

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

## T9 — owner-manual human-visible smoke (`t9-owner-smoke-*.png`)

A **manual** owner run of `target/release/gallery-rust.exe` (not the
scripted capture above): the owner maximised / resized the window so the
four body-external Buttons (`Add` / `Remove` / `Clear` / `Reset`) and the
`for`-generated set were both visible, then clicked the Buttons and
captured each step. Eight frames record the run; the assistant analysed
each frame against its claim ([log.md](../implementation/log.md) T9 end
gate). The smoke covers all four authored mutation forms plus WrapPanel
reflow and ScrollView behaviour around the generated set.

| Frame | Action | Observed |
|---|---|---|
| `t9-owner-smoke-1-init` | launch | 6 thumbnails `S01 #0`…`S06 #5` |
| `t9-owner-smoke-2-add-3times` | `Add` ×3 | 9 thumbnails; prefix unmoved + `NEW #6/7/8` |
| `t9-owner-smoke-3-remove-4times` | `Remove` ×4 | 5 thumbnails `S01 #0`…`S05 #4` (tail removed) |
| `t9-owner-smoke-4-clear` | `Clear` | 0 thumbnails; empty, no crash |
| `t9-owner-smoke-5-add` | `Add` after clear | 1 thumbnail `NEW #0` (member live) |
| `t9-owner-smoke-6-reset` | `Reset` | 6 thumbnails restored |
| `t9-owner-smoke-7-narrowing` | shrink window width | WrapPanel reflow (Photo 10→5+5; `for`-set 4/row) |
| `t9-owner-smoke-8-scrolldown` | `Scroll down (+100)` | `for`-set scrolls; `S05 #4` `S06 #5` reach the bottom |

Count trajectory 6 → 9 → 5 → 0 → 1 → 6 tracks the Button clicks (the
positive control). Because this is a manual run, there is no single
scripted invocation; reproduce it by enlarging the window per the T8
viewport-visibility note and clicking the four Buttons in the order above.
