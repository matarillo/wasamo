# M3-Phase 7b — implementation handoff

> **Status: finalised at phase close (2026-06-24).** Per
> [workflow.md §6.3](../../../procedures/workflow.md) and
> [retrospectives.md](../../../procedures/retrospectives.md) item 15,
> this file is the confirmed phase-close deliverable. It carries the
> **DD-set carry-forward residuals** plus the **mid-phase-surfaced
> residuals** (T5 / T6b, distilled from the T7 candidate ledger in
> [log.md](./log.md)) and the **Main Learnings**. The deferred-items
> **正本** (activation triggers + responsibility landings) remains the
> framing scope table; this file is the forward-carry input to the Phase 8
> pre-doc, not a parallel table.

## Main Learnings

- **Last-task / phase-end ownership split holds.** T7 (the final step)
  rightly owned the Moment 2 docs sync, the local clean rebuild, the M3
  `plan.md` row flip, and the candidate ledger; the phase-end batch owns
  the CI run id, this handoff finalization, the phase-end retrospective,
  and the `preamble.md` `active → closing` flip. Keeping the spec /
  architecture / M3-plan flips on T7 (not the phase-end batch) is what the
  T7 review corrected in the `preamble.md` Lifecycle wording.
- **Pin the living spec to landed *source*, not to the design draft.** The
  T7 Moment 2 sync found two divergences a status-only flip would have
  frozen: the §5 AST `PlacementBind` variant that the implementation never
  adopted (it rides `PropertyBind`), and the `Widget { node, slot_data }`
  struct-variant sketch the implementation landed as the tuple
  `Widget(IrChildSlot)`. Reading the type definitions (not grepping) is
  what surfaced them.
- **A "constraint finding" can be two independent problems.** The T5
  Grid-in-ZStack finding was a checker bug (problem A, fixed in T6b) **and**
  a long-deferred sizing gap (problem B, carried). Bundling them under one
  "deferred" label nearly hid the in-scope fix; git-verifying that 7b
  changed no layout maths is what separated them.

Forward-carry material for the next phase's pre-doc framing. The next
planned implementation phase is **M3-Phase 8** (editorial: public draft
A12 + full gallery A1). A few entries target the **pre-1.0
stabilization** window or later feature phases.

The deferred-items **正本** (with activation triggers and responsibility
landings) is the framing scope table
([../requirements/framing.md §Out of scope](../requirements/framing.md));
this file points to it rather than re-deriving a parallel table.

## Known carry-forward candidates (from the ADR set)

- **PM-2 → PM-1 / PM-3 pre-1.0 wrapper-rule decision.** Phase 7b ships
  **two** author forms for Grid placement (`Cell` and direct `slot.*`) —
  a deliberately **provisional** state with **no normative canonical
  form**. A rule for *which widgets / containers may use a wrapper form*
  must be decided before 1.0, resolving Grid toward PM-1 (keep `Cell`,
  drop Grid direct `slot.*`) or PM-3 (drop `Cell`). Re-triggers
  (DD-M3-P7b-001 §Out of scope): a new container wants a wrapper form; a
  public code-construction API / builder is designed; custom containers /
  custom slot attrs are designed; the first non-layout parent-data is
  introduced; **Wasamo reaching 1.0 (hard deadline)**. Carry path: here
  → folded into the **M3 `handoff.md`** at milestone close as a pre-1.0
  residual.

- **VS-2 / VS-3 `SlotData` carrier triggers.** `SlotData` ships as a
  closed `Grid` / `ZStack` enum (VS-1a) under the CB-B integration
  condition (no out-of-slot storage; one `slot.*` admission path).
  Trigger to **VS-2** (collapse variants into a shared placement enum):
  a third placement-bearing container with overlapping keys. Trigger to
  **VS-3** (additive enum → struct, no rename): the first non-layout
  parent-data (hit-test / focus / accessibility), designed jointly with
  the framing R1 generic-modifier decision.

- **Grid structural-mutation trigger (DD-M3-P7-006 recursive).** SM-B
  migrated Grid **storage** onto `SlotData` this phase, but **not** the
  mutation paths: no `for` / `if` of `Cell`s is built. The DD-M3-P7-006
  recursive trigger stands — Grid is now child-slot-carried, so a future
  Grid mutation path is built against the migrated model from the start.

- **Bindable / reactive placement trigger.** Placement is
  constant-per-instance this phase; a binding RHS is rejected by a named
  diagnostic. Re-trigger (shared DD-001 / DD-002): a concrete app needs
  placement that varies after construction — designed **together with**
  the `BindingTarget` machinery (a new binding-target variant) and the
  child-slot **effect lifecycle** (placement rides the child-slot record,
  so a future per-item placement effect attaches / detaches with that
  slot), not as a `slot.*`-local addition.

- **Default-alignment unification.** Grid `stretch` / ZStack `center`
  stay per-container; the surface unification does **not** unify the
  defaults. Re-trigger: the public draft finds the default mismatch a
  real explicability debt, or an app needs cross-container default
  consistency. Lands in a future layout-behavior phase.

- **Placement key/value spelling revision** (e.g. `h-align` → `hAlign`).
  Existing spelling inherited unchanged. Re-trigger: a DSL
  naming-convention / ergonomics pass, or public-draft stabilization.

- **FD-7b-D future code-construction constraint.** No public
  code-construction API / ABI added. The only recorded constraint is
  non-committal: a future builder must **not** express placement as a
  generic child property setter (it would re-introduce the
  intrinsic-widget-property reading). The positive shape (parent-scoped
  insertion / child-slot builder) is non-normative and not frozen.

## Mid-phase-surfaced residuals (T5 / T6b — not in the ADR set)

These surfaced during implementation (the T0 ADR set predates them) and
were distilled into the T7 candidate ledger ([log.md](./log.md)). They are
**not** Phase 7b regressions.

- **Problem B — author-controllable `width` / `height` sizing (the
  primary mid-phase residual).** A Fill-default container (Grid / ZStack)
  nested on a Shrink ancestor axis collapses to 0×0; with T6b the checker
  now *accepts* `slot.*` on a Grid that is a ZStack direct child, but the
  Grid only **renders** when the ZStack has a definite size. This is the
  symptom of the long-deferred explicit-sizing surface (deferred since
  M3-Phase 2), **not** a slot-redesign regression — git-verified that 7b
  changed neither `measure_grid` (Fill→0) nor `axis_is_stretchy`. Live
  home: [docs/notes/author-controllable-sizing.md](../../../../docs/notes/author-controllable-sizing.md).
  **Responsibility: a Vision DR at Phase 8 framing** assigns the milestone
  home; **hard backstop = pre-1.0 / M6 ABI-freeze prep** (ABI impact
  pending — an explicit `width` / `height` surface may touch the value
  union). Default-center / start / end alignment being a visual no-op on a
  Fill container is a facet of the same gap, not a separate item.

- **Phase-8 removal of the placement-demo verification surface.** The T5
  placement-demo sub-screen in `examples/gallery/gallery.ui`
  (`is_placement_demo_open` state + button + `if`-overlay, marked for
  Phase-8 removal) and its capture driver
  `evidence/capture-placement-demo.ps1` are throwaway verification
  scaffolding. Re-trigger: the Phase 8 close cleanup sweep that removes the
  per-phase gallery verification surfaces (P5 Footer clip, P6/7 lightbox,
  P7 reactive list, **P7b placement-demo**).

- **`aspect`-in-cell arrange abort.** An `aspect` Box in a Grid cell
  aborts arrange under an unbounded intrinsic probe
  (`BoxAspectUnboundedBoth`), silently dropping the subtree. Pre-existing
  layout behaviour (the documented aspect-needs-a-bounded-axis rule), a
  facet of the same sizing gap; folds into the Problem B Phase 8 triage.

- **Capture-driver layout-coupled coordinates.** The re-tuned navigation
  coordinates in `capture-placement-demo.ps1` assume the current gallery
  layout; a later layout change re-staleness them (as happened to the
  inherited script). Re-trigger: whoever next changes the gallery layout
  re-derives the coordinates (the script header documents its assumption).

The T5-recorded **Grid-as-ZStack-child checker reject** is **resolved**
(T6b fixed the checker half — `slot.*` on a Grid that is a ZStack direct
child now compiles); only its sizing half survives as Problem B above, so
it is not carried as an open checker-design decision.

## Framing deferred-items reconciliation (正本 rows → handoff status)

The deferred-items **正本** is the framing scope table
([../requirements/framing.md §Out of scope](../requirements/framing.md));
this handoff **refines** it, it does not replace it. Every 正本 row is
reconciled below so none is dropped — no Phase 8 row is silently lost.

| 正本 row (framing) | Handoff status | Where / responsibility |
|---|---|---|
| Public code-construction API / ABI | **Carried** | The FD-7b-D non-committal constraint above (no generic child property setter); responsibility = future code-construction phase / M6 ABI freeze prep. |
| Generic modifier system | **Still deferred — no Phase 8 action** | Future DSL ergonomics / styling phase; the syntax-doesn't-narrow-future-modifiers reservation is in DD-001 (implementation deferred). Touched only as the joint-design partner of the VS-3 non-layout-parent-data trigger above. |
| User-defined containers and custom slot attributes | **Still deferred — no Phase 8 action** | Component / custom-layout phase; its placement-key-collision reservation rides the PM-2 wrapper-rule decision and the VS-2 carrier trigger above (a custom container is a "new container" / "new placement-bearing container" re-trigger). |
| Non-layout parent-data | **Covered by the VS-3 trigger** above | The first non-layout parent-data (hit-test / focus / accessibility) is exactly the `SlotData` enum → struct additive migration trigger (VS-3), designed jointly with the framing generic-modifier decision. M4+ input / accessibility phase. |
| Keyed child metadata / retained identity | **Still deferred — no Phase 8 action** | Future keyed-identity / reorder phase. **Orthogonal to placement**: Phase 7b placement is the structural parent-child edge, not element identity / keyed diff; a `key:` surface is a separate problem and is not opened by the child-slot record. |
| Grid structural mutation under `Cell` | **Covered by the Grid structural-mutation trigger** above | DD-M3-P7-006 recursive; storage migrated onto the child slot this phase, mutation surface not built. Future Grid mutation phase. |
| Layout algorithm changes | **Still deferred — no Phase 8 action** (distinct from Problem B) | Future layout-primitive-refinement phase; no new measure-arrange semantics this phase. **Problem B (author-controllable sizing) is a *different* residual** — it is a missing author *surface* over the existing algorithm, not a change to the geometry itself; do not conflate the two. |
| Backward-compatibility guarantee for old placement syntax | **Resolved for now — reopens only under a public compat policy** | DD-001 shipped **no long-lived alias**: bare ZStack `h-align` / `v-align` are rejected (a firing test), the loader rejects + regenerates stale IR. Pre-1.0 keeps migration minimal. Re-trigger: a pre-1.0 public compatibility policy declares the shipped syntax stable, or external users depend on it. Pre-1.0 compatibility-policy phase. |

## Phase 8 (editorial) note

Phase 8 promotes `docs/dsl_spec.md` to the first public draft (A12) and
assembles the full gallery (A1). The placement surface is **frozen** by
Phase 7b for exactly this reason — Phase 8 should read the synced
`docs/dsl_spec.md` §4.16 + `docs/architecture.md` §6.8.6 / §6.8.4
directly and must **not** re-decide the surface. Two items Phase 8 must
surface (not silently freeze):

- the **PM-2 provisional two-form Grid state** above (the public draft must
  flag that the wrapper-rule decision is pre-1.0, not settled); and
- the **Problem B sizing Vision DR** — Phase 8 framing is the assigned
  home for the author-controllable `width` / `height` decision, so the
  public draft must not present the Fill-default sizing as final.
