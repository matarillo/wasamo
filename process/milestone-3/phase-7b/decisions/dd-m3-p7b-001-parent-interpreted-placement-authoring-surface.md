# DD-M3-P7b-001 — Parent-interpreted placement authoring surface

**Status:** Proposed
**Phase:** M3-Phase 7b
**AC:** contingent — see [preamble §Acceptance relation](./preamble.md);
the new-AC disposition (branch (a) AC-revision exception vs branch (b)
A11/A12 discharge) is fixed at this DD's Accepted flip.

## The question

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

This DD decides the conceptual boundary first, then the surface form,
then the supporting rules (vocabulary, collision, control-flow,
bindability, forward-compat, diagnostics, migration). Per the owner
prior, options are compared on **product merit / thesis fit first**;
migration cost is a tie-breaker, never a rejection ground.

## Settled floor consumed from framing

Placement is parent-interpreted, not an intrinsic widget property
(FD-7b-A). This DD does **not** re-litigate that; it decides how the
author *writes* something that is already agreed to be
parent-interpreted.

## Sub-issue 1 — Conceptual boundary (decided first)

The upper-level judgment that frames every surface option:

- **CB-A — container-specific sugar.** Placement is per-container
  syntax: Grid's `Cell` and ZStack's annotations are each a feature of
  that container, with no claim of a shared grammar.
- **CB-B — generalizable parent-data, scoped to layout this phase.**
  Placement is one instance of a cross-container "data the parent
  interprets about a child" grammar (e.g. a `slot.*` namespace),
  *restricted to layout placement in Phase 7b* but shaped so the same
  grammar could later carry other parent-data (with triggers; the
  generic case stays deferred, framing §Out of scope / R1).

CB-A is simpler to ship and matches the code as it stands; its cost is
that the public draft must explain two unrelated-looking surfaces for
one concept, and a future custom-container / non-layout-parent-data
need re-opens the surface question from scratch. CB-B pays a small
up-front legibility/structure cost to make the parent-interpreted
nature explicit and to reserve (not build) a coherent extension path.
The choice is not free of the storage question — CB-B pairs naturally
with DD-002's child-slot model, CB-A with either.

This DD does not assume the answer; the surface options below are
evaluated under whichever boundary is chosen, and the recommendation
states the boundary it assumes.

## Sub-issue 2 — Surface form

Each option carries a one-line **family-impact** note per FD-7b-C
(family (1) = tree-description grammar; family (2) = view-function
re-execution).

- **Option 0 — principled asymmetry / documented status quo.** Keep
  Grid `Cell` (structured wrapper carrying `row` / `column` / `span` /
  alignment) and ZStack direct annotation, and **document the
  asymmetry as an intended model**: "placement that carries structural
  data uses a wrapper; simple alignment is written directly." This is a
  first-class option, not mere ratification — it must be judged as a
  deliberate model, not an accident.
  - Family impact: within family (1) (no grammar change).
  - Merit: zero churn; matches shipped examples. Cost: the public draft
    still presents two shapes; the rule "structural data ⇒ wrapper" is
    a post-hoc rationalisation that a reader must be taught, and it does
    not by itself make the parent-interpreted nature legible (ZStack
    `h-align` still *looks* like a widget property).
- **Option 1 — edge wrapper unification (`Cell` / `Layer`).** Give
  ZStack a `Layer` wrapper mirroring Grid's `Cell`, so both containers
  place children through a structural wrapper.
  - Family impact: within family (1).
  - Merit: visually uniform "wrapper carries placement". Cost: a
    `Layer` wrapper around every ZStack child is heavy for a single
    alignment pair, and wrappers read as nodes — the opposite legibility
    risk from Option 0 (placement looks like a widget rather than
    parent-data).
- **Option 2 — no prefix, semantics-only unification.** Keep direct
  child attrs with no prefix, but unify the *semantics* (same
  admission / rejection / defaulting rules) and drop the Grid wrapper
  in favour of direct `row` / `column` on the child.
  - Family impact: within family (1).
  - Merit: lightest syntax. Cost: does nothing for the
    "widget-property-or-not" ambiguity and maximises the name-collision
    risk (sub-issue 5); a future custom container's slot keys collide
    with widget props with no namespace to disambiguate.
- **Option 3 — fixed prefix (`slot.` / `placement.` / `parent.` /
  `layout.`).** Write placement as a reserved-prefix attribute on the
  child: e.g. `slot.row` / `slot.column` / `slot.h-align`.
  - Family impact: within family (1) (prefix is a parse-level
    namespace, not re-execution).
  - Merit: the prefix makes "this is parent-interpreted, not a widget
    property" legible at the call site, unifies Grid and ZStack on one
    grammar, and structurally avoids the name-collision class. Cost: a
    new prefix token to specify and migrate existing examples to; the
    `Cell` structural-grouping affordance must be re-expressed or kept
    as sugar.
- **Option 4 — XAML-style attached property (`Grid.row` /
  `ZStack.hAlign`).** Qualify placement by the *parent type*.
  - Family impact: within family (1), but it is the closest to a
    type-directed dispatch surface.
  - Merit: explicit about which parent interprets the attr; familiar to
    XAML readers. Cost: couples the child's authored attr to the
    concrete parent type name (brittle under re-parenting / custom
    containers), and is heavier than a single neutral prefix for a tree
    whose parent is already structurally known.
- **Option 5 — parent-declared modifier namespace.** The parent
  container *declares* the slot attributes it admits; children use that
  namespace.
  - Family impact: within family (1) for the declaration-as-data form;
    **escalates toward family (2)** only if the namespace becomes a
    host-language scope construct (not proposed here) — flagged per
    FD-7b-C as the one option whose pivot risk must be checked at exit.
  - Merit: the most extensible toward user-defined containers. Cost:
    over-built for two built-in containers in a corrective phase
    (framing R1 / R5); the custom-container case is explicitly deferred.

## Sub-issue 3 — Fixed-prefix vocabulary (if Option 3)

If a fixed prefix is chosen, the token candidates and their read:

- `slot.` — "the slot this child occupies in its parent"; neutral
  about *what* the parent does with it; reserves room for non-alignment
  slot data later. Leading candidate.
- `placement.` — explicit but long; reads as the verb, slightly
  narrower than `slot.`.
- `parent.` — emphasises interpretation locus, but reads as "a property
  *of* the parent" rather than "of this child's relationship".
- `layout.` — scopes to layout, which *under-reserves* if CB-B later
  carries non-layout parent-data; rejected under CB-B, acceptable under
  CB-A.

## Sub-issue 4 — Grid and ZStack surface mapping

Per container, the concrete authored form under the chosen surface:

- **Grid:** keep `Cell`; migrate to `slot.row` / `slot.column`; or keep
  `Cell` as sugar / legacy over a direct surface. `Cell` also groups a
  child's full placement payload (`row` + `column` + `span` +
  alignment) into one node — a structural affordance a flat prefix
  loses unless retained as sugar.
- **ZStack:** keep direct `h-align` / `v-align`; migrate to
  `slot.h-align` / `slot.v-align`; or introduce a `Layer` wrapper.

The Grid and ZStack mappings must agree with the sub-issue 2 choice;
they are listed separately because `Cell`'s grouping role makes the
Grid mapping non-trivial even once the prefix question is settled.

## Sub-issue 5 — Name-collision rule

A surface-comparison axis as much as a rule: what happens when a
parent-consumed placement attr shares a name with an ordinary child
widget prop (now or in future).

- No-prefix direct attrs (Options 0 / 2): collision is real and must be
  resolved by a "parent consumes first" precedence rule + a diagnostic,
  which is fragile as the widget vocabulary grows.
- Prefix / namespace (Options 3 / 5) / attached property (Option 4):
  the namespace **structurally** prevents collision — `slot.h-align` is
  never confusable with a widget `h-align`.

This axis favours a namespaced surface and is recorded as a direct
input to the recommendation.

## Sub-issue 6 — Control-flow × placement author surface

How placement is written on `for` / `if`-generated children. Runtime
already carries generated-child placement through staging → commit
(architecture.md §6.7.10 / §6.8.5, Phase 7); what this DD decides is
the *author-visible* form: placement on the body's root child, on the
block, or replicated per generated child. Whether per-iteration
placement can *vary* is the bindability question (sub-issue 7), coupled
to DD-002.

## Sub-issue 7 — Placement bindability (surface side)

Placement is literal / constant today. Phase 7b does **not** implement
bindable placement, but the surface must be consistent with DD-002's
storage stance on whether "placement is a future-bindable concept" or
"constant-only for layout stability". This DD records the
author-surface read; DD-002 records the storage/reactive boundary.

## Sub-issue 8 — Forward-compat reservation (check now, build later)

The chosen surface must not *accidentally* foreclose later additive
surfaces, even though none is built in Phase 7b: custom-container
custom slot attributes, and non-layout parent-data (hit-test / focus /
accessibility). A namespaced surface reserves this cleanly; a no-prefix
surface narrows it. This is a reservation condition, not a feature
(framing §Out of scope rows for those items hold).

## Sub-issue 9 — Diagnostics

- The parent contexts under which a placement attr is admitted (Grid
  child / ZStack child) vs rejected (everywhere else).
- Stray placement attr (placement under a non-admitting parent) is a
  named check error, re-checked by the loader.
- A placement attr must not be silently accepted as an unknown widget
  prop, and vice-versa — the check distinguishes the two classes.

## Sub-issue 10 — Migration

- Existing `examples/gallery/` and other `.ui` files updated to the
  chosen surface (A11 sync).
- Whether a pre-1.0 compatibility alias for the old surface is admitted
  (default: pre-1.0 minimal migration, no long-lived alias — but
  decided here, not assumed).
- `docs/dsl_spec.md` stale placement prose swept.

## Comparison

The decision hinges on three merit axes, in priority order: **(1)
legibility of the parent-interpreted nature** (the thesis), **(2)
collision-safety as the widget vocabulary grows**, **(3) reservation
of additive extension paths** — with churn / migration cost as the
tie-breaker only.

- On (1), Options 3 / 4 / 5 make placement legibly parent-data at the
  call site; Option 0 / 2 leave `h-align` looking like a widget prop;
  Option 1 swings to the opposite risk (placement looks like a node).
- On (2), the namespaced options (3 / 4 / 5) are structurally
  collision-safe; 0 / 2 need a precedence rule.
- On (3), 3 and 5 reserve cleanly; 5 over-reserves for a two-container
  corrective phase and carries the only family-pivot risk; 4 couples to
  parent type names.
- On churn (tie-breaker), 0 is free, 2 is cheap, 3 is a bounded
  example-migration, 1 / 4 / 5 are heavier.

Option 0 is the steel-man for "do nothing structural": it is cheapest
and a defensible *documented* model. It loses on axis (1) — documenting
an asymmetry does not make `h-align` legible as parent-data — and on
axis (2). Option 5 is the steel-man for "maximally future-proof"; it
loses on proportionality (framing R1 / R5) for a corrective phase.

## Recommendation

**Proposed direction (subject to owner review and the FD-7b-C
family-impact confirm at exit):** conceptual boundary **CB-B**
(generalizable parent-data, scoped to layout this phase), surface
**Option 3** with the **`slot.`** prefix, unifying Grid and ZStack on
`slot.*`; **retain `Cell` as a structural-grouping sugar** over the
`slot.*` model (not removed), so Grid authors keep the grouped-payload
affordance while the underlying model is the unified one. Name
collision is resolved structurally by the prefix; the surface reserves
custom slots / non-layout parent-data additively without building them;
bindability is recorded as a *future-possible* concept consistent with
DD-002. Migration: update `examples/gallery/` in-phase; **no long-lived
compatibility alias** (pre-1.0 minimal migration), with the old forms
rejected by a named diagnostic.

This direction maximises the thesis axes (1)–(3) and pays only a
bounded, in-phase migration cost. It is recorded as a **Proposed**
recommendation: the conceptual boundary and the prefix token are
exactly the points where owner merit-judgment is invited, and Option 0
remains a live alternative if the owner weights churn-avoidance and the
"documented asymmetry" model above cross-container legibility. The
family-impact line for the chosen option (Option 3: within family (1))
is confirmed and written back to `architectural-family.md` at Moment 1
/ Moment 2 (revise-in-place; no VDR expected).

## Coupling to DD-002

The conceptual boundary and the bindability read here constrain
DD-002's internal model and reactive-boundary choices; DD-002's storage
choice bounds whether per-iteration placement can vary. The two are
Accepted together. See
[DD-M3-P7b-002](./dd-m3-p7b-002-placement-internal-model-and-construction-boundary.md).

## Spec content seed

`docs/dsl_spec.md`: the author-facing placement surface (chosen syntax
for Grid and ZStack), the admission / rejection rules, and invalid
examples — at the external-reader bar, with no DD option labels in the
prose (living-spec vocabulary rule). `docs/architecture.md` placement
model is DD-002's seed, not this DD's.

## Revision history

- 2026-06-19 — Initial draft (Status: Proposed), expanded from the
  framing §論点 slate (DD-M3-P7b-001). Conceptual boundary, six surface
  options (0–5) with family-impact lines, prefix vocabulary, per-
  container mapping, collision / control-flow / bindability /
  forward-compat / diagnostics / migration sub-issues, comparison, and
  a Proposed recommendation (CB-B + Option 3 `slot.` + `Cell` as sugar)
  recorded. Pending owner review.
