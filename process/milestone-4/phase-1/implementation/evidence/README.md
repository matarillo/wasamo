# M4-Phase 1 evidence — which frame set is current

A pointer, not a record: the reasoning lives in
[../log.md](../log.md) and the procedure in [../plan.md](../plan.md).
This file exists because the directory now holds several six-frame sets
that look interchangeable and are not.

## The current reference set

**`t6-r1-final-a/` and `t6-r1-final-b/`** — the current T6 success-path
artifact. The two sets are byte-identical over all six client interiors and
are also byte-identical to their `t6-final/` predecessor. The later
`1220b10` change is confined to ABI ownership error handling and a Rustdoc
correction; it does not alter rendering code or replace these frames.

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
