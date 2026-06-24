---
phase: M3-Phase 7b
title: Parent-interpreted placement attributes
status: closing
adr: process/milestone-3/phase-7b/decisions/preamble.md
plan: process/milestone-3/plan.md
opened: 2026-06-21
closing: 2026-06-24
---

# M3-Phase 7b — Parent-interpreted placement attributes: Implementation

This is the live task list and execution framing for M3-Phase 7b. The
design decisions are frozen in the ADR set under
[../decisions/](../decisions/preamble.md) (preamble + DD-M3-P7b-001 and
DD-M3-P7b-002, both `Status: Accepted` on 2026-06-21 after the PM-2
integration review). This file and its sibling [plan.md](./plan.md) are
mutable during the phase per
[../../../README.md §SSOT distribution](../../../README.md#ssot-distribution);
the in-flight decisions log and CI evidence land in [log.md](./log.md)
as the phase progresses. Cross-phase residuals land in
[handoff.md](./handoff.md) at phase close.

## Phase 7b scope

Phase 7b is a **corrective phase** (no new layout primitive, no new app
feature). It aligns the parent-interpreted placement surface that
Phases 5–7 shipped piecemeal, **before** the Phase 8 public draft
freezes it. The two Accepted DDs decide *one* surface and *one*
internal model and Phase 7b implements them across parser / checker /
lowering / textual IR / loader / runtime / examples (FD-7b-E — not docs
alone). The decisions and their rationale are frozen in the
[ADR §Decisions table](../decisions/preamble.md#decisions); this list
is only *what each task builds* (DD pointers, not a re-derivation):

- **Author surface (DD-M3-P7b-001 — A13)** — CB-B (generalizable
  parent-data, scoped to layout this phase) + Option 3 fixed prefix,
  token **`slot.`** (SI-1); **PM-2** — Grid admits **both** `Cell`
  (grouped sugar) *and* direct `slot.*` on the child, one form per child
  (strict mixing reject); ZStack moves to `slot.h-align` /
  `slot.v-align` (bare `h-align` / `v-align` rejected — no long-lived
  alias); CF-1 (placement on the body's root child). No normative
  canonical Grid form this phase; `Cell` is the provisional examples
  convention, resolved at the pre-1.0 wrapper-rule gate.
- **Internal model (DD-M3-P7b-002)** — **IM-4** child-slot-carried
  placement, **phase-wide** (DD-M3-P7-006 verb = **revise**); **SM-B**
  the Grid `cell_placements` parallel vector migrates onto the child
  slot this phase; **VS-1 / VS-1a** one slot field carrying a closed
  payload enum, the carrier broadly named **`SlotData`** (`SlotData::{
  Grid(..), ZStack(..) }` today; additive enum → struct at the first
  non-layout parent-data, no rename) under the CB-B integration
  condition; **IR-2 + IR-B** the IR carries an explicit child-slot
  record (`IrMember::Widget(IrChildSlot)` with
  `IrChildSlot { node, slot_data }` + `IrSlotData`), textual IR
  normalises all three authored forms to the `child { placement <kind>
  { … } node … }` skeleton, stale `Cell` / bare-placement IR is **reject
  + regenerate** with a named loader diagnostic. Placement is constant
  per instance (binding RHS rejected).

Out of scope — bindable / reactive placement; the PM-2 → PM-1 / PM-3
pre-1.0 wrapper-rule decision; Grid structural mutation paths (`for` /
`if` of `Cell`s — storage migrates, the mutation surface does not);
default-alignment unification (Grid `stretch` / ZStack `center` stay
per-container); key/value spelling revision; the shared placement enum
(VS-2) and extensible record (VS-3); generic modifier system;
user-defined containers / custom slot attrs; non-layout parent-data;
keyed identity; any public code-construction API / ABI
([../decisions/preamble.md §Scope and out of scope](../decisions/preamble.md#scope-and-out-of-scope);
the deferred-items **正本** with activation triggers is the framing
scope table
[../requirements/framing.md §Out of scope](../requirements/framing.md)).

## Acceptance relation (branch (a) — A13)

DD-M3-P7b-001 resolved to **branch (a)**: PM-2 changes the public
author surface for both containers, so a new acceptance criterion
**A13** ("parent-interpreted placement authoring surface") was added
under the M3 acceptance-criteria revision exception and mirrored into
[../../../_roadmap.md](../../../_roadmap.md) and the
[../../plan.md Revision log](../../plan.md). A13 states the **accept-set**
(Grid accepts `Cell` and `slot.*`; ZStack `slot.*`), **not** the
provisional `Cell`-default examples convention. **A11** (`.ui` /
`wasamo-ir` / `wasamoc` / `wasamo-runtime` / `docs/dsl_spec.md` /
`examples/gallery/` sync) and **A12** (public-draft explicability) apply
throughout.

## Verification closure

The ADR's
[§Verification closure](../decisions/preamble.md#verification-closure-what-counts-as-phase-7b-evidence)
fixes the five evidence lines and their content, including the positive
controls (a single static frame a wrong implementation could equally
produce is **not** evidence). This plan adds only the *where* (task
mapping); the per-item assertions live in the ADR / DD spec-impact
sections and are referenced — not re-stated — by the owning task:

| ADR evidence item | Task(s) |
|---|---|
| (1) `wasamoc check` (positive + the full reject matrix / forcing table) | T4 |
| (2) lowering / textual-IR roundtrip / loader rejection | T2 (emit + loader parse + stale-form reject) |
| (3) Windows-runtime integration (CI-gated, fail-not-skip): layout reads placement from child-slot storage; insert/remove preserves order + Visual sibling order + placement + invalidation; destroy leaks no placement metadata | T3 |
| (4) assistant-visible GUI + positive control (ZStack `slot.h-align: end` right-aligned + a contrasting alignment frame; Grid row/column/span/alignment reflected; stray/omitted → default) | T5 (assistant) + T6 (owner GUI smoke) |
| (5) A12 spec-closure gate (`docs/dsl_spec.md` placement chapter + `docs/architecture.md` model; Moment 1 → Moment 2 marker flip) | T7 |

The Windows-runtime fixtures (item 3) fail — not skip — on a runner
that cannot create the Compositor; the skip-guard inherits the Phase
2–7 pattern (`0x80070005` from `wasamo_init`), and multi-test binaries
reuse the Phase 6 keep-alive apartment helper.

**Positive-control discipline:** the placement proof is **same-position
re-rendering under the new model** (the old surface and the `slot.*`
surface land children at the same place) **plus a contrast** (a
different alignment lands at a *different* position). For Grid, stray /
omitted placement falling to the per-container default (`stretch`) is the
contrast against an explicit alignment.

## Obligations carried from the ADR (represented in this plan from the start)

1. **First task: pre-implementation spike** — T1 reads every landing
   file named by the task list, **compiler-verifies** the
   `IrSlotData` / `SlotData` carrier migration (throwaway variant →
   build → revert, not grep), and **records** the carrier spelling,
   the bisectable sequencing, the trap-#1 call-site hotspots, and the
   T2 impl-gates selection in [log.md](./log.md) *before* T2 opens. The
   DD-fixed shapes (`SlotData` broad name; VS-1a closed enum; IR-B
   skeleton) stay fixed; T1 decides only the in-memory / IR spelling the
   DDs left as an implementer recommendation
   ([spike discipline](../../../procedures/implementation-gates.md): exit
   criterion is "every open point is assigned to a downstream task and
   its scope is seen", not "no surprises expected").
2. **Bisectable sequencing** — T1 fixes (and records in [log.md](./log.md))
   the order of the IR carrier migration (T2), the runtime `SlotData`
   migration (T3), and the author-surface flip (T4) so intermediate
   commits stay buildable. The default order encoded in this plan is
   T2 (IR + textual IR, loader → legacy-runtime adapter seam) → T3
   (runtime `SlotData`, adapter removed) → T4 (`slot.` author surface +
   in-repo `.ui` migration). T1 may revise it with reasons.
3. **Parallel-data drift is the central trap (#3)** — the migration's
   reason for existing is to remove the child-list-vs-placement drift
   class *by construction* (IM-4). T3's close artifact is the structural
   proof that **no parallel placement vector remains on any mutated
   path** (`cell_placements` migrated; `zstack_placements` already gone)
   — greppable — plus the splice side-effect re-enumeration for the Grid
   path (trap #2).

## Step-end / phase-end retrospective split (final-task ownership)

Per the inherited final-task ownership split (Phase 6 / Phase 7
phase-end learning), the final-task split is represented from the start:

- The **final-task (T7) step-end retrospective**
  ([retrospectives.md](../../../procedures/retrospectives.md) items
  1–11; step → phase merge gate) is **owned by T7** and is a T7
  deliverable. Local `cargo fmt` / clean-rebuild evidence returns to T7
  ownership because T7 follows production-Rust tasks.
- The **phase-end retrospective** (items 12–18; phase → main merge
  gate), the **phase-branch CI run id**, the **handoff finalization**,
  and this file's **status flip** are **NOT owned by T7**. They land on
  the phase branch after T7 merges in, by separate phase-end commits,
  and are the precondition for the phase → main merge gate. The
  corresponding T7 plan bullets **stay `[ ]` at T7 close** and are
  checked by the phase-end commits.

Before the final task closes, the T0-frozen task list is cross-checked
against any mid-phase owner decisions and the mutable phase plan is
revised where they diverge (revise, do not work around).

## Lifecycle transition

This implementation file opens at `status: active` and transitions to
`status: closing` at the **phase-end batch commit** — the phase-branch
commit that records the CI-verified on-CI gates (the GitHub Actions CI
run id), finalizes `handoff.md`, appends the CI evidence to `log.md`, and
flips this front-matter `status` — **not** at T7 step-close. The Moment 2
spec / architecture docs sync (`docs/dsl_spec.md` §4.16 / §8.5 / §8.11 +
`docs/architecture.md` §6.7.9 / §6.8.4 / §6.8.6 markers) and the M3
`plan.md` Phase 7b row flip are **T7-owned and land at T7 step-close**
(with the T7 step-end gates + candidate ledger in `log.md`); the
phase-end batch does **not** re-do those flips. The on-CI gates are
phase-end-owned and verified only after the phase branch runs
`workflow_dispatch` CI, so **T7 step-close itself leaves
`status: active`**. The phase-end retrospective is a separate commit in
the same phase-end cluster, recorded under
[../retrospectives/phase-end.md](../retrospectives/) (with sibling
`tN.md` per-step retros). The `closing` → `retired` transition belongs
to the phase → main merge / post-merge distillation.

Implementation start is gated on Moment 1 commit-set completion (ADR
Accepted; `docs/dsl_spec.md` §4.16 placement chapter + §8.5 / §8.11
supporting additions at v1.10; `docs/architecture.md` §6.8.6 `SlotData`
storage + §6.8.4 Grid SM-B; `process/_roadmap.md` A13 + m3 `plan.md`
Phase 7b row; this implementation `preamble.md` + `plan.md`). At T0 land
time, the Moment 1 commit set closes and T1 may open.

## Implementation gates

Every implementation task runs
[implementation-gates.md](../../../procedures/implementation-gates.md)
at **task start and task close**: record the selected failure-mode
gates (with reasons for non-applicable ones) in [log.md](./log.md)
*before* choosing an approach, and close with the auditable artifacts.
Known phase-wide gate load (from the ADR
[§Implementation gate expectations](../decisions/preamble.md#implementation-gate-expectations)):

- **Trap #1 (semantic migration / call-site audit)** — the
  `IrMember::Widget(IrNode)` → `IrMember::Widget(IrChildSlot)` carrier
  change (T2) and the runtime `zstack_placement` + Grid
  `cell_placements` → unified child-slot `SlotData` change (T3): the close
  artifact is the `rg`-enumerated match-site table over `IrMember` /
  `IrNode` placement extraction / `WidgetData::Grid` / arrange-loop
  call-sites, with `IrNode::widget_children()` and every widget-only /
  placement-filter helper classified. Prefer a compile-error-forcing
  carrier shape so Rust enumerates the breakage.
- **Trap #2 (structural side-effect enumeration)** — the Grid-path
  splice / insert / remove / replace side-effect bundle (T3): child list splice
  (placement riding along), Visual sibling order, layout invalidation,
  widget-pointer registry, effect ownership — checked off per change,
  re-using the DD-M3-P7-006 enumeration restated for the Grid path.
- **Trap #3 (parallel data drift)** — the central trap; closed
  structurally by IM-4 (T3). **There are two parallel vectors, not
  one:** `WidgetData::Grid.cell_placements` *and* the layout mirror
  `LayoutNode.cell_placements` (consumed by `arrange_grid`'s
  `children.zip(cell_placements)`, populated by `build_layout_tree`).
  The close artifact has an **independent audit row per site** —
  `WidgetData::Grid.cell_placements`, `LayoutNode.cell_placements`,
  `LayoutNode::grid`, `arrange_grid`, `build_layout_tree` — each shown
  migrated to the child slot or deleted (greppable: no `cell_placements`
  / `zstack_placements` survives a mutated path), plus the
  no-placement-container `None` invariant. (A preamble-only review must
  not drop the layout mirror — see [plan.md](./plan.md) T3.)
- **Trap #4 (untested authored branch)** — every DD-001 §Spec-impact
  forcing-table row (the 8-row `slot.*` accept/reject table, the mixing
  reject, the non-admitting-parent reject, the unknown-key reject, the
  constant-RHS reject, the value-namespace rule) fires a direct test
  (T4); the loader stale-form / malformed-placement rejects fire direct
  tests (T2).
- **Trap #7 (GUI positive control)** — T5 assistant screenshot +
  analysis + a contrasting-alignment positive control.
- **Trap #5 (carry-forward)** — the pre-1.0 wrapper-rule decision, the
  VS-2 / VS-3 carrier triggers, the Grid structural-mutation trigger,
  and the bindable-placement trigger are recorded with re-trigger
  criteria (T7 → phase-end handoff).
- **Review tiers** — T2 (IR / schema migration) and T3 (runtime
  structural change) take the **full independent review**; T4 (author
  surface / reject branches) takes the **branch/test-focused review**;
  T5 (GUI-render evidence) takes the **full independent review** (GUI
  high-risk class). If a full-review task also adds reject branches, the
  full review includes the trap #4 branch/test check.

## Technical risks (planning-time recon)

The per-DD §Risk-mitigation sections carry the full set; the plan-level
top risks and their mitigating tasks (T1 sharpens this against the
current source before T2 opens):

| ID | Risk | Mitigation |
|---|---|---|
| R-A | **The IR carrier change is compile-error-forcing.** `IrMember::Widget(IrNode)` → `IrMember::Widget(IrChildSlot)` breaks every match on the tuple variant across `wasamo-ir`, `wasamoc` (lower / emit / check), and the runtime loader; the workspace will not build until all sites migrate together. | T2 bundles the `IrChildSlot` carrier + emit (IR-B) + loader parse + stale-form reject as one buildable commit (AGENTS.md §Commit rules); call-site audit artifact at close; the wrapper shape forces every site to classify whether it needs the slot record or only `slot.node`. |
| R-B | **The runtime `SlotData` migration touches shipped Grid arrange (Phase 5) and ZStack (Phase 6/7) paths.** Grid `cell_placements` is a parallel vector read in `build_layout_tree` / `arrange_grid`; ZStack reads `zstack_placement`. | T3 is its own commit; Phase 5 Grid fixtures (track sizing, spanning, membership/conflict, arrange overflow) + Phase 6/7 ZStack fixtures are the regression gate; trap #3 greppable close. |
| R-C | **The author-surface flip is breaking.** ZStack bare `h-align` / `v-align` become named rejects (no long-lived alias), so every in-repo `.ui` using bare ZStack alignment stops compiling. | T4 bundles the in-repo `.ui` migration (gallery + examples + test fixtures) in the same commit so the build stays green; the sweep is bounded and greppable; the bare-form reject ships with a firing test. |
| R-D | **PM-2 two-form Grid surface — mixing reject vs non-admitting-parent reject are easily conflated.** `slot.*` among a `Cell` node's own attrs and `slot.*` on a widget *inside* a `Cell` are **two distinct** named diagnostics. | The DD-001 §Spec-impact 8-row forcing table is the checklist; each row a direct firing `wasamoc check` test; the two Cell-related rejects are separate test names. |
| R-E | **`slot.` dotted-key lexeme vs expression-grammar qualified access.** A `slot.h-align` key must not be parsed as an expression member-access, and the placement value namespace (`end`) must not resolve through state. | Parser reads `slot.` + key as a dotted property key, not a member-access expr (T4); a test pins `slot.h-align` is never an expression; the value-namespace test pins `end` is the placement keyword even when a state named `end` exists. |
| R-F | **IR-B reject + regenerate must reject, not silently slot-ise, stale IR.** A `Cell` node with placement `IrProp`s or bare ZStack placement props in textual IR must be a named loader reject. | T2 ships the `legacy-placement-ir-form`-style named loader diagnostic with a firing test; roundtrip fixtures pin the single canonical `child { placement <kind> { … } node … }` shape. |
| R-G | **CF-1 placement on `for` / `if`-generated children.** Generated children must carry `SlotData` through staging → commit (Phase 7 path) under the migrated model. | T3 reuses the Phase 7 staging → commit path; a ZStack-under-`for` placement integration fixture proves the generated child carries placement; this is storage only — no Grid mutation path is built (DD-002 §Out of scope). |

## Cross-references

- ADR set: [../decisions/preamble.md](../decisions/preamble.md) +
  [DD-M3-P7b-001](../decisions/dd-m3-p7b-001-parent-interpreted-placement-authoring-surface.md)
  +
  [DD-M3-P7b-002](../decisions/dd-m3-p7b-002-placement-internal-model-and-construction-boundary.md).
- Phase 7b framing:
  [../requirements/framing.md](../requirements/framing.md) (FD-7b-A …
  FD-7b-F; deferred-items **正本** scope table; verification strategy).
- Revised baseline:
  [../../phase-7/decisions/dd-m3-p7-006-placement-storage-and-structural-side-effects.md](../../phase-7/decisions/dd-m3-p7-006-placement-storage-and-structural-side-effects.md)
  (DD-002 verb = revise).
- Specification (Moment 1 design draft, marker flips at T7 Moment 2):
  [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) §4.16 placement
  chapter + §8.5 / §8.11; [`docs/architecture.md`](../../../../docs/architecture.md)
  §6.7.9 member-level structural IR + §6.8.6 `SlotData` storage +
  §6.8.4 Grid (SM-B).
- ABI: [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no touch**
  (FD-7b-D); the future-API non-committal constraint lives in
  `architecture.md` prose, not the ABI. If implementation surfaces an
  unavoidable ABI need it is recorded at Moment 2 with owner
  confirmation.
- `docs/notes/architectural-family.md` — the FD-7b-C
  confirm-within-family (1) entry lands revise-in-place at **Moment 1 or
  Moment 2** (T7).
- Landing source (T1 reads all): [`wasamo-ir/src/lib.rs`](../../../../wasamo-ir/src/lib.rs)
  (`IrMember` / `IrNode` / `KindPayload`),
  [`wasamo-runtime/src/widget.rs`](../../../../wasamo-runtime/src/widget.rs)
  (`WidgetNode.zstack_placement`, `WidgetData::Grid.cell_placements`,
  insert/remove/replace),
  [`wasamo-runtime/src/layout.rs`](../../../../wasamo-runtime/src/layout.rs)
  (`ZStackPlacement` / `CellPlacement` / `LayoutNode::grid`),
  [`wasamo-runtime/src/ir_loader.rs`](../../../../wasamo-runtime/src/ir_loader.rs)
  (Grid `Cell` extraction, ZStack placement),
  [`wasamoc/src/`](../../../../wasamoc/src/) (lexer / parser / ast /
  check / lower / emit — `ast.rs` `Member::PropertyBind` is where the
  `slot.` dotted-key storage decision lands, T1).
