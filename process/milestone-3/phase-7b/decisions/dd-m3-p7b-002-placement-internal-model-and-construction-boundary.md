# DD-M3-P7b-002 — Placement internal model and construction boundary

**Status:** Proposed
**Phase:** M3-Phase 7b
**AC:** A11 (IR / runtime / loader sync); discharges the framing
parallel-data-drift obligation with implementation gates #2 / #3.
Storage changes are not author-visible, so the contingent new-AC
question (preamble §Acceptance relation) is owned by
[DD-M3-P7b-001](./dd-m3-p7b-001-parent-interpreted-placement-authoring-surface.md),
not this DD.

## The question

How should parent-interpreted placement metadata — once written in the
DSL (DD-001) — be represented across IR, textual IR, runtime storage,
and structural mutation, and what direction (not API) is recorded for a
future code-construction surface?

## Decision dependency summary

Consumes DD-M3-P7b-001 (conceptual boundary + author-visible
bindability read). Re-opens and re-connects
[DD-M3-P7-006](../../phase-7/decisions/dd-m3-p7-006-placement-storage-and-structural-side-effects.md),
which decided child-carried placement (ST2) for ZStack and **deferred
Grid** behind a trigger, under the assumption that the author surface
was unchanged — an assumption Phase 7b reopens. The **first thing this
DD fixes is the relationship to DD-M3-P7-006**: consume / revise /
supersede / split (framing R3).

## Current state consumed (verified at drafting)

- ZStack: **child-carried** placement (Phase 7 ST2); the parallel
  `zstack_placements` vector was removed (architecture.md §6.8.5).
- Grid: **parallel** `cell_placements` vector, static-only; migration
  held behind DD-M3-P7-006's trigger (Grid rejects direct `for`).
- The DD-M3-P7-006 splice primitive owns the structural-mutation
  side-effect set (child list, placement, layout dirty, Visual sibling
  order, registry, effects) as one composed operation.
- Placement is literal / constant; no host write / replace API; no
  code-construction surface in the stable ABI beyond handle-based tree
  mutation (abi_spec).

## Sub-issue 1 — Relationship to DD-M3-P7-006 (decided first)

- **Consume.** Treat DD-M3-P7-006 as the settled ZStack baseline and
  build only the Grid / surface re-connection on top. Cleanest if the
  DD-001 surface choice does not disturb the storage contract.
- **Revise.** Keep DD-M3-P7-006's child-carried thesis but extend it:
  migrate Grid in-phase and/or restate the contract in terms of the
  DD-001 surface. Likely if DD-001 picks a unified surface (CB-B) and
  Grid is pulled in.
- **Supersede.** Replace DD-M3-P7-006's storage decision if Phase 7b's
  surface choice forces a different model. Reserved for a pivot; not
  expected.
- **Split.** DD-M3-P7-006 keeps structural-mutation atomicity; this DD
  owns the placement *representation* re-connection.

The recommendation states which verb is taken and why, so the two DDs
do not hold the same decision twice (framing R3).

## Sub-issue 2 — Internal model

Child-slot-carried is the **leading hypothesis** (it removed the
observed Phase 6 drift class structurally and is already shipped for
ZStack), but it is **not** a default; the alternatives are compared on
merit (owner prior).

- **IM-1 — widget property model.** Store placement as an ordinary
  property on the child widget node.
  - Rejected on the thesis: it makes placement an *intrinsic widget
    property*, contradicting FD-7b-A's settled floor. Listed for
    completeness.
- **IM-2 — parent parallel metadata.** The current Grid model: a
  placement vector kept parallel to `children`.
  - Merit: zero migration for Grid; contiguous reads in arrange. Cost:
    the **observed** drift class (Phase 6, implementation-gates trap
    #3) — child list and placement vector desynchronise on insert /
    remove / splice / reorder, policed by helper discipline rather than
    structure.
- **IM-3 — encapsulated SoA + splice-only mutation.** Keep the parallel
  representation but behind a module boundary; all structural mutation
  enters one splice primitive.
  - Merit: removes the *bypass* drift class by language visibility,
    zero data-model migration. Cost: preserves the representational
    split (placement stored apart from the child it places); staged
    subtrees still carry placement beside children until commit; the
    splice signature keeps parallel-vector bookkeeping. This is the
    fair structural SoA steel-man (it is DD-M3-P7-006's ST1').
- **IM-4 — child-slot-carried placement (leading hypothesis).** The
  child slot carries the node plus its optional parent-interpreted
  placement (`None` for placement-free containers).
  - Merit: the drift class is removed **by construction** — a child and
    its placement are one record, so no insert / remove / splice /
    reorder can desynchronise them; the splice signature shrinks
    (children in, children out); generated subtrees carry placement
    through staging → commit as ordinary data; storage matches the
    authored parent-interpreted model without making placement an
    intrinsic widget property. Already shipped for ZStack.
  - Cost: a runtime structural migration of the Grid arrange / loader
    read path (full-review lane); the concrete value space (one enum
    vs per-container child-entry type) is an implementation choice left
    open.
- **IM-5 — keyed metadata map.** Placement keyed by child identity.
  - Rejected on merit: imports an identity key into a phase whose
    identity baseline is positional / un-keyed (DD-M3-P7-005); the key
    must be invented precisely where identity is position, and the map
    can still desync with the child list on remove.

## Sub-issue 3 — IR / textual IR + compatibility policy

- Representation: existing child `IrProp` consumption vs an explicit
  child-slot record vs a parent-specific placement payload; loader
  validation policy for malformed placement metadata.
- **Compatibility policy (named explicitly).** If a child-slot record
  is a breaking change to textual IR, the old form is either migrated
  by the loader, rejected, or treated as an IR schema revision. The
  pre-1.0 textual IR is a **build-internal artifact** that `wasamoc`
  regenerates from `.ui` every build, so the default leans
  **reject + regenerate** — but it is decided here, not assumed, and
  matched to the DD-001 migration stance (no long-lived alias).

## Sub-issue 4 — Placement bindability / reactive mutation

Placement is literal / constant today. Phase 7b does **not** implement
bindable placement. What this DD records is a **policy + trigger**:
whether placement is a future-bindable public concept or constant-only
for layout stability, and where the reactive-architecture boundary sits
if it later becomes bindable. This must be consistent with DD-001's
author-surface read (sub-issue 7 there). The leading position: placement
is *constant-per-instance* in Phase 7b, with per-iteration variation
(distinct literal placement per generated child) admitted only as far
as the staging → commit path already supports; full reactive
re-binding of placement is deferred with a trigger.

## Sub-issue 5 — Runtime storage

- `Vec<WidgetNode>` + side metadata (IM-2 shape) vs a `Vec<ChildSlot>`
  conceptual model (IM-4 shape) vs a container-specific child-entry
  type.
- Common placement enum vs per-container placement payload — left as an
  implementation choice under IM-4 (what is fixed is the
  parent-interpreted, child-slot-carried *shape*, not a global enum).

## Sub-issue 6 — Structural mutation

- Adopt the DD-M3-P7-006 splice primitive as-is, or re-enumerate its
  side effects for Phase 7b's paths.
- Grid migration trigger: keep it held (Grid stays static-only) or
  **migrate Grid in Phase 7b** so both containers share one model. If
  DD-001 unifies the surface (CB-B) while Grid storage stays parallel,
  the surface and storage disagree across containers — an argument to
  pull the Grid migration into this phase. Pulling Grid in is the one
  place Phase 7b might exceed a minimal corrective (framing R5); the
  cost is bounded (Grid arrange loop + loader extraction, with Phase 5
  fixtures as the regression gate).

## Sub-issue 7 — Future code-construction boundary

- **No new API in Phase 7b** (FD-7b-D).
- The only non-committal constraint recorded: a future code-construction
  API **must not express placement as a generic child property setter**
  (`child.set_property("h-align", …)`) — that would re-introduce the
  intrinsic-widget-property reading the thesis rejects.
- The positive shape (parent-scoped insertion / child-slot builder /
  …) is **non-normative** and explicitly does **not** freeze ABI shape;
  concrete signatures are deferred to a future code-construction phase
  (framing scope table).

## Sub-issue 8 — Documentation

- `docs/architecture.md`: the chosen placement internal model
  (child-slot-carried if IM-4), the structural-mutation contract, and
  the ZStack-implemented / Grid-disposition state, re-connected.
- `docs/dsl_spec.md`: owned by DD-001 (author surface); this DD does
  not touch it (storage is not author-visible).
- `docs/abi_spec.md`: **no touch** (FD-7b-D), unless this DD finds an
  unavoidable future-compatibility note — judged here as **not
  needed**, since no API is added and the non-committal constraint is
  recorded in architecture prose, not the ABI.

## Comparison

The internal-model axis decides on **which model removes the drift
class** (the central Phase 7b obligation, implementation-gates trap
#3), with migration cost as tie-breaker. IM-1 contradicts the thesis;
IM-5 is keyed machinery before the keyed thesis; IM-2 leaves the
observed drift policed-not-structural. The real contest is **IM-3
(encapsulated SoA)** vs **IM-4 (child-slot-carried)**: IM-3 removes the
bypass class with zero data migration, but keeps the representational
split (placement stored apart from its child), so range staging and any
future reorder / key work keep paying the parallel-bookkeeping cost.
IM-4 removes the split itself and matches the authored model; it is
already shipped for ZStack, so adopting it phase-wide is *convergence*,
not a fresh bet, and its only cost is the bounded Grid migration.

## Recommendation

**Proposed direction (subject to owner review):**

1. **DD-M3-P7-006 relationship: revise.** Keep its child-carried
   thesis; extend it to Grid and restate the contract against the
   DD-001 surface. (Not *consume*, because Grid is pulled in; not
   *supersede*, because the storage thesis is unchanged.)
2. **Internal model: IM-4 (child-slot-carried)**, phase-wide,
   converging Grid onto the model ZStack already uses. Concrete value
   space (shared enum vs per-container child-entry type) left to
   implementation, as in DD-M3-P7-006.
3. **Grid migration: pull into Phase 7b** so surface and storage agree
   across containers, *if* DD-001 unifies the surface (CB-B); if DD-001
   keeps the asymmetry (Option 0), Grid migration may stay
   trigger-held. This conditional is the explicit coupling to DD-001's
   outcome.
4. **IR / textual IR: explicit child-slot record; reject + regenerate**
   for stale old IR forms (build-internal artifact; matches DD-001's
   no-long-lived-alias migration), with a named loader diagnostic.
5. **Bindability: constant-per-instance in Phase 7b**; per-iteration
   distinct literals admitted via staging → commit; full reactive
   re-binding deferred with a trigger.
6. **Structural mutation: adopt the DD-M3-P7-006 splice primitive**,
   re-enumerating its side-effect set for the Grid path as the trap
   #2 / #3 close artifact.
7. **Future code-construction: record the non-committal constraint
   only** (no generic child property setter); no API, no ABI shape,
   abi_spec no-touch.

This direction removes the drift class by construction, unifies the two
containers on the model already shipped for one, and keeps the future
API unfrozen. It is **Proposed**: the load-bearing owner-merit point is
item 3 (whether to pull the Grid migration into this corrective phase),
which is coupled to DD-001's surface outcome and to the framing R5
scope-creep risk; the rest follows from IM-4 once chosen.

## Forward-compat exposure

- **Reorder / keyed identity** — IM-4 makes "move a child" carry its
  placement for free; a future reconciler diffs whole child records.
- **New placement-bearing containers** — adopt child-slot-carried,
  parent-interpreted placement from birth; per-container value space
  until a shared enum has implementation merit.
- **Host state boundary / code-construction** — the recorded
  non-committal constraint keeps a future builder API unblocked without
  freezing it.

## Revision history

- 2026-06-19 — Initial draft (Status: Proposed), expanded from the
  framing §論点 slate (DD-M3-P7b-002). DD-M3-P7-006 relationship,
  internal-model options (IM-1..5 with child-slot as leading
  hypothesis, not default), IR / textual-IR compatibility policy,
  bindability policy+trigger, runtime storage, structural mutation +
  Grid-migration conditional, future code-construction boundary, and
  documentation touch/no-touch recorded, with a Proposed recommendation
  (revise P7-006 / IM-4 phase-wide / Grid pulled in conditional on
  DD-001). Pending owner review.
