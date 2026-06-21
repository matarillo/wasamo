---
title: Parent-interpreted placement authoring surface
status: Accepted
phase: M3-Phase 7b
ac: branch (a) — PM-2 changes the public author surface for both containers (ZStack h-align → slot.h-align; Grid gains direct slot.*); new AC for the unified placement surface, or an A2/A4/A12 refinement, recorded at the Accepted flip; see §Decision outcome and preamble §Acceptance relation
date: 2026-06-19
related:
  - ./preamble.md
  - ./dd-m3-p7b-002-placement-internal-model-and-construction-boundary.md
  - ../requirements/framing.md
---

# DD-M3-P7b-001 — Parent-interpreted placement authoring surface

**Status:** Accepted

## Context

How should an author write parent-interpreted placement metadata in
`.ui`?

Today Grid writes it through a `Cell` wrapper and ZStack writes it as
direct child props (`h-align` / `v-align`). Both are
parent-interpreted placement, but the author-facing surfaces disagree.
Entering the M3 public draft (Phase 8) this way makes the surface hard
to explain — is `h-align` a widget property or a parent-relationship
attribute? — and a child-property-shaped surface can mislead a future
code-construction API toward a generic child setter
([preamble §Context](./preamble.md)).

Per the owner prior, the options below are compared on **product merit
/ thesis fit first**; migration cost is a tie-breaker, never a
rejection ground.

**Settled floor (not re-litigated).** Placement is parent-interpreted,
not an intrinsic widget property (FD-7b-A). This DD does **not**
re-litigate that; it decides how the author *writes* something that is
already agreed to be parent-interpreted.

## Dependencies

- **Consumes** framing FD-7b-A (the parent-interpreted floor that frames
  every option) and FD-7b-C (each surface option records a one-line
  family-impact judgment; the chosen option's line is written back to
  `architectural-family.md` at Moment 1 / Moment 2 — revise-in-place, no
  VDR expected).
- **Couples to**
  [DD-M3-P7b-002](./dd-m3-p7b-002-placement-internal-model-and-construction-boundary.md):
  the conceptual boundary and the author-visible bindability read
  decided here constrain DD-002's internal-model and reactive-boundary
  choices, and DD-002's storage choice bounds whether per-iteration
  placement can vary. The two are **Accepted together** as one phase ADR
  set, so neither holds the other's decision twice.
- **Bindability (carried, not decided here).** Phase 7b does not
  implement bindable placement; the surface only needs to stay
  consistent with DD-002's *constant-per-instance* storage stance. The
  author-surface read is recorded as a future-possible concept (see the
  Main decision §Forward-compat impact), the storage/reactive boundary
  is DD-002's.

## Main decision — author-facing surface

The single load-bearing decision: the **conceptual boundary** plus the
**surface form** for writing parent-interpreted placement. The boundary
is the upper-level premise; the surface form is what an author types.
They are decided together because the boundary changes which surfaces
are coherent.

The conceptual boundary, two readings:

- **CB-A — container-specific sugar.** Placement is per-container
  syntax: Grid's `Cell` and ZStack's annotations are each a feature of
  that container, with no claim of a shared grammar.
- **CB-B — generalizable parent-data, scoped to layout this phase.**
  Placement is one instance of a cross-container "data the parent
  interprets about a child" grammar (e.g. a `slot.*` namespace),
  *restricted to layout placement in Phase 7b* but shaped so the same
  grammar could later carry other parent-data.

The surface options are evaluated under whichever boundary they assume;
each names the boundary it pairs with.

### Options

Each option carries a one-line **family-impact** note per FD-7b-C
(family (1) = tree-description grammar; family (2) = view-function
re-execution).

1. **Option 0 — principled asymmetry / documented status quo** (assumes
   CB-A). Keep Grid `Cell` (structured wrapper carrying `row` / `column`
   / `span` / alignment) and ZStack direct annotation, and **document
   the asymmetry as an intended model**: "placement that carries
   structural data uses a wrapper; simple alignment is written
   directly." A first-class option, judged as a deliberate model, not an
   accident.
   - What you gain: zero churn; matches shipped examples; family impact
     within family (1).
   - What you give up: the public draft still presents two shapes;
     "structural data ⇒ wrapper" is a post-hoc rationalisation a reader
     must be taught, and it does not make the parent-interpreted nature
     legible (ZStack `h-align` still *looks* like a widget property).
   - Technical risk: none — no grammar or storage change.
   - Follow-up if chosen: none — the asymmetry is the status quo; only
     the wording of the justifying principle is written into the spec.
     No new name, token, or syntax is decided.
2. **Option 1 — edge wrapper unification (`Cell` / `Layer`)** (CB-A or
   CB-B). Give ZStack a `Layer` wrapper mirroring Grid's `Cell`, so both
   containers place children through a structural wrapper.
   - What you gain: visually uniform "wrapper carries placement"; family
     impact within family (1).
   - What you give up: a `Layer` wrapper around every ZStack child is
     heavy for a single alignment pair; wrappers read as nodes — the
     opposite legibility risk from Option 0 (placement looks like a
     widget rather than parent-data).
   - Technical risk: new wrapper node in parser / IR / loader for
     ZStack; low but non-zero, and adds a node type the runtime must
     arrange transparently.
   - Follow-up if chosen: the ZStack wrapper name (`Layer` is only an
     example here, not a decision) — see SI-3 (resolved in pre-doc,
     before Accepted; not left to implementation).
3. **Option 2 — no prefix, semantics-only unification** (CB-A). Keep
   direct child attrs with no prefix, but unify the *semantics* (same
   admission / rejection / defaulting rules) and drop the Grid wrapper in
   favour of direct `row` / `column` on the child.
   - What you gain: lightest syntax; family impact within family (1).
   - What you give up: does nothing for the "widget-property-or-not"
     ambiguity and **maximises the name-collision risk** — a future
     custom container's slot keys collide with widget props with no
     namespace to disambiguate, forcing a fragile "parent consumes
     first" precedence rule + diagnostic.
   - Technical risk: the precedence rule grows more fragile as the
     widget vocabulary grows; collision is policed, not structural.
   - Follow-up if chosen: the post-`Cell` direct-key spelling (inherited
     `row` / `column` / `span` / alignment) and the "parent consumes
     first" precedence rule. The precedence rule is surface semantics —
     resolved in §Spec impact in pre-doc (or an added conditional SI if
     contested), never at implementation; the spelling is inherited. No
     new name.
4. **Option 3 — fixed prefix (`slot.` / `placement.` / `parent.` /
   `layout.`)** (CB-B). Write placement as a reserved-prefix attribute
   on the child: e.g. `slot.row` / `slot.column` / `slot.h-align`. Among
   the token candidates, **`slot.`** leads ("the slot this child
   occupies in its parent"; neutral about *what* the parent does with
   it; reserves room for non-alignment slot data later); `placement.`
   reads as the verb and is narrower; `parent.` reads as "a property *of*
   the parent"; `layout.` under-reserves if CB-B later carries
   non-layout parent-data.
   - What you gain: the prefix makes "this is parent-interpreted, not a
     widget property" legible at the call site, unifies Grid and ZStack
     on one grammar, and **structurally avoids the name-collision class**
     (`slot.h-align` is never confusable with a widget `h-align`); family
     impact within family (1) (prefix is a parse-level namespace, not
     re-execution).
   - What you give up: a new prefix token to specify and migrate examples
     to; the `Cell` structural-grouping affordance (grouping `row` +
     `column` + `span` + alignment into one node) must be re-expressed or
     kept as sugar.
   - Technical risk: bounded — a dotted-key lexeme distinguished from
     ordinary property keys; no expression-grammar impact.
   - Follow-up if chosen: the prefix token (SI-1) and the per-container
     `Cell` disposition (SI-2) — both resolved in pre-doc, before
     Accepted.
5. **Option 4 — XAML-style attached property (`Grid.row` /
   `ZStack.hAlign`)** (CB-A/CB-B). Qualify placement by the *parent
   type*.
   - What you gain: explicit about which parent interprets the attr;
     familiar to XAML readers; family impact within family (1), though
     it is the closest to a type-directed dispatch surface.
   - What you give up: couples the child's authored attr to the concrete
     parent type name (brittle under re-parenting / custom containers);
     heavier than a single neutral prefix for a tree whose parent is
     already structurally known.
   - Technical risk: the parser must resolve `Type.attr` against the
     parent type; re-parenting / future custom containers strain the
     coupling.
   - Follow-up if chosen: the attached-property spelling convention
     (`Grid.row` vs `Grid.Row`, `hAlign` vs `h-align`, the `.`
     separator) and how parent type names bind under re-parenting. If
     Option 4 becomes a live contender, these are resolved by a
     conditional SI added in pre-doc, before this DD is Accepted — not
     left to implementation.
6. **Option 5 — parent-declared modifier namespace** (CB-B). The parent
   container *declares* the slot attributes it admits; children use that
   namespace.
   - What you gain: the most extensible toward user-defined containers.
   - **Lightweight variant (5a) — internal slot-declaration table now,
     public declaration *syntax* later.** Built-in containers declare
     their admitted slot keys in an *implementation-side* registry (a
     table the checker/loader consult), and the user-facing
     *declaration syntax* for custom containers is opened only post-1.0.
     This is the intermediate between Option 3's fixed `slot.` prefix and
     full Option 5: it buys a uniform admission mechanism without
     publishing a namespace grammar in the M3 draft. From the author's
     view it collapses toward Option 3 — children still write `slot.*`;
     only the *source of admitted keys* differs — so for Phase 7b's two
     built-in containers it adds machinery the author never sees.
   - What you give up: over-built for two built-in containers in a
     corrective phase (framing R1 / R5); the custom-container case is
     explicitly deferred. Variant 5a narrows the cost (no public grammar)
     but its admission table is still machinery built now for a benefit
     that lands in the user-container era.
   - Technical risk: **the only family-pivot risk** — within family (1)
     for the declaration-as-data form, but *escalates toward family (2)*
     if the namespace becomes a host-language scope construct (not
     proposed here). Flagged per FD-7b-C as the option whose pivot must
     be checked at exit.
   - Follow-up if chosen: the namespace mechanism itself — how a parent
     declares its admitted slot keys, the child reference syntax, and
     where built-in containers' declarations live. This is a mechanism
     design beyond a corrective phase (framing R1 / R5); reserved, to be
     designed only if Option 5 is adopted (not pre-drafted here).
     Adopting Option 5 is an M3 responsibility discharged by inserting
     phase 7c to design the mechanism — it is not deferred to M4+.
   - **Accepted-time process meaning (Option 5 only).** Every other
     option, if Accepted, makes this DD set a **Phase 7b implementation
     directive** (preamble FD-7b-E: parser / checker / lowering / runtime
     / examples). Option 5 is the exception: its mechanism cannot be
     implemented inside 7b's corrective scope, so selecting it does **not**
     Accept this DD set as a 7b implementation directive. Instead the
     owner choice triggers an **M3 plan revision that inserts Phase 7c**,
     and 7c's own framing + ADR decide the namespace mechanism; the 7b
     surface DD is then closed as "surface decided = Option 5, mechanism
     delegated to 7c" rather than driving 7b code. This keeps the 7c
     insertion an explicit pre-Accepted plan decision, not a choice that
     slips in during 7b implementation.

### Forward-compat impact

A namespaced surface (Options 3 / 5) reserves two additive paths
without building them: **custom-container slot attributes** and
**non-layout parent-data** (hit-test / focus / accessibility). A
no-prefix surface (Options 0 / 2) narrows both. These are reservation
conditions, not features — the framing §Out of scope rows for those
items hold. Bindable placement is likewise *reservable* under a
namespaced surface and *foreclosed by nothing* here, consistent with
DD-002's constant-per-instance stance.

### Recommendation

**CB-B + Option 3 with the `slot.` prefix, paired with SI-2 PM-1** —
i.e. **model-level unification**: Grid and ZStack lower to one `slot.*`
parent-interpreted placement model, with **`Cell` retained as a
structural-grouping sugar** over that model (not removed) so Grid authors
keep the grouped-payload affordance.

This bundles two distinct senses of "unify", which the owner judges
separately:

- **Model / semantic unification** — both containers store and mean the
  same parent-interpreted `slot.*` placement. This *is* recommended;
  PM-1 carries it.
- **Surface-visible unification** — the author *writes* the same direct
  `slot.*` form under both containers (Grid drops `Cell`, or admits
  `slot.*` alongside it). This is **PM-2 / PM-3 and is *not* recommended
  for Phase 7b:** under PM-1 the authored surface stays asymmetric
  (Grid = `Cell`, ZStack = `slot.*`).

So the recommendation does **not** make the author-facing surface
uniform; it unifies the *model* and keeps a **documented, narrowed**
authored asymmetry — one form per container, a sugar over a shared
model — which is a different thing from Option 0's **two-grammar**
asymmetry (wrapper vs bare child prop, with no shared model underneath).
The axis the owner therefore weighs across Option 0 / Option 3+PM-1 /
Option 3+PM-2 is **how much authored asymmetry to keep**: Option 0 keeps
two grammars and no shared model; Option 3+PM-1 keeps one
grammar-with-sugar over a shared model; Option 3+PM-2/3 removes the
authored asymmetry entirely, at a two-ways-to-write cost (PM-2) or a full
Grid rewrite (PM-3). PM-1 is recommended because it makes the *model*
legibly parent-interpreted (the thesis) without spending the public
draft's budget on two co-equal Grid surfaces; the asymmetry that remains
is sugar over a unified model, not a second public model.

The decision hinges on three merit axes in priority order — **(1)
legibility of the parent-interpreted nature** (the thesis), **(2)
collision-safety as the widget vocabulary grows**, **(3) reservation of
additive extension paths** — with churn as the tie-breaker only. On (1),
Options 3 / 4 / 5 make placement legibly parent-data; 0 / 2 leave
`h-align` looking like a widget prop; 1 swings to "looks like a node".
On (2), 3 / 4 / 5 are structurally collision-safe; 0 / 2 need a
precedence rule. On (3), 3 reserves cleanly; 5 over-reserves and carries
the only family-pivot risk; 4 couples to parent type names.

Option 0 is the steel-man for "do nothing structural" (cheapest, a
defensible *documented* model) — it loses on axes (1) and (2), and its
asymmetry is two grammars rather than PM-1's one-grammar-over-one-model.
Option 5 (and its lighter variant 5a) is the steel-man for "maximally
future-proof" — both lose on proportionality for a corrective phase:
5a's admission-table machinery is invisible to the author yet must be
built and spec-positioned now, whereas `slot.`'s fixed prefix already
reserves the user-container and non-layout-parent-data paths additively
(§Forward-compat impact) **without publishing a namespace grammar in the
M3 public draft**. Option 3 maximises (1)–(3) and pays only a bounded,
in-phase migration cost.

Recorded as **Proposed**: the conceptual boundary, the prefix token, and
the PM-1-vs-PM-2 authored-asymmetry call are exactly where owner
merit-judgment is invited. Option 0 remains a live alternative if the
owner weights churn-avoidance and the "documented two-grammar asymmetry"
model above cross-container legibility; **PM-2 is the live alternative if
the owner wants the authored surface itself unified**, not only the
model.

**Owner merit-judgment (2026-06-21): CB-B + Option 3 (`slot.`) + PM-2.**
The owner took the PM-2 live alternative — the authored surface unified
(Grid gains direct `slot.*` alongside `Cell`) — as a *provisional* state
pending the pre-1.0 wrapper-rule decision. See §Decision outcome.

## Sub-issues

**Follow-up decision discipline.** Every "Follow-up if chosen" in
§Options is assigned a decision home, so none is settled implicitly
during implementation — a surface fixed in code rather than in the ADR
is a failure mode this DD explicitly avoids. Surface-determining
follow-ups are resolved **in the pre-doc phase, before this DD is
Accepted**: the recommended and live-contender options as conditional
sub-issues now (SI-1..3), the less-likely Options 2 / 4 as a conditional
SI added in pre-doc *if that option becomes a live contender* (kept out
of the document until then to avoid expanding unchosen branches). The
one follow-up that exceeds 7b's corrective scope — Option 5's namespace
mechanism — is an M3 responsibility discharged by inserting phase 7c,
not deferred to M4+. Option 0 decides nothing (spec wording only). No
follow-up is left to implementation.

Most aspects of the surface follow from the Main decision and are
covered there or in later sections: the name-collision rule is a
consequence of the chosen surface; bindability is under §Dependencies;
diagnostics, default-preservation, and key spelling are under §Spec
impact / §Out of scope. What remain are the **follow-up decisions a
surface option forces once chosen**, plus one unconditional
sub-decision. The conditional sub-issues are live only under the option
named; under any other option they are N/A:

- **SI-1 — Fixed-prefix token** *(only if Option 3 is chosen)* — which
  reserved prefix the unified surface uses.
- **SI-2 — Per-container surface mapping** *(only if Option 3 is
  chosen)* — what becomes of Grid `Cell`, and how ZStack moves onto the
  prefix.
- **SI-3 — ZStack edge wrapper name** *(only if Option 1 is chosen)* —
  the name of the ZStack placement wrapper.
- **SI-4 — Control-flow × placement author surface** *(unconditional)* —
  where placement is written on `for` / `if`-generated children.

Options 0 / 2 / 4 / 5 carry their own follow-ups too; those are noted
inline in §Options ("Follow-up if chosen") rather than expanded here,
because they are either empty (Option 0), a spelling / diagnostics
detail (Option 2 / 4), or a mechanism design deferred unless adopted
(Option 5).

### Dependencies among sub-issues

SI-1 and SI-2 are both gated on Option 3 and are decided together: the
prefix token chosen in SI-1 is the spelling the SI-2 mapping uses. SI-3
is gated on Option 1 and is mutually exclusive with SI-1/SI-2 (a
surface is either prefix-based or wrapper-based, not both). SI-4 is
independent of the surface option except that its recommended form
(CF-1) inherits whatever the chosen surface writes on a child.

## SI-1: Fixed-prefix token

*Applies only if the Main decision selects Option 3 (fixed prefix).*

### Context

If a reserved prefix is adopted, its actual token must be chosen. The
candidates differ only in what they connote to a reader; the parse-level
behaviour is identical.

### Options

1. **`slot.`** — "the slot this child occupies in its parent"; neutral
   about *what* the parent does with it; reserves room for non-alignment
   slot data later.
   - What you gain: neutral and short; does not over-commit to "layout",
     so a later non-layout parent-data reservation reads naturally.
   - What you give up: slightly abstract for a first-time reader who
     expects a layout-flavoured word.
2. **`placement.`** — names the concept directly.
   - What you gain: most explicit about meaning.
   - What you give up: long; reads as the verb and is narrower than
     `slot.` (awkward if non-placement slot data is reserved later).
3. **`parent.`** — emphasises the interpretation locus.
   - What you gain: makes "the parent interprets this" explicit.
   - What you give up: reads as "a property *of* the parent" rather than
     "of this child's relationship to it".
4. **`layout.`** — scopes to layout.
   - What you gain: immediately legible to a layout-minded reader.
   - What you give up: **under-reserves** — if CB-B later carries
     non-layout parent-data, the token mis-describes it.

Technical risk: identical across candidates (a fixed dotted-key lexeme);
the choice is a naming judgment, not a risk trade.

### Forward-compat impact

`slot.` keeps the non-layout-parent-data reservation (hit-test / focus /
accessibility) readable under the same prefix; `layout.` forecloses it
in spirit and would force a second prefix later.

### Recommendation

**`slot.`** — neutral and forward-reserving — *recorded as the
suggested token, but this is exactly an owner naming preference*, so it
is left open for the owner to pick among the four at the Accepted flip.

## SI-2: Per-container surface mapping

*Applies only if the Main decision selects Option 3 (fixed prefix).*

### Context

Grid currently groups a child's full placement payload (`row` +
`column` + `span` + alignment) in a `Cell` wrapper; ZStack writes
`h-align` / `v-align` directly. Adopting `slot.*` raises two questions:
what becomes of `Cell`'s grouping affordance, and whether Grid children
may *also* write `slot.*` directly. The tension to resolve here:
retaining `Cell` keeps Grid authors on a wrapper while ZStack moves to
`slot.*`, so the author-visible surface stays asymmetric even though the
underlying model is unified — partly undercutting the unification the
Main decision is chosen for.

### Options

1. **PM-1 — `Cell` as pure sugar; Grid children authored via `Cell`
   only.** `Cell { row, column, span, align }` lowers to the same
   per-child `slot.*` model; `slot.*` direct on a Grid child is not an
   author surface (Grid stays `Cell`-authored).
   - Depends on (main option): Option 3.
   - What you gain: Grid keeps the grouped-payload affordance; one
     authored form per container (no two ways to write Grid placement).
   - What you give up: the author-visible asymmetry remains (Grid =
     `Cell`, ZStack = `slot.*`); the unification is real in the model
     but not on the surface.
2. **PM-2 — `Cell` as sugar *and* `slot.*` direct allowed on Grid
   children.** Both forms admitted; `Cell` is the grouped convenience.
   - Depends on (main option): Option 3.
   - What you gain: Grid and ZStack share one direct surface (`slot.*`)
     *and* Grid keeps `Cell` for grouping; the unification is visible.
   - What you give up: two ways to write Grid placement — a spec /
     diagnostics burden, and a "which is canonical?" question for
     examples.
3. **PM-3 — drop `Cell`; Grid children authored via `slot.*` only.**
   - Depends on (main option): Option 3.
   - What you gain: fully symmetric surface; one form everywhere.
   - What you give up: loses `Cell`'s grouped-payload affordance;
     largest example migration (every Grid child rewritten).

### Forward-compat impact

PM-2/PM-3 make `slot.*` the single direct surface a future
custom-container or code-construction path would target; PM-1 keeps
`Cell` as a Grid-specific form that such a path must special-case or
re-sugar.

### Recommendation

**PM-1** (Main decision's "retain `Cell` as sugar"), with **PM-2 as the
live alternative** if the owner weights surface symmetry over "one form
per container". PM-1 delivers the Main decision's **model-level**
unification (both lower to `slot.*`) while leaving the **authored
surface** asymmetric; PM-2 / PM-3 are the **surface-visible**
unification choices (see Main decision §Recommendation for the two
senses). The authored-asymmetry cost of PM-1 is the explicit reason PM-2
is kept on the table; this is an owner merit call.

**Owner decision (2026-06-21): PM-2** — the documented live alternative.
The owner weights cross-container surface uniformity over "one form per
container", accepting the two-form Grid spec / diagnostics burden (strict
mixing reject) as a **provisional** state resolved at the pre-1.0
wrapper-rule gate (§Out of scope). The "structural data ⇒ wrapper"
principle is not adopted as the rule now (insufficient grounding).

## SI-3: ZStack edge wrapper name

*Applies only if the Main decision selects Option 1 (edge wrapper).*

### Context

Option 1 gives ZStack a structural wrapper mirroring Grid's `Cell`
(itself not a runtime widget). Only the name is open; the candidates
differ in what they connote.

### Options

1. **`Layer`** — connotes a stacked layer, matching ZStack's z-order
   model.
   - What you gain: reads naturally for an overlapping-children stack.
   - What you give up: a noun that future readers might expect to carry
     z-index semantics it does not.
2. **`Overlay`** — connotes something laid over content.
   - What you gain: evokes the lightbox / overlay use case directly.
   - What you give up: narrower than the general stacking case.
3. **`Item`** — a generic slot wrapper name.
   - What you gain: neutral; pairs with a generic slot story.
   - What you give up: too generic to signal "ZStack placement".

Technical risk: identical across names (a structural wrapper node); the
choice is a naming judgment.

### Forward-compat impact

A ZStack-specific wrapper name does not generalise to other containers;
if Option 1 were ever extended to more containers, a per-container
wrapper-name proliferation would follow (a point against Option 1
itself, recorded in the Main decision).

### Recommendation

**`Layer`** as the suggested name (best fit for the stacking model), but
this is an owner naming preference and Option 1 is not the recommended
surface; recorded for completeness so the choice is not left undefined
if the owner picks Option 1.

## SI-4: Control-flow × placement author surface

*Unconditional.*

### Context

Runtime already carries generated-child placement through staging →
commit (architecture.md §6.7.10 / §6.8.5, Phase 7); what this DD decides
is the *author-visible* form for placement on a `for`/`if` body. Whether
per-iteration placement can *vary* is a storage question owned by
DD-002; this sub-issue decides only where the author writes it.

### Options

1. **CF-1 — on the body's root child.** Placement attrs sit on the
   single root widget of the body template
   (`for t in xs { Box { slot.row: … } }`).
   - Depends on (main option): inherits whatever the chosen surface
     writes on a child (under Option 3, `slot.*`) — falls out directly,
     since the body root *is* a child of the placement-bearing parent.
   - What you gain: no new syntax; the body root is already a child, so
     the placement surface is exactly the static-child surface; uniform
     with the one-widget-per-iteration body rule (DD-M3-P7-001 B1).
   - What you give up: nothing relative to the chosen surface;
     multi-widget items wrap in a container that then carries the
     placement, same as static.
   - Technical risk: none beyond the static-child path already built.
2. **CF-2 — on the control-flow block.** Placement attrs sit on the
   `for` / `if` block itself, replicated to each generated child.
   - What you gain: one place to write shared placement.
   - What you give up: the block is not a widget; hanging child-placement
     on it invents a second placement locus and contradicts "placement is
     a child-slot fact"; collides with per-iteration variation.
   - Technical risk: a new placement-bearing position the checker /
     loader must special-case.
3. **CF-3 — replicated per generated child (explicit).** Author writes
   placement separately for each materialised child.
   - What you gain: maximal per-child control.
   - What you give up: impossible at author time — the children are
     generated, not written; this is a runtime/storage concern, not a
     surface.
   - Technical risk: not an authoring surface at all.

### Forward-compat impact

CF-1 keeps the generated-child placement surface identical to the
static-child surface, so a future bindable-placement or member-range
body extension widens storage (DD-002) without touching the author
surface. CF-2/CF-3 would each create a placement locus that a later
extension must carry or deprecate.

### Recommendation

**CF-1 — on the body's root child.** It falls out of the chosen surface
with no new syntax and matches the one-widget-per-iteration body rule.

## Decision outcome

**Accepted 2026-06-21** (paired with DD-002 — one phase ADR set), after
the PM-2 integration review (owner pass). The decision:

**Decision (2026-06-21):**

- **Conceptual boundary:** CB-B (generalizable parent-data, scoped to
  layout this phase).
- **Surface option:** Option 3 (fixed prefix), token **`slot.`** (SI-1).
- **Per-container mapping (SI-2):** **PM-2** — `Cell` retained as the
  grouped form *and* direct `slot.*` admitted on Grid children; one form
  per child (strict mixing reject). Chosen for cross-container surface
  uniformity. **Provisional:** the two-form Grid surface is resolved to
  PM-1 or PM-3 at the pre-1.0 wrapper-rule gate (§Out of scope).
- **Control-flow (SI-4):** CF-1 (placement on the body's root child).
- **Examples convention:** Grid examples default to `Cell` (provisional,
  not normative, not an AC); direct `slot.*` shown to illustrate the
  unified surface.
- **No normative canonical form** is declared for Grid this phase.

**Acceptance disposition (branch (a)).** PM-2 changes the *public* author
surface for **both** containers (ZStack `h-align` → `slot.h-align`; Grid
*gains* direct `slot.*` alongside `Cell`), so under the preamble
§Acceptance relation it is **branch (a)**. The AC form — **either** a new
AC for the unified placement surface **or** a refinement of A2 / A4 / A12
wording under the M3 acceptance-criteria revision exception — is an
**owner choice**, recorded in the plan Revision log and mirrored to
`process/_roadmap.md` (preamble Moment 1 touch list) as the immediate
follow-on of this flip. The AC states the **accept-set** (Grid accepts
`Cell` and `slot.*`; ZStack `slot.*`), *not* the provisional examples
convention.

## Spec impact

`docs/dsl_spec.md` (author-facing, external-reader bar, no DD option
labels per the living-spec vocabulary rule):

- The chosen placement surface for Grid and ZStack. **Under the
  owner-selected direction (Option 3 + PM-2 — model-level unification
  *and* a unified direct surface, with `Cell` retained):**
  - **Grid children may be authored two ways: the `Cell { row / column /
    row-span / column-span / h-align / v-align }` wrapper (the grouped
    form) *and* direct `slot.*` keys on the child.** Both are accepted and
    lower to the same per-child `slot.*` model (DD-002). **No normative
    canonical form is declared** for Grid this phase; `Cell` is the
    **provisional conventional form** for Grid examples (convention note
    below), pending the pre-1.0 wrapper-rule decision (§Out of scope).
  - **A Grid child is *either* a `Cell` wrapper *or* a content widget
    carrying direct `slot.*`** (in the direct form the content widget is
    itself the Grid child — no `Cell`). **Two distinct rejects keep this
    unambiguous for the checker:**
    - `slot.*` keys appearing among a `Cell` node's own attributes →
      **mixing reject** (a `Cell` carries placement through its own `row`
      / `column` / `row-span` / `column-span` / `h-align` / `v-align`
      keys, never `slot.*`);
    - `slot.*` on a widget *inside* a `Cell` → **non-admitting-parent
      reject** (the widget's immediate parent is the `Cell` wrapper, which
      admits no placement) — a *different* diagnostic from mixing.

    Both are named check errors with firing tests.
  - **ZStack author surface becomes `slot.h-align` / `slot.v-align`** on
    the child, replacing today's bare `h-align` / `v-align`.
  - Grammar additions: the `slot.` prefix lexeme (ZStack path **and** the
    Grid direct path) and the retained `Cell` wrapper-as-sugar production
    (Grid grouped path) — no new node type for the `slot.*` path.
  - **Provisional convention (not normative, not an AC):** Grid examples
    in `docs/dsl_spec.md` and `examples/gallery/` are written with `Cell`
    by default; direct `slot.*` is shown where it illustrates the unified
    surface. This convention is **provisional** and is revisited at the
    pre-1.0 wrapper-rule decision (§Out of scope), which resolves toward
    PM-1 (drop Grid direct `slot.*`) or PM-3 (drop `Cell`).
- **Per-key admission table (the diagnostics — kept as a forcing
  artifact, not summarised away):** which placement keys each container
  admits — Grid: `row` / `column` / `row-span` / `column-span` /
  `h-align` / `v-align`; ZStack: `h-align` / `v-align` — and that they
  are **rejected everywhere else**. Under PM-2, Grid keys are admitted
  **both** inside `Cell` *and* as direct `slot.*` on the child (but not
  both on the same child — see the strict mixing reject above); ZStack
  keys as `slot.h-align` / `slot.v-align`. A stray placement attr (under a
  non-admitting parent) is a named check error, re-checked by the loader;
  the check distinguishes a placement attr from an unknown widget prop in
  both directions (neither is silently accepted as the other). Each reject
  is a named diagnostic with a firing test.
- **`slot.*` grammar — accepted / rejected examples (forcing table, the
  owner-visible boundary):** the *internal* parse mechanics — whether the
  lexer emits one `slot.h-align` token or `Ident("slot") Dot
  Ident("h-align")` folded by the parser into a dotted property key, and
  how the canonical key is stored on the AST / IR — is an **implementer
  recommendation, not an owner decision.** Recommended: the parser reads
  `slot.` + key as a *dotted property-key* (not an expression
  member-access), storing the canonical key (`slot.h-align`) on the
  placement slot, so it never collides with expression-grammar qualified
  access. What the **spec fixes** is the author-visible accept / reject
  set and which stage rejects:

  | Example | Disposition | Stage |
  |---|---|---|
  | `slot.h-align: end` on a ZStack child | accepted | — |
  | `Cell { row: 1 column: 0  Box {} }` Grid child | accepted (grouped form) | — |
  | `Box { slot.row: 1 slot.column: 0 }` directly under `Grid` (no `Cell`) | accepted (PM-2 direct form; the widget is the Grid child) | — |
  | `Cell { row: 1 slot.column: 0  Box {} }` — `slot.*` among a `Cell`'s own attrs | rejected — mixing: a `Cell` carries placement via its own keys, not `slot.*` | checker |
  | `Cell { row: 1  Box { slot.column: 0 } }` — `slot.*` on a widget *inside* a `Cell` | rejected — non-admitting parent: the widget's parent is `Cell` | checker |
  | `slot.h-align: end` under a non-admitting parent (e.g. VStack) | rejected — parent admits no placement | checker |
  | `slot.foo: …` on a ZStack child | rejected — unknown slot key | checker |
  | `slot.h-align: some_state` (binding RHS) | rejected — placement is constant per instance | checker |
  | `slot.h-align: end` *where a state named `end` exists* | accepted — `end` is the placement keyword, **not** the state | checker |
  | `slot:` / `slot..h-align` / `slot.` (malformed key) | rejected — malformed placement key | parser |

  Parser-stage rejects (malformed key shape) are distinguished from
  checker-stage rejects (admission / mixing / unknown-key / constant-RHS);
  each row is a named diagnostic with a firing test (`wasamoc check`),
  re-checked by the loader where it survives to IR. The strict Cell-vs-
  `slot.*` mixing reject is the PM-2-specific diagnostic (one author form
  per Grid child).
- **Placement value namespace (the resolution rule):** a placement key's
  RHS is resolved against the **closed placement-keyword set** for that
  key (`h-align` / `v-align` → `start` / `center` / `end` / `stretch`;
  `row` / `column` / `row-span` / `column-span` → integer literals),
  **not** through the state namespace. A bare keyword like `end` is therefore *always* the placement
  constant even if a state of the same name exists — placement values do
  not shadow- or resolve-through state, so the same `.ui` cannot flip
  accepted/rejected by checker ordering. Reading a state into placement
  requires explicit binding-expression syntax, which is the
  constant-per-instance reject above (a future bindable-placement
  surface, §Out of scope).
- **Placement RHS is constant per instance (the diagnostics):** a
  placement key whose RHS is a state- or loop-local binding *expression*
  (not a literal / constant) is a **named check error** with a firing
  test. This is what keeps `slot.*` from reading as a *bindable*
  parent-data grammar before that is designed (§Out of scope re-visit
  trigger); it is rejected, not silently dropped.
- **Defaults preserved per container:** omitted alignment falls to the
  existing per-container default (Grid `stretch`, ZStack `center`);
  unifying the surface does not unify the defaults (see §Out of scope).
- Existing key/value spelling is **inherited unchanged** — the full Grid
  key set `row` / `column` / `row-span` / `column-span` / `h-align` /
  `v-align`, the ZStack `h-align` / `v-align`, and the alignment values
  `start` / `center` / `end` / `stretch`; the surface adds only the chosen
  prefix/wrapper, not new key names.
- Stale placement prose swept in the same touch.
- **If Option 2 or Option 4 is the chosen surface, its surface-semantics
  follow-up is specified here, not at implementation:** for Option 2,
  the "parent consumes first" precedence rule; for Option 4, the
  attached-property spelling convention and parent-type binding. (Under
  the recommended Option 3 neither applies.)

`docs/architecture.md` placement model is DD-002's seed, not this DD's.

## Risk mitigation

(Assuming the recommended CB-B + Option 3 + `Cell`-sugar.)

- **Parser / lexer:** the `slot.` prefix is a bounded, parse-level
  namespace on child attribute keys; the `Cell`-as-sugar arm reuses the
  existing wrapper production, so no new node type is introduced for the
  ZStack path.
- **Checker admission sweep:** every non-admitting parent gets a reject
  branch (impl-gates trap #4); each reject is a pure-logic `wasamoc
  check` test plus a loader re-check, and paired accept/reject fixtures
  pin the placement-vs-ordinary-property distinction.
- **Migration:** the `examples/gallery/` sweep is bounded and greppable;
  with no long-lived alias, the old **ZStack bare** placement attrs
  (`h-align` / `v-align` written directly, without the `slot.` prefix)
  become named diagnostics, each with a firing test. **Grid `Cell` is not
  an old form** — it is retained as the grouped form, and Grid also gains
  the direct `slot.*` surface — so the ZStack bare path is the only
  removal. This cost is a tie-breaker, not a counter-argument.
- **PM-2 two-form burden (accepted cost):** admitting both `Cell` and
  direct `slot.*` on Grid adds **two named checker diagnostics** (each
  with a firing test) — a **mixing reject** (`slot.*` among a `Cell`
  node's own attributes) and the existing **non-admitting-parent reject**
  (`slot.*` on a widget *inside* a `Cell`) — plus the provisional-
  convention prose so the public draft is not a coin-flip. This is the spec / diagnostics cost
  the owner accepts for cross-container surface uniformity; it is bounded
  to the corrective phase and resolved at the pre-1.0 wrapper-rule gate
  (§Out of scope).
- **Family-pivot guard:** the FD-7b-C family-impact confirm (Option 3
  within family (1)) is the exit check; only a pivot-level choice
  (Option 5 becoming a scope construct) would escalate to a VDR.

## Out of scope

- **Bindable placement** — not implemented this phase; recorded as a
  future-possible concept only (see §Dependencies and the Main decision
  §Forward-compat impact). Because `slot.*` reads property-like, a reader
  may expect `slot.row: some_state` or per-item placement binding; Phase
  7b admits only a **constant per-instance** placement RHS. A state- or
  loop-local *expression* RHS on a placement key is **rejected by a named
  diagnostic** (§Spec impact), not silently accepted, so the surface does
  not pre-promise bindability it cannot yet honour. **Re-visit trigger:**
  a concrete app needs state- / loop-local placement that varies after
  construction; at that point bindable placement is designed *together
  with* the `BindingTarget` machinery (a new binding-target variant) and
  DD-002's child-slot effect lifecycle — not as a `slot.*`-local addition
  — so the reactive landing is decided once, whole.
- **Generic modifier system / custom-container custom slot attributes /
  non-layout parent-data** (hit-test / focus / accessibility) — reserved
  by a namespaced surface, built later with triggers (framing §Out of
  scope / R1).
- **Long-lived backward-compatibility alias for the old placement
  syntax** — under the owner-selected PM-2, none is retained (pre-1.0
  minimal migration); the old **ZStack bare** placement attrs (`h-align`
  / `v-align` written directly without the `slot.` prefix) are rejected by
  a named diagnostic. Grid `Cell` is **not** an old form (retained as the
  grouped form) and Grid additionally **gains** the direct `slot.*`
  surface, so no Grid **author-syntax** is removed — but this does **not**
  waive DD-002's Grid **storage** migration, which SM-B performs under
  CB-B (DD-002 SI-2). If the pre-1.0 wrapper-rule decision later moves to
  PM-3, the Grid `Cell` author form migrates then.
- **PM-2 → PM-1 / PM-3 wrapper-rule decision (pre-1.0 gate).** PM-2 ships
  *two* author forms for Grid placement (`Cell` and direct `slot.*`) — a
  deliberately **provisional** state. Before Wasamo reaches 1.0 a rule
  must be decided for *which widgets / containers may use a wrapper form*,
  resolving the Grid surface toward **PM-1** (keep `Cell`, drop Grid
  direct `slot.*`) or **PM-3** (drop `Cell`, keep `slot.*` only). The
  "structural data ⇒ wrapper" principle is **not** adopted as that rule
  now (insufficient grounding — Grid is the only instance). **Re-visit
  triggers:**
  - a new container wants an edge / wrapper form for any reason (whether
    or not its payload is structural data);
  - a public code-construction API / builder surface is designed (it bears
    directly on whether `Cell` stays DSL-only sugar — a child-slot builder
    favours direct `slot.*`);
  - custom containers / custom slot attributes are designed;
  - the first non-layout parent-data (hit-test / focus / accessibility) is
    introduced;
  - **Wasamo reaching 1.0 (hard deadline)** — the provisional two-form
    state must not freeze into the public 1.0 contract.

  **Carry path:** recorded here now; carried into
  `phase-7b/implementation/handoff.md` after the Accepted flip; folded
  into the **M3 `handoff.md`** at milestone close as a pre-1.0 residual.
- **Layout-algorithm changes** — placement is parent-interpreted
  metadata; no measure/arrange algorithm is re-decided here.
- **Default-alignment unification (Grid `stretch` / ZStack `center` →
  one default)** — deliberately *not* done; the defaults stay
  per-container because each is natural for its container (a Grid cell
  fills, a stacked child centres). Re-visit trigger: the public draft
  finds the default mismatch a real explicability debt, or an app needs
  cross-container default consistency. Lands in a future
  layout-behavior phase (changing a default is a layout-behavior change,
  not a surface change).
- **Placement key/value spelling revision** (e.g. `h-align` → `hAlign`)
  — not done; existing spelling is inherited. Re-visit trigger: a DSL
  naming-convention / ergonomics pass, or public-draft stabilization.
  Lands in a future DSL-ergonomics phase.

## Revision history

- 2026-06-19 — Initial draft (Proposed). Conceptual boundary + six surface
  options (0–5), the control-flow sub-issue, recommendation CB-B + Option 3
  (`slot.`) with `Cell` as sugar.
- 2026-06-20 — Strategic / recommendation-choice / implementation-readiness
  review folds reflected (Status: Proposed; no recommendation reversed):
  added the conditional follow-up sub-issues (SI-1..4) and Option 5a, split
  model- vs surface-level unification (Option 3 + PM-1), and pinned the
  owner-visible boundaries — `slot.*` grammar/diagnostics table, branch-(a)
  acceptance disposition, Option 5 → Phase 7c, and Grid author-syntax vs
  DD-002 storage migration.
- 2026-06-21 — PM-2 integration (Status: Proposed; Accepted flip pending
  integration review). Recorded the owner-selected direction (CB-B +
  Option 3 `slot.` + **PM-2** + CF-1). Flipped Grid direct-`slot.*`
  admission to **accepted**; added the **strict `Cell` / `slot.*` mixing
  reject** and the full 6-key Grid set (`row` / `column` / `row-span` /
  `column-span` / `h-align` / `v-align`, correcting the earlier `span` /
  `align` shorthand); set **no normative canonical** + provisional
  `Cell`-default examples convention; added the **pre-1.0 wrapper-rule
  re-visit trigger** (PM-2 → PM-1 / PM-3) with its carry path; fixed
  branch-(a) framing to cover the dual Grid surface.
- 2026-06-21 — Codex re-review folds (Status: Proposed): corrected the
  placement value-namespace and key-spelling bullets to the full 6-key
  Grid set (`row-span` / `column-span`, not `span`); split the strict
  **mixing reject** (`slot.*` on a `Cell` node) from the
  **non-admitting-parent reject** (`slot.*` on a widget inside a `Cell`)
  as two distinct named diagnostics, with concrete syntax.
