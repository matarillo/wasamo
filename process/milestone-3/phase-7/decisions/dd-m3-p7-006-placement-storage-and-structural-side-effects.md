# DD-M3-P7-006 — Placement storage model and structural side-effect atomicity

**Status:** Proposed
**Phase:** M3-Phase 7
**AC:** A8 (range mutation correctness); named by the Phase 6 handoff
as "a Phase 7 decision **before** range mutation grows" (constraints §4)

## Context

Parent-owned per-child placement metadata is stored SoA today:
`WidgetData::ZStack { zstack_placements: Vec<ZStackPlacement> }` and
`WidgetData::Grid { cell_placements: Vec<CellPlacement> }`, each kept
**parallel** to `WidgetNode.children` (`placements[i]` places
`children[i]`). Every structural mutation must therefore update child
list + placement vector + live Visual sibling order as **one
invariant**. Phase 6's single-child conditional already required a
dedicated `insert_child_with_zstack_placement` + paired removal, and
the T1/T2/T3 ZStack placement follow-ups produced real drift — the
implementation-gates **trap #3 (parallel data drift)** failure mode,
observed, not hypothetical.

Iteration raises the stakes: one mutation inserts / removes **multiple
children**, and (per DD-M3-P7-001's sweep) ZStack admits direct `for`,
so the range path crosses placement-bearing storage in-phase. The
handoff requires the storage model to be decided **before** the range
primitive is implemented, so the primitive is built against the chosen
model rather than migrated after.

Owner prior (FD-P) applies sharply here: the comparison is decided on
which model **structurally removes the drift class**, with migration
cost as tie-breaker only.

## Decision dependency summary

Consumes DD-M3-P7-001 (which containers admit direct `for`) and
DD-M3-P7-004 (seam-computed offsets). Provides the **splice primitive**
DD-M3-P7-005's plans execute through. Close artifacts: gates traps
#2 / #3.

## Sub-issues

- **Storage model** — SoA parallel vectors vs child-carried vs keyed
  map.
- **Migration scope** — ZStack only, or Grid too.
- **The splice primitive** — the single mutation entry point and its
  side-effect enumeration.

## Storage model

### Options

- **ST1 — keep SoA parallel vectors + mandated atomic helpers**
  - Keep `zstack_placements` parallel to `children`; require every
    mutation to go through paired splice helpers.
  - What you gain: zero migration; layout code reads contiguous
    placement vectors as today.
  - What you give up: the invariant remains **policed, not
    structural** — nothing prevents a new code path (the very thing
    Phase 7 adds) from touching `children` without the helper; that is
    precisely how the Phase 6 drift happened with *one* child and
    *one* mutating path. Range mutation multiplies both the paths and
    the per-mutation index arithmetic. The model survives on
    discipline (a Forcing-tier artifact at best) where a structural
    fix is available.

- **ST1' — encapsulated SoA + splice-only mutation**
  - Keep the SoA representation, but move `children` and the parallel
    placement vector behind a dedicated module boundary with private
    fields; all structural mutation must enter through a splice
    primitive that updates both vectors.
  - What you gain: structural enforcement by language visibility for
    the specific "touch `children` and forget placement" drift class,
    with zero data-model migration; new call paths cannot bypass the
    primitive from outside the owning module.
  - What you give up: the authored model still says placement is written
    on the child and interpreted by the parent, while storage remains a
    separate parent vector; staged subtrees must still carry placement
    beside children until commit; the splice primitive's signature must
    preserve parallel-vector bookkeeping. This is a real structural
    improvement over ST1, but it preserves the representational split
    that Phase 7's range staging and future reorder/key work would keep
    paying for.

- **ST2 — child-carried placement**
  - The placement annotation moves onto the parent-child edge: the child
    slot carries the node plus its optional parent-interpreted placement
    kind (`None` for placement-free containers). The author surface
    already *writes* placement on the child (`h-align:` / `v-align:` are
    child props interpreted by the parent — DD-M3-P6-002); ST2 makes
    storage match the authored model without making placement an
    intrinsic widget property.
  - The concrete value space stays open for implementation: either one
    enum over placement-bearing containers or per-container child-entry
    types can satisfy the contract. What is settled here is the
    parent-interpreted, child-slot-carried shape, not a global
    placement enum.
  - What you gain: **the drift class is removed by construction** —
    a child and its placement are one record, so no insert, remove,
    range splice, or future reorder can desynchronise them; the splice
    primitive's signature shrinks (children in, children out — no
    parallel-vector bookkeeping); generated subtrees (whose placements
    are instantiated per item from the body template) carry their
    placement through staging → commit (DD-005 PF2) as ordinary data.
  - What you give up: a runtime structural migration of the ZStack
    (and eventually Grid) layout read path — arrange iterates children
    and reads the carried placement instead of indexing a parallel
    vector; the IR loader's placement extraction re-targets. The
    full-review lane applies (runtime structural change).

- **ST3 — keyed metadata map** (placement keyed by child identity)
  - What you give up: it imports an *identity key* into a phase whose
    identity baseline is deliberately positional / un-keyed
    (DD-M3-P7-005) — the map's key has to be invented (pointer
    identity? slot?) precisely where the design says identity is
    position; it solves neither lookup (children are walked in order
    anyway) nor drift (map and child list can still desynchronise on
    remove). It is keyed-identity machinery arriving before the keyed
    thesis. Rejected on merit.

### Comparison

ST1 is discipline vs structure for an invariant whose violation has
already been observed once. ST1' is the fair structural SoA alternative:
module visibility can remove the bypass class without migrating stored
shape. ST2 is still stronger on its own merits: it aligns storage with
the authored, parent-interpreted placement model; it lets staged
subtrees carry placement as ordinary child-slot data through
staging → commit (DD-005 PF2); and it removes parallel-vector
bookkeeping from the splice primitive's signature instead of merely
centralising that bookkeeping. Under FD-P, ST2's migration cost is a
tie-breaker, not a counter-argument — and it is bounded: ZStack's
arrange loop and the loader's extraction site, both with existing Phase
6 fixtures as regressions.

### Recommendation

**ST2 — child-carried placement.** The structural invariant ("a child
slot and its parent-interpreted placement are one record") is stated in
architecture.md as the accepted contract. ST1' is rejected on merit, not
ignored: it enforces mutation entry but keeps the storage / staging /
splice-signature split that ST2 removes.

## Migration scope

- **ZStack: migrate this phase.** It is on the direct-`for` path
  (DD-001 sweep) and was the observed drift site.
- **Grid: defer with trigger.** Grid rejects direct `for` this phase
  (children are `Cell`-mediated), so `cell_placements` sees no range
  mutation. On merit / proportionality, migrating a storage path that no
  admitted mutation crosses would not protect an invariant this phase;
  the recursive trigger below preserves the guarantee by migrating
  before any Grid mutation path exists. **Trigger:** Grid admitting
  structural mutation under it (direct `for` of `Cell`s, conditional
  `Cell`s, or any second parent-owned per-child metadata kind arriving)
  ⇒ migrate `cell_placements` to the same child-carried model **before**
  that mutation path is built (this DD's rule applied recursively).
  Until then the Grid path is static-only and the SoA comment in
  `widget.rs` gains a pointer to this DD.
- **WrapPanel / VStack / HStack / ScrollView / Box:** no per-child
  placement — the no-placement path stays free of placement logic
  (carried field `None`), asserted by the container sweep tests.

## The splice primitive

One entry point owns every structural child mutation on the
materialised tree (conditional 0/1 — migrated via DD-004's C1 seam
work — and `for` ranges alike):

```
splice_children(parent, declared_slot, materialised_range, staged_new_children)
```

semantics: replace `materialised_range` (possibly empty) under
`parent` with `staged_new_children` (possibly empty), at offsets
computed through the C1 seam. Its **side-effect enumeration** (gates
trap #2 close artifact — listed here so the implementation audits
against it, not from memory):

1. `children` vector splice (placements ride along — ST2);
2. **Visual sibling order**: removed children's Visuals detached;
   staged Visuals inserted at the correct sibling positions (declared
   order with live cardinalities, never just appended on top — the
   Phase 6 declared-sibling-order lesson generalised to ranges);
3. **layout invalidation**: parent subtree marked dirty so the next
   pass re-measures / re-arranges;
4. **widget-pointer registry**: removed subtrees' entries released
   (destroy path), inserted subtrees' entries registered;
5. **effect ownership**: removed subtrees' effects disposed *before*
   teardown (§6.7.6, executed by DD-005's ordering), staged subtrees'
   effects attached at commit;
6. **parent-owned metadata other than placement**: none exists after
   ST2 for the admitted containers — asserted, and re-checked by the
   sweep if a future container adds one (the trigger above).

All six happen inside the primitive; no caller composes them. This is
the trap-#3 structural complement: ST2 removes the placement copy of
the problem, the single primitive removes the multi-call-site copy.

## Spec content seed

architecture.md (§6.9 layout-primitives section + structural-mutation
text): child-carried placement stated as the storage contract (ZStack
now, Grid on its trigger), explicitly as parent-interpreted
per-container placement carried by the child slot; the splice
primitive's side-effect set as the accepted structural-mutation
contract. dsl_spec is **not** touched by this DD (storage is not
author-visible; the author surface `h-align` / `v-align` is unchanged).

## Forward-compat exposure

- **Grid migration** — the recorded trigger above.
- **Reorder / keyed identity** — ST2 makes "move a child" carry its
  placement for free; the reconciler-era diff operates on whole child
  records.
- **Member-range bodies** — splice already takes ranges; arity changes
  only the staged list length.
- **New placement-bearing containers** — must adopt child-slot-carried,
  parent-interpreted placement from birth; their concrete placement
  value space may be per-container unless / until a shared enum has
  implementation merit.

## Revision history

- Strategic owner-alignment review fold: added ST1' as the steel-man SoA
  option; clarified child-slot carried placement and open value-space;
  reframed Grid defer on proportionality / trigger grounds; status
  remains Proposed.

## Technical risk re-evaluation

- **The ZStack migration** touches shipped arrange / loader code:
  Phase 6 ZStack fixtures (union sizing, alignment defaults /
  overrides, conditional-under-ZStack placement) run unchanged as the
  regression gate; the migration is its own commit preceding the range
  primitive.
- **The primitive's Visual-order step** is the WinRT-fallible part of
  DD-005's commit phase; its failure handling follows PF2's recorded
  contract (logged with range context; re-trigger on observation).
- **Trap #2/#3 close artifacts** for every task touching the
  primitive: the side-effect enumeration above checked off
  per-change, and the parallel-data audit reduced to "no parallel
  vectors remain on mutated paths" (greppable: `zstack_placements`
  deleted; `cell_placements` static-only with the DD pointer).
- **Performance** is a non-axis at gallery N; no caching/segmenting of
  the seam offsets (recompute-per-mutation stands, DD-004).
