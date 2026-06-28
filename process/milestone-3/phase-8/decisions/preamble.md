# M3-Phase 8 — Selected state + Gallery integration + DSL spec public draft: Architecture Decisions

**Phase:** M3-Phase 8 (selected state + gallery integration + DSL spec
public draft — **the final M3 phase**)
**Date:** 2026-06-27
**Status:** Proposed (both DDs at Proposed; DD-001 under multi-pass review,
DD-002 partially drafted — its §DD-001 coupling is concrete to satisfy the
pre-Accept gate below; its broader public-draft policy options are still
being drafted)

## Context

Phase 8 is the **closing phase of M3**. It delivers three things and adds
**no new layout primitive and no new app feature**
([framing FD-8-A](../requirements/framing.md)):

- **(i) Selected state (A10).** Settle how an author writes a button's
  **selected / toggle** state in `.ui` and drive the gallery's tab-style
  section (or the minimal demonstration vehicle the spike fixes) with it.
  This is the **only new authoring surface** Phase 8 introduces — it shows
  that a **boolean binding (Phase 1) can drive a widget *attribute***.
- **(ii) Gallery integration (A1).** Fold the per-phase verification
  screens that Phases 5–7b grew into the **single Photo Gallery target
  app** ([gallery-wireframe.html](../requirements/gallery-wireframe.html))
  and run it end-to-end in **all three hosts (C / Rust / Zig)**.
  "Integration" means everything runs inside that one target app — **not**
  a pile of verification screens or a demo menu.
- **(iii) DSL spec public draft (A12).** Promote `docs/dsl_spec.md` to its
  **first public draft**. Phases 1–7b updated the spec per-phase, so this
  is an **editorial** pass (polish an existing surface to public-draft
  quality), not greenfield writing.

Of the three, only (i) is a new look-and-feel feature; (ii) and (iii) are
**integration and editing**. The placement author surface frozen in
Phase 7b is **read and polished, not re-decided**
([constraints §1](../requirements/constraints.md)).

Because Phase 8 is the **final M3 phase**, it also carries milestone-close
responsibilities that earlier phases could defer: residual hand-off
folding (PM-2 wrapper-rule, the Problem B sizing question), the
external-reader smoke, and the "no silently-deferred M3 surface" audit
([constraints §8](../requirements/constraints.md);
[plan.md §Milestone-end criteria](../../plan.md)).

### The settled floor (not re-litigated)

The premises Phase 8 treats as fixed (framing §再検討しない前提;
[constraints §1](../requirements/constraints.md)):

- **Placement surface is frozen (Phase 7b).** `slot.*`, Grid `Cell` +
  direct `slot.*` (PM-2), and child-slot `SlotData` storage are synced to
  [docs/dsl_spec.md](../../../../docs/dsl_spec.md) §4.16 /
  [docs/architecture.md](../../../../docs/architecture.md) §6.8.6 / §6.8.4.
  Phase 8 reads the **landed source** (not a design draft) and polishes it
  editorially; it does not re-derive the surface (constraints §1; the
  7b "pin to landed source" learning).
- **No new layout primitive and no new measure/arrange.** Grid /
  WrapPanel / ZStack / ScrollView / Box are complete. A dedicated toggle
  widget (if DD-001 chooses one) reuses Button's existing leaf
  measure/arrange — a new *node*, not a new layout primitive.
- **TypedValue stays deferred.** Selected state rides the existing
  **boolean** binding, so no new value type is pressured
  ([typed-value-evaluator.md](../../../../docs/notes/typed-value-evaluator.md)).
- **Selected state is driven by a boolean on the existing binding.** The
  driving value is a boolean (packet C, owner-aligned 2026-06-25); *which
  surface carries it* is DD-001's load-bearing question, and the
  alternatives were left **un-rejected** by the framing for DD-001 to
  decide (framing §66).

What stays **open and decided by the DDs**, not by this preamble: the
selected-state authoring form and its driving-boolean / exclusion
mechanism (DD-001); and how the public draft positions reserved,
provisional, and unsettled surface (DD-002). The framing's scope table,
verification strategy, and risk register are the authority for these
boundaries ([../requirements/framing.md](../requirements/framing.md)).

### Acceptance relation (no new AC expected — contingent on DD-002)

Phase 8's three deliverables are each **already named by an existing
acceptance criterion**, so — unlike Phase 7b, which minted A13 for a
surface no AC named — **no new AC is expected** (framing FD-8-F):

- **A10** reserves the selected-state surface from the start; DD-001
  discharges it.
- **A1** names the integrated `examples/gallery/gallery.ui` running M3
  end-to-end through DSL → IR → runtime in all three hosts.
- **A12** names the `docs/dsl_spec.md` public-draft promotion.
- **A11** (continuing obligation) keeps `.ui` / IR / `wasamoc` / runtime /
  `docs/dsl_spec.md` / `examples/gallery/` in lock-step.

The one **contingency**: if **DD-002 adds a new public promise** (e.g.
reserving M4 syntax in a way that commits author-facing surface), the
M3 acceptance-criteria revision exception applies — a new AC or an A12
wording refinement is recorded, ROADMAP mirrored, plan Revision log
updated. If DD-002 adds no public promise, "no new AC" is recorded in the
plan Revision log. This preamble does not pre-commit either way; the
disposition is fixed at the **DD-002 Accepted flip**.

### Owner-agreed framing decisions

The pre-doc framing was owner-aligned on 2026-06-25 and is recorded in
[../requirements/framing.md](../requirements/framing.md) ("オーナー合意の
記録" / the Owner alignment packet). The seven framing decisions this ADR
consumes:

- **FD-8-A** — Phase 8 thesis: M3-closing phase; three deliverables
  (A10 selected state + A1 gallery integration + A12 public draft); "gallery
  integration" = folding into the **Photo Gallery target app**, not a
  verification-screen collection / dashboard; no new layout primitive; the
  A1 feature-mapping table is an **initial hypothesis** updatable at
  FD-8-G(1) / DD-001.
- **FD-8-B** — DD slate: **two DDs** (DD-001 selected-state surface,
  DD-002 public-draft promotion). Gallery integration is
  implementation/verification work, **not** a DD.
- **FD-8-C** — selected state rides the existing boolean binding; visuals
  minimal in M3; exclusion expressed with shipped surface only (no host
  state write-back); the demonstration vehicle and the carrying widget
  surface are left to the **pre-DD spike + DD-001** (alternatives **not**
  rejected by the framing — §66).
- **FD-8-D** — the explicit-sizing question (Problem B) is **raised as a
  cross-milestone Vision DR** at the Phase 8 framing stage, but **does not
  block M3 close**: Phase 8 raises it and fixes the issue; implementation
  and final disposition are a separate gate. Not a phase DD.
- **FD-8-E** — scope: implement the selected-state surface end-to-end
  (parse / check / lower / runtime / example sync), integrate the gallery
  in three hosts, **sweep the per-phase verification screens**, and edit
  the spec. No extra verification menus / demo screens / new primitives;
  the minimal state-toggle UI an A1 positive control needs is allowed
  within the wireframe / owner-agreed placeholder budget. M4+ features and
  the carry-forward items are forwarded.
- **FD-8-F** — new AC unlikely (the §Acceptance relation contingency);
  recorded in the plan Revision log either way.
- **FD-8-G** — five **owner UI checkpoints staged across the plan** (not
  piled at phase end): (1) wireframe-fidelity / placeholder agreement →
  reflected into the A1 feature-mapping table; (2) first-render UI check in
  one representative host; (3) selected-state / lightbox **positive
  control** (two frames); (4) public-draft + M3 handoff draft review;
  (5) final human-visible smoke.

### Pre-DD feasibility spike (DD-001's option set was rebuilt from it)

Unlike a normal phase, DD-001's option **set** was not assumed — it was
**rebuilt from a pre-DD feasibility spike**
([../requirements/dd-001-stage1-spike.md](../requirements/dd-001-stage1-spike.md),
stage 1, run *before* DD-001's comparison per the framing "次にやること"
order). The spike compiled each candidate (`wasamoc check` / `build`) and
ran the adopted candidate on the live runtime, fixing the shipped-surface
facts that bound the design space: **no `==` in the expression grammar**;
`if` conditions are `BOOL_LIT | IDENT` with no operators; handlers are
block assignments only; and there is **no public API to write component
state while displayed**. These facts — not assertion — are what make
exclusion expressible only as a *composition of boolean states*, and what
put two-way binding and widget-owned state out of M3's reach. DD-001 cites
the spike as authority and does not re-derive it. The spike is an
**immutable record**; where DD-001's recommendation **reverses** the
spike's S1-lead conclusion, the supersession is recorded in DD-001
(§Accepted-time re-sync), not by editing the spike.

### Architectural-family confirmation (FD-8-C / notes triage)

Phase 8 fires `architectural-family.md` re-evaluation **trigger 1** (M3
DSL spec drafting — the public draft is its capstone). Per the framing
notes triage, the **working expectation is confirm-within-family (1)**:
selected state is a boolean *attribute* describing the tree, Phase 8 adds
no new grammar beyond it, and the gallery introduces no view-function
re-execution. If the expectation holds, the family note is updated
**revise-in-place** (no Vision DR), consistent with DD-V-026's
proportional-recording principle (heavy artifacts reserved for
thesis-reversal / family-pivot changes). This is **separate** from the
Problem B Vision DR (next), which concerns sizing-surface responsibility
allocation on the roadmap, not the architectural family.

### Problem B (explicit sizing) — a cross-milestone Vision DR, not a phase DD

Phase 8's framing stage **raises** the explicit-`width`/`height` sizing
question (Problem B) as a **cross-milestone Vision DR** under
[process/cross-milestone/decisions/](../../../cross-milestone/decisions/)
(same pattern as DD-V-022), because responsibility allocation touches the
roadmap SSOT
([author-controllable-sizing.md §7.2](../../../../docs/notes/author-controllable-sizing.md);
[constraints §2](../requirements/constraints.md)). Phase 8 owns **raising
it and fixing the issue**, not final disposition — that is a separate gate
that **does not block M3 close** (FD-8-D). DD-002's only Problem B
responsibility is **editorial**: the public draft must not present
Fill-default sizing as final, and must position explicit sizing as a
**future surface**. The `aspect`-in-cell arrange-abort facet folds into
the same triage (constraints §2), not a separate item.

### End-state shape this phase re-connects (verified at drafting time)

- **DSL surface** ([docs/dsl_spec.md](../../../../docs/dsl_spec.md)):
  per-phase-synced through Phase 7b; carries placement (§4.16), `if`
  conditional rendering, boolean/string bindings, the layout primitives.
  No selected-state surface yet — DD-001 adds it; A12 promotes the whole to
  public draft.
- **Architecture contract**
  ([docs/architecture.md](../../../../docs/architecture.md) §6.8.x):
  child-slot-carried placement (`SlotData`), the single-boolean binding
  model, Button's leaf measure/arrange. DD-001's toggle surface re-connects
  here (no new binding-target class, no new measure/arrange).
- **Examples / gallery** (`examples/gallery/gallery.ui` + per-phase
  verification sub-screens): A1 folds the per-phase screens into the one
  target app and re-derives layout-coupled capture coordinates
  ([constraints §6](../requirements/constraints.md)).
- **C ABI** ([docs/abi_spec.md](../../../../docs/abi_spec.md)): Phase 8 adds
  no code-construction surface; abi_spec is no-touch unless DD-002 finds an
  unavoidable future-compat note (judged at draft time).

## Decisions

The Phase 8 ADR carries the two framing-slate DDs (FD-8-B):

| DD | Title | Status | Decision summary |
|---|---|---|---|
| [DD-M3-P8-001](./dd-m3-p8-001-button-selected-state-surface.md) | Selected / toggle-state authoring surface | Proposed | Layer 1 (authoring form): **S2a — `ToggleButton { checked }`** recommended (controlled + one-way), **S1 — `Button { selected }`** the minimal alternative; S2b (SwiftUI two-way) / S2c (WPF/Qt widget-owned) deferred, S3 rejected, S4–S6 out (per-reason). Two-stage Accept (S1-vs-S2; then SI-4 lexeme). Layer 2 (driving boolean / exclusion): **α** (handwritten block assignment, live-proven) recommended, **β** (two-button single bool) the documented alternative; γ/δ deferred on the `==` axis. Recommendation **reverses** the framing/spike S1 lead — eyes-open, with §Accepted-time re-sync. |
| [DD-M3-P8-002](./dd-m3-p8-002-dsl-spec-public-draft-promotion.md) | DSL spec public-draft promotion: reservation & how unsettled surface is shown | Proposed (partial draft) | Decides (a) whether M4 syntax is reserved, (b) how provisional / unsettled future surface is positioned (PM-2 wrapper-rule, explicit sizing / Problem B, default-alignment asymmetry, placement spelling, placement bindability), (c) the status marker + CHANGELOG + external-reader pass line. Policy options pending; **§DD-001 coupling drafted concretely** (the four pre-Accept items DD-001's α leans on — see §Cross-DD dependency). |

Both DDs are at **Proposed**. DD-001 is the load-bearing surface decision
and is under multi-pass review; DD-002 is **partially drafted** — its
§DD-001 coupling is **concrete** (so DD-001's pre-Accept gate, §Cross-DD
dependency, is satisfied with an inspectable mitigation), while its broader
public-draft policy options (Main decisions A/B/C) are still being drafted.
Neither flips to Accepted in this preamble; per-DD detail is in each DD.

## Cross-DD decision dependency (one-directional + a pre-Accept gate)

Unlike Phase 7b's mutually-coupled DD pair, Phase 8's two DDs couple
**one-directionally at the documentation seam only**:

- **DD-002 does not decide DD-001's authoring form.** Whether selected
  state is `ToggleButton { checked }` or `Button { selected }`, and how the
  driving boolean is produced, is entirely DD-001's call.
- **DD-001's deferred surface is *positioned* by DD-002.** DD-001 defers
  four axes (equality-operator family; group-surface family; two-way
  binding; widget-owned state) and ships an M3-era exclusion pattern. How
  those reserved axes and that provisional pattern read as **public
  contract** in the first public draft is DD-002's editorial call.

Because that positioning is where the owner sees the reserved axes as
public contract, and because DD-001's α recommendation *leans on* it as the
teaching-risk mitigation, **DD-001 should not Accept before DD-002 carries
that mitigation in *concrete, inspectable form*** — not merely a skeleton
that names it (DD-001 §Couples-to). Concretely, DD-002 §DD-001 coupling
must hold, drafted, the four items DD-001's Layer-2 recommendation and
Out-of-scope depend on:

1. **Who authors the α provisional public-draft note** (the spec / gallery
   note that marks the O(N²) handwritten exclusion as the *M3-era* shape).
2. **How strong the gallery comment / spec note is** (so readers do not
   read O(N²) as the intended long-term idiom).
3. **The migration trigger to the future discriminant (`==`) form.**
4. **How the four deferred DD-001 axes are represented** in the public
   draft (reserved vs future-note, per DD-002's reservation policy).

This is the **pre-Accept gate**: DD-001's α recommendation *leans
on* this mitigation, so it cannot be a TODO at α's Accept. Per DD-001
§Couples-to the mitigation must **carry its concrete form so the owner can
confirm it is real before Accepting α** — therefore DD-002 drafts
§DD-001 coupling to **concrete recommended proposals** (note authorship,
note wording strength, the `==`-migration trigger, the deferred-axis
default = future-note), even though §Main decisions A/B/C stay skeletal.
They remain *recommendations* pending DD-002's own Accept — what α requires
is that the mitigation be **written and inspectable**, not that DD-002 be
fully Accepted first. (An earlier draft of this preamble said the items need
only "exist as enumerated open items"; that understated DD-001's gate and is
corrected here.)

## Scope and out of scope

The deferred-items **正本** (with activation triggers and responsibility
landings) is the framing scope table
([../requirements/framing.md §Phase 8 の対象範囲](../requirements/framing.md))
and the [constraints carry-forward table](../requirements/constraints.md);
this ADR does not duplicate them. In Phase 8 scope by decision: the
selected-state authoring surface and its parser / checker / lowering /
runtime / example implementation; gallery integration in three hosts; the
per-phase verification-screen cleanup and capture-coordinate re-derivation;
the `docs/dsl_spec.md` public-draft editorial pass and `docs/architecture.md`
re-connection; the Problem B Vision DR **raising** (not implementation); and
the milestone-close hand-off folding.

Out of Phase 8 scope (triggers held — constraints carry-forward table):
explicit-sizing **implementation** (Problem B Vision DR's assigned
milestone); PM-2 wrapper-rule **decision** (pre-1.0, via M3 handoff);
default-alignment unification **implementation** (future layout-behavior
phase); image primitives / asset pipeline / scrollbar widget / input /
modal / theme (M4 / M5); public code-construction API / ABI (M6 ABI prep);
generic modifiers / user-defined containers / non-layout parent-data /
keyed identity / layout-algorithm changes (later DSL / layout phases). The
lightbox **structural** proof (Button handler toggles a boolean, a subtree
shows/hides) is **in** M3; real hit-testing / modal focus / gesture polish
is M4 ([spec.md](../../requirements/spec.md) Interaction).

## Verification closure (what counts as Phase 8 evidence)

Per the framing verification strategy
([../requirements/framing.md §検証方針](../requirements/framing.md)) and
the positive-control discipline
([../../../../AGENTS.md §Testing rules](../../../../AGENTS.md) — a single
static frame a wrong implementation could equally produce is not evidence),
Phase 8 closes only when all of the following are observed (exact set
finalised against the chosen options at the Accepted flips):

1. **`wasamoc check` evidence (pure logic).** Positive: the chosen
   selected-state surface compiles and lowers; the boolean binding drives
   the attribute. Negative: the attribute is **rejected on widgets that do
   not support it** (a named check error with a firing test — DD-001 SI-3);
   existing examples regress-free.
2. **Lowering / IR / loader evidence.** The chosen surface lowers to the
   single-boolean binding model; the loader re-rejects malformed metadata;
   placement / binding defaults preserved across the gallery integration.
3. **`selected` / `checked` propagation audit (the central A10 evidence).**
   The surface crosses parser → check → lower → IR emit → runtime loader →
   widget visual → cross-host parity. Beyond the impl-gates call-site audit
   table, three points pinned by firing tests / positive controls:
   (i) the attribute on a non-supporting widget **rejects**; (ii) a
   bool-binding change **reaches the visual**; (iii) **C / Rust / Zig
   render the same** (cross-host parity).
4. **Layout-skeleton technical smoke (before owner UI review).** Before the
   full gallery is assembled, the wireframe skeleton (Grid frame / WrapPanel
   / ScrollView / ZStack / Box `aspect` folded into one screen) is rendered
   in a representative host and checked for 0-sizing / clip / aspect-abort /
   scroll breakage (Problem B's Fill→0 collapse surfaced early). What the
   existing surface can fix is fixed; what it cannot is triaged to the A1
   table placeholder / FD-8-G(1) owner agreement / Problem B Vision DR — no
   layout-engine change in Phase 8 (framing R7).
5. **Assistant-visible GUI evidence + positive control.** Gallery / lightbox
   / selected-state shown via launch + DPI-aware screenshot (`CopyFromScreen`)
   + assistant analysis; the **selected visual shown changing across two
   frames** (and, under Layer-2 α, the exclusion — one on, others off — in
   the same two frames). UI review is in one representative host; the other
   two are checked for identical render / no regression. Owner human-visible
   smoke is a separate gate.
6. **A12 spec-closure gate (non-test).** `docs/dsl_spec.md` carries the
   selected-state surface and the unsettled-surface positioning (DD-002) at
   the external-reader bar; the **external-reader smoke** passes (the spec
   alone reproduces M3 surface against a C-ABI virtual host); no
   silently-deferred M3 surface; status marker + CHANGELOG present; the
   Moment 1 → Moment 2 marker flip is complete.

## Implementation gate expectations

Per [../../../procedures/implementation-gates.md](../../../procedures/implementation-gates.md),
the selected-state task is expected to trip: **#1 semantic migration**
(the attribute crosses parser / check / lower / IR / loader / runtime
visual — call-site audit table), **#4 untested authored branch** (the new
attribute + its reject diagnostic on non-supporting widgets get direct
firing tests — DD-001 SI-3), **#5 carry-forward** (the four deferred axes,
Problem B, PM-2 wrapper-rule recorded in the M3 handoff), and **#7 GUI
positive control** (the two-frame selected/exclusion proof). The gallery
integration tasks additionally trip **#2 missed side effects** (capture
coordinates re-derived after layout changes) and the **layout-skeleton
smoke** (verification closure #4). The binding selection is recorded
per-task at task start.

## Upstream document revisions (Moment 1 / Moment 2)

Per-review-concern commit rule applies
([../../../../AGENTS.md §Commit rules](../../../../AGENTS.md)). Exact
touch / no-touch judgments depend on the chosen options and are finalised
at the Accepted flips; the anticipated set:

**Moment 1 — ADR Accepted commit set (design-spec draft):**

- This directory — DD Accepted flips (separable review concerns: DD-001
  surface, DD-002 public-draft policy, preamble — they may converge at
  different rates and need not land in one commit).
- [`docs/dsl_spec.md`](../../../../docs/dsl_spec.md) — **touch (expected).**
  The selected-state surface + its admission/rejection table (DD-001); the
  unsettled-surface positioning, status marker, and CHANGELOG (DD-002). No
  DD/option labels in spec prose (living-spec vocabulary rule); provenance
  via ADR hyperlink only.
- [`docs/architecture.md`](../../../../docs/architecture.md) — **touch
  (expected).** The toggle node / `checked` (or `selected`) attribute
  representation through lower / IR / runtime loader / widget visual,
  consistent with the single-boolean binding model.
- [`docs/abi_spec.md`](../../../../docs/abi_spec.md) — **no touch (judged);**
  revisited only if DD-002 finds an unavoidable future-compat note.
- [`docs/notes/architectural-family.md`](../../../../docs/notes/architectural-family.md)
  — alignment table + triggers updated with the Phase 8 confirm-within-family
  note (revise-in-place), at Moment 1 or Moment 2.
- The **Problem B Vision DR** lands as a **separate review unit** under
  [process/cross-milestone/decisions/](../../../cross-milestone/decisions/)
  (not part of the phase ADR commit set; cross-milestone governance, M3-close
  non-blocking).
- [`../../plan.md`](../../plan.md) — Phase 8 row populated; the new-AC
  disposition (none, or the DD-002 contingency) recorded in the Revision
  log at the relevant Accepted flip.
- [`../../../_roadmap.md`](../../../_roadmap.md) — **conditional touch.**
  DD-001 §Accepted-time re-sync touches A9 / A10 / A12 wording **if S2a is
  Accepted** (the `ToggleButton` / `checked` lexeme); DD-002's contingency
  touches the AC SSOT **only if a new public promise is added**.
- `implementation/plan.md` / `log.md` — opened after the Accepted flips.

**Moment 2 — Phase / milestone close commit set (impl re-sync):** dsl_spec /
architecture markers flip to `closed; implementation-synced` with
divergence corrections; the architectural-family confirm entry lands if
not already; the external-reader smoke result recorded; the plan rows flip
complete; the M3 `handoff.md` folds the deferred群 (PM-2 wrapper-rule,
Problem B final landing); phase-end + milestone-end retrospectives + CI
run-id ownership per the final-task / phase-end ownership split (the 7b
learning).

## Inputs absorbed

| Source | Disposition | Consumed at |
|---|---|---|
| FD-8-A — M3-closing thesis; gallery = Photo Gallery target app; no new primitive | Settled framing | §Context; §Scope |
| FD-8-B — two-DD slate (gallery integration not a DD) | Structure | §Decisions |
| FD-8-C — boolean-on-existing-binding; minimal visuals; exclusion on shipped surface; vehicle/surface left to spike+DD-001 | Constraint | §Settled floor; DD-001 |
| FD-8-D — Problem B = cross-milestone Vision DR, M3-close non-blocking | Constraint | §Problem B; §Scope |
| FD-8-E — implement-not-docs + cleanup scope + containment | Constraint | §Scope; §Verification closure |
| FD-8-F — new AC unlikely (contingent on DD-002) | Constraint | §Acceptance relation; plan Revision log |
| FD-8-G — five staged owner UI checkpoints | Discipline | §Verification closure; impl plan |
| Stage-1 feasibility spike — shipped-surface facts; α live-proven; option set rebuilt | Authority (cited, not re-derived) | §Pre-DD spike; DD-001 |
| constraints §1–§9 + carry-forward table | Carry-in / forward-maintained | §Settled floor; §Scope; §Problem B |
| phase-7b handoff — placement frozen; Problem B raise-timing; PM-2 / spelling / default-alignment carries; cleanup; ownership-split learning | Carry-in | §Context; §End-state; Moment 2 |
| plan.md — Phase 8 row; milestone-end criteria; M4-reservation affirmative-judgement discipline | Constraint | §Acceptance relation; §Scope |
| author-controllable-sizing.md §7.2 — Problem B evidence + raise timing | Family/sizing input | §Problem B |
| architectural-family.md — tree-with-bindings hypothesis; DSL spec = trigger 1 | Family input | §Architectural-family confirmation |
| gallery-wireframe.html / spec.md — Photo Gallery visual + interaction spec | Current-state input | §Verification closure; DD-001 |

## Revision history

| Date | Change |
|---|---|
| 2026-06-27 | Initial draft (Status: Proposed). DD-001 at Proposed under multi-pass review; DD-002 at skeleton stage. Framing owner-aligned 2026-06-25 ([../requirements/framing.md](../requirements/framing.md) §オーナー合意の記録). Records the one-directional DD-001→DD-002 documentation coupling and DD-001's pre-Accept gate (the four α-mitigation / deferred-axis items the DD-002 skeleton must enumerate). |
| 2026-06-28 | Accept-discipline review fold (Status: Proposed). Corrected §Cross-DD dependency: the pre-Accept gate requires DD-002's §DD-001 coupling to carry its **concrete form** (not merely "exist as open items"), matching DD-001 §Couples-to; DD-002 accordingly drafted those four items concretely. Status line + DD-002 Decisions row updated to "partial draft" (coupling concrete, policy options pending); also swept the stale "skeleton stage" prose under the Decisions table to the same "partial draft" wording. Follow-up sweep: the §Cross-DD dependency lead sentence ("DD-002 + preamble **skeleton** exists … the skeleton must enumerate") replaced with the concrete-form gate, so the gate is no longer double-read within the section. |
