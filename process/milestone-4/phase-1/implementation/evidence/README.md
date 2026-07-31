# M4-Phase 1 evidence — which frame set is current

A pointer, not a record: the reasoning lives in
[../log.md](../log.md) and the procedure in [../plan.md](../plan.md).
This file exists because the directory now holds several six-frame sets
that look interchangeable and are not.

## The current reference set

**`t10-aware-a/`, `t10-aware-b/` and `t10-aware-restored/`** — the current
branch-tip artifact, and the first frames taken against the **landed**
awareness declaration. All three are byte-identical over the full 1200 × 720
client. They are **not** interchangeable with any earlier set: they are
*client-rectangle* captures at a controlled 1200 × 720, where every set below
is a *window-rectangle* capture at the size its own task chose. A comparison
across the two shapes is a size mismatch, not a regression.

**`t6-r1-final-a/` and `t6-r1-final-b/`** — the T6 success-path artifact and
the last reference before the declaration landed. The two sets are
byte-identical over all six client interiors and are also byte-identical to
their `t6-final/` predecessor. The later `1220b10` change is confined to ABI
ownership error handling and a Rustdoc correction; it does not alter rendering
code or replace these frames. **They were captured at effective 96 DPI in a
still-undeclared process**, so they are the record of T6's claim and not a
baseline for anything after T9.

`t5-review-after/` remains the branch-tip record at T5 close.
`t5-f23-after/` is byte-identical to it over the client interior and is kept
because it is the "after" half of the F-23 pair.

**`after/` (T3's) is stale for two frames.** It predates the F-23 fix, so
`labelupdate-clicked` and `labelupdate-clicked-twice` differ from the
current tree by 30,800 of 224,224 pixels each — the Grid-stretched Button
that used to vanish on any property write. The other four frames are
identical. A later task that reuses `after/` as "the last known-good set"
would report a regression that is not one.

**Do not reuse a committed set as a baseline. Re-capture** (finding F-33). The sets below are kept as the record of a claim
that was made, not as a substitute for capturing.

## What each set is the record of

| Set | Claim it evidences |
|---|---|
| `before/`, `after/` | T3 — the Button-family label writes moved into the sync pass with no rendered change |
| `mutations/` | T3 — four deliberately wrong implementations, each shown to change the frame |
| `t4-probe/` | T4 — three window states measured in one session (unaware+correction, aware without, aware with) |
| `t5-baseline/` | T5 — the pre-change tree, captured in the same session as the comparison |
| `t5-baseline-run1/` | T5 — the session's **first** capture, kept because it is the outlier F-33 measures (149 px against the two later captures) |
| `t5-after/` | T5 — the conversion seams: byte-identical to `after/`, which is the regression claim |
| `t5-f23-after/` | T5 — the F-23 layout-entry fix: two post-click frames change, four do not |
| `t5-review-after/` | T5 — the branch tip; identical to `t5-f23-after/`, which is what shows the R-2 correction is behaviour-preserving at scale 1 |
| `t5-probe/` | T5 — the positive control at 125% under a throwaway declaration: 7 tiles with the inbound seam, 9 without, and the outbound half with the node cache seeded |
| `t6-baseline-*`, `t6-after-*` | T6 — three fresh captures on each side of the 125% rendering change; repeatability, comparison numbers and screenshot analysis are in [t6-analysis/README.md](./t6-analysis/README.md) |
| `t6-100-baseline-*`, `t6-100-after-*` | T6 — identity-path control showing that `ceil` allocation plus DD-M4-P1-006 is intentionally observable even though D2D DPI and atlas-origin conversion are identities |
| `t6-brush-default/` | T6 mutation control — the implementation with only DD-M4-P1-006's three setters removed |
| `t6-final/` | T6 — six branch-tip frames captured after a forced clean rebuild of the accepted source; client interiors are byte-identical to `t6-100-after-b/` |
| `t6-scaled-surface-identity-a/b/` | T6 review mutation — effective 120-DPI geometry/cache with every text surface and D2D context forced to 96 DPI; the direct positive control for the scaled rasterization path |
| `t6-r1-final-a/b/` | T6 review remediation — two live six-frame sets from `fad59e2`; mutually byte-identical and byte-identical to `t6-final/`, with the render-neutral R1 distinction fired by the mock-free integration control |
| `t10-shipped-created/` | T10 — the gallery's full signature at the **landed** declaration, window never moved or resized: outer 1000 × 750, client 982 × 703, **7** tiles per row |
| `t10-unaware-created/` | T10 — the unaware half of that signature, same executable under `__COMPAT_LAYER=DPIUNAWARE`: outer 1000 × 750, client 980 × 701, **7** tiles per row. So the tile count no longer separates the two postures at this size |
| `t10-aware-a/b/` | T10 — control A / B, `s = 1.25` side at a controlled 1200 × 720 physical client (960 × 576 DIP) |
| `t10-unaware-a/b/` | T10 — control A2 / B, `s = 1` side, **the same executable** under `__COMPAT_LAYER=DPIUNAWARE`; also the first rendered evidence of DD-M4-P1-001's tolerated-declaration-failure path |
| `t10-base-a/b/` | T10 — control A1's "before": the phase base `80d79c4`, built in a separate worktree under its own `CARGO_TARGET_DIR` |
| `t10-mutation-inbound/` | T10 — the run that shows control B can fail: T5's inbound seam removed at all three sites, giving 11 clipped tiles per row and a toolbar pushed off the window |
| `t10-aware-restored/` | T10 — F-40 restoration after the mutation: package clean plus accepted-source rebuild, byte-identical to `t10-aware-a/` |
| `t10-delivery-check/` | T10 — the **staged** T11 runnable set (`gallery-zig.exe` + `wasamo.dll`) launched from its delivery directory, declaring PMv2 and rendering 7 tiles per row |
| `t10-control-c/` | T10 — control C's path form: three legs across an **owner-driven** 125% -> 150% -> 125% display-scale change, with the window untouched after the first leg so the rectangle is the one the OS supplied |
| `t10-analysis/` | T10 — the comparison numbers, the magnified crispness crops and the assistant's reading of them ([README](./t10-analysis/README.md)) |

**Frame shape is part of a set's identity**, and the table does not repeat it
per row: every set above `t10-*` is a six-frame *window-rectangle* capture,
and every `t10-*` set is a single *client-rectangle* capture. That is why
`compare-frames.ps1` takes zero insets for the T10 sets and its defaults for
the others, and why a T10 set and a T6 set cannot be compared at all — the
result would be a size mismatch, not a regression.

## Scripts

- `capture-t3-label-writes.ps1 -Tag <name>` — the six-frame set. Requires
  a `cargo build --release --workspace` build (T3 finding F-21).
- `capture-t4-probe.ps1 -Tag <name>` — one frame plus the window and
  client rectangles, never moving or resizing the window.
- `compare-frames.ps1 -Left <dir> -Right <dir>` — pixel comparison over
  the client interior, and a check that the two file *sets* match.
  **Exits non-zero on any difference**, so it can be used as a gate rather
  than only read. It also reports the **max per-channel delta**, which is
  asymmetric evidence: a large one proves only that the difference is
  outside the drift bound this phase measured, a small one proves nothing.
  Neither says what moved — an intensity-only rasterization defect can
  land on either side. `-AllowDrift` opts into passing a
  small-delta difference — a judgement to record, never a default.
  **Its default insets are tuned to the 96-DPI non-client frame**, whose top
  inset is 31 and side inset 8. Measured at T10, an aware window's frame is
  9 / 38 / 9 / 9 physical against the unaware 10 / 39 / 10 / 10, so the
  defaults still clear it — top margin 13 at 96 DPI against 6 aware and 5
  unaware here, side margin 4 against 3 and 2. Pass zero insets for
  client-rectangle captures, and re-derive them above 125%.
- `capture-t10-controls.ps1 -Tag <name> [-Exe <path>] [-Unaware]
  [-ClientW <n> -ClientH <n>]` — one launch, one **client-rectangle** capture,
  and a `measurements.txt` recording the harness's own DPI-awareness readback,
  an occlusion check over four interior client points,
  the window's awareness and DPI, both rectangles, the non-client frame and the
  extent the layout engine received. Aborts if the harness readback is not
  PMv2. `-ClientW/-ClientH` drives the *client* to an exact physical size by
  measure-and-adjust, which is what lets an aware and an unaware run be
  compared without the non-client frame in the way.
- `capture-t10-control-c.ps1 [-Tag <name>] [-WaitSeconds <n>]` — control C's
  path form. Positions the window, captures a before frame, then polls
  `GetDpiForWindow` while **a human** changes Settings > System > Display >
  Scale, capturing on the change and again on the way back. It never takes the
  keyboard or the foreground, and it does not touch the window after the first
  frame — on this path the OS chooses the rectangle, so resizing it would
  destroy what is being measured.
- `magnify-crop.ps1 -In <png> -Out <png> -X -Y -W -H [-Factor 5]` —
  nearest-neighbour magnification for the glyph-shape judgement, with the
  interpolation mode pinned so the script cannot make either side of a pair
  look sharper than it was captured.
