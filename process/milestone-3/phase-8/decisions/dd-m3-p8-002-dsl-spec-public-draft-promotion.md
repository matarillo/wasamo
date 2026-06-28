---
title: DSL spec public-draft promotion — reservation & how unsettled surface is shown
status: Proposed
phase: M3-Phase 8
ac: A12 (existing) — the public-draft promotion is named by A12; discharges under A12 + A11. A **new public-promise AC** is added only if a reservation/promise in §Main decision A commits author-facing surface (framing FD-8-F); that contingency is decided at this DD's Accepted flip and recorded in the plan Revision log.
date: 2026-06-27
related:
  - ./preamble.md
  - ./dd-m3-p8-001-button-selected-state-surface.md
  - ../requirements/framing.md
  - ../requirements/constraints.md
---

# DD-M3-P8-002 — DSL spec public-draft promotion: reservation & how unsettled surface is shown

**Status:** Proposed

> **Partial-draft stage.** §Main decisions A/B/C carry **structure,
> questions, and direction** but their **option comparisons are not yet
> drafted** (marked *(to draft)*). **Exception — §DD-001 coupling is drafted
> to concrete recommended proposals**, because DD-001's α recommendation
> *leans on* that mitigation: per DD-001 §Couples-to the mitigation must
> "carry its concrete form so the owner can confirm the mitigation is real
> before Accepting α", so it cannot be a TODO at α's Accept. The four
> coupling items are therefore written and inspectable (recommendations
> pending this DD's own Accept), while the broader public-draft policy is
> still being drafted. This DD does **not** decide DD-001's authoring form;
> it is drafted in parallel with DD-001 per the Phase-8 plan.

## Context

`docs/dsl_spec.md` was updated **per-phase** across M3 (Phases 1–7b), so
promoting it to its **first public draft** (A12) is an **editorial** pass,
not greenfield writing
([constraints §1](../requirements/constraints.md);
[framing FD-8-A](../requirements/framing.md)). But the first public draft
is the **first public contract**: external readers and downstream tooling
(the M5 VS Code extension) start to depend on it, and changing "what is
written as settled" later becomes a breaking change. So the editorial pass
must make a set of **honest-positioning** decisions — what to reserve, and
how to write what is *not yet settled* so it does not read as settled.

This DD does **not** re-decide any frozen surface (placement is read from
the landed source — constraints §1). It decides three things:

- **(a) Reservation** — whether M4-facing syntax is reserved in the public
  draft, or recorded as declined.
- **(b) Positioning of unsettled / provisional future surface** — PM-2's
  two-form Grid wrapper rule, explicit sizing (Problem B), the
  default-alignment asymmetry, placement spelling, placement bindability.
- **(c) Publication mechanics** — the status marker, the M3 CHANGELOG, and
  the external-reader pass line.

Per the owner prior, each option is compared on **product merit first**;
documentation cost is not a con, and the over-positioning brake (do not
reserve so much that M4's design space shrinks) stays in force.

## Dependencies

- **Consumes** framing FD-8-A (editorial thesis), FD-8-B (this is the
  second of the two DDs), FD-8-D (Problem B is a cross-milestone Vision DR;
  this DD owns only its **editorial** positioning, not its disposition),
  FD-8-F (a new public promise here is the **only** path to a new AC), and
  the plan's M4-reservation affirmative-judgement discipline (silence
  defaults to "do not reserve", but exercising the reservation permission
  is an **active** act — [plan.md](../../plan.md)).
- **Carry-in constraints** ([constraints §2–§7](../requirements/constraints.md)):
  Problem B sizing (§2), PM-2 provisional wrapper-rule (§3),
  default-alignment asymmetry (§4), placement spelling (§5), placement
  bindability / backward-compat positioning (§7). Each is **Phase 8's
  editorial responsibility** to position honestly — surface re-litigation
  is **out** (constraints §1).
- **Couples to** [DD-M3-P8-001](./dd-m3-p8-001-button-selected-state-surface.md)
  **one-directionally**: this DD **positions** the selected-state surface
  DD-001 ships and the four axes DD-001 defers; it does **not** decide
  DD-001's authoring form. See §DD-001 coupling for the four items DD-001's
  Accept depends on.
- **References** the **Problem B Vision DR** (raised under
  [process/cross-milestone/decisions/](../../../cross-milestone/decisions/)
  per FD-8-D) — a separate review unit. This DD links to it for the sizing
  positioning; it does not contain its disposition.

## Main decision A — M4 syntax reservation

**Question:** does the first public draft **reserve** any M4-facing syntax,
or record that reservation was **declined**? (The plan requires an explicit
judgement; silence defaults to "do not reserve".)

**Direction (to draft):** keep reservation **minimal** — write unsettled
items as **future notes**, not as strong reservations. Over-reserving
shrinks M4's design space; under-reserving leaves compatibility
expectations vague; the minimal-reservation middle is the target.

- **Options (to draft):** (A-1) reserve nothing, record as declined;
  (A-2) minimal future-note only (recommended direction, not yet drafted);
  (A-3) explicit reservation of named M4 constructs.
- **AC contingency:** if a reservation here **commits author-facing
  surface** (a public promise), the new-AC exception fires (FD-8-F); if it
  is a non-committal future note, no new AC. Decided at this DD's Accepted
  flip; recorded in the plan Revision log either way.

## Main decision B — positioning unsettled / provisional future surface

**Question:** for each carry-in item that is **not yet settled**, how is it
written so the public draft does not read it as settled? Each is an
**editorial** positioning call (the surface itself is not re-litigated —
constraints §1).

| Sub-decision | Carry-in | What must be positioned | Direction (to draft) |
|---|---|---|---|
| B-1 — PM-2 two-form Grid | [constraints §3](../requirements/constraints.md) | State the accept-set (both `Cell` and direct `slot.*` are accepted) **while flagging the wrapper-rule as pre-1.0 undecided / provisional**. The wrapper-rule *decision* is **not** made here (pre-1.0, via M3 handoff). | Describe accept-set; mark wrapper-rule provisional |
| B-2 — explicit sizing (Problem B) | [constraints §2](../requirements/constraints.md); Problem B Vision DR | Do **not** present Fill-default sizing as final; position explicit `width`/`height` as a **future surface**, linked to the Problem B Vision DR. Fold the `aspect`-in-cell arrange-abort facet into the same note. | Future-surface note + Vision DR link |
| B-3 — default-alignment asymmetry | [constraints §4](../requirements/constraints.md) | Describe the current defaults (Grid `stretch` / ZStack `center`) **accurately**, and make an **explicit judgement** whether the asymmetry is explicable or explicability-debt. Explicable → close on documentation accuracy; debt → forward to a future layout-behavior phase. No unification implemented here. | Accurate description + explicit explicability judgement |
| B-4 — placement spelling | [constraints §5](../requirements/constraints.md) | Public-draft stabilization is the **last pre-publication revision chance** for `h-align` etc. Decide keep vs revise **affirmatively** (silence is not allowed). | Recommended: **keep** inherited spelling — affirmed, not silent |
| B-5 — placement bindability / compat | [constraints §7](../requirements/constraints.md) | Describe placement as **constant-per-instance** accurately (binding RHS rejected), and position the draft as a **first public draft, not a permanent compatibility commitment** (public compat is M6). | Accurate "constant" description + "not a stability commitment" framing |

## Main decision C — publication mechanics

**Question:** what status marker, change history, and pass line mark the
draft as public?

- **C-1 — status marker:** a `status: public-draft` (or equivalent) header
  marker on `docs/dsl_spec.md`. *(form to draft)*
- **C-2 — CHANGELOG:** an M3 change-history entry. *(scope to draft)*
- **C-3 — external-reader pass line:** can the spec **alone** (against a
  C-ABI virtual host) reproduce M3 surface? If not, the gap is editorial
  remaining work (milestone-end criterion 5). *(pass criterion to draft)*

## DD-001 coupling — the four pre-Accept gate items

DD-001's Layer-2 (α) recommendation carries a **public-example teaching
risk**: the O(N²) handwritten one-true-others-false exclusion pattern, if
shipped in a public gallery without positioning, risks teaching an
anti-pattern as the canonical way to express tab exclusion. DD-001's α
recommendation **leans on this note as the mitigation** ("α's teaching-risk
is mitigable by the provisional note rather than by dropping the
demonstration"), so the mitigation cannot be a TODO at α's Accept: per
DD-001 §Couples-to it must **carry its concrete form so the owner can
confirm the mitigation is real before Accepting α**. The items below are
therefore drafted to **concrete recommended proposals** (not `to draft`)
even while §Main decisions A/B/C remain skeletal — they are the part of
this DD α's Accept actually depends on. (They are *recommendations* pending
this DD's own Accept; what α requires is that the mitigation be **written
and inspectable**, which it now is.)

1. **Note authorship (文責) — recommended split.** The **spec note** (in
   `docs/dsl_spec.md`, by this DD at Moment 1) and the **gallery `.ui`
   comment** (in `examples/gallery/gallery.ui`, by the A1 integration task).
   The spec note is the **load-bearing** one (external readers depend on the
   spec, not the example source); the gallery comment is a local
   reinforcement so a reader copying the `.ui` sees the caveat in place. The
   spec note is the artifact the owner reviews at FD-8-G(4) (public-draft
   review).
2. **Gallery / spec note strength — recommended wording.** Strong enough to
   deny canonical status, bounded so it does not over-commit the future
   syntax. Recommended spec form (illustrative, final prose at Moment 1):
   *"Exactly-one-selected exclusion is expressed here by composing one
   boolean state per option and assigning them together in each handler.
   This is the **M3-era** way to express it; a future equality operator
   could allow a single-discriminant form instead (see below). Authors
   should not treat the per-option assignment as the language's intended
   long-term idiom."* — i.e. it (a) names it M3-era, (b) points forward
   without specifying syntax **and without promising the future operator
   ships or that this example is replaced**, (c) explicitly disclaims
   canonical status. The wording stays a *future note*, not a reservation,
   per Main decision A — "could allow", not "is expected to replace". The
   gallery comment is a one-line echo pointing at the spec section. **Not**
   stronger than this: it must not promise the `==` form's exact spelling
   (that is the `==` phase's call) — Main decision A's minimal-reservation
   policy applies.
3. **Discriminant (`==`) migration trigger — recommended.** The forward
   pointer names the future form `checked: tab == value` (O(N), single
   assignment, intrinsically exclusive) as a **candidate future replacement
   for the M3-era pattern** — *candidate*, not a committed replacement, since
   the `==` operator is not promised to ship — with the **revival trigger =
   an equality operator `==` entering the expression grammar** (DD-001
   Axis 1; the spec note points at, not reproduces, that axis). Recorded explicitly: whether `examples/gallery/`
   is **migrated** from α to the discriminant form when `==` lands is an
   **independent decision for the `==` phase**, *not* promised by this note —
   the note creates a forward pointer, not a migration commitment.
4. **Representation of DD-001's four deferred axes — recommended default =
   future-note, not reservation.** Per Main decision A's minimal-reservation
   policy, all four are written as **future notes** (not strong
   reservations): **Axis 1** equality-operator family (`==`); **Axis 2**
   group-surface family (`RadioGroup` / `TabBar` / `SegmentedControl`);
   **Axis 3** two-way binding (SwiftUI model); **Axis 4** widget-owned state
   (WPF/Qt model, family-level / Vision-DR-scale). The draft states honestly
   that **"exactly one selected" is author-composed, not a built-in group
   construct**, and that `checked` (or `selected`) is M3's minimal surface,
   not the only future selection model. Whether any axis is *promoted* from
   future-note to reservation is decided in §Main decision A at this DD's
   Accept; the **default carried for α's gate is future-note**.

If DD-001 instead Accepts **β** (static tab highlight, no live exclusion),
item 1–3's teaching-risk note is not needed, but item 4 (deferred-axis
representation) still applies. If DD-001 Accepts the **S1** authoring form
(`Button { selected }`), the surface named in the spec changes accordingly
(§Spec impact); the coupling structure is unchanged.

## Sub-issues *(to draft)*

- **SI-1 — granularity of the future-note convention.** A single
  "Provisional / future surface" convention used uniformly (B-1…B-5,
  reservation A, the DD-001 axes) vs per-item bespoke wording. *(to draft)*
- **SI-2 — abi_spec future-compat note.** Whether the public draft forces
  any `docs/abi_spec.md` touch (default: no-touch unless a future-compat
  note proves unavoidable — preamble §End-state). *(to draft)*

## Spec impact *(to draft)*

`docs/dsl_spec.md` (author-facing, external-reader bar, **no DD/option
labels** per the living-spec vocabulary rule; provenance via ADR hyperlink
only):

- The status marker (C-1), M3 CHANGELOG (C-2), and the unsettled-surface
  positioning (B-1…B-5) folded into the relevant chapters.
- The selected-state surface DD-001 ships (`ToggleButton { checked }` or
  `Button { selected }` per DD-001's Accept) positioned with the
  author-composed-exclusion note (§DD-001 coupling) and the deferred-axis
  representation.
- Stale prose swept in the same touch.

`docs/architecture.md`: no new model from this DD (DD-001 owns the toggle
node's representation); this DD touches architecture only if a positioning
decision needs the internal model described for accuracy. *(judged at
draft time)*

`docs/abi_spec.md`: no-touch unless SI-2 finds an unavoidable
future-compat note.

## Out of scope

- **Surface re-litigation of any frozen Phase 7b placement surface** — read
  from landed source (constraints §1).
- **The PM-2 wrapper-rule *decision*** — pre-1.0, via M3 handoff (this DD
  only flags it provisional — B-1).
- **Explicit-sizing *implementation* and the Problem B *disposition*** —
  the Vision DR's assigned milestone (this DD only positions it
  editorially — B-2).
- **Default-alignment *unification* implementation** — future
  layout-behavior phase (this DD only judges explicability — B-3).
- **DD-001's authoring-form decision** — DD-001 owns it; this DD positions
  only its deferred surface.
- **Backward-compatibility *guarantees*** — public compat is M6; the first
  public draft is not a stability commitment (B-5).

## Revision history

- 2026-06-27 — Initial **skeleton** (Status: Proposed). Structure,
  questions (Main decisions A/B/C), sub-issues, and the **DD-001 coupling
  gate** (four pre-Accept items: note authorship, note strength,
  discriminant-migration trigger, deferred-axis representation) fixed;
  per-decision option comparisons and recommendations *not yet drafted*.
  Exists to satisfy DD-001's pre-Accept gate (§Couples-to /
  [preamble §Cross-DD dependency](./preamble.md)); drafted in parallel with
  DD-001 per the Phase-8 plan.
- 2026-06-28 — Accept-discipline review fold (Status: Proposed). Resolved
  the **High** finding that DD-001's α mitigation was still a TODO: **§DD-001
  coupling items 1–4 are drafted to concrete recommended proposals** (note
  authorship = spec note by this DD + gallery comment by the A1 task;
  recommended spec wording naming the pattern "M3-era" with a non-committal
  forward pointer; the `==`-grammar revival trigger with migration left to
  the `==` phase; deferred-axis default = future-note). DD-001's α can now
  Accept against a **written, inspectable** mitigation rather than an open
  item. §Main decisions A/B/C option comparisons remain *(to draft)*.
- 2026-06-28 — Future-note strength fold (Status: Proposed). Softened the
  recommended `==`-note wording so it reads as a **future note, not a public
  reservation** (item 2: "a future equality operator **could allow** a
  single-discriminant form" rather than "**is expected to replace** it";
  item 3: the discriminant form is a **candidate** future replacement, with
  the `==` operator not promised to ship), aligning the recommended public
  prose with Main decision A's minimal-reservation policy. Recommendation
  unchanged.
