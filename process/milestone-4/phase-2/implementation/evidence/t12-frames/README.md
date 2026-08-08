# T12 — the four positive controls, taken at one sitting

Every frame here comes from **one** `-Capture` run of
[capture-t12-controls.ps1](../capture-t12-controls.ps1) (2026-08-09):
one `cargo build --release --workspace`, one launch of
`target\release\gallery-rust.exe`, one window geometry, one measured
scale. That is what the plan item's "captured at one sitting" asks for,
and it is why control A was re-taken here rather than cited from T10 —
the four controls are mutually comparable only if they share a window.

Both frames of every set are kept. The second of each pair is what makes
the noise floor a **measurement** rather than an assumption (F-33), and
`-Compare` reads exactly the files committed here, so the whole
comparison can be re-run against this directory without re-capturing.

Capture conditions, from `t12-controls-meta.txt`: commit **`fd5d192`**,
display scale **1.25** (120 DPI), **client** rectangle 982x703 px =
785.6x562.4 DIP, window fixed at (120,120) 1000x750 for the whole run,
**real key presses** (`keybd_event`) with foreground activation earned by
a click and read back. The meta file also carries the SHA-256 and the
derived click points, so the artifact records its own provenance.

The numeric legs and the assistant's reading of these images are in
[log.md](../../log.md) §T12 close gate.

## Why 1.25 and not 100%

A wrong pointer conversion is invisible at 100%
([preamble.md](../../preamble.md) §What "green" is worth). The plan item
asks for at least one control at a scale ≠ 100%; this monitor is 120 DPI,
so **all four** are. Every frame states its scale either way.

## The four controls

| Control | Difference leg | Agreement leg |
|---|---|---|
| **A** — click routing and item identity | `a0` vs `a3`: the caption reads `Photo #0` against `Photo #3` | `a0` vs `a0b`: the same thumbnail clicked twice; and the `[photo]` box, which does not depend on `selected_index` this phase, agrees across `a0`/`a3` so the difference is localised to the caption |
| **B** — traversal order | `b1`..`b4` vs `b-n`: four disjoint, left-to-right-increasing painted regions — `All`, `Scroll down`, `Scroll up`, `Open lightbox` | two frames with no input between them; plus `b5` ≡ `b1` (wrap), `b3b` ≡ `b3` (determinism), `brev` ≡ `b2` (Shift+Tab reverses) |
| **C** — containment and occlusion | `c-fired` vs `c-closed`: the same coordinate that did nothing under the open lightbox switches the tab once it is closed; `c-tab` vs `c-openB` over the lightbox's own columns: five Tabs visibly move focus `<` → `x` | `c-openA-click` ≡ `c-openA` and `c-blocked` ≡ `c-closed`: the covered click changed neither the picture nor the state; `c-tab` ≡ `c-openB` over the toolbar band: those same five Tabs never reached it |
| **D** — Esc | `d-closed` vs `d-open`: Escape closes the lightbox | `d-home` ≡ `d-open`: `Home` — a **recognised** key name with no handler on this scope — changes nothing; `d-closed` ≡ `d-pre`: it closed back to the state it opened from |

## Two things this set had to measure before it could claim anything

**The scrim divides the background's contrast by five.** The lightbox's
`fill: #101820cc` has alpha `cc` = 0.8, and the measurement matches:
the toolbar's checked-state swing is `max_channel` **157** unscrimmed and
**31** through the scrim — a 5.06x attenuation against the `1/(1-0.8)` = 5
the alpha predicts — while `px_differing_at_all` is **2608 either way**.
The scrim does not hide the toolbar; it dims it below the 60-summed
"visible change" bar the other legs use. So control C's two
lightbox-open toolbar-band legs are judged on `px_differing_at_all`
against a floor measured from those sets' own frame pairs. That is a
**tightening** — the agreement bar becomes "no pixel differs by any
amount" — and it is why the sensor leg exists at all: a no-change claim
about a region the instrument cannot see would have been unfalsifiable.

**A click on a Button moves focus, so "the click fired" cannot be read
off the button that was clicked.** After the `Albums` click that button
is both checked *and* focused, which is a third colour again. The leg
therefore lives on **`All`** — never clicked, never focused in this
sequence — whose face can only change because the handler wrote
`tab_all_selected = false`. The frames carry their own exclusion of the
one look-alike: `b1` shows what a checked **and focused** `All` looks
like (R=144 G=153 B=150), and `c-fired`'s `All` is R=67 G=67 B=67.

## Environment requirement

An **interactive desktop** with the window visible and foreground:
keyboard input is routed to the focused window of the foreground thread,
so the script earns activation with a real click and reads it back,
retrying rather than concluding from one refusal. See
[verification-environments.md](../../../../../docs/notes/verification-environments.md)
Observation 4.

## Re-running

```
cargo build --release --workspace
powershell.exe -NoProfile -STA -File capture-t12-controls.ps1 -Capture
powershell.exe -NoProfile -STA -File capture-t12-controls.ps1 -Compare
powershell.exe -NoProfile -STA -File capture-t12-controls.ps1 -SelfCheck
```

Windows PowerShell 5.1, not `pwsh` 7 — `System.Drawing`.

`-SelfCheck` is the artifact that makes the rest of them worth reading:
it feeds **every** verdict a deliberately wrong pairing drawn from these
same committed frames and requires each one to fail. A leg nobody has
seen go red is not a leg (M4-Phase 2 T11 retrospective, lesson c).

## This is the assistant baseline, not the owner's smoke

It observes pixels; it does not judge whether the app feels responsive.
The owner's half is
[owner-smoke/protocol.md](../owner-smoke/protocol.md)
([CLAUDE.md §Testing rules](../../../../../CLAUDE.md)).
