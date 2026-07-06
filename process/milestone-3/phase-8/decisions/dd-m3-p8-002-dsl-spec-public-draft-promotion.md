---
title: DSL spec public-draft promotion — reservation & how unsettled surface is shown
status: Accepted
phase: M3-Phase 8
ac: A12 (existing) — the public-draft promotion is named by A12; discharges under A12 + A11. A new public-promise AC is added only if a reservation/promise in Main decision A commits author-facing surface (framing FD-8-F); this draft recommends no such new AC.
date: 2026-07-01
related:
  - ./preamble.md
  - ./dd-m3-p8-001-button-selected-state-surface.md
  - ../requirements/framing.md
  - ../requirements/constraints.md
  - ../../../cross-milestone/decisions/author-controllable-sizing-surface.md
---

# DD-M3-P8-002 — DSL spec public-draft promotion: reservation & how unsettled surface is shown

**Status:** Accepted 2026-07-02

**Accepted disposition (owner decision, 2026-07-02):**

- **Main A — future-surface reservation policy:** A-2 — future notes, no
  syntax reservation. No new public-promise AC is created.
- **Main B — positioning unsettled / provisional surface:**
  - **B-1b** — both `Cell { ... }` and direct `slot.*` forms documented as
    valid M3 surface; the canonical wrapper rule stays a pre-1.0 handoff
    decision.
  - **B-2c** — future-note explicit sizing. The `docs/dsl_spec.md` wording
    stops at "pre-1.0 unresolved; exact syntax / IR / ABI shape not reserved";
    the accepted Problem B VDR's M4/M5 spike schedule stays out of the public
    draft (cited only internally in this DD).
  - **B-3b** — Grid/ZStack defaults documented as container-owned semantics
    and judged explicable. A reader-smoke downgrade to B-3c is a separate
    procedural step, not an outcome left open at this Accept.
  - **B-4a** — inherited kebab-case placement spellings kept, affirmed (not
    silent carry).
  - **B-5b** — placement is constant-per-instance with binding RHS rejected;
    the public draft is not an M6 compatibility guarantee.
  - **B-6b** — the chosen M3 toggle surface is normative; the five DD-001 axes
    are future notes (not accepted M3 syntax, not reserved as M4 syntax).
- **Main C — publication mechanics:** C-2 — status marker + M3 change history
  + external-reader smoke.
- **DD-001 coupling:** α items 1-3 active; item 4 active.
- **Plan Revision-log outcome:** no new AC (A-2/C-2 accepted without syntax
  reservation).

## Context

`docs/dsl_spec.md` was updated per-phase across M3 (Phases 1-7b), so
promoting it to the **first public draft** (A12) is an editorial pass, not
greenfield writing. But it is also the first version external readers and
downstream tools can reasonably treat as a public reference. This DD
therefore decides how the public draft distinguishes:

- surface M3 actually ships and documents as reproducible;
- surface M3 accepts but keeps **provisional** before 1.0;
- future surface that is known, named, and deliberately **not yet designed**;
- future surface that is explicitly **not reserved**, so later milestones
  remain free to design it.

The main design risk is **false certainty**. If the public draft writes an
unsettled item as normative, later correction becomes a breaking public-doc
change. The opposite risk is **false emptiness**: if the draft merely omits
known unresolved surface, external readers will infer that the current
behavior is final. This DD adopts a middle vocabulary that is visible enough
to be honest, but weak enough not to pre-design M4/M5/M6.

This DD does **not** re-decide the frozen Phase 7b placement surface. It
reads the landed `slot.*` / Grid `Cell` + direct `slot.*` model from
`docs/dsl_spec.md` and `docs/architecture.md` (constraints §1). It also did
not decide DD-001's control surface: DD-001 has accepted T1
`ToggleButton` / `checked`. This DD only decides how that accepted surface
and the deferred axes are written in the public draft.

Per the owner prior, options are compared on **design fit and public-contract
honesty first**. Documentation cost is not counted as a con; implementation
size is only a tie-breaker.

## Dependencies

- **Consumes** framing FD-8-A/B/D/F/G and constraints §2-§8.
- **Couples to** [DD-M3-P8-001](./dd-m3-p8-001-button-selected-state-surface.md)
  one-directionally: DD-001 has chosen `ToggleButton { checked: <bool> }`,
  W1, and α; this DD positions that accepted surface and the deferred axes in
  the public draft.
- **References** the Problem B Vision DR
  ([author-controllable-sizing-surface.md](../../../cross-milestone/decisions/author-controllable-sizing-surface.md)).
  That VDR owns the roadmap-level responsibility question for explicit
  sizing; this DD owns only the public-draft wording.

## Main decision A — Future-surface reservation policy

**Question:** should the first public draft reserve M4/M5/M6-facing syntax,
or should it use non-committal future notes and record reservation as
declined?

### Options

1. **A-1 — reserve nothing; omit future surfaces except in handoff.**
   - What you gain: the public draft stays small and purely normative.
   - What you give up: known unresolved items become invisible to external
     readers. In particular, PM-2, explicit sizing, and DD-001's exclusion
     axes would look accidentally final because the draft would not say
     otherwise.
   - Assessment: too weak for A12. It protects future design freedom, but by
     hiding known uncertainty.
2. **A-2 — future notes, no syntax reservation.** The draft names known
   future surfaces and their triggers, but uses explicitly non-committal
   language (`future surface`, `candidate`, `not reserved`, `not a stability
   commitment`) and avoids promising exact syntax unless a surface is already
   accepted.
   - What you gain: honest public documentation without narrowing M4/M5/M6's
     design space. External readers see that the current M3 idioms are real
     but not necessarily the long-term idioms.
   - What you give up: the draft is slightly more editorially complex, and
     future readers must distinguish normative sections from future notes.
   - Assessment: best fit for Phase 8. It satisfies the plan's affirmative
     judgement requirement while avoiding premature grammar / runtime /
     reactive-architecture commitments.
3. **A-3 — reserve named future syntax now.** Reserve exact spellings such
   as `width`, `height`, `tab == value`, group widgets, material/backdrop
   syntax, or two-way binding notation.
   - What you gain: downstream tooling can avoid claiming those names for
     other purposes, and external readers see likely direction.
   - What you give up: it converts unsettled architecture into public
     promises before the owning milestone has compared grammar, IR, runtime,
     and host-API consequences. It would especially over-constrain M4 input /
     focus and M6 ABI design.
   - Assessment: too strong for M3 close. Use only if a specific future
     surface has already been accepted elsewhere; none has.

### Recommendation

Adopt **A-2 — future notes, no syntax reservation**.

This is not chosen because it is the smallest implementation. It is chosen
because it matches the public-draft thesis: M3 publishes what exists, and it
is honest about known unsettled surface without drafting later milestones in
advance. It also avoids creating a new public-promise AC: future notes are
not acceptance commitments. The plan Revision log should record **no new AC**
unless this DD is changed at Accept time to reserve exact author-facing
syntax.

## Main decision B — Positioning unsettled / provisional surface

**Question:** for each carry-in item, what does the public draft say so that
readers neither over-trust the current shape nor assume the project forgot
the issue?

### B-1 — PM-2 two-form Grid wrapper rule

Options:

- **B-1a — present both forms as final peers.** Accurate for M3's accept-set,
  but too strong: it hides the pre-1.0 wrapper-rule decision.
- **B-1b — document both accepted forms and mark the wrapper rule
  provisional.** State that both `Cell { ... }` and direct `slot.*` are valid
  in M3, while the canonical wrapper rule remains a pre-1.0 decision carried
  through M3 handoff.
- **B-1c — pick PM-1 or PM-3 now.** Would settle the future rule before a
  concrete new wrapper pressure or public code-construction API exists.

**Recommendation:** **B-1b.** It preserves the accepted M3 surface and tells
the truth about the unresolved 1.0 rule. It does not over-state either form
as canonical.

### B-2 — explicit sizing (Problem B)

Options:

- **B-2a — describe current kind-default sizing as final.** Simple, but
  false: Phase 2 / 4 / 7b already exposed explicit sizing as a known future
  surface.
- **B-2b — reserve exact `width` / `height` syntax.** Clear, but premature:
  the now-accepted Vision DR (2026-07-02) deliberately leaves the surface
  shape (grammar-only, modifier-like, layout-parent data, runtime state,
  host-construction API, or some combination) to a scheduled M4/M5 spike, so
  no spelling is settled to reserve.
- **B-2c — future-note explicit sizing, linked to the Vision DR.** In
  `docs/dsl_spec.md`, state that M3 sizing is kind-default and that
  author-controllable sizing is a known **pre-1.0 unresolved future surface**
  whose exact syntax / IR / ABI shape is **not reserved**. The public-draft
  wording stops there: it carries the future-work framing **only** and does
  **not** publish the VDR's M4/M5 spike schedule, which the accepted Vision DR
  records as a process / roadmap commitment, not an external promise
  ([author-controllable-sizing-surface.md](../../../cross-milestone/decisions/author-controllable-sizing-surface.md)
  §Recommendation, §Consequent edits). DD-002 may cite that scheduled spike
  internally as the reason the shape is left open; the schedule itself stays
  out of the DSL public draft.

**Recommendation:** **B-2c.** The public draft should not call Fill/Shrink
defaults final, but it should also not reserve a spelling or implementation
architecture. DD-002's internal rationale for leaving the shape open is that
the accepted Vision DR defers the surface shape to a scheduled M4/M5 spike; the
public-draft note conveys only "pre-1.0 unresolved; shape not reserved" and
keeps that schedule internal. The `aspect`-in-cell arrange abort is folded into
the same note, not split into a second future feature.

### B-3 — default-alignment asymmetry

Options:

- **B-3a — unify defaults now.** Changes behavior inside the public-draft
  phase and reopens Phase 7b's frozen placement model.
- **B-3b — document current defaults as container-owned semantics and judge
  them explicable.** Grid defaults to `stretch` because grid cells allocate
  tracks; ZStack defaults to `center` because overlay composition has no
  track fill contract.
- **B-3c — document current defaults but mark them as explicability debt.**
  Keeps M3 behavior while explicitly sending unification to a future
  layout-behavior phase.

**Recommendation:** **B-3b.** At Accept this selects B-3b definitively: the
spec presents defaults as **container-owned semantics**, not as a global
alignment rule (Grid `stretch` because grid cells allocate tracks; ZStack
`center` because overlay composition has no track-fill contract), and judges
the asymmetry explicable. Accept does **not** leave a B-3b/B-3c toss-up open.
If the external-reader smoke *later* shows the defaults still read as
arbitrary, that is handled by a **separate procedural step** — revise the
Accepted disposition to B-3c and carry a future layout-behavior residual — not
by treating the Accept outcome as undecided. Do not implement B-3a in Phase 8.

### B-4 — placement spelling

Options:

- **B-4a — keep inherited kebab-case spellings** (`h-align`, `v-align`,
  `row-span`, `column-span`), explicitly affirmed.
- **B-4b — revise to camelCase / alternative names** before the public draft.
- **B-4c — leave spelling unexamined.**

**Recommendation:** **B-4a.** This is a positive decision, not silent carry.
The current spelling is internally consistent across placement keys and
already appears in the landed spec. Revising now would spend the last
pre-publication chance on naming churn without evidence that the existing
names block comprehension. B-4c is rejected because public-draft
stabilization is the named trigger for an affirmative keep/revise judgement.

### B-5 — placement bindability / compatibility positioning

Options:

- **B-5a — present placement as permanently constant.**
- **B-5b — describe current placement as constant-per-instance, with binding
  RHS rejected, and state that the first public draft is not a permanent
  compatibility guarantee.**
- **B-5c — reserve bindable placement now.**

**Recommendation:** **B-5b.** It is the only option that is both accurate and
non-foreclosing. Public compatibility commitments are an M6 concern, not an
M3 public-draft side effect.

### B-6 — DD-001 selected/toggle deferred axes

Options:

- **B-6a — write only the chosen M3 toggle surface.**
- **B-6b — write the chosen M3 surface plus future notes for the five
  non-foreclosed DD-001 axes.**
- **B-6c — reserve the five axes as expected future syntax.**

**Recommendation:** **B-6b.** The chosen M3 surface must be normative. The
deferred axes must be visible because they are real design alternatives, but
they should remain future notes: equality/discriminant selection, group
surface, two-way binding, widget-owned state, and generic Toggle appearance
are not accepted M3 syntax and not reserved as M4 syntax.

## Main decision C — Publication mechanics

**Question:** what concrete artifacts make the draft publicly reviewable?

### Options

1. **C-1 — frontmatter/status marker only.**
   - Too weak: a marker says "public draft" but does not show what changed or
     whether an external reader can reproduce M3.
2. **C-2 — status marker + M3 change history + external-reader smoke.**
   - Balanced: the marker gives state, the change history gives provenance,
     and the smoke check tests the spec as a reader-facing artifact.
3. **C-3 — full compatibility policy and versioning model.**
   - Too strong: public compatibility is M6, and adopting it here would turn
     the first draft into a stability contract.

### Recommendation

Adopt **C-2**:

- `docs/dsl_spec.md` gets `status: public-draft` (or equivalent existing
  frontmatter style).
- A concise M3 change-history / CHANGELOG entry links the M3 ADRs and the
  public-draft anchor.
- The external-reader smoke asks whether a reader with only `docs/dsl_spec.md`
  could reproduce the M3 surface against a hypothetical host that already
  provides the C ABI. A "not yet" answer is remaining editorial work, not a
  test skip.
- The draft explicitly says the public draft is **not yet** the M6
  backward-compatibility commitment.

## DD-001 coupling — concrete items active under accepted α

DD-001 accepted Layer-3 α, which carries a public-example teaching risk:
the O(N^2) handwritten one-true-others-false exclusion pattern could be
mistaken for the intended long-term idiom. DD-001's pre-Accept gate is
satisfied because this DD carries the mitigation in concrete, inspectable
form; DD-002 is now **Accepted** (2026-07-02), so those items are settled
disposition (α items 1-4 active).

Recommended coupling:

1. **Note authorship.** The load-bearing note lives in `docs/dsl_spec.md`
   under this DD's Moment 1 spec work. `examples/gallery/gallery.ui` also
   gets a local comment when A1 integrates the gallery.
2. **Note strength.** The spec note names the pattern as the **M3-era**
   author-composed exclusion idiom, not a canonical long-term language
   design. Illustrative wording: "Exactly-one-selected exclusion is expressed
   in M3 by composing one boolean state per option and assigning them
   together in each handler. This is an M3-era pattern; a future equality
   operator could allow a single-discriminant form. Do not treat the
   per-option assignment pattern as a long-term reservation."
3. **Migration trigger.** The discriminant form revives when an equality
   operator enters the expression grammar. Whether `examples/gallery/` is
   migrated at that time is a future decision, not promised here.
4. **Deferred-axis representation.** The five DD-001 axes are written as
   **future notes, not reservations**, unless this DD is changed at Accept
   time: equality/discriminant, group surface, two-way binding,
   widget-owned state, generic Toggle appearance/control-family.

Because DD-001 accepted α, items 1-3 are active. Item 4 applies regardless.

## Spec impact

`docs/dsl_spec.md`:

- Add the public-draft status marker, M3 change history, and a short
  "public draft vs compatibility commitment" note.
- Fold in the DD-001-selected toggle surface using the accepted lexeme:
  `ToggleButton { checked: <bool> }`, without DD option labels in living spec
  prose.
- Add future-note wording for PM-2, explicit sizing / Problem B,
  placement defaults, placement spelling, placement bindability, and DD-001's
  deferred axes.
- Keep normative M3 syntax separate from future notes.

`docs/architecture.md`:

- Touched only where the public-draft wording needs an architecture pointer
  for accuracy. DD-001 owns the toggle representation.

`docs/abi_spec.md`:

- No touch expected. Revisit only if the external-reader smoke shows the
  public draft cannot explain the M3 surface without an ABI note.

`process/milestone-3/plan.md`:

- Record the FD-8-F disposition in the Revision log: **no new AC** if this
  DD Accepts A-2/C-2 without syntax reservation; otherwise record the
  public-promise exception.

## Out of scope

- Re-deciding Phase 7b placement surface.
- PM-2 wrapper-rule final decision.
- Explicit-sizing implementation or the Problem B roadmap disposition.
- Default-alignment behavior unification.
- Re-deciding DD-001's accepted `ToggleButton` / `checked` control surface.
- M4 input/focus, M5 theme/tooling, and M6 compatibility policy.

## Accepted-time disposition checklist

At Accept, record:

- A policy: A-2 recommended; if changed to A-3, name the exact public promise
  and the AC/roadmap update.
- B sub-decisions: B-1b, B-2c (public-draft note stops at "pre-1.0 unresolved;
  shape not reserved" — the M4/M5 schedule stays out of `docs/dsl_spec.md`),
  B-3b (revision to B-3c only via the separate reader-smoke procedure, not left
  open at Accept), B-4a, B-5b, B-6b.
- C mechanics: C-2 recommended.
- DD-001 coupling: α items 1-3 are active; item 4 is active.
- Plan Revision-log outcome: no-new-AC vs public-promise exception.

## Revision history

| Date | Change |
|---|---|
| 2026-06-27 | Initial skeleton (Status: Proposed). |
| 2026-06-28 | Drafted DD-001 coupling to concrete recommended proposals so DD-001's α mitigation is inspectable before Accept. |
| 2026-06-28 | Synced to DD-001's control-taxonomy restructure (B2/T1 co-equal, five deferred axes). |
| 2026-07-01 | Completed Main decisions A/B/C as a full Proposed draft. Recommendation: future notes with no syntax reservation; honest positioning of PM-2 / Problem B / defaults / spelling / bindability / DD-001 axes; publication mechanics = status marker + M3 change history + external-reader smoke. |
| 2026-07-01 | Synced to DD-001 owner acceptance: DD-001 now fixes T1 `ToggleButton` / `checked`, W1, and α. Coupling section now treats α items 1-3 as active and spec impact names the concrete `ToggleButton { checked }` surface. DD-002 status remains Proposed. |
| 2026-07-02 | Reflected the Problem B Vision DR Accept in B-2b / B-2c: the surface shape is now described as deliberately deferred to the VDR's scheduled M4/M5 spike (not "the VDR has not yet decided" / "before the VDR is accepted"). Recommendation (B-2c: future-note, no reservation) unchanged. DD-002 status remains Proposed. |
| 2026-07-02 | Codex review folds. (1) B-2c: bounded the DSL public-draft wording to "pre-1.0 unresolved; shape not reserved" and made explicit that the VDR's M4/M5 schedule stays out of `docs/dsl_spec.md` (schedule is a process/roadmap commitment, cited only internally) — aligns with the accepted VDR. (2) B-3: Accept now selects B-3b definitively; reader-smoke downgrade to B-3c is a separate procedural step, not an outcome left open at Accept (checklist synced). (3) B-6b: "not accepted M3 or M4 syntax" → "not accepted M3 syntax and not reserved as M4 syntax" to match DD-001's non-foreclosed axes. DD-002 status remains Proposed. |
| 2026-07-02 | **Accepted (owner decision).** A-2 / B-1b / B-2c / B-3b / B-4a / B-5b / B-6b / C-2; DD-001 coupling α items 1-4 active; plan Revision-log outcome = no new AC. Accepted disposition recorded at the top of the DD. Moment 1/2 spec sync and gallery integration follow. |
