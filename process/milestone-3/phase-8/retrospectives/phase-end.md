---
title: M3-Phase 8 phase-end retrospective
status: recorded
created: 2026-07-06
scope: phase-end
phase: M3-Phase 8 — Selected state + Gallery integration + DSL spec public draft
---

# M3-Phase 8 phase-end retrospective

## Scope

M3-Phase 8 is the **final M3 phase**. It closed M3 implementation with
three deliverables and no new layout primitive: **A10** (`ToggleButton`
/ `checked` end-to-end across parser / check / lower / IR emit / loader
/ widget visual), **A1** (the per-phase verification screens folded into
the single integrated Photo Gallery, running on the Rust, C, and Zig
hosts with CI steps), and **A12** (the DSL spec promoted to
`public-draft` at Moment 2), under continuing **A11** sync. This retro
is the separate phase-end record (checklist items 12–18) after T11 was
merged into `feat/m3-phase-8` (`882ff20`). The per-task records (T1–T11)
and their step-end retros remain the implementation evidence; the
phase-end batch records (start gate, handoff finalization, procedure
folds) are in [implementation/log.md](../implementation/log.md) and
[implementation/handoff.md](../implementation/handoff.md).

## Main Learnings

- **The staged owner-checkpoint plan (FD-8-G) worked: five gates, no
  pile-up at phase end.** G(1) placeholder agreement (T2), G(2) first
  render (T5), G(3) two-frame positive controls (T7), G(4) public draft
  + M3 handoff (T9), and G(5) human-visible smoke (T10) each landed on
  the task that produced their surface, so the phase close is
  record-keeping, not judgment.
- **Evidence is bound to surface state, and the binding is checkable.**
  The G(5) acceptance was bound to the surface unchanged since the T7
  capture commit (`5b66321`); T10, T11, and this batch each re-verified
  the path-scoped product range empty instead of assuming it. This
  turned "is the owner smoke still valid?" from a judgment into a git
  query.
- **A string-carried widget kind changes what "proof" means.** Because
  unknown widgets are warning-only, the T1 spike's wrong-kind probe —
  not compiler errors — established the T3/T4 evidence shape (positive
  no-unknown-warning fixture + runtime catalog mirroring). The spike
  discipline (read every landing file, compiler-verify, assign every
  open point) paid for itself here.
- **Document-sync tasks accumulated their own reusable trap surface.**
  Parallel-doc drift (cite, don't restate) and the
  final-branch-state re-run after post-retrospective remediation
  recurred often enough (T4–T9) to be folded into the procedure SSOTs
  at this close, rather than carried as prose.

## Phase-End Gate

Final verification-closure mapping (workflow.md §6.1): the ADR's six
fixed evidence lines
([decisions/preamble.md §Verification closure](../decisions/preamble.md#verification-closure-what-counts-as-phase-8-evidence))
→ the discharging task + concrete evidence. The full per-test close-gate
tables live in [implementation/log.md](../implementation/log.md); this
is the closure index.

| # | ADR evidence line | Discharged by | Concrete evidence (representative; full tables in log.md) |
|---|---|---|---|
| (1) | `wasamoc check` positive + SI-3 reject firing tests | **T3** | Positive: `togglebutton_known_widget_and_attrs_accepted_without_warning` (no-unknown-warning proof), `togglebutton_alpha_tab_band_shape_accepted`. SI-3 firing rejects: `checked_on_button_rejected`, `checked_on_text_rejected`, `checked_on_other_widget_rejected`, `checked_on_scrollview_rejected_by_container_attr_gate`, `checked_on_zstack_rejected_by_container_attr_gate`, `component_level_checked_routes_to_host_attr_reject`, `togglebutton_checked_non_bool_rhs_rejected`, `togglebutton_checked_i32_state_rejected`, `togglebutton_unknown_attr_rejected`. (log.md T3 trap-#1/#4 tables.) |
| (2) | lowering / IR roundtrip / loader re-reject | **T3** (emit / roundtrip) + **T4** (loader) | T3: `togglebutton_surface_emits_literal_and_binding_forms` public pipeline fixture; lower/emit carry-through incl. the absent-`checked` pair (`togglebutton_absent_checked_{lowers_no_ir_prop_or_binding,emits_no_checked_prop_or_binding}`). T4: loader re-reject matrix (`validate_rejects_checked_on_{button,text}_runtime_ir`, `validate_rejects_checked_binding_on_{button,text}_runtime_ir`, `validate_rejects_togglebutton_{unknown_attr,unknown_binding,style_binding,checked_non_bool_literal,checked_non_bool_binding,checked_wrong_read_tag}_runtime_ir`, malformed Button-family literals) + runtime default-`false` materialization. (log.md T3/T4 trap-#1/#4 tables.) |
| (3) | `checked` propagation audit — (i) reject on non-supporting widgets, (ii) bool binding reaches visual, (iii) cross-host parity | **T3** (i) + **T4** (ii) + **T6/T7** (iii) | (i): the SI-3 + loader reject matrices above (dual gate). (ii): `togglebutton_bool_state_flip_reaches_checked_visual`, `togglebutton_default_false_and_literal_checked_drive_distinct_visuals`, `togglebutton_alpha_exclusion_click_leaves_exactly_one_checked`, colour-priority matrix (disabled > checked > state). (iii): `t6-parity-{rust,c,zig}.png` identical default view on all three hosts; T7 selected/exclusion frames on the final surface. |
| (4) | layout-skeleton technical smoke before owner UI review | **T2** | T2 skeleton smoke (build + launch + DPI-aware capture + triage; no layout-engine change) recorded in log.md ahead of the G(1) packet; owner "OK accept all" 2026-07-04 on the wireframe-fidelity / M3-placeholder A1 table. |
| (5) | assistant GUI evidence + two-frame positive controls (selected + exclusion, lightbox) | **T7** (surface from T5/T6) | Seven `evidence/t7-gallery-*.png` frames + analysis in `evidence/README.md`, captured visible-desktop outside the sandbox on the post-T6 surface: selected/exclusion two-frame control (All → Albums → Favorites, previous tab clears), lightbox subtree present/absent pair, narrow-width reflow + scroll-offset pair, aspect visual backed by the Phase 2 aspect tests. G(3) owner confirmation 2026-07-05. |
| (6) | A12 spec-closure gate (external-reader smoke, marker, CHANGELOG) | **T8** (smoke + editorial) + **T11** (promotion) | T8 external-reader smoke: per-surface verdicts all "yes" (A1–A10, A13, grammar surfaces; log.md table); DD-002 disposition check all "yes"; A11 audit all nine phases traced; no-silently-deferred-surface audit clean. T11: `status: public-draft` marker + `## Public draft change history` anchor + appended revision row 1.15 in `docs/dsl_spec.md`; architecture status sync; CHANGELOG M3 entry linking all nine phase ADRs + the anchor. |

The Windows-runtime fixtures remain CI-gated fail-not-skip; their
remote confirmation is item 16's run. The positive-control discipline
(a single static frame a wrong implementation could equally produce is
not evidence) is met by the two-frame selected/exclusion control, the
lightbox present/absent pair, and the firing reject tests in (1)/(2).

## Checklist

12. **Acceptance criteria (Ax) achieved:** **achieved**
    - **A10** is discharged. `ToggleButton` is a dedicated widget
      carrying Button's `text` / `style` / `enabled` / `clicked` plus
      controlled one-way `checked: <bool>` (literal or bool-state
      binding; runtime default `false`; background-colour-only selected
      visual; admission on `ToggleButton` only, compiler reject + loader
      re-reject each with firing tests). The Gallery tab band drives it
      live with α author-composed exclusion.
    - **A1** is discharged. `examples/gallery/gallery.ui` drives the
      integrated Photo Gallery end-to-end on all three hosts
      (`gallery-rust`, new `gallery-c` / `gallery-zig` + CI steps),
      exercising every M3 layout primitive, both grammar surfaces, the
      bool scalar, `ToggleButton.checked`, and `slot.*` placement — with
      the per-phase verification surfaces swept (FD-8-E, no verification
      menu remains).
    - **A12** is discharged. `docs/dsl_spec.md` is promoted to
      `public-draft` (document version 1.15) with the promotion anchor,
      the T8 external-reader smoke recorded (all-"yes"), and the
      CHANGELOG M3 entry; DD-002 dispositions verified present and
      honestly worded (no DD/option labels in spec prose).
    - **A11** is discharged for the Phase 8 slice and auditable
      milestone-wide (T8 A11 audit: all nine phase ADR sets name their
      spec sections; no pointer fix required).
    - ADR "discharged" statements and implementation are consistent; no
      phase-end ADR-touch case fired (checked at T11 and re-checked at
      this batch: no AC-discharge divergence, no out-of-phase residual
      filing, no thesis-level addition). Formal discharge *recording*
      (M3 plan Progress rows + criterion mapping) is milestone-close
      owned per [plan.md §Milestone-end criteria](../../plan.md)
      criterion 1 — an ownership split, not a divergence.

13. **`CHANGELOG.md` / `process/_roadmap.md` consistency:** **consistent**
    - `CHANGELOG.md` Unreleased carries the `M3-Phase 8` milestone entry
      (2026-07-06) covering A10 / A1 / A12, linking each of the nine M3
      phase ADR preambles and the public-draft anchor, and pointing to
      the M3 handoff for carry-forwards (citation, not restatement).
    - `process/_roadmap.md` is unchanged this batch and its A1 / A9 /
      A10 / A12 wording matches the shipped scope (the `ToggleButton` /
      `checked` lexeme re-sync landed 2026-07-03 per DD-001 item 6). The
      M3 completion flip is milestone-close owned. Phase status lives in
      `process/milestone-3/plan.md`, whose Phase 8 row is
      `implementation complete; phase-end pending` and flips at the
      phase → main merge / post-merge distillation.

14. **`VISION.md` / thesis-level claim impact:** **no update**
    - Phase 8 completes the M3 thesis (the DSL surface expresses a real
      layout and is published as a public draft) without changing it.
      The T8 architectural-family capstone re-read confirmed
      `ToggleButton.checked` and the public draft introduce no
      view-function / host-language composition model — family (1)
      stands, no vision decision record opened.
    - The carry-forwards (DD-001 axes, PM-2, Problem B, sizing) are
      future design inputs living in the handoffs, not thesis revisions.

15. **Next-phase framing inputs:** **organized**
    - [implementation/handoff.md](../implementation/handoff.md) is
      finalized from the T11 candidate ledger: six carry-forward
      constraints (new-widget-kind positive fixture; runtime
      defensive-reader catalog mirroring; `CopyFromScreen` capture
      discipline; declarative C/Zig host boundary; append-only spec
      revision history; owner-gate evidence-binding check), two ledger
      items **doc-folded** into the procedure SSOTs
      (`retrospectives.md` final-branch-state note;
      `implementation-gates.md` trap-#3 document analogue), the G(5)
      binding closed **local-only** beyond the phase, and the
      milestone-level residual set pointed at the T9 owner-reviewed
      `process/milestone-3/handoff.md` draft (finalized at milestone
      close, where its `status: recorded` flip lands).
    - **Zero item-10 `phase-sync` classifications existed across
      t1–t11**, so no open phase-sync item survives this close (the
      Moment 2 sync was fully discharged by T11's promotion set).

16. **CI green:** **green**
    - The phase branch `feat/m3-phase-8` was pushed at head
      `d72090bcfb4e9325f4efcce09d22b5f9991c4fe8` (instrumental push for
      `workflow_dispatch`; the owner push gate remains the post-merge
      main push).
    - GitHub Actions workflow `CI`, event `workflow_dispatch`, run
      [28784793695](https://github.com/matarillo/wasamo/actions/runs/28784793695)
      concluded **success** (`2026-07-06T10:24:30Z` →
      `2026-07-06T10:28:10Z`).
    - Green CI steps: release + debug workspace builds, workspace tests
      (Windows-runtime fixtures CI-gated fail-not-skip), C ABI smoke
      (`cl` / `clang-cl`), CMake smoke, Zig binding smoke,
      `counter-c` / `counter-rust` / `counter-zig`,
      `wasamoc check counter.ui`, and the **first remote execution** of
      the T6-added `gallery-c (CMake, Release)`,
      `gallery-zig (Zig, ReleaseSafe)`, and
      `wasamoc check gallery.ui` steps. This discharges the CI-gated
      cells of Phase-End Gate lines (2) / (3) and retro item 18's
      remote confirmation. Annotations were upstream deprecation
      warnings only; the workflow has no `cargo fmt` step, so fmt
      remains local evidence.
    - Phase-end local clean-rebuild ground truth is recorded in
    [implementation/log.md](../implementation/log.md) (T11 local gates:
    `cargo fmt --all -- --check`, `cargo clean` → release → debug →
    `cargo test --workspace`, C / Zig hosts in the AGENTS.md build
    order, all green), and every commit since is docs / process only —
    re-verified by the path-scoped product-range check
    (`5b66321..HEAD` empty for `wasamoc` / `wasamo-runtime` /
    `wasamo-ir` / `examples` / `bindings` / `.github`).

17. **Human-visible GUI smoke:** **needed; satisfied by the T10 G(5)
    owner smoke**
    - Phase 8 changed user-visible behaviour (new widget, integrated
      Gallery, two new hosts), so human-visible smoke was needed. The
      owner ran the scripted smoke
      (`evidence/t10-owner-smoke-script.md`) on a visible Windows
      desktop per
      [human-visible-smoke.md](../../../../docs/notes/human-visible-smoke.md):
      Rust host over the agreed state set (default view, lightbox
      open/close, tab selection with exclusion positive control,
      narrow-width reflow + scroll, crash-free close) plus launch +
      default-view confirmation on the C and Zig hosts. Owner acceptance
      "G(5) OK" recorded 2026-07-06, no fail observation.
    - The retrospectives.md item-17 counter-host wording maps to this
      phase as follows: the phase's user-visible acceptance surface is
      the integrated Gallery (smoked above on all three hosts); the
      counter hosts are unchanged this phase and their CI smokes remain
      the base-path regression gate (item 16).
    - No additional phase-end smoke is needed: every post-T10 commit is
      docs / process only (path-scoped range check in item 16), so the
      G(5) binding to the T7-capture surface (`5b66321`) holds. Any
      product-path change before the merge invalidates it and requires
      an owner re-run.

18. **CI YAML sanity check:** **updated in-phase; no further change
    required**
    - Phase 8 added no new language or build system (AGENTS.md CI
      rules), but T6 extended the per-example enumeration:
      `gallery-c (CMake, Release)`, `gallery-zig (Zig, ReleaseSafe)`,
      and `wasamoc check examples/gallery/gallery.ui` steps mirroring
      the counter ordering after the release workspace build. Item 16's
      run is their first remote execution; no further YAML change is
      required.

## Merge Readiness

Checklist items 12–18 are all recorded and the phase-branch CI gate is
**green** (run 28784793695). `implementation/handoff.md` is finalized
and `implementation/preamble.md` flips to `status: closing` in the same
phase-end close commit that records the CI run id. Per the retrospective
procedure, this record is **evidence** for the phase → main merge gate,
not merge authorization: the phase → main no-ff merge requires explicit
owner approval, and push to main is a separate owner gate. Before the
merge executes, the doc gates re-run over the final branch state and
the path-scoped product-range check re-verifies the G(5) binding.
