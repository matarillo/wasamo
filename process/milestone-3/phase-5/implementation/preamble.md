---
phase: M3-Phase 5
title: Grid layout primitive
status: active
adr: process/milestone-3/phase-5/decisions/preamble.md
plan: process/milestone-3/plan.md
opened: 2026-05-29
---

# M3-Phase 5 — Grid layout primitive: Implementation

This is the live task list and execution framing for M3-Phase 5. The
design decisions are frozen in the ADR set under
[../decisions/](../decisions/preamble.md) (preamble + DD-M3-P5-001
through DD-M3-P5-006, Status: Accepted on 2026-05-28). This file and
its sibling [plan.md](./plan.md) are mutable during the phase per
[../../../README.md §SSOT distribution](../../../README.md#ssot-distribution);
the in-flight decisions log and CI evidence land in
[log.md](./log.md) as the phase progresses. Cross-phase residuals
land in [handoff.md](./handoff.md) at phase close.

## Phase 5 scope

Phase 5 satisfies milestone acceptance criterion **A2** by adding a
`Grid` layout primitive that arranges children across a declared
row × column track matrix:

- **Surface A2** (DD-M3-P5-001) — `Grid` declares tracks once via
  `columns:` / `rows:`; each child is wrapped in a `Cell` carrying
  explicit zero-based `row` / `column` placement, optional
  `row-span` / `column-span`, and optional per-cell `h-align` /
  `v-align`. Content widgets carry no Grid-specific metadata. `Cell`
  is IR-only — it does not appear in the runtime widget catalog and
  does not materialise as a WidgetNode or Visual.
- **Track sizing** (DD-M3-P5-002) — fixed integer pixels and
  weighted-star tokens (`n*`, `n in [1, 1024]`). `auto` / intrinsic,
  `minmax`, floating-point weights, and named lines are deferred
  (reserved-future diagnostic at `wasamoc check` for `auto`).
- **Both-axis spanning** (DD-M3-P5-003) — `row-span` and
  `column-span` both default to `1`. Same-cell / overlapping-
  rectangle conflicts are rejected; intentional overlay is Phase 6
  ZStack's responsibility.
- **Track resolution** (DD-M3-P5-004) — pure-data fixed-first /
  weighted-star distribution over `f32` prefix boundaries; star
  tracks meeting an unbounded parent axis raise
  `LayoutError::GridUnboundedStarAxis` (host-internal, no new ABI
  tag). A no-op slot is reserved before star distribution for a
  future `auto` demand pass.
- **Arrange / overflow / z-order** (DD-M3-P5-005) — stretch /
  stretch default with `h-align` / `v-align` overrides; Grid
  outer-bounds clip on Grid's own Visual
  (`Visual.Clip = InsetClip { 0, 0, 0, 0 }`); paint order is
  document order; no intermediate Visual (the 1 WidgetNode = 1
  Visual convention is preserved).
- **IR-loader invariants** (DD-M3-P5-006) — all structural
  invariants are dual-gated at `wasamoc check` and runtime
  `validate()` and surface `WASAMO_ERR_IR_MALFORMED`. All
  placement / span / conflict checks are **reject-at-validate**, not
  clamp-at-arrange; the only layout-time gate is
  `LayoutError::GridUnboundedStarAxis`.

Same-cell overlap, per-cell clipping, `auto` / `minmax` / named-
lines / template-areas grammar, bindable Grid attributes, iteration-
template-generated `Cell`s, drag-resizable splitters, an author-
facing `z-index:`, a Grid-level `fill:`, and the `TypedValue`
generic value union are out of Phase 5 scope; see
[../decisions/preamble.md §Out of scope](../decisions/preamble.md#out-of-scope)
and §Post-Phase-5 hand-off for the explicit deferrals and their
forward-compat landing points.

## Verification closure

The automated A2 evidence set is items (1)-(4) in the ADR's
[Phase 5 verification closure](../decisions/preamble.md#phase-5-verification-closure-what-counts-as-a2-evidence)
section:

- **(1) `wasamoc check` compile-time evidence** — Surface lowering
  positive controls, track-list shape rejection, placement / span
  value rejection, Cell single-child rejection, same-cell / span
  conflict rejection, unknown Grid / Cell attribute rejection.
- **(2) Measure-arrange unit-test evidence** — Fixed-only / weighted-
  star / mixed / both-axis spanning / negative remaining / unbounded
  star-axis / per-Cell alignment / Grid outer-bounds clip presence /
  document-order z-order coverage of the pure-logic algorithm.
- **(3) IR-loader `validate()` invariant evidence** — Min row /
  column count, track value range, placement value range, span value
  range, same-cell / overlapping-span conflicts, Cell child-count.
- **(4) Windows-runtime layout evidence (CI-gated)** — Grid-rooted
  integration fixture **and** `VStack { Grid { ... } }`-rooted
  fixture (production-root coverage per Phase 4 T6 carry-forward).
  Both assert Grid's resolved rectangle, per-Cell content Visual
  offsets, Grid outer Visual clip **presence**, and Cell content
  Visual clip **absence**. Unbounded star-axis runtime fixture is
  preferred when ergonomic; otherwise the unbounded-parent case
  downgrades to pure-logic coverage in (2) per FD-C.

The phase-close / A11 gallery proof is item (5):

- **(5) End-to-end host evidence (visible smoke)** — gallery.ui
  grows additively with a Grid composition matching the **FD-H
  minimum visible-proof shape** (3-row × 3-column Grid with at
  least five Cells: one spanning header, three middle-row Cells in
  separate columns, one spanning footer; fixed + at least one star
  track; column-span exercised). The assistant builds and launches
  `examples/gallery-rust/` and records `Start-Process` success.
  Visual correctness is **owner-manual GUI smoke** per FD-I; the
  assistant does not assert on pixel- or eyeball-level correctness.
  Row-span is discharged by items (2)-(4) per FD-C, not by the
  visible proof.

Per FD-C "evidence lines do not collapse", items (1)-(4) each carry
distinct evidence meanings and do not merge even when they share
helper infrastructure; items (5) further splits across T5
(assistant-automated build / launch) and T6 (owner-manual GUI smoke
with any visible-correctness fix iteration absorbed in-step) per
FD-I.

## Step-end / phase-end retrospective split (constraints §5)

Per
[../requirements/constraints.md §5](../requirements/constraints.md#5-phase-最終-step-の-retrospective--progress-checklist-は-step-end-と-phase-end-を分割する)
(FD-G carry-forward from Phase 4 T7), the final-step retrospective
is split from the start in this plan, not after the fact:

- The **T7 step-end retrospective** (retrospectives.md checklist
  items 1-11; step → phase merge gate) is **owned by T7** and is a
  T7 deliverable. T7's checkbox flips to `[x]` when this retro is
  recorded.
- The **phase-end retrospective** (retrospectives.md checklist
  items 12-18; phase → main merge gate) is **NOT owned by T7**.
  It is recorded on the phase branch after T7 merges in, by a
  separate retro commit, and is the precondition for the
  phase → main merge gate. The corresponding T7 plan bullet
  **stays `[ ]` at T7 close** and is checked by the phase-end retro
  commit on the phase branch.

This split exists so the reviewer's mental model of "T7 closes the
phase" matches the operational reality of "T7 closes the step, the
phase-end retro closes the phase". If Phase 5 runs through this
structure without friction, the phase-end retro for Phase 5 may
promote constraints §5 into the project-wide
`process/procedures/retrospectives.md` rule set; that promotion is
explicitly deferred to Phase 5 close and not pre-decided here.

## Lifecycle transition

This implementation file opens at `status: active` and transitions
to `status: closing` at T7 step-close in the same commit that flips
T7's checkbox. The file remains mutable during the `closing` window
only for the phase-end retrospective evidence pointer, the CI run
pointer, and any final post-merge distillation; no further task
checkboxes are added once T7 closes. The phase-end retrospective
itself is recorded under
`process/milestone-3/phase-5/retrospectives/phase-end.md` (with
sibling `tN.md` per-step retros under the same directory) and is a
separate commit on the phase branch — it is not a T7 deliverable
per the split above.

Per
[../../../procedures/retrospectives.md](../../../procedures/retrospectives.md),
implementation start is gated on Moment 1 commit set completion
(ADR Accepted; dsl_spec §4.12 + §4.4 + §8.11; architecture §6.8.7;
m3-plan.md Phase 5 row; this implementation/preamble.md + plan.md).
At C3 land time, the Moment 1 commit set closes and T1 may open.

## Technical risks (planning-time recon)

Spot-checked against the current source on 2026-05-29 to surface
risks where ADR-level assumptions sit further from the existing
abstractions than the ADR text implies. Recorded here so the
[plan.md](./plan.md) tasks that mitigate them are explicit pre-
implementation spikes, not implicit discoveries; revisit and close
each risk in [log.md](./log.md) as it lands or evolves.

| ID | Risk | Severity | Source pin | Mitigation |
|---|---|---|---|---|
| R-A | `wasamoc` parser receives Grid `columns:` / `rows:` track lists as whitespace-separated multi-token sequences (`180 1* 2*`), but the existing `parse_property_bind` reads a single `Expr` and `parse_expr` only admits single tokens (`IntLit` / `StringLit` / `Ident` / `Bool` / `FloatLit` / `Measurement`). The narrow Grid-specific track-list parser path (DD-M3-P5-002) requires `parse_widget_decl`-level context routing and may require a lexer token for `n*`. | **High** | [`wasamoc/src/parser.rs:233`](../../../../wasamoc/src/parser.rs#L233) (`parse_property_bind`), [`wasamoc/src/parser.rs:381`](../../../../wasamoc/src/parser.rs#L381) (`parse_expr`) | [plan.md T1 pre-implementation spike](./plan.md#t1--wasamoc-check-grid-surface-and-diagnostics) |
| R-B | `ir_loader::build_node` walks every IR child through a generic `for child in &node.children { build_node(child); widget.append_child(...) }` loop, and `construct_widget` matches on `widget_type` then falls through to `UnknownWidget`. ADR DD-M3-P5-001's "Cell is IR-only, not in the runtime widget catalog, never materialises as a WidgetNode or Visual" requires the **Grid path in `build_node`** to bypass the generic child append loop and flatten IR Cell subtrees into `WidgetNode` children + a per-Cell `Vec<CellPlacement>` on `WidgetData::Grid`. `construct_widget`'s Grid arm only creates the Grid widget shell; the Cell flattening is the special case at the `build_node` layer, beyond the Phase 2-4 per-kind constructor pattern. | **High** | [`wasamo-runtime/src/ir_loader.rs:954`](../../../../wasamo-runtime/src/ir_loader.rs#L954) (`build_node`), [`wasamo-runtime/src/ir_loader.rs:1016`](../../../../wasamo-runtime/src/ir_loader.rs#L1016) (`construct_widget`) | [plan.md T3 pre-implementation spike](./plan.md#t3--ir-loader--validate-invariant-evidence) |
| R-C | Adding `kind_payload: Option<KindPayload>` to `IrNode` per DD-M3-P5-001 carrier c1 forces every existing `IrNode { ... }` struct-literal construction site to update (no `Default` impl, no `..Default::default()` pattern in current code). Affects `wasamoc` emit, `wasamo-runtime` ir_loader, and the unit-test corpus. ADR notes this as "minor" on allocation overhead but does not address construction-site spread. Possible mitigations: derive `Default`, introduce `IrNode::new(widget_type)` builder, or accept the broad construction-site edit. | Medium | [`wasamo-ir/src/lib.rs:115`](../../../../wasamo-ir/src/lib.rs#L115) (`IrNode`) | Settled inside [T1](./plan.md#t1--wasamoc-check-grid-surface-and-diagnostics) / [T3](./plan.md#t3--ir-loader--validate-invariant-evidence) implementation; record the chosen construction-site discipline in [log.md](./log.md). |
| R-D | `LayoutNode` is a flat struct sharing all widget-kind fields (Box `aspect`, WrapPanel `item_cross_size` / `item_spacing` / `line_spacing` / `wrap_measured_cross_bound`, ScrollView `offset_y` / `applied_offset_y`). Grid adds `Vec<TrackSize>` × 2 + `Vec<CellPlacement>` + likely an arrange-result cache, which is heavier than the Phase 2-4 scalar additions. Continuing the flat-struct pattern is the lowest-friction path; an enum-shaped `LayoutNode` refactor is **out of Phase 5 scope** unless the flat extension proves untenable. | Medium | [`wasamo-runtime/src/layout.rs:83`](../../../../wasamo-runtime/src/layout.rs#L83) (`LayoutNode`) | [plan.md T2 mitigation bullet](./plan.md#t2--layout-engine-grid-track-resolution-and-arrange); record the field shape in [log.md](./log.md). |

Low-severity items confirmed during recon (no mitigation needed,
listed here for traceability):

- `LayoutError::GridUnboundedStarAxis` add + match expansion is a
  direct extension of Phase 4's `LayoutError::ScrollViewUnboundedAxis`
  pattern.
- `validate_phase5_grid_invariants` is a direct extension of the
  `validate_phase2_node_invariants` / `_phase3_` / `_phase4_`
  pattern in [`wasamo-runtime/src/ir_loader.rs:191-291`](../../../../wasamo-runtime/src/ir_loader.rs#L191).
- `WidgetNode::run_layout_as_window_root` is WidgetKind-agnostic
  ([`wasamo-runtime/src/widget.rs:1325`](../../../../wasamo-runtime/src/widget.rs#L1325)),
  so a Grid-rooted fixture rides the same entry point as
  ScrollView's T6 fix.

## Cross-references

- ADR set: [../decisions/preamble.md](../decisions/preamble.md) +
  DD-M3-P5-001 through DD-M3-P5-006.
- Phase 5 framing decisions: [../requirements/framing.md](../requirements/framing.md)
  (FD-A through FD-K).
- Phase 5 carry-forward inputs from Phase 4:
  [../requirements/constraints.md](../requirements/constraints.md)
  (§1 production root shape; §2 non-root Shrink × Fill — out of
  Phase 5 thesis per FD-D; §3 M4 handoff; §4 R1 owning-phase
  assignment → Phase 6 per FD-E; §5 step-end / phase-end split —
  applied in this plan from the start).
- Specification chapter (design-spec draft): [`docs/dsl_spec.md`
  §4.12](../../../../docs/dsl_spec.md) (Phase status: `M3-Phase 5
  design accepted; implementation pending` until T7 Moment 2 flip).
- Architecture entry: [`docs/architecture.md`
  §6.8.7](../../../../docs/architecture.md) Phase 5 paragraph and
  top Status header (top Status flips to `M3-Phase 5 complete` at
  T7 Moment 2).
- ABI: [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — no
  touch in Phase 5 per ADR (Grid adds no host-facing ABI surface;
  `LayoutError::GridUnboundedStarAxis` is host-internal).
