# M3-Phase 5 — implementation handoff

Forward-carry material for the next phase's pre-doc framing, prepared for
the Phase 5 phase-end retrospective (retrospectives.md item 15 / §6.3);
the retro itself records the final phase-sync dispositions and flips the
plan checkboxes that gate phase close. The next phase is **M3-Phase 6
(ZStack + conditional rendering)**; one carry-forward constraint (DPI)
targets **M4**, recorded here as the engineering input its framing will
read.

`doc-folded` dispositions are not transcribed here — see the pointer
notes at the end. Only the `carry-forward` constraints, the out-of-phase
residuals, and the next-phase-relevant learnings are written out.

## Carry-forward constraints (confirmed handoff targets)

- **Per-monitor DPI awareness is unimplemented in the runtime — owned by
  M4.** Surfaced by the Phase 5 T6 owner smoke on a 125% high-DPI
  display, but the gap is **pre-existing** (the runtime has been
  DPI-unaware since M1) and **runtime-wide**, not Grid- or
  `gallery-rust`-specific. The engineering substance the M4 framing
  should read:
  - [`wasamo-runtime/src/window.rs`](../../../../wasamo-runtime/src/window.rs)
    `create_hwnd` declares no process/window DPI awareness (no
    `SetProcessDpiAwarenessContext` / app manifest), so on a high-DPI
    monitor DWM bitmap-scales the whole window (125% → a logical 800×600
    window rendered as physical 1000×750, uniformly blurred).
  - Layout consumes `GetClientRect` client pixels as logical units 1:1,
    with no DPI scale factor applied, and there is no `WM_DPICHANGED`
    handler. Grid (and every M3 primitive) computes correctly in logical
    pixels; DPI is an orthogonal runtime-quality axis.
  - The vision/roadmap decision is already landed (separate governance
    commit, not bundled into the Phase 5 Moment-2 sync): M4 acceptance
    criterion added per
    [DD-V-022 / DD-V-023](../../../cross-milestone/decisions/dpi-awareness-m4-deferral.md)
    in [`process/_roadmap.md`](../../../_roadmap.md), and the
    [`README.md`](../../../../README.md) vision-state note. This handoff
    entry is the **engineering** input the VDR explicitly defers to it;
    it does not restate the VDR's vision decision.
  - Adjacent assistant-tooling constraint (not the runtime fix): screenshot
    capture for assistant-visible GUI evidence must be
    per-monitor-DPI-aware — see
    [`docs/notes/verification-environments.md`](../../../../docs/notes/verification-environments.md)
    Observation 4.

## Out-of-phase residuals

- **R1 — Gallery host Window-title wiring → Phase 6.** Carried from the
  Phase 4 handoff (`MainWindowTitle = "Wasamo"` framework default while
  `examples/gallery/gallery.ui` declares `title: "Gallery"`). Phase 5
  did not touch the host window-metadata path, so R1 is unchanged. Its
  owning phase is **M3-Phase 6**, already recorded in the
  [`process/milestone-3/plan.md`](../../plan.md) Phase 6 row Notes
  (M3-Phase 5 FD-E: R1 lands alongside the first conditional-rendering /
  `bool`-driven property-update slice). Implementation deadline remains
  no later than M3-Phase 8 Gallery E2E close.

## Main learnings carried forward

- **A T0-frozen task list can carry stale ownership when a mid-phase
  owner decision moves an item.** The Phase 5 T7 list, frozen at T0,
  assigned the `phase-sync` close and the `handoff.md` clean-up to T7,
  but the later T6 owner decision (2026-05-30) plus
  [retrospectives.md §15 / §6.3](../../../procedures/retrospectives.md)
  reassign both to the phase-end retro. Resolved by **plan revise A**
  (re-tag both `[ ]`, NOT-owned-by-T7) rather than an ad-hoc
  reinterpretation. For the **final step of a multi-step phase**:
  cross-check the T0-frozen task list against mid-phase owner decisions
  before close, and revise the mutable phase plan rather than work
  around it. (Precedent: M3-Phase 4 "T5/T6 split for owner-manual GUI
  smoke".)

## Pointers (doc-folded — not transcribed)

- **Grid carrier-c1 textual IR grammar** (T1 emit / T3 loader) is folded
  into [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) §8.5
  (`track_decl`) plus the §5 / §2.2 / §3 author-surface re-sync
  (revision history v1.4). Next phase reads the spec directly.
- **T5/T6-origin assistant-visible evidence standard** (constraint #2 —
  T5 raised the bar, T6 pinned the DPI-aware capture mechanics) is folded
  into [`CLAUDE.md`](../../../../CLAUDE.md) §Testing rules (rule core) and
  [`docs/notes/verification-environments.md`](../../../../docs/notes/verification-environments.md)
  Observation 4 (capture mechanics / environment). M3-Phase 6 gallery
  visible work will exercise this rule.
- **Positive-control discipline for visible verification** (T6 owner-smoke
  learning) is folded into [`CLAUDE.md`](../../../../CLAUDE.md) §Testing
  rules and
  [`docs/notes/verification-environments.md`](../../../../docs/notes/verification-environments.md)
  Observation 4: a single static frame a wrong implementation could equally
  produce is not evidence — prove a star track's flexibility by resizing,
  prove a clip by what is *missing* against the source, prove conditional
  rendering by toggling state. Directly relevant to M3-Phase 6 conditional
  rendering (the initial frame alone cannot prove a `bool`-driven slice
  toggles). The Grid-specific observation steps stay local-only
  (`plan.md` observation list); only the general principle is folded.

## Phase-end disposition note

The DPI residual is recorded here and in its VDR; the Phase 5 Grid ADR
([`decisions/preamble.md`](../decisions/preamble.md)) is deliberately
**not** cross-referenced to it (phase-sync ADR-touch case 2 **not
fired**). DPI is orthogonal to Grid's design and already has a unique
owner (the VDR + roadmap M4 acceptance criterion + this handoff); a
cross-ref from the Grid ADR would add noise for its readers. The ADR set
stays at its Moment-1 Accepted state. (T7 deferred this determination to
phase-end — see [t7.md item 6](../retrospectives/t7.md).)
