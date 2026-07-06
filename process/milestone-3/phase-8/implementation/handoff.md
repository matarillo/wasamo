---
title: M3-Phase 8 handoff
status: final
source-phase: M3-Phase 8
---

# M3-Phase 8 — Handoff

> **Status: finalized at phase close (2026-07-06).** Per
> [workflow.md §6.3](../../../procedures/workflow.md) and
> [retrospectives.md](../../../procedures/retrospectives.md) item 15,
> this file is the confirmed phase-close deliverable, distilled from the
> T11 candidate ledger in [log.md](./log.md). Phase 8 is the **final M3
> phase**: the milestone-level residuals (the five DD-001 deferred axes,
> the PM-2 wrapper rule, the Problem B / author-controllable sizing
> disposition, default alignment, placement spelling + bindability, the
> M4 residual cluster, and the public-facing phase-vocabulary cleanup)
> are owned by the T9 owner-reviewed
> [`process/milestone-3/handoff.md`](../../handoff.md) draft (finalized
> at milestone close) and are **pointed to, not duplicated** here. This
> file carries only the phase-level engineering constraints a successor
> implementation phase (M4+) needs beyond that.

## Main Learnings

- **New widget kinds need positive proof at both catalog boundaries.**
  Because the IR widget-kind carrier is a string and unknown widgets are
  warning-only at `wasamoc check`, compiler errors do not enumerate the
  landing surface: T3 had to prove `ToggleButton` is *known* with a
  no-unknown-warning fixture, and T4 had to mirror the compiler catalog
  at the runtime defensive-reader boundary instead of inheriting
  Button's older loose path. Both constraints are carried below.
- **Owner-gate evidence is bound to surface state, and the binding must
  be checked, not assumed.** The G(5) owner smoke was bound to the
  surface unchanged since the T7 capture commit (`5b66321`); T10, T11,
  and the phase-end batch each re-verified the path-scoped product
  range empty before reusing prior evidence. The same coupling applies
  to operational scripts (`t10-owner-smoke-script.md`, capture scripts)
  that weave current-surface facts into their steps.
- **Coordinate-based screenshot capture is a fragile, surface-coupled
  evidence path, not a portable test.** T2 and T5 both reproduced
  sandboxed off-screen `CopyFromScreen` failure; T5 additionally missed
  a close-button coordinate until a state-confirming frame was added.
  T7 succeeded by planning the capture as a visible-desktop,
  outside-sandbox activity on the final surface.
- **Document-sync tasks have their own trap catalog.** The T9 pre-merge
  review surfaced parallel-doc drift (a second source of truth in
  CHANGELOG / ledger prose) as the document analogue of trap #3, and the
  post-retrospective-remediation gap (recorded verification left behind
  the final branch state, recurred T4–T9). Both are folded into the
  procedure SSOTs at this phase close (see Dispositions below).

## Carry-forward constraints (confirmed at phase-end item 15)

1. **No-unknown-warning positive fixture for new widget kinds.** While
   the widget kind remains string-carried and unknown widgets remain
   warning-only (exit 0) at `wasamoc check`, a task adding a widget kind
   must prove known-widget admission with a positive fixture asserting
   the absence of the unknown-widget warning — check success alone is
   not evidence. *Evidence:* T1 wrong-kind probe;
   `togglebutton_known_widget_and_attrs_accepted_without_warning` (T3);
   preserved `unknown_widget_type_is_warning_not_error`. *Re-trigger:*
   any new widget kind; re-examine if the diagnostic policy is
   intentionally hardened to reject unknown widgets.
2. **Runtime defensive-reader catalog mirroring.** A new widget kind
   mirrors its compiler admission catalog at the runtime loader boundary
   (closed attribute/binding surface with named re-rejects), rather than
   inheriting Button's older loose direct-IR path. *Evidence:* T4
   `validate_phase8_togglebutton_node_invariants` + loader reject
   matrix. *Re-trigger:* any new widget kind or direct textual-IR
   catalog change; supersede if a broader runtime catalog policy lands.
3. **Coordinate-based `CopyFromScreen` capture discipline.** Treat such
   scripts as visible-desktop, final-surface, surface-coupled evidence
   paths: plan capture outside the sandbox, re-derive coordinates per
   surface (never inherit them as ground truth), and include a
   state-confirming frame after every state-changing click (modal close,
   tab select) before capturing dependent frames. *Evidence:* T2/T5
   sandboxed capture failures; T5 close-coordinate miss and the added
   `closed-after-lightbox` frame; T7 outside-sandbox capture at `(0,0)`.
   *Re-trigger:* any GUI-evidence task reusing or adapting a capture
   script, or any layout-affecting UI change.
4. **Declarative C / Zig host boundary.** The Gallery (and counter)
   C / Zig hosts stay declarative: they embed the compiled `.uic` and
   call `wasamo_load_ui(WASAMO_LOAD_MEMORY, …)` with no host-side widget
   mutation; changing that boundary is a task/decision of its own, not a
   host-port detail. Build ordering (release `wasamoc.exe` /
   `wasamo.dll.lib` before the C / Zig host builds) is already normative
   in [AGENTS.md §Build ordering](../../../../AGENTS.md) and enforced by
   the hosts' configure-time checks — pointer only. *Evidence:*
   `examples/gallery-c/main.c`, `examples/gallery-zig/main.zig` (T6).
   *Re-trigger:* any edit to the C / Zig hosts or their build scripts.
5. **Append-only revision history in normative specs.** Corrections to
   `docs/dsl_spec.md` / `docs/architecture.md` / `docs/abi_spec.md`
   revision-history tables land as new dated rows; prior rows are never
   retroactively rewritten. *Evidence:* T9 pre-merge review finding;
   applied at T11 (row 1.15 appended, prior rows untouched, verified by
   the T11 pre-merge review). *Re-trigger:* any future normative-spec
   correction; promote to a doc-system process note if it recurs as a
   review finding.
6. **Owner-gate prep re-verifies the evidence↔surface binding.** Before
   reusing prior frames, scripts, or acceptance records as prep material
   for an owner gate, run the path-scoped git range check against the
   commit the evidence was captured at; surface-coupled operational
   documents (owner smoke scripts) need a surface-unchanged check or a
   script revision first. *Evidence:* T10 prep git check;
   `evidence/t10-owner-smoke-script.md`. *Re-trigger:* any owner gate
   that reuses earlier evidence or operational scripts.

## Dispositions closed outside this file

- **Parallel-doc drift as the document analogue of trap #3** (T9
  pre-merge review; applied by T10/T11 and this batch) — **doc-folded**:
  an example added to the
  [implementation-gates.md](../../../procedures/implementation-gates.md)
  trap catalog at this phase close (minor edit).
- **Final-branch-state verification after post-retrospective
  remediation** (recurred T4–T9) — **doc-folded**: a note added to
  [retrospectives.md](../../../procedures/retrospectives.md) item 3 at
  this phase close (minor edit).
- **G(5) / T7-surface binding through the phase-end batch** —
  **local-only** beyond this phase: discharged by the batch's
  path-scoped range check and mooted by the phase → main merge; the
  durable general principle is carry-forward item 6 above.
- **Milestone-level residuals** — owned by
  [`process/milestone-3/handoff.md`](../../handoff.md) (T9 draft,
  finalized at milestone close); not restated here.
