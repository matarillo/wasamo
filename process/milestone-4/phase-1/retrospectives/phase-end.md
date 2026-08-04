---
phase: M4-Phase 1
title: Phase-end retrospective
date: 2026-08-04
status: recorded
---

# M4-Phase 1 — Phase-end retrospective

This retrospective closes the phase-end batch after T1–T12 were merged
no-ff. It does not redo T12 or replace any task retrospective. The phase
branch remains unmerged and unpushed because the phase→main approval gate is
owner-controlled and the required CI gate is not green.

## Phase-end checklist (items 12–18)

### 12. Acceptance criteria

AC7 is implemented and its four accepted decisions are reflected by the
landed runtime, host probes, integration evidence, and Moment 1/Moment 2
documentation sync. The phase's acceptance claim is bounded by the evidence
limits recorded in the task retrospectives and in the final handoff. No
`Proposed` phase ADR remains open; the phase-end CI failure is a merge-gate
residual, not silently accepted as green evidence.

### 13. CHANGELOG and roadmap consistency

The M4 milestone is not complete, so no `CHANGELOG.md` release entry is due.
The M4 plan's Phase 1 progress row now records the phase-end state and links
the implementation preamble; `_roadmap.md` remains the acceptance-criteria
SSOT and needs no wording change. No milestone-level status was advanced.

### 14. Vision/thesis impact

There is no thesis or product-vision change: this phase discharges the
existing M4 AC7 and the M3 runtime-DPI residual. The process decisions made
at close are separately recorded: workflow status vocabulary now distinguishes
annotation from supersession, and DD-V-029 adds a narrow pure-logic red-test
artifact. Neither changes the runtime contract or the milestone thesis.

### 15. Framing, handoff, and T1–T12 item-10 classification

The T0 framing agreements remain intact. The implementation handoff is now
`status: recorded`; its durable table is the single carry-forward ledger.
Every task retrospective item 10 was audited as follows:

| Task | Classification | Phase-end disposition |
|---|---|---|
| T1 | carry-forward | Per-node cache ownership and the cold test-only Cargo prerequisite remain in the handoff with re-trigger criteria. |
| T2 | carry-forward | Explicit rounding-operation and scale-dependent-measure obligations remain in the handoff; its seven-mutation evidence is retained. |
| T3 | carry-forward | Single geometry-write pass and the two `label_size` writers remain handoff invariants. |
| T4 | carry-forward | Per-window scale, creation ordering/flags, and outer-vs-client semantics remain handoff invariants. |
| T5 | carry-forward + doc-folded | Hit-test preconditions, conversion cancellation, and DIP callback units remain carry-forward; the asymmetric frame-comparison rule is folded into Observation 4, the compare procedure, and the handoff evidence residue. |
| T6 | local-only + doc-folded | The target-isolation implementation detail is local-only because T6 closed it; the source-tree/rebuild discipline is folded into the phase evidence procedure and remains referenced by the handoff. |
| T7 | carry-forward | The no-`&mut WindowState`/step-4 ordering, `resize_fn` boundary, and diagnostic-function boundary remain handoff invariants. DD-M4-P1-003's step-ordering record is confirmed at T7; phase-end does not alter it. |
| T8 | carry-forward | Physical-client divisibility, assertion sensitivity, the test seam, non-reproducible debug artifacts, and stale-receiver limits remain handoff residue. |
| T9 | carry-forward | Unaware-coordinate virtualization, entry clearing, the `0x80070005` collision, declaration placement, and before-state fixtures remain handoff rows. |
| T10 | carry-forward | Shipped DPIUNAWARE posture, client-rectangle capture, and Rust-host absolute-path delivery limits remain handoff rows. |
| T11 | carry-forward | Stale projection, width-driven toolbar overlap, positive-control agreement, list readback scope, and human-procedure deviation remain handoff rows. |
| T12 | carry-forward | F-54 remains an unscheduled diagnostic/build-graph item with its precise re-trigger criterion. |

The only local-only item is T6's already-closed target-directory implementation
mechanic. The doc-folded items are not duplicated as new claims; their owning
procedure or evidence artifact is cited. All other item-10 candidates have a
destination and re-trigger criterion in `implementation/handoff.md`.

### 16. Actual GitHub Actions CI

The required workflow was dispatched against the verified phase HEAD
`544574c366d6f4fded4a4232a3bda0710e3fa9d9`:

| Run | Result | Observation |
|---|---|---|
| [30873359437](https://github.com/matarillo/wasamo/actions/runs/30873359437) | failure | Release and debug workspace builds succeeded; `dip_layout_is_invariant_while_every_visual_moves_by_the_ratio` failed in `dpi_scale_matrix_integration`. |
| [30873615639](https://github.com/matarillo/wasamo/actions/runs/30873615639) | failure | Same HEAD and same test failed again; release/debug builds again succeeded. |

The exact integration test passes when run alone and in the full local
`cargo test --workspace`, but the two same-HEAD CI failures are a recurring
failure under the implementation-gates #6 rule, not a flake to roll away.
The runner-only failure has no assertion detail in the workflow log and is
carried forward for root-cause work. The phase→main merge gate therefore
remains open. No push or merge was performed.

### 17. Human-visible GUI smoke

T11's owner smoke is the phase's human-visible evidence: aware/unaware pairs,
cross-monitor drag and return, click/hover interaction, and the 100% agreement
leg were performed and recorded in `evidence/t11-owner-smoke/`. The narrowed
window deviation and the toolbar overlap it exposed are scoped in the
handoff. This phase-end batch did not create a new GUI claim or substitute
process survival for the screenshot/positive-control requirement.

### 18. CI YAML sanity

`.github/workflows/ci.yml` was audited and unchanged. Its release workspace
build → debug workspace build → workspace test ordering is the required cold
artifact order, and the release/debug build portions passed in both runs. No
new language or build system was introduced by M4-Phase 1, so no CI update was
required by project policy.

## Phase-end implementation-gate record

The phase-end start gate was recorded in `implementation/log.md` before the
close approach:

| Trap | Selection | Reason / close artifact |
|---|---|---|
| #1 semantic migration | no | No enum, IR, or schema migration in this batch. |
| #2 side effects | no runtime; doc analogue covered by #3 | No runtime/tree mutation; document status and cross-reference effects were enumerated. |
| #3 parallel/derived data | yes | Plan, roadmap, handoff, retro, workflow, and VDR claim sites were audited; no duplicate CHANGELOG claim was added. |
| #4 authored branches | no | No executable branch was added; DD-V-029 changes only the future close artifact. |
| #5 carry-forward | yes | All item-10 candidates were classified and the recorded handoff table has destinations and triggers. |
| #6 deterministic failures | yes | CI failure was rerun once on the same HEAD, reproduced, and carried forward with local-vs-CI evidence. |
| #7 GUI evidence | no new evidence | The batch cites T11's existing owner smoke and adds no render claim. |

Review lane: Normal documentation/process review, with the process-rule
change recorded as DD-V-029 and its `implementation-gates.md` SSOT edit in the
same batch. Runtime/schema/IR and new GUI-render review lanes were not
triggered. The historical T7 post-remediation independent-review limitation
is recorded rather than retroactively relabelled.

## Phase-End Gate

| Required artifact | Result |
|---|---|
| Acceptance and ADR closure | AC7 evidence and all phase ADRs audited; CI residual explicit |
| T1–T12 item-10 classification | Complete; table above and handoff ledger agree |
| Handoff finalization | `implementation/handoff.md` → `recorded` |
| Implementation preamble | `active` → `closing` |
| Workflow vocabulary decision | Annotation vs supersession rule recorded |
| T7 safety-net check | DD-M4-P1-003 step-ordering record confirmed at T7 |
| Pure-logic red-test VDR | DD-V-029 accepted; gate SSOT updated in the same batch |
| Local verification | `cargo fmt --all -- --check`, `git diff --check`, targeted integration test, and full local workspace test passed before this doc batch |
| Actual CI | Runs 30873359437 and 30873615639 both failed the same test; phase→main gate remains open |

The phase-end documents are recorded, but the phase is not represented as
merge-ready. Owner approval is still required for any phase→main merge and
push, after the CI residual is resolved or explicitly dispositioned by the
owner.
