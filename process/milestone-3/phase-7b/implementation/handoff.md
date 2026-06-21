# M3-Phase 7b — implementation handoff

> **Status: skeleton — finalised at phase close.** Per
> [workflow.md §6.3](../../../procedures/workflow.md) and
> [retrospectives.md](../../../procedures/retrospectives.md) item 15,
> this file is written as a confirmed deliverable at the **phase-end
> retrospective**, not during implementation. The entries below are the
> **known carry-forward candidates from the ADR set**, recorded now so
> they cannot be lost; the phase-end retro confirms, expands (with the
> task-retrospective item-10 `carry-forward` candidates), and re-cuts
> them into the final structure. Do not treat this skeleton as the
> finalised handoff.

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

## Phase 8 (editorial) note

Phase 8 promotes `docs/dsl_spec.md` to the first public draft (A12) and
assembles the full gallery (A1). The placement surface is **frozen** by
Phase 7b for exactly this reason — Phase 8 should read the synced
`docs/dsl_spec.md` §4.16 + `docs/architecture.md` §6.8.6 / §6.8.4
directly and must **not** re-decide the surface. The one open placement
item Phase 8 must surface (not resolve) is the **PM-2 provisional
two-form Grid state** above, so the public draft does not silently freeze
it.
