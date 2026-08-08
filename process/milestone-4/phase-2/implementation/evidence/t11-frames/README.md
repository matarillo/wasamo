# T11 touch-versus-mouse counter frames

Two full `-Capture` runs of
[capture-t11-touch-counter.ps1](../capture-t11-touch-counter.ps1)
(2026-08-09), one per input family, plus the `-Compare` output in
`meta.txt`. Both frames of every step are kept — the second of each pair
is what makes the within-set noise floor a measurement rather than an
assumption, and `-Compare` reads exactly the files committed here, so the
comparison can be re-run against this directory without re-capturing.

## What this set is evidence for, and what it is not

It is the **desktop-tier** half of T11's evidence: that a real OS touch
contact reaches the shipped runtime and activates the widget it lands on,
**once per contact**. The CI-gated half — that the window procedure
converts, resolves and dispatches a `WM_POINTER*` message correctly —
is `wasamo-runtime/tests/touch_pointer_integration.rs`, and it cannot
make this claim: a `SendMessageW`-borne pointer message carries no real
pointer id, so `DefWindowProcW` promotes nothing whether or not the
runtime claims the message. Removing the suppression left that whole
suite green, which is why this artifact exists.

It says nothing about a physical digitizer. Injection establishes the
message path, not the hardware — the same limit shape as M4-Phase 1's
synthesized `WM_DPICHANGED`.

## The control

The **same** script drives the **same** host through the **same** three
activations at the **same** client point, differing only in the input
family: `mouse` uses `SetCursorPos` + `mouse_event`, `touch` uses
`InitializeTouchInjection` + `InjectTouchInput`. Both runs park the
physical cursor at a fixed off-window screen point before every capture,
so hover state is out of both sets and the input family is the only
variable.

| Step | State |
|---|---|
| `step0` | Before any input. `Count: 0`, the Button in its accent fill |
| `step1` | After activation 1. `Count: 1`, and the Button now paints the focus indicator — a touch contact moves focus exactly as a click does, which is one of the two behaviours T11 decided |
| `step2` | After activation 2. `Count: 2` |
| `step3` | After activation 3. `Count: 3` |

- **The difference leg** is `step0` versus `step1` within a family: the
  count must change at all. Measured 6,628 differing px in both families.
  The script also throws if any activation fails to change its frame, so
  a tap that missed the Button is a red run, never a silently identical
  set.
- **The agreement leg is the actual claim**: `touch` step *N* must match
  `mouse` step *N*. Three touch contacts must produce the same rendered
  counts three mouse clicks do. **A contact delivered twice — the state
  the promotion suppression exists to prevent — would render `Count: 2`
  at touch step 1 against the mouse run's `Count: 1`**, and the
  comparison would fail on a whole digit rather than on a rounding
  difference.

## What the numbers mean

`meta.txt` reports three quantities per comparison, because two of them
are easy to confuse:

- **`max_channel`** — the largest per-channel delta anywhere in the
  frame. This is what the F-33 tolerance (13 per channel, measured, never
  bit-identity) is checked against, and what the verdicts use.
- **`px_differing_at_all`** — pixels that differ by any amount. This is
  the real noise floor.
- **`px_over_visible_change_threshold`** — pixels whose channel-sum
  differs by more than 60, i.e. *visible* change such as a digit
  appearing.

Measured here: the agreement legs are `max_channel` 0 or 1, with 0 px
over the visible-change threshold at every step. The mouse run's own two
frames of steps 1 and 2 differ by `max_channel` 1 over 4,638 px while the
touch run's are identical, so **the sets are not byte-identical and are
not expected to be** — the `±1` is capture noise on the anti-aliased
text, which is exactly the quantity F-33 says to measure rather than
assume away.

## Provenance

`meta.txt` records both run start times, and the SHA-256 and capture
timestamp of every retained file, so the artifact carries its own
evidence that two separate runs produced it. It also records the commit,
the display scale, the client rectangle, the cursor park point, and how
the Button centre was derived (scanned out of a frame by its accent fill,
never a hand-worked-out pixel constant).

## Environment requirement

An **interactive desktop**, with the target window visible, foreground
and unobstructed: injection is addressed to the desktop, not to a window,
so the contact goes to whatever window sits at the screen point. Both
scripts assert `WindowFromPoint` against their own window before
injecting and fail loudly rather than continuing. See
[verification-environments.md](../../../../../docs/notes/verification-environments.md)
Observation 6.

## Re-running

```
cargo build --release --workspace
powershell.exe -NoProfile -STA -File capture-t11-touch-counter.ps1 -Capture -Input mouse
powershell.exe -NoProfile -STA -File capture-t11-touch-counter.ps1 -Capture -Input touch
powershell.exe -NoProfile -STA -File capture-t11-touch-counter.ps1 -Compare
```

Windows PowerShell 5.1, not `pwsh` 7: the contact structure is built in
an `Add-Type` C# layer that the 5.1 host compiles without extra assembly
references.
