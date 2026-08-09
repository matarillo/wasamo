---
phase: M4-Phase 2
title: Phase-end retrospective
date: 2026-08-09
status: recorded
scope: phase-end
merge_target: main
---

# M4-Phase 2 — Phase-end retrospective

This is the phase retrospective for the phase-to-`main` close gate. T13
does not write a task retrospective: its merge target is `main`, and its
work is the phase close itself. The branch was pushed only after explicit
owner authorization and its phase-branch CI is green. The phase-to-`main`
merge remains a separate owner gate.

## Phase-end checklist (items 12–18)

### 12. Acceptance and verification closure

M4 AC1 is implemented for this phase's consumer and evidence scope. The
two later-consumer rule halves are fixed without claiming their later
implementations:

| Acceptance obligation | Verification closure |
|---|---|
| Mouse input, one-target resolution and target-to-ancestor routing | Pure hit-resolution tests and `event_routing_integration`; T10's gallery slice; T12 controls A / C. Disabled occlusion and consume-on-handle are re-synced in `dsl_spec.md` §4.8 / §4.19 and `architecture.md` §13.1 / §13.2. |
| Keyboard input and per-window focus | `focus_core` unit tests, focus/key integration suites and gallery slice G1–G6 / G10; T12 controls B / D. Tab order, group arrows, unconsumed-key fallthrough and focus repaint are re-synced in §4.19 / §13.2–§13.3. |
| Touch input | T11's direct message-level integration plus interactive synthesized-touch capture establish screen→client→DIP→target→focus→single dispatch, with mouse-promotion suppression. The stated limit remains: this is not evidence from a physical digitizer. |
| Generic click and per-item handlers inside repetition | T3 generic routing and T9 per-item integration cover invocation-time item/index reads, positional identity and handler lifetime; T12 control A carries rendered item identity. |
| Structure-independent modal focus scope, exercised by a root `ZStack` branch | T6 admission/projection, T7 entry/exit/restoration and T10 gallery G3/G6/G9; T12 controls C / D and the owner's ten-step smoke. The subtree's presence is entry and pointer blocking comes from an authored covering widget. |
| AC10 / AC5 rule halves | DD-M4-P2-004 and the synchronized §4.19 / §13 bind top-layer and screen-reader modality to the same focus-scope concept. Phase 9 and Phase 11 still own those consumers; this phase does not claim their implementation. |
| M3 residual: thumbnail hit-testing | Closed by generic click plus T9/T10 item routing and T12 control A. |
| M3 residual: lightbox focus and input containment | Closed for the `ZStack` consumer by T7/T10/T12. Top-layer and accessibility consumers remain the planned later implementations, not residual special cases. |
| Phase-end criterion 4: specification synchronization | `dsl_spec.md` 1.21 and `architecture.md` §12.3 / §13 are implementation-synced. DD-M4-P2-001 has the owner-approved dated qualification; no decision is superseded. `abi_spec.md` is unchanged because the C ABI is unchanged. |
| Phase-end criterion 5: consumer slice | The gallery's `.ui` → IR → runtime lightbox slice runs through C, Rust and Zig hosts in T10/T12 evidence; the criterion requires at least one. |

All five phase DDs remain Accepted. No `Proposed` DD remains, and the
only post-acceptance change is DD-001's explanatory annotation. The
stretch re-evaluation was already discharged at the Accepted flip: both
stretch intakes remain in their planned later phases.

### 13. CHANGELOG, roadmap and progress consistency

M4 is not complete, so no milestone `CHANGELOG.md` entry is due.
`_roadmap.md` remains the AC SSOT and needs no wording change: AC1's
milestone row is broader than a per-phase progress marker and M4 has
twelve later phases. The M4 plan's Phase 2 progress row is the correct
place to record phase close and link these artifacts. No milestone-level
status is advanced.

### 14. Vision and thesis impact

The milestone thesis survives: input, later text/IME, multiple windows
and accessibility can consume one focus model. No `VISION.md` edit or
new vision decision is warranted. The one explanation that failed —
DD-001's claim that regeneration waits for handler return — narrowed by
dated annotation; it did not change the routing decision or runtime.

The phase's main learning is evidentiary as well as architectural:
mechanism tests, a real `.ui`/IR/runtime consumer and rendered positive
controls answer different failure classes. A green mechanism fixture
cannot establish painted focus, while a plausible frame cannot establish
the traversal rule unless its sequence excludes the look-alike. T12's
independent review found both forms in practice, and that learning is
carried without silently creating a new process rule.

### 15. T1–T12 carry-forward classification

Every distinct CF identifier in `implementation/log.md` is classified
below. Restated rows such as CF-T6-1 / CF-T6-3 are one identifier, not a
second obligation.

| Source IDs | Phase-end classification |
|---|---|
| CF-1, CF-2, CF-3 | **closed** — T8 made childless admission shared at both gates, restored literal `Button.enabled`, and completed generic signal admission; T13 synchronized the public text. |
| CF-4, CF-5, CF-6 | **carry-forward** — subtree-entry boundary loss, self-destroying native closures and the registry ABA window remain bounded hazards with explicit triggers. |
| CF-T4-1, CF-T4-2, CF-T4-3 | **carry-forward** — index-based hover identity, the redundant-state guard and live root replacement remain unclosed. |
| CF-T4-4, CF-T4-5 | **closed / doc-folded** — touch deliberately writes no hover/pressed state and §13.2 now says so; the preamble's false occlusion prediction was corrected and T12 supplied the phase-level render control. |
| CF-T5-1, CF-T5-2 | **closed by T7/T13** — focus anchors replaced in-range path identity, and scope restoration versus post-mutation structural succession is now both implemented and worded accurately. The allocator-address residual continues separately as CF-T7-1. |
| CF-T5-3, CF-T5-4, CF-T5-5, CF-T5-6 | **closed / doc-folded** — T12 observes the indicator; foreground activation is in Observation 4; T8 pins authored-key ordering around the host slot; T13 corrected §13.3. |
| CF-T6-1, CF-T6-3, CF-T6-5 | **closed** — T7 made presence-entry reachable and fixed group click landing; T8 removed the per-kind signal-gate asymmetry. |
| CF-T6-2, CF-T6-4 | **carry-forward** — combined group/scope semantics remain a candidate-pool question, and the cross-crate focus-container lists remain deliberately duplicated with a new-container trigger. |
| CF-T7-1, CF-T9-1 | **repaired at T13a / identity remainder carried forward** — T13's first cold suite and one exact rerun observed address reuse and exposed a retained-record / false-presentation divergence. Owner-authorized T13a reconciles the final retained target through the existing focus writer; an allocator-independent fixture is deterministically red without that call, the full independent review is complete and the new cold suite is green. Pointer-address ABA can still select an unexpected fresh same-address target, so that identity-policy remainder stays in the handoff rather than being claimed closed. |
| CF-T7-2, CF-T7-3 | **carry-forward** — direct ABI mutations still bypass the focus seam; nested-scope multi-entry integration belongs to M4-Phase 9. |
| CF-T7-4, CF-T7-5 | **closed by phase sync** — §4.19 now fixes arrow directions and outside-scope click focus. |
| CF-T8-1 | **candidate-pool carry-forward** — Button keyboard activation is intentionally decided with M5's keyboard-operable widget family. No implementation intent is implied. |
| CF-T8-2, CF-T8-3 | **closed** — all four layout-childless kinds are shared and documented; both handler grammars and §4.5 are synchronized. |
| CF-T8-4, CF-T8-6 | **carry-forward** — unknown-signal diagnostics remain unspecified with a fourth-signal/bug-report trigger; key-down host-listener instantiation belongs to the ABI phase or first host consumer. |
| CF-T8-5 | **doc-folded invariant** — the upward-only key walk and placement of gallery scope handlers are documented beside the implementation and exercised by T10. |
| CF-T9-2 | **carry-forward split** — M4-Phase 5 owns string assignment capability; M4-Phase 3 pre-doc owns the missing diagnostic intake. The current normative statement is deliberately not weakened. |
| CF-T9-3 | **closed by phase sync** — the two false diagnostics rows and the contradictory body bullet are removed. |
| CF-T9-4, CF-T9-5 | **carry-forward** — invocation-time resolution has one discriminating fixture; per-item modal-scope composition belongs to M4-Phase 9. |
| CF-T10-1, CF-T10-3 | **carry-forward to named owners** — M4-Phase 4 owns toolbar overflow semantics; M4-Phase 3 must define out-of-range reads and handler predicates per the owner's recorded expectation. |
| CF-T10-2, CF-T10-6 | **closed** — the gallery now consumes `focus-group`, and G9 exercises the authored `x` close route. |
| CF-T10-4, CF-T10-5 | **doc-folded** — gallery fixture coupling and the DIP unit of `__resolve_topmost_for_test` remain beside their call sites. |
| CF-T11-1 | **stated-limit carry-forward** — synthesized touch is not physical-digitizer evidence. |
| CF-T11-2, CF-T11-3, CF-T11-4, CF-T11-6 | **carry-forward** — pointer capture/drag, multi-contact meaning, untyped coordinate-space callers and the optional CI injection tier retain their explicit triggers. |
| CF-T11-5 | **doc-folded reusable evidence constraint** — capture scripts poll to a deadline and report owned windows; a later script must not reintroduce a fixed wait. |
| CF-T12-1, CF-T12-2 | **carry-forward evidence hazards** — a cover requires a sensor leg, and comparison verdict/band construction needs complete discriminating self-check coverage. These are findings, not a new standing rule. |
| CF-T12-3, CF-T12-4 | **closed** — §13.3 now states checked/focused composition; the owner completed all ten smoke steps on 2026-08-09. |
| CF-T12-5 | **open question carried to the next phase pre-doc** — decide whether any positive-control falsification obligation should exist. Deferral is not an intended yes; “no rule” remains a valid outcome. Any rule would require a successor to DD-V-029 and must be narrower than the broad version already rejected. |

The `carry-forward` and named `doc-folded` entries above are distilled in
`implementation/handoff.md` after this classification. Closed items are
not reintroduced there as future work.

### 16. Actual GitHub Actions CI

After explicit owner authorization, `feat/m4-phase-2-t13` was pushed and
the workflow was dispatched against exact phase HEAD
`11f77b689bc234453d2e9ff2f6a1a540c879320a`.
[Run 31298945418](https://github.com/matarillo/wasamo/actions/runs/31298945418)
completed successfully in job
[93208499151](https://github.com/matarillo/wasamo/actions/runs/31298945418/job/93208499151)
(4m08s). Release/debug workspace builds, workspace tests, both C ABI
smokes, CMake and Zig smokes, all counter/gallery hosts and both DSL checks
were green.

The run's sole annotation is non-failing: `mlugg/setup-zig@v2` declares
Node.js 20 and GitHub forced it onto Node.js 24. Zig installation and all
downstream Zig consumers passed, so no CI YAML change is required for this
phase. Only this CI evidence and status record under `process/` follows the
verified SHA; the current code tree is directly verified. Item 16 and T13
are complete. Phase-to-`main` merge is not authorized or performed.

### 17. Human-visible GUI smoke

Required and complete. T12's assistant baseline is the 48 committed
frames with four difference/agreement controls, comparison coverage and
self-check. The owner ran the ten-step protocol on 2026-08-09 and saw
every expected result, with no discomfort in the open-ended step. T13
inspected the frames again for traversal, group single-stop, containment,
restoration, dismissal and checked/focused composition; it adds no new
GUI claim.

### 18. CI YAML sanity

`.github/workflows/ci.yml` needs no change. This phase adds Rust code to
existing crates and existing C/Rust/Zig examples; it introduces no new
language or build system. The workflow already owns release workspace
build, debug workspace build, workspace tests, host builds and DSL
checks in the required order.

## Phase-end gate state

| Artifact | State |
|---|---|
| AGENTS / implementation gate procedures read | complete; AGENTS recorded at T13 start, implementation gates read again and dispositioned at T13a close |
| Five Accepted DDs | complete; DD-001 dated qualification only |
| Moment 2 normative sync | complete locally (`dsl_spec.md` 1.21, `architecture.md` §12.3 / §13) |
| Verification closure mapping | complete above |
| CF-* classification | complete above |
| Handoff | `status: recorded`; includes T13a's repaired / carried split and the green phase-branch CI pointer |
| Local clean evidence-profile verification | green after T13a: release + debug workspace builds, 1,271 tests with 0 failed / ignored, C / Rust / Zig hosts and DSL check |
| Actual phase-branch CI | green: [run 31298945418](https://github.com/matarillo/wasamo/actions/runs/31298945418) on `11f77b689bc234453d2e9ff2f6a1a540c879320a` |
| Owner GUI smoke | complete 2026-08-09 |
| Review lane | Full independent review for T13a complete; no blocking finding, one stale caller comment remediated in `9a4610b` |
| Phase→main merge | owner gate; not authorized or performed |

## Main learning forwarded

The durable design constraint is not merely “there is focus state.” A
consumer must enter through the same traversal root, focus write
primitive and scope presence seam, or it creates a second focus model.
That constraint is forwarded to Phase 3's handler expressions, Phase 4's
scroll/drag input, Phase 5/6's text controls, Phase 9's top layer and
Phase 11's accessibility tree with concrete triggers in the handoff.

The durable evidence learning is narrower: a positive control must be
able to exclude the relevant look-alike, and a comparison implementation
must be checked together with the band that judges it. Whether that
becomes an obligation is deliberately unresolved as CF-T12-5.

T13 adds a separate test-design learning: an allocator-dependent fixture
can be a valuable sensor without being a sufficient regression gate. Once
its condition fired and exposed a real implementation defect, the repair
kept that natural observer and added a deterministic fixture for the exact
derived state. The assertion was not weakened to make non-reuse runs green.
