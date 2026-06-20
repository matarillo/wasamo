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
  reactive re-binding is deferred. **Re-visit trigger (shared with
  DD-001 §Out of scope):** a concrete app needs placement that varies
  after construction; the re-binding is then designed together with the
  `BindingTarget` machinery (a new binding-target variant) and the
  child-slot **effect lifecycle** owned here — placement rides the
  child-slot record, so a future per-item placement effect attaches /
  detaches with that slot rather than through a storage-local patch. This
  must stay consistent with DD-001's author-surface read (which rejects a
  binding RHS by a named diagnostic this phase).

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
     / loader read path; the concrete value space (per-container payload
     vs shared enum vs extensible record) is **decided in SI-3** — under
     DD-001's CB-B it is a forward-compat choice, not left to
     implementation.
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
  child-slot-carried, parent-interpreted placement from birth; the
  per-container vs shared vs extensible payload trade is SI-3's, with a
  trigger to a shared enum once a third container's keys overlap.
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
The concrete value space (per-container typed payload vs shared placement
enum vs extensible slot-metadata record) is **not** left to
implementation under CB-B — DD-001's "generalizable parent-data" boundary
makes the payload shape a forward-compat decision, taken in **SI-3**,
which pins the condition that the Phase 7b shape keep the `slot.*` model
generalizable.

Recorded as **Proposed**: the load-bearing owner-merit point is whether
to pull the Grid migration into this corrective phase (SI-2), coupled to
DD-001's surface outcome.

## Sub-issues

Four sub-decisions carry their own option sets; all follow the Main
decision (IM-4) and feed the spec / mitigation sections:

- **SI-1 — IR / textual IR representation** — the placement carrier
  (Rust IR abstract form) and the stale-form compatibility policy.
- **SI-2 — Structural mutation** — splice-primitive reuse and the
  Grid-migration conditional (the load-bearing owner-merit point).
- **SI-3 — Child-slot value space** — the placement payload shape
  (per-container vs shared vs extensible; minimum carrier shape) and the
  CB-B integration condition that keeps a later shared/extensible carrier
  additive.
- **SI-4 — Canonical textual-IR shape for placement slots** — whether
  `Cell` survives in *textual* IR or is normalised to a child-slot
  record, and how stale `Cell` placement IR is handled. Owner-visible
  because it sets the regenerate / compatibility contract, not just an
  encoding.

### Dependencies among sub-issues

The IR carrier (child-slot record vs parallel) and the structural
mutation path move together: a child-slot IR record presumes IM-4 and
feeds the splice primitive's "children in, children out" signature.
SI-2's Grid-migration conditional additionally depends on DD-001's
boundary choice (CB-B vs Option 0). SI-3's payload shape is what the
SI-1 IR record carries (the record holds whatever payload SI-3 picks),
and its integration condition binds only under CB-B — under Option 0 the
per-container payloads need no shared-path guarantee. SI-4 decides the
*textual* spelling of the SI-1 record (Rust abstract form is SI-1's,
textual canonical form is SI-4's); it depends on DD-001's PM disposition
(under PM-1, Grid is authored as `Cell` but its textual-IR canonical form
is still SI-4's call — keep the `Cell` wrapper or normalise it away).

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

**Rust IR abstract form (in-memory, IR-2).** Today a child member is
`IrMember::Widget(IrNode)`, Grid placement rides `IrNode.kind_payload`
plus a `Cell` `IrProp` extraction (`Cell` is IR-only, DD-M3-P5-001), and
ZStack placement rides ordinary child `IrProp`s. Under IR-2 the slot
becomes node + optional placement payload — sketch:

```text
IrMember::Widget { node: IrNode, placement: Option<IrPlacement> }
   IrPlacement = the SI-3 carrier (recommended VS-1a: a closed
                 Grid/ZStack payload), None for placement-free parents
```

Both **Grid `Cell` sugar (DD-001 PM-1)** and **ZStack `slot.*`** lower to
this *one* slot record — that is where the model-level unification lives.
The *textual* spelling of that record (does `Cell` survive in textual IR,
or is it normalised away?) is **SI-4**, not decided here.

## SI-2: Structural mutation

### Context

Whether to adopt the DD-M3-P7-006 splice primitive as-is or re-enumerate
its side effects for Phase 7b's paths, and **whether to migrate Grid in
Phase 7b**. This is the load-bearing owner-merit point.

### Options

The branch key is **DD-001's conceptual boundary (CB-A/CB-B), not its
authored-surface symmetry.** PM-1 keeps the *authored* surface asymmetric
(Grid `Cell`, ZStack `slot.*`) yet is **CB-B** — model unified — so the
storage must converge even though the surface does not. Reading "PM-1 is
not surface-unified, therefore SM-A" is the trap this sub-issue closes.

1. **SM-A — keep the DD-M3-P7-006 splice primitive; Grid stays
   trigger-held.** ZStack stays child-carried; Grid keeps its parallel
   `cell_placements` vector (static-only) until its trigger fires.
   - Depends on (main option): compatible with IM-4 for ZStack only.
   - **Applies when:** DD-001 selects **Option 0 / CB-A** (the model is
     *not* unified, so storage divergence is consistent with the surface).
   - What you gain: minimal corrective footprint (framing R5).
   - What you give up: under CB-B it leaves model unified but storage
     split — the exact incoherence Phase 7b exists to remove.
   - Technical risk: low; but leaves a known split in place.
2. **SM-B — migrate Grid into Phase 7b (recommended under CB-B,
   *including PM-1*).** Pull the Grid `cell_placements` migration into
   this phase so both containers share one child-slot model; re-enumerate
   the splice side-effect set for the Grid path.
   - Depends on (main option): IM-4 phase-wide.
   - **Applies when:** DD-001 selects **CB-B / Option 3 — at *any* SI-2
     mapping, PM-1 included.** The model unification (not the authored
     surface) is what makes storage divergence incoherent, so PM-1 (model
     unified, surface asymmetric) still requires SM-B.
   - What you gain: surface *model* and storage agree across containers;
     one model, one splice signature.
   - What you give up: this is the one place Phase 7b might exceed a
     minimal corrective (framing R5).
   - Technical risk: Grid arrange loop + loader extraction is a runtime
     structural change; bounded — Phase 5 Grid fixtures are the regression
     gate, migration is its own commit preceding any Grid mutation path.
3. **SM-C — migrate Grid only if the *authored* Grid surface admits
   direct `slot.*` (PM-2 / PM-3).** The conservative reading: defer Grid
   storage migration unless Grid children are actually authored on
   `slot.*`.
   - Depends on (main option): IM-4 for ZStack; Grid deferred.
   - **Applies when:** DD-001 selects CB-B but the owner wants to bound
     7b to the surface that *visibly* changed; under PM-1 Grid stays
     trigger-held.
   - What you gain: smallest 7b footprint that still unifies ZStack.
   - What you give up: under PM-1 it re-opens the CB-B model/storage
     incoherence SM-B closes — model says "one placement model", storage
     says "Grid is still parallel". The cost SM-B pays once is deferred to
     the Grid mutation trigger instead.
   - Technical risk: leaves the split; the DD-M3-P7-006 recursive trigger
     must still fire before any Grid mutation path.

### Forward-compat impact

Migrating Grid now (SM-B) means any future Grid structural mutation
(`for` of `Cell`s, conditional `Cell`s) is built against the
child-carried model from the start; deferring it (SM-A / SM-C) preserves
the DD-M3-P7-006 recursive trigger (migrate before any Grid mutation path
exists).

### Recommendation

**SM-B (migrate Grid in Phase 7b) whenever DD-001 selects CB-B / Option
3 — explicitly including PM-1**, because model unification (not
authored-surface symmetry) is what makes a parallel Grid vector
incoherent. **SM-A (trigger-held)** applies only under **Option 0 / CB-A**
(no model unification). **SM-C** is the live conservative alternative if
the owner wants 7b bounded to the surface that visibly changed (ZStack)
and accepts the residual CB-B model/storage split under PM-1 — this is an
**owner scope call**, recorded as such, not an agent default. Either way,
adopt the DD-M3-P7-006 splice primitive and re-enumerate its side-effect
set for the migrated path as the trap #2 / #3 close artifact.

## SI-3: Child-slot value space (placement payload shape)

### Context

IM-4 stores placement on the child slot, but *what shape* the placement
payload takes is a forward-compat decision, not a mere implementation
detail — because DD-001 recommends CB-B (`slot.*` as one instance of a
*generalizable* parent-data grammar). The payload shape decides how
naturally a future **custom container**, **non-layout parent-data**
(hit-test / focus / accessibility), or **code-construction builder**
extends the slot. Too one-off and CB-B's "generalizable" promise narrows
in practice; too general too early and the slot drifts toward the generic
modifier system Phase 7b explicitly defers (framing R1).

### Options

1. **VS-1 — per-container typed payload (recommended for Phase 7b).**
   Each placement-bearing container's slot carries a small typed payload
   for *its* keys (Grid: `row` / `column` / `span` / alignment; ZStack:
   `h-align` / `v-align`), `None` where the container takes no placement.
   - What you gain: minimal, exact storage; no key-space invention beyond
     today's two containers; matches the bounded corrective scope.
   - What you give up: a third placement-bearing container adds a third
     payload type rather than reusing a shared carrier.
2. **VS-2 — shared placement enum.** One enum across containers, variants
   per placement kind.
   - What you gain: one carrier type; a new container reuses variants.
   - What you give up: pushes a cross-container vocabulary *now* — closer
     to the generic-modifier surface Phase 7b defers; premature while
     only two containers exist.
3. **VS-3 — extensible slot-metadata record.** The slot carries an open
   metadata record (keyed entries) accommodating arbitrary parent-data,
   not only layout.
   - What you gain: the most general — non-layout parent-data and custom
     slot keys drop in without a type change.
   - What you give up: an open record is effectively the generic
     parent-data bag whose design Phase 7b defers (R1); admission / typing
     guarantees weaken; over-built for two containers.

### Forward-compat impact (the CB-B integration condition)

Phase 7b may ship **VS-1**, but only under a pinned condition, so CB-B's
"generalizable" promise is not quietly narrowed into per-container
one-off storage: the per-container payloads must sit **behind the one
child-slot record** (IM-4) and behind the `slot.*` admission path
(DD-001), so that moving to VS-2 / VS-3 later is an **additive carrier
change**, not a re-litigation of where placement lives or how it is
authored. Concretely — (i) no container stores its placement *outside*
the child slot (no return to a parallel vector), and (ii) the loader /
checker treats `slot.*` keys through the same admission table regardless
of payload type. Under that condition VS-1 is a storage *encoding*, not a
boundary commitment; VS-2 / VS-3 stay open with triggers (a new
placement-bearing container with overlapping keys → VS-2; the first
non-layout parent-data → VS-3, jointly with the framing R1
generic-modifier decision).

### Minimum accepted carrier shape (owner-visible — VS-1 is not enough on its own)

"Per-container typed payload behind one child-slot record" still admits
three concrete carrier shapes, and the choice changes how much of CB-B's
"generalizable" promise is real. The implementer must **not** pick this;
it is an owner judgment because it sets the future-extensibility floor.
The current runtime baseline is already split — ZStack child-carried via
`WidgetNode.zstack_placement: Option<ZStackPlacement>`, Grid still on the
parallel `WidgetData::Grid { cell_placements: Vec<CellPlacement> }` — so
the carrier shape *is* the central Phase 7b storage change:

- **VS-1a — one slot field carrying a closed payload enum**
  (e.g. `placement: Option<Placement>` where `Placement::{Grid(..),
  ZStack(..)}`). One field, one closed carrier; a third container adds a
  variant. **Recommended:** it is the smallest shape that keeps a single
  `slot.*` admission path and makes VS-2 (collapse variants into a shared
  enum) a pure additive refactor, *without* publishing a cross-container
  vocabulary now (so it is not VS-2 — variants stay container-named, not
  unified by placement *kind*).
- **VS-1b — separate optional fields on the child slot**
  (e.g. `grid_placement: Option<..>` *and* `zstack_placement:
  Option<..>`). Lowest-churn from today's ZStack field, but multiplies
  fields per container and re-creates a soft parallelism (which field is
  live depends on the parent), weakening the "one record" invariant the
  Main decision buys.
- **VS-1c — per-container child-slot wrapper types** (a distinct slot
  struct per container kind). Most explicit per container, but the splice
  primitive then needs a wrapper-aware signature, eroding the "children
  in, children out" shrink IM-4 is chosen for.

### Recommendation

**VS-1 (per-container typed payload) for Phase 7b, in the VS-1a shape
(one slot field, closed payload enum), under the CB-B integration
condition above.** VS-1a is the minimum carrier that holds the condition
structurally — one field, one admission path, additive to VS-2 — while
VS-1b/VS-1c either re-introduce parallelism (1b) or complicate the splice
signature (1c). The shared enum (VS-2) and the extensible record (VS-3)
stay **deferred with triggers, not rejected**: Phase 7b declines to
invent a cross-container vocabulary for two containers but pins the
storage so adopting one later is additive. The owner judgment invited
here is the carrier shape (VS-1a vs 1b vs 1c) and the agreement that
"`Phase 7b VS-1a is the minimum, and VS-2/VS-3 are additive`" — choosing
"generalizable parent-data" (DD-001 CB-B) buys the *reserved* path (the
condition), not an *immediately shared* carrier.

## SI-4: Canonical textual-IR shape for placement slots

### Context

SI-1 fixes the *Rust* IR abstract form (node + optional placement
payload). What it does **not** fix is the **textual** IR — the
build-internal artifact `wasamoc` emits and the loader re-parses — for the
placement slot. Three things must be unambiguous before the parser /
emitter / loader / roundtrip test are written by different hands: (i) does
the Grid `Cell` wrapper survive in textual IR, or is it normalised to the
child-slot record at emit time; (ii) what does the canonical textual slot
record look like; (iii) is stale old-form placement IR rejected or
dual-parsed. (i)+(iii) are the compatibility contract — owner-visible, not
an encoding the implementer should settle.

The current textual IR has no slot record: a child is an `IrNode` with
`props` / `children`, Grid `Cell` placement is emitted as `Cell` `IrProp`s
the loader extracts into `cell_placements`, ZStack placement as ordinary
child `IrProp`s.

### Options

1. **IR-A — keep `Cell` in textual IR; the loader slot-ises.** Textual IR
   continues to carry a `Cell` wrapper (and ZStack bare placement props);
   the loader converts both into the child-slot record at load time.
   - What you gain: smallest emitter change; old textual IR keeps parsing.
   - What you give up: textual IR and runtime model disagree (the split
     persists in the artifact); the loader keeps re-deriving the record;
     "canonical IR shape" is never expressed where roundtrip can pin it.
2. **IR-B — normalise to a child-slot record in textual IR; reject +
   regenerate old `Cell` placement IR (recommended).** Textual IR emits
   the child-slot record directly (both Grid `Cell` sugar and ZStack
   `slot.*` lower to it at emit); stale `Cell`/bare-placement textual IR
   is rejected with a named loader diagnostic and regenerated by
   `wasamoc` (matches SI-1 reject+regenerate and DD-001's no-long-lived-
   alias stance).
   - What you gain: textual IR expresses the canonical model; roundtrip
     pins one shape; loader is single-form; the build-internal IR is free
     to break because `wasamoc` regenerates it every build.
   - What you give up: an emitter + loader change landing together; old
     hand-held IR snapshots (none ship) would not load.
3. **IR-C — transitional dual-parse.** Loader accepts both the old `Cell`
   form and the new record for a transition window.
   - What you gain: nothing durable pre-1.0 — the IR is build-internal and
     regenerated, so there is no external producer to transition.
   - What you give up: a dual-form parser carried for no real consumer;
     the drift class SI-1 closes structurally re-enters via the legacy arm.

### Canonical textual-IR shape (recommended IR-B — normative skeleton)

The slot **grammar skeleton is normative** (so parser / emitter /
roundtrip test do not each invent it); only surface trivia (whitespace,
inter-field comma vs newline, key ordering within a placement block) is
the emitter's to fix. The fixed skeleton:

```text
child {
  placement <kind> { <key>: <const>, ... }   # <kind> ∈ { grid, zstack }
  node <Widget> { ... }
}
```

Both authored forms lower to this one record (SI-1 `{ node, placement }`):

```text
# ZStack child (authored slot.h-align / slot.v-align)
child { placement zstack { h-align: end, v-align: center }
        node Text { ... } }

# Grid child (authored via Cell { row/column/span/align } — PM-1 sugar)
child { placement grid { row: 1, column: 0, span: 2, align: stretch }
        node Button { ... } }
```

Normative in the skeleton: the `child` / `placement <kind>` / `node`
keywords and nesting, the `<kind>` set, and that placement values are
constants. Non-normative (emitter trivia): exact whitespace, comma vs
newline separators, and key order inside a placement block. The Grid
`Cell` *authoring* wrapper (DD-001 PM-1) is sugar that lowers to the same
`child { placement grid { … } node … }` record; it does **not**
survive as a `Cell` node in textual IR under IR-B.

### Stale-form diagnostic

Old-form placement IR (a `Cell` node with placement `IrProp`s, or bare
ZStack placement props on a child) is rejected by a **named loader
diagnostic** (e.g. `legacy-placement-ir-form`), not silently slot-ised —
the build re-emits the canonical record. The name is the forcing artifact;
the spelling is the emitter's.

### Recommendation

**IR-B (normalise to a child-slot record in textual IR + reject +
regenerate)**, because the textual IR is build-internal and regenerated,
so expressing the canonical record there costs only an emitter+loader
change while buying a single-form loader and a roundtrip-pinnable shape;
IR-A keeps the split in the artifact and IR-C carries a dual parser for a
non-existent external producer. This is owner-visible as the
**compatibility contract** (reject old `Cell` placement IR vs keep it), so
it is recorded for owner confirmation, not settled by the implementer.

## Decision outcome

TBD (Proposed). Filled at the Accepted flip with the P7-006 verb, the
internal model (IM-n), the IR carrier + compatibility policy (SI-1), the
Grid-migration disposition (SM-A / SM-B / SM-C, conditional on DD-001's
**CB-A/CB-B boundary** — SM-B under CB-B including PM-1), the child-slot
value-space shape (VS-n; recommended minimum carrier VS-1a) + CB-B
integration condition, the **canonical textual-IR shape (IR-A/B/C) and
stale-form compatibility contract (SI-4)**, and the bindability policy +
trigger.

## Spec impact

`docs/architecture.md`:

- The chosen placement internal model (under the recommendation:
  child-slot-carried, IM-4), stated as parent-interpreted per-container
  placement carried by the child slot, re-connecting the
  ZStack-implemented model with Grid and the DD-001 surface. The
  value-space shape (under the recommendation: per-container payload
  behind one slot record, VS-1) is stated together with the CB-B
  integration condition that keeps a later shared / extensible carrier an
  additive change (no out-of-slot storage; one `slot.*` admission path).
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
- **Bindable placement** — not implemented; constant-per-instance. The
  named re-visit trigger and the joint `BindingTarget` + child-slot
  effect-lifecycle landing are in §Dependencies (shared with DD-001).
- **Keyed child metadata / retained identity** — rejected for this phase
  (IM-5); identity baseline stays positional (DD-M3-P7-005).
- **Grid structural mutation under `Cell`** — out of scope unless SM-B
  pulls the storage migration in; the mutation paths themselves
  (`for`/`if` of `Cell`s) remain deferred with the DD-M3-P7-006 trigger.

## Revision history

- 2026-06-19 — Initial draft (Proposed). P7-006 = revise; internal-model
  options IM-1..5; IR + structural-mutation sub-issues; recommendation IM-4
  phase-wide, Grid migration conditional on DD-001.
- 2026-06-20 — Strategic / recommendation-choice / implementation-readiness
  review folds reflected (Status: Proposed; no recommendation reversed):
  added SI-3 (value-space, recommended carrier VS-1a) and SI-4 (canonical
  textual-IR normative skeleton, IR-B); re-keyed SI-2 Grid migration on the
  CB-A/CB-B boundary (SM-B under CB-B incl. PM-1); sharpened the
  bindability re-visit trigger.
