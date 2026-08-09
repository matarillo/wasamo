# T10 item-identity frames

One representative frame per set from a single `-Capture` run of
[capture-t10-item-identity.ps1](../capture-t10-item-identity.ps1)
(2026-08-08). The script takes **two** frames per set; the second of each
pair is not kept here — its only job is to establish the within-set
jitter, which the run measured at **0 differing pixels** for every set.
The numeric comparison and the assistant's reading of these images are in
[log.md](../../log.md) §T10 close gate #7.

Capture conditions, from `t10-item-identity-meta.txt`: display scale
**1.25** (120 DPI), **client** rectangle 982x703 px = 785.6x562.4 DIP,
window at a fixed position and size for the whole run, real key presses
(`keybd_event`) with foreground activation acquired and read back.
Preceded by `cargo build --release --workspace`.

| File | What it is |
|---|---|
| `t10-item-identity-closed.png` | The before-state: the lightbox absent. Also the frame that shows the toolbar **not** overlapping at this client width — the overlap G7 measures at a 360 DIP client is width-driven |
| `t10-item-identity-a0.png` | Thumbnail 0 clicked. Caption `Photo #0`; the `<` Button paints the focus indicator and `>` does not, which is the scope entry `gallery_slice_integration.rs`'s G2 pins at the state level |
| `t10-item-identity-a3.png` | Thumbnail 3 clicked. Caption `Photo #3` — the difference leg |
| `t10-item-identity-k5.png` | Thumbnail 5 clicked. Caption `Photo #5` |
| `t10-item-identity-k6.png` | `ArrowRight` from the same open lightbox. Caption `Photo #6` |
| `t10-item-identity-k4.png` | `ArrowLeft` twice from there. Caption `Photo #4` |

The agreement legs are not represented by a file because they are
*absences of difference*: thumbnail 0 clicked a second time reproduced
`t10-item-identity-a0.png` at 0 differing pixels in the caption region,
the `[photo]` box agreed at 0 across every set, and the client after
`Escape` agreed with `t10-item-identity-closed.png` at 0 differing pixels
over the whole 982x703 rectangle.
