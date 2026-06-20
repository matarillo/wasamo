---
title: Placement internal model and construction boundary
status: Proposed
phase: M3-Phase 7b
ac: A11 (IR / runtime / loader sync) — discharges the framing parallel-data-drift obligation with impl-gates #2 / #3; the contingent new-AC question is owned by DD-M3-P7b-001, not this DD (storage is not author-visible)
date: 2026-06-19
related:
  - ./preamble.md
  - ./dd-m3-p7b-001-parent-interpreted-placement-authoring-surface.md
  - ../../phase-7/decisions/dd-m3-p7-006-placement-storage-and-structural-side-effects.md
  - ../requirements/framing.md
---

# DD-M3-P7b-002 — Placement internal model and construction boundary

**Status:** Proposed

## Context

How should parent-interpreted placement metadata — once written in the
DSL (DD-001) — be represented across IR, textual IR, runtime storage,
and structural mutation, and what direction (not API) is recorded for a
future code-construction surface?

Child-slot-carried is the **leading hypothesis** (it removed the
observed Phase 6 drift class structurally and is already shipped for
ZStack), but it is **not** a default; the alternatives are compared on
merit (owner prior — decide on which model structurally removes the
drift class, migration cost as tie-breaker only).

**Current state consumed (verified at drafting):**

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

## Dependencies

- **Consumes** DD-M3-P7b-001 (conceptual boundary + author-visible
  bindability read). DD-001 and DD-002 are **Accepted together** as one
  phase ADR set.
- **Re-opens and re-connects**
  [DD-M3-P7-006](../../phase-7/decisions/dd-m3-p7-006-placement-storage-and-structural-side-effects.md),
  which decided child-carried placement (ST2) for ZStack and **deferred
  Grid** behind a trigger, under the assumption that the author surface
  was unchanged — an assumption Phase 7b reopens. Fixing the verb for
  that relationship (consume / revise / supersede / split, framing R3)
  is the upper-level premise of the Main decision below.
- **Bindability (carried, not implemented).** Phase 7b does not
  implement bindable placement. This DD records a *policy + trigger*:
  placement is **constant-per-instance** in Phase 7b, with per-iteration
  variation (distinct literal placement per generated child) admitted
  only as far as the staging → commit path already supports; full
  reactive re-binding is deferred with a trigger. This must stay
  consistent with DD-001's author-surface read.

## Main decision — internal model (and the P7-006 relationship it assumes)

The single load-bearing decision is which **internal model** stores
parent-interpreted placement. The upper-level premise it assumes is the
verb taken on DD-M3-P7-006: the recommendation below takes **revise**
(keep its child-carried thesis, extend to Grid, restate against the
DD-001 surface) — *consume* understates Grid being pulled in, *supersede*
overstates it (the storage thesis is unchanged), *split* would leave the
representation re-connection unowned.

### Options

1. **IM-1 — widget property model.** Store placement as an ordinary
   property on the child widget node.
   - What you gain: trivially uniform with widget props.
   - What you give up: makes placement an *intrinsic widget property*,
     contradicting FD-7b-A's settled floor.
   - Technical risk: thesis-violating; listed for completeness, rejected.
2. **IM-2 — parent parallel metadata.** The current Grid model: a
   placement vector kept parallel to `children`.
   - What you gain: zero migration for Grid; contiguous reads in arrange.
   - What you give up: the **observed** drift class (Phase 6, impl-gates
     trap #3) — child list and placement vector desynchronise on insert
     / remove / splice / reorder, policed by helper discipline rather
     than structure.
   - Technical risk: the invariant is policed, not structural; every new
     code path can re-introduce the drift.
3. **IM-3 — encapsulated SoA + splice-only mutation.** Keep the parallel
   representation but behind a module boundary; all structural mutation
   enters one splice primitive (DD-M3-P7-006's ST1').
   - What you gain: removes the *bypass* drift class by language
     visibility, zero data-model migration.
   - What you give up: preserves the representational split (placement
     stored apart from the child it places); staged subtrees still carry
     placement beside children until commit; the splice signature keeps
     parallel-vector bookkeeping.
   - Technical risk: low (no data migration), but the split keeps costing
     range staging and any future reorder / key work.
4. **IM-4 — child-slot-carried placement (leading hypothesis).** The
   child slot carries the node plus its optional parent-interpreted
   placement (`None` for placement-free containers).
   - What you gain: the drift class is removed **by construction** — a
     child and its placement are one record, so no insert / remove /
     splice / reorder can desynchronise them; the splice signature
     shrinks (children in, children out); generated subtrees carry
     placement through staging → commit as ordinary data; storage matches
     the authored parent-interpreted model without making placement an
     intrinsic widget property. **Already shipped for ZStack.**
   - What you give up: a runtime structural migration of the Grid arrange
     / loader read path; the concrete value space (one enum vs
     per-container child-entry type) is left open.
   - Technical risk: the Grid read-path migration is a runtime structural
     change (full-review lane); bounded by existing Phase 5 Grid fixtures.
5. **IM-5 — keyed metadata map.** Placement keyed by child identity.
   - What you gain: direct lookup by identity.
   - What you give up: imports an identity key into a phase whose
     identity baseline is positional / un-keyed (DD-M3-P7-005); the key
     must be invented precisely where identity is position, and the map
     can still desync with the child list on remove.
   - Technical risk: keyed-identity machinery before the keyed thesis;
     rejected on merit.

### Forward-compat impact

- **Reorder / keyed identity** — IM-4 makes "move a child" carry its
  placement for free; a future reconciler diffs whole child records.
- **New placement-bearing containers** — under IM-4 they adopt
  child-slot-carried, parent-interpreted placement from birth;
  per-container value space until a shared enum has implementation merit.
- IM-2/IM-3 keep the representational split that each of those future
  extensions would have to keep paying for.

### Recommendation

**IM-4 (child-slot-carried), phase-wide**, converging Grid onto the
model ZStack already uses; P7-006 relationship = **revise**. The real
contest is IM-3 vs IM-4: IM-3 removes the bypass class with zero data
migration but keeps the representational split; IM-4 removes the split
itself and matches the authored model, and because it is already shipped
for ZStack, adopting it phase-wide is *convergence*, not a fresh bet.
IM-1 contradicts the thesis; IM-2 leaves the observed drift
policed-not-structural; IM-5 is keyed machinery before the keyed thesis.
Concrete value space (shared enum vs per-container child-entry type) is
left to implementation, as in DD-M3-P7-006.

Recorded as **Proposed**: the load-bearing owner-merit point is whether
to pull the Grid migration into this corrective phase (SI-2), coupled to
DD-001's surface outcome.

## Sub-issues

Two sub-decisions carry their own option sets; both follow the Main
decision (IM-4) and feed the spec / mitigation sections:

- **SI-1 — IR / textual IR representation** — the placement carrier and
  the stale-form compatibility policy.
- **SI-2 — Structural mutation** — splice-primitive reuse and the
  Grid-migration conditional (the load-bearing owner-merit point).

### Dependencies among sub-issues

The IR carrier (child-slot record vs parallel) and the structural
mutation path move together: a child-slot IR record presumes IM-4 and
feeds the splice primitive's "children in, children out" signature.
SI-2's Grid-migration conditional additionally depends on DD-001's
boundary choice (CB-B vs Option 0).

## SI-1: IR / textual IR representation

### Context

How placement is carried across IR and textual IR, and what happens to
stale forms. The pre-1.0 textual IR is a **build-internal artifact**
that `wasamoc` regenerates from `.ui` every build.

### Options

1. **IR-1 — existing child `IrProp` consumption.** Keep placement as
   ordinary child props in the IR.
   - Depends on (main option): pairs with IM-2/IM-3 (parent reads props
     into a parallel vector).
   - What you gain: no IR shape change.
   - What you give up: the IR does not express the child-slot record IM-4
     wants; the loader keeps re-deriving the split.
   - Technical risk: low IR risk, but locks in the representational split.
2. **IR-2 — explicit child-slot record (recommended).** The IR carries a
   child entry = node + optional placement payload.
   - Depends on (main option): IM-4.
   - What you gain: the storage shape is expressed in the IR; loader
     extraction targets one record; roundtrip pins it.
   - What you give up: a breaking change to the textual-IR shape.
   - Technical risk: an IR schema change — mitigated by reject + regenerate
     (below), since the IR is build-internal.

Compatibility policy (decided, not assumed): **reject + regenerate.**
Stale old-form IR is rejected with a named loader diagnostic and
regenerated by `wasamoc`, rather than dual-parsed. Matches DD-001's
no-long-lived-alias migration stance.

### Forward-compat impact

A child-slot IR record is the shape a future reconciler / reorder diff
operates on (whole child records). Reject + regenerate keeps the loader
single-form, so a later IR addition is an additive record field, not a
dual-form parser.

### Recommendation

**IR-2 (explicit child-slot record) + reject + regenerate** for stale
old IR forms, with a named loader diagnostic.

## SI-2: Structural mutation

### Context

Whether to adopt the DD-M3-P7-006 splice primitive as-is or re-enumerate
its side effects for Phase 7b's paths, and **whether to migrate Grid in
Phase 7b**. This is the load-bearing owner-merit point.

### Options

1. **SM-A — keep the DD-M3-P7-006 splice primitive; Grid stays
   trigger-held.** ZStack stays child-carried; Grid keeps its parallel
   vector (static-only) until its trigger fires.
   - Depends on (main option): compatible with IM-4 for ZStack only.
   - Depends on (other sub-issue): viable if DD-001 keeps the surface
     asymmetry (Option 0).
   - What you gain: minimal corrective footprint (framing R5).
   - What you give up: if DD-001 unifies the surface (CB-B) while Grid
     storage stays parallel, surface and storage disagree across
     containers.
   - Technical risk: low; but leaves a known split in place.
2. **SM-B — migrate Grid into Phase 7b (recommended if CB-B).** Pull the
   Grid `cell_placements` migration into this phase so both containers
   share one model; re-enumerate the splice side-effect set for the Grid
   path.
   - Depends on (main option): IM-4 phase-wide.
   - Depends on (other sub-issue): conditional on DD-001 choosing CB-B
     (unified surface); if DD-001 keeps Option 0, SM-A may stand.
   - What you gain: surface and storage agree across containers; one
     model, one splice signature.
   - What you give up: this is the one place Phase 7b might exceed a
     minimal corrective (framing R5).
   - Technical risk: Grid arrange loop + loader extraction is a runtime
     structural change; bounded — Phase 5 Grid fixtures are the regression
     gate, migration is its own commit preceding any Grid mutation path.

### Forward-compat impact

Migrating Grid now (SM-B) means any future Grid structural mutation
(`for` of `Cell`s, conditional `Cell`s) is built against the
child-carried model from the start; keeping it held (SM-A) preserves the
DD-M3-P7-006 recursive trigger (migrate before any Grid mutation path
exists).

### Recommendation

**SM-B (migrate Grid in Phase 7b)** if DD-001 unifies the surface
(CB-B); **SM-A (trigger-held)** if DD-001 keeps the asymmetry (Option
0). Either way, adopt the DD-M3-P7-006 splice primitive and re-enumerate
its side-effect set for the migrated path as the trap #2 / #3 close
artifact.

## Decision outcome

TBD (Proposed). Filled at the Accepted flip with the P7-006 verb, the
internal model (IM-n), the IR carrier + compatibility policy, the
Grid-migration disposition (SM-A/SM-B, conditional on DD-001), and the
bindability policy + trigger.

## Spec impact

`docs/architecture.md`:

- The chosen placement internal model (under the recommendation:
  child-slot-carried, IM-4), stated as parent-interpreted per-container
  placement carried by the child slot, re-connecting the
  ZStack-implemented model with Grid and the DD-001 surface.
- The splice primitive's **side-effect set re-enumerated for the
  migrated path** (kept as a forcing artifact, not summarised away):
  child list splice (placement riding along), Visual sibling order,
  layout invalidation, widget-pointer registry, effect ownership — as
  one composed operation; the Grid / ZStack storage state and the
  remaining migration trigger (if SM-A).
- A non-normative note that a future code-construction API **must not**
  express placement as a generic child property setter (recorded in
  prose, not the ABI).

`docs/dsl_spec.md`: **not touched** by this DD (storage is not
author-visible; the author surface is DD-001's seed).

`docs/abi_spec.md`: **no touch** (FD-7b-D) — no API is added; the
non-committal constraint lives in architecture prose, not the ABI.

## Risk mitigation

(Assuming the recommended IM-4 + IR-2 + SM-B under CB-B.)

- **Grid migration:** a runtime structural change on the full-review
  lane, bounded — Phase 5 Grid fixtures (track sizing, spanning,
  membership / conflict, arrange overflow) run unchanged as the
  regression gate; the migration is its own commit preceding any Grid
  mutation path. A tie-breaker cost, not a counter-argument.
- **IR schema change:** reject + regenerate keeps the loader single-form
  (no dual-form parser); roundtrip fixtures pin the new child-slot shape;
  malformed placement metadata is re-rejected by a named loader
  diagnostic.
- **Splice side-effect set:** re-enumerated for the Grid path as the
  trap #2 / #3 close artifact; the parallel-data audit reduces to "no
  parallel vectors remain on mutated paths" (greppable: `cell_placements`
  migrated or static-only with the DD pointer).
- **Convergence, not a fresh bet:** IM-4 is already shipped for ZStack,
  so the novel-risk perimeter is the Grid read / loader path only, not a
  new storage model.
- **Performance** is a non-axis at gallery N; child-slot reads replace
  parallel-vector indexing with no caching / segmenting introduced.

## Out of scope

- **No new code-construction API / ABI** (FD-7b-D). The only recorded
  constraint is non-committal: a future builder must not express
  placement as a generic child property setter
  (`child.set_property("h-align", …)`), which would re-introduce the
  intrinsic-widget-property reading. The positive shape (parent-scoped
  insertion / child-slot builder) is non-normative and not frozen.
- **Bindable placement** — not implemented; constant-per-instance with a
  deferral trigger (see §Dependencies).
- **Keyed child metadata / retained identity** — rejected for this phase
  (IM-5); identity baseline stays positional (DD-M3-P7-005).
- **Grid structural mutation under `Cell`** — out of scope unless SM-B
  pulls the storage migration in; the mutation paths themselves
  (`for`/`if` of `Cell`s) remain deferred with the DD-M3-P7-006 trigger.

## Revision history

- 2026-06-19 — Initial draft (Proposed). P7-006 relationship (revise),
  internal-model options (IM-1..5), the IR and structural-mutation
  sub-issues (SI-1 / SI-2), and a recommendation of IM-4 phase-wide with
  the Grid migration conditional on DD-001. Pending owner review.
