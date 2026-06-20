# M3-Phase 7b — Parent-interpreted placement attributes: Architecture Decisions

**Phase:** M3-Phase 7b (parent-interpreted placement attributes — owner-inserted corrective)
**Date:** 2026-06-19
**Status:** Proposed

## Context

Phase 7b is a **corrective phase**, inserted between Phase 7 and
Phase 8 by the owner on 2026-06-19 (tier-2 additive plan revision —
[../../plan.md §Revision log](../../plan.md)). It adds **no new layout
primitive and no new app feature**. Its purpose is to align the
parent-interpreted placement surface that Phases 5–7 shipped piecemeal
before the M3 public draft (Phase 8) freezes it.

By the end of Phase 7 the gallery surface carries three placement
facts that do not agree on how placement is expressed or stored:

- **Grid** authors placement through a `Cell` wrapper (`row` /
  `column` / `span` / alignment), and stores it in a parallel
  `cell_placements` vector kept alongside `children`
  ([wasamo-runtime/src/widget.rs](../../../../wasamo-runtime/src/widget.rs)).
  Grid is static-only, so the parallel vector has not yet been migrated
  ([DD-M3-P7-006](../../phase-7/decisions/dd-m3-p7-006-placement-storage-and-structural-side-effects.md)
  held the migration behind a trigger).
- **ZStack** authors placement as direct child props (`h-align` /
  `v-align`) consumed by the parent, and — as of Phase 7 — stores it
  **child-carried** on the child slot (the parallel `zstack_placements`
  vector was removed; architecture.md §6.8.5).
- The two are authored with **different syntax** (wrapper vs direct
  annotation) for what is, in both cases, **parent-interpreted child
  placement**, and the defaults differ per container (Grid `Cell`
  alignment defaults `stretch`; ZStack defaults `center`).

Phase 8 is an **editorial** phase: it promotes the cumulative
`docs/dsl_spec.md` to first public draft (A12) and assembles the full
gallery (A1). It does not re-decide surface. If the placement
divergence reaches Phase 8 unexamined, the public draft either freezes
the inconsistency or forces surface re-litigation inside the close
phase — exactly what a pre-publication corrective phase exists to
avoid.

### The settled floor (not re-litigated)

The one premise Phase 7b treats as settled, agreed by the owner in
framing FD-7b-A:

> **Placement is parent-interpreted, not an intrinsic widget
> property.** `Text.text` and `Button.enabled` are attributes of the
> widget itself; Grid `row` / `column` and ZStack `h-align` /
> `v-align` describe how a child is treated *by its immediate parent
> container*. Both the "container-specific sugar" and the
> "generalizable parent-data" readings agree on this floor.

What stays **open and decided by the DDs**, not by this preamble:
whether placement is container-specific DSL sugar or a generalizable
cross-container parent-data grammar; whether the author surface is
unified or its asymmetry kept as a documented principle; and whether
storage is child-slot-carried (a leading hypothesis on the observed
Phase 6 drift, **not** a default) or otherwise. The framing's
verification strategy, scope table, and risk register are the
authority for these boundaries
([../requirements/framing.md](../requirements/framing.md)).

### Acceptance relation (contingent — not pre-decided)

The existing acceptance criteria never name the placement *author
surface*: A2 promises Grid as "1 cell 1 child, star sizing +
spanning", A4 promises ZStack as "sibling z-order by document order";
neither promises *how placement is written*, still less a
cross-container-coherent way to write it. Whether Phase 7b needs a new
AC is therefore **contingent on DD-M3-P7b-001's outcome** and is fixed
at that DD's Accepted flip (FD-7b-F):

- **(a)** if DD-001 changes the public author-facing surface, a new AC
  is added or A2 / A4 / A12 wording is refined under the M3
  acceptance-criteria revision exception, ROADMAP is mirrored, and the
  plan Revision log records it;
- **(b)** if DD-001 holds the surface (documenting the asymmetry as a
  principle), Phase 7b discharges under the existing **A11**
  (per-phase `.ui` / IR / `wasamoc` / runtime / `docs/dsl_spec.md` /
  `examples/gallery/` sync) and **A12** (public-draft explicability).

Both A11 and A12 apply in either branch. This preamble does not
pre-commit either way; doing so would substitute the goal for the DD.

### Owner-agreed framing decisions

The pre-doc framing was owner-aligned on 2026-06-19 and is recorded in
[../requirements/framing.md](../requirements/framing.md) ("オーナー合意の記録"
and "Owner-agreed framing decisions"). The five framing decisions this
ADR consumes:

- **FD-7b-A** — Phase 7b thesis: corrective phase; the settled floor is
  "placement is parent-interpreted, not intrinsic"; container-specific
  vs generalizable and child-slot vs parallel storage are DD-open axes,
  not conclusions. No new layout primitive.
- **FD-7b-B** — DD slate: two DDs (DD-001 author surface, DD-002
  internal model and construction boundary); migration / compatibility
  / diagnostics are sub-issues of those DDs, not independent DDs.
- **FD-7b-C** — architectural family: trigger 1 (M3 DSL spec drafting)
  fires; VDR requirement is **not pre-decided** — each DD-001 surface
  option's family impact is confirmed, and if all sit within family (1)
  the record is revise-in-place (no VDR); a pivot-level choice escalates
  to a vision decision record. Expectation is confirm-within-family
  (all options are tree-description grammar, not view-function
  re-execution), settled at the DD exit.
- **FD-7b-D** — future code-construction boundary: no API/ABI added;
  the only non-committal constraint recorded is "placement is not
  expressed as a generic child property setter"; the positive shape
  (parent-scoped insertion / child-slot builder) is non-normative and
  not frozen.
- **FD-7b-E** — scope: if Accepted, Phase 7b implements the chosen
  surface across parser / checker / lowering / runtime / examples (not
  docs alone); no new layout algorithm; generic modifier system, custom
  layout containers, non-layout parent-data, keyed identity, and public
  API design are carried out with activation triggers. **Exception:**
  DD-001 Option 5 (parent-declared namespace) is a pivot-level surface
  7b cannot implement in-phase — selecting it does **not** Accept the DD
  set as a 7b implementation directive but triggers an M3 plan revision
  inserting Phase 7c, which decides the mechanism (DD-001 §Options
  Option 5, "Accepted-time process meaning").

### Architectural-family confirmation (FD-7b-C)

Phase 7b fires `architectural-family.md` re-evaluation trigger 1 (M3
DSL spec drafting): it re-organises author-facing placement grammar,
which touches the public-contract layer. Per FD-7b-C the VDR
requirement is **not pre-empted as a conclusion**. Each DD-001 surface
option (edge wrapper / fixed prefix / no-prefix sugar / XAML-style
attached property / parent-declared namespace / documented status quo)
records a one-line family-impact judgment; the working expectation is
that all sit **within family (1)** because each is a tree-*description*
grammar, not a view-function-with-re-execution (family 2) construct,
and Phase 7b introduces no host-language scope modifier or embedded
scripting runtime. If the expectation holds, the family note is updated
**revise-in-place** at Moment 1 / Moment 2 (no VDR); only a pivot-level
DD choice escalates to a vision decision record. This conditional
operation is the application of DD-V-026's proportional-recording
principle (heavy artifacts reserved for thesis reversal / family-pivot
changes), and is the same shape as the Phase 7 Moment-2
confirm-no-VDR.

### End-state shape this phase re-connects (verified at drafting time)

- **`wasamo-runtime` placement storage**
  ([wasamo-runtime/src/widget.rs](../../../../wasamo-runtime/src/widget.rs)):
  ZStack placement is **child-carried** (Phase 7 ST2); Grid
  `cell_placements` remains a **parallel vector** (static-only,
  migration trigger held). DD-002 re-connects these.
- **DSL surface** ([docs/dsl_spec.md](../../../../docs/dsl_spec.md)):
  Grid uses the `Cell` wrapper / placement carrier (not a free-standing
  runtime widget); ZStack uses direct child placement annotations (no
  `Layer` / `Cell` wrapper). DD-001 re-connects these.
- **Architecture contract**
  ([docs/architecture.md](../../../../docs/architecture.md) §6.8.5 /
  §6.9): ZStack child-carried placement and the splice primitive
  side-effect contract from DD-M3-P7-006 are the storage baseline
  DD-002 consumes / revises / supersedes.
- **C ABI** ([docs/abi_spec.md](../../../../docs/abi_spec.md)):
  handle-based with tree mutation in the stable core; M1 experimental
  constructors deferred. Phase 7b adds no code-construction surface
  (FD-7b-D); abi_spec is no-touch unless a future-compat note proves
  necessary (judged in DD-002).

## Decisions

The Phase 7b ADR carries the two framing-slate DDs (FD-7b-B):

| DD | Title | Status |
|---|---|---|
| [DD-M3-P7b-001](./dd-m3-p7b-001-parent-interpreted-placement-authoring-surface.md) | Parent-interpreted placement authoring surface | Proposed |
| [DD-M3-P7b-002](./dd-m3-p7b-002-placement-internal-model-and-construction-boundary.md) | Placement internal model and construction boundary | Proposed |

`Decision summary` cells are intentionally left out of this table while
both DDs are `Proposed`: filling a decision summary before owner review
would pre-empt the comparison. ADRs stay `Proposed` through the full
review pass and are not rushed to an Accepted flip; the table gains
decision summaries at that flip.

## Cross-DD decision dependency

DD-001 (author surface) and DD-002 (internal model) are coupled: the
conceptual boundary chosen in DD-001 (container-specific vs
generalizable) constrains DD-002's internal model space, and DD-002's
storage choice (child-slot vs parallel) bounds whether DD-001 can
promise per-iteration / bindable placement. DD-001 owns the conceptual
boundary and the author-visible bindability question; DD-002 owns the
storage model, the textual-IR compatibility policy, and the
construction boundary. Neither is Accepted without the other (they ship
as one phase ADR set).

## Scope and out of scope

The deferred-items **正本** (with activation triggers and
responsibility landings) is the framing scope table
([../requirements/framing.md §Out of scope](../requirements/framing.md));
this ADR does not duplicate it. Out of Phase 7b scope by decision:
generic modifier system; user-defined containers and custom slot
attributes; non-layout parent-data (hit-test / focus / accessibility);
keyed child metadata / retained identity; Grid structural mutation
under `Cell` (unless a DD explicitly pulls it in); layout algorithm
changes; backward-compatibility guarantees for old placement syntax;
and any public code-construction API / ABI (FD-7b-D / FD-7b-E).

In scope: the Grid / ZStack author-facing placement surface and its
unification policy; the placement-vs-ordinary-property boundary; the
parser / checker / lowering / textual-IR / loader / runtime
representation of placement; admission / rejection rules; existing
examples / gallery `.ui` migration policy; the `docs/dsl_spec.md`
placement chapter and `docs/architecture.md` placement model; the
relationship to DD-M3-P7-006; and the future-API non-committal
constraint.

## Verification closure (what counts as Phase 7b evidence)

Per the framing verification strategy
([../requirements/framing.md §Verification strategy](../requirements/framing.md))
and the positive-control discipline
([../../../../AGENTS.md §Testing rules](../../../../AGENTS.md) — a
single static frame a wrong implementation could equally produce is not
evidence), Phase 7b closes only when all of the following are observed
(exact set finalised against the chosen options at the Accepted flip):

1. **`wasamoc check` evidence (pure logic).** Positive: the chosen
   placement syntax compiles and lowers to the chosen storage model for
   both Grid and ZStack fixtures. Negative: placement attrs are
   rejected under parents that do not admit them; stray placement is
   rejected; an ordinary widget-property check does not accidentally
   accept a placement attr; every non-chosen / deferred form named in
   DD-001's matrix fires its own diagnostic; compatibility-alias tests
   if aliases are admitted.
2. **Lowering / textual-IR roundtrip / loader evidence.** The chosen
   DSL surface lowers to the chosen placement storage; the loader
   re-rejects malformed placement metadata; stale old-IR forms are
   accepted via migration or rejected with a named diagnostic per
   DD-002's compatibility policy; placement defaults preserved.
3. **Windows-runtime integration evidence (mock-free, CI-gated,
   fail-not-skip).** ZStack layout reads placement from child-slot
   storage (not a parallel vector); Grid layout reads placement from
   its chosen path if Grid is migrated in-phase; structural insert /
   remove under ZStack preserves child order, Visual sibling order,
   placement, and layout invalidation; destroy / detach leaks no
   placement metadata; no-placement containers carry `None`-equivalent
   and pay no placement-specific cost outside the generic path.
4. **Assistant-visible GUI evidence + positive control.** Gallery /
   lightbox / overlay show ZStack child placement at the same positions
   as the old surface; a Grid sub-screen shows cell placement at the
   same positions. Positive controls: a `slot.h-align: end`-equivalent
   child is right-aligned **and** a contrasting alignment frame lands
   at a different position; Grid row / column / span / alignment is
   visibly reflected and stray / omitted placement falls to the
   expected default. Launch + DPI-aware screenshot + assistant analysis
   is the close artifact for GUI-render tasks; owner human-visible
   smoke remains a separate gate.
5. **A12 spec-closure gate (non-test).** `docs/dsl_spec.md` carries the
   author-facing placement surface and invalid examples at the
   external-reader bar; `docs/architecture.md` carries the chosen
   placement model and structural-mutation contract; the Moment 1 →
   Moment 2 marker flip is completed.

## Implementation gate expectations

Per the framing, most Phase 7b implementation tasks are expected to
trip: **#1 semantic migration** (`IrProp` / placement extraction /
child traversal / loader / validator / roundtrip / arrange-loop
call-site audit), **#2 missed side effects** (placement migration
co-touches layout invalidation, Visual sibling order, registry
teardown, effect ownership), **#3 parallel data drift** (the central
trap — close the child-list-vs-placement desync structurally or
enumerate the remaining paths explicitly), **#4 untested authored
branch** (new syntax / reject diagnostics / aliases get direct firing
tests), **#5 carry-forward** (future code-construction API, generic
modifier, custom container, keyed identity, Grid structural-mutation
trigger recorded in the handoff), and **#7 GUI positive control**. The
binding selection is recorded per-task at task start per
[../../../procedures/implementation-gates.md](../../../procedures/implementation-gates.md).

## Upstream document revisions (Moment 1 / Moment 2)

Per-review-concern commit rule applies
([../../../../AGENTS.md §Commit rules](../../../../AGENTS.md)). Exact
touch / no-touch judgments depend on the chosen options and are
finalised at the Accepted flip; the anticipated set:

**Moment 1 — ADR Accepted commit set (design-spec draft):**

- This directory — ADR `Status: Accepted` flip.
- [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) — **touch
  (expected).** Author-facing placement surface chapter + invalid
  examples for the chosen surface (DD-001); grammar additions if the
  surface introduces a prefix / namespace / wrapper; any stale
  placement prose swept in the same touch. No DD option labels in spec
  prose (living-spec vocabulary rule).
- [`docs/architecture.md`](../../../../docs/architecture.md) — **touch
  (expected).** The chosen placement internal model and
  structural-mutation contract (DD-002); re-connection of the
  ZStack-implemented child-carried model with Grid and the DSL surface.
- [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no touch
  (judged, FD-7b-D);** revisited only if DD-002 finds an unavoidable
  future-compat note.
- [`docs/notes/architectural-family.md`](../../../../docs/notes/architectural-family.md)
  — alignment table + re-evaluation triggers updated with the Phase 7b
  surface-option family-impact confirm (revise-in-place per FD-7b-C),
  at Moment 1 or Moment 2.
- [`../../plan.md`](../../plan.md) — Phase 7b row populated; the
  contingent new-AC disposition recorded in the Revision log at this
  Accepted flip (branch (a) or (b)).
- [`../../../_roadmap.md`](../../../_roadmap.md) — **conditional touch
  (branch (a) only).** If DD-001's chosen surface changes the public
  author-facing surface (the recommended `h-align` → `slot.h-align` is
  branch (a)), the new AC or the A2 / A4 / A12 refinement is mirrored into
  the ROADMAP acceptance-criteria SSOT in the same Accepted-flip commit
  set (M3 acceptance-criteria revision exception). **No touch under branch
  (b)** (surface held, Option 0).
- `implementation/plan.md` / `log.md` — opened after the Accepted flip.

**Moment 2 — Phase close commit set (impl re-sync):** dsl_spec /
architecture markers flip to `closed; implementation-synced` with
divergence corrections; the architectural-family confirm entry lands if
not already; the plan row flips complete; phase-end retrospective + CI
run-id ownership per the final-task split.

## Inputs absorbed

| Source | Disposition | Consumed at |
|---|---|---|
| FD-7b-A — corrective thesis; parent-interpreted floor; open axes | Settled framing | §Context (settled floor); DD-001 / DD-002 |
| FD-7b-B — 2-DD slate; migration/compat/diagnostics as sub-issues | Structure | §Decisions |
| FD-7b-C — conditional architectural-family confirm / pivot-escalation | Discipline | §Architectural-family confirmation; Moment 1/2 write-back |
| FD-7b-D — future code-construction non-committal constraint only | Constraint | DD-002 construction boundary; abi_spec no-touch |
| FD-7b-E — implement-not-docs scope + containment | Constraint | §Scope; §Verification closure |
| FD-7b-F — new-AC requirement contingent on DD-001 | Constraint | §Acceptance relation; plan Revision log |
| DD-M3-P7-006 — placement storage model + splice side-effect set | Baseline to consume/revise/supersede | DD-002 |
| docs/architecture.md §6.8.5 / §6.9 — ZStack child-carried, Grid parallel, defaults per container | Current-state input | §End-state shape; DD-002 |
| docs/dsl_spec.md — Grid `Cell` wrapper, ZStack direct annotation | Current-state input | §End-state shape; DD-001 |
| architectural-family.md — tree-with-bindings working hypothesis; DSL grammar = re-evaluation trigger | Family input | §Architectural-family confirmation |

## Revision history

| Date | Change |
|---|---|
| 2026-06-19 | Initial draft (Status: Proposed). Both DDs at Proposed pending owner review. Framing owner-aligned 2026-06-19 ([../requirements/framing.md](../requirements/framing.md) §オーナー合意の記録). Inserted into the M3 plan as a tier-2 additive revision the same day. |
