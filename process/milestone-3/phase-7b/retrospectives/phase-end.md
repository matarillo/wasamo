---
title: M3-Phase 7b phase-end retrospective
status: recorded
created: 2026-06-24
scope: phase-end
phase: M3-Phase 7b — Parent-interpreted placement attributes
---

# M3-Phase 7b phase-end retrospective

## Scope

M3-Phase 7b is a **corrective** phase (no new layout primitive, no new app
feature): it aligns the parent-interpreted placement surface that Phases
5–7 shipped piecemeal onto **one author surface** (`slot.*`) and **one
internal model** (child-slot `SlotData`) across parser / checker /
lowering / textual IR / loader / runtime / examples, before the Phase 8
public draft freezes it. This retro is the separate phase-end record
(checklist items 12–18), after T7 was merged into `feat/m3-phase-7b`
(`a2ebeef`). The per-task records (T1–T6b, T7) and their step-end retros
remain the implementation evidence.

## Main Learnings

- **The last-task / phase-end ownership split held and was sharpened.** T7
  (the final step) owned the Moment 2 docs sync, the local clean rebuild,
  the M3 `plan.md` row flip, and the candidate ledger; this phase-end batch
  owns the CI run id, the handoff finalization, this retrospective, and the
  `preamble.md` `active → closing` flip. The T7 review caught the
  `preamble.md` Lifecycle wording still attributing the spec / architecture
  / M3-plan flips to the phase-end batch and corrected it — a reminder that
  the split must be auditable in *every* doc that states it, not just the
  plan.

- **Pin the living spec to landed source, not to the design draft.** The
  Moment 2 sync found two divergences a status-only flip would have frozen:
  the §5 AST `PlacementBind` variant the implementation never adopted (it
  rides `PropertyBind`), and the `Widget { node, slot_data }` struct sketch
  that landed as the tuple `Widget(IrChildSlot)`. Reading the type
  definitions, not grepping markers, surfaced them.

- **A "constraint finding" can decompose into independent problems with
  different owners.** The T5 Grid-in-ZStack finding was a checker bug
  (problem A, fixed in T6b) **and** a deferred sizing gap (problem B,
  carried to a Phase 8 Vision DR). Git-verifying that 7b changed no layout
  maths is what let the in-scope fix be separated from the deferred surface
  instead of bundling both under one "deferred" label.

## Phase-End Gate

Final verification-closure mapping (workflow.md §6.1): the ADR's five
fixed evidence lines
([decisions/preamble.md §Verification closure](../decisions/preamble.md#verification-closure-what-counts-as-phase-7b-evidence))
→ the discharging task + concrete evidence. The full per-test close-gate
tables live in [implementation/log.md](../implementation/log.md); this is
the closure index.

| # | ADR evidence line | Discharged by | Concrete evidence (representative; full tables in log.md) |
|---|---|---|---|
| (1) | `wasamoc check` — positive + the full reject matrix / forcing table | **T4** (+ T6b) | The DD-001 forcing-table firing tests: `check::tests::{grid_direct_slot_child_accepted, grid_direct_slot_lowers_to_same_grid_slot_data_as_cell, slot_property_inside_cell_attrs_is_mixing_reject, slot_property_inside_cell_content_is_non_admitting_parent_reject, slot_property_under_non_admitting_parent_rejected, grid_direct_slot_unknown_key_rejected, grid_direct_slot_constant_rhs_rejected, *_value_namespace_prefers_alignment_keyword, zstack_child_bare_alignment_rejected, …}`; parser `malformed_slot_property_keys_rejected_at_parse`. T6b: `check::tests::{zstack_grid_child_slot_alignment_accepted, *_value_still_validated, nonadmitting_parent_grid_child_slot_still_rejected, …}`. (log.md T4 / T6b close maps.) |
| (2) | lowering / textual-IR roundtrip / loader rejection | **T2** | IR roundtrip: `wasamo_ir::tests::child_slot_carries_optional_slot_data`; `ir_loader::tests::grid_slot_emit_then_parse_preserves_payload_values`. Emit canonicalization: `emit::tests::{grid_cell_emitted_as_child_slot_with_grid_placement, zstack_emitted_as_node_with_direct_children_in_order}`. Loader stale-form / malformed rejects: `{grid_legacy_cell_*_rejected_as_stale_ir, zstack_legacy_bare_child_placement_rejected_as_stale_ir, child_slot_*_rejected_at_parse, grid_slot_*_rejected}`. (log.md T2 trap-#1 / trap-#4 maps.) |
| (3) | Windows-runtime integration (CI-gated, fail-not-skip): layout reads placement from child-slot storage; insert / remove / replace preserves order + Visual sibling order + placement + invalidation; destroy leaks no placement metadata; `if` / `for`-generated children carry placement | **T3** | Integration fixtures: `zstack_replace_child_preserves_child_slot_placement`, `conditional_zstack_reinsert_uses_declared_placement_metadata`, `reactive_for_zstack_tail_append_uses_child_carried_placement`, `static_for_under_zstack_preserves_child_carried_placement`, `grid_rooted_fixture_lays_out_cells_through_visual_tree`; trap-#3 grep (no `cell_placements` / `zstack_placement` survives). Local `cargo test --workspace` green (T7 ground truth); the CI-gated confirmation is **item 16 — green**, run [28072510434](https://github.com/matarillo/wasamo/actions/runs/28072510434). (log.md T3 trap-#1/#2/#3 maps.) |
| (4) | assistant-visible GUI + positive control; owner-visible smoke | **T5** (assistant) + **T6** (owner) | T5: `evidence/t5-placement-demo.png` (ZStack start/center/end three x-positions; Grid stretch-vs-centered), `t5-lightbox-{slot-current,bare-baseline}.png` (same-position, pixel-identical vs T4-pre `3134287`), analysis in `evidence/README.md`. T6: owner accepted `evidence/t6-{placement-demo,lightbox,home}.png` (2026-06-23, no fix iteration). |
| (5) | A12 spec-closure gate (`docs/dsl_spec.md` placement chapter + `docs/architecture.md` model; Moment 1 → Moment 2 marker flip) | **T7** | `docs/dsl_spec.md` §4.16 placement chapter at the external-reader bar + §4.12 / §4.13 / §8.5 / §8.11; `docs/architecture.md` §6.7.9 / §6.8.4 / §6.8.6; markers flipped to closed / implementation-synced; divergences D1 (`PlacementBind`→`PropertyBind`) and D2 (`Widget{…}`→`Widget(IrChildSlot)`) corrected; `abi_spec.md` untouched. (log.md T7 docs-sync close artifact + A12 gate note.) |

All five evidence lines are discharged: line (3)'s CI-gated confirmation
went green (item 16, run 28072510434). The positive-control discipline
(a single static frame a wrong implementation could equally produce is not
evidence) is met by the same-position-plus-contrast proof in (4) and the
firing reject tests in (1) / (2).

## Checklist

12. **Acceptance criteria (Ax) achieved:** **achieved**
    - **A13** is discharged. Grid cell placement (`row` / `column` /
      `row-span` / `column-span` / `h-align` / `v-align`) and ZStack
      alignment (`h-align` / `v-align`) are authored as parent-interpreted
      `slot.*` metadata on one `slot.` namespace; Grid accepts **both**
      `Cell` and direct `slot.*` (one form per child), ZStack accepts
      `slot.*`. `wasamoc` check / lower, textual IR, `wasamo-runtime`, and
      the gallery exercise the surface; the PM-2 accept-set matches the
      `_roadmap` A13 wording.
    - **A11** is discharged for the Phase 7b slice. `.ui`, `wasamo-ir`,
      `wasamoc`, `wasamo-runtime`, `docs/dsl_spec.md`,
      `docs/architecture.md`, and `examples/gallery/` are synchronized for
      the shipped placement surface (T7 Moment 2 sync).
    - **A12** is discharged for the Phase 7b increment. The
      `docs/dsl_spec.md` §4.16 placement chapter is at the external-reader
      bar (admission table, accepted/rejected examples matching the shipped
      diagnostics, the two distinct mixing vs non-admitting-parent rejects,
      the value-namespace rule, constant-per-instance, per-container
      defaults); `docs/architecture.md` §6.7.9 / §6.8.4 / §6.8.6 are
      implementation-synced. The full A12 public draft remains Phase 8.
    - ADR "discharged" statements and implementation are consistent. No
      phase-end ADR-touch case fired (no AC-discharge divergence to fold, no
      out-of-phase residual cross-ref, no thesis-level addition); the two
      DDs stay Accepted at Moment 1.

13. **`CHANGELOG.md` / `process/_roadmap.md` consistency:** **consistent**
    - `CHANGELOG.md` Unreleased now has an `M3-Phase 7b —
      Parent-interpreted placement (2026-06-24)` entry covering A13, the
      `slot.*` surface, the child-slot `SlotData` model, the textual-IR
      record, no-ABI, the gallery proof, and the carry-forward residuals.
    - `process/_roadmap.md` is unchanged: it is the acceptance-criteria
      SSOT and the A13 / A11 / A12 wording still matches the shipped Phase
      7b scope. Phase status lives in `process/milestone-3/plan.md`, whose
      Phase 7b row is `implementation complete; phase-end pending` and
      flips to `complete` at the phase → main merge / post-merge
      distillation.

14. **`VISION.md` / thesis-level claim impact:** **no update**
    - Phase 7b is corrective: it strengthens the M3 thesis that the DSL
      distinguishes "data the parent interprets about a child" from a
      widget's own properties (A13), but it adds no roadmap category and
      changes no product-thesis wording.
    - The carry-forwards (PM-2 wrapper rule, author-controllable sizing)
      are future design inputs, not thesis-level revisions; they live in
      `implementation/handoff.md`.

15. **Next-phase framing inputs:** **organized**
    - `implementation/handoff.md` is finalized as the phase-close handoff.
      Confirmed DD-set residuals: the PM-2 → PM-1 / PM-3 pre-1.0
      wrapper-rule decision; the VS-2 / VS-3 `SlotData` carrier triggers;
      the Grid structural-mutation trigger (DD-M3-P7-006 recursive); the
      bindable / reactive placement trigger; default-alignment unification;
      placement key/value spelling revision; the FD-7b-D code-construction
      constraint.
    - Confirmed mid-phase-surfaced residuals (distilled from the T7
      candidate ledger, not in the ADR set): **Problem B —
      author-controllable `width` / `height` sizing** (Phase 8 framing
      Vision DR, hard backstop pre-1.0 / M6 ABI-freeze prep); the **Phase-8
      removal** of the placement-demo verification surface + capture driver;
      the `aspect`-in-cell arrange abort; the capture-driver layout-coupled
      coordinates. The Grid-as-ZStack-child checker reject is **resolved**
      (T6b) and not carried open.
    - Doc-folded material stays in the living docs rather than handoff
      prose: the `slot.*` author surface (`dsl_spec.md` §4.16), the
      child-slot `SlotData` storage and the IR member shape
      (`architecture.md` §6.7.9 / §6.8.4 / §6.8.6), and the textual-IR
      record (`dsl_spec.md` §8.5).
    - Phase 8 must **surface, not re-decide**, the PM-2 provisional two-form
      Grid state and the Problem B sizing Vision DR, so the public draft
      does not silently freeze either.

16. **CI green:** **green**
    - The phase branch `feat/m3-phase-7b` was pushed at head
      `59f9be6903349f37a2fe0f5f84e42e9653e9ca52`.
    - GitHub Actions workflow `CI`, event `workflow_dispatch`, run
      [28072510434](https://github.com/matarillo/wasamo/actions/runs/28072510434)
      concluded **success** (`2026-06-24T03:12:21Z` →
      `2026-06-24T03:16:14Z`).
    - Green CI steps: release workspace build, debug workspace build for
      tests, workspace tests, C ABI smoke (`cl` / `clang-cl`), CMake smoke,
      Zig binding smoke, `counter-c`, `counter-rust`, `counter-zig`, and
      `wasamoc check counter.ui`. This discharges the CI-gated cell of
      Phase-End Gate line (3) (the T3 Windows-runtime integration fixtures).
    - Phase-end local clean-rebuild ground truth is already recorded in
      `implementation/log.md` (T7 verification): `cargo fmt --all --
      --check` exit 0; `cargo clean` (6270 files / 1.7 GiB) → release
      workspace build (53.27s) → debug workspace build → `cargo test
      --workspace`, all exit 0. The CI workflow has no `cargo fmt` step, so
      fmt remains local evidence.

17. **Human-visible GUI smoke:** **needed; satisfied by the T6 gallery
    placement smoke**
    - Phase 7b changed runtime storage, `wasamoc` lowering, and
      gallery-visible placement, so human-visible smoke was needed. The
      relevant visible acceptance target is the placement-demo sub-screen +
      lightbox in the Rust gallery host.
    - The T6 owner-manual gallery smoke (2026-06-23) passed with no fix
      iteration: ZStack `slot.h-align` start / center / end at three
      distinct x-positions, Grid stretch-fill vs centered cells with
      distinct row/column/span, lightbox same-position corroboration, and
      crash-free close. The counter hosts (`counter-c` / `-rust` / `-zig`)
      carry no placement but exercise the migrated IR carrier / runtime
      child-slot loading on the base path; their CI smokes are the
      base-path regression gate (item 16).
    - No additional phase-end human-visible smoke is needed: the only
      post-T6 change is T6b (a `wasamoc check` accept/reject fix with no new
      GUI behaviour — the gallery keeps its VStack-wrap workaround) plus
      T7 / phase-end documentation work. C / Zig gallery hosts remain Phase
      8 full-gallery scope.

18. **CI YAML sanity check:** **no change required**
    - Phase 7b added Rust code (IR carrier, runtime/layout storage,
      `wasamoc` surface) and a gallery verification surface, but **no new
      language, external build system, or CI matrix axis**. Existing Windows
      CI already covers workspace build/test, the C / CMake / Zig binding
      smokes, and the counter host smokes. No `.github/workflows/ci.yml`
      update is required (AGENTS.md CI rules).

## Merge Readiness

Checklist items 12–18 are all recorded and the phase-branch CI gate is
**green** (run 28072510434). `implementation/handoff.md` is finalized and
`implementation/preamble.md` flips to `status: closing` in the same
phase-end batch commit that records the CI run id. Per the retrospective
procedure, this record is **evidence** for the phase → main merge gate, not
merge authorization: the phase → main no-ff merge requires explicit owner
approval, and push to main is a separate gate.
