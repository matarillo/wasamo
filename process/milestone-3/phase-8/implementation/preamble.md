---
phase: M3-Phase 8
title: Selected state + Gallery integration + DSL spec public draft
status: closing
adr: process/milestone-3/phase-8/decisions/preamble.md
plan: process/milestone-3/plan.md
opened: 2026-07-02
---

# M3-Phase 8 — Selected state + Gallery integration + DSL spec public draft: Implementation

This is the execution framing for M3-Phase 8 — **the final M3 phase**.
The design decisions are frozen in the ADR set under
[../decisions/](../decisions/preamble.md) (preamble + DD-M3-P8-001
Accepted 2026-07-01 + DD-M3-P8-002 Accepted 2026-07-02). This file and
its sibling [plan.md](./plan.md) are mutable during the phase; the
in-flight decisions log and CI evidence land in [log.md](./log.md); the
phase residuals land in [handoff.md](./handoff.md) at phase close. This
front-matter `status` flips `draft` → `active` when the owner approves
the T0 review, and `active` → `closing` at the phase-end batch commit.

## Phase 8 scope

Phase 8 closes M3 with **three deliverables and no new layout primitive**
(FD-8-A; [ADR §Context](../decisions/preamble.md#context)):

- **(i) A10 — `ToggleButton` / `checked`** (DD-M3-P8-001 accepted tuple:
  T1 dedicated `ToggleButton`; W1 controlled one-way; α live tab-band
  exclusion with β only as the triggered fallback; SI-1 V-a background
  colour only; SI-2 TH-a no thumbnail highlight; SI-3 admit-on-type
  diagnostics; SI-4 `ToggleButton` / `checked`; SI-5 empty). Implemented
  end-to-end: parser → check → lower → IR emit → runtime loader → widget
  visual → cross-host parity. `ToggleButton` reuses Button's leaf
  measure/arrange and carries Button's `text` / `style` / `enabled` /
  `clicked`; `checked` is the only new attribute
  ([dsl_spec §4.17](../../../../docs/dsl_spec.md) is the Moment 1
  normative target).
- **(ii) A1 — gallery integration.** Fold the per-phase verification
  screens into the **single Photo Gallery target app**
  ([gallery-wireframe.html](../../requirements/gallery-wireframe.html)),
  and run it end-to-end in **all three hosts**. Codebase fact (T1
  recon-confirmed at planning): only `examples/gallery-rust/` exists —
  **`examples/gallery-c/` and `examples/gallery-zig/` must be created**
  (from the `counter-c` / `counter-zig` templates) and added to CI's
  per-example build steps.
- **(iii) A12 — DSL spec public draft.** Editorial pass at the
  external-reader bar; DD-002 accepted dispositions
  (A-2 / B-1b / B-2c / B-3b / B-4a / B-5b / B-6b / C-2) already drafted
  into §4.17 / §4.18 at Moment 1; the `status: public-draft` marker, the
  promotion change-history entry, and the external-reader smoke land at
  Moment 2 (phase close), per the
  [ADR Moment 1/2 split](../decisions/preamble.md#upstream-document-revisions-moment-1--moment-2).

Because Phase 8 is the final M3 phase, it also owns the milestone-close
inputs: the M3 `handoff.md` draft (PM-2 wrapper rule, Problem B
disposition, DD-001's five deferred axes), the no-silently-deferred-
surface audit, the A11 auditability check, and the M3 CHANGELOG entry
([plan.md §Milestone-end criteria](../../plan.md);
[constraints §8](../requirements/constraints.md)).

Out of scope — the frozen Phase 7b placement surface (read, not
re-decided); explicit-sizing implementation (Problem B Vision DR
**Accepted 2026-07-02**: scheduled M4/M5 spike + M6 backstop — Phase 8
carries only the documentation posture); PM-2 wrapper-rule decision;
default-alignment unification; two-way binding / widget-owned state /
group widgets / generic Toggle (DD-001 Axes 1–5); M4 input / modal
focus / hit-testing / real images / theme. The deferred-items 正本 is
the [framing scope table](../requirements/framing.md) + the
[constraints carry-forward table](../requirements/constraints.md).

## Acceptance relation

No new AC (FD-8-F resolved at the DD-002 Accept; recorded in the
[M3 plan Revision log](../../plan.md)). Phase 8 discharges **A10**
(ToggleButton/checked), **A1** (integrated gallery, three hosts),
**A12** (public-draft promotion), under continuing **A11** sync. As the
final phase it also carries the seven milestone-end criteria
([plan.md §Milestone-end criteria](../../plan.md)); their task mapping
is in [plan.md](./plan.md) §Milestone-close batch.

## Verification closure (ADR items → tasks)

The ADR's [§Verification closure](../decisions/preamble.md#verification-closure-what-counts-as-phase-8-evidence)
fixes six evidence lines; this plan adds only the task mapping:

| ADR evidence item | Task(s) |
|---|---|
| (1) `wasamoc check` positive + SI-3 reject firing tests | T3 |
| (2) lowering / IR roundtrip / loader re-reject | T3 (emit/roundtrip) + T4 (loader) |
| (3) `checked` propagation audit — (i) reject on non-supporting widgets, (ii) bool binding reaches visual, (iii) cross-host parity | (i) T3, (ii) T4, (iii) T6 + T7 |
| (4) layout-skeleton technical smoke (before owner UI review) | T2 |
| (5) assistant GUI evidence + two-frame positive controls (selected + exclusion, lightbox) | T7 (T5/T6 supply the surface) |
| (6) A12 spec-closure gate (external-reader smoke, marker, CHANGELOG) | T8 (smoke + editorial) + T11 (marker flip + CHANGELOG) |

The Windows-runtime fixtures fail — not skip — on a runner without
Compositor capability (Phase 2–7b pattern, `0x80070005` guard +
keep-alive apartment helper).

**Positive-control discipline:** a single static frame a wrong
implementation could equally produce is not evidence. The A10 proof is
the **two-frame** toggle (selected visual changes; under α, exclusion —
one on, others off — in the same two frames). The lightbox proof is the
subtree present/absent pair. The wrap/overflow proof is the narrow-width
reflow frame set.

## Owner checkpoints (FD-8-G — staged, not piled at phase end)

| Checkpoint | Content | Task |
|---|---|---|
| G(1) | wireframe-fidelity / M3-placeholder agreement → A1 feature-mapping table updated | T2 |
| G(2) | first render of the integrated gallery, representative host (Rust) | T5 |
| G(3) | two-frame positive controls (selected/exclusion + lightbox) | T7 |
| G(4) | public-draft + M3 handoff draft review | T9 |
| G(5) | final human-visible smoke over the agreed state set | T10 |

UI review is on the representative host (Rust); C / Zig are checked for
identical render / no regression only (framing §検証方針).

## Obligations carried from the ADR / framing (represented from the start)

1. **First implementation task is a spike (T1).** Reads every landing
   file end-to-end (not grep-sample), compiler-verifies the
   `ToggleButton` widget-kind addition (throwaway kind → build →
   enumerate sites → revert; no production code), fixes the bisectable
   sequencing, and records the T3 gate selection in [log.md](./log.md)
   before T3 opens. Exit criterion: every open point is assigned to a
   downstream task and its scope is seen — not "no surprises expected"
   ([implementation-gates.md](../../../procedures/implementation-gates.md)).
2. **α fallback trigger is defined, not open.** β (single-toggle
   static tab band) substitutes **only if** implementation shows the
   live tab-band exclusion unworkable; the substitution is recorded in
   the A1 table / this plan with SI-2 static-approximation accounting
   (DD-001 §α/β disposition). It is not an owner re-choice.
3. **Layout-skeleton smoke precedes owner UI review** (framing R7).
   What existing surface can fix is fixed; what it cannot is triaged to
   the A1-table placeholder / G(1) agreement / Problem B — **no
   layout-engine change in Phase 8**.
4. **Final-task / phase-end / milestone-end ownership split.** T11 owns
   local gates, Moment 2 docs sync (including the `public-draft` marker
   flip + CHANGELOG), the M3 plan row flip, the candidate ledger, and
   its own step retro. The **phase-end batch** owns the CI run id,
   `handoff.md` finalization, the phase-end retrospective, and this
   file's status flip. The **milestone-close batch** (workflow §7) owns
   the milestone retrospective, M3 `handoff.md` finalization, and the
   ROADMAP completion flip. The corresponding T11 bullets stay `[ ]` at
   T11 close.

## Implementation gates

Every task runs
[implementation-gates.md](../../../procedures/implementation-gates.md)
at task start and close (selection recorded in [log.md](./log.md) with
reasons for non-applicable gates, before choosing an approach). Known
phase-wide load (from the
[ADR §Implementation gate expectations](../decisions/preamble.md#implementation-gate-expectations)):

- **Trap #1 (semantic migration / call-site audit)** — the new widget
  kind + `checked` attribute cross parser / check / lower / IR emit /
  loader / runtime visual. T3/T4 close with the `rg`-enumerated
  call-site audit table over widget-kind dispatch sites (check admission
  tables, lower/emit kind mapping, loader kind dispatch, widget
  construction), each site classified.
- **Trap #4 (untested authored branch)** — SI-3 rejects (`checked` on
  `Button` / `Text` / others; loader re-reject of malformed IR) each
  fire a direct test; positive fixtures pin `ToggleButton` carrying
  `text` / `style` / `enabled` / `clicked` + `checked` literal and
  binding forms.
- **Trap #2 (structural side effects)** — the gallery restructure
  invalidates layout-coupled capture coordinates; retained capture
  scripts re-derive them (T5); no parallel-data class is introduced
  (ToggleButton is additive, no storage migration).
- **Trap #7 (GUI positive control)** — T2 smoke (de-risk) and T7
  (authoritative evidence) with the two-frame controls above.
- **Trap #5 (carry-forward)** — the five DD-001 axes, PM-2 wrapper
  rule, Problem B disposition, default-alignment, spelling
  affirmed-keep, and any gallery workaround land in the candidate
  ledger → phase handoff → M3 handoff.
- **Review tiers** — T3 (compiler/IR surface; full independent review,
  with the branch/test-focused check folded in for the reject matrix),
  T4 (runtime structural: new widget node; full), T7 (GUI-render
  evidence; full). T5/T6 are example/host-side and take a normal review
  with the trap-#2 coordinate check; T2's smoke is an internal de-risk
  recorded in log.md, not the authoritative GUI evidence.

## Technical risks (planning-time recon; T1 sharpens)

| ID | Risk | Mitigation |
|---|---|---|
| R-1 | **`ToggleButton` crosses every layer** (framing R6): parser widget-name admission, check attribute tables (today string-pair rows like `("Button", "enabled")` in `wasamoc/src/check.rs`), lower/emit kind mapping, loader dispatch, widget visual. A missed site can silently drop the widget or its attribute; T1 recon found the IR kind carrier is a string and an unknown widget currently produces a `wasamoc check` warning with exit 0, so compiler errors alone will not enumerate the surface. | T1 source audit + deliberate wrong-kind probe + T3/T4 trap-#1 audit table; T3 positive fixtures must prove `ToggleButton` is known (no unknown-widget warning) and the checked-propagation audit's three pinned points must fire. |
| R-2 | **V-a checked visual must be unambiguous in two frames.** Background-only cue could wash out against theme/backdrop (Mica). | SI-1 implementation checkpoint (DD-001): if the single cue is ambiguous in the two-frame control, that is a design-revision trigger, not a new option choice; verify at T4 fixture + T7 frames. |
| R-3 | **Full-gallery assembly surfaces latent Fill→0 collapse / aspect abort** (framing R7, Problem B). The wireframe Grid frame (40 / 1* / 20 rows) + ScrollView + aspect Boxes in one tree is new composition. | T2 skeleton smoke before any owner review; fix-with-existing-surface or triage to placeholder / G(1) / Problem B; no layout-engine change. |
| R-4 | **α tab-band exclusion proves unworkable in the real gallery** (framing R8). | Defined β fallback with recorded substitution (obligation 2); the stage-1 spike already ran α on the live runtime, so residual risk is composition-level only. |
| R-5 | **Verification-screen sweep breaks shipped behaviour** (the gallery `.ui` is the regression surface for Phases 2–7b). | T5 keeps the workspace + fixtures green per commit; wrap/scroll/lightbox/iteration remain exercised in the integrated app (A1 table); capture coordinates re-derived (trap #2). |
| R-6 | **New C / Zig hosts + CI steps fail on CI, not locally** (build ordering: `wasamoc.exe` must exist before CMake / `zig build`; AGENTS.md §Build ordering). T1 recon found the counter templates hard-code counter-specific artifact names (`COUNTER_UI`, `COUNTER_UIC`, `COUNTER_UIC_H`, `COUNTER_UIC` array/import names) and the CI Zig step relies on the default release `wasamoc.exe` path rather than passing `-Dwasamoc`. | T6 mirrors the proven `counter-c` / `counter-zig` CI steps (which already encode the ordering), ports all artifact names to gallery names, and runs a local clean-order rehearsal before push. |
| R-7 | **Editorial pass drifts to "later" (framing R1) or writes unsettled surface as final (R2).** | Spec-sync is a hard phase-end gate; DD-002 dispositions are a checklist (T8); the marker flip is T11-gated on the surface actually running. |

## Lifecycle transition

Implementation start is gated on Moment 1 commit-set completion (T0):
ADR set Accepted (done); `docs/dsl_spec.md` §4.17 / §4.18 +
`docs/architecture.md` §6.7.7 design draft (done, commits `3a1af26` /
`28a991a`); the remaining re-sync set (the `_roadmap.md` A10 / A1
wording in the accepted `ToggleButton` / `checked` lexeme + the framing
packet-C annotation, per DD-001 §Accepted disposition item 6); and this
`preamble.md` + `plan.md` + skeleton `log.md` / `handoff.md`
owner-reviewed and landed. T11 owns the Moment 2 sync; the phase-end
batch and the milestone-close batch follow per the ownership split
(§Obligations item 4).

## Cross-references

- ADR set: [../decisions/preamble.md](../decisions/preamble.md) +
  [DD-M3-P8-001](../decisions/dd-m3-p8-001-button-selected-state-surface.md) +
  [DD-M3-P8-002](../decisions/dd-m3-p8-002-dsl-spec-public-draft-promotion.md).
- Framing / constraints:
  [../requirements/framing.md](../requirements/framing.md) (FD-8-A…G,
  A1 feature-mapping table, 検証方針, R1–R8);
  [../requirements/constraints.md](../requirements/constraints.md)
  (§1–§9 + carry-forward table).
- Stage-1 spike (authority for α/β feasibility):
  [../requirements/dd-001-stage1-spike.md](../requirements/dd-001-stage1-spike.md).
- Problem B Vision DR (Accepted 2026-07-02):
  [author-controllable-sizing-surface.md](../../../cross-milestone/decisions/author-controllable-sizing-surface.md).
- Specification (Moment 1 design draft; marker flips at T11):
  [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) §4.17
  (`ToggleButton` / `checked`) + §4.18 (public-draft future notes);
  [`docs/architecture.md`](../../../../docs/architecture.md) §6.7.7.
- ABI: [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no touch
  judged**; revisited only if the external-reader smoke forces a
  future-compat note (owner confirmation required).
- Landing source (T1 reads all): `wasamoc/src/{lexer,parser,ast,check,lower,emit}.rs`
  (widget-kind + attribute admission), `wasamo-ir/src/lib.rs`
  (`IrNode` kind carrier), `wasamo-runtime/src/ir_loader.rs` (kind
  dispatch + binding registration), `wasamo-runtime/src/widget.rs`
  (Button visual / `enabled` propagation), `wasamo-runtime/src/layout.rs`
  (Button leaf measure), `examples/gallery/gallery.ui` +
  `examples/gallery-rust/`, `examples/counter-c/` +
  `examples/counter-zig/` (host templates),
  `.github/workflows/ci.yml` (per-example steps).
- Visual contract:
  [gallery-wireframe.html](../../requirements/gallery-wireframe.html) +
  [spec.md](../../requirements/spec.md) (Interaction: lightbox
  close/prev/next as Button click handlers; Out-of-scope §Visual).
